//! # 账户管理核心逻辑
//!
//! 负责账户的创建、更新、余额管理等操作

use crate::errors::{AppError, Result};
use crate::storage::{Account, AccountType};
use std::collections::HashMap;
use uuid::Uuid;

/// 账户管理器
#[derive(Debug)]
pub struct AccountManager {
    /// 当前默认货币
    default_currency: String,
    /// 账户余额缓存
    account_balances: HashMap<Uuid, f64>,
}

impl AccountManager {
    /// 创建新的账户管理器
    pub fn new() -> Self {
        Self {
            default_currency: "CNY".to_string(),
            account_balances: HashMap::new(),
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

    /// 设置账户余额（直接设置）
    pub fn set_account_balance(&mut self, account_id: Uuid, balance: f64) -> Result<()> {
        self.account_balances.insert(account_id, balance);
        log::debug!("设置账户余额: {} = {}", account_id, balance);
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

    /// 验证账户余额是否足够
    pub fn validate_sufficient_balance(&self, account_id: Uuid, amount: f64) -> Result<()> {
        let current_balance = self.get_account_balance(account_id);
        if current_balance < amount {
            return Err(AppError::Validation(format!(
                "账户余额不足，当前余额: {:.2}，需要: {:.2}",
                current_balance, amount
            )));
        }
        Ok(())
    }

    /// 批量更新账户余额
    pub fn batch_update_balances(&mut self, updates: Vec<(Uuid, f64)>) -> Result<()> {
        for (account_id, amount) in updates {
            self.update_account_balance(account_id, amount)?;
        }
        Ok(())
    }

    /// 清空余额缓存
    pub fn clear_balance_cache(&mut self) {
        self.account_balances.clear();
        log::debug!("清空账户余额缓存");
    }

    /// 重新加载账户余额缓存
    pub fn reload_balance_cache(&mut self, accounts: &[Account]) {
        self.account_balances.clear();
        for account in accounts {
            self.account_balances.insert(account.id, account.balance);
        }
        log::debug!("重新加载了 {} 个账户的余额缓存", accounts.len());
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_manager_creation() {
        let manager = AccountManager::new();
        assert_eq!(manager.get_default_currency(), "CNY");
        assert!(manager.account_balances.is_empty());
    }

    #[test]
    fn test_create_account() {
        let mut manager = AccountManager::new();
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
    fn test_balance_operations() {
        let mut manager = AccountManager::new();
        let account_id = Uuid::new_v4();

        // 设置初始余额
        manager.set_account_balance(account_id, 1000.0).unwrap();
        assert_eq!(manager.get_account_balance(account_id), 1000.0);

        // 更新余额
        manager.update_account_balance(account_id, -100.0).unwrap();
        assert_eq!(manager.get_account_balance(account_id), 900.0);

        // 验证余额充足
        assert!(manager
            .validate_sufficient_balance(account_id, 500.0)
            .is_ok());
        assert!(manager
            .validate_sufficient_balance(account_id, 1500.0)
            .is_err());
    }
}
