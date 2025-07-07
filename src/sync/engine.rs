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

/// 数据比较结果
#[derive(Debug, Clone, PartialEq)]
enum DataComparisonResult {
    /// 本地数据更新
    LocalNewer,
    /// 远程数据更新
    RemoteNewer,
    /// 数据冲突
    Conflict,
    /// 数据相同
    Same,
}

/// 冲突解决方案
#[derive(Debug, Clone, PartialEq)]
enum ConflictResolution {
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
struct DataIntegrityReport {
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
    fn new() -> Self {
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

    fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }
}

/// 同步策略类型
#[derive(Debug, Clone, PartialEq)]
enum SyncStrategyType {
    /// 完全同步
    Full,
    /// 增量同步
    Incremental,
}

/// 同步策略
#[derive(Debug, Clone)]
struct SyncStrategy {
    /// 策略类型
    pub strategy_type: SyncStrategyType,
    /// 上次同步时间
    pub last_sync_time: Option<DateTime<Local>>,
    /// 冲突解决策略
    pub conflict_resolution: ConflictStrategy,
    /// 是否启用压缩
    pub compression_enabled: bool,
    /// 最大文件大小
    pub max_file_size: u32,
}

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
            self.compare_data(&local_data, &remote_files).await?;

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
                    resolved_conflicts = self.handle_conflicts(&conflicts).await?;
                    log::info!("自动解决了 {} 个冲突", resolved_conflicts.len());
                }
            }
        }

        // 合并解决的冲突项到上传/下载队列
        let mut final_upload_items = upload_items;
        let mut final_download_items = download_items;

        for resolved_item in resolved_conflicts {
            match resolved_item.direction {
                super::SyncDirection::Upload => final_upload_items.push(resolved_item),
                super::SyncDirection::Download => final_download_items.push(resolved_item),
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
        let local_data = self.export_local_data().await?;
        let local_json: serde_json::Value = serde_json::from_slice(&local_data)?;

        let integrity_report = self.verify_data_integrity(&local_json).await?;

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
            let incremental_data = self.create_incremental_backup(last_sync_time).await?;

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
                direction: super::SyncDirection::Upload,
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
    async fn compare_data(
        &self,
        local_data: &[u8],
        remote_files: &[SyncItem],
    ) -> Result<(Vec<SyncItem>, Vec<SyncItem>, Vec<SyncItem>)> {
        let mut upload_items = Vec::new();
        let mut download_items = Vec::new();
        let mut conflicts = Vec::new();

        // 解析本地数据
        let local_json: serde_json::Value = serde_json::from_slice(local_data)?;
        let local_hash = self.calculate_hash(local_data);

        // 创建本地数据项
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
            // 比较数据差异
            let comparison_result = self.compare_data_content(&local_json, remote_item).await?;

            match comparison_result {
                DataComparisonResult::LocalNewer => {
                    log::info!("本地数据更新，需要上传");
                    upload_items.push(local_item);
                }
                DataComparisonResult::RemoteNewer => {
                    log::info!("远程数据更新，需要下载");
                    download_items.push(remote_item.clone());
                }
                DataComparisonResult::Conflict => {
                    log::warn!("发现数据冲突，需要处理");
                    conflicts.push(local_item);
                }
                DataComparisonResult::Same => {
                    log::info!("数据已同步，无需操作");
                }
            }
        } else {
            // 远程不存在，需要上传
            log::info!("远程数据不存在，需要上传");
            upload_items.push(local_item);
        }

        Ok((upload_items, download_items, conflicts))
    }

    /// 比较数据内容
    async fn compare_data_content(
        &self,
        local_data: &serde_json::Value,
        remote_item: &SyncItem,
    ) -> Result<DataComparisonResult> {
        log::info!("比较本地和远程数据内容");

        // 检查本地数据是否为空
        if self.is_empty_data(local_data) {
            log::info!("本地数据为空，需要从远程下载");
            return Ok(DataComparisonResult::RemoteNewer);
        }

        // 首先尝试下载远程数据进行内容比较
        let remote_data = match self.download_and_parse_remote_data(remote_item).await {
            Ok(data) => data,
            Err(e) => {
                log::warn!("无法下载远程数据进行比较: {}", e);
                // 如果无法下载远程数据，基于时间戳比较
                return self.compare_by_timestamp(local_data, remote_item);
            }
        };

        // 检查远程数据是否为空
        if self.is_empty_data(&remote_data) {
            log::info!("远程数据为空，需要上传本地数据");
            return Ok(DataComparisonResult::LocalNewer);
        }

        // 比较数据内容（排除时间戳字段）
        let local_content = self.extract_content_for_comparison(local_data)?;
        let remote_content = self.extract_content_for_comparison(&remote_data)?;

        // 计算内容哈希
        let local_hash = self.calculate_content_hash(&local_content);
        let remote_hash = self.calculate_content_hash(&remote_content);

        log::info!("本地数据哈希: {}", local_hash);
        log::info!("远程数据哈希: {}", remote_hash);

        if local_hash == remote_hash {
            log::info!("数据内容相同，无需同步");
            Ok(DataComparisonResult::Same)
        } else {
            log::info!("数据内容不同，需要同步");
            // 内容不同时，通过时间戳判断优先级
            self.compare_by_timestamp_with_data(local_data, &remote_data)
        }
    }

    /// 基于实际数据时间戳比较
    fn compare_by_timestamp_with_data(
        &self,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> Result<DataComparisonResult> {
        // 获取本地数据的时间戳
        let local_timestamp = self.get_data_timestamp(local_data)?;

        // 获取远程数据的时间戳
        let remote_timestamp = self.get_data_timestamp(&remote_data)?;

        log::info!(
            "本地数据时间戳: {}",
            local_timestamp.format("%Y-%m-%d %H:%M:%S")
        );
        log::info!(
            "远程数据时间戳: {}",
            remote_timestamp.format("%Y-%m-%d %H:%M:%S")
        );

        // 允许30秒的时间差异（考虑网络延迟和时钟偏差）
        let time_diff = if local_timestamp > remote_timestamp {
            local_timestamp.signed_duration_since(remote_timestamp)
        } else {
            remote_timestamp.signed_duration_since(local_timestamp)
        };

        if time_diff.num_seconds() <= 30 {
            log::info!(
                "时间戳差异很小（{} 秒），认为数据相同",
                time_diff.num_seconds()
            );
            Ok(DataComparisonResult::Same)
        } else if local_timestamp > remote_timestamp {
            log::info!("本地数据较新，需要上传");
            Ok(DataComparisonResult::LocalNewer)
        } else {
            log::info!("远程数据较新，需要下载");
            Ok(DataComparisonResult::RemoteNewer)
        }
    }

    /// 下载并解析远程数据
    async fn download_and_parse_remote_data(
        &self,
        remote_item: &SyncItem,
    ) -> Result<serde_json::Value> {
        log::info!("下载远程数据进行比较: {}", remote_item.name);

        let provider = self
            .provider
            .as_ref()
            .ok_or_else(|| AppError::Sync("同步提供者未设置".to_string()))?;

        // 下载远程文件
        let data = provider.download_file(remote_item).await?;

        // 解析JSON数据
        let json_data: serde_json::Value = serde_json::from_slice(&data)
            .map_err(|e| AppError::Sync(format!("解析远程数据失败: {}", e)))?;

        log::info!("成功下载并解析远程数据，大小: {} 字节", data.len());

        Ok(json_data)
    }

    /// 提取用于比较的内容（排除时间戳等元数据）
    fn extract_content_for_comparison(
        &self,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let mut content = data.clone();

        // 移除时间戳相关字段
        if let Some(obj) = content.as_object_mut() {
            obj.remove("export_time");
            obj.remove("import_time");
            obj.remove("sync_time");
        }

        Ok(content)
    }

    /// 计算内容哈希
    fn calculate_content_hash(&self, content: &serde_json::Value) -> String {
        let content_str = serde_json::to_string(content).unwrap_or_default();
        format!("{:x}", md5::compute(content_str.as_bytes()))
    }

    /// 获取数据时间戳
    fn get_data_timestamp(&self, data: &serde_json::Value) -> Result<DateTime<Local>> {
        if let Some(export_time) = data.get("export_time") {
            if let Some(time_str) = export_time.as_str() {
                return DateTime::parse_from_rfc3339(time_str)
                    .map(|dt| dt.with_timezone(&Local))
                    .map_err(|e| AppError::Sync(format!("时间解析失败: {}", e)));
            }
        }

        // 如果没有时间戳，使用当前时间
        Ok(Local::now())
    }

    /// 通过时间戳比较数据（回退方法）
    fn compare_by_timestamp(
        &self,
        local_data: &serde_json::Value,
        remote_item: &SyncItem,
    ) -> Result<DataComparisonResult> {
        // 获取本地数据的时间戳
        let local_timestamp = self.get_data_timestamp(local_data)?;

        // 获取远程数据的时间戳
        let remote_timestamp = remote_item.remote_modified.unwrap_or(Local::now());

        log::info!(
            "本地时间戳: {}",
            local_timestamp.format("%Y-%m-%d %H:%M:%S")
        );
        log::info!(
            "远程时间戳: {}",
            remote_timestamp.format("%Y-%m-%d %H:%M:%S")
        );

        // 允许30秒的时间差异（考虑网络延迟和时钟偏差）
        let time_diff = if local_timestamp > remote_timestamp {
            local_timestamp.signed_duration_since(remote_timestamp)
        } else {
            remote_timestamp.signed_duration_since(local_timestamp)
        };

        if time_diff.num_seconds() <= 30 {
            log::info!(
                "时间戳差异很小（{} 秒），认为数据相同",
                time_diff.num_seconds()
            );
            Ok(DataComparisonResult::Same)
        } else if local_timestamp > remote_timestamp {
            log::info!("本地数据较新，需要上传");
            Ok(DataComparisonResult::LocalNewer)
        } else {
            log::info!("远程数据较新，需要下载");
            Ok(DataComparisonResult::RemoteNewer)
        }
    }

    /// 处理数据冲突
    async fn handle_conflicts(&self, conflicts: &[SyncItem]) -> Result<Vec<SyncItem>> {
        let mut resolved_items = Vec::new();

        for conflict in conflicts {
            log::info!("处理冲突: {}", conflict.name);

            let resolution = self.resolve_conflict(conflict).await?;

            match resolution {
                ConflictResolution::UseLocal => {
                    let mut upload_item = conflict.clone();
                    upload_item.direction = super::SyncDirection::Upload;
                    resolved_items.push(upload_item);
                }
                ConflictResolution::UseRemote => {
                    let mut download_item = conflict.clone();
                    download_item.direction = super::SyncDirection::Download;
                    resolved_items.push(download_item);
                }
                ConflictResolution::Merge => {
                    // 合并数据
                    let merged_item = self.merge_conflicted_data(conflict).await?;
                    resolved_items.push(merged_item);
                }
                ConflictResolution::Skip => {
                    log::info!("跳过冲突项: {}", conflict.name);
                }
            }
        }

        Ok(resolved_items)
    }

    /// 解决冲突
    async fn resolve_conflict(&self, conflict: &SyncItem) -> Result<ConflictResolution> {
        match self.config.conflict_strategy {
            ConflictStrategy::Manual => {
                // 手动解决，这里应该有用户界面支持
                log::info!("需要手动解决冲突: {}", conflict.name);
                Ok(ConflictResolution::Skip)
            }
            ConflictStrategy::LocalWins => {
                log::info!("本地优先解决冲突: {}", conflict.name);
                Ok(ConflictResolution::UseLocal)
            }
            ConflictStrategy::RemoteWins => {
                log::info!("远程优先解决冲突: {}", conflict.name);
                Ok(ConflictResolution::UseRemote)
            }
            ConflictStrategy::KeepBoth => {
                log::info!("保留双方数据解决冲突: {}", conflict.name);
                Ok(ConflictResolution::Merge)
            }
        }
    }

    /// 合并冲突数据
    async fn merge_conflicted_data(&self, conflict: &SyncItem) -> Result<SyncItem> {
        log::info!("合并冲突数据: {}", conflict.name);

        // 这里应该实现具体的数据合并逻辑
        // 为了简化，我们返回本地数据
        let mut merged_item = conflict.clone();
        merged_item.direction = super::SyncDirection::Upload;
        merged_item.name = format!("{}_merged", conflict.name);

        Ok(merged_item)
    }

    /// 验证数据完整性
    async fn verify_data_integrity(&self, data: &serde_json::Value) -> Result<DataIntegrityReport> {
        let mut report = DataIntegrityReport::new();

        // 验证任务数据
        if let Some(tasks) = data.get("tasks") {
            if let Some(tasks_array) = tasks.as_array() {
                for (i, task) in tasks_array.iter().enumerate() {
                    if let Err(e) = self.validate_task_format(task) {
                        report.add_error(format!("任务 {} 格式错误: {}", i, e));
                    }
                }
                report.task_count = tasks_array.len();
            }
        }

        // 验证分类数据
        if let Some(categories) = data.get("categories") {
            if let Some(categories_array) = categories.as_array() {
                for (i, category) in categories_array.iter().enumerate() {
                    if let Err(e) = self.validate_category_format(category) {
                        report.add_error(format!("分类 {} 格式错误: {}", i, e));
                    }
                }
                report.category_count = categories_array.len();
            }
        }

        // 验证时间记录数据
        if let Some(time_entries) = data.get("time_entries") {
            if let Some(time_entries_array) = time_entries.as_array() {
                for (i, entry) in time_entries_array.iter().enumerate() {
                    if let Err(e) = self.validate_time_entry_format(entry) {
                        report.add_error(format!("时间记录 {} 格式错误: {}", i, e));
                    }
                }
                report.time_entry_count = time_entries_array.len();
            }
        }

        // 验证引用完整性
        self.verify_reference_integrity(data, &mut report)?;

        report.is_valid = report.errors.is_empty();

        Ok(report)
    }

    /// 验证引用完整性
    fn verify_reference_integrity(
        &self,
        data: &serde_json::Value,
        report: &mut DataIntegrityReport,
    ) -> Result<()> {
        // 收集所有分类ID
        let mut category_ids = std::collections::HashSet::new();
        if let Some(categories) = data.get("categories") {
            if let Some(categories_array) = categories.as_array() {
                for category in categories_array {
                    if let Some(id) = category.get("id").and_then(|v| v.as_str()) {
                        category_ids.insert(id.to_string());
                    }
                }
            }
        }

        // 验证时间记录中的分类引用
        if let Some(time_entries) = data.get("time_entries") {
            if let Some(time_entries_array) = time_entries.as_array() {
                for entry in time_entries_array {
                    if let Some(category_id) = entry.get("category_id").and_then(|v| v.as_str()) {
                        if !category_ids.contains(category_id) {
                            report.add_error(format!(
                                "时间记录引用了不存在的分类ID: {}",
                                category_id
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 计算数据哈希
    fn calculate_hash(&self, data: &[u8]) -> String {
        let crypto = crate::utils::crypto::CryptoManager::new();
        crypto.calculate_hash(data)
    }

    /// 检查数据是否为空（无有效记录）
    fn is_empty_data(&self, data: &serde_json::Value) -> bool {
        // 检查任务数量
        let task_count = data
            .get("tasks")
            .and_then(|v| v.as_array())
            .map_or(0, |arr| arr.len());

        // 检查时间记录数量
        let entry_count = data
            .get("time_entries")
            .and_then(|v| v.as_array())
            .map_or(0, |arr| arr.len());

        // 检查分类数量（排除默认分类）
        let category_count = data
            .get("categories")
            .and_then(|v| v.as_array())
            .map_or(0, |arr| arr.len());

        // 检查账户数量
        let account_count = data
            .get("accounts")
            .and_then(|v| v.as_array())
            .map_or(0, |arr| arr.len());

        // 检查交易数量
        let transaction_count = data
            .get("transactions")
            .and_then(|v| v.as_array())
            .map_or(0, |arr| arr.len());

        // 如果所有主要数据都为空，认为是空数据
        // 允许有少量分类（默认分类）
        let is_empty =
            task_count == 0 && entry_count == 0 && account_count == 0 && transaction_count == 0;

        log::info!(
            "数据统计 - 任务: {}, 时间记录: {}, 分类: {}, 账户: {}, 交易: {}, 判断为空: {}",
            task_count,
            entry_count,
            category_count,
            account_count,
            transaction_count,
            is_empty
        );

        is_empty
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

    /// 验证任务格式
    fn validate_task_format(&self, task: &serde_json::Value) -> Result<()> {
        if !task.is_object() {
            return Err(AppError::Sync("任务不是对象格式".to_string()));
        }

        // 检查必要字段
        let required_fields = ["id", "name", "created_at"];
        for field in &required_fields {
            if task.get(field).is_none() {
                return Err(AppError::Sync(format!("缺少必要字段: {}", field)));
            }
        }

        // 验证ID格式
        if let Some(id) = task.get("id") {
            if let Some(id_str) = id.as_str() {
                if uuid::Uuid::parse_str(id_str).is_err() {
                    return Err(AppError::Sync("ID格式无效".to_string()));
                }
            }
        }

        Ok(())
    }

    /// 验证分类格式
    fn validate_category_format(&self, category: &serde_json::Value) -> Result<()> {
        if !category.is_object() {
            return Err(AppError::Sync("分类不是对象格式".to_string()));
        }

        // 检查必要字段
        let required_fields = ["id", "name", "color", "created_at"];
        for field in &required_fields {
            if category.get(field).is_none() {
                return Err(AppError::Sync(format!("缺少必要字段: {}", field)));
            }
        }

        Ok(())
    }

    /// 验证时间记录格式
    fn validate_time_entry_format(&self, entry: &serde_json::Value) -> Result<()> {
        if !entry.is_object() {
            return Err(AppError::Sync("时间记录不是对象格式".to_string()));
        }

        // 检查必要字段
        let required_fields = ["id", "task_name", "start_time", "duration_seconds"];
        for field in &required_fields {
            if entry.get(field).is_none() {
                return Err(AppError::Sync(format!("缺少必要字段: {}", field)));
            }
        }

        // 验证持续时间
        if let Some(duration) = entry.get("duration_seconds") {
            if let Some(duration_num) = duration.as_i64() {
                if duration_num < 0 {
                    return Err(AppError::Sync("持续时间不能为负数".to_string()));
                }
            }
        }

        Ok(())
    }

    /// 创建增量备份
    async fn create_incremental_backup(&self, since: DateTime<Local>) -> Result<Vec<u8>> {
        log::info!(
            "创建增量备份，自 {} 以来的更改",
            since.format("%Y-%m-%d %H:%M:%S")
        );

        let db = self.storage.get_database();

        // 获取自指定时间以来的更改
        let changed_tasks = self.get_changed_tasks_since(since, db)?;
        let changed_categories = self.get_changed_categories_since(since, db)?;
        let changed_time_entries = self.get_changed_time_entries_since(since, db)?;
        let changed_accounts = self.get_changed_accounts_since(since, db)?;
        let changed_transactions = self.get_changed_transactions_since(since, db)?;

        // 创建增量备份数据
        let backup_data = serde_json::json!({
            "backup_type": "incremental",
            "backup_time": Local::now(),
            "since": since,
            "version": env!("CARGO_PKG_VERSION"),
            "tasks": changed_tasks,
            "categories": changed_categories,
            "time_entries": changed_time_entries,
            "accounts": changed_accounts,
            "transactions": changed_transactions
        });

        let backup_json = serde_json::to_vec(&backup_data)?;
        log::info!("增量备份创建完成，大小: {} 字节", backup_json.len());

        Ok(backup_json)
    }

    /// 获取自指定时间以来变更的任务
    fn get_changed_tasks_since(
        &self,
        since: DateTime<Local>,
        db: &crate::storage::Database,
    ) -> Result<Vec<serde_json::Value>> {
        // 这里需要实现具体的查询逻辑
        // 为了简化，我们返回所有任务
        let tasks = db.get_all_tasks()?;
        let tasks_json: Vec<serde_json::Value> = tasks
            .into_iter()
            .map(|task| serde_json::to_value(task).unwrap_or_default())
            .collect();
        Ok(tasks_json)
    }

    /// 获取自指定时间以来变更的分类
    fn get_changed_categories_since(
        &self,
        since: DateTime<Local>,
        db: &crate::storage::Database,
    ) -> Result<Vec<serde_json::Value>> {
        let categories = db.get_all_categories()?;
        let categories_json: Vec<serde_json::Value> = categories
            .into_iter()
            .map(|category| serde_json::to_value(category).unwrap_or_default())
            .collect();
        Ok(categories_json)
    }

    /// 获取自指定时间以来变更的时间记录
    fn get_changed_time_entries_since(
        &self,
        since: DateTime<Local>,
        db: &crate::storage::Database,
    ) -> Result<Vec<serde_json::Value>> {
        let time_entries = db.get_all_time_entries()?;
        let time_entries_json: Vec<serde_json::Value> = time_entries
            .into_iter()
            .map(|entry| serde_json::to_value(entry).unwrap_or_default())
            .collect();
        Ok(time_entries_json)
    }

    /// 获取自指定时间以来变更的账户
    fn get_changed_accounts_since(
        &self,
        since: DateTime<Local>,
        db: &crate::storage::Database,
    ) -> Result<Vec<serde_json::Value>> {
        let accounts = db.get_all_accounts()?;
        let accounts_json: Vec<serde_json::Value> = accounts
            .into_iter()
            .map(|account| serde_json::to_value(account).unwrap_or_default())
            .collect();
        Ok(accounts_json)
    }

    /// 获取自指定时间以来变更的交易
    fn get_changed_transactions_since(
        &self,
        since: DateTime<Local>,
        db: &crate::storage::Database,
    ) -> Result<Vec<serde_json::Value>> {
        let transactions = db.get_all_transactions()?;
        let transactions_json: Vec<serde_json::Value> = transactions
            .into_iter()
            .map(|transaction| serde_json::to_value(transaction).unwrap_or_default())
            .collect();
        Ok(transactions_json)
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
        log::info!("开始导入数据，大小: {} 字节", data.len());

        // 1. 验证数据格式
        self.validate_data(data)?;

        // 2. 解析JSON数据
        let import_data: serde_json::Value = serde_json::from_slice(data)?;
        if !import_data.is_object() {
            return Err(AppError::Sync("导入数据格式无效".to_string()));
        }

        // 3. 创建数据备份
        let backup_data = self.create_backup().await?;

        // 4. 开始事务
        let db = self.storage.get_database();
        if let Err(e) = db.get_connection()?.begin_transaction() {
            log::error!("开始事务失败: {}", e);
            return Err(AppError::Sync(format!("开始事务失败: {}", e)));
        }

        // 5. 执行导入
        let import_result = self.perform_import(&import_data).await;

        match import_result {
            Ok(_) => {
                // 6. 提交事务
                if let Err(e) = db.get_connection()?.commit_transaction() {
                    log::error!("提交事务失败: {}", e);
                    // 尝试回滚到备份数据
                    let _ = self.restore_from_backup(&backup_data).await;
                    return Err(AppError::Sync(format!("提交事务失败: {}", e)));
                }
                log::info!("数据导入成功");
                Ok(())
            }
            Err(e) => {
                // 7. 回滚事务
                log::error!("导入失败: {}", e);
                if let Err(rollback_err) = db.get_connection()?.rollback_transaction() {
                    log::error!("回滚事务失败: {}", rollback_err);
                }

                // 8. 恢复备份数据
                if let Err(restore_err) = self.restore_from_backup(&backup_data).await {
                    log::error!("恢复备份失败: {}", restore_err);
                }

                Err(AppError::Sync(format!("导入失败: {}", e)))
            }
        }
    }

    /// 创建数据备份
    async fn create_backup(&self) -> Result<Vec<u8>> {
        log::info!("创建数据备份");
        self.serialize_all_data().await
    }

    /// 从备份恢复数据
    async fn restore_from_backup(&self, backup_data: &[u8]) -> Result<()> {
        log::info!("开始从备份恢复数据，大小: {} 字节", backup_data.len());

        // 验证备份数据
        if backup_data.is_empty() {
            return Err(AppError::Sync("备份数据为空".to_string()));
        }

        // 解析备份数据
        let backup_json: serde_json::Value = serde_json::from_slice(backup_data)?;

        // 开始恢复事务
        let db = self.storage.get_database();
        db.get_connection()?.begin_transaction()?;

        match self.restore_all_data(&backup_json).await {
            Ok(_) => {
                db.get_connection()?.commit_transaction()?;
                log::info!("数据恢复成功");
                Ok(())
            }
            Err(e) => {
                log::error!("数据恢复失败: {}", e);
                db.get_connection()?.rollback_transaction()?;
                Err(AppError::Sync(format!("数据恢复失败: {}", e)))
            }
        }
    }

    /// 恢复所有数据
    async fn restore_all_data(&self, backup_data: &serde_json::Value) -> Result<()> {
        log::info!("恢复所有数据");

        let db = self.storage.get_database();

        // 首先清空当前数据
        self.clear_existing_data().await?;

        // 按正确的依赖顺序恢复数据
        // 1. 先恢复分类数据（被任务引用）
        if let Some(categories) = backup_data.get("categories") {
            self.import_categories(categories, db).await?;
        }

        // 2. 恢复账户数据（被交易引用）
        if let Some(accounts) = backup_data.get("accounts") {
            self.import_accounts(accounts, db).await?;
        }

        // 3. 恢复任务数据（引用分类）
        if let Some(tasks) = backup_data.get("tasks") {
            self.import_tasks(tasks, db).await?;
        }

        // 4. 恢复时间记录（引用任务）
        if let Some(time_entries) = backup_data.get("time_entries") {
            self.import_time_entries(time_entries, db).await?;
        }

        // 5. 恢复交易数据（引用账户）
        if let Some(transactions) = backup_data.get("transactions") {
            self.import_transactions(transactions, db).await?;
        }

        log::info!("所有数据恢复完成");
        Ok(())
    }

    /// 执行实际的数据导入
    async fn perform_import(&self, import_data: &serde_json::Value) -> Result<()> {
        log::info!("执行数据导入");

        let db = self.storage.get_database();

        // 清空现有数据 (谨慎操作)
        self.clear_existing_data().await?;

        // 按正确的依赖顺序导入数据
        // 1. 先导入分类数据（被任务引用）
        if let Some(categories) = import_data.get("categories") {
            self.import_categories(categories, db).await?;
        }

        // 2. 导入账户数据（被交易引用）
        if let Some(accounts) = import_data.get("accounts") {
            self.import_accounts(accounts, db).await?;
        }

        // 3. 导入任务数据（引用分类）
        if let Some(tasks) = import_data.get("tasks") {
            self.import_tasks(tasks, db).await?;
        }

        // 4. 导入时间记录（引用任务）
        if let Some(time_entries) = import_data.get("time_entries") {
            self.import_time_entries(time_entries, db).await?;
        }

        // 5. 导入交易数据（引用账户）
        if let Some(transactions) = import_data.get("transactions") {
            self.import_transactions(transactions, db).await?;
        }

        log::info!("数据导入完成");
        Ok(())
    }

    /// 清空现有数据
    async fn clear_existing_data(&self) -> Result<()> {
        log::info!("清空现有数据");

        let db = self.storage.get_database();
        let connection = db.get_connection()?;

        // 按依赖关系顺序删除
        connection.execute("DELETE FROM time_entries", &[])?;
        connection.execute("DELETE FROM transactions", &[])?;
        connection.execute("DELETE FROM tasks", &[])?;
        connection.execute("DELETE FROM categories", &[])?;
        connection.execute("DELETE FROM accounts", &[])?;

        log::info!("现有数据已清空");
        Ok(())
    }

    /// 导入任务数据
    async fn import_tasks(
        &self,
        tasks_data: &serde_json::Value,
        db: &crate::storage::Database,
    ) -> Result<()> {
        if let Some(tasks_array) = tasks_data.as_array() {
            for task_value in tasks_array {
                let task: crate::storage::TaskInsert = serde_json::from_value(task_value.clone())?;
                db.insert_task(&task)?;
            }
            log::info!("导入了 {} 个任务", tasks_array.len());
        }
        Ok(())
    }

    /// 导入分类数据
    async fn import_categories(
        &self,
        categories_data: &serde_json::Value,
        db: &crate::storage::Database,
    ) -> Result<()> {
        if let Some(categories_array) = categories_data.as_array() {
            for category_value in categories_array {
                let category: crate::storage::CategoryInsert =
                    serde_json::from_value(category_value.clone())?;
                db.insert_category(&category)?;
            }
            log::info!("导入了 {} 个分类", categories_array.len());
        }
        Ok(())
    }

    /// 导入时间记录
    async fn import_time_entries(
        &self,
        time_entries_data: &serde_json::Value,
        db: &crate::storage::Database,
    ) -> Result<()> {
        if let Some(time_entries_array) = time_entries_data.as_array() {
            for time_entry_value in time_entries_array {
                let time_entry: crate::storage::TimeEntryInsert =
                    serde_json::from_value(time_entry_value.clone())?;
                db.insert_time_entry(&time_entry)?;
            }
            log::info!("导入了 {} 个时间记录", time_entries_array.len());
        }
        Ok(())
    }

    /// 导入账户数据
    async fn import_accounts(
        &self,
        accounts_data: &serde_json::Value,
        db: &crate::storage::Database,
    ) -> Result<()> {
        if let Some(accounts_array) = accounts_data.as_array() {
            for account_value in accounts_array {
                let account: crate::storage::AccountInsert =
                    serde_json::from_value(account_value.clone())?;
                db.insert_account(&account)?;
            }
            log::info!("导入了 {} 个账户", accounts_array.len());
        }
        Ok(())
    }

    /// 导入交易数据
    async fn import_transactions(
        &self,
        transactions_data: &serde_json::Value,
        db: &crate::storage::Database,
    ) -> Result<()> {
        if let Some(transactions_array) = transactions_data.as_array() {
            for transaction_value in transactions_array {
                let transaction: crate::storage::TransactionInsert =
                    serde_json::from_value(transaction_value.clone())?;
                db.insert_transaction(&transaction)?;
            }
            log::info!("导入了 {} 个交易", transactions_array.len());
        }
        Ok(())
    }

    /// 验证数据完整性
    pub fn validate_data(&self, data: &[u8]) -> Result<bool> {
        log::info!("验证数据完整性，大小: {} 字节", data.len());

        // 1. 检查数据是否为空
        if data.is_empty() {
            return Err(AppError::Sync("导入数据为空".to_string()));
        }

        // 2. 检查数据大小是否合理
        if data.len() > 100 * 1024 * 1024 {
            // 100MB限制
            return Err(AppError::Sync("导入数据过大，超过100MB限制".to_string()));
        }

        // 3. 尝试解析JSON数据
        let parsed_data: serde_json::Value = serde_json::from_slice(data)
            .map_err(|e| AppError::Sync(format!("数据解析失败: {}", e)))?;

        // 4. 检查数据结构
        if !parsed_data.is_object() {
            return Err(AppError::Sync(
                "数据格式错误，不是有效的JSON对象".to_string(),
            ));
        }

        // 5. 验证版本信息
        if let Some(version) = parsed_data.get("version") {
            if let Some(version_str) = version.as_str() {
                if !self.is_compatible_version(version_str) {
                    return Err(AppError::Sync(format!("数据版本不兼容: {}", version_str)));
                }
            }
        }

        // 6. 验证导出时间
        if let Some(export_time) = parsed_data.get("export_time") {
            if export_time.as_str().is_none() {
                log::warn!("导出时间格式无效");
            }
        }

        // 7. 验证必要字段存在
        let required_fields = [
            "tasks",
            "categories",
            "time_entries",
            "transactions",
            "accounts",
        ];
        for field in &required_fields {
            if let Some(field_data) = parsed_data.get(field) {
                if !field_data.is_array() {
                    return Err(AppError::Sync(format!("字段 {} 不是数组格式", field)));
                }
            }
        }

        // 8. 验证数据记录数量
        let total_records = self.count_total_records(&parsed_data)?;
        if total_records > 1_000_000 {
            // 100万条记录限制
            return Err(AppError::Sync(format!(
                "数据记录过多: {} 条，超过100万条限制",
                total_records
            )));
        }

        // 9. 快速验证数据格式
        self.validate_data_format(&parsed_data)?;

        log::info!("数据验证通过，包含 {} 条记录", total_records);
        Ok(true)
    }

    /// 检查版本兼容性
    fn is_compatible_version(&self, version: &str) -> bool {
        // 当前版本
        let current_version = env!("CARGO_PKG_VERSION");

        // 简单的版本检查，实际应该使用更精确的版本比较
        if version == current_version {
            return true;
        }

        // 允许同一大版本的不同小版本
        let current_major = current_version.split('.').next().unwrap_or("0");
        let import_major = version.split('.').next().unwrap_or("0");

        current_major == import_major
    }

    /// 计算总记录数
    fn count_total_records(&self, data: &serde_json::Value) -> Result<usize> {
        let mut total = 0;

        if let Some(tasks) = data.get("tasks") {
            if let Some(tasks_array) = tasks.as_array() {
                total += tasks_array.len();
            }
        }

        if let Some(categories) = data.get("categories") {
            if let Some(categories_array) = categories.as_array() {
                total += categories_array.len();
            }
        }

        if let Some(time_entries) = data.get("time_entries") {
            if let Some(time_entries_array) = time_entries.as_array() {
                total += time_entries_array.len();
            }
        }

        if let Some(transactions) = data.get("transactions") {
            if let Some(transactions_array) = transactions.as_array() {
                total += transactions_array.len();
            }
        }

        if let Some(accounts) = data.get("accounts") {
            if let Some(accounts_array) = accounts.as_array() {
                total += accounts_array.len();
            }
        }

        Ok(total)
    }

    /// 验证数据格式
    fn validate_data_format(&self, data: &serde_json::Value) -> Result<()> {
        // 验证任务数据格式
        if let Some(tasks) = data.get("tasks") {
            if let Some(tasks_array) = tasks.as_array() {
                for (i, task) in tasks_array.iter().enumerate() {
                    if let Err(e) = self.validate_task_format(task) {
                        return Err(AppError::Sync(format!("任务 {} 格式错误: {}", i, e)));
                    }
                }
            }
        }

        // 验证分类数据格式
        if let Some(categories) = data.get("categories") {
            if let Some(categories_array) = categories.as_array() {
                for (i, category) in categories_array.iter().enumerate() {
                    if let Err(e) = self.validate_category_format(category) {
                        return Err(AppError::Sync(format!("分类 {} 格式错误: {}", i, e)));
                    }
                }
            }
        }

        // 验证时间记录数据格式
        if let Some(time_entries) = data.get("time_entries") {
            if let Some(time_entries_array) = time_entries.as_array() {
                for (i, entry) in time_entries_array.iter().enumerate() {
                    if let Err(e) = self.validate_time_entry_format(entry) {
                        return Err(AppError::Sync(format!("时间记录 {} 格式错误: {}", i, e)));
                    }
                }
            }
        }

        Ok(())
    }

    /// 验证任务格式
    fn validate_task_format(&self, task: &serde_json::Value) -> Result<()> {
        if !task.is_object() {
            return Err(AppError::Sync("任务不是对象格式".to_string()));
        }

        // 检查必要字段
        let required_fields = ["id", "name", "created_at"];
        for field in &required_fields {
            if task.get(field).is_none() {
                return Err(AppError::Sync(format!("缺少必要字段: {}", field)));
            }
        }

        // 验证ID格式
        if let Some(id) = task.get("id") {
            if let Some(id_str) = id.as_str() {
                if uuid::Uuid::parse_str(id_str).is_err() {
                    return Err(AppError::Sync("ID格式无效".to_string()));
                }
            }
        }

        Ok(())
    }

    /// 验证分类格式
    fn validate_category_format(&self, category: &serde_json::Value) -> Result<()> {
        if !category.is_object() {
            return Err(AppError::Sync("分类不是对象格式".to_string()));
        }

        // 检查必要字段
        let required_fields = ["id", "name", "color", "created_at"];
        for field in &required_fields {
            if category.get(field).is_none() {
                return Err(AppError::Sync(format!("缺少必要字段: {}", field)));
            }
        }

        Ok(())
    }

    /// 验证时间记录格式
    fn validate_time_entry_format(&self, entry: &serde_json::Value) -> Result<()> {
        if !entry.is_object() {
            return Err(AppError::Sync("时间记录不是对象格式".to_string()));
        }

        // 检查必要字段
        let required_fields = ["id", "task_name", "start_time", "duration_seconds"];
        for field in &required_fields {
            if entry.get(field).is_none() {
                return Err(AppError::Sync(format!("缺少必要字段: {}", field)));
            }
        }

        // 验证持续时间
        if let Some(duration) = entry.get("duration_seconds") {
            if let Some(duration_num) = duration.as_i64() {
                if duration_num < 0 {
                    return Err(AppError::Sync("持续时间不能为负数".to_string()));
                }
            }
        }

        Ok(())
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
