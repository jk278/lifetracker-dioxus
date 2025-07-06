//! # 同步模块
//!
//! 提供多端数据同步功能，支持 WebDAV、GitHub、本地网络等多种同步方式

pub mod conflict;
pub mod engine;
pub mod providers;
pub mod scheduler;

use crate::errors::{AppError, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 同步状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// 空闲状态
    Idle,
    /// 同步中
    Syncing,
    /// 同步成功
    Success,
    /// 同步失败
    Failed(String),
    /// 需要用户处理冲突
    ConflictPending,
}

/// 同步方向
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyncDirection {
    /// 上传（本地到远程）
    Upload,
    /// 下载（远程到本地）
    Download,
    /// 双向同步
    Bidirectional,
}

/// 同步项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncItem {
    /// 项目ID
    pub id: String,
    /// 项目名称
    pub name: String,
    /// 本地路径
    pub local_path: String,
    /// 远程路径
    pub remote_path: String,
    /// 文件大小
    pub size: u64,
    /// 本地修改时间
    pub local_modified: DateTime<Local>,
    /// 远程修改时间
    pub remote_modified: Option<DateTime<Local>>,
    /// 文件哈希值
    pub hash: String,
    /// 同步状态
    pub status: SyncStatus,
    /// 同步方向
    pub direction: SyncDirection,
}

/// 同步结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// 同步是否成功
    pub success: bool,
    /// 同步开始时间
    pub start_time: DateTime<Local>,
    /// 同步结束时间
    pub end_time: DateTime<Local>,
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
    /// 冲突的文件
    pub conflicts: Vec<SyncItem>,
}

/// 同步配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// 同步提供者
    pub provider: String,
    /// 同步设置
    pub settings: HashMap<String, String>,
    /// 同步间隔（分钟）
    pub interval: u32,
    /// 是否启用自动同步
    pub auto_sync: bool,
    /// 冲突解决策略
    pub conflict_strategy: ConflictStrategy,
    /// 忽略文件模式
    pub ignore_patterns: Vec<String>,
    /// 最大文件大小（MB）
    pub max_file_size: u32,
    /// 是否启用压缩
    pub compression: bool,
}

/// 冲突解决策略
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictStrategy {
    /// 手动解决
    Manual,
    /// 本地优先
    LocalWins,
    /// 远程优先
    RemoteWins,
    /// 保留两个版本
    KeepBoth,
}

/// 同步提供者特质
#[async_trait::async_trait]
pub trait SyncProvider: Send + Sync {
    /// 获取提供者名称
    fn name(&self) -> &str;

    /// 测试连接
    async fn test_connection(&self) -> Result<bool>;

    /// 获取远程文件列表
    async fn list_remote_files(&self, path: &str) -> Result<Vec<SyncItem>>;

    /// 上传文件
    async fn upload_file(&self, item: &SyncItem, data: &[u8]) -> Result<()>;

    /// 下载文件
    async fn download_file(&self, item: &SyncItem) -> Result<Vec<u8>>;

    /// 删除远程文件
    async fn delete_remote_file(&self, item: &SyncItem) -> Result<()>;

    /// 创建远程目录
    async fn create_remote_directory(&self, path: &str) -> Result<()>;

    /// 获取文件元数据
    async fn get_file_metadata(&self, path: &str) -> Result<SyncItem>;

    /// 克隆提供者
    fn clone_provider(&self) -> Box<dyn SyncProvider>;
}

/// 同步事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncEvent {
    /// 同步开始
    Started,
    /// 文件上传开始
    UploadStarted { file: String },
    /// 文件上传完成
    UploadCompleted { file: String },
    /// 文件下载开始
    DownloadStarted { file: String },
    /// 文件下载完成
    DownloadCompleted { file: String },
    /// 同步进度更新
    Progress { current: u32, total: u32 },
    /// 发现冲突
    ConflictDetected { file: String },
    /// 同步完成
    Completed { result: SyncResult },
    /// 同步失败
    Failed { error: String },
}

/// 同步事件监听器
pub trait SyncEventListener: Send + Sync {
    /// 处理同步事件
    fn on_sync_event(&self, event: SyncEvent);
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            provider: "webdav".to_string(),
            settings: HashMap::new(),
            interval: 30,
            auto_sync: false,
            conflict_strategy: ConflictStrategy::Manual,
            ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.log".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
            ],
            max_file_size: 10,
            compression: true,
        }
    }
}

impl SyncResult {
    /// 创建新的同步结果
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            success: true,
            start_time: now,
            end_time: now,
            uploaded_count: 0,
            downloaded_count: 0,
            skipped_count: 0,
            failed_count: 0,
            total_bytes: 0,
            errors: Vec::new(),
            conflicts: Vec::new(),
        }
    }

    /// 标记同步完成
    pub fn complete(&mut self, success: bool) {
        self.success = success;
        self.end_time = Local::now();
    }

    /// 添加错误
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.success = false;
    }

    /// 添加冲突
    pub fn add_conflict(&mut self, item: SyncItem) {
        self.conflicts.push(item);
    }

    /// 获取同步耗时
    pub fn duration(&self) -> chrono::Duration {
        self.end_time - self.start_time
    }
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::Idle => write!(f, "空闲"),
            SyncStatus::Syncing => write!(f, "同步中"),
            SyncStatus::Success => write!(f, "成功"),
            SyncStatus::Failed(msg) => write!(f, "失败: {}", msg),
            SyncStatus::ConflictPending => write!(f, "冲突待处理"),
        }
    }
}

impl std::fmt::Display for ConflictStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictStrategy::Manual => write!(f, "手动解决"),
            ConflictStrategy::LocalWins => write!(f, "本地优先"),
            ConflictStrategy::RemoteWins => write!(f, "远程优先"),
            ConflictStrategy::KeepBoth => write!(f, "保留两个版本"),
        }
    }
}

/// 创建同步配置的便捷函数
pub fn create_webdav_config(
    url: &str,
    username: &str,
    password: &str,
    directory: &str,
) -> SyncConfig {
    let mut settings = HashMap::new();
    settings.insert("url".to_string(), url.to_string());
    settings.insert("username".to_string(), username.to_string());
    settings.insert("password".to_string(), password.to_string());
    settings.insert("directory".to_string(), directory.to_string());

    SyncConfig {
        provider: "webdav".to_string(),
        settings,
        ..Default::default()
    }
}

/// 验证同步配置
pub fn validate_sync_config(config: &SyncConfig) -> Result<()> {
    if config.provider.is_empty() {
        return Err(AppError::Validation("同步提供者不能为空".to_string()));
    }

    if config.interval < 5 {
        return Err(AppError::Validation("同步间隔不能小于5分钟".to_string()));
    }

    if config.max_file_size == 0 {
        return Err(AppError::Validation("最大文件大小必须大于0".to_string()));
    }

    // 验证提供者特定的设置
    match config.provider.as_str() {
        "webdav" => {
            if !config.settings.contains_key("url") {
                return Err(AppError::Validation("WebDAV配置缺少URL".to_string()));
            }
            if !config.settings.contains_key("username") {
                return Err(AppError::Validation("WebDAV配置缺少用户名".to_string()));
            }
        }
        "github" => {
            if !config.settings.contains_key("token") {
                return Err(AppError::Validation("GitHub配置缺少访问令牌".to_string()));
            }
            if !config.settings.contains_key("repo") {
                return Err(AppError::Validation("GitHub配置缺少仓库名".to_string()));
            }
        }
        _ => {
            return Err(AppError::Validation(format!(
                "不支持的同步提供者: {}",
                config.provider
            )));
        }
    }

    Ok(())
}

/// 格式化字节数
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_result_creation() {
        let mut result = SyncResult::new();
        assert!(result.success);
        assert_eq!(result.uploaded_count, 0);

        result.add_error("测试错误".to_string());
        assert!(!result.success);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_sync_config_validation() {
        let mut config = SyncConfig::default();
        config.provider = "webdav".to_string();

        // 缺少WebDAV设置
        assert!(validate_sync_config(&config).is_err());

        // 添加必要的设置
        config
            .settings
            .insert("url".to_string(), "https://example.com".to_string());
        config
            .settings
            .insert("username".to_string(), "user".to_string());
        assert!(validate_sync_config(&config).is_ok());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1536), "1.5 KB");
    }

    #[test]
    fn test_webdav_config_creation() {
        let config = create_webdav_config(
            "https://example.com/webdav",
            "testuser",
            "testpass",
            "LifeTracker",
        );

        assert_eq!(config.provider, "webdav");
        assert_eq!(
            config.settings.get("url").unwrap(),
            "https://example.com/webdav"
        );
        assert_eq!(config.settings.get("username").unwrap(), "testuser");
        assert_eq!(config.settings.get("directory").unwrap(), "LifeTracker");
    }
}
