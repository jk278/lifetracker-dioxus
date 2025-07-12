//! # 冲突解决模块
//!
//! 处理同步过程中的数据冲突

use crate::errors::{AppError, Result};
use crate::sync::{ConflictStrategy, SyncItem, SyncResult};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// 冲突信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// 冲突ID
    pub id: String,
    /// 本地项目
    pub local_item: SyncItem,
    /// 远程项目
    pub remote_item: SyncItem,
    /// 冲突类型
    pub conflict_type: ConflictType,
    /// 冲突发现时间
    pub detected_at: DateTime<Local>,
    /// 解决方案
    pub resolution: Option<ConflictResolution>,
}

/// 冲突类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictType {
    /// 内容冲突（同时修改）
    ContentConflict,
    /// 删除冲突（一方删除，一方修改）
    DeleteConflict,
    /// 类型冲突（文件vs目录）
    TypeConflict,
}

/// 冲突解决方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// 使用本地版本
    UseLocal,
    /// 使用远程版本
    UseRemote,
    /// 保留两个版本
    KeepBoth,
    /// 手动合并
    ManualMerge { merged_content: Vec<u8> },
}

/// 冲突解决器
pub struct ConflictResolver {
    /// 冲突策略
    strategy: ConflictStrategy,
}

impl ConflictResolver {
    /// 创建新的冲突解决器
    pub fn new(strategy: ConflictStrategy) -> Self {
        Self { strategy }
    }

    /// 检测冲突
    pub fn detect_conflicts(
        local_items: &[SyncItem],
        remote_items: &[SyncItem],
    ) -> Vec<ConflictInfo> {
        let mut conflicts = Vec::new();

        for local_item in local_items {
            if let Some(remote_item) = remote_items.iter().find(|r| r.name == local_item.name) {
                if let Some(conflict) = Self::check_item_conflict(local_item, remote_item) {
                    conflicts.push(conflict);
                }
            }
        }

        conflicts
    }

    /// 检查单个项目的冲突
    fn check_item_conflict(local_item: &SyncItem, remote_item: &SyncItem) -> Option<ConflictInfo> {
        // 如果哈希值不同，可能存在冲突
        if local_item.hash != remote_item.hash {
            let conflict_type = if let Some(remote_modified) = remote_item.remote_modified {
                // 比较修改时间来确定冲突类型
                if local_item.local_modified > remote_modified {
                    ConflictType::ContentConflict
                } else if local_item.local_modified < remote_modified {
                    ConflictType::ContentConflict
                } else {
                    ConflictType::ContentConflict // 同时修改
                }
            } else {
                ConflictType::ContentConflict
            };

            Some(ConflictInfo {
                id: format!("conflict_{}", local_item.id),
                local_item: local_item.clone(),
                remote_item: remote_item.clone(),
                conflict_type,
                detected_at: Local::now(),
                resolution: None,
            })
        } else {
            None
        }
    }

    /// 自动解决冲突
    pub fn resolve_conflicts(&self, conflicts: &mut [ConflictInfo]) -> Result<()> {
        for conflict in conflicts {
            if conflict.resolution.is_none() {
                let resolution = match self.strategy {
                    ConflictStrategy::Manual => {
                        // 手动解决，等待用户选择
                        continue;
                    }
                    ConflictStrategy::LocalWins => ConflictResolution::UseLocal,
                    ConflictStrategy::RemoteWins => ConflictResolution::UseRemote,
                    ConflictStrategy::KeepBoth => ConflictResolution::KeepBoth,
                };

                conflict.resolution = Some(resolution);
            }
        }

        Ok(())
    }

    /// 应用冲突解决方案
    pub fn apply_resolution(&self, conflict: &ConflictInfo) -> Result<Vec<SyncItem>> {
        match &conflict.resolution {
            Some(ConflictResolution::UseLocal) => Ok(vec![conflict.local_item.clone()]),
            Some(ConflictResolution::UseRemote) => Ok(vec![conflict.remote_item.clone()]),
            Some(ConflictResolution::KeepBoth) => {
                let mut local_item = conflict.local_item.clone();
                local_item.name = format!("{}.local", local_item.name);
                let mut remote_item = conflict.remote_item.clone();
                remote_item.name = format!("{}.remote", remote_item.name);
                Ok(vec![local_item, remote_item])
            }
            Some(ConflictResolution::ManualMerge { merged_content: _ }) => {
                // TODO: 实现手动合并逻辑
                Err(AppError::Sync("手动合并功能尚未实现".to_string()))
            }
            None => Err(AppError::Sync("冲突尚未解决".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::{SyncDirection, SyncStatus};

    #[test]
    fn test_conflict_detection() {
        let local_item = SyncItem {
            id: "1".to_string(),
            name: "test.txt".to_string(),
            local_path: "test.txt".to_string(),
            remote_path: "test.txt".to_string(),
            size: 100,
            local_modified: Local::now(),
            remote_modified: None,
            hash: "local_hash".to_string(),
            status: SyncStatus::Idle,
            direction: SyncDirection::Upload,
        };

        let remote_item = SyncItem {
            id: "1".to_string(),
            name: "test.txt".to_string(),
            local_path: "test.txt".to_string(),
            remote_path: "test.txt".to_string(),
            size: 100,
            local_modified: Local::now(),
            remote_modified: Some(Local::now()),
            hash: "remote_hash".to_string(),
            status: SyncStatus::Idle,
            direction: SyncDirection::Download,
        };

        let conflicts = ConflictResolver::detect_conflicts(&[local_item], &[remote_item]);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::ContentConflict);
    }
}
