//! # 数据库操作模块
//!
//! 提供SQLite数据库的连接管理和基本操作

use super::accounting_models::*;
use super::models::*;
use super::task_models::*;
use crate::errors::{AppError, Result};
use chrono::{DateTime, Local, NaiveDate};
use rusqlite::{params, Connection, Result as SqliteResult};
use serde_json;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// 数据库连接池
///
/// 提供读写分离的数据库连接管理
#[derive(Debug)]
pub struct DatabaseConnection {
    /// 数据库文件路径
    database_path: String,
    /// 写连接（互斥）
    write_connection: Arc<Mutex<Connection>>,
}

impl DatabaseConnection {
    /// 创建新的数据库连接池
    pub fn new<P: AsRef<Path>>(database_path: P) -> Result<Self> {
        let path_str = database_path.as_ref().to_string_lossy().to_string();

        // 确保数据库父目录存在
        if let Some(parent) = database_path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Storage(format!("无法创建数据库目录 {}: {}", parent.display(), e))
            })?;
        }

        // 创建写连接
        let write_conn = Connection::open(&database_path)?;

        // 配置数据库参数
        let _journal_mode: String =
            write_conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
        write_conn.busy_timeout(std::time::Duration::from_secs(30))?;
        write_conn.execute("PRAGMA foreign_keys=ON", [])?;
        write_conn.execute("PRAGMA synchronous=NORMAL", [])?;

        Ok(Self {
            database_path: path_str,
            write_connection: Arc::new(Mutex::new(write_conn)),
        })
    }

    /// 创建只读连接
    fn create_read_connection(&self) -> Result<Connection> {
        let conn = Connection::open(&self.database_path)?;
        // 只读连接配置
        let _journal_mode: String =
            conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
        conn.busy_timeout(std::time::Duration::from_secs(10))?;
        conn.execute("PRAGMA query_only=ON", [])?;
        Ok(conn)
    }

    /// 执行读操作（不需要锁）
    pub fn read<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let read_conn = self.create_read_connection()?;
        f(&read_conn)
    }

    /// 执行写操作（需要独占锁）
    pub fn write<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T>,
    {
        let mut write_conn = self
            .write_connection
            .lock()
            .map_err(|_| AppError::System("Failed to acquire write lock".to_string()))?;
        f(&mut *write_conn)
    }

    /// 执行SQL语句（写操作）
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize> {
        self.write(|conn| Ok(conn.execute(sql, params)?))
    }

    /// 查询单行数据（读操作）
    pub fn query_row<T, F>(&self, sql: &str, params: &[&dyn rusqlite::ToSql], f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> SqliteResult<T>,
    {
        self.read(|conn| Ok(conn.query_row(sql, params, f)?))
    }

    /// 开始事务（写操作）
    pub fn begin_transaction(&self) -> Result<()> {
        self.write(|conn| {
            conn.execute("BEGIN TRANSACTION", [])?;
            Ok(())
        })
    }

    /// 提交事务（写操作）
    pub fn commit_transaction(&self) -> Result<()> {
        self.write(|conn| {
            conn.execute("COMMIT", [])?;
            Ok(())
        })
    }

    /// 回滚事务（写操作）
    pub fn rollback_transaction(&self) -> Result<()> {
        self.write(|conn| {
            conn.execute("ROLLBACK", [])?;
            Ok(())
        })
    }

    /// 获取写连接的引用（用于迁移等特殊操作）
    pub fn get_raw_connection(&self) -> Arc<Mutex<Connection>> {
        self.write_connection.clone()
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

        // 使用写连接进行迁移
        self.connection.write(|conn| {
            let mut migration_manager = MigrationManager::new_with_connection(conn);
            migration_manager.run_migrations()?;
            log::info!("Database migration completed");
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
        log::debug!("Database connection closed: {}", self.database_path);
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

        log::debug!("Inserting time entry: {}", entry.id);
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

        log::debug!("Querying time entries SQL: {} to {}", start_date, end_date);

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

        log::debug!("Updating time entry: {}", id);
        Ok(())
    }

    /// 删除时间记录
    pub fn delete_time_entry(&self, id: Uuid) -> Result<()> {
        let sql = "DELETE FROM time_entries WHERE id = ?1";

        let rows_affected = self.connection.execute(sql, &[&id.to_string()])?;

        if rows_affected == 0 {
            return Err(AppError::TaskNotFound(id.to_string()));
        }

        log::debug!("Deleting time entry: {}", id);
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

        log::debug!("Inserting task: {}", task.name);
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

        self.connection.read(|conn| {
            let mut stmt = conn.prepare(sql)?;

            let task_iter = stmt.query_map([], |row| {
                let tags_json: String = row.get(8)?;
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

                Ok(TaskModel {
                    id: Self::uuid_from_str(row.get(0)?)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    category_id: row
                        .get::<_, Option<String>>(3)?
                        .map(|s| Self::uuid_from_str(s))
                        .transpose()?,
                    status: row.get(4)?,
                    priority: row.get(5)?,
                    estimated_duration_seconds: row.get(6)?,
                    total_duration_seconds: row.get(7)?,
                    tags: tags_json,
                    due_date: row
                        .get::<_, Option<String>>(9)?
                        .map(|s| Self::datetime_from_str(s))
                        .transpose()?,
                    is_completed: row.get(10)?,
                    completed_at: row
                        .get::<_, Option<String>>(11)?
                        .map(|s| Self::datetime_from_str(s))
                        .transpose()?,
                    created_at: Self::datetime_from_str(row.get(12)?)?,
                    updated_at: row
                        .get::<_, Option<String>>(13)?
                        .map(|s| Self::datetime_from_str(s))
                        .transpose()?,
                })
            })?;

            let mut tasks = Vec::new();
            for task_result in task_iter {
                tasks.push(task_result?);
            }

            log::debug!("Retrieved {} tasks", tasks.len());
            Ok(tasks)
        })
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

        log::debug!("Updating task: {}", id);
        Ok(())
    }

    /// 删除任务
    pub fn delete_task(&self, id: Uuid) -> Result<()> {
        let sql = "DELETE FROM tasks WHERE id = ?1";
        let rows_affected = self.connection.execute(sql, &[&id.to_string()])?;

        if rows_affected == 0 {
            return Err(AppError::System(format!("任务未找到: {}", id)));
        }

        log::debug!("Deleting task: {}", id);
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

        let conn = self.connection.get_raw_connection();
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let tasks = stmt.query_map([category_id.to_string()], |row| {
            let tags_json: String = row.get("tags")?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

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
                tags: tags_json,
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

        log::debug!("Inserting category: {}", category.name);
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

        let conn = self.connection.get_raw_connection();
        let conn = conn.lock().unwrap();
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

        log::debug!("Updating category: {}", category.name);
        Ok(())
    }

    /// 删除分类
    pub fn delete_category(&self, id: Uuid) -> Result<()> {
        let sql = "DELETE FROM categories WHERE id = ?1";
        self.connection.execute(sql, &[&id.to_string()])?;

        log::debug!("Deleting category: {}", id);
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

        let conn = self.connection.get_raw_connection();
        let conn = conn.lock().unwrap();
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

        let conn = self.connection.get_raw_connection();
        let conn = conn.lock().unwrap();
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

        let conn = self.connection.get_raw_connection();
        let conn = conn.lock().unwrap();
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

        let conn = self.connection.get_raw_connection();
        let conn = conn.lock().unwrap();
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

    // ==================== 账户操作 ====================

    /// 插入账户
    pub fn insert_account(&self, account: &AccountInsert) -> Result<i64> {
        let sql = r#"
            INSERT INTO accounts (
                id, name, account_type, currency, balance, initial_balance,
                description, is_active, is_default, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#;

        self.connection.execute(
            sql,
            &[
                &account.id.to_string(),
                &account.name,
                &format!("{:?}", account.account_type).to_lowercase(),
                &account.currency,
                &account.balance,
                &account.initial_balance,
                &account.description,
                &account.is_active,
                &account.is_default,
                &account.created_at.to_rfc3339(),
            ],
        )?;

        let row_id = self
            .connection
            .query_row("SELECT last_insert_rowid()", &[], |row| {
                row.get::<_, i64>(0)
            })?;

        log::debug!("Inserting account: {}", account.id);
        Ok(row_id)
    }

    /// 获取所有账户
    pub fn get_all_accounts(&self) -> Result<Vec<crate::storage::Account>> {
        let sql = r#"
            SELECT id, name, account_type, currency, balance, initial_balance,
                   description, is_active, is_default, created_at, updated_at
            FROM accounts
            WHERE is_active = true
            ORDER BY is_default DESC, name ASC
        "#;

        self.connection.read(|conn| {
            let mut stmt = conn.prepare(sql)?;

            let account_iter = stmt.query_map([], |row| {
                Ok(crate::storage::Account {
                    id: Self::uuid_from_str(row.get(0)?)?,
                    name: row.get(1)?,
                    account_type: Self::parse_account_type_sql(&row.get::<_, String>(2)?)?,
                    currency: row.get(3)?,
                    balance: row.get(4)?,
                    initial_balance: row.get(5)?,
                    description: row.get(6)?,
                    is_active: row.get(7)?,
                    is_default: row.get(8)?,
                    created_at: Self::datetime_from_str(row.get(9)?)?,
                    updated_at: row
                        .get::<_, Option<String>>(10)?
                        .map(|s| Self::datetime_from_str(s))
                        .transpose()?,
                })
            })?;

            let mut accounts = Vec::new();
            for account_result in account_iter {
                accounts.push(account_result?);
            }

            log::debug!("Retrieved {} accounts", accounts.len());
            Ok(accounts)
        })
    }

    /// 将文本解析为 Uuid 并转换为 rusqlite 错误类型
    fn uuid_from_str(text: String) -> std::result::Result<Uuid, rusqlite::Error> {
        Uuid::parse_str(&text).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
    }

    /// 将 RFC3339 文本解析为 DateTime<Local>
    fn datetime_from_str(text: String) -> std::result::Result<DateTime<Local>, rusqlite::Error> {
        DateTime::parse_from_rfc3339(&text)
            .map(|dt| dt.with_timezone(&Local))
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
    }

    fn parse_account_type_sql(
        type_str: &str,
    ) -> std::result::Result<crate::storage::AccountType, rusqlite::Error> {
        match type_str {
            "cash" => Ok(crate::storage::AccountType::Cash),
            "bank" => Ok(crate::storage::AccountType::Bank),
            "creditcard" => Ok(crate::storage::AccountType::CreditCard),
            "investment" => Ok(crate::storage::AccountType::Investment),
            "other" => Ok(crate::storage::AccountType::Other),
            _ => Err(rusqlite::Error::InvalidQuery),
        }
    }

    /// 根据ID获取账户
    pub fn get_account_by_id(&self, id: Uuid) -> Result<Option<crate::storage::Account>> {
        let sql = r#"
            SELECT id, name, account_type, currency, balance, initial_balance,
                   description, is_active, is_default, created_at, updated_at
            FROM accounts WHERE id = ?1
        "#;

        let conn = self.connection.get_raw_connection();
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query([id.to_string()])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            let account_type_str: String = row.get(2)?;
            let created_str: String = row.get(9)?;

            let updated_opt: Option<String> = row.get(10)?;

            let account = crate::storage::Account {
                id: Self::uuid_from_str(id_str)?,
                name: row.get(1)?,
                account_type: Self::parse_account_type_sql(&account_type_str)?,
                currency: row.get(3)?,
                balance: row.get(4)?,
                initial_balance: row.get(5)?,
                description: row.get(6)?,
                is_active: row.get(7)?,
                is_default: row.get(8)?,
                created_at: Self::datetime_from_str(created_str)?,
                updated_at: match updated_opt {
                    Some(s) => Some(Self::datetime_from_str(s)?),
                    None => None,
                },
            };
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    /// 更新账户
    pub fn update_account(&self, id: Uuid, account: &crate::storage::AccountUpdate) -> Result<()> {
        let mut sql_parts = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(name) = &account.name {
            sql_parts.push("name = ?");
            params.push(Box::new(name.clone()));
        }

        if let Some(account_type) = &account.account_type {
            sql_parts.push("account_type = ?");
            params.push(Box::new(format!("{:?}", account_type).to_lowercase()));
        }

        if let Some(currency) = &account.currency {
            sql_parts.push("currency = ?");
            params.push(Box::new(currency.clone()));
        }

        if let Some(description) = &account.description {
            sql_parts.push("description = ?");
            params.push(Box::new(description.clone()));
        }

        if let Some(is_active) = &account.is_active {
            sql_parts.push("is_active = ?");
            params.push(Box::new(*is_active));
        }

        if let Some(is_default) = &account.is_default {
            sql_parts.push("is_default = ?");
            params.push(Box::new(*is_default));
        }

        if sql_parts.is_empty() {
            return Ok(());
        }

        sql_parts.push("updated_at = ?");
        params.push(Box::new(Local::now().to_rfc3339()));

        let sql = format!("UPDATE accounts SET {} WHERE id = ?", sql_parts.join(", "));
        params.push(Box::new(id.to_string()));

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        self.connection.execute(&sql, &param_refs)?;

        log::debug!("Updating account: {}", id);
        Ok(())
    }

    /// 删除账户（软删除）
    pub fn delete_account(&self, id: Uuid) -> Result<()> {
        let sql = "UPDATE accounts SET is_active = false, updated_at = ?1 WHERE id = ?2";
        self.connection
            .execute(sql, &[&Local::now().to_rfc3339(), &id.to_string()])?;
        log::debug!("Deleting account: {}", id);
        Ok(())
    }

    /// 更新账户余额
    pub fn update_account_balance(&self, id: Uuid, new_balance: f64) -> Result<()> {
        let sql = "UPDATE accounts SET balance = ?1, updated_at = ?2 WHERE id = ?3";
        self.connection.execute(
            sql,
            &[&new_balance, &Local::now().to_rfc3339(), &id.to_string()],
        )?;
        log::debug!("Updating account balance: {} = {}", id, new_balance);
        Ok(())
    }

    // ==================== 交易操作 ====================

    /// 插入交易
    pub fn insert_transaction(
        &self,
        transaction: &crate::storage::TransactionInsert,
    ) -> Result<i64> {
        let sql = r#"
            INSERT INTO transactions (
                id, transaction_type, amount, currency, description, account_id,
                category_id, to_account_id, status, transaction_date, tags,
                receipt_path, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#;

        let tags_json = serde_json::to_string(&transaction.tags)?;

        self.connection.execute(
            sql,
            &[
                &transaction.id.to_string(),
                &format!("{:?}", transaction.transaction_type).to_lowercase(),
                &transaction.amount,
                &transaction.currency,
                &transaction.description,
                &transaction.account_id.to_string(),
                &transaction.category_id.map(|id| id.to_string()),
                &transaction.to_account_id.map(|id| id.to_string()),
                &format!("{:?}", transaction.status).to_lowercase(),
                &transaction.transaction_date.format("%Y-%m-%d").to_string(),
                &tags_json,
                &transaction.receipt_path,
                &transaction.created_at.to_rfc3339(),
            ],
        )?;

        let row_id = self
            .connection
            .query_row("SELECT last_insert_rowid()", &[], |row| {
                row.get::<_, i64>(0)
            })?;

        log::debug!("Inserting transaction: {}", transaction.id);
        Ok(row_id)
    }

    /// 获取所有交易
    pub fn get_all_transactions(&self) -> Result<Vec<crate::storage::Transaction>> {
        let sql = r#"
            SELECT t.id, t.transaction_type, t.amount, t.currency, t.description,
                   t.account_id, t.category_id, t.to_account_id, t.status,
                   t.transaction_date, t.tags, t.receipt_path, t.created_at, t.updated_at
            FROM transactions t
            ORDER BY t.transaction_date DESC, t.created_at DESC
        "#;

        let conn = self.connection.get_raw_connection();
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let transaction_iter = stmt.query_map([], |row| {
            let tags_json: String = row.get(10)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    10,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            Ok(crate::storage::Transaction {
                id: Self::uuid_from_str(row.get::<_, String>(0)?)?,
                transaction_type: Self::parse_transaction_type_sql(&row.get::<_, String>(1)?)?,
                amount: row.get(2)?,
                currency: row.get(3)?,
                description: row.get(4)?,
                account_id: Self::uuid_from_str(row.get::<_, String>(5)?)?,
                category_id: row
                    .get::<_, Option<String>>(6)?
                    .map(|s| Self::uuid_from_str(s))
                    .transpose()?,
                to_account_id: row
                    .get::<_, Option<String>>(7)?
                    .map(|s| Self::uuid_from_str(s))
                    .transpose()?,
                status: Self::parse_transaction_status_sql(&row.get::<_, String>(8)?)?,
                transaction_date: Self::naive_date_from_str(row.get::<_, String>(9)?)?,
                tags,
                receipt_path: row.get(11)?,
                created_at: Self::datetime_from_str(row.get::<_, String>(12)?)?,
                updated_at: row
                    .get::<_, Option<String>>(13)?
                    .map(|s| Self::datetime_from_str(s))
                    .transpose()?
                    .map(|dt| dt.with_timezone(&Local)),
            })
        })?;

        let mut transactions = Vec::new();
        for transaction in transaction_iter {
            transactions.push(transaction?);
        }

        Ok(transactions)
    }

    /// 根据日期范围获取交易
    pub fn get_transactions_by_date_range(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<crate::storage::Transaction>> {
        let sql = r#"
            SELECT t.id, t.transaction_type, t.amount, t.currency, t.description,
                   t.account_id, t.category_id, t.to_account_id, t.status,
                   t.transaction_date, t.tags, t.receipt_path, t.created_at, t.updated_at
            FROM transactions t
            WHERE t.transaction_date >= ?1 AND t.transaction_date <= ?2
            ORDER BY t.transaction_date DESC, t.created_at DESC
        "#;

        let conn = self.connection.get_raw_connection();
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let transaction_iter = stmt.query_map(
            params![
                start_date.format("%Y-%m-%d").to_string(),
                end_date.format("%Y-%m-%d").to_string()
            ],
            |row| {
                let tags_json: String = row.get(10)?;
                let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        10,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

                Ok(crate::storage::Transaction {
                    id: Self::uuid_from_str(row.get::<_, String>(0)?)?,
                    transaction_type: Self::parse_transaction_type_sql(&row.get::<_, String>(1)?)?,
                    amount: row.get(2)?,
                    currency: row.get(3)?,
                    description: row.get(4)?,
                    account_id: Self::uuid_from_str(row.get::<_, String>(5)?)?,
                    category_id: row
                        .get::<_, Option<String>>(6)?
                        .map(|s| Self::uuid_from_str(s))
                        .transpose()?,
                    to_account_id: row
                        .get::<_, Option<String>>(7)?
                        .map(|s| Self::uuid_from_str(s))
                        .transpose()?,
                    status: Self::parse_transaction_status_sql(&row.get::<_, String>(8)?)?,
                    transaction_date: Self::naive_date_from_str(row.get::<_, String>(9)?)?,
                    tags,
                    receipt_path: row.get(11)?,
                    created_at: Self::datetime_from_str(row.get::<_, String>(12)?)?,
                    updated_at: row
                        .get::<_, Option<String>>(13)?
                        .map(|s| Self::datetime_from_str(s))
                        .transpose()?
                        .map(|dt| dt.with_timezone(&Local)),
                })
            },
        )?;

        let mut transactions = Vec::new();
        for transaction in transaction_iter {
            transactions.push(transaction?);
        }

        Ok(transactions)
    }

    /// 根据账户ID获取交易
    pub fn get_transactions_by_account(
        &self,
        account_id: Uuid,
    ) -> Result<Vec<crate::storage::Transaction>> {
        let sql = r#"
            SELECT t.id, t.transaction_type, t.amount, t.currency, t.description,
                   t.account_id, t.category_id, t.to_account_id, t.status,
                   t.transaction_date, t.tags, t.receipt_path, t.created_at, t.updated_at
            FROM transactions t
            WHERE t.account_id = ?1 OR t.to_account_id = ?1
            ORDER BY t.transaction_date DESC, t.created_at DESC
        "#;

        let conn = self.connection.get_raw_connection();
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let transaction_iter = stmt.query_map(params![account_id.to_string()], |row| {
            let tags_json: String = row.get(10)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    10,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            Ok(crate::storage::Transaction {
                id: Self::uuid_from_str(row.get::<_, String>(0)?)?,
                transaction_type: Self::parse_transaction_type_sql(&row.get::<_, String>(1)?)?,
                amount: row.get(2)?,
                currency: row.get(3)?,
                description: row.get(4)?,
                account_id: Self::uuid_from_str(row.get::<_, String>(5)?)?,
                category_id: row
                    .get::<_, Option<String>>(6)?
                    .map(|s| Self::uuid_from_str(s))
                    .transpose()?,
                to_account_id: row
                    .get::<_, Option<String>>(7)?
                    .map(|s| Self::uuid_from_str(s))
                    .transpose()?,
                status: Self::parse_transaction_status_sql(&row.get::<_, String>(8)?)?,
                transaction_date: Self::naive_date_from_str(row.get::<_, String>(9)?)?,
                tags,
                receipt_path: row.get(11)?,
                created_at: Self::datetime_from_str(row.get::<_, String>(12)?)?,
                updated_at: row
                    .get::<_, Option<String>>(13)?
                    .map(|s| Self::datetime_from_str(s))
                    .transpose()?
                    .map(|dt| dt.with_timezone(&Local)),
            })
        })?;

        let mut transactions = Vec::new();
        for transaction in transaction_iter {
            transactions.push(transaction?);
        }

        Ok(transactions)
    }

    /// 根据ID获取交易
    pub fn get_transaction_by_id(&self, id: Uuid) -> Result<Option<crate::storage::Transaction>> {
        let sql = r#"
            SELECT t.id, t.transaction_type, t.amount, t.currency, t.description,
                   t.account_id, t.category_id, t.to_account_id, t.status,
                   t.transaction_date, t.tags, t.receipt_path, t.created_at, t.updated_at
            FROM transactions t
            WHERE t.id = ?1
        "#;

        let conn = self.connection.get_raw_connection();
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;

        let mut transaction_iter = stmt.query_map(params![id.to_string()], |row| {
            let tags_json: String = row.get(10)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    10,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            Ok(crate::storage::Transaction {
                id: Self::uuid_from_str(row.get::<_, String>(0)?)?,
                transaction_type: Self::parse_transaction_type_sql(&row.get::<_, String>(1)?)?,
                amount: row.get(2)?,
                currency: row.get(3)?,
                description: row.get(4)?,
                account_id: Self::uuid_from_str(row.get::<_, String>(5)?)?,
                category_id: row
                    .get::<_, Option<String>>(6)?
                    .map(|s| Self::uuid_from_str(s))
                    .transpose()?,
                to_account_id: row
                    .get::<_, Option<String>>(7)?
                    .map(|s| Self::uuid_from_str(s))
                    .transpose()?,
                status: Self::parse_transaction_status_sql(&row.get::<_, String>(8)?)?,
                transaction_date: Self::naive_date_from_str(row.get::<_, String>(9)?)?,
                tags,
                receipt_path: row.get(11)?,
                created_at: Self::datetime_from_str(row.get::<_, String>(12)?)?,
                updated_at: row
                    .get::<_, Option<String>>(13)?
                    .map(|s| Self::datetime_from_str(s))
                    .transpose()?
                    .map(|dt| dt.with_timezone(&Local)),
            })
        })?;

        // 获取第一个（也应该是唯一的）结果
        match transaction_iter.next() {
            Some(transaction) => Ok(Some(transaction?)),
            None => Ok(None),
        }
    }

    /// 更新交易
    pub fn update_transaction(
        &self,
        id: Uuid,
        transaction: &crate::storage::TransactionUpdate,
    ) -> Result<()> {
        let mut sql_parts = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(transaction_type) = &transaction.transaction_type {
            sql_parts.push("transaction_type = ?");
            params.push(Box::new(format!("{:?}", transaction_type).to_lowercase()));
        }

        if let Some(amount) = &transaction.amount {
            sql_parts.push("amount = ?");
            params.push(Box::new(*amount));
        }

        if let Some(currency) = &transaction.currency {
            sql_parts.push("currency = ?");
            params.push(Box::new(currency.clone()));
        }

        if let Some(description) = &transaction.description {
            sql_parts.push("description = ?");
            params.push(Box::new(description.clone()));
        }

        if let Some(account_id) = &transaction.account_id {
            sql_parts.push("account_id = ?");
            params.push(Box::new(account_id.to_string()));
        }

        if let Some(category_id) = &transaction.category_id {
            sql_parts.push("category_id = ?");
            params.push(Box::new(category_id.map(|id| id.to_string())));
        }

        if let Some(to_account_id) = &transaction.to_account_id {
            sql_parts.push("to_account_id = ?");
            params.push(Box::new(to_account_id.map(|id| id.to_string())));
        }

        if let Some(status) = &transaction.status {
            sql_parts.push("status = ?");
            params.push(Box::new(format!("{:?}", status).to_lowercase()));
        }

        if let Some(transaction_date) = &transaction.transaction_date {
            sql_parts.push("transaction_date = ?");
            params.push(Box::new(transaction_date.format("%Y-%m-%d").to_string()));
        }

        if let Some(tags) = &transaction.tags {
            sql_parts.push("tags = ?");
            let tags_json = serde_json::to_string(tags)?;
            params.push(Box::new(tags_json));
        }

        if let Some(receipt_path) = &transaction.receipt_path {
            sql_parts.push("receipt_path = ?");
            params.push(Box::new(receipt_path.clone()));
        }

        if sql_parts.is_empty() {
            return Ok(());
        }

        sql_parts.push("updated_at = ?");
        params.push(Box::new(Local::now().to_rfc3339()));

        let sql = format!(
            "UPDATE transactions SET {} WHERE id = ?",
            sql_parts.join(", ")
        );
        params.push(Box::new(id.to_string()));

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        self.connection.execute(&sql, &param_refs)?;

        log::debug!("Updating transaction: {}", id);
        Ok(())
    }

    /// 删除交易
    pub fn delete_transaction(&self, id: Uuid) -> Result<()> {
        let sql = "DELETE FROM transactions WHERE id = ?1";
        self.connection.execute(sql, &[&id.to_string()])?;
        log::debug!("Deleting transaction: {}", id);
        Ok(())
    }

    // ==================== 辅助方法 ====================

    /// 解析账户类型
    fn parse_account_type(&self, type_str: &str) -> Result<crate::storage::AccountType> {
        match type_str {
            "cash" => Ok(crate::storage::AccountType::Cash),
            "bank" => Ok(crate::storage::AccountType::Bank),
            "creditcard" => Ok(crate::storage::AccountType::CreditCard),
            "investment" => Ok(crate::storage::AccountType::Investment),
            "other" => Ok(crate::storage::AccountType::Other),
            _ => Err(AppError::Storage(format!("未知的账户类型: {}", type_str))),
        }
    }

    /// 解析交易类型
    fn parse_transaction_type(&self, type_str: &str) -> Result<crate::storage::TransactionType> {
        match type_str {
            "income" => Ok(crate::storage::TransactionType::Income),
            "expense" => Ok(crate::storage::TransactionType::Expense),
            "transfer" => Ok(crate::storage::TransactionType::Transfer),
            _ => Err(AppError::Storage(format!("未知的交易类型: {}", type_str))),
        }
    }

    /// 解析交易状态
    fn parse_transaction_status(
        &self,
        status_str: &str,
    ) -> Result<crate::storage::TransactionStatus> {
        match status_str {
            "pending" => Ok(crate::storage::TransactionStatus::Pending),
            "completed" => Ok(crate::storage::TransactionStatus::Completed),
            "cancelled" => Ok(crate::storage::TransactionStatus::Cancelled),
            _ => Err(AppError::Storage(format!("未知的交易状态: {}", status_str))),
        }
    }

    /// 解析预算周期
    fn parse_budget_period(&self, period_str: &str) -> Result<crate::storage::BudgetPeriod> {
        match period_str {
            "daily" => Ok(crate::storage::BudgetPeriod::Daily),
            "weekly" => Ok(crate::storage::BudgetPeriod::Weekly),
            "monthly" => Ok(crate::storage::BudgetPeriod::Monthly),
            "yearly" => Ok(crate::storage::BudgetPeriod::Yearly),
            _ => Err(AppError::Storage(format!("未知的预算周期: {}", period_str))),
        }
    }

    // ==================== 设置存储 ====================

    /// 获取设置值
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let sql = "SELECT value FROM settings WHERE key = ?1";

        match self
            .connection
            .query_row(sql, &[&key], |row| Ok(row.get::<_, String>(0)?))
        {
            Ok(value) => Ok(Some(value)),
            Err(AppError::Database(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 设置配置值
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let sql = r#"
            INSERT OR REPLACE INTO settings (key, value, updated_at)
            VALUES (?1, ?2, CURRENT_TIMESTAMP)
        "#;

        self.connection.execute(sql, &[&key, &value])?;
        Ok(())
    }

    /// 删除设置
    pub fn delete_setting(&self, key: &str) -> Result<()> {
        let sql = "DELETE FROM settings WHERE key = ?1";
        self.connection.execute(sql, &[&key])?;
        Ok(())
    }

    /// 获取所有设置
    pub fn get_all_settings(&self) -> Result<HashMap<String, String>> {
        let sql = "SELECT key, value FROM settings ORDER BY key";

        self.connection.read(|conn| {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;

            let mut settings = HashMap::new();
            for row in rows {
                let (key, value) = row?;
                settings.insert(key, value);
            }
            Ok(settings)
        })
    }

    // ==================== 统计查询 ====================

    /// 获取账户余额统计
    pub fn get_account_balance_summary(&self) -> Result<HashMap<String, f64>> {
        let sql = r#"
            SELECT currency, SUM(balance) as total_balance
            FROM accounts
            WHERE is_active = true
            GROUP BY currency
        "#;

        self.connection.read(|conn| {
            let mut stmt = conn.prepare(sql)?;

            let balance_iter = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })?;

            let mut balances = HashMap::new();
            for balance in balance_iter {
                let (currency, total) = balance?;
                balances.insert(currency, total);
            }

            Ok(balances)
        })
    }

    /// 获取交易统计
    pub fn get_transaction_statistics(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<crate::storage::FinancialStats> {
        let sql = r#"
            SELECT 
                transaction_type,
                SUM(amount) as total_amount,
                COUNT(*) as transaction_count
            FROM transactions
            WHERE transaction_date >= ?1 AND transaction_date <= ?2
            GROUP BY transaction_type
        "#;

        self.connection.read(|conn| {
            let mut stmt = conn.prepare(sql)?;

            let stats_iter = stmt.query_map(
                params![
                    start_date.format("%Y-%m-%d").to_string(),
                    end_date.format("%Y-%m-%d").to_string()
                ],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )?;

            let mut total_income = 0.0;
            let mut total_expense = 0.0;
            let mut transaction_count = 0;

            for stat in stats_iter {
                let (transaction_type, amount, count) = stat?;
                transaction_count += count;

                match transaction_type.as_str() {
                    "income" => total_income += amount,
                    "expense" => total_expense += amount,
                    "transfer" => {} // 转账不计入收支统计
                    _ => {}
                }
            }

            let net_income = total_income - total_expense;
            let account_balance = self.get_account_balance_summary()?.values().sum();

            Ok(crate::storage::FinancialStats {
                total_income,
                total_expense,
                net_income,
                account_balance,
                transaction_count,
                period_start: start_date,
                period_end: end_date,
                currency: "CNY".to_string(), // 默认货币，后续可配置
            })
        })
    }

    /// 将 YYYY-MM-DD 文本解析为 NaiveDate
    fn naive_date_from_str(text: String) -> std::result::Result<NaiveDate, rusqlite::Error> {
        NaiveDate::parse_from_str(&text, "%Y-%m-%d").map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
    }

    /// 将交易类型字符串解析为枚举（用于 SQL 结果集）
    fn parse_transaction_type_sql(
        type_str: &str,
    ) -> std::result::Result<crate::storage::TransactionType, rusqlite::Error> {
        match type_str {
            "income" => Ok(crate::storage::TransactionType::Income),
            "expense" => Ok(crate::storage::TransactionType::Expense),
            "transfer" => Ok(crate::storage::TransactionType::Transfer),
            _ => Err(rusqlite::Error::InvalidQuery),
        }
    }

    /// 将交易状态字符串解析为枚举（用于 SQL 结果集）
    fn parse_transaction_status_sql(
        status_str: &str,
    ) -> std::result::Result<crate::storage::TransactionStatus, rusqlite::Error> {
        match status_str {
            "pending" => Ok(crate::storage::TransactionStatus::Pending),
            "completed" => Ok(crate::storage::TransactionStatus::Completed),
            "cancelled" => Ok(crate::storage::TransactionStatus::Cancelled),
            _ => Err(rusqlite::Error::InvalidQuery),
        }
    }

    // ==================== 笔记操作 ====================

    /// 插入笔记
    pub fn insert_note(&self, note: &Note) -> Result<i64> {
        let sql = r#"
            INSERT INTO notes (
                id, title, content, mood, tags, is_favorite, is_archived, 
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#;

        let tags_json = serde_json::to_string(&note.tags)?;

        self.connection.execute(
            sql,
            &[
                &note.id.to_string(),
                &note.title,
                &note.content,
                &note.mood,
                &tags_json,
                &note.is_favorite,
                &note.is_archived,
                &note.created_at.to_rfc3339(),
                &note.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(1) // 返回插入的行数
    }

    /// 获取所有笔记
    pub fn get_all_notes(&self) -> Result<Vec<Note>> {
        let sql = r#"
            SELECT id, title, content, mood, tags, is_favorite, is_archived, 
                   created_at, updated_at
            FROM notes 
            ORDER BY updated_at DESC
        "#;

        self.connection.read(|conn| {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([], |row| {
                let id_str: String = row.get(0)?;
                let tags_json: String = row.get(4)?;
                let created_at_str: String = row.get(7)?;
                let updated_at_str: String = row.get(8)?;

                let id = Uuid::parse_str(&id_str).map_err(|e| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        format!("Invalid UUID: {}", e).into(),
                        rusqlite::types::Type::Text,
                    )
                })?;
                let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
                    rusqlite::Error::InvalidColumnType(
                        4,
                        format!("Invalid JSON: {}", e).into(),
                        rusqlite::types::Type::Text,
                    )
                })?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| {
                        rusqlite::Error::InvalidColumnType(
                            7,
                            format!("Invalid datetime: {}", e).into(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Local);
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|e| {
                        rusqlite::Error::InvalidColumnType(
                            8,
                            format!("Invalid datetime: {}", e).into(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Local);

                Ok(Note {
                    id,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    mood: row.get(3)?,
                    tags,
                    is_favorite: row.get(5)?,
                    is_archived: row.get(6)?,
                    created_at,
                    updated_at,
                })
            })?;

            let mut notes = Vec::new();
            for row in rows {
                notes.push(row?);
            }
            Ok(notes)
        })
    }

    /// 根据ID获取笔记
    pub fn get_note_by_id(&self, id: Uuid) -> Result<Option<Note>> {
        let sql = r#"
            SELECT id, title, content, mood, tags, is_favorite, is_archived, 
                   created_at, updated_at
            FROM notes 
            WHERE id = ?1
        "#;

        match self.connection.query_row(sql, &[&id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let tags_json: String = row.get(4)?;
            let created_at_str: String = row.get(7)?;
            let updated_at_str: String = row.get(8)?;

            let id = Uuid::parse_str(&id_str).map_err(|e| {
                rusqlite::Error::InvalidColumnType(
                    0,
                    format!("Invalid UUID: {}", e).into(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
                rusqlite::Error::InvalidColumnType(
                    4,
                    format!("Invalid JSON: {}", e).into(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| {
                    rusqlite::Error::InvalidColumnType(
                        7,
                        format!("Invalid datetime: {}", e).into(),
                        rusqlite::types::Type::Text,
                    )
                })?
                .with_timezone(&Local);
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| {
                    rusqlite::Error::InvalidColumnType(
                        8,
                        format!("Invalid datetime: {}", e).into(),
                        rusqlite::types::Type::Text,
                    )
                })?
                .with_timezone(&Local);

            Ok(Note {
                id,
                title: row.get(1)?,
                content: row.get(2)?,
                mood: row.get(3)?,
                tags,
                is_favorite: row.get(5)?,
                is_archived: row.get(6)?,
                created_at,
                updated_at,
            })
        }) {
            Ok(note) => Ok(Some(note)),
            Err(AppError::Database(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 更新笔记
    pub fn update_note(&self, id: Uuid, update: &NoteUpdate) -> Result<()> {
        let mut sql_parts = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(title) = &update.title {
            sql_parts.push("title = ?");
            params.push(Box::new(title.clone()));
        }

        if let Some(content) = &update.content {
            sql_parts.push("content = ?");
            params.push(Box::new(content.clone()));
        }

        if let Some(mood) = &update.mood {
            sql_parts.push("mood = ?");
            params.push(Box::new(mood.clone()));
        }

        if let Some(tags) = &update.tags {
            sql_parts.push("tags = ?");
            let tags_json = serde_json::to_string(tags)?;
            params.push(Box::new(tags_json));
        }

        if let Some(is_favorite) = &update.is_favorite {
            sql_parts.push("is_favorite = ?");
            params.push(Box::new(*is_favorite));
        }

        if let Some(is_archived) = &update.is_archived {
            sql_parts.push("is_archived = ?");
            params.push(Box::new(*is_archived));
        }

        // 始终更新 updated_at
        sql_parts.push("updated_at = ?");
        params.push(Box::new(update.updated_at.to_rfc3339()));

        if sql_parts.is_empty() {
            return Ok(()); // 没有需要更新的字段
        }

        let sql = format!("UPDATE notes SET {} WHERE id = ?", sql_parts.join(", "));
        params.push(Box::new(id.to_string()));

        // 转换参数为引用
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        self.connection.execute(&sql, &param_refs)?;
        Ok(())
    }

    /// 删除笔记
    pub fn delete_note(&self, id: Uuid) -> Result<()> {
        let sql = "DELETE FROM notes WHERE id = ?1";
        self.connection.execute(sql, &[&id.to_string()])?;
        Ok(())
    }

    /// 根据查询条件搜索笔记
    pub fn search_notes(&self, query: &NoteQuery) -> Result<Vec<Note>> {
        let mut where_conditions = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        // 构建搜索条件
        if let Some(search) = &query.search {
            // 使用 FTS 进行全文搜索
            where_conditions
                .push("notes.id IN (SELECT notes.id FROM notes_fts WHERE notes_fts MATCH ?)");
            params.push(Box::new(search.clone()));
        }

        if let Some(tags) = &query.tags {
            for tag in tags {
                where_conditions.push("JSON_EXTRACT(tags, '$') LIKE ?");
                params.push(Box::new(format!("%\"{}\"%%", tag)));
            }
        }

        if let Some(mood) = &query.mood {
            where_conditions.push("mood = ?");
            params.push(Box::new(mood.clone()));
        }

        if let Some(is_favorite) = query.is_favorite {
            where_conditions.push("is_favorite = ?");
            params.push(Box::new(is_favorite));
        }

        if let Some(is_archived) = query.is_archived {
            where_conditions.push("is_archived = ?");
            params.push(Box::new(is_archived));
        }

        if let Some(created_from) = &query.created_from {
            where_conditions.push("created_at >= ?");
            params.push(Box::new(created_from.to_rfc3339()));
        }

        if let Some(created_to) = &query.created_to {
            where_conditions.push("created_at <= ?");
            params.push(Box::new(created_to.to_rfc3339()));
        }

        // 构建 ORDER BY 子句
        let order_by = match query.sort_by.as_ref().unwrap_or(&NoteSortBy::UpdatedAt) {
            NoteSortBy::CreatedAt => "created_at",
            NoteSortBy::UpdatedAt => "updated_at",
            NoteSortBy::Title => "title",
            NoteSortBy::Mood => "mood",
        };

        let sort_order = match query.sort_order.as_ref().unwrap_or(&SortOrder::Desc) {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        };

        // 构建完整的 SQL 查询
        let mut sql = format!(
            r#"
            SELECT id, title, content, mood, tags, is_favorite, is_archived, 
                   created_at, updated_at
            FROM notes 
            {}
            ORDER BY {} {}
            "#,
            if where_conditions.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", where_conditions.join(" AND "))
            },
            order_by,
            sort_order
        );

        // 添加分页
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
            if let Some(offset) = query.offset {
                sql.push_str(&format!(" OFFSET {}", offset));
            }
        }

        self.connection.read(|conn| {
            let mut stmt = conn.prepare(&sql)?;
            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            let rows = stmt.query_map(&param_refs[..], |row| {
                let id_str: String = row.get(0)?;
                let tags_json: String = row.get(4)?;
                let created_at_str: String = row.get(7)?;
                let updated_at_str: String = row.get(8)?;

                let id = Uuid::parse_str(&id_str).map_err(|e| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        format!("Invalid UUID: {}", e).into(),
                        rusqlite::types::Type::Text,
                    )
                })?;
                let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
                    rusqlite::Error::InvalidColumnType(
                        4,
                        format!("Invalid JSON: {}", e).into(),
                        rusqlite::types::Type::Text,
                    )
                })?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| {
                        rusqlite::Error::InvalidColumnType(
                            7,
                            format!("Invalid datetime: {}", e).into(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Local);
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|e| {
                        rusqlite::Error::InvalidColumnType(
                            8,
                            format!("Invalid datetime: {}", e).into(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Local);

                Ok(Note {
                    id,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    mood: row.get(3)?,
                    tags,
                    is_favorite: row.get(5)?,
                    is_archived: row.get(6)?,
                    created_at,
                    updated_at,
                })
            })?;

            let mut notes = Vec::new();
            for row in rows {
                notes.push(row?);
            }
            Ok(notes)
        })
    }

    /// 获取所有笔记标签
    pub fn get_all_note_tags(&self) -> Result<Vec<String>> {
        let sql = "SELECT DISTINCT tags FROM notes WHERE tags != '[]'";

        self.connection.read(|conn| {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([], |row| {
                let tags_json: String = row.get(0)?;
                Ok(tags_json)
            })?;

            let mut all_tags = std::collections::HashSet::new();
            for row in rows {
                let tags_json = row?;
                if let Ok(tags) = serde_json::from_str::<Vec<String>>(&tags_json) {
                    for tag in tags {
                        all_tags.insert(tag);
                    }
                }
            }

            let mut tags_vec: Vec<String> = all_tags.into_iter().collect();
            tags_vec.sort();
            Ok(tags_vec)
        })
    }

    /// 获取笔记统计信息
    pub fn get_notes_stats(&self) -> Result<NoteStats> {
        let total_notes = self.connection.query_row(
            "SELECT COUNT(*) FROM notes",
            &[] as &[&dyn rusqlite::ToSql],
            |row| Ok(row.get::<_, i64>(0)?),
        )?;

        let favorite_notes = self.connection.query_row(
            "SELECT COUNT(*) FROM notes WHERE is_favorite = 1",
            &[] as &[&dyn rusqlite::ToSql],
            |row| Ok(row.get::<_, i64>(0)?),
        )?;

        let archived_notes = self.connection.query_row(
            "SELECT COUNT(*) FROM notes WHERE is_archived = 1",
            &[] as &[&dyn rusqlite::ToSql],
            |row| Ok(row.get::<_, i64>(0)?),
        )?;

        // 计算本周笔记数
        let week_start = Local::now().date_naive() - chrono::Duration::days(7);
        let week_start_str = week_start.to_string();
        let notes_this_week = self.connection.query_row(
            "SELECT COUNT(*) FROM notes WHERE created_at >= ?",
            &[&week_start_str as &dyn rusqlite::ToSql],
            |row| Ok(row.get::<_, i64>(0)?),
        )?;

        // 计算本月笔记数
        let month_start = Local::now().date_naive() - chrono::Duration::days(30);
        let month_start_str = month_start.to_string();
        let notes_this_month = self.connection.query_row(
            "SELECT COUNT(*) FROM notes WHERE created_at >= ?",
            &[&month_start_str as &dyn rusqlite::ToSql],
            |row| Ok(row.get::<_, i64>(0)?),
        )?;

        // 获取标签统计
        let most_used_tags = self.get_tag_stats()?;

        // 获取心情分布
        let mood_distribution = self.get_mood_stats()?;

        // 获取每日笔记趋势（最近30天）
        let daily_notes_trend = self.get_daily_notes_trend(30)?;

        Ok(NoteStats {
            total_notes,
            favorite_notes,
            archived_notes,
            notes_this_week,
            notes_this_month,
            most_used_tags,
            mood_distribution,
            daily_notes_trend,
        })
    }

    /// 获取标签统计
    fn get_tag_stats(&self) -> Result<Vec<super::models::TagStats>> {
        let all_tags = self.get_all_note_tags()?;
        let total_notes = self.connection.query_row(
            "SELECT COUNT(*) FROM notes WHERE tags != '[]'",
            &[] as &[&dyn rusqlite::ToSql],
            |row| Ok(row.get::<_, i64>(0)?),
        )?;

        let mut tag_stats = Vec::new();
        for tag in all_tags {
            let search_pattern = format!("%\"{}\"%%", tag);
            let count = self.connection.query_row(
                "SELECT COUNT(*) FROM notes WHERE JSON_EXTRACT(tags, '$') LIKE ?",
                &[&search_pattern as &dyn rusqlite::ToSql],
                |row| Ok(row.get::<_, i64>(0)?),
            )?;

            let percentage = if total_notes > 0 {
                (count as f64 / total_notes as f64) * 100.0
            } else {
                0.0
            };

            tag_stats.push(super::models::TagStats {
                tag,
                count,
                percentage,
            });
        }

        // 按使用次数排序
        tag_stats.sort_by(|a, b| b.count.cmp(&a.count));
        tag_stats.truncate(10); // 只返回前10个最常用的标签

        Ok(tag_stats)
    }

    /// 获取心情分布统计
    fn get_mood_stats(&self) -> Result<Vec<super::models::MoodStats>> {
        let sql = r#"
            SELECT mood, COUNT(*) as count
            FROM notes 
            WHERE mood IS NOT NULL 
            GROUP BY mood 
            ORDER BY count DESC
        "#;

        let total_notes_with_mood = self.connection.query_row(
            "SELECT COUNT(*) FROM notes WHERE mood IS NOT NULL",
            &[] as &[&dyn rusqlite::ToSql],
            |row| Ok(row.get::<_, i64>(0)?),
        )?;

        self.connection.read(|conn| {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([], |row| {
                let mood: String = row.get(0)?;
                let count: i64 = row.get(1)?;
                Ok((mood, count))
            })?;

            let mut mood_stats = Vec::new();
            for row in rows {
                let (mood, count) = row?;
                let percentage = if total_notes_with_mood > 0 {
                    (count as f64 / total_notes_with_mood as f64) * 100.0
                } else {
                    0.0
                };

                mood_stats.push(super::models::MoodStats {
                    mood,
                    count,
                    percentage,
                });
            }
            Ok(mood_stats)
        })
    }

    /// 获取每日笔记趋势
    fn get_daily_notes_trend(&self, days: i32) -> Result<Vec<super::models::DailyNoteStats>> {
        let start_date = Local::now().date_naive() - chrono::Duration::days(days as i64);

        let sql = r#"
            SELECT DATE(created_at) as date, COUNT(*) as count
            FROM notes 
            WHERE created_at >= ?
            GROUP BY DATE(created_at)
            ORDER BY date
        "#;

        self.connection.read(|conn| {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([&start_date.to_string()], |row| {
                let date_str: String = row.get(0)?;
                let count: i64 = row.get(1)?;
                let date =
                    chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
                        rusqlite::Error::InvalidColumnType(
                            0,
                            format!("Invalid date: {}", e).into(),
                            rusqlite::types::Type::Text,
                        )
                    })?;
                Ok(super::models::DailyNoteStats { date, count })
            })?;

            let mut trend = Vec::new();
            for row in rows {
                trend.push(row?);
            }
            Ok(trend)
        })
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
