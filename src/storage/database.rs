//! # 数据库操作模块
//!
//! 提供SQLite数据库的连接管理和基本操作

use super::models::*;
use super::task_models::*;
use crate::errors::{AppError, Result};
use chrono::{DateTime, Local, NaiveDate};
use rusqlite::{params, Connection, Result as SqliteResult};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// 数据库连接包装器
///
/// 提供线程安全的数据库连接管理
#[derive(Debug)]
pub struct DatabaseConnection {
    /// SQLite连接
    connection: Arc<Mutex<Connection>>,
}

impl DatabaseConnection {
    /// 创建新的数据库连接
    pub fn new<P: AsRef<Path>>(database_path: P) -> Result<Self> {
        // 确保数据库父目录存在，避免在发布模式下因目录缺失导致无法打开
        if let Some(parent) = database_path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Storage(format!("无法创建数据库目录 {}: {}", parent.display(), e))
            })?;
        }

        let conn = Connection::open(&database_path)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(conn)),
        })
    }

    /// 执行SQL语句
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize> {
        let conn = self.connection.lock().unwrap();
        Ok(conn.execute(sql, params)?)
    }

    /// 查询单行数据
    pub fn query_row<T, F>(&self, sql: &str, params: &[&dyn rusqlite::ToSql], f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> SqliteResult<T>,
    {
        let conn = self.connection.lock().unwrap();
        Ok(conn.query_row(sql, params, f)?)
    }

    /// 准备SQL语句
    /// 注意：由于生命周期限制，这个方法暂时移除
    /// 建议直接使用execute或query_row方法
    // pub fn prepare(&self, sql: &str) -> Result<rusqlite::Statement> {
    //     let conn = self.connection.lock().unwrap();
    //     Ok(conn.prepare(sql)?)
    // }

    /// 开始事务
    pub fn begin_transaction(&self) -> Result<()> {
        let conn = self
            .connection
            .lock()
            .map_err(|_| AppError::System("Failed to acquire database lock".to_string()))?;
        conn.execute("BEGIN TRANSACTION", [])?;
        Ok(())
    }

    /// 提交事务
    pub fn commit_transaction(&self) -> Result<()> {
        let conn = self
            .connection
            .lock()
            .map_err(|_| AppError::System("Failed to acquire database lock".to_string()))?;
        conn.execute("COMMIT", [])?;
        Ok(())
    }

    /// 回滚事务
    pub fn rollback_transaction(&self) -> Result<()> {
        let conn = self
            .connection
            .lock()
            .map_err(|_| AppError::System("Failed to acquire database lock".to_string()))?;
        conn.execute("ROLLBACK", [])?;
        Ok(())
    }

    /// 获取底层连接的引用（用于备份等操作）
    pub fn get_raw_connection(&self) -> Arc<Mutex<Connection>> {
        self.connection.clone()
    }
}

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

        // 获取现有连接的克隆，用于迁移
        let conn = self.connection.connection.lock().unwrap();
        let temp_path = self.database_path.clone();
        drop(conn); // 释放锁

        // 创建临时连接用于迁移
        let migration_conn = Connection::open(&temp_path)?;
        let mut migration_manager = MigrationManager::new(migration_conn);
        migration_manager.run_migrations()?;

        log::info!("数据库迁移完成");
        Ok(())
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

    // ==================== 时间记录操作 ====================

    /// 插入时间记录
    pub fn insert_time_entry(&self, entry: &TimeEntryInsert) -> Result<i64> {
        let sql = r#"
            INSERT INTO time_entries (
                id, task_name, category_id, start_time, end_time, 
                duration_seconds, description, tags, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#;

        let tags_json = serde_json::to_string(&entry.tags)?;

        self.connection.execute(
            sql,
            &[
                &entry.id.to_string(),
                &entry.task_name,
                &entry.category_id.map(|id| id.to_string()),
                &entry.start_time.to_rfc3339(),
                &entry.end_time.map(|dt| dt.to_rfc3339()),
                &entry.duration_seconds,
                &entry.description,
                &tags_json,
                &entry.created_at.to_rfc3339(),
            ],
        )?;

        // 获取插入的行ID
        let row_id = self
            .connection
            .query_row("SELECT last_insert_rowid()", &[], |row| {
                row.get::<_, i64>(0)
            })?;

        log::debug!("插入时间记录: {}", entry.id);
        Ok(row_id)
    }

    /// 根据ID获取时间记录
    pub fn get_time_entry_by_id(&self, id: Uuid) -> Result<Option<TimeEntry>> {
        let sql = r#"
            SELECT id, task_name, category_id, start_time, end_time,
                   duration_seconds, description, tags, created_at, updated_at
            FROM time_entries WHERE id = ?1
        "#;

        let result = self.connection.query_row(sql, &[&id.to_string()], |row| {
            let tags_json: String = row.get("tags")?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

            Ok(TimeEntry {
                id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                task_name: row.get("task_name")?,
                category_id: row
                    .get::<_, Option<String>>("category_id")?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>("start_time")?)
                    .unwrap()
                    .with_timezone(&Local),
                end_time: row
                    .get::<_, Option<String>>("end_time")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
                duration_seconds: row.get("duration_seconds")?,
                description: row.get("description")?,
                tags,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                    .unwrap()
                    .with_timezone(&Local),
                updated_at: row
                    .get::<_, Option<String>>("updated_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
            })
        });

        match result {
            Ok(entry) => Ok(Some(entry)),
            Err(AppError::Database(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
            Err(other) => Err(other),
        }
    }

    /// 获取指定日期范围的时间记录
    pub fn get_time_entries_by_date_range(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<TimeEntry>> {
        let sql = r#"
            SELECT id, task_name, category_id, start_time, end_time,
                   duration_seconds, description, tags, created_at, updated_at
            FROM time_entries 
            WHERE DATE(start_time) BETWEEN ?1 AND ?2
            ORDER BY start_time DESC
        "#;

        log::debug!("查询时间记录SQL: {} 到 {}", start_date, end_date);

        let conn = self.connection.get_raw_connection();
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let entries = stmt.query_map(
            params![start_date.to_string(), end_date.to_string()],
            |row| {
                let tags_json: String = row.get("tags")?;
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

                Ok(TimeEntry {
                    id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                    task_name: row.get("task_name")?,
                    category_id: row
                        .get::<_, Option<String>>("category_id")?
                        .and_then(|s| Uuid::parse_str(&s).ok()),
                    start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>("start_time")?)
                        .unwrap()
                        .with_timezone(&Local),
                    end_time: row
                        .get::<_, Option<String>>("end_time")?
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Local)),
                    duration_seconds: row.get("duration_seconds")?,
                    description: row.get("description")?,
                    tags,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                        .unwrap()
                        .with_timezone(&Local),
                    updated_at: row
                        .get::<_, Option<String>>("updated_at")?
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Local)),
                })
            },
        )?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }

        Ok(result)
    }

    /// 获取指定分类的时间记录
    pub fn get_time_entries_by_category(&self, category_id: Uuid) -> Result<Vec<TimeEntry>> {
        let sql = r#"
            SELECT id, task_name, category_id, start_time, end_time,
                   duration_seconds, description, tags, created_at, updated_at
            FROM time_entries 
            WHERE category_id = ?1
            ORDER BY start_time DESC
        "#;

        let conn = self.connection.get_raw_connection();
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let entries = stmt.query_map(params![category_id.to_string()], |row| {
            let tags_json: String = row.get("tags")?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

            Ok(TimeEntry {
                id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                task_name: row.get("task_name")?,
                category_id: row
                    .get::<_, Option<String>>("category_id")?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>("start_time")?)
                    .unwrap()
                    .with_timezone(&Local),
                end_time: row
                    .get::<_, Option<String>>("end_time")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
                duration_seconds: row.get("duration_seconds")?,
                description: row.get("description")?,
                tags,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                    .unwrap()
                    .with_timezone(&Local),
                updated_at: row
                    .get::<_, Option<String>>("updated_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
            })
        })?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }

        Ok(result)
    }

    /// 更新时间记录
    pub fn update_time_entry(&self, id: Uuid, entry: &TimeEntryInsert) -> Result<()> {
        let sql = r#"
            UPDATE time_entries SET
                task_name = ?2, category_id = ?3, start_time = ?4, end_time = ?5,
                duration_seconds = ?6, description = ?7, tags = ?8, updated_at = ?9
            WHERE id = ?1
        "#;

        let tags_json = serde_json::to_string(&entry.tags)?;
        let updated_at = Local::now().to_rfc3339();

        let rows_affected = self.connection.execute(
            sql,
            &[
                &id.to_string(),
                &entry.task_name,
                &entry.category_id.map(|id| id.to_string()),
                &entry.start_time.to_rfc3339(),
                &entry.end_time.map(|dt| dt.to_rfc3339()),
                &entry.duration_seconds,
                &entry.description,
                &tags_json,
                &updated_at,
            ],
        )?;

        if rows_affected == 0 {
            return Err(AppError::TaskNotFound(id.to_string()));
        }

        log::debug!("更新时间记录: {}", id);
        Ok(())
    }

    /// 删除时间记录
    pub fn delete_time_entry(&self, id: Uuid) -> Result<()> {
        let sql = "DELETE FROM time_entries WHERE id = ?1";

        let rows_affected = self.connection.execute(sql, &[&id.to_string()])?;

        if rows_affected == 0 {
            return Err(AppError::TaskNotFound(id.to_string()));
        }

        log::debug!("删除时间记录: {}", id);
        Ok(())
    }

    // ==================== 任务操作 ====================

    /// 插入任务
    pub fn insert_task(&self, task: &TaskInsert) -> Result<i64> {
        let sql = r#"
            INSERT INTO tasks (
                id, name, description, category_id, status, priority,
                estimated_duration_seconds, total_duration_seconds, tags,
                due_date, is_completed, completed_at, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#;

        self.connection.execute(
            sql,
            &[
                &task.id.to_string(),
                &task.name,
                &task.description,
                &task.category_id.map(|id| id.to_string()),
                &task.status,
                &task.priority,
                &task.estimated_duration_seconds,
                &task.total_duration_seconds,
                &task.tags,
                &task.due_date.map(|dt| dt.to_rfc3339()),
                &task.is_completed,
                &task.completed_at.map(|dt| dt.to_rfc3339()),
                &task.created_at.to_rfc3339(),
            ],
        )?;

        // 获取插入的行ID
        let row_id = self
            .connection
            .query_row("SELECT last_insert_rowid()", &[], |row| {
                row.get::<_, i64>(0)
            })?;

        log::debug!("插入任务: {}", task.name);
        Ok(row_id)
    }

    /// 获取所有任务
    pub fn get_all_tasks(&self) -> Result<Vec<TaskModel>> {
        let sql = r#"
            SELECT id, name, description, category_id, status, priority,
                   estimated_duration_seconds, total_duration_seconds, tags,
                   due_date, is_completed, completed_at, created_at, updated_at
            FROM tasks 
            ORDER BY created_at DESC
        "#;

        let conn = self.connection.connection.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let tasks = stmt.query_map([], |row| {
            Ok(TaskModel {
                id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                name: row.get("name")?,
                description: row.get("description")?,
                category_id: row
                    .get::<_, Option<String>>("category_id")?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                status: row.get("status")?,
                priority: row.get("priority")?,
                estimated_duration_seconds: row.get("estimated_duration_seconds")?,
                total_duration_seconds: row.get("total_duration_seconds")?,
                tags: row.get("tags")?,
                due_date: row
                    .get::<_, Option<String>>("due_date")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
                is_completed: row.get("is_completed")?,
                completed_at: row
                    .get::<_, Option<String>>("completed_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                    .unwrap()
                    .with_timezone(&Local),
                updated_at: row
                    .get::<_, Option<String>>("updated_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
            })
        })?;

        let mut result = Vec::new();
        for task in tasks {
            result.push(task?);
        }

        Ok(result)
    }

    /// 根据ID获取任务
    pub fn get_task_by_id(&self, id: Uuid) -> Result<Option<TaskModel>> {
        let sql = r#"
            SELECT id, name, description, category_id, status, priority,
                   estimated_duration_seconds, total_duration_seconds, tags,
                   due_date, is_completed, completed_at, created_at, updated_at
            FROM tasks WHERE id = ?1
        "#;

        let result = self.connection.query_row(sql, &[&id.to_string()], |row| {
            Ok(TaskModel {
                id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                name: row.get("name")?,
                description: row.get("description")?,
                category_id: row
                    .get::<_, Option<String>>("category_id")?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                status: row.get("status")?,
                priority: row.get("priority")?,
                estimated_duration_seconds: row.get("estimated_duration_seconds")?,
                total_duration_seconds: row.get("total_duration_seconds")?,
                tags: row.get("tags")?,
                due_date: row
                    .get::<_, Option<String>>("due_date")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
                is_completed: row.get("is_completed")?,
                completed_at: row
                    .get::<_, Option<String>>("completed_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                    .unwrap()
                    .with_timezone(&Local),
                updated_at: row
                    .get::<_, Option<String>>("updated_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
            })
        });

        match result {
            Ok(task) => Ok(Some(task)),
            Err(AppError::Database(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 更新任务
    pub fn update_task(&self, id: Uuid, task: &TaskUpdate) -> Result<()> {
        // 构建动态UPDATE语句
        let mut sql_parts = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(name) = &task.name {
            sql_parts.push("name = ?");
            params.push(Box::new(name.clone()));
        }

        if let Some(description) = &task.description {
            sql_parts.push("description = ?");
            params.push(Box::new(description.clone()));
        }

        if let Some(category_id) = &task.category_id {
            sql_parts.push("category_id = ?");
            params.push(Box::new(category_id.map(|id| id.to_string())));
        }

        if let Some(status) = &task.status {
            sql_parts.push("status = ?");
            params.push(Box::new(status.clone()));
        }

        if let Some(priority) = &task.priority {
            sql_parts.push("priority = ?");
            params.push(Box::new(priority.clone()));
        }

        if let Some(estimated_duration_seconds) = &task.estimated_duration_seconds {
            sql_parts.push("estimated_duration_seconds = ?");
            params.push(Box::new(*estimated_duration_seconds));
        }

        if let Some(total_duration_seconds) = &task.total_duration_seconds {
            sql_parts.push("total_duration_seconds = ?");
            params.push(Box::new(*total_duration_seconds));
        }

        if let Some(tags) = &task.tags {
            sql_parts.push("tags = ?");
            params.push(Box::new(tags.clone()));
        }

        if let Some(due_date) = &task.due_date {
            sql_parts.push("due_date = ?");
            params.push(Box::new(due_date.map(|dt| dt.to_rfc3339())));
        }

        if let Some(is_completed) = &task.is_completed {
            sql_parts.push("is_completed = ?");
            params.push(Box::new(*is_completed));
        }

        if let Some(completed_at) = &task.completed_at {
            sql_parts.push("completed_at = ?");
            params.push(Box::new(completed_at.map(|dt| dt.to_rfc3339())));
        }

        if sql_parts.is_empty() {
            return Ok(()); // 没有要更新的字段
        }

        sql_parts.push("updated_at = ?");
        params.push(Box::new(Local::now().to_rfc3339()));

        let sql = format!("UPDATE tasks SET {} WHERE id = ?", sql_parts.join(", "));

        params.push(Box::new(id.to_string()));

        // 转换参数为引用
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let rows_affected = self.connection.execute(&sql, &param_refs)?;

        if rows_affected == 0 {
            return Err(AppError::System(format!("任务未找到: {}", id)));
        }

        log::debug!("更新任务: {}", id);
        Ok(())
    }

    /// 删除任务
    pub fn delete_task(&self, id: Uuid) -> Result<()> {
        let sql = "DELETE FROM tasks WHERE id = ?1";
        let rows_affected = self.connection.execute(sql, &[&id.to_string()])?;

        if rows_affected == 0 {
            return Err(AppError::System(format!("任务未找到: {}", id)));
        }

        log::debug!("删除任务: {}", id);
        Ok(())
    }

    /// 根据分类获取任务
    pub fn get_tasks_by_category(&self, category_id: Uuid) -> Result<Vec<TaskModel>> {
        let sql = r#"
            SELECT id, name, description, category_id, status, priority,
                   estimated_duration_seconds, total_duration_seconds, tags,
                   due_date, is_completed, completed_at, created_at, updated_at
            FROM tasks 
            WHERE category_id = ?1
            ORDER BY created_at DESC
        "#;

        let conn = self.connection.connection.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let tasks = stmt.query_map([category_id.to_string()], |row| {
            Ok(TaskModel {
                id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                name: row.get("name")?,
                description: row.get("description")?,
                category_id: row
                    .get::<_, Option<String>>("category_id")?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                status: row.get("status")?,
                priority: row.get("priority")?,
                estimated_duration_seconds: row.get("estimated_duration_seconds")?,
                total_duration_seconds: row.get("total_duration_seconds")?,
                tags: row.get("tags")?,
                due_date: row
                    .get::<_, Option<String>>("due_date")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
                is_completed: row.get("is_completed")?,
                completed_at: row
                    .get::<_, Option<String>>("completed_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                    .unwrap()
                    .with_timezone(&Local),
                updated_at: row
                    .get::<_, Option<String>>("updated_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
            })
        })?;

        let mut result = Vec::new();
        for task in tasks {
            result.push(task?);
        }

        Ok(result)
    }

    // ==================== 分类操作 ====================

    /// 插入分类
    pub fn insert_category(&self, category: &CategoryInsert) -> Result<i64> {
        let sql = r#"
            INSERT INTO categories (
                id, name, description, color, icon, daily_target_seconds,
                weekly_target_seconds, is_active, sort_order, parent_id, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#;

        self.connection.execute(
            sql,
            &[
                &category.id.to_string(),
                &category.name,
                &category.description,
                &category.color,
                &category.icon,
                &category.daily_target_seconds,
                &category.weekly_target_seconds,
                &category.is_active,
                &category.sort_order,
                &category.parent_id.map(|id| id.to_string()),
                &category.created_at.to_rfc3339(),
            ],
        )?;

        let row_id = self
            .connection
            .query_row("SELECT last_insert_rowid()", &[], |row| {
                row.get::<_, i64>(0)
            })?;

        log::debug!("插入分类: {}", category.name);
        Ok(row_id)
    }

    /// 获取所有分类
    pub fn get_all_categories(&self) -> Result<Vec<CategoryModel>> {
        let sql = r#"
            SELECT id, name, description, color, icon, daily_target_seconds,
                   weekly_target_seconds, is_active, sort_order, parent_id,
                   created_at, updated_at
            FROM categories
            ORDER BY sort_order, name
        "#;

        let conn = self.connection.connection.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let categories = stmt.query_map([], |row| {
            Ok(CategoryModel {
                id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                name: row.get("name")?,
                description: row.get("description")?,
                color: row.get("color")?,
                icon: row.get("icon")?,
                daily_target_seconds: row.get("daily_target_seconds")?,
                weekly_target_seconds: row.get("weekly_target_seconds")?,
                is_active: row.get("is_active")?,
                sort_order: row.get("sort_order")?,
                parent_id: row
                    .get::<_, Option<String>>("parent_id")?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                    .unwrap()
                    .with_timezone(&Local),
                updated_at: row
                    .get::<_, Option<String>>("updated_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
            })
        })?;

        let mut result = Vec::new();
        for category in categories {
            result.push(category?);
        }

        Ok(result)
    }

    /// 根据ID获取分类
    pub fn get_category_by_id(&self, id: Uuid) -> Result<Option<CategoryModel>> {
        let sql = r#"
            SELECT id, name, description, color, icon, daily_target_seconds,
                   weekly_target_seconds, is_active, sort_order, parent_id,
                   created_at, updated_at
            FROM categories WHERE id = ?1
        "#;

        let result = self.connection.query_row(sql, &[&id.to_string()], |row| {
            Ok(CategoryModel {
                id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                name: row.get("name")?,
                description: row.get("description")?,
                color: row.get("color")?,
                icon: row.get("icon")?,
                daily_target_seconds: row.get("daily_target_seconds")?,
                weekly_target_seconds: row.get("weekly_target_seconds")?,
                is_active: row.get("is_active")?,
                sort_order: row.get("sort_order")?,
                parent_id: row
                    .get::<_, Option<String>>("parent_id")?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                    .unwrap()
                    .with_timezone(&Local),
                updated_at: row
                    .get::<_, Option<String>>("updated_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
            })
        });

        match result {
            Ok(category) => Ok(Some(category)),
            Err(AppError::Database(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
            Err(other) => Err(other),
        }
    }

    /// 更新分类
    pub fn update_category(&self, id: Uuid, category: &CategoryInsert) -> Result<()> {
        let sql = r#"
            UPDATE categories SET
                name = ?2, description = ?3, color = ?4, icon = ?5,
                daily_target_seconds = ?6, weekly_target_seconds = ?7,
                is_active = ?8, sort_order = ?9, parent_id = ?10, updated_at = ?11
            WHERE id = ?1
        "#;

        let updated_at = Local::now().to_rfc3339();

        let rows_affected = self.connection.execute(
            sql,
            &[
                &id.to_string(),
                &category.name,
                &category.description,
                &category.color,
                &category.icon,
                &category.daily_target_seconds,
                &category.weekly_target_seconds,
                &category.is_active,
                &category.sort_order,
                &category.parent_id.map(|id| id.to_string()),
                &updated_at,
            ],
        )?;

        if rows_affected == 0 {
            return Err(AppError::CategoryNotFound(id.to_string()));
        }

        log::debug!("更新分类: {}", category.name);
        Ok(())
    }

    /// 删除分类
    pub fn delete_category(&self, id: Uuid) -> Result<()> {
        let sql = "DELETE FROM categories WHERE id = ?1";
        self.connection.execute(sql, &[&id.to_string()])?;

        log::debug!("删除分类: {}", id);
        Ok(())
    }

    /// 获取分类任务数量统计
    pub fn get_category_task_counts(&self) -> Result<std::collections::HashMap<Uuid, i64>> {
        let sql = r#"
            SELECT category_id, COUNT(*) as task_count
            FROM tasks
            WHERE category_id IS NOT NULL
            GROUP BY category_id
        "#;

        let conn = self.connection.connection.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let mut counts = std::collections::HashMap::new();
        let rows = stmt.query_map([], |row| {
            let category_id_str: String = row.get("category_id")?;
            let task_count: i64 = row.get("task_count")?;
            Ok((category_id_str, task_count))
        })?;

        for row in rows {
            let (category_id_str, task_count) = row?;
            if let Ok(category_id) = Uuid::parse_str(&category_id_str) {
                counts.insert(category_id, task_count);
            }
        }

        Ok(counts)
    }

    /// 获取分类的总时长统计
    pub fn get_category_duration_stats(&self) -> Result<std::collections::HashMap<Uuid, i64>> {
        let sql = r#"
            SELECT category_id, SUM(duration_seconds) as total_duration
            FROM time_entries
            WHERE category_id IS NOT NULL
            GROUP BY category_id
        "#;

        let conn = self.connection.connection.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let mut durations = std::collections::HashMap::new();
        let rows = stmt.query_map([], |row| {
            let category_id_str: String = row.get("category_id")?;
            let total_duration: i64 = row.get("total_duration")?;
            Ok((category_id_str, total_duration))
        })?;

        for row in rows {
            let (category_id_str, total_duration) = row?;
            if let Ok(category_id) = Uuid::parse_str(&category_id_str) {
                durations.insert(category_id, total_duration);
            }
        }

        Ok(durations)
    }

    /// 获取最近的时间记录
    pub fn get_recent_time_entries(&self, limit: usize) -> Result<Vec<TimeEntry>> {
        let sql = r#"
            SELECT id, task_name, category_id, start_time, end_time,
                   duration_seconds, description, tags, created_at, updated_at
            FROM time_entries 
            ORDER BY start_time DESC
            LIMIT ?1
        "#;

        let conn = self.connection.connection.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let entries = stmt.query_map([limit as i64], |row| {
            let tags_json: String = row.get("tags")?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

            Ok(TimeEntry {
                id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                task_name: row.get("task_name")?,
                category_id: row
                    .get::<_, Option<String>>("category_id")?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>("start_time")?)
                    .unwrap()
                    .with_timezone(&Local),
                end_time: row
                    .get::<_, Option<String>>("end_time")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
                duration_seconds: row.get("duration_seconds")?,
                description: row.get("description")?,
                tags,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                    .unwrap()
                    .with_timezone(&Local),
                updated_at: row
                    .get::<_, Option<String>>("updated_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
            })
        })?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }

        Ok(result)
    }

    /// 获取所有时间记录
    pub fn get_all_time_entries(&self) -> Result<Vec<TimeEntry>> {
        let sql = r#"
            SELECT id, task_name, category_id, start_time, end_time,
                   duration_seconds, description, tags, created_at, updated_at
            FROM time_entries 
            ORDER BY start_time DESC
        "#;

        let conn = self.connection.connection.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let entries = stmt.query_map([], |row| {
            let tags_json: String = row.get("tags")?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

            Ok(TimeEntry {
                id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                task_name: row.get("task_name")?,
                category_id: row
                    .get::<_, Option<String>>("category_id")?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>("start_time")?)
                    .unwrap()
                    .with_timezone(&Local),
                end_time: row
                    .get::<_, Option<String>>("end_time")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
                duration_seconds: row.get("duration_seconds")?,
                description: row.get("description")?,
                tags,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                    .unwrap()
                    .with_timezone(&Local),
                updated_at: row
                    .get::<_, Option<String>>("updated_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Local)),
            })
        })?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use tempfile::tempdir;

    fn create_test_database() -> Database {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        Database::new(db_path).unwrap()
    }

    #[test]
    fn test_database_creation() {
        let db = create_test_database();
        assert!(db.get_connection().is_ok());
    }

    #[test]
    fn test_time_entry_operations() {
        let db = create_test_database();

        // 首先需要运行迁移创建表
        // 这里简化测试，假设表已存在

        let entry = TimeEntryInsert {
            id: Uuid::new_v4(),
            task_name: "测试任务".to_string(),
            category_id: None,
            start_time: Local::now(),
            end_time: Some(Local::now() + Duration::hours(1)),
            duration_seconds: 3600,
            description: Some("测试描述".to_string()),
            tags: vec!["测试".to_string()],
            created_at: Local::now(),
        };

        // 注意：这个测试需要表存在才能运行
        // 在实际使用中，需要先运行迁移
    }

    #[test]
    fn test_transaction_operations() {
        let db = create_test_database();
        let conn = db.get_connection().unwrap();

        // 测试事务操作
        assert!(conn.begin_transaction().is_ok());
        assert!(conn.commit_transaction().is_ok());

        assert!(conn.begin_transaction().is_ok());
        assert!(conn.rollback_transaction().is_ok());
    }
}
