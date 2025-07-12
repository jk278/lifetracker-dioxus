//! # 日记模块
//!
//! 包含日记写作、心情记录、笔记库管理和统计功能

// 声明子模块
pub mod diary_page;
pub mod editor;
pub mod library;
pub mod overview;
pub mod stats;

// 重新导出主要组件供外部使用
pub use diary_page::DiaryPage;
pub use editor::NotesEditor;
pub use library::NotesLibrary;
pub use overview::NotesOverview;
pub use stats::NotesStats;
