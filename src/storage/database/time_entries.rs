//! # 时间记录数据库操作模块
//!
//! 提供时间记录相关的数据库操作功能

use super::connection::DatabaseConnection;
use crate::errors::{AppError, Result};
use crate::storage::models::{TimeEntry, TimeEntryInsert};
use chrono::{DateTime, Local, NaiveDate};
use rusqlite::params;
use serde_json;
use uuid::Uuid;

/// 时间记录数据库操作
pub struct TimeEntriesRepository<'a> {
    connection: &'a DatabaseConnection,
}

impl<'a> TimeEntriesRepository<'a> {
    /// 创建新的时间记录仓库实例
    pub fn new(connection: &'a DatabaseConnection) -> Self {
        Self { connection }
    }

    /// 插入时间记录
    pub fn insert(&self, entry: &TimeEntryInsert) -> Result<i64> {
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
    pub fn get_by_id(&self, id: Uuid) -> Result<Option<TimeEntry>> {
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
    pub fn get_by_date_range(
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
    pub fn get_by_category(&self, category_id: Uuid) -> Result<Vec<TimeEntry>> {
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
    pub fn update(&self, id: Uuid, entry: &TimeEntryInsert) -> Result<()> {
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
    pub fn delete(&self, id: Uuid) -> Result<()> {
        let sql = "DELETE FROM time_entries WHERE id = ?1";

        let rows_affected = self.connection.execute(sql, &[&id.to_string()])?;

        if rows_affected == 0 {
            return Err(AppError::TaskNotFound(id.to_string()));
        }

        log::debug!("删除时间记录: {}", id);
        Ok(())
    }
}