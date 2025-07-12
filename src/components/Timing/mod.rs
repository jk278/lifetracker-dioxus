//! # 时间追踪模块
//!
//! 包含时间追踪相关的所有组件和功能

// 声明子模块
pub mod category_management;
pub mod dashboard;
pub mod statistics;
pub mod task_management;
pub mod timing_page;

// 重新导出主要组件供外部使用
pub use category_management::CategoryManagement;
pub use dashboard::{TimerState, TimingDashboard};
pub use statistics::StatisticsPlaceholder;
pub use task_management::{TaskFormData, TaskManagementContent};
pub use timing_page::{TimingPage, TimingTab};
