//! # 记账功能核心业务逻辑
//!
//! 提供完整的记账功能业务逻辑实现：
//! - 账户管理
//! - 交易处理
//! - 预算控制
//! - 财务分析

use crate::errors::{AppError, Result};
use crate::storage::{
    Account, AccountBalance, AccountInsert, AccountType, AccountUpdate, Budget, BudgetInsert,
    BudgetPeriod, BudgetUpdate, CategoryBreakdown, FinancialReport, FinancialStats, MonthlyTrend,
    Transaction, TransactionCategory, TransactionCategoryInsert, TransactionCategoryUpdate,
    TransactionInsert, TransactionQuery, TransactionStatus, TransactionType, TransactionUpdate,
};
use chrono::{DateTime, Datelike, Local, NaiveDate};
use std::collections::HashMap;
use uuid::Uuid;

/// 记账系统核心管理器
#[derive(Debug)]
pub struct AccountingManager {
    /// 当前默认货币
    default_currency: String,
    /// 账户余额缓存
    account_balances: HashMap<Uuid, f64>,
    /// 预算使用情况缓存
    budget_usage: HashMap<Uuid, f64>,
}

impl AccountingManager {
    /// 创建新的记账管理器
    pub fn new() -> Self {
        Self {
            default_currency: "CNY".to_string(),
            account_balances: HashMap::new(),
            budget_usage: HashMap::new(),
        }
    }

    /// 设置默认货币
    pub fn set_default_currency(&mut self, currency: String) {
        self.default_currency = currency;
    }

    /// 获取默认货币
    pub fn get_default_currency(&self) -> &str {
        &self.default_currency
    }

    // ==================== 账户管理 ====================

    /// 创建新账户
    pub fn create_account(
        &mut self,
        name: String,
        account_type: AccountType,
        initial_balance: f64,
        currency: Option<String>,
        description: Option<String>,
    ) -> Result<Account> {
        let currency = currency.unwrap_or_else(|| self.default_currency.clone());

        let mut account = Account::new(name, account_type, currency, initial_balance);

        if let Some(desc) = description {
            account.description = Some(desc);
        }

        // 更新余额缓存
        self.account_balances.insert(account.id, initial_balance);

        log::info!("创建账户: {} ({})", account.name, account.id);
        Ok(account)
    }

    /// 更新账户余额
    pub fn update_account_balance(&mut self, account_id: Uuid, amount: f64) -> Result<()> {
        // 更新缓存
        *self.account_balances.entry(account_id).or_insert(0.0) += amount;

        log::debug!("更新账户余额: {} += {}", account_id, amount);
        Ok(())
    }

    /// 获取账户余额
    pub fn get_account_balance(&self, account_id: Uuid) -> f64 {
        self.account_balances
            .get(&account_id)
            .copied()
            .unwrap_or(0.0)
    }

    /// 设置默认账户
    pub fn set_default_account(&mut self, account_id: Uuid) -> Result<()> {
        // 这里需要与数据库交互来更新默认账户状态
        log::info!("设置默认账户: {}", account_id);
        Ok(())
    }

    // ==================== 交易处理 ====================

    /// 创建新交易
    pub fn create_transaction(
        &mut self,
        transaction_type: TransactionType,
        amount: f64,
        description: String,
        account_id: Uuid,
        category_id: Option<Uuid>,
        to_account_id: Option<Uuid>,
        transaction_date: Option<NaiveDate>,
        tags: Option<Vec<String>>,
    ) -> Result<Transaction> {
        if amount <= 0.0 {
            return Err(AppError::Validation("交易金额必须大于0".to_string()));
        }

        let transaction_date = transaction_date.unwrap_or_else(|| Local::now().date_naive());
        let currency = self.default_currency.clone();

        let mut transaction = Transaction::new(
            transaction_type,
            amount,
            currency,
            description,
            account_id,
            transaction_date,
        );

        if let Some(cat_id) = category_id {
            transaction.set_category(cat_id);
        }

        if let Some(to_id) = to_account_id {
            transaction.to_account_id = Some(to_id);
        }

        if let Some(tag_list) = tags {
            transaction.tags = tag_list;
        }

        // 根据交易类型更新账户余额
        match transaction_type {
            TransactionType::Income => {
                self.update_account_balance(account_id, amount)?;
            }
            TransactionType::Expense => {
                self.update_account_balance(account_id, -amount)?;
            }
            TransactionType::Transfer => {
                if let Some(to_id) = to_account_id {
                    self.update_account_balance(account_id, -amount)?;
                    self.update_account_balance(to_id, amount)?;
                } else {
                    return Err(AppError::Validation("转账必须指定目标账户".to_string()));
                }
            }
        }

        log::info!(
            "创建交易: {} {} {}",
            transaction_type,
            amount,
            transaction.id
        );
        Ok(transaction)
    }

    /// 更新交易
    pub fn update_transaction(
        &mut self,
        transaction_id: Uuid,
        original_transaction: &Transaction,
        update: TransactionUpdate,
    ) -> Result<()> {
        // 首先撤销原交易对余额的影响
        self.revert_transaction_balance_effect(original_transaction)?;

        // 然后应用新的余额变化
        let new_amount = update.amount.unwrap_or(original_transaction.amount);
        let new_type = update
            .transaction_type
            .unwrap_or(original_transaction.transaction_type);
        let new_account = update.account_id.unwrap_or(original_transaction.account_id);
        let new_to_account = update
            .to_account_id
            .unwrap_or(original_transaction.to_account_id);

        match new_type {
            TransactionType::Income => {
                self.update_account_balance(new_account, new_amount)?;
            }
            TransactionType::Expense => {
                self.update_account_balance(new_account, -new_amount)?;
            }
            TransactionType::Transfer => {
                if let Some(to_id) = new_to_account {
                    self.update_account_balance(new_account, -new_amount)?;
                    self.update_account_balance(to_id, new_amount)?;
                }
            }
        }

        log::info!("更新交易: {}", transaction_id);
        Ok(())
    }

    /// 删除交易
    pub fn delete_transaction(&mut self, transaction: &Transaction) -> Result<()> {
        // 撤销交易对余额的影响
        self.revert_transaction_balance_effect(transaction)?;

        log::info!("删除交易: {}", transaction.id);
        Ok(())
    }

    /// 撤销交易对余额的影响
    fn revert_transaction_balance_effect(&mut self, transaction: &Transaction) -> Result<()> {
        match transaction.transaction_type {
            TransactionType::Income => {
                self.update_account_balance(transaction.account_id, -transaction.amount)?;
            }
            TransactionType::Expense => {
                self.update_account_balance(transaction.account_id, transaction.amount)?;
            }
            TransactionType::Transfer => {
                if let Some(to_id) = transaction.to_account_id {
                    self.update_account_balance(transaction.account_id, transaction.amount)?;
                    self.update_account_balance(to_id, -transaction.amount)?;
                }
            }
        }
        Ok(())
    }

    /// 搜索和过滤交易
    pub fn filter_transactions(
        &self,
        transactions: &[Transaction],
        query: &TransactionQuery,
    ) -> Vec<Transaction> {
        transactions
            .iter()
            .filter(|t| {
                // 账户过滤
                if let Some(account_id) = &query.account_id {
                    if t.account_id != *account_id {
                        return false;
                    }
                }

                // 分类过滤
                if let Some(category_id) = &query.category_id {
                    if t.category_id != Some(*category_id) {
                        return false;
                    }
                }

                // 交易类型过滤
                if let Some(transaction_type) = &query.transaction_type {
                    if t.transaction_type != *transaction_type {
                        return false;
                    }
                }

                // 状态过滤
                if let Some(status) = &query.status {
                    if t.status != *status {
                        return false;
                    }
                }

                // 日期范围过滤
                if let Some(start_date) = &query.start_date {
                    if t.transaction_date < *start_date {
                        return false;
                    }
                }

                if let Some(end_date) = &query.end_date {
                    if t.transaction_date > *end_date {
                        return false;
                    }
                }

                // 金额范围过滤
                if let Some(min_amount) = &query.min_amount {
                    if t.amount < *min_amount {
                        return false;
                    }
                }

                if let Some(max_amount) = &query.max_amount {
                    if t.amount > *max_amount {
                        return false;
                    }
                }

                // 标签过滤
                if let Some(tags) = &query.tags {
                    if !tags.iter().any(|tag| t.tags.contains(tag)) {
                        return false;
                    }
                }

                // 搜索文本过滤
                if let Some(search) = &query.search {
                    let search_lower = search.to_lowercase();
                    if !t.description.to_lowercase().contains(&search_lower) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    // ==================== 预算管理 ====================

    /// 创建新预算
    pub fn create_budget(
        &mut self,
        name: String,
        category_id: Uuid,
        amount: f64,
        period: BudgetPeriod,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
        currency: Option<String>,
    ) -> Result<Budget> {
        if amount <= 0.0 {
            return Err(AppError::Validation("预算金额必须大于0".to_string()));
        }

        let currency = currency.unwrap_or_else(|| self.default_currency.clone());
        let mut budget = Budget::new(name, category_id, amount, currency, period, start_date);

        if let Some(end) = end_date {
            budget.end_date = Some(end);
        }

        // 初始化预算使用情况
        self.budget_usage.insert(budget.id, 0.0);

        log::info!("创建预算: {} ({})", budget.name, budget.id);
        Ok(budget)
    }

    /// 更新预算使用情况
    pub fn update_budget_usage(&mut self, budget_id: Uuid, spent_amount: f64) -> Result<()> {
        self.budget_usage.insert(budget_id, spent_amount);
        log::debug!("更新预算使用: {} = {}", budget_id, spent_amount);
        Ok(())
    }

    /// 检查预算状态
    pub fn check_budget_status(&self, budget: &Budget) -> BudgetStatus {
        let spent = self.budget_usage.get(&budget.id).copied().unwrap_or(0.0);
        let usage_percentage = if budget.amount > 0.0 {
            spent / budget.amount * 100.0
        } else {
            0.0
        };

        if spent > budget.amount {
            BudgetStatus::OverBudget {
                spent,
                over_amount: spent - budget.amount,
            }
        } else if usage_percentage >= 90.0 {
            BudgetStatus::NearLimit {
                spent,
                usage_percentage,
            }
        } else if usage_percentage >= 75.0 {
            BudgetStatus::Warning {
                spent,
                usage_percentage,
            }
        } else {
            BudgetStatus::OnTrack {
                spent,
                usage_percentage,
            }
        }
    }

    /// 获取预算警告
    pub fn get_budget_warnings(&self, budgets: &[Budget]) -> Vec<BudgetWarning> {
        budgets
            .iter()
            .filter_map(|budget| match self.check_budget_status(budget) {
                BudgetStatus::OverBudget { over_amount, .. } => Some(BudgetWarning {
                    budget_id: budget.id,
                    budget_name: budget.name.clone(),
                    message: format!("预算已超支 {:.2} {}", over_amount, budget.currency),
                    severity: WarningSeverity::Critical,
                }),
                BudgetStatus::NearLimit {
                    usage_percentage, ..
                } => Some(BudgetWarning {
                    budget_id: budget.id,
                    budget_name: budget.name.clone(),
                    message: format!("预算即将用尽 ({:.1}%)", usage_percentage),
                    severity: WarningSeverity::High,
                }),
                BudgetStatus::Warning {
                    usage_percentage, ..
                } => Some(BudgetWarning {
                    budget_id: budget.id,
                    budget_name: budget.name.clone(),
                    message: format!("预算使用较多 ({:.1}%)", usage_percentage),
                    severity: WarningSeverity::Medium,
                }),
                _ => None,
            })
            .collect()
    }

    // ==================== 分类管理 ====================

    /// 创建交易分类
    pub fn create_transaction_category(
        &self,
        name: String,
        transaction_type: TransactionType,
        color: String,
        icon: Option<String>,
        parent_id: Option<Uuid>,
        description: Option<String>,
    ) -> Result<TransactionCategory> {
        let mut category = TransactionCategory::new(name, transaction_type, color);

        if let Some(desc) = description {
            category.description = Some(desc);
        }

        if let Some(icon_name) = icon {
            category.icon = Some(icon_name);
        }

        if let Some(parent) = parent_id {
            category.parent_id = Some(parent);
        }

        log::info!("创建交易分类: {} ({})", category.name, category.id);
        Ok(category)
    }

    /// 验证分类层级
    pub fn validate_category_hierarchy(
        &self,
        category_id: Uuid,
        parent_id: Uuid,
        categories: &[TransactionCategory],
    ) -> Result<()> {
        // 防止循环引用
        if category_id == parent_id {
            return Err(AppError::Validation("分类不能以自己为父分类".to_string()));
        }

        // 检查是否会形成循环
        let mut current_parent = Some(parent_id);
        while let Some(parent) = current_parent {
            if parent == category_id {
                return Err(AppError::Validation("分类层级不能形成循环".to_string()));
            }

            current_parent = categories
                .iter()
                .find(|c| c.id == parent)
                .and_then(|c| c.parent_id);
        }

        Ok(())
    }

    // ==================== 财务分析 ====================

    /// 生成财务统计
    pub fn generate_financial_stats(
        &self,
        transactions: &[Transaction],
        accounts: &[Account],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<FinancialStats> {
        let filtered_transactions: Vec<_> = transactions
            .iter()
            .filter(|t| t.transaction_date >= start_date && t.transaction_date <= end_date)
            .collect();

        let total_income = filtered_transactions
            .iter()
            .filter(|t| t.transaction_type == TransactionType::Income)
            .map(|t| t.amount)
            .sum();

        let total_expense = filtered_transactions
            .iter()
            .filter(|t| t.transaction_type == TransactionType::Expense)
            .map(|t| t.amount)
            .sum();

        let net_income = total_income - total_expense;

        let account_balance = accounts
            .iter()
            .filter(|a| a.is_active)
            .map(|a| a.balance)
            .sum();

        let transaction_count = filtered_transactions.len() as i64;

        Ok(FinancialStats {
            total_income,
            total_expense,
            net_income,
            account_balance,
            transaction_count,
            period_start: start_date,
            period_end: end_date,
            currency: self.default_currency.clone(),
        })
    }

    /// 生成分类统计
    pub fn generate_category_breakdown(
        &self,
        transactions: &[Transaction],
        categories: &[TransactionCategory],
        transaction_type: TransactionType,
    ) -> Result<Vec<CategoryBreakdown>> {
        let filtered_transactions: Vec<_> = transactions
            .iter()
            .filter(|t| t.transaction_type == transaction_type)
            .collect();

        let total_amount: f64 = filtered_transactions.iter().map(|t| t.amount).sum();

        let mut category_stats: HashMap<Uuid, (String, f64, i64)> = HashMap::new();

        for transaction in filtered_transactions {
            if let Some(cat_id) = transaction.category_id {
                if let Some(category) = categories.iter().find(|c| c.id == cat_id) {
                    let entry =
                        category_stats
                            .entry(cat_id)
                            .or_insert((category.name.clone(), 0.0, 0));
                    entry.1 += transaction.amount;
                    entry.2 += 1;
                }
            }
        }

        let mut breakdown: Vec<CategoryBreakdown> = category_stats
            .into_iter()
            .map(
                |(category_id, (category_name, amount, transaction_count))| {
                    let percentage = if total_amount > 0.0 {
                        amount / total_amount * 100.0
                    } else {
                        0.0
                    };

                    CategoryBreakdown {
                        category_id,
                        category_name,
                        amount,
                        percentage,
                        transaction_count,
                    }
                },
            )
            .collect();

        // 按金额降序排序
        breakdown.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());

        Ok(breakdown)
    }

    /// 生成月度趋势
    pub fn generate_monthly_trend(
        &self,
        transactions: &[Transaction],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<MonthlyTrend>> {
        let mut monthly_data: HashMap<String, (f64, f64)> = HashMap::new();

        for transaction in transactions {
            if transaction.transaction_date >= start_date
                && transaction.transaction_date <= end_date
            {
                let month_key = format!(
                    "{:04}-{:02}",
                    transaction.transaction_date.year(),
                    transaction.transaction_date.month()
                );

                let entry = monthly_data.entry(month_key).or_insert((0.0, 0.0));

                match transaction.transaction_type {
                    TransactionType::Income => entry.0 += transaction.amount,
                    TransactionType::Expense => entry.1 += transaction.amount,
                    TransactionType::Transfer => {} // 转账不计入趋势
                }
            }
        }

        let mut trends: Vec<MonthlyTrend> = monthly_data
            .into_iter()
            .map(|(month, (income, expense))| MonthlyTrend {
                month,
                income,
                expense,
                net: income - expense,
            })
            .collect();

        // 按月份排序
        trends.sort_by(|a, b| a.month.cmp(&b.month));

        Ok(trends)
    }

    /// 生成完整财务报表
    pub fn generate_financial_report(
        &self,
        transactions: &[Transaction],
        accounts: &[Account],
        categories: &[TransactionCategory],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<FinancialReport> {
        let summary =
            self.generate_financial_stats(transactions, accounts, start_date, end_date)?;

        let income_breakdown =
            self.generate_category_breakdown(transactions, categories, TransactionType::Income)?;

        let expense_breakdown =
            self.generate_category_breakdown(transactions, categories, TransactionType::Expense)?;

        let monthly_trend = self.generate_monthly_trend(transactions, start_date, end_date)?;

        let account_balances = accounts
            .iter()
            .filter(|a| a.is_active)
            .map(|a| AccountBalance {
                account_id: a.id,
                account_name: a.name.clone(),
                balance: a.balance,
                currency: a.currency.clone(),
            })
            .collect();

        Ok(FinancialReport {
            period: (start_date, end_date),
            summary,
            income_breakdown,
            expense_breakdown,
            monthly_trend,
            account_balances,
        })
    }
}

// ==================== 辅助类型定义 ====================

/// 预算状态枚举
#[derive(Debug, Clone)]
pub enum BudgetStatus {
    /// 在预算范围内
    OnTrack { spent: f64, usage_percentage: f64 },
    /// 使用较多（75-90%）
    Warning { spent: f64, usage_percentage: f64 },
    /// 接近限额（90%+）
    NearLimit { spent: f64, usage_percentage: f64 },
    /// 超预算
    OverBudget { spent: f64, over_amount: f64 },
}

/// 预算警告
#[derive(Debug, Clone)]
pub struct BudgetWarning {
    pub budget_id: Uuid,
    pub budget_name: String,
    pub message: String,
    pub severity: WarningSeverity,
}

/// 警告严重程度
#[derive(Debug, Clone)]
pub enum WarningSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for AccountingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accounting_manager_creation() {
        let manager = AccountingManager::new();
        assert_eq!(manager.get_default_currency(), "CNY");
        assert!(manager.account_balances.is_empty());
    }

    #[test]
    fn test_create_account() {
        let mut manager = AccountingManager::new();
        let account = manager
            .create_account(
                "测试账户".to_string(),
                AccountType::Bank,
                1000.0,
                None,
                Some("测试描述".to_string()),
            )
            .unwrap();

        assert_eq!(account.name, "测试账户");
        assert_eq!(account.balance, 1000.0);
        assert_eq!(manager.get_account_balance(account.id), 1000.0);
    }

    #[test]
    fn test_create_transaction() {
        let mut manager = AccountingManager::new();
        let account_id = Uuid::new_v4();

        // 模拟账户存在
        manager.account_balances.insert(account_id, 1000.0);

        let transaction = manager
            .create_transaction(
                TransactionType::Expense,
                100.0,
                "测试支出".to_string(),
                account_id,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        assert_eq!(transaction.amount, 100.0);
        assert_eq!(transaction.transaction_type, TransactionType::Expense);
        // 余额应该减少
        assert_eq!(manager.get_account_balance(account_id), 900.0);
    }

    #[test]
    fn test_budget_status() {
        let mut manager = AccountingManager::new();
        let budget = Budget::new(
            "测试预算".to_string(),
            Uuid::new_v4(),
            1000.0,
            "CNY".to_string(),
            BudgetPeriod::Monthly,
            Local::now().date_naive(),
        );

        // 测试正常状态
        manager.update_budget_usage(budget.id, 300.0).unwrap();
        if let BudgetStatus::OnTrack {
            usage_percentage, ..
        } = manager.check_budget_status(&budget)
        {
            assert_eq!(usage_percentage, 30.0);
        } else {
            panic!("预期状态为 OnTrack");
        }

        // 测试超预算状态
        manager.update_budget_usage(budget.id, 1200.0).unwrap();
        if let BudgetStatus::OverBudget { over_amount, .. } = manager.check_budget_status(&budget) {
            assert_eq!(over_amount, 200.0);
        } else {
            panic!("预期状态为 OverBudget");
        }
    }

    #[test]
    fn test_transaction_filtering() {
        let manager = AccountingManager::new();
        let account_id = Uuid::new_v4();
        let category_id = Uuid::new_v4();

        let transaction = Transaction::new(
            TransactionType::Expense,
            100.0,
            "CNY".to_string(),
            "测试支出".to_string(),
            account_id,
            Local::now().date_naive(),
        );

        let transactions = vec![transaction];

        let query = TransactionQuery {
            account_id: Some(account_id),
            transaction_type: Some(TransactionType::Expense),
            min_amount: Some(50.0),
            max_amount: Some(150.0),
            ..Default::default()
        };

        let filtered = manager.filter_transactions(&transactions, &query);
        assert_eq!(filtered.len(), 1);
    }
}
