//! # 同步引擎核心模块
//!
//! 实现数据同步的核心逻辑和管理

use super::providers::{create_provider, SyncProviderBox};
use super::{
    ConflictStrategy, SyncConfig, SyncEvent, SyncEventListener, SyncItem, SyncResult, SyncStatus,
};
use crate::errors::{AppError, Result};
use crate::storage::StorageManager;
use crate::utils::crypto::{decrypt_password, encrypt_password};
use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::{interval, sleep};

/// 同步引擎
pub struct SyncEngine {
    /// 存储管理器
    storage: Arc<StorageManager>,
    /// 同步配置
    config: SyncConfig,
    /// 同步提供者
    provider: Option<SyncProviderBox>,
    /// 事件监听器
    listeners: Vec<Box<dyn SyncEventListener>>,
    /// 当前同步状态
    status: Arc<Mutex<SyncStatus>>,
    /// 最后一次同步结果
    last_result: Arc<Mutex<Option<SyncResult>>>,
    /// 是否正在运行
    running: Arc<Mutex<bool>>,
}

/// 数据序列化器
pub struct DataSerializer {
    storage: Arc<StorageManager>,
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

impl SyncEngine {
    /// 创建新的同步引擎
    pub fn new(storage: Arc<StorageManager>, config: SyncConfig) -> Result<Self> {
        Ok(Self {
            storage,
            config,
            provider: None,
            listeners: Vec::new(),
            status: Arc::new(Mutex::new(SyncStatus::Idle)),
            last_result: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// 初始化同步引擎
    pub async fn initialize(&mut self) -> Result<()> {
        log::info!("初始化同步引擎");

        // 验证配置
        super::validate_sync_config(&self.config)?;

        // 创建同步提供者
        self.provider = Some(create_provider(&self.config).await?);

        // 测试连接
        if let Some(provider) = &self.provider {
            match provider.test_connection().await {
                Ok(true) => {
                    log::info!("同步提供者连接测试成功");
                }
                Ok(false) => {
                    log::warn!("同步提供者连接测试失败");
                    return Err(AppError::Sync("连接测试失败".to_string()));
                }
                Err(e) => {
                    log::error!("同步提供者连接测试出错: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// 添加事件监听器
    pub fn add_listener(&mut self, listener: Box<dyn SyncEventListener>) {
        self.listeners.push(listener);
    }

    /// 发送事件
    fn emit_event(&self, event: SyncEvent) {
        for listener in &self.listeners {
            listener.on_sync_event(event.clone());
        }
    }

    /// 开始同步
    pub async fn sync(&self) -> Result<SyncResult> {
        log::info!("开始数据同步");

        // 检查是否已在同步中
        {
            let status = self.status.lock().unwrap();
            if matches!(*status, SyncStatus::Syncing) {
                return Err(AppError::Sync("同步已在进行中".to_string()));
            }
        }

        // 更新状态
        {
            let mut status = self.status.lock().unwrap();
            *status = SyncStatus::Syncing;
        }

        self.emit_event(SyncEvent::Started);

        let mut result = SyncResult::new();

        // 执行同步
        match self.perform_sync().await {
            Ok(sync_result) => {
                result = sync_result;
                {
                    let mut status = self.status.lock().unwrap();
                    *status = if result.success {
                        SyncStatus::Success
                    } else {
                        SyncStatus::Failed("同步过程中出现错误".to_string())
                    };
                }
            }
            Err(e) => {
                log::error!("同步失败: {}", e);
                result.add_error(e.to_string());
                result.complete(false);
                {
                    let mut status = self.status.lock().unwrap();
                    *status = SyncStatus::Failed(e.to_string());
                }
            }
        }

        // 保存结果
        {
            let mut last_result = self.last_result.lock().unwrap();
            *last_result = Some(result.clone());
        }

        // 发送完成事件
        if result.success {
            self.emit_event(SyncEvent::Completed {
                result: result.clone(),
            });
        } else {
            self.emit_event(SyncEvent::Failed {
                error: result.errors.join(", "),
            });
        }

        log::info!("同步完成: {}", if result.success { "成功" } else { "失败" });

        Ok(result)
    }

    /// 执行实际的同步操作
    async fn perform_sync(&self) -> Result<SyncResult> {
        let mut result = SyncResult::new();

        // 获取本地数据
        let local_data = self.export_local_data().await?;

        // 获取远程数据列表
        let provider = self.provider.as_ref().unwrap();
        let remote_directory = self.get_remote_directory();

        // 确保远程目录存在
        provider.create_remote_directory(&remote_directory).await?;

        // 获取远程文件列表
        let remote_files = provider.list_remote_files(&remote_directory).await?;

        // 比较本地和远程数据
        let (upload_items, download_items, conflicts) =
            self.compare_data(&local_data, &remote_files)?;

        // 处理冲突
        if !conflicts.is_empty() {
            for conflict in conflicts {
                result.add_conflict(conflict);
            }

            // 根据冲突策略处理
            match self.config.conflict_strategy {
                ConflictStrategy::Manual => {
                    // 手动处理，暂停同步等待用户选择
                    {
                        let mut status = self.status.lock().unwrap();
                        *status = SyncStatus::ConflictPending;
                    }
                    return Ok(result);
                }
                ConflictStrategy::LocalWins => {
                    // 本地优先，将冲突文件加入上传队列
                    // 这里可以添加具体的处理逻辑
                }
                ConflictStrategy::RemoteWins => {
                    // 远程优先，将冲突文件加入下载队列
                    // 这里可以添加具体的处理逻辑
                }
                ConflictStrategy::KeepBoth => {
                    // 保留两个版本
                    // 这里可以添加具体的处理逻辑
                }
            }
        }

        // 上传文件
        for item in upload_items {
            self.emit_event(SyncEvent::UploadStarted {
                file: item.name.clone(),
            });

            match self.upload_item(&item, &local_data).await {
                Ok(_) => {
                    result.uploaded_count += 1;
                    result.total_bytes += item.size;
                    self.emit_event(SyncEvent::UploadCompleted {
                        file: item.name.clone(),
                    });
                }
                Err(e) => {
                    log::error!("上传文件失败 {}: {}", item.name, e);
                    result.failed_count += 1;
                    result.add_error(format!("上传 {} 失败: {}", item.name, e));
                }
            }

            // 更新进度
            let current = result.uploaded_count + result.downloaded_count + result.failed_count;
            let total = result.uploaded_count
                + result.downloaded_count
                + result.failed_count
                + (download_items.len() as u32);
            self.emit_event(SyncEvent::Progress { current, total });
        }

        // 下载文件
        for item in download_items {
            self.emit_event(SyncEvent::DownloadStarted {
                file: item.name.clone(),
            });

            match self.download_item(&item).await {
                Ok(_) => {
                    result.downloaded_count += 1;
                    result.total_bytes += item.size;
                    self.emit_event(SyncEvent::DownloadCompleted {
                        file: item.name.clone(),
                    });
                }
                Err(e) => {
                    log::error!("下载文件失败 {}: {}", item.name, e);
                    result.failed_count += 1;
                    result.add_error(format!("下载 {} 失败: {}", item.name, e));
                }
            }

            // 更新进度
            let current = result.uploaded_count + result.downloaded_count + result.failed_count;
            let total = current; // 这里应该根据实际情况计算
            self.emit_event(SyncEvent::Progress { current, total });
        }

        result.complete(result.failed_count == 0);
        Ok(result)
    }

    /// 导出本地数据
    async fn export_local_data(&self) -> Result<Vec<u8>> {
        log::info!("导出本地数据");

        let serializer = DataSerializer::new(self.storage.clone());
        serializer.serialize_all_data().await
    }

    /// 上传数据项
    async fn upload_item(&self, item: &SyncItem, data: &[u8]) -> Result<()> {
        let provider = self.provider.as_ref().unwrap();

        // 检查文件大小限制
        if item.size > (self.config.max_file_size as u64 * 1024 * 1024) {
            return Err(AppError::Sync(format!(
                "文件 {} 超过大小限制 {}MB",
                item.name, self.config.max_file_size
            )));
        }

        // 如果启用压缩，压缩数据
        let upload_data = if self.config.compression {
            // 这里可以添加压缩逻辑
            data.to_vec()
        } else {
            data.to_vec()
        };

        provider.upload_file(item, &upload_data).await
    }

    /// 下载数据项
    async fn download_item(&self, item: &SyncItem) -> Result<()> {
        let provider = self.provider.as_ref().unwrap();

        let data = provider.download_file(item).await?;

        // 如果启用压缩，解压数据
        let decompressed_data = if self.config.compression {
            // 这里可以添加解压逻辑
            data
        } else {
            data
        };

        // 导入数据到本地存储
        let serializer = DataSerializer::new(self.storage.clone());
        serializer.import_data(&decompressed_data).await
    }

    /// 比较本地和远程数据
    fn compare_data(
        &self,
        local_data: &[u8],
        remote_files: &[SyncItem],
    ) -> Result<(Vec<SyncItem>, Vec<SyncItem>, Vec<SyncItem>)> {
        let mut upload_items = Vec::new();
        let mut download_items = Vec::new();
        let mut conflicts = Vec::new();

        let local_hash = self.calculate_hash(local_data);
        let local_item = SyncItem {
            id: "local_data".to_string(),
            name: "data.json".to_string(),
            local_path: "local".to_string(),
            remote_path: format!("{}/data.json", self.get_remote_directory()),
            size: local_data.len() as u64,
            local_modified: Local::now(),
            remote_modified: None,
            hash: local_hash,
            status: SyncStatus::Idle,
            direction: super::SyncDirection::Upload,
        };

        // 查找远程的数据文件
        if let Some(remote_item) = remote_files.iter().find(|item| item.name == "data.json") {
            // 比较哈希值
            if local_item.hash != remote_item.hash {
                // 文件不同，检查修改时间
                if let Some(remote_modified) = remote_item.remote_modified {
                    if local_item.local_modified > remote_modified {
                        // 本地更新，上传
                        upload_items.push(local_item);
                    } else if local_item.local_modified < remote_modified {
                        // 远程更新，下载
                        download_items.push(remote_item.clone());
                    } else {
                        // 时间相同但内容不同，存在冲突
                        conflicts.push(local_item);
                    }
                } else {
                    // 无法确定修改时间，标记为冲突
                    conflicts.push(local_item);
                }
            }
            // 如果哈希相同，则文件已同步，无需操作
        } else {
            // 远程不存在，需要上传
            upload_items.push(local_item);
        }

        Ok((upload_items, download_items, conflicts))
    }

    /// 计算数据哈希
    fn calculate_hash(&self, data: &[u8]) -> String {
        let crypto = crate::utils::crypto::CryptoManager::new();
        crypto.calculate_hash(data)
    }

    /// 获取远程目录路径
    fn get_remote_directory(&self) -> String {
        self.config
            .settings
            .get("directory")
            .cloned()
            .unwrap_or_else(|| "LifeTracker".to_string())
    }

    /// 获取当前同步状态
    pub fn get_status(&self) -> SyncStatus {
        let status = self.status.lock().unwrap();
        status.clone()
    }

    /// 获取最后一次同步结果
    pub fn get_last_result(&self) -> Option<SyncResult> {
        let result = self.last_result.lock().unwrap();
        result.clone()
    }

    /// 停止同步
    pub fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        *running = false;

        let mut status = self.status.lock().unwrap();
        *status = SyncStatus::Idle;

        log::info!("同步已停止");
        Ok(())
    }

    /// 启动自动同步
    pub async fn start_auto_sync(&self) -> Result<()> {
        if !self.config.auto_sync {
            return Ok(());
        }

        let mut running = self.running.lock().unwrap();
        *running = true;
        drop(running);

        let interval_duration = Duration::from_secs(self.config.interval as u64 * 60);
        let mut interval_timer = interval(interval_duration);

        log::info!("启动自动同步，间隔: {} 分钟", self.config.interval);

        loop {
            {
                let running = self.running.lock().unwrap();
                if !*running {
                    break;
                }
            }

            interval_timer.tick().await;

            // 执行同步
            if let Err(e) = self.sync().await {
                log::error!("自动同步失败: {}", e);
            }
        }

        log::info!("自动同步已停止");
        Ok(())
    }
}

impl DataSerializer {
    /// 创建新的数据序列化器
    pub fn new(storage: Arc<StorageManager>) -> Self {
        Self { storage }
    }

    /// 序列化所有数据
    pub async fn serialize_all_data(&self) -> Result<Vec<u8>> {
        log::info!("序列化所有应用数据");

        // 获取所有数据
        let tasks = self.storage.get_database().get_all_tasks()?;
        let categories = self.storage.get_database().get_all_categories()?;
        let time_entries = self.storage.get_database().get_all_time_entries()?;
        let transactions = self.storage.get_database().get_all_transactions()?;
        let accounts = self.storage.get_database().get_all_accounts()?;

        // 创建导出数据结构
        let export_data = serde_json::json!({
            "tasks": tasks,
            "categories": categories,
            "time_entries": time_entries,
            "transactions": transactions,
            "accounts": accounts,
            "export_time": Local::now(),
            "version": env!("CARGO_PKG_VERSION")
        });

        // 序列化为JSON
        let json_data = serde_json::to_vec(&export_data)?;

        log::info!("数据序列化完成，大小: {} 字节", json_data.len());
        Ok(json_data)
    }

    /// 导入数据
    pub async fn import_data(&self, data: &[u8]) -> Result<()> {
        log::info!("导入数据，大小: {} 字节", data.len());

        // 解析JSON数据
        let import_data: serde_json::Value = serde_json::from_slice(data)?;

        // 验证数据格式
        if !import_data.is_object() {
            return Err(AppError::Sync("导入数据格式无效".to_string()));
        }

        // 这里应该实现具体的数据导入逻辑
        // 由于涉及数据库操作，需要谨慎处理，避免数据丢失

        log::warn!("数据导入功能尚未完全实现");

        // TODO: 实现具体的导入逻辑
        // 1. 备份当前数据
        // 2. 清空相关表
        // 3. 导入新数据
        // 4. 验证数据完整性

        Ok(())
    }

    /// 验证数据完整性
    pub fn validate_data(&self, data: &[u8]) -> Result<bool> {
        // 尝试解析数据
        match serde_json::from_slice::<serde_json::Value>(data) {
            Ok(parsed) => {
                // 检查必要字段
                if parsed.get("tasks").is_some()
                    && parsed.get("categories").is_some()
                    && parsed.get("time_entries").is_some()
                {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }
}

/// 默认事件监听器
pub struct DefaultSyncEventListener;

impl SyncEventListener for DefaultSyncEventListener {
    fn on_sync_event(&self, event: SyncEvent) {
        match event {
            SyncEvent::Started => log::info!("同步开始"),
            SyncEvent::UploadStarted { file } => log::info!("开始上传: {}", file),
            SyncEvent::UploadCompleted { file } => log::info!("上传完成: {}", file),
            SyncEvent::DownloadStarted { file } => log::info!("开始下载: {}", file),
            SyncEvent::DownloadCompleted { file } => log::info!("下载完成: {}", file),
            SyncEvent::Progress { current, total } => {
                log::info!("同步进度: {}/{}", current, total)
            }
            SyncEvent::ConflictDetected { file } => log::warn!("发现冲突: {}", file),
            SyncEvent::Completed { result } => {
                log::info!(
                    "同步完成 - 成功: {}, 耗时: {:?}",
                    result.success,
                    result.duration()
                )
            }
            SyncEvent::Failed { error } => log::error!("同步失败: {}", error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_data_serializer() {
        // 创建临时数据库
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db_config = crate::storage::DatabaseConfig {
            database_path: db_path.to_string_lossy().to_string(),
            ..Default::default()
        };

        let storage = Arc::new(StorageManager::new(db_config).unwrap());
        storage.initialize().unwrap();

        let serializer = DataSerializer::new(storage);

        // 测试序列化
        let data = serializer.serialize_all_data().await.unwrap();
        assert!(!data.is_empty());

        // 测试验证
        assert!(serializer.validate_data(&data).unwrap());
    }

    #[test]
    fn test_sync_metadata() {
        let metadata = SyncMetadata {
            path: "/test/file.txt".to_string(),
            size: 1024,
            modified: Local::now(),
            hash: "abcd1234".to_string(),
            is_directory: false,
        };

        assert_eq!(metadata.path, "/test/file.txt");
        assert_eq!(metadata.size, 1024);
        assert!(!metadata.is_directory);
    }
}
