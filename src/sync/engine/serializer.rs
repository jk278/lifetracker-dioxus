//! # 数据序列化器模块
//!
//! 负责数据的序列化、反序列化、导入和导出操作

use super::types::*;
use crate::errors::{AppError, Result};
use crate::storage::StorageManager;
use chrono::{DateTime, Local};
use std::sync::Arc;

/// 数据序列化器
pub struct DataSerializer {
    storage: Arc<StorageManager>,
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

        // 获取数据来源追踪信息
        let base_remote_hash = self.get_base_remote_hash().await.unwrap_or(String::new());
        let is_fresh_install = self.is_fresh_install().await.unwrap_or(false);
        let last_sync_time = self.get_last_sync_time_from_storage().await;

        // 创建导出数据结构
        let export_data = serde_json::json!({
            "tasks": tasks,
            "categories": categories,
            "time_entries": time_entries,
            "transactions": transactions,
            "accounts": accounts,
            "export_time": Local::now(),
            "version": env!("CARGO_PKG_VERSION"),
            // 数据来源追踪字段
            "base_remote_hash": base_remote_hash,
            "is_fresh_install": is_fresh_install,
            "last_sync_time": last_sync_time
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
                // 6. 更新来源追踪信息
                if let Err(e) = self.update_origin_tracking(&import_data, None).await {
                    log::warn!("更新来源追踪信息失败: {}", e);
                    // 不中断导入流程，只记录警告
                }

                // 7. 提交事务
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

        // 9. 使用验证器验证数据格式
        let validator = super::DataValidator::new();
        validator.validate_data(data)?;

        log::info!("数据验证通过，包含 {} 条记录", total_records);
        Ok(true)
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

    /// 提取用于比较的内容（排除时间戳等元数据）
    pub fn extract_content_for_comparison(
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
    pub fn calculate_content_hash(&self, content: &serde_json::Value) -> String {
        let content_str = serde_json::to_string(content).unwrap_or_default();
        format!("{:x}", md5::compute(content_str.as_bytes()))
    }

    /// 获取本地数据基于的远程版本哈希
    async fn get_base_remote_hash(&self) -> Result<String> {
        // 从存储中读取上次同步时的远程数据哈希
        match self.storage.get_database().get_setting("base_remote_hash") {
            Ok(Some(hash)) => Ok(hash),
            _ => Ok(String::new()),
        }
    }

    /// 设置远程数据哈希
    async fn set_base_remote_hash(&self, hash: &str) -> Result<()> {
        self.storage
            .get_database()
            .set_setting("base_remote_hash", hash)?;
        log::info!("已更新远程数据哈希: {}", hash);
        Ok(())
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

    /// 设置已同步标记
    async fn set_synced_flag(&self) -> Result<()> {
        self.storage
            .get_database()
            .set_setting("has_synced", "true")?;
        Ok(())
    }

    /// 检查是否有本地数据
    async fn has_local_data(&self) -> Result<bool> {
        let tasks = self.storage.get_database().get_all_tasks()?;
        let time_entries = self.storage.get_database().get_all_time_entries()?;
        let transactions = self.storage.get_database().get_all_transactions()?;

        Ok(!tasks.is_empty() || !time_entries.is_empty() || !transactions.is_empty())
    }

    /// 从存储中获取上次同步时间
    async fn get_last_sync_time_from_storage(&self) -> Option<DateTime<Local>> {
        if let Ok(Some(time_str)) = self.storage.get_database().get_setting("last_sync_time") {
            if let Ok(parsed_time) = DateTime::parse_from_rfc3339(&time_str) {
                return Some(parsed_time.with_timezone(&Local));
            }
        }
        None
    }

    /// 设置上次同步时间
    async fn set_last_sync_time(&self, time: DateTime<Local>) -> Result<()> {
        let time_str = time.to_rfc3339();
        self.storage
            .get_database()
            .set_setting("last_sync_time", &time_str)?;
        log::info!("已更新同步时间: {}", time_str);
        Ok(())
    }

    /// 在导入数据时更新来源追踪信息
    pub async fn update_origin_tracking(
        &self,
        import_data: &serde_json::Value,
        remote_hash: Option<&str>,
    ) -> Result<()> {
        // 如果导入的数据包含来源信息，则更新本地记录
        if let Some(base_hash) = import_data.get("base_remote_hash").and_then(|v| v.as_str()) {
            if !base_hash.is_empty() {
                self.set_base_remote_hash(base_hash).await?;
            }
        }

        // 如果从远程下载，设置远程哈希
        if let Some(hash) = remote_hash {
            self.set_base_remote_hash(hash).await?;
        }

        // 如果导入的数据有同步时间信息，更新本地记录
        if let Some(sync_time_str) = import_data.get("last_sync_time").and_then(|v| v.as_str()) {
            if let Ok(sync_time) = DateTime::parse_from_rfc3339(sync_time_str) {
                self.set_last_sync_time(sync_time.with_timezone(&Local))
                    .await?;
            }
        }

        // 标记已进行过同步
        self.set_synced_flag().await?;

        Ok(())
    }

    /// 导出特定类型的数据
    pub async fn export_data_by_type(&self, data_types: &[&str]) -> Result<Vec<u8>> {
        log::info!("导出指定类型的数据: {:?}", data_types);

        let mut export_data = serde_json::Map::new();

        for data_type in data_types {
            match *data_type {
                "tasks" => {
                    let tasks = self.storage.get_database().get_all_tasks()?;
                    export_data.insert("tasks".to_string(), serde_json::to_value(tasks)?);
                }
                "categories" => {
                    let categories = self.storage.get_database().get_all_categories()?;
                    export_data.insert("categories".to_string(), serde_json::to_value(categories)?);
                }
                "time_entries" => {
                    let time_entries = self.storage.get_database().get_all_time_entries()?;
                    export_data.insert(
                        "time_entries".to_string(),
                        serde_json::to_value(time_entries)?,
                    );
                }
                "transactions" => {
                    let transactions = self.storage.get_database().get_all_transactions()?;
                    export_data.insert(
                        "transactions".to_string(),
                        serde_json::to_value(transactions)?,
                    );
                }
                "accounts" => {
                    let accounts = self.storage.get_database().get_all_accounts()?;
                    export_data.insert("accounts".to_string(), serde_json::to_value(accounts)?);
                }
                _ => {
                    log::warn!("未知的数据类型: {}", data_type);
                }
            }
        }

        // 添加元数据
        export_data.insert(
            "export_time".to_string(),
            serde_json::Value::String(Local::now().to_rfc3339()),
        );
        export_data.insert(
            "version".to_string(),
            serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()),
        );
        export_data.insert("partial_export".to_string(), serde_json::Value::Bool(true));
        export_data.insert(
            "exported_types".to_string(),
            serde_json::to_value(data_types)?,
        );

        let json_data = serde_json::to_vec(&export_data)?;
        log::info!("部分数据导出完成，大小: {} 字节", json_data.len());
        Ok(json_data)
    }

    /// 创建增量备份
    pub async fn create_incremental_backup(&self, since: DateTime<Local>) -> Result<Vec<u8>> {
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
