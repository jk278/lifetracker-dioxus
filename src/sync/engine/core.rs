//! # 同步引擎核心模块
//!
//! 实现数据同步的核心逻辑和管理

use super::{
    types::*, ConflictResolver, DataComparator, DataMerger, DataSerializer, DataValidator,
};
use crate::errors::{AppError, Result};
use crate::storage::StorageManager;
use crate::sync::{
    ConflictStrategy, SyncConfig, SyncDirection, SyncEvent, SyncEventListener, SyncItem,
    SyncResult, SyncStatus,
};
use chrono::{DateTime, Local};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::interval;

/// 同步引擎
pub struct SyncEngine {
    /// 存储管理器
    storage: Arc<StorageManager>,
    /// 同步配置
    config: SyncConfig,
    /// 同步提供者
    provider: Option<Box<dyn crate::sync::SyncProvider>>,
    /// 事件监听器
    listeners: Vec<Box<dyn SyncEventListener>>,
    /// 当前同步状态
    status: Arc<Mutex<SyncStatus>>,
    /// 最后一次同步结果
    last_result: Arc<Mutex<Option<SyncResult>>>,
    /// 是否正在运行
    running: Arc<Mutex<bool>>,
    /// 数据比较器
    comparator: DataComparator,
    /// 冲突解决器
    conflict_resolver: ConflictResolver,
    /// 数据合并器
    merger: DataMerger,
    /// 数据序列化器
    serializer: DataSerializer,
    /// 数据验证器
    validator: DataValidator,
}

impl SyncEngine {
    /// 创建新的同步引擎
    pub fn new(storage: Arc<StorageManager>, config: SyncConfig) -> Result<Self> {
        let _storage_clone = storage.clone();

        Ok(Self {
            storage: storage.clone(),
            config,
            provider: None,
            listeners: Vec::new(),
            status: Arc::new(Mutex::new(SyncStatus::Idle)),
            last_result: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            comparator: DataComparator::new(storage.clone()),
            conflict_resolver: ConflictResolver::new(storage.clone()),
            merger: DataMerger::new(storage.clone()),
            serializer: DataSerializer::new(storage.clone()),
            validator: DataValidator::new(),
        })
    }

    /// 初始化同步引擎
    pub async fn initialize(&mut self) -> Result<()> {
        log::info!("初始化同步引擎");

        // 验证配置
        super::super::validate_sync_config(&self.config)?;

        // 创建同步提供者
        self.provider = Some(crate::sync::providers::create_provider(&self.config).await?);

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
        let local_data = self.serializer.serialize_all_data().await?;

        // 获取远程数据列表
        let provider = self.provider.as_ref().unwrap();
        let remote_directory = self.get_remote_directory();

        // 确保远程目录存在
        provider.create_remote_directory(&remote_directory).await?;

        // 获取远程文件列表
        let remote_files = provider.list_remote_files(&remote_directory).await?;

        // 比较本地和远程数据
        let (upload_items, download_items, conflicts) = self
            .comparator
            .compare_data(
                &local_data,
                &remote_files,
                &remote_directory,
                provider.as_ref(),
            )
            .await?;

        // 处理冲突
        let mut resolved_conflicts = Vec::new();
        if !conflicts.is_empty() {
            log::info!("发现 {} 个冲突项", conflicts.len());

            for conflict in &conflicts {
                result.add_conflict(conflict.clone());
            }

            // 根据冲突策略处理
            match self.config.conflict_strategy {
                ConflictStrategy::Manual => {
                    // 手动处理，暂停同步等待用户选择
                    {
                        let mut status = self.status.lock().unwrap();
                        *status = SyncStatus::ConflictPending;
                    }
                    log::info!("需要手动解决冲突，同步暂停");
                    return Ok(result);
                }
                _ => {
                    // 自动解决冲突
                    resolved_conflicts = self
                        .conflict_resolver
                        .handle_conflicts(
                            &conflicts,
                            &self.config.conflict_strategy,
                            provider.as_ref(),
                        )
                        .await?;
                    log::info!("自动解决了 {} 个冲突", resolved_conflicts.len());
                }
            }
        }

        // 合并解决的冲突项到上传/下载队列
        let mut final_upload_items = upload_items;
        let mut final_download_items = download_items;

        for resolved_item in resolved_conflicts {
            match resolved_item.direction {
                SyncDirection::Upload => final_upload_items.push(resolved_item),
                SyncDirection::Download => final_download_items.push(resolved_item),
                _ => {}
            }
        }

        // 计算总操作数
        let total_operations = final_upload_items.len() + final_download_items.len();
        log::info!(
            "开始同步: {} 个上传项, {} 个下载项",
            final_upload_items.len(),
            final_download_items.len()
        );

        // 上传文件
        for (index, item) in final_upload_items.iter().enumerate() {
            self.emit_event(SyncEvent::UploadStarted {
                file: item.name.clone(),
            });

            match self.upload_item(item, &local_data).await {
                Ok(_) => {
                    result.uploaded_count += 1;
                    result.total_bytes += item.size;
                    self.emit_event(SyncEvent::UploadCompleted {
                        file: item.name.clone(),
                    });
                    log::info!("上传成功: {}", item.name);
                }
                Err(e) => {
                    log::error!("上传文件失败 {}: {}", item.name, e);
                    result.failed_count += 1;
                    result.add_error(format!("上传 {} 失败: {}", item.name, e));
                }
            }

            // 更新进度
            let current = index + 1;
            self.emit_event(SyncEvent::Progress {
                current: current as u32,
                total: total_operations as u32,
            });
        }

        // 下载文件
        for (index, item) in final_download_items.iter().enumerate() {
            self.emit_event(SyncEvent::DownloadStarted {
                file: item.name.clone(),
            });

            match self.download_item(item).await {
                Ok(_) => {
                    result.downloaded_count += 1;
                    result.total_bytes += item.size;
                    self.emit_event(SyncEvent::DownloadCompleted {
                        file: item.name.clone(),
                    });
                    log::info!("下载成功: {}", item.name);
                }
                Err(e) => {
                    log::error!("下载文件失败 {}: {}", item.name, e);
                    result.failed_count += 1;
                    result.add_error(format!("下载 {} 失败: {}", item.name, e));
                }
            }

            // 更新进度
            let current = final_upload_items.len() + index + 1;
            self.emit_event(SyncEvent::Progress {
                current: current as u32,
                total: total_operations as u32,
            });
        }

        // 验证同步结果
        if result.failed_count == 0 {
            log::info!("同步验证开始");
            if let Err(e) = self.verify_sync_result(&result).await {
                log::warn!("同步验证失败: {}", e);
                result.add_error(format!("同步验证失败: {}", e));
            }
        }

        result.complete(result.failed_count == 0 && result.errors.is_empty());

        log::info!(
            "同步操作完成 - 成功: {}, 上传: {}, 下载: {}, 失败: {}",
            result.success,
            result.uploaded_count,
            result.downloaded_count,
            result.failed_count
        );

        Ok(result)
    }

    /// 验证同步结果
    async fn verify_sync_result(&self, result: &SyncResult) -> Result<()> {
        log::info!("验证同步结果");

        // 检查数据完整性
        let local_data = self.serializer.serialize_all_data().await?;
        let local_json: serde_json::Value = serde_json::from_slice(&local_data)?;

        let integrity_report = self.validator.verify_data_integrity(&local_json).await?;

        if !integrity_report.is_valid {
            return Err(AppError::Sync(format!(
                "数据完整性验证失败: {:?}",
                integrity_report.errors
            )));
        }

        // 检查同步统计
        if result.uploaded_count == 0 && result.downloaded_count == 0 && result.failed_count == 0 {
            log::info!("无需同步，数据已是最新");
        } else {
            log::info!(
                "同步验证通过: 上传{}, 下载{}, 失败{}",
                result.uploaded_count,
                result.downloaded_count,
                result.failed_count
            );
        }

        Ok(())
    }

    /// 创建增量同步策略
    async fn create_incremental_sync_strategy(&self) -> Result<SyncStrategy> {
        let last_sync_time = self.get_last_sync_time()?;

        Ok(SyncStrategy {
            strategy_type: SyncStrategyType::Incremental,
            last_sync_time,
            conflict_resolution: self.config.conflict_strategy.clone(),
            compression_enabled: self.config.compression,
            max_file_size: self.config.max_file_size,
        })
    }

    /// 获取上次同步时间
    fn get_last_sync_time(&self) -> Result<Option<DateTime<Local>>> {
        if let Some(last_result) = self.get_last_result() {
            Ok(Some(last_result.end_time))
        } else {
            Ok(None)
        }
    }

    /// 执行增量同步
    async fn perform_incremental_sync(&self, strategy: &SyncStrategy) -> Result<SyncResult> {
        log::info!("执行增量同步");

        if let Some(last_sync_time) = strategy.last_sync_time {
            // 创建增量备份
            let incremental_data = self
                .serializer
                .create_incremental_backup(last_sync_time)
                .await?;

            // 上传增量数据
            let incremental_item = SyncItem {
                id: "incremental_data".to_string(),
                name: format!("incremental_{}.json", Local::now().format("%Y%m%d_%H%M%S")),
                local_path: "local".to_string(),
                remote_path: format!(
                    "{}/incremental_{}.json",
                    self.get_remote_directory(),
                    Local::now().format("%Y%m%d_%H%M%S")
                ),
                size: incremental_data.len() as u64,
                local_modified: Local::now(),
                remote_modified: None,
                hash: self.calculate_hash(&incremental_data),
                status: SyncStatus::Idle,
                direction: SyncDirection::Upload,
            };

            let mut result = SyncResult::new();

            match self.upload_item(&incremental_item, &incremental_data).await {
                Ok(_) => {
                    result.uploaded_count = 1;
                    result.total_bytes = incremental_data.len() as u64;
                    result.complete(true);
                }
                Err(e) => {
                    result.add_error(format!("增量同步失败: {}", e));
                    result.complete(false);
                }
            }

            Ok(result)
        } else {
            // 如果没有上次同步时间，执行完整同步
            self.perform_sync().await
        }
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

        // 解析数据以获取远程哈希
        let remote_data: serde_json::Value = serde_json::from_slice(&decompressed_data)?;
        let remote_content_hash = {
            let content = self
                .serializer
                .extract_content_for_comparison(&remote_data)?;
            self.serializer.calculate_content_hash(&content)
        };

        // 导入数据到本地存储，并更新来源追踪
        self.serializer.import_data(&decompressed_data).await?;

        // 更新来源追踪信息，标记为从远程下载
        self.serializer
            .update_origin_tracking(&remote_data, Some(&remote_content_hash))
            .await?;

        Ok(())
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

    /// 获取冲突详情
    pub async fn get_conflict_details(
        &self,
        conflicts: &[SyncItem],
    ) -> Result<Vec<super::conflict_resolver::ConflictDetails>> {
        let provider = self.provider.as_ref().unwrap();
        self.conflict_resolver
            .get_conflict_details(conflicts, provider.as_ref())
            .await
    }

    /// 手动解决冲突
    pub async fn resolve_conflicts_manually(
        &self,
        _resolutions: &[(String, super::types::ConflictResolution)],
    ) -> Result<Vec<SyncItem>> {
        // 这里可以实现手动冲突解决逻辑
        // 目前返回空列表
        Ok(Vec::new())
    }

    /// 执行智能合并
    pub async fn smart_merge(
        &self,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
        config: &MergeConfig,
    ) -> Result<serde_json::Value> {
        self.merger
            .smart_merge(local_data, remote_data, config)
            .await
    }

    /// 验证数据一致性
    pub async fn validate_data_consistency(&self, data: &serde_json::Value) -> Result<Vec<String>> {
        self.validator.validate_consistency(data)
    }

    /// 导出特定类型的数据
    pub async fn export_data_by_type(&self, data_types: &[&str]) -> Result<Vec<u8>> {
        self.serializer.export_data_by_type(data_types).await
    }

    /// 获取同步统计信息
    pub fn get_sync_statistics(&self) -> SyncStatistics {
        let last_result = self.get_last_result();

        SyncStatistics {
            total_syncs: 1, // 这里需要从存储中获取实际统计
            successful_syncs: if last_result.as_ref().map_or(false, |r| r.success) {
                1
            } else {
                0
            },
            failed_syncs: if last_result.as_ref().map_or(false, |r| !r.success) {
                1
            } else {
                0
            },
            last_sync_time: last_result.as_ref().map(|r| r.end_time),
            total_bytes_uploaded: last_result.as_ref().map_or(0, |r| r.total_bytes),
            total_bytes_downloaded: 0,   // 需要跟踪下载字节数
            total_conflicts_resolved: 0, // 需要跟踪解决的冲突数
        }
    }
}

/// 同步统计信息
#[derive(Debug, Clone)]
pub struct SyncStatistics {
    /// 总同步次数
    pub total_syncs: u64,
    /// 成功同步次数
    pub successful_syncs: u64,
    /// 失败同步次数
    pub failed_syncs: u64,
    /// 最后同步时间
    pub last_sync_time: Option<DateTime<Local>>,
    /// 总上传字节数
    pub total_bytes_uploaded: u64,
    /// 总下载字节数
    pub total_bytes_downloaded: u64,
    /// 总解决冲突数
    pub total_conflicts_resolved: u64,
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
