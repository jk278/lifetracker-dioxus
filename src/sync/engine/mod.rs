//! # 同步引擎模块
//!
//! 提供数据同步的核心功能，包括：
//! - 同步引擎核心逻辑
//! - 数据序列化和反序列化
//! - 数据比较和冲突检测
//! - 冲突解决策略
//! - 数据合并逻辑
//! - 数据完整性验证

pub mod comparator;
pub mod conflict_resolver;
pub mod core;
pub mod integrity_checker;
pub mod merger;
pub mod serializer;
pub mod types;
pub mod validator;

// 重新导出核心类型和功能
pub use comparator::DataComparator;
pub use conflict_resolver::ConflictResolver;
pub use core::SyncEngine;
pub use integrity_checker::{ConflictDetectionResult, DataIntegrityChecker, RiskLevel};
pub use merger::DataMerger;
pub use serializer::DataSerializer;
pub use types::*;
pub use validator::DataValidator;
