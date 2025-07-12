//! # 数据合并器模块
//!
//! 负责合并本地和远程数据，处理不同的合并策略

use super::types::*;
use crate::errors::{AppError, Result};
use crate::storage::StorageManager;
use chrono::Local;
use std::sync::Arc;

/// 数据合并器
pub struct DataMerger {
    storage: Arc<StorageManager>,
}

impl DataMerger {
    /// 创建新的数据合并器
    pub fn new(storage: Arc<StorageManager>) -> Self {
        Self { storage }
    }

    /// 合并本地和远程数据
    ///
    /// 根据数据来源类型决定合并策略：
    /// - Fresh本地数据 + 远程数据：按模块合并，避免重复
    /// - 其他情况：使用冲突解决策略
    pub async fn merge_data(
        &self,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        log::info!("开始合并本地和远程数据");

        // 检测本地数据来源
        let local_origin = self.detect_data_origin(local_data).await?;

        let merged_data = match local_origin {
            DataOrigin::Fresh => {
                log::info!("执行新数据合并策略");
                self.merge_fresh_data(local_data, remote_data).await?
            }
            _ => {
                log::info!("执行标准数据合并策略");
                self.merge_standard_data(local_data, remote_data).await?
            }
        };

        log::info!("数据合并完成");
        Ok(merged_data)
    }

    /// 合并新安装后的本地数据与远程数据
    /// 策略：保留所有数据，去除重复项
    async fn merge_fresh_data(
        &self,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        log::info!("合并新数据：保留本地和远程的所有数据");

        let mut merged = serde_json::Map::new();

        // 复制元数据，使用最新的
        self.merge_metadata(&mut merged, local_data, remote_data)?;

        // 按模块合并数据
        self.merge_tasks(&mut merged, local_data, remote_data)
            .await?;
        self.merge_categories(&mut merged, local_data, remote_data)
            .await?;
        self.merge_time_entries(&mut merged, local_data, remote_data)
            .await?;
        self.merge_accounts(&mut merged, local_data, remote_data)
            .await?;
        self.merge_transactions(&mut merged, local_data, remote_data)
            .await?;

        Ok(serde_json::Value::Object(merged))
    }

    /// 标准数据合并策略
    /// 策略：时间戳较新的数据优先，但保留本地的修改
    async fn merge_standard_data(
        &self,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        log::info!("执行标准合并：基于时间戳的智能合并");

        // 获取时间戳
        let local_time = self
            .get_data_timestamp(local_data)
            .unwrap_or_else(|_| Local::now());
        let remote_time = self
            .get_data_timestamp(remote_data)
            .unwrap_or_else(|_| Local::now());

        // 选择基础数据（时间较新的）
        let (base_data, overlay_data) = if local_time > remote_time {
            (local_data, remote_data)
        } else {
            (remote_data, local_data)
        };

        log::info!(
            "基础数据时间戳: {}, 覆盖数据时间戳: {}",
            if local_time > remote_time {
                "本地"
            } else {
                "远程"
            },
            if local_time > remote_time {
                "远程"
            } else {
                "本地"
            }
        );

        // 以较新的数据为基础，合并较旧数据中的唯一项
        let mut merged = base_data.clone();

        // 合并分类
        self.merge_categories_selective(&mut merged, overlay_data)
            .await?;

        // 合并任务
        self.merge_tasks_selective(&mut merged, overlay_data)
            .await?;

        // 合并时间记录
        self.merge_time_entries_selective(&mut merged, overlay_data)
            .await?;

        // 合并账户
        self.merge_accounts_selective(&mut merged, overlay_data)
            .await?;

        // 合并交易
        self.merge_transactions_selective(&mut merged, overlay_data)
            .await?;

        // 更新元数据
        if let Some(merged_obj) = merged.as_object_mut() {
            merged_obj.insert(
                "export_time".to_string(),
                serde_json::Value::String(Local::now().to_rfc3339()),
            );
            merged_obj.insert(
                "merged_at".to_string(),
                serde_json::Value::String(Local::now().to_rfc3339()),
            );
        }

        Ok(merged)
    }

    /// 合并元数据
    fn merge_metadata(
        &self,
        merged: &mut serde_json::Map<String, serde_json::Value>,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> Result<()> {
        // 使用最新的版本信息
        if let Some(version) = local_data
            .get("version")
            .or_else(|| remote_data.get("version"))
        {
            merged.insert("version".to_string(), version.clone());
        }

        // 设置合并时间
        merged.insert(
            "export_time".to_string(),
            serde_json::Value::String(Local::now().to_rfc3339()),
        );
        merged.insert(
            "merged_at".to_string(),
            serde_json::Value::String(Local::now().to_rfc3339()),
        );

        // 记录合并来源
        merged.insert(
            "merge_sources".to_string(),
            serde_json::json!(["local", "remote"]),
        );

        Ok(())
    }

    /// 合并任务数据
    async fn merge_tasks(
        &self,
        merged: &mut serde_json::Map<String, serde_json::Value>,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> Result<()> {
        let mut all_tasks = Vec::new();
        let mut task_ids = std::collections::HashSet::new();

        // 添加本地任务
        if let Some(local_tasks) = local_data.get("tasks").and_then(|t| t.as_array()) {
            for task in local_tasks {
                if let Some(id) = task.get("id").and_then(|id| id.as_str()) {
                    if task_ids.insert(id.to_string()) {
                        all_tasks.push(task.clone());
                    }
                }
            }
        }

        // 添加远程任务（去重）
        if let Some(remote_tasks) = remote_data.get("tasks").and_then(|t| t.as_array()) {
            for task in remote_tasks {
                if let Some(id) = task.get("id").and_then(|id| id.as_str()) {
                    if task_ids.insert(id.to_string()) {
                        all_tasks.push(task.clone());
                    }
                }
            }
        }

        merged.insert("tasks".to_string(), serde_json::Value::Array(all_tasks));
        log::info!("合并了 {} 个任务", task_ids.len());
        Ok(())
    }

    /// 合并分类数据
    async fn merge_categories(
        &self,
        merged: &mut serde_json::Map<String, serde_json::Value>,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> Result<()> {
        let mut all_categories = Vec::new();
        let mut category_names = std::collections::HashSet::new();
        let mut category_ids = std::collections::HashSet::new();

        // 添加本地分类
        if let Some(local_categories) = local_data.get("categories").and_then(|c| c.as_array()) {
            for category in local_categories {
                if let Some(name) = category.get("name").and_then(|n| n.as_str()) {
                    if let Some(id) = category.get("id").and_then(|id| id.as_str()) {
                        if category_names.insert(name.to_string())
                            && category_ids.insert(id.to_string())
                        {
                            all_categories.push(category.clone());
                        }
                    }
                }
            }
        }

        // 添加远程分类（去重，优先按名称）
        if let Some(remote_categories) = remote_data.get("categories").and_then(|c| c.as_array()) {
            for category in remote_categories {
                if let Some(name) = category.get("name").and_then(|n| n.as_str()) {
                    if let Some(id) = category.get("id").and_then(|id| id.as_str()) {
                        // 如果名称已存在，跳过；如果名称不存在但ID存在，也跳过
                        if !category_names.contains(name) && category_ids.insert(id.to_string()) {
                            category_names.insert(name.to_string());
                            all_categories.push(category.clone());
                        }
                    }
                }
            }
        }

        let categories_count = all_categories.len();
        merged.insert(
            "categories".to_string(),
            serde_json::Value::Array(all_categories),
        );
        log::info!("合并了 {} 个分类", categories_count);
        Ok(())
    }

    /// 合并时间记录
    async fn merge_time_entries(
        &self,
        merged: &mut serde_json::Map<String, serde_json::Value>,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> Result<()> {
        let mut all_entries = Vec::new();
        let mut entry_ids = std::collections::HashSet::new();

        // 添加本地时间记录
        if let Some(local_entries) = local_data.get("time_entries").and_then(|e| e.as_array()) {
            for entry in local_entries {
                if let Some(id) = entry.get("id").and_then(|id| id.as_str()) {
                    if entry_ids.insert(id.to_string()) {
                        all_entries.push(entry.clone());
                    }
                }
            }
        }

        // 添加远程时间记录（去重）
        if let Some(remote_entries) = remote_data.get("time_entries").and_then(|e| e.as_array()) {
            for entry in remote_entries {
                if let Some(id) = entry.get("id").and_then(|id| id.as_str()) {
                    if entry_ids.insert(id.to_string()) {
                        all_entries.push(entry.clone());
                    }
                }
            }
        }

        let entries_count = all_entries.len();
        merged.insert(
            "time_entries".to_string(),
            serde_json::Value::Array(all_entries),
        );
        log::info!("合并了 {} 个时间记录", entries_count);
        Ok(())
    }

    /// 合并账户数据
    async fn merge_accounts(
        &self,
        merged: &mut serde_json::Map<String, serde_json::Value>,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> Result<()> {
        let mut all_accounts = Vec::new();
        let mut account_ids = std::collections::HashSet::new();

        // 添加本地账户
        if let Some(local_accounts) = local_data.get("accounts").and_then(|a| a.as_array()) {
            for account in local_accounts {
                if let Some(id) = account.get("id").and_then(|id| id.as_str()) {
                    if account_ids.insert(id.to_string()) {
                        all_accounts.push(account.clone());
                    }
                }
            }
        }

        // 添加远程账户（去重）
        if let Some(remote_accounts) = remote_data.get("accounts").and_then(|a| a.as_array()) {
            for account in remote_accounts {
                if let Some(id) = account.get("id").and_then(|id| id.as_str()) {
                    if account_ids.insert(id.to_string()) {
                        all_accounts.push(account.clone());
                    }
                }
            }
        }

        let accounts_count = all_accounts.len();
        merged.insert(
            "accounts".to_string(),
            serde_json::Value::Array(all_accounts),
        );
        log::info!("合并了 {} 个账户", accounts_count);
        Ok(())
    }

    /// 合并交易数据
    async fn merge_transactions(
        &self,
        merged: &mut serde_json::Map<String, serde_json::Value>,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> Result<()> {
        let mut all_transactions = Vec::new();
        let mut transaction_ids = std::collections::HashSet::new();

        // 添加本地交易
        if let Some(local_transactions) = local_data.get("transactions").and_then(|t| t.as_array())
        {
            for transaction in local_transactions {
                if let Some(id) = transaction.get("id").and_then(|id| id.as_str()) {
                    if transaction_ids.insert(id.to_string()) {
                        all_transactions.push(transaction.clone());
                    }
                }
            }
        }

        // 添加远程交易（去重）
        if let Some(remote_transactions) =
            remote_data.get("transactions").and_then(|t| t.as_array())
        {
            for transaction in remote_transactions {
                if let Some(id) = transaction.get("id").and_then(|id| id.as_str()) {
                    if transaction_ids.insert(id.to_string()) {
                        all_transactions.push(transaction.clone());
                    }
                }
            }
        }

        let transactions_count = all_transactions.len();
        merged.insert(
            "transactions".to_string(),
            serde_json::Value::Array(all_transactions),
        );
        log::info!("合并了 {} 个交易记录", transactions_count);
        Ok(())
    }

    /// 选择性合并分类（用于标准合并）
    async fn merge_categories_selective(
        &self,
        base_data: &mut serde_json::Value,
        overlay_data: &serde_json::Value,
    ) -> Result<()> {
        // 实现选择性合并逻辑，只添加基础数据中不存在的分类
        if let (Some(base_categories), Some(overlay_categories)) = (
            base_data
                .get_mut("categories")
                .and_then(|c| c.as_array_mut()),
            overlay_data.get("categories").and_then(|c| c.as_array()),
        ) {
            let mut existing_names: std::collections::HashSet<String> = base_categories
                .iter()
                .filter_map(|c| {
                    c.get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())
                })
                .collect();

            for category in overlay_categories {
                if let Some(name) = category.get("name").and_then(|n| n.as_str()) {
                    if existing_names.insert(name.to_string()) {
                        base_categories.push(category.clone());
                    }
                }
            }
        }
        Ok(())
    }

    /// 选择性合并任务（用于标准合并）  
    async fn merge_tasks_selective(
        &self,
        base_data: &mut serde_json::Value,
        overlay_data: &serde_json::Value,
    ) -> Result<()> {
        // 实现选择性合并逻辑，只添加基础数据中不存在的任务
        if let (Some(base_tasks), Some(overlay_tasks)) = (
            base_data.get_mut("tasks").and_then(|t| t.as_array_mut()),
            overlay_data.get("tasks").and_then(|t| t.as_array()),
        ) {
            let mut existing_ids: std::collections::HashSet<String> = base_tasks
                .iter()
                .filter_map(|t| {
                    t.get("id")
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_string())
                })
                .collect();

            for task in overlay_tasks {
                if let Some(id) = task.get("id").and_then(|id| id.as_str()) {
                    if existing_ids.insert(id.to_string()) {
                        base_tasks.push(task.clone());
                    }
                }
            }
        }
        Ok(())
    }

    /// 选择性合并时间记录（用于标准合并）
    async fn merge_time_entries_selective(
        &self,
        base_data: &mut serde_json::Value,
        overlay_data: &serde_json::Value,
    ) -> Result<()> {
        // 实现选择性合并逻辑，只添加基础数据中不存在的时间记录
        if let (Some(base_entries), Some(overlay_entries)) = (
            base_data
                .get_mut("time_entries")
                .and_then(|e| e.as_array_mut()),
            overlay_data.get("time_entries").and_then(|e| e.as_array()),
        ) {
            let mut existing_ids: std::collections::HashSet<String> = base_entries
                .iter()
                .filter_map(|e| {
                    e.get("id")
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_string())
                })
                .collect();

            for entry in overlay_entries {
                if let Some(id) = entry.get("id").and_then(|id| id.as_str()) {
                    if existing_ids.insert(id.to_string()) {
                        base_entries.push(entry.clone());
                    }
                }
            }
        }
        Ok(())
    }

    /// 选择性合并账户（用于标准合并）
    async fn merge_accounts_selective(
        &self,
        base_data: &mut serde_json::Value,
        overlay_data: &serde_json::Value,
    ) -> Result<()> {
        // 实现选择性合并逻辑，只添加基础数据中不存在的账户
        if let (Some(base_accounts), Some(overlay_accounts)) = (
            base_data.get_mut("accounts").and_then(|a| a.as_array_mut()),
            overlay_data.get("accounts").and_then(|a| a.as_array()),
        ) {
            let mut existing_ids: std::collections::HashSet<String> = base_accounts
                .iter()
                .filter_map(|a| {
                    a.get("id")
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_string())
                })
                .collect();

            for account in overlay_accounts {
                if let Some(id) = account.get("id").and_then(|id| id.as_str()) {
                    if existing_ids.insert(id.to_string()) {
                        base_accounts.push(account.clone());
                    }
                }
            }
        }
        Ok(())
    }

    /// 选择性合并交易（用于标准合并）
    async fn merge_transactions_selective(
        &self,
        base_data: &mut serde_json::Value,
        overlay_data: &serde_json::Value,
    ) -> Result<()> {
        // 实现选择性合并逻辑，只添加基础数据中不存在的交易
        if let (Some(base_transactions), Some(overlay_transactions)) = (
            base_data
                .get_mut("transactions")
                .and_then(|t| t.as_array_mut()),
            overlay_data.get("transactions").and_then(|t| t.as_array()),
        ) {
            let mut existing_ids: std::collections::HashSet<String> = base_transactions
                .iter()
                .filter_map(|t| {
                    t.get("id")
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_string())
                })
                .collect();

            for transaction in overlay_transactions {
                if let Some(id) = transaction.get("id").and_then(|id| id.as_str()) {
                    if existing_ids.insert(id.to_string()) {
                        base_transactions.push(transaction.clone());
                    }
                }
            }
        }
        Ok(())
    }

    /// 智能合并策略
    /// 根据数据类型和冲突情况选择最佳合并策略
    pub async fn smart_merge(
        &self,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
        merge_config: &MergeConfig,
    ) -> Result<serde_json::Value> {
        log::info!("执行智能合并策略");

        match merge_config.priority_strategy {
            MergePriorityStrategy::LocalFirst => {
                self.merge_with_local_priority(local_data, remote_data, merge_config)
                    .await
            }
            MergePriorityStrategy::RemoteFirst => {
                self.merge_with_remote_priority(local_data, remote_data, merge_config)
                    .await
            }
            MergePriorityStrategy::TimestampFirst => {
                self.merge_with_timestamp_priority(local_data, remote_data, merge_config)
                    .await
            }
        }
    }

    /// 本地优先合并
    async fn merge_with_local_priority(
        &self,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
        merge_config: &MergeConfig,
    ) -> Result<serde_json::Value> {
        let mut merged = local_data.clone();

        // 选择性添加远程数据中的新项
        if merge_config.deduplicate {
            self.merge_categories_selective(&mut merged, remote_data)
                .await?;
            self.merge_tasks_selective(&mut merged, remote_data).await?;
            self.merge_time_entries_selective(&mut merged, remote_data)
                .await?;
            self.merge_accounts_selective(&mut merged, remote_data)
                .await?;
            self.merge_transactions_selective(&mut merged, remote_data)
                .await?;
        }

        Ok(merged)
    }

    /// 远程优先合并
    async fn merge_with_remote_priority(
        &self,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
        merge_config: &MergeConfig,
    ) -> Result<serde_json::Value> {
        let mut merged = remote_data.clone();

        // 选择性添加本地数据中的新项
        if merge_config.deduplicate {
            self.merge_categories_selective(&mut merged, local_data)
                .await?;
            self.merge_tasks_selective(&mut merged, local_data).await?;
            self.merge_time_entries_selective(&mut merged, local_data)
                .await?;
            self.merge_accounts_selective(&mut merged, local_data)
                .await?;
            self.merge_transactions_selective(&mut merged, local_data)
                .await?;
        }

        Ok(merged)
    }

    /// 时间戳优先合并
    async fn merge_with_timestamp_priority(
        &self,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
        merge_config: &MergeConfig,
    ) -> Result<serde_json::Value> {
        // 使用现有的标准合并策略
        self.merge_standard_data(local_data, remote_data).await
    }

    /// 检测本地数据来源
    async fn detect_data_origin(&self, local_data: &serde_json::Value) -> Result<DataOrigin> {
        // 1. 检查本地数据中的来源追踪字段
        if let Some(base_remote_hash) = local_data.get("base_remote_hash").and_then(|v| v.as_str())
        {
            if !base_remote_hash.is_empty() {
                return Ok(DataOrigin::BasedOnRemote);
            }
        }

        // 2. 检查存储中的来源追踪信息
        let stored_base_hash = self.get_base_remote_hash().await.unwrap_or_default();
        if !stored_base_hash.is_empty() {
            return Ok(DataOrigin::BasedOnRemote);
        }

        // 3. 检查是否为新安装后的数据
        if self.is_fresh_install().await? {
            return Ok(DataOrigin::Fresh);
        }

        // 4. 检查数据中的新安装标记
        if let Some(is_fresh) = local_data.get("is_fresh_install").and_then(|v| v.as_bool()) {
            if is_fresh {
                return Ok(DataOrigin::Fresh);
            }
        }

        // 5. 默认情况
        Ok(DataOrigin::Unknown)
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

    /// 获取本地数据基于的远程版本哈希
    async fn get_base_remote_hash(&self) -> Result<String> {
        match self.storage.get_database().get_setting("base_remote_hash") {
            Ok(Some(hash)) => Ok(hash),
            _ => Ok(String::new()),
        }
    }

    /// 检查是否为新安装后首次创建的数据
    async fn is_fresh_install(&self) -> Result<bool> {
        let has_synced = self
            .storage
            .get_database()
            .get_setting("has_synced")
            .unwrap_or(None)
            .map(|v| v == "true")
            .unwrap_or(false);

        let has_data = self.has_local_data().await?;
        let has_base_hash = !self.get_base_remote_hash().await?.is_empty();

        Ok(has_data && !has_base_hash && !has_synced)
    }

    /// 检查是否有本地数据
    async fn has_local_data(&self) -> Result<bool> {
        let tasks = self.storage.get_database().get_all_tasks()?;
        let time_entries = self.storage.get_database().get_all_time_entries()?;
        let transactions = self.storage.get_database().get_all_transactions()?;

        Ok(!tasks.is_empty() || !time_entries.is_empty() || !transactions.is_empty())
    }
}
