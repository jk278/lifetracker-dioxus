//! # 财务分析核心逻辑
//!
//! 负责财务数据分析、报表生成、趋势计算等操作

use crate::errors::{AppError, Result};
use crate::storage::{
    Account, AccountBalance, CategoryBreakdown, FinancialReport, FinancialStats, MonthlyTrend,
    Transaction, TransactionCategory, TransactionType,
};
use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;
use uuid::Uuid;

/// 财务分析管理器
#[derive(Debug)]
pub struct AnalyticsManager {
    /// 默认货币
    default_currency: String,
}

impl AnalyticsManager {
    /// 创建新的财务分析管理器
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

    /// 生成财务统计
    pub fn generate_financial_stats(
        &self,
        transactions: &[Transaction],
        accounts: &[Account],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<FinancialStats> {
        if start_date > end_date {
            return Err(AppError::Validation("开始日期不能晚于结束日期".to_string()));
        }

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
            } else {
                // 处理未分类的交易
                let entry =
                    category_stats
                        .entry(Uuid::nil())
                        .or_insert(("未分类".to_string(), 0.0, 0));
                entry.1 += transaction.amount;
                entry.2 += 1;
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
        if start_date > end_date {
            return Err(AppError::Validation("开始日期不能晚于结束日期".to_string()));
        }

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

    /// 计算储蓄率
    pub fn calculate_savings_rate(
        &self,
        transactions: &[Transaction],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<f64> {
        let filtered_transactions: Vec<_> = transactions
            .iter()
            .filter(|t| t.transaction_date >= start_date && t.transaction_date <= end_date)
            .collect();

        let total_income: f64 = filtered_transactions
            .iter()
            .filter(|t| t.transaction_type == TransactionType::Income)
            .map(|t| t.amount)
            .sum();

        let total_expense: f64 = filtered_transactions
            .iter()
            .filter(|t| t.transaction_type == TransactionType::Expense)
            .map(|t| t.amount)
            .sum();

        if total_income > 0.0 {
            Ok((total_income - total_expense) / total_income * 100.0)
        } else {
            Ok(0.0)
        }
    }

    /// 计算平均日支出
    pub fn calculate_average_daily_expense(
        &self,
        transactions: &[Transaction],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<f64> {
        let total_expense: f64 = transactions
            .iter()
            .filter(|t| {
                t.transaction_type == TransactionType::Expense
                    && t.transaction_date >= start_date
                    && t.transaction_date <= end_date
            })
            .map(|t| t.amount)
            .sum();

        let days = (end_date - start_date).num_days() + 1;
        if days > 0 {
            Ok(total_expense / days as f64)
        } else {
            Ok(0.0)
        }
    }
}

impl Default for AnalyticsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::AccountType;
    use chrono::{Local, NaiveDate};

    #[test]
    fn test_analytics_manager_creation() {
        let manager = AnalyticsManager::new();
        assert_eq!(manager.get_default_currency(), "CNY");
    }

    #[test]
    fn test_generate_financial_stats() {
        let manager = AnalyticsManager::new();

        let today = Local::now().date_naive();
        let start_date = today - chrono::Duration::days(30);

        let transactions = vec![
            Transaction::new(
                TransactionType::Income,
                1000.0,
                "CNY".to_string(),
                "工资".to_string(),
                uuid::Uuid::new_v4(),
                today,
            ),
            Transaction::new(
                TransactionType::Expense,
                300.0,
                "CNY".to_string(),
                "餐饮".to_string(),
                uuid::Uuid::new_v4(),
                today,
            ),
        ];

        let accounts = vec![Account::new(
            "银行卡".to_string(),
            AccountType::Bank,
            "CNY".to_string(),
            5000.0,
        )];

        let stats = manager
            .generate_financial_stats(&transactions, &accounts, start_date, today)
            .unwrap();

        assert_eq!(stats.total_income, 1000.0);
        assert_eq!(stats.total_expense, 300.0);
        assert_eq!(stats.net_income, 700.0);
        assert_eq!(stats.account_balance, 5000.0);
        assert_eq!(stats.transaction_count, 2);
    }

    #[test]
    fn test_invalid_date_range() {
        let manager = AnalyticsManager::new();

        let today = Local::now().date_naive();
        let tomorrow = today + chrono::Duration::days(1);

        let result = manager.generate_financial_stats(&[], &[], tomorrow, today);
        assert!(result.is_err());
    }

    #[test]
    fn test_category_breakdown() {
        let manager = AnalyticsManager::new();

        let category1 = TransactionCategory::new(
            "餐饮".to_string(),
            TransactionType::Expense,
            "#FF0000".to_string(),
        );

        let category2 = TransactionCategory::new(
            "交通".to_string(),
            TransactionType::Expense,
            "#00FF00".to_string(),
        );

        let mut transaction1 = Transaction::new(
            TransactionType::Expense,
            300.0,
            "CNY".to_string(),
            "午餐".to_string(),
            uuid::Uuid::new_v4(),
            Local::now().date_naive(),
        );
        transaction1.set_category(category1.id);

        let mut transaction2 = Transaction::new(
            TransactionType::Expense,
            100.0,
            "CNY".to_string(),
            "地铁".to_string(),
            uuid::Uuid::new_v4(),
            Local::now().date_naive(),
        );
        transaction2.set_category(category2.id);

        let transactions = vec![transaction1, transaction2];
        let categories = vec![category1, category2];

        let breakdown = manager
            .generate_category_breakdown(&transactions, &categories, TransactionType::Expense)
            .unwrap();

        assert_eq!(breakdown.len(), 2);
        assert_eq!(breakdown[0].amount, 300.0); // 按金额降序排序
        assert_eq!(breakdown[0].percentage, 75.0); // 300 / 400 * 100
        assert_eq!(breakdown[1].amount, 100.0);
        assert_eq!(breakdown[1].percentage, 25.0); // 100 / 400 * 100
    }

    #[test]
    fn test_savings_rate() {
        let manager = AnalyticsManager::new();

        let today = Local::now().date_naive();
        let start_date = today - chrono::Duration::days(30);

        let transactions = vec![
            Transaction::new(
                TransactionType::Income,
                1000.0,
                "CNY".to_string(),
                "工资".to_string(),
                uuid::Uuid::new_v4(),
                today,
            ),
            Transaction::new(
                TransactionType::Expense,
                200.0,
                "CNY".to_string(),
                "餐饮".to_string(),
                uuid::Uuid::new_v4(),
                today,
            ),
        ];

        let savings_rate = manager
            .calculate_savings_rate(&transactions, start_date, today)
            .unwrap();

        assert_eq!(savings_rate, 80.0); // (1000 - 200) / 1000 * 100
    }

    #[test]
    fn test_average_daily_expense() {
        let manager = AnalyticsManager::new();

        let today = Local::now().date_naive();
        let start_date = today - chrono::Duration::days(9); // 10 days total

        let transactions = vec![
            Transaction::new(
                TransactionType::Expense,
                100.0,
                "CNY".to_string(),
                "餐饮".to_string(),
                uuid::Uuid::new_v4(),
                today,
            ),
            Transaction::new(
                TransactionType::Expense,
                200.0,
                "CNY".to_string(),
                "购物".to_string(),
                uuid::Uuid::new_v4(),
                today - chrono::Duration::days(5),
            ),
        ];

        let average = manager
            .calculate_average_daily_expense(&transactions, start_date, today)
            .unwrap();

        assert_eq!(average, 30.0); // (100 + 200) / 10 days
    }
}
