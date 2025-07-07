//! # 冲突解决器模块
//!
//! 负责处理数据同步时的冲突，提供多种冲突解决策略

use super::types::*;
use super::DataMerger;
use crate::errors::{AppError, Result};
use crate::storage::StorageManager;
use crate::sync::{ConflictStrategy, SyncDirection, SyncItem};
use chrono::Local;
use std::sync::Arc;

/// 冲突解决器
pub struct ConflictResolver {
    storage: Arc<StorageManager>,
}

impl ConflictResolver {
    /// 创建新的冲突解决器
    pub fn new(storage: Arc<StorageManager>) -> Self {
        Self { storage }
    }

    /// 处理数据冲突
    pub async fn handle_conflicts(
        &self,
        conflicts: &[SyncItem],
        conflict_strategy: &ConflictStrategy,
        provider: &dyn crate::sync::SyncProvider,
    ) -> Result<Vec<SyncItem>> {
        let mut resolved_items = Vec::new();

        for conflict in conflicts {
            log::info!("处理冲突: {}", conflict.name);

            let resolution = self.resolve_conflict(conflict, conflict_strategy).await?;

            match resolution {
                ConflictResolution::UseLocal => {
                    let mut upload_item = conflict.clone();
                    upload_item.direction = SyncDirection::Upload;
                    resolved_items.push(upload_item);
                }
                ConflictResolution::UseRemote => {
                    let mut download_item = conflict.clone();
                    download_item.direction = SyncDirection::Download;
                    resolved_items.push(download_item);
                }
                ConflictResolution::Merge => {
                    // 合并数据
                    let merged_item = self.merge_conflicted_data(conflict, provider).await?;
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
    async fn resolve_conflict(
        &self,
        conflict: &SyncItem,
        conflict_strategy: &ConflictStrategy,
    ) -> Result<ConflictResolution> {
        match conflict_strategy {
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
    async fn merge_conflicted_data(
        &self,
        conflict: &SyncItem,
        provider: &dyn crate::sync::SyncProvider,
    ) -> Result<SyncItem> {
        log::info!("合并冲突数据: {}", conflict.name);

        // 获取本地数据
        let local_data = self.export_local_data().await?;
        let local_json: serde_json::Value = serde_json::from_slice(&local_data)?;

        // 获取远程数据
        let remote_data = provider.download_file(conflict).await?;
        let remote_json: serde_json::Value = serde_json::from_slice(&remote_data)?;

        // 创建数据合并器
        let merger = DataMerger::new(self.storage.clone());

        // 执行数据合并
        let merged_data = merger.merge_data(&local_json, &remote_json).await?;
        let merged_bytes = serde_json::to_vec(&merged_data)?;

        // 创建合并后的数据项
        let mut merged_item = conflict.clone();
        merged_item.direction = SyncDirection::Upload;
        merged_item.size = merged_bytes.len() as u64;
        merged_item.hash = self.calculate_hash(&merged_bytes);
        merged_item.local_modified = Local::now();

        // 将合并后的数据导入本地存储
        let serializer = super::DataSerializer::new(self.storage.clone());
        serializer.import_data(&merged_bytes).await?;

        // 更新来源追踪信息
        serializer
            .update_origin_tracking(&merged_data, Some(&merged_item.hash))
            .await?;

        log::info!("数据合并完成，新哈希: {}", merged_item.hash);
        Ok(merged_item)
    }

    /// 导出本地数据
    async fn export_local_data(&self) -> Result<Vec<u8>> {
        let serializer = super::DataSerializer::new(self.storage.clone());
        serializer.serialize_all_data().await
    }

    /// 计算数据哈希
    fn calculate_hash(&self, data: &[u8]) -> String {
        let crypto = crate::utils::crypto::CryptoManager::new();
        crypto.calculate_hash(data)
    }

    /// 获取所有冲突项的详细信息
    pub async fn get_conflict_details(
        &self,
        conflicts: &[SyncItem],
        provider: &dyn crate::sync::SyncProvider,
    ) -> Result<Vec<ConflictDetails>> {
        let mut details = Vec::new();

        for conflict in conflicts {
            let local_data = self.export_local_data().await?;
            let local_json: serde_json::Value = serde_json::from_slice(&local_data)?;

            let remote_data = match provider.download_file(conflict).await {
                Ok(data) => match serde_json::from_slice::<serde_json::Value>(&data) {
                    Ok(json) => Some(json),
                    Err(_) => None,
                },
                Err(_) => None,
            };

            let local_timestamp = self.get_data_timestamp(&local_json).ok();
            let conflict_type = self.analyze_conflict_type(&local_json, &remote_data).await;

            let conflict_detail = ConflictDetails {
                item: conflict.clone(),
                local_data: local_json,
                remote_data,
                local_timestamp,
                remote_timestamp: conflict.remote_modified,
                conflict_type,
            };

            details.push(conflict_detail);
        }

        Ok(details)
    }

    /// 分析冲突类型
    async fn analyze_conflict_type(
        &self,
        local_data: &serde_json::Value,
        remote_data: &Option<serde_json::Value>,
    ) -> ConflictType {
        if let Some(remote_data) = remote_data {
            // 检查是否是数据结构冲突
            if self.has_structural_conflict(local_data, remote_data) {
                return ConflictType::Structural;
            }

            // 检查是否是内容冲突
            if self.has_content_conflict(local_data, remote_data) {
                return ConflictType::Content;
            }

            // 检查是否是时间戳冲突
            if self.has_timestamp_conflict(local_data, remote_data) {
                return ConflictType::Timestamp;
            }

            ConflictType::Unknown
        } else {
            ConflictType::RemoteMissing
        }
    }

    /// 检查是否有结构冲突
    fn has_structural_conflict(
        &self,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> bool {
        // 检查主要数据结构是否一致
        let local_keys: std::collections::HashSet<_> = local_data
            .as_object()
            .map(|obj| obj.keys().collect())
            .unwrap_or_default();

        let remote_keys: std::collections::HashSet<_> = remote_data
            .as_object()
            .map(|obj| obj.keys().collect())
            .unwrap_or_default();

        local_keys != remote_keys
    }

    /// 检查是否有内容冲突
    fn has_content_conflict(
        &self,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> bool {
        // 比较数据内容（排除时间戳）
        let local_content = self.extract_content_for_comparison(local_data);
        let remote_content = self.extract_content_for_comparison(remote_data);

        local_content != remote_content
    }

    /// 检查是否有时间戳冲突
    fn has_timestamp_conflict(
        &self,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> bool {
        let local_timestamp = self.get_data_timestamp(local_data);
        let remote_timestamp = self.get_data_timestamp(remote_data);

        match (local_timestamp, remote_timestamp) {
            (Ok(local_time), Ok(remote_time)) => {
                let diff = if local_time > remote_time {
                    local_time.signed_duration_since(remote_time)
                } else {
                    remote_time.signed_duration_since(local_time)
                };
                diff.num_seconds() > 30 // 超过30秒认为是冲突
            }
            _ => false,
        }
    }

    /// 提取用于比较的内容
    fn extract_content_for_comparison(&self, data: &serde_json::Value) -> serde_json::Value {
        let mut content = data.clone();

        // 移除时间戳相关字段
        if let Some(obj) = content.as_object_mut() {
            obj.remove("export_time");
            obj.remove("import_time");
            obj.remove("sync_time");
            obj.remove("merged_at");
        }

        content
    }

    /// 获取数据时间戳
    fn get_data_timestamp(
        &self,
        data: &serde_json::Value,
    ) -> Result<chrono::DateTime<chrono::Local>> {
        if let Some(export_time) = data.get("export_time") {
            if let Some(time_str) = export_time.as_str() {
                return chrono::DateTime::parse_from_rfc3339(time_str)
                    .map(|dt| dt.with_timezone(&chrono::Local))
                    .map_err(|e| AppError::Sync(format!("时间解析失败: {}", e)));
            }
        }

        Ok(Local::now())
    }
}

/// 冲突详细信息
#[derive(Debug, Clone)]
pub struct ConflictDetails {
    /// 冲突项
    pub item: SyncItem,
    /// 本地数据
    pub local_data: serde_json::Value,
    /// 远程数据
    pub remote_data: Option<serde_json::Value>,
    /// 本地时间戳
    pub local_timestamp: Option<chrono::DateTime<chrono::Local>>,
    /// 远程时间戳
    pub remote_timestamp: Option<chrono::DateTime<chrono::Local>>,
    /// 冲突类型
    pub conflict_type: ConflictType,
}

/// 冲突类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    /// 结构冲突
    Structural,
    /// 内容冲突
    Content,
    /// 时间戳冲突
    Timestamp,
    /// 远程数据缺失
    RemoteMissing,
    /// 未知冲突
    Unknown,
}
