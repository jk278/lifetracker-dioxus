//! # 记账功能核心业务逻辑模块
//!
//! 按功能领域拆分的记账系统核心逻辑：
//! - account: 账户管理
//! - transaction: 交易处理
//! - budget: 预算控制
//! - category: 分类管理
//! - analytics: 财务分析

pub mod account;
pub mod analytics;
pub mod budget;
pub mod category;
pub mod transaction;
pub mod types;

// 重新导出主要类型和管理器
pub use account::AccountManager;
pub use analytics::AnalyticsManager;
pub use budget::BudgetManager;
pub use category::CategoryManager;
pub use transaction::TransactionManager;

pub use types::*;

/// 记账系统的聚合根管理器
#[derive(Debug)]
pub struct AccountingManager {
    pub account: AccountManager,
    pub transaction: TransactionManager,
    pub budget: BudgetManager,
    pub category: CategoryManager,
    pub analytics: AnalyticsManager,
}

impl AccountingManager {
    /// 创建新的记账管理器
    pub fn new() -> Self {
        Self {
            account: AccountManager::new(),
            transaction: TransactionManager::new(),
            budget: BudgetManager::new(),
            category: CategoryManager::new(),
            analytics: AnalyticsManager::new(),
        }
    }

    /// 设置默认货币
    pub fn set_default_currency(&mut self, currency: String) {
        self.account.set_default_currency(currency.clone());
        self.transaction.set_default_currency(currency.clone());
        self.budget.set_default_currency(currency.clone());
        self.analytics.set_default_currency(currency);
    }

    /// 获取默认货币
    pub fn get_default_currency(&self) -> &str {
        self.account.get_default_currency()
    }
}

impl Default for AccountingManager {
    fn default() -> Self {
        Self::new()
    }
}
