//! # 组件模块
//!
//! 包含所有UI组件的定义和组织

pub mod about;
pub mod accounting; // 现在是模块化的 accounting/ 目录
pub mod animation;
pub mod app;
pub mod common;
pub mod dashboard;
pub mod data_management;
pub mod diary; // 现在是模块化的 diary/ 目录
pub mod habits;
pub mod settings;
pub mod system_page;
pub mod timing; // 现在是模块化的 timing/ 目录
pub mod title_bar;

// 重新导出主要组件
pub use about::AboutPage;
pub use app::App;
pub use dashboard::Dashboard;
pub use data_management::DataManagementPage;
pub use settings::SettingsPage;
pub use system_page::SystemPage;
pub use title_bar::TitleBar;

// 重新导出 timing 模块的主要组件
pub use timing::{
    CategoryManagement, StatisticsPlaceholder, TaskManagementContent, TimingDashboard, TimingPage,
};

// 重新导出 accounting 模块的主要组件
pub use accounting::AccountingPage;

// 重新导出 diary 模块的主要组件
pub use diary::DiaryPage;

// 重新导出通用组件
pub use common::{
    clear_error_info, set_error_info, use_error_handler, Button, ButtonSize, ButtonVariant, Card,
    EmptyState, ErrorBoundary, ErrorInfo, ErrorType, Input, Loading, Modal, Notification,
    NotificationVariant, Tag, TagVariant, Textarea,
};

// 重新导出动画组件
pub use animation::{
    AnimatedList, AnimationDirection, AnimationType, BottomSheet, GestureWrapper,
    InteractiveButton, PageTransition, TabTransition, ViewContainer,
};
