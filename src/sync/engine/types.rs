//! # 同步引擎共享类型定义
//!
//! 定义同步引擎各模块间共享的数据结构和枚举

use chrono::{DateTime, Local};

/// 数据比较结果
#[derive(Debug, Clone, PartialEq)]
pub enum DataComparisonResult {
    /// 本地数据更新
    LocalNewer,
    /// 远程数据更新
    RemoteNewer,
    /// 数据冲突
    Conflict,
    /// 需要合并（全新本地数据 + 远程数据存在）
    NeedsMerge,
    /// 数据相同
    Same,
}

/// 数据来源类型
#[derive(Debug, Clone, PartialEq)]
pub enum DataOrigin {
    /// 全新创建的数据
    Fresh,
    /// 基于远程数据的本地修改
    BasedOnRemote,
    /// 未知来源
    Unknown,
}

/// 冲突解决方案
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    /// 使用本地数据
    UseLocal,
    /// 使用远程数据
    UseRemote,
    /// 合并数据
    Merge,
    /// 跳过冲突
    Skip,
}

/// 数据完整性报告
#[derive(Debug, Clone)]
pub struct DataIntegrityReport {
    /// 是否有效
    pub is_valid: bool,
    /// 错误信息
    pub errors: Vec<String>,
    /// 任务数量
    pub task_count: usize,
    /// 分类数量
    pub category_count: usize,
    /// 时间记录数量
    pub time_entry_count: usize,
    /// 账户数量
    pub account_count: usize,
    /// 交易数量
    pub transaction_count: usize,
}

impl DataIntegrityReport {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            task_count: 0,
            category_count: 0,
            time_entry_count: 0,
            account_count: 0,
            transaction_count: 0,
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }
}

/// 同步策略类型
#[derive(Debug, Clone, PartialEq)]
pub enum SyncStrategyType {
    /// 完全同步
    Full,
    /// 增量同步
    Incremental,
}

/// 同步策略
#[derive(Debug, Clone)]
pub struct SyncStrategy {
    /// 策略类型
    pub strategy_type: SyncStrategyType,
    /// 上次同步时间
    pub last_sync_time: Option<DateTime<Local>>,
    /// 冲突解决策略
    pub conflict_resolution: crate::sync::ConflictStrategy,
    /// 是否启用压缩
    pub compression_enabled: bool,
    /// 最大文件大小
    pub max_file_size: u32,
}

/// 同步项元数据
#[derive(Debug, Clone)]
pub struct SyncMetadata {
    /// 文件路径
    pub path: String,
    /// 文件大小
    pub size: u64,
    /// 修改时间
    pub modified: DateTime<Local>,
    /// 文件哈希
    pub hash: String,
    /// 是否为目录
    pub is_directory: bool,
}

/// 合并配置
#[derive(Debug, Clone)]
pub struct MergeConfig {
    /// 是否去重
    pub deduplicate: bool,
    /// 优先级策略
    pub priority_strategy: MergePriorityStrategy,
}

/// 合并优先级策略
#[derive(Debug, Clone, PartialEq)]
pub enum MergePriorityStrategy {
    /// 本地优先
    LocalFirst,
    /// 远程优先
    RemoteFirst,
    /// 时间戳优先
    TimestampFirst,
}

impl Default for MergeConfig {
    fn default() -> Self {
        Self {
            deduplicate: true,
            priority_strategy: MergePriorityStrategy::TimestampFirst,
        }
    }
}
