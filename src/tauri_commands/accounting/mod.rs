//! # 记账功能 Tauri 命令模块
//!
//! 按功能领域拆分的记账系统命令：
//! - account: 账户管理命令
//! - transaction: 交易管理命令  
//! - budget: 预算管理命令
//! - category: 分类管理命令
//! - statistics: 财务统计命令

pub mod account;
pub mod budget;
pub mod category;
pub mod statistics;
pub mod transaction;
pub mod types;

// 重新导出所有命令函数
pub use account::*;
pub use budget::*;
pub use category::*;
pub use statistics::*;
pub use transaction::*;

// 重新导出请求类型
pub use types::*;
