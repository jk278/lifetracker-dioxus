//! # 数据存储模块
//!
//! 提供数据持久化功能，包括：
//! - 数据库操作
//! - 数据模型定义
//! - 数据库迁移

pub mod accounting_models;
pub mod database;
pub mod migrations;
pub mod models;
pub mod task_models;

// 重新导出主要类型
pub use database::Database;
pub use models::*;
pub use task_models::*;

// 导出记账功能相关模型
pub use accounting_models::{
    // 主要数据模型
    Account,
    AccountBalance,
    AccountInsert,
    AccountType,
    AccountUpdate,
    Budget,
    BudgetInsert,
    BudgetPeriod,

    BudgetUpdate,

    CategoryBreakdown,
    FinancialReport,
    // 统计和查询模型
    FinancialStats,
    MonthlyTrend,
    Transaction,
    TransactionCategory,
    TransactionCategoryInsert,
    TransactionCategoryUpdate,
    TransactionInsert,
    TransactionQuery,
    TransactionStatus,
    // 枚举类型
    TransactionType,
    TransactionUpdate,
    // 趋势数据模型
    TrendData,
    TrendGranularity,
};

use crate::errors::AppError;
use rusqlite::Connection;
use std::path::Path;

/// 数据库配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// 数据库文件路径
    pub database_path: String,
    /// 是否启用WAL模式
    pub enable_wal: bool,
    /// 连接池大小
    pub pool_size: u32,
    /// 查询超时时间（秒）
    pub timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        // 获取应用数据目录，失败时使用./data目录
        let app_dir =
            crate::utils::get_app_data_dir().unwrap_or_else(|_| std::path::PathBuf::from("./data"));

        Self {
            database_path: app_dir.join("timetracker.db").to_string_lossy().to_string(),
            enable_wal: true,
            pool_size: 10,
            timeout_seconds: 30,
        }
    }
}

/// 存储管理器
///
/// 协调数据库操作和数据模型转换
#[derive(Debug)]
pub struct StorageManager {
    /// 数据库连接
    database: Database,
    /// 配置信息
    config: DatabaseConfig,
}

impl StorageManager {
    /// 创建新的存储管理器
    ///
    /// # 参数
    /// * `config` - 数据库配置
    pub fn new(config: DatabaseConfig) -> crate::errors::Result<Self> {
        let database = Database::new(&config.database_path)?;

        let storage_manager = Self { database, config };

        log::info!("存储管理器初始化完成");
        Ok(storage_manager)
    }

    /// 使用默认配置创建存储管理器
    pub fn with_default_config() -> crate::errors::Result<Self> {
        let config = DatabaseConfig::default();
        Self::new(config)
    }

    /// 初始化存储系统
    pub fn initialize(&mut self) -> crate::errors::Result<()> {
        // 确保数据库表结构存在
        self.database.run_migrations()?;

        // 配置数据库优化参数
        self.configure_database()?;

        log::info!("存储系统初始化完成");
        Ok(())
    }

    /// 配置数据库优化参数
    fn configure_database(&self) -> crate::errors::Result<()> {
        let conn = self.database.get_connection()?.get_raw_connection();
        let conn = conn.lock().unwrap();

        // 启用外键约束
        conn.execute("PRAGMA foreign_keys = ON", rusqlite::params![])?;

        // 设置WAL模式（如果启用）
        if self.config.enable_wal {
            let journal_mode: String =
                conn.query_row("PRAGMA journal_mode = WAL", rusqlite::params![], |row| {
                    row.get(0)
                })?;
            log::debug!("数据库日志模式设置为: {}", journal_mode);
        }

        // 设置同步模式
        conn.execute("PRAGMA synchronous = NORMAL", rusqlite::params![])?;

        // 设置缓存大小
        conn.execute("PRAGMA cache_size = -64000", rusqlite::params![])?; // 64MB

        // 设置临时存储
        conn.execute("PRAGMA temp_store = MEMORY", rusqlite::params![])?;

        log::debug!("数据库配置完成");
        Ok(())
    }

    /// 获取数据库引用
    pub fn get_database(&self) -> &Database {
        &self.database
    }

    /// 获取数据库可变引用
    pub fn get_database_mut(&mut self) -> &mut Database {
        &mut self.database
    }

    /// 备份数据库到文件
    ///
    /// # 参数
    /// * `backup_path` - 备份文件路径
    pub fn backup_database<P: AsRef<Path>>(&self, backup_path: P) -> crate::errors::Result<()> {
        let source_conn = self.database.get_connection()?.get_raw_connection();
        let source_conn = source_conn.lock().unwrap();
        let mut backup_conn = Connection::open(backup_path)?;

        // 使用SQLite的备份API
        let backup = rusqlite::backup::Backup::new(&*source_conn, &mut backup_conn)?;
        backup.run_to_completion(5, std::time::Duration::from_millis(250), None)?;

        log::info!("数据库备份完成");
        Ok(())
    }

    /// 从备份恢复数据库（共享引用版本）
    ///
    /// # 参数
    /// * `backup_path` - 备份文件路径
    ///
    /// 注意：此方法不需要可变引用，通过内部的数据库连接管理来实现恢复
    pub fn restore_database_from_backup<P: AsRef<Path>>(
        &self,
        backup_path: P,
    ) -> crate::errors::Result<()> {
        let backup_path = backup_path.as_ref();
        log::info!("开始恢复数据库从备份文件: {}", backup_path.display());

        if !backup_path.exists() {
            let error_msg = format!("备份文件不存在: {}", backup_path.display());
            log::error!("{}", error_msg);
            return Err(AppError::Storage(error_msg));
        }

        // 检查备份文件是否是有效的SQLite文件
        let backup_file_size = std::fs::metadata(backup_path)
            .map_err(|e| AppError::Storage(format!("无法读取备份文件信息: {}", e)))?
            .len();

        log::info!("备份文件大小: {} bytes", backup_file_size);

        if backup_file_size == 0 {
            return Err(AppError::Storage("备份文件为空".to_string()));
        }

        log::info!("正在打开备份文件连接...");
        let backup_conn = Connection::open(backup_path)
            .map_err(|e| AppError::Storage(format!("无法打开备份文件: {}", e)))?;

        log::info!("正在验证备份文件的完整性...");
        // 检查备份文件的完整性
        let integrity_check: String = backup_conn
            .query_row("PRAGMA integrity_check", rusqlite::params![], |row| {
                row.get(0)
            })
            .map_err(|e| AppError::Storage(format!("备份文件完整性检查失败: {}", e)))?;

        if integrity_check != "ok" {
            return Err(AppError::Storage(format!(
                "备份文件已损坏: {}",
                integrity_check
            )));
        }

        log::info!("备份文件完整性验证通过");

        // 获取目标数据库连接
        log::info!("正在获取目标数据库连接...");
        let target_conn = self
            .database
            .get_connection()
            .map_err(|e| AppError::Storage(format!("无法获取目标数据库连接: {}", e)))?
            .get_raw_connection();

        log::info!("正在获取数据库写锁...");
        let mut target_conn_guard = target_conn
            .lock()
            .map_err(|_| AppError::Storage("无法获取数据库写锁".to_string()))?;

        log::info!("开始执行数据库恢复...");
        // 使用 rusqlite 的 backup API 进行恢复
        // 从备份文件(source)恢复到当前数据库(destination)
        {
            let backup = rusqlite::backup::Backup::new(&backup_conn, &mut *target_conn_guard)
                .map_err(|e| AppError::Storage(format!("创建备份对象失败: {}", e)))?;

            log::info!("正在执行数据恢复...");
            backup
                .run_to_completion(5, std::time::Duration::from_millis(250), None)
                .map_err(|e| AppError::Storage(format!("数据恢复执行失败: {}", e)))?;
        } // backup 在这里被释放，结束可变借用

        log::info!("数据库恢复完成，正在验证恢复结果...");

        // 验证恢复后的数据库完整性
        let restored_integrity: String = target_conn_guard
            .query_row("PRAGMA integrity_check", rusqlite::params![], |row| {
                row.get(0)
            })
            .map_err(|e| AppError::Storage(format!("恢复后完整性检查失败: {}", e)))?;

        if restored_integrity != "ok" {
            return Err(AppError::Storage(format!(
                "恢复后数据库完整性验证失败: {}",
                restored_integrity
            )));
        }

        log::info!("数据库恢复完成并验证通过");
        Ok(())
    }

    /// 从备份恢复数据库
    ///
    /// # 参数
    /// * `backup_path` - 备份文件路径
    pub fn restore_database<P: AsRef<Path>>(
        &mut self,
        backup_path: P,
    ) -> crate::errors::Result<()> {
        // 直接调用共享引用版本
        self.restore_database_from_backup(backup_path)
    }

    /// 优化数据库
    ///
    /// 执行VACUUM和ANALYZE操作
    pub fn optimize_database(&self) -> crate::errors::Result<()> {
        let conn = self.database.get_connection()?.get_raw_connection();
        let conn = conn.lock().unwrap();

        // 清理数据库
        conn.execute("VACUUM", rusqlite::params![])?;
        conn.execute("PRAGMA optimize", rusqlite::params![])?;

        log::info!("数据库优化完成");
        Ok(())
    }

    /// 获取数据库统计信息
    pub fn get_database_stats(&self) -> crate::errors::Result<DatabaseStats> {
        let conn = self.database.get_connection()?.get_raw_connection();
        let conn = conn.lock().unwrap();
        let mut stats = DatabaseStats::default();

        // 获取表的记录数
        let tables = vec!["time_entries", "categories"];
        for table in tables {
            let count: i64 = conn.query_row(
                &format!("SELECT COUNT(*) FROM {}", table),
                rusqlite::params![],
                |row| row.get(0),
            )?;

            stats.table_stats.push(TableStats {
                table_name: table.to_string(),
                record_count: count as usize,
            });
        }

        // 获取数据库文件大小
        if let Ok(metadata) = std::fs::metadata(&self.config.database_path) {
            stats.database_size = metadata.len();
            stats.size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        }

        // 获取页面信息
        let page_count: i64 =
            conn.query_row("PRAGMA page_count", rusqlite::params![], |row| row.get(0))?;
        let page_size: i64 =
            conn.query_row("PRAGMA page_size", rusqlite::params![], |row| row.get(0))?;

        stats.page_count = page_count as usize;
        stats.page_size = page_size as usize;
        stats.last_updated = chrono::Local::now();

        Ok(stats)
    }

    /// 检查数据库完整性
    pub fn check_integrity(&self) -> crate::errors::Result<bool> {
        let conn = self.database.get_connection()?.get_raw_connection();
        let conn = conn.lock().unwrap();
        let result: String =
            conn.query_row("PRAGMA integrity_check", rusqlite::params![], |row| {
                row.get(0)
            })?;

        let is_ok = result == "ok";
        if !is_ok {
            log::warn!("数据库完整性检查失败: {}", result);
        }

        Ok(is_ok)
    }

    /// 关闭数据库连接
    pub fn close(self) -> crate::errors::Result<()> {
        self.database.close()?;
        log::info!("数据库连接已关闭");
        Ok(())
    }

    /// 创建数据库备份
    pub fn create_backup<P: AsRef<Path>>(&self, backup_path: P) -> crate::errors::Result<()> {
        self.backup_database(backup_path)
    }

    /// 导出数据到文件
    pub fn export_data<P: AsRef<Path>>(&self, export_path: P) -> crate::errors::Result<()> {
        // 获取所有数据
        let time_entries = self.database.get_all_time_entries()?;
        let category_models = self.database.get_all_categories()?;
        let categories: Vec<crate::core::Category> = category_models
            .into_iter()
            .map(|model| model.into())
            .collect();

        // 保存长度以避免借用问题
        let entries_len = time_entries.len();
        let categories_len = categories.len();

        let export_data = crate::utils::export::ExportData {
            time_entries,
            categories,
            metadata: crate::utils::export::ExportMetadata {
                export_time: chrono::Local::now(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                total_entries: entries_len,
                total_categories: categories_len,
                date_range: None,
                filters_applied: vec![],
            },
            statistics: Some(crate::utils::export::ExportStatistics {
                total_time: chrono::Duration::zero(),
                average_session_time: chrono::Duration::zero(),
                category_breakdown: std::collections::HashMap::new(),
                daily_totals: std::collections::HashMap::new(),
                most_productive_day: None,
                most_used_category: None,
            }),
        };

        // 导出为JSON格式
        crate::utils::export::export_to_json(&export_data, export_path)
            .map_err(|e| AppError::Storage(e.to_string()))
    }

    /// 从文件导入数据（不需要可变借用）
    pub fn import_data_from_file<P: AsRef<Path>>(
        &self,
        import_path: P,
    ) -> crate::errors::Result<()> {
        let import_path = import_path.as_ref();
        log::info!("开始从文件导入数据: {}", import_path.display());

        // 从JSON文件导入
        let import_data = crate::utils::import::import_from_json(import_path)?;

        // 获取数据库连接
        let conn = self.database.get_connection()?.get_raw_connection();
        let conn = conn.lock().unwrap();

        // 导入分类
        for category in import_data.categories {
            let category_model: crate::storage::models::CategoryModel = category.into();
            if let Err(e) = conn.execute(
                "INSERT OR REPLACE INTO categories (id, name, description, color, icon, created_at, updated_at, is_active, sort_order, parent_id, daily_target_seconds, weekly_target_seconds) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                rusqlite::params![
                    category_model.id.to_string(),
                    category_model.name,
                    category_model.description,
                    category_model.color,
                    category_model.icon,
                    category_model.created_at.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                    category_model.updated_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string()),
                    category_model.is_active,
                    category_model.sort_order,
                    category_model.parent_id.map(|id| id.to_string()),
                    category_model.daily_target_seconds,
                    category_model.weekly_target_seconds,
                ]
            ) {
                log::warn!("导入分类失败: {}", e);
            }
        }

        // 导入时间条目
        for entry in import_data.time_entries {
            let entry_model: crate::storage::models::TimeEntry = entry.into();
            if let Err(e) = conn.execute(
                "INSERT OR REPLACE INTO time_entries (id, start_time, end_time, duration_seconds, task_name, category_id, description, tags, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    entry_model.id.to_string(),
                    entry_model.start_time.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                    entry_model.end_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string()),
                    entry_model.duration_seconds,
                    entry_model.task_name,
                    entry_model.category_id.map(|id| id.to_string()),
                    entry_model.description,
                    serde_json::to_string(&entry_model.tags).unwrap_or_default(),
                    entry_model.created_at.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                    entry_model.updated_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string()).unwrap_or_else(|| entry_model.created_at.format("%Y-%m-%d %H:%M:%S%.3f").to_string()),
                ]
            ) {
                log::warn!("导入时间条目失败: {}", e);
            }
        }

        log::info!("数据导入完成");
        Ok(())
    }

    /// 从文件导入数据
    pub fn import_data<P: AsRef<Path>>(&mut self, import_path: P) -> crate::errors::Result<()> {
        // 直接调用共享引用版本
        self.import_data_from_file(import_path)
    }

    /// 清空所有数据
    pub fn clear_all_data(&mut self) -> crate::errors::Result<()> {
        let conn = self.database.get_connection()?.get_raw_connection();
        let conn = conn.lock().unwrap();

        // 删除所有时间条目
        conn.execute("DELETE FROM time_entries", rusqlite::params![])?;

        // 删除所有分类
        conn.execute("DELETE FROM categories", rusqlite::params![])?;

        // 重置自增ID（如果存在的话）
        if let Err(e) = conn.execute("DELETE FROM sqlite_sequence", rusqlite::params![]) {
            log::debug!("清理sqlite_sequence表失败（可能不存在）: {}", e);
        }

        log::info!("所有数据已清空");
        Ok(())
    }

    /// 获取每日统计范围
    pub fn get_daily_stats_range(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> crate::errors::Result<Vec<crate::storage::models::DailyStats>> {
        // 这里应该实现实际的查询逻辑
        // 暂时返回空向量
        Ok(vec![])
    }

    /// 获取每周统计范围
    pub fn get_weekly_stats_range(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> crate::errors::Result<Vec<crate::storage::models::WeeklyStats>> {
        // 这里应该实现实际的查询逻辑
        // 暂时返回空向量
        Ok(vec![])
    }

    /// 获取每月统计范围
    pub fn get_monthly_stats_range(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> crate::errors::Result<Vec<crate::storage::models::MonthlyStats>> {
        // 这里应该实现实际的查询逻辑
        // 暂时返回空向量
        Ok(vec![])
    }

    /// 获取分类统计
    pub fn get_category_stats(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> crate::errors::Result<Vec<crate::storage::models::CategoryStats>> {
        // 这里应该实现实际的查询逻辑
        // 暂时返回空向量
        Ok(vec![])
    }

    /// 获取配置信息
    pub fn get_config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// 获取最近的时间记录
    pub fn get_recent_time_entries(&self, limit: usize) -> crate::errors::Result<Vec<TimeEntry>> {
        self.database.get_recent_time_entries(limit)
    }

    // ==================== 笔记操作 ====================

    /// 插入笔记
    pub async fn insert_note(&self, note: &Note) -> crate::errors::Result<()> {
        self.database.insert_note(note)?;
        Ok(())
    }

    /// 获取所有笔记
    pub async fn get_notes(&self, query: &NoteQuery) -> crate::errors::Result<Vec<Note>> {
        if let Some(search) = &query.search {
            // 如果有搜索条件，使用搜索方法
            self.database.search_notes(query)
        } else {
            // 否则获取所有笔记
            self.database.get_all_notes()
        }
    }

    /// 根据ID获取笔记
    pub async fn get_note_by_id(&self, id: uuid::Uuid) -> crate::errors::Result<Option<Note>> {
        self.database.get_note_by_id(id)
    }

    /// 更新笔记
    pub async fn update_note(
        &self,
        id: uuid::Uuid,
        update: &NoteUpdate,
    ) -> crate::errors::Result<()> {
        self.database.update_note(id, update)?;
        Ok(())
    }

    /// 删除笔记
    pub async fn delete_note(&self, id: uuid::Uuid) -> crate::errors::Result<()> {
        self.database.delete_note(id)?;
        Ok(())
    }

    /// 搜索笔记
    pub async fn search_notes(&self, query: &NoteQuery) -> crate::errors::Result<Vec<Note>> {
        self.database.search_notes(query)
    }

    /// 获取笔记统计信息
    pub async fn get_notes_stats(&self) -> crate::errors::Result<NoteStats> {
        self.database.get_notes_stats()
    }

    /// 获取所有笔记标签
    pub async fn get_all_note_tags(&self) -> crate::errors::Result<Vec<String>> {
        self.database.get_all_note_tags()
    }

    /// 数据库维护
    pub fn maintenance(&self) -> crate::errors::Result<()> {
        // VACUUM 操作：回收空间并整理数据库
        self.database.get_connection()?.execute("VACUUM", &[])?;

        // ANALYZE 操作：更新查询优化器的统计信息
        self.database.get_connection()?.execute("ANALYZE", &[])?;

        // 重建FTS索引（如果存在笔记表）
        if let Ok(_) = self
            .database
            .get_connection()?
            .execute("INSERT INTO notes_fts(notes_fts) VALUES('rebuild')", &[])
        {
            log::debug!("FTS索引重建完成");
        }

        log::info!("数据库维护完成");
        Ok(())
    }
}

/// 数据库统计信息
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    /// 数据库文件大小（字节）
    pub database_size: u64,
    /// 页面数量
    pub page_count: usize,
    /// 页面大小（字节）
    pub page_size: usize,
    /// 各表记录数量
    pub table_stats: Vec<TableStats>,
    /// 数据库大小（MB）
    pub size_mb: f64,
    /// 最后更新时间
    pub last_updated: chrono::DateTime<chrono::Local>,
}

impl DatabaseStats {
    /// 获取格式化的数据库大小
    pub fn get_formatted_size(&self) -> String {
        let size = self.database_size as f64;

        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// 获取总记录数
    pub fn get_total_records(&self) -> usize {
        self.table_stats.iter().map(|t| t.record_count).sum()
    }
}

impl Default for DatabaseStats {
    fn default() -> Self {
        Self {
            database_size: 0,
            page_count: 0,
            page_size: 0,
            table_stats: Vec::new(),
            size_mb: 0.0,
            last_updated: chrono::Local::now(),
        }
    }
}

/// 表统计信息
#[derive(Debug, Clone)]
pub struct TableStats {
    pub table_name: String,
    pub record_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_storage_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = DatabaseConfig {
            database_path: db_path.to_string_lossy().to_string(),
            ..Default::default()
        };

        let mut manager = StorageManager::new(config).unwrap();
        assert!(manager.initialize().is_ok());
    }

    #[test]
    fn test_database_stats() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = DatabaseConfig {
            database_path: db_path.to_string_lossy().to_string(),
            ..Default::default()
        };

        let mut manager = StorageManager::new(config).unwrap();
        manager.initialize().unwrap();

        let stats = manager.get_database_stats().unwrap();
        assert!(stats.database_size > 0);
        assert!(stats.page_count > 0);
        assert!(stats.page_size > 0);
    }

    #[test]
    fn test_integrity_check() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = DatabaseConfig {
            database_path: db_path.to_string_lossy().to_string(),
            ..Default::default()
        };

        let mut manager = StorageManager::new(config).unwrap();
        manager.initialize().unwrap();

        assert!(manager.check_integrity().unwrap());
    }

    #[test]
    fn test_backup_restore() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let backup_path = temp_dir.path().join("backup.db");

        let config = DatabaseConfig {
            database_path: db_path.to_string_lossy().to_string(),
            ..Default::default()
        };

        let mut manager = StorageManager::new(config).unwrap();
        manager.initialize().unwrap();

        // 备份
        assert!(manager.backup_database(&backup_path).is_ok());
        assert!(backup_path.exists());

        // 恢复
        assert!(manager.restore_database(&backup_path).is_ok());
    }
}
