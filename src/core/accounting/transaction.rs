//! # 交易管理核心逻辑
//!
//! 负责交易的创建、更新、删除和查询过滤等操作

use crate::errors::{AppError, Result};
use crate::storage::{Transaction, TransactionQuery, TransactionType, TransactionUpdate};
use chrono::{Local, NaiveDate};
use uuid::Uuid;

/// 交易管理器
#[derive(Debug)]
pub struct TransactionManager {
    /// 默认货币
    default_currency: String,
}

impl TransactionManager {
    /// 创建新的交易管理器
    pub fn new() -> Self {
        Self {
            default_currency: "CNY".to_string(),
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
        // 验证交易金额
        if amount <= 0.0 {
            return Err(AppError::Validation("交易金额必须大于0".to_string()));
        }

        // 验证转账必须有目标账户
        if transaction_type == TransactionType::Transfer && to_account_id.is_none() {
            return Err(AppError::Validation("转账必须指定目标账户".to_string()));
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

        log::info!(
            "创建交易: {:?} {} {}",
            transaction_type,
            amount,
            transaction.id
        );
        Ok(transaction)
    }

    /// 验证交易更新
    pub fn validate_transaction_update(
        &self,
        original_transaction: &Transaction,
        update: &TransactionUpdate,
    ) -> Result<()> {
        // 验证金额
        if let Some(amount) = update.amount {
            if amount <= 0.0 {
                return Err(AppError::Validation("交易金额必须大于0".to_string()));
            }
        }

        // 验证转账类型
        let new_type = update
            .transaction_type
            .unwrap_or(original_transaction.transaction_type);
        let new_to_account = update
            .to_account_id
            .unwrap_or(original_transaction.to_account_id);

        if new_type == TransactionType::Transfer && new_to_account.is_none() {
            return Err(AppError::Validation("转账必须指定目标账户".to_string()));
        }

        Ok(())
    }

    /// 计算交易对账户余额的影响
    pub fn calculate_balance_impact(&self, transaction: &Transaction) -> Result<Vec<(Uuid, f64)>> {
        let mut impacts = Vec::new();

        match transaction.transaction_type {
            TransactionType::Income => {
                impacts.push((transaction.account_id, transaction.amount));
            }
            TransactionType::Expense => {
                impacts.push((transaction.account_id, -transaction.amount));
            }
            TransactionType::Transfer => {
                if let Some(to_id) = transaction.to_account_id {
                    impacts.push((transaction.account_id, -transaction.amount));
                    impacts.push((to_id, transaction.amount));
                } else {
                    return Err(AppError::Validation("转账缺少目标账户".to_string()));
                }
            }
        }

        Ok(impacts)
    }

    /// 计算撤销交易对账户余额的影响
    pub fn calculate_revert_balance_impact(
        &self,
        transaction: &Transaction,
    ) -> Result<Vec<(Uuid, f64)>> {
        let impacts = self.calculate_balance_impact(transaction)?;
        // 撤销操作：将所有影响取反
        Ok(impacts
            .into_iter()
            .map(|(id, amount)| (id, -amount))
            .collect())
    }

    /// 搜索和过滤交易
    pub fn filter_transactions(
        &self,
        transactions: &[Transaction],
        query: &TransactionQuery,
    ) -> Vec<Transaction> {
        transactions
            .iter()
            .filter(|t| self.matches_query(t, query))
            .cloned()
            .collect()
    }

    /// 检查交易是否匹配查询条件
    fn matches_query(&self, transaction: &Transaction, query: &TransactionQuery) -> bool {
        // 账户过滤
        if let Some(account_id) = &query.account_id {
            if transaction.account_id != *account_id {
                return false;
            }
        }

        // 分类过滤
        if let Some(category_id) = &query.category_id {
            if transaction.category_id != Some(*category_id) {
                return false;
            }
        }

        // 交易类型过滤
        if let Some(transaction_type) = &query.transaction_type {
            if transaction.transaction_type != *transaction_type {
                return false;
            }
        }

        // 状态过滤
        if let Some(status) = &query.status {
            if transaction.status != *status {
                return false;
            }
        }

        // 日期范围过滤
        if let Some(start_date) = &query.start_date {
            if transaction.transaction_date < *start_date {
                return false;
            }
        }

        if let Some(end_date) = &query.end_date {
            if transaction.transaction_date > *end_date {
                return false;
            }
        }

        // 金额范围过滤
        if let Some(min_amount) = &query.min_amount {
            if transaction.amount < *min_amount {
                return false;
            }
        }

        if let Some(max_amount) = &query.max_amount {
            if transaction.amount > *max_amount {
                return false;
            }
        }

        // 标签过滤
        if let Some(tags) = &query.tags {
            if !tags.iter().any(|tag| transaction.tags.contains(tag)) {
                return false;
            }
        }

        // 搜索文本过滤
        if let Some(search) = &query.search {
            let search_lower = search.to_lowercase();
            if !transaction
                .description
                .to_lowercase()
                .contains(&search_lower)
            {
                return false;
            }
        }

        true
    }

    /// 按类型分组交易
    pub fn group_by_type(
        &self,
        transactions: &[Transaction],
    ) -> std::collections::HashMap<TransactionType, Vec<Transaction>> {
        let mut groups = std::collections::HashMap::new();

        for transaction in transactions {
            groups
                .entry(transaction.transaction_type)
                .or_insert_with(Vec::new)
                .push(transaction.clone());
        }

        groups
    }

    /// 按日期分组交易
    pub fn group_by_date(
        &self,
        transactions: &[Transaction],
    ) -> std::collections::HashMap<NaiveDate, Vec<Transaction>> {
        let mut groups = std::collections::HashMap::new();

        for transaction in transactions {
            groups
                .entry(transaction.transaction_date)
                .or_insert_with(Vec::new)
                .push(transaction.clone());
        }

        groups
    }

    /// 计算总金额
    pub fn calculate_total_amount(
        &self,
        transactions: &[Transaction],
        transaction_type: Option<TransactionType>,
    ) -> f64 {
        transactions
            .iter()
            .filter(|t| transaction_type.map_or(true, |tt| t.transaction_type == tt))
            .map(|t| t.amount)
            .sum()
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_transaction_manager_creation() {
        let manager = TransactionManager::new();
        assert_eq!(manager.get_default_currency(), "CNY");
    }

    #[test]
    fn test_create_valid_transaction() {
        let mut manager = TransactionManager::new();
        let account_id = Uuid::new_v4();

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
        assert_eq!(transaction.account_id, account_id);
    }

    #[test]
    fn test_invalid_transaction_amount() {
        let mut manager = TransactionManager::new();
        let account_id = Uuid::new_v4();

        let result = manager.create_transaction(
            TransactionType::Expense,
            -100.0,
            "无效支出".to_string(),
            account_id,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_transfer_without_target_account() {
        let mut manager = TransactionManager::new();
        let account_id = Uuid::new_v4();

        let result = manager.create_transaction(
            TransactionType::Transfer,
            100.0,
            "转账".to_string(),
            account_id,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_balance_impact_calculation() {
        let manager = TransactionManager::new();
        let account_id = Uuid::new_v4();
        let to_account_id = Uuid::new_v4();

        // 测试收入
        let income_transaction = Transaction::new(
            TransactionType::Income,
            100.0,
            "CNY".to_string(),
            "收入".to_string(),
            account_id,
            Local::now().date_naive(),
        );

        let impact = manager
            .calculate_balance_impact(&income_transaction)
            .unwrap();
        assert_eq!(impact.len(), 1);
        assert_eq!(impact[0], (account_id, 100.0));

        // 测试转账
        let mut transfer_transaction = Transaction::new(
            TransactionType::Transfer,
            100.0,
            "CNY".to_string(),
            "转账".to_string(),
            account_id,
            Local::now().date_naive(),
        );
        transfer_transaction.to_account_id = Some(to_account_id);

        let impact = manager
            .calculate_balance_impact(&transfer_transaction)
            .unwrap();
        assert_eq!(impact.len(), 2);
        assert!(impact.contains(&(account_id, -100.0)));
        assert!(impact.contains(&(to_account_id, 100.0)));
    }

    #[test]
    fn test_transaction_filtering() {
        let manager = TransactionManager::new();
        let account_id = Uuid::new_v4();

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
