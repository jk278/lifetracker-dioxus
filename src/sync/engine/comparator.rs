//! # 数据比较模块
//!
//! 负责比较本地和远程数据，检测数据差异和变化

use super::integrity_checker::{ConflictDetectionResult, DataIntegrityChecker, RiskLevel};
use super::types::*;
use crate::errors::{AppError, Result};
use crate::storage::StorageManager;
use crate::sync::SyncItem;
use chrono::{DateTime, Local};
use std::sync::Arc;

/// 数据比较器
pub struct DataComparator {
    storage: Arc<StorageManager>,
    integrity_checker: DataIntegrityChecker,
}

impl DataComparator {
    /// 创建新的数据比较器
    pub fn new(storage: Arc<StorageManager>) -> Self {
        Self {
            storage,
            integrity_checker: DataIntegrityChecker::new(),
        }
    }

    /// 比较本地和远程数据
    pub async fn compare_data(
        &self,
        local_data: &[u8],
        remote_files: &[SyncItem],
        remote_directory: &str,
        provider: &dyn crate::sync::SyncProvider,
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
            remote_path: format!("{}/data.json", remote_directory),
            size: local_data.len() as u64,
            local_modified: Local::now(),
            remote_modified: None,
            hash: local_hash,
            status: crate::sync::SyncStatus::Idle,
            direction: crate::sync::SyncDirection::Upload,
        };

        // 优先检查本地数据是否为空
        let local_is_empty = self.is_empty_data(&local_json);

        // 查找远程的数据文件
        if let Some(remote_item) = remote_files.iter().find(|item| item.name == "data.json") {
            if local_is_empty {
                // 本地数据为空，直接下载远程数据
                log::info!("本地数据为空，直接从远程下载");
                download_items.push(remote_item.clone());
            } else {
                // 本地有数据，进行正常比较
                let comparison_result = self
                    .compare_data_content(&local_json, remote_item, provider)
                    .await?;

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
                    DataComparisonResult::NeedsMerge => {
                        log::info!("检测到需要合并的数据（本地新数据 + 远程数据存在）");
                        conflicts.push(local_item); // 暂时作为冲突处理
                    }
                    DataComparisonResult::Same => {
                        log::info!("数据已同步，无需操作");
                    }
                }
            }
        } else {
            // 远程不存在
            if local_is_empty {
                log::info!("本地和远程都没有数据，无需操作");
            } else {
                log::info!("远程数据不存在，需要上传");
                upload_items.push(local_item);
            }
        }

        Ok((upload_items, download_items, conflicts))
    }

    /// 比较数据内容
    pub async fn compare_data_content(
        &self,
        local_data: &serde_json::Value,
        remote_item: &SyncItem,
        provider: &dyn crate::sync::SyncProvider,
    ) -> Result<DataComparisonResult> {
        log::info!("比较本地和远程数据内容");

        // 检查本地数据是否为空
        if self.is_empty_data(local_data) {
            log::info!("本地数据为空，需要从远程下载");
            return Ok(DataComparisonResult::RemoteNewer);
        }

        // 首先尝试下载远程数据进行内容比较
        let remote_data = match self
            .download_and_parse_remote_data(remote_item, provider)
            .await
        {
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

        // ========== 使用完整性检查器进行深度分析 ==========
        log::info!("开始使用完整性检查器进行深度冲突分析");

        let conflict_result = self
            .integrity_checker
            .detect_conflicts(local_data, &remote_data)?;

        // 根据完整性检查结果决定同步策略
        if !conflict_result.has_conflict {
            log::info!("完整性检查通过，数据可以安全同步");

            // 比较数据内容（排除时间戳字段）
            let local_content = self.extract_content_for_comparison(local_data)?;
            let remote_content = self.extract_content_for_comparison(&remote_data)?;
            let local_hash = self.calculate_content_hash(&local_content);
            let remote_hash = self.calculate_content_hash(&remote_content);

            if local_hash == remote_hash {
                log::info!("数据内容相同，无需同步");
                Ok(DataComparisonResult::Same)
            } else {
                // 进行时间戳比较
                self.compare_by_timestamp_with_data(local_data, &remote_data)
            }
        } else {
            log::warn!("完整性检查发现冲突: {:?}", conflict_result.conflict_type);
            log::warn!("风险级别: {:?}", conflict_result.risk_assessment.risk_level);
            log::warn!("用户消息: {}", conflict_result.user_message);

            // 根据风险级别决定处理策略
            match conflict_result.risk_assessment.risk_level {
                RiskLevel::Safe => {
                    log::info!("风险级别为安全，继续正常同步流程");
                    self.compare_by_timestamp_with_data(local_data, &remote_data)
                }
                RiskLevel::NeedsConfirmation => {
                    log::info!("需要用户确认，标记为需要合并");
                    Ok(DataComparisonResult::NeedsMerge)
                }
                RiskLevel::HighRisk | RiskLevel::Dangerous => {
                    log::warn!("检测到高风险或危险操作，强制标记为冲突");
                    // 保存冲突详情到全局状态
                    self.save_conflict_details(&conflict_result).await?;
                    Ok(DataComparisonResult::Conflict)
                }
            }
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
        provider: &dyn crate::sync::SyncProvider,
    ) -> Result<serde_json::Value> {
        log::info!("下载远程数据进行比较: {}", remote_item.name);

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

    /// 检测本地数据的来源类型
    pub async fn detect_data_origin(
        &self,
        local_data: &serde_json::Value,
        remote_hash: &str,
    ) -> Result<DataOrigin> {
        log::info!("检测本地数据来源");

        // 1. 检查本地数据中的来源追踪字段
        if let Some(base_remote_hash) = local_data.get("base_remote_hash").and_then(|v| v.as_str())
        {
            if !base_remote_hash.is_empty() && base_remote_hash == remote_hash {
                log::info!("本地数据基于当前远程版本，来源：BasedOnRemote");
                return Ok(DataOrigin::BasedOnRemote);
            } else if !base_remote_hash.is_empty() {
                log::info!("本地数据基于不同的远程版本，来源：Unknown");
                return Ok(DataOrigin::Unknown);
            }
        }

        // 2. 检查存储中的来源追踪信息
        let stored_base_hash = self.get_base_remote_hash().await.unwrap_or_default();
        if !stored_base_hash.is_empty() {
            if stored_base_hash == remote_hash {
                log::info!("存储记录显示本地数据基于当前远程版本，来源：BasedOnRemote");
                return Ok(DataOrigin::BasedOnRemote);
            } else {
                log::info!("存储记录显示本地数据基于不同远程版本，来源：Unknown");
                return Ok(DataOrigin::Unknown);
            }
        }

        // 3. 检查是否为新安装后的数据
        if self.is_fresh_install().await? {
            log::info!("检测到新安装后首次创建的数据，来源：Fresh");
            return Ok(DataOrigin::Fresh);
        }

        // 4. 检查数据中的新安装标记
        if let Some(is_fresh) = local_data.get("is_fresh_install").and_then(|v| v.as_bool()) {
            if is_fresh {
                log::info!("数据标记为新安装创建，来源：Fresh");
                return Ok(DataOrigin::Fresh);
            }
        }

        // 5. 默认情况
        log::warn!("无法确定数据来源，标记为：Unknown");
        Ok(DataOrigin::Unknown)
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

        // 检查分类数量
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

        // 检查数据文件大小，如果很小（如148字节）也认为是空数据
        let data_size = data.to_string().len();
        let is_minimal_data = data_size < 500; // 小于500字节认为是空数据

        // 判断是否为空数据的条件：
        // 1. 所有主要业务数据都为空
        // 2. 或者数据文件很小（只包含基本结构）
        let is_empty =
            (task_count == 0 && entry_count == 0 && account_count == 0 && transaction_count == 0)
                || is_minimal_data;

        log::info!(
            "数据统计 - 任务: {}, 时间记录: {}, 分类: {}, 账户: {}, 交易: {}, 数据大小: {} 字节, 判断为空: {}",
            task_count,
            entry_count,
            category_count,
            account_count,
            transaction_count,
            data_size,
            is_empty
        );

        is_empty
    }

    /// 计算数据哈希
    fn calculate_hash(&self, data: &[u8]) -> String {
        let crypto = crate::utils::crypto::CryptoManager::new();
        crypto.calculate_hash(data)
    }

    /// 获取本地数据基于的远程版本哈希
    async fn get_base_remote_hash(&self) -> Result<String> {
        // 从存储中读取上次同步时的远程数据哈希
        match self.storage.get_database().get_setting("base_remote_hash") {
            Ok(Some(hash)) => Ok(hash),
            _ => Ok(String::new()),
        }
    }

    /// 检查是否为新安装后首次创建的数据
    async fn is_fresh_install(&self) -> Result<bool> {
        // 检查是否从未进行过同步
        let has_synced = self
            .storage
            .get_database()
            .get_setting("has_synced")
            .unwrap_or(None)
            .map(|v| v == "true")
            .unwrap_or(false);

        // 检查是否有数据但没有远程哈希
        let has_data = self.has_local_data().await?;
        let has_base_hash = !self.get_base_remote_hash().await?.is_empty();

        // 如果有数据但没有远程哈希记录，且从未同步过，则认为是新安装后的数据
        Ok(has_data && !has_base_hash && !has_synced)
    }

    /// 检查是否有本地数据
    async fn has_local_data(&self) -> Result<bool> {
        let tasks = self.storage.get_database().get_all_tasks()?;
        let time_entries = self.storage.get_database().get_all_time_entries()?;
        let transactions = self.storage.get_database().get_all_transactions()?;

        Ok(!tasks.is_empty() || !time_entries.is_empty() || !transactions.is_empty())
    }

    /// 保存冲突详情到全局状态
    async fn save_conflict_details(&self, conflict_result: &ConflictDetectionResult) -> Result<()> {
        use crate::tauri_commands::sync::conflicts::PENDING_CONFLICTS;
        use crate::tauri_commands::sync::types::ConflictItem;

        // 创建冲突项
        let conflict_item = ConflictItem {
            id: "data_integrity_conflict".to_string(),
            name: "数据完整性冲突".to_string(),
            local_modified: chrono::Local::now().to_rfc3339(),
            remote_modified: Some(chrono::Local::now().to_rfc3339()),
            conflict_type: match conflict_result.conflict_type {
                super::integrity_checker::ConflictType::DataLossRisk => {
                    "data_loss_risk".to_string()
                }
                super::integrity_checker::ConflictType::DataVolumeConflict => {
                    "data_volume_conflict".to_string()
                }
                super::integrity_checker::ConflictType::DataIntegrityConflict => {
                    "data_integrity_conflict".to_string()
                }
                super::integrity_checker::ConflictType::StructuralConflict => {
                    "structural_conflict".to_string()
                }
                super::integrity_checker::ConflictType::TimestampConflict => {
                    "timestamp_conflict".to_string()
                }
                _ => "unknown_conflict".to_string(),
            },
            local_preview: serde_json::json!({
                "tasks": conflict_result.local_stats.tasks,
                "time_entries": conflict_result.local_stats.time_entries,
                "transactions": conflict_result.local_stats.transactions,
                "data_size": conflict_result.local_stats.data_size,
                "user_message": conflict_result.user_message
            }),
            remote_preview: serde_json::json!({
                "tasks": conflict_result.remote_stats.tasks,
                "time_entries": conflict_result.remote_stats.time_entries,
                "transactions": conflict_result.remote_stats.transactions,
                "data_size": conflict_result.remote_stats.data_size,
                "risk_level": match conflict_result.risk_assessment.risk_level {
                    RiskLevel::Safe => "safe",
                    RiskLevel::NeedsConfirmation => "needs_confirmation",
                    RiskLevel::HighRisk => "high_risk",
                    RiskLevel::Dangerous => "dangerous",
                }
            }),
            file_size: conflict_result.local_stats.data_size as u64,
            local_hash: format!(
                "{:x}",
                md5::compute(
                    serde_json::to_string(&conflict_result.local_stats).unwrap_or_default()
                )
            ),
            remote_hash: Some(format!(
                "{:x}",
                md5::compute(
                    serde_json::to_string(&conflict_result.remote_stats).unwrap_or_default()
                )
            )),
        };

        // 保存到全局状态
        let mut pending_conflicts = PENDING_CONFLICTS.lock().unwrap();
        pending_conflicts.clear(); // 清除旧的冲突
        pending_conflicts.push(conflict_item);

        log::info!(
            "已保存冲突详情到全局状态，冲突类型: {:?}",
            conflict_result.conflict_type
        );
        Ok(())
    }
}
