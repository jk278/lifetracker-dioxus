//! # 数据库模块
//!
//! 重构后的模块化数据库操作接口

pub mod connection;
pub mod time_entries;
pub mod tasks;
pub mod utils;

// 重新导出主要结构体和函数
pub use connection::DatabaseConnection;
pub use tasks::TasksRepository;
pub use time_entries::TimeEntriesRepository;

use crate::errors::{AppError, Result};
use std::path::Path;

/// 数据库管理器
///
/// 提供高级数据库操作接口
#[derive(Debug)]
pub struct Database {
    /// 数据库连接
    connection: DatabaseConnection,
    /// 数据库文件路径
    database_path: String,
}

impl Database {
    /// 创建新的数据库实例
    ///
    /// # 参数
    /// * `database_path` - 数据库文件路径
    pub fn new<P: AsRef<Path>>(database_path: P) -> Result<Self> {
        let path_str = database_path.as_ref().to_string_lossy().to_string();
        let connection = DatabaseConnection::new(database_path)?;

        Ok(Self {
            connection,
            database_path: path_str,
        })
    }

    /// 运行数据库迁移
    pub fn run_migrations(&self) -> Result<()> {
        use crate::storage::migrations::MigrationManager;

        // 使用写连接进行迁移
        self.connection.write(|conn| {
            let mut migration_manager = MigrationManager::new_with_connection(conn);
            migration_manager.run_migrations()?;
            log::info!("数据库迁移完成");
            Ok(())
        })
    }

    /// 获取数据库连接
    pub fn get_connection(&self) -> Result<&DatabaseConnection> {
        Ok(&self.connection)
    }

    /// 关闭数据库连接
    pub fn close(self) -> Result<()> {
        // 连接会在Drop时自动关闭
        log::debug!("数据库连接已关闭: {}", self.database_path);
        Ok(())
    }

    /// 获取时间记录仓库
    pub fn time_entries(&self) -> TimeEntriesRepository {
        TimeEntriesRepository::new(&self.connection)
    }

    /// 获取任务仓库
    pub fn tasks(&self) -> TasksRepository {
        TasksRepository::new(&self.connection)
    }

    // ==================== 时间记录操作代理方法 ====================

    /// 插入时间记录
    pub fn insert_time_entry(&self, entry: &crate::storage::models::TimeEntryInsert) -> Result<i64> {
        self.time_entries().insert(entry)
    }

    /// 根据ID获取时间记录
    pub fn get_time_entry_by_id(&self, id: uuid::Uuid) -> Result<Option<crate::storage::models::TimeEntry>> {
        self.time_entries().get_by_id(id)
    }

    /// 获取指定日期范围的时间记录
    pub fn get_time_entries_by_date_range(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<Vec<crate::storage::models::TimeEntry>> {
        self.time_entries().get_by_date_range(start_date, end_date)
    }

    /// 获取指定分类的时间记录
    pub fn get_time_entries_by_category(&self, category_id: uuid::Uuid) -> Result<Vec<crate::storage::models::TimeEntry>> {
        self.time_entries().get_by_category(category_id)
    }

    /// 更新时间记录
    pub fn update_time_entry(&self, id: uuid::Uuid, entry: &crate::storage::models::TimeEntryInsert) -> Result<()> {
        self.time_entries().update(id, entry)
    }

    /// 删除时间记录
    pub fn delete_time_entry(&self, id: uuid::Uuid) -> Result<()> {
        self.time_entries().delete(id)
    }

    // ==================== 任务操作代理方法 ====================

    /// 插入任务
    pub fn insert_task(&self, task: &crate::storage::task_models::TaskInsert) -> Result<i64> {
        self.tasks().insert(task)
    }

    /// 获取所有任务
    pub fn get_all_tasks(&self) -> Result<Vec<crate::storage::task_models::TaskModel>> {
        self.tasks().get_all()
    }

    /// 根据ID获取任务
    pub fn get_task_by_id(&self, id: uuid::Uuid) -> Result<Option<crate::storage::task_models::TaskModel>> {
        self.tasks().get_by_id(id)
    }

    /// 更新任务
    pub fn update_task(&self, id: uuid::Uuid, task: &crate::storage::task_models::TaskUpdate) -> Result<()> {
        self.tasks().update(id, task)
    }

    /// 删除任务
    pub fn delete_task(&self, id: uuid::Uuid) -> Result<()> {
        self.tasks().delete(id)
    }

    /// 根据分类获取任务
    pub fn get_tasks_by_category(&self, category_id: uuid::Uuid) -> Result<Vec<crate::storage::task_models::TaskModel>> {
        self.tasks().get_by_category(category_id)
    }

    /// 获取最近的时间记录
    pub fn get_recent_time_entries(&self, limit: usize) -> Result<Vec<crate::storage::models::TimeEntry>> {
        let sql = r#"
            SELECT id, task_name, category_id, start_time, end_time,
                   duration_seconds, description, tags, created_at, updated_at
            FROM time_entries 
            ORDER BY created_at DESC
            LIMIT ?1
        "#;

        self.connection.read(|conn| {
            let mut stmt = conn.prepare(sql)?;
            let entries = stmt.query_map([limit as i64], |row| {
                let tags_json: String = row.get("tags")?;
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

                Ok(crate::storage::models::TimeEntry {
                    id: uuid::Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                    task_name: row.get("task_name")?,
                    category_id: row
                        .get::<_, Option<String>>("category_id")?
                        .and_then(|s| uuid::Uuid::parse_str(&s).ok()),
                    start_time: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>("start_time")?)
                        .unwrap()
                        .with_timezone(&chrono::Local),
                    end_time: row
                        .get::<_, Option<String>>("end_time")?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Local)),
                    duration_seconds: row.get("duration_seconds")?,
                    description: row.get("description")?,
                    tags,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                        .unwrap()
                        .with_timezone(&chrono::Local),
                    updated_at: row
                        .get::<_, Option<String>>("updated_at")?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Local)),
                })
            })?;

            let mut result = Vec::new();
            for entry in entries {
                result.push(entry?);
            }
            Ok(result)
        })
    }

    /// 获取所有时间记录
    pub fn get_all_time_entries(&self) -> Result<Vec<crate::storage::models::TimeEntry>> {
        let sql = r#"
            SELECT id, task_name, category_id, start_time, end_time,
                   duration_seconds, description, tags, created_at, updated_at
            FROM time_entries 
            ORDER BY start_time DESC
        "#;

        self.connection.read(|conn| {
            let mut stmt = conn.prepare(sql)?;
            let entries = stmt.query_map([], |row| {
                let tags_json: String = row.get("tags")?;
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

                Ok(crate::storage::models::TimeEntry {
                    id: uuid::Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                    task_name: row.get("task_name")?,
                    category_id: row
                        .get::<_, Option<String>>("category_id")?
                        .and_then(|s| uuid::Uuid::parse_str(&s).ok()),
                    start_time: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>("start_time")?)
                        .unwrap()
                        .with_timezone(&chrono::Local),
                    end_time: row
                        .get::<_, Option<String>>("end_time")?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Local)),
                    duration_seconds: row.get("duration_seconds")?,
                    description: row.get("description")?,
                    tags,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                        .unwrap()
                        .with_timezone(&chrono::Local),
                    updated_at: row
                        .get::<_, Option<String>>("updated_at")?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Local)),
                })
            })?;

            let mut result = Vec::new();
            for entry in entries {
                result.push(entry?);
            }
            Ok(result)
        })
    }

    /// 获取所有分类
    pub fn get_all_categories(&self) -> Result<Vec<crate::storage::models::CategoryModel>> {
        let sql = r#"
            SELECT id, name, description, color, icon, created_at, updated_at,
                   is_active, sort_order, parent_id, daily_target_seconds, weekly_target_seconds
            FROM categories 
            WHERE is_active = true
            ORDER BY sort_order ASC, name ASC
        "#;

        self.connection.read(|conn| {
            let mut stmt = conn.prepare(sql)?;
            let categories = stmt.query_map([], |row| {
                Ok(crate::storage::models::CategoryModel {
                    id: uuid::Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                    name: row.get("name")?,
                    description: row.get("description")?,
                    color: row.get("color")?,
                    icon: row.get("icon")?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                        .unwrap()
                        .with_timezone(&chrono::Local),
                    updated_at: row
                        .get::<_, Option<String>>("updated_at")?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Local)),
                    is_active: row.get("is_active")?,
                    sort_order: row.get("sort_order")?,
                    parent_id: row
                        .get::<_, Option<String>>("parent_id")?
                        .and_then(|s| uuid::Uuid::parse_str(&s).ok()),
                    daily_target_seconds: row.get("daily_target_seconds")?,
                    weekly_target_seconds: row.get("weekly_target_seconds")?,
                })
            })?;

            let mut result = Vec::new();
            for category in categories {
                result.push(category?);
            }
            Ok(result)
        })
    }

    // ==================== 笔记操作（占位方法） ====================

    /// 插入笔记
    pub fn insert_note(&self, _note: &crate::storage::models::Note) -> Result<i64> {
        // TODO: 实现笔记插入逻辑
        Err(crate::errors::AppError::System("笔记功能尚未实现".to_string()))
    }

    /// 获取所有笔记
    pub fn get_all_notes(&self) -> Result<Vec<crate::storage::models::Note>> {
        // TODO: 实现获取所有笔记逻辑
        Ok(vec![])
    }

    /// 根据ID获取笔记
    pub fn get_note_by_id(&self, _id: uuid::Uuid) -> Result<Option<crate::storage::models::Note>> {
        // TODO: 实现根据ID获取笔记逻辑
        Ok(None)
    }

    /// 更新笔记
    pub fn update_note(&self, _id: uuid::Uuid, _update: &crate::storage::models::NoteUpdate) -> Result<()> {
        // TODO: 实现笔记更新逻辑
        Err(crate::errors::AppError::System("笔记功能尚未实现".to_string()))
    }

    /// 删除笔记
    pub fn delete_note(&self, _id: uuid::Uuid) -> Result<()> {
        // TODO: 实现笔记删除逻辑
        Err(crate::errors::AppError::System("笔记功能尚未实现".to_string()))
    }

    /// 搜索笔记
    pub fn search_notes(&self, _query: &crate::storage::models::NoteQuery) -> Result<Vec<crate::storage::models::Note>> {
        // TODO: 实现笔记搜索逻辑
        Ok(vec![])
    }

    /// 获取笔记统计信息
    pub fn get_notes_stats(&self) -> Result<crate::storage::models::NoteStats> {
        // TODO: 实现获取笔记统计逻辑
        Ok(crate::storage::models::NoteStats {
            total_notes: 0,
            total_words: 0,
            total_characters: 0,
            tags_count: 0,
            last_updated: chrono::Local::now(),
        })
    }

    /// 获取所有笔记标签
    pub fn get_all_note_tags(&self) -> Result<Vec<String>> {
        // TODO: 实现获取所有笔记标签逻辑
        Ok(vec![])
    }

    // ==================== 遗留的方法（待迁移） ====================
    // 这些方法暂时保留以确保向后兼容，将在后续版本中移除

    /// 执行SQL语句（写操作）
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize> {
        self.connection.execute(sql, params)
    }

    /// 查询单行数据（读操作）
    pub fn query_row<T, F>(&self, sql: &str, params: &[&dyn rusqlite::ToSql], f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        self.connection.query_row(sql, params, f)
    }

    /// 开始事务
    pub fn begin_transaction(&self) -> Result<()> {
        self.connection.begin_transaction()
    }

    /// 提交事务
    pub fn commit_transaction(&self) -> Result<()> {
        self.connection.commit_transaction()
    }

    /// 回滚事务
    pub fn rollback_transaction(&self) -> Result<()> {
        self.connection.rollback_transaction()
    }
}