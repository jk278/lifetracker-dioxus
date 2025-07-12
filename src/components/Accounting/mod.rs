//! # 财务管理模块
//!
//! 包含账户管理、交易记录、财务统计等功能

pub mod accounting_page;
pub mod accounts;
pub mod overview;
pub mod stats;
pub mod transactions;
pub mod trend_chart;

pub use accounting_page::AccountingPage;
pub use accounts::{AccountsTab, AccountsTabProps};
pub use overview::{OverviewTab, OverviewTabProps};
pub use stats::{StatsTab, StatsTabProps};
pub use transactions::{TransactionsTab, TransactionsTabProps};
pub use trend_chart::FinancialTrendChart;
