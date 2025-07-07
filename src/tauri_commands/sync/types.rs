//! # 同步模块类型定义
//!
//! 包含同步功能相关的所有数据结构定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 同步配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfigRequest {
    /// 是否启用同步
    pub enabled: bool,
    /// 同步提供者
    pub provider: String,
    /// 是否启用自动同步
    pub auto_sync: bool,
    /// 同步间隔（分钟）
    pub sync_interval: u32,
    /// 冲突解决策略
    pub conflict_strategy: String,
    /// WebDAV配置
    pub webdav_config: Option<WebDavConfigRequest>,
}

/// WebDAV配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebDavConfigRequest {
    /// 服务器URL
    pub url: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 同步目录
    pub directory: String,
}

/// 同步状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusResponse {
    /// 同步状态
    pub status: String,
    /// 是否正在同步
    pub is_syncing: bool,
    /// 最后同步时间
    pub last_sync_time: Option<String>,
    /// 下次同步时间
    pub next_sync_time: Option<String>,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 同步结果响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResultResponse {
    /// 同步是否成功
    pub success: bool,
    /// 开始时间
    pub start_time: String,
    /// 结束时间
    pub end_time: String,
    /// 上传的文件数量
    pub uploaded_count: u32,
    /// 下载的文件数量
    pub downloaded_count: u32,
    /// 跳过的文件数量
    pub skipped_count: u32,
    /// 失败的文件数量
    pub failed_count: u32,
    /// 同步的字节数
    pub total_bytes: u64,
    /// 错误信息
    pub errors: Vec<String>,
    /// 冲突数量
    pub conflicts_count: u32,
    /// 耗时（秒）
    pub duration_seconds: i64,
}

/// 冲突项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictItem {
    pub id: String,
    pub name: String,
    pub local_modified: String,
    pub remote_modified: Option<String>,
    pub conflict_type: String,
    pub local_preview: serde_json::Value,
    pub remote_preview: serde_json::Value,
    pub file_size: u64,
    pub local_hash: String,
    pub remote_hash: Option<String>,
}
