//! # 任务数据库操作模块
//!
//! 提供任务相关的数据库操作功能

use super::connection::DatabaseConnection;
use super::utils::{datetime_from_str, uuid_from_str};
use crate::errors::{AppError, Result};
use crate::storage::task_models::{TaskInsert, TaskModel, TaskUpdate};
use chrono::{DateTime, Local};
use serde_json;
use uuid::Uuid;

/// 任务数据库操作
pub struct TasksRepository<'a> {
    connection: &'a DatabaseConnection,
}

impl<'a> TasksRepository<'a> {
    /// 创建新的任务仓库实例
    pub fn new(connection: &'a DatabaseConnection) -> Self {
        Self { connection }
    }

    /// 插入任务
    pub fn insert(&self, task: &TaskInsert) -> Result<i64> {
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
    pub fn get_all(&self) -> Result<Vec<TaskModel>> {
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
                let _tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

                Ok(TaskModel {
                    id: uuid_from_str(row.get(0)?)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    category_id: row
                        .get::<_, Option<String>>(3)?
                        .map(|s| uuid_from_str(s))
                        .transpose()?,
                    status: row.get(4)?,
                    priority: row.get(5)?,
                    estimated_duration_seconds: row.get(6)?,
                    total_duration_seconds: row.get(7)?,
                    tags: tags_json,
                    due_date: row
                        .get::<_, Option<String>>(9)?
                        .map(|s| datetime_from_str(s))
                        .transpose()?,
                    is_completed: row.get(10)?,
                    completed_at: row
                        .get::<_, Option<String>>(11)?
                        .map(|s| datetime_from_str(s))
                        .transpose()?,
                    created_at: datetime_from_str(row.get(12)?)?,
                    updated_at: row
                        .get::<_, Option<String>>(13)?
                        .map(|s| datetime_from_str(s))
                        .transpose()?,
                })
            })?;

            let mut tasks = Vec::new();
            for task_result in task_iter {
                tasks.push(task_result?);
            }

            log::debug!("获取到 {} 个任务", tasks.len());
            Ok(tasks)
        })
    }

    /// 根据ID获取任务
    pub fn get_by_id(&self, id: Uuid) -> Result<Option<TaskModel>> {
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
    pub fn update(&self, id: Uuid, task: &TaskUpdate) -> Result<()> {
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
    pub fn delete(&self, id: Uuid) -> Result<()> {
        let sql = "DELETE FROM tasks WHERE id = ?1";
        let rows_affected = self.connection.execute(sql, &[&id.to_string()])?;

        if rows_affected == 0 {
            return Err(AppError::System(format!("任务未找到: {}", id)));
        }

        log::debug!("删除任务: {}", id);
        Ok(())
    }

    /// 根据分类获取任务
    pub fn get_by_category(&self, category_id: Uuid) -> Result<Vec<TaskModel>> {
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
            let _tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

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
}
