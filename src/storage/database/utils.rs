//! # 数据库工具函数模块
//!
//! 提供数据库操作中常用的工具函数

use chrono::{DateTime, Local, NaiveDate};
use rusqlite;
use uuid::Uuid;

/// 将文本解析为 Uuid 并转换为 rusqlite 错误类型
pub fn uuid_from_str(text: String) -> std::result::Result<Uuid, rusqlite::Error> {
    Uuid::parse_str(&text).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

/// 将 RFC3339 文本解析为 DateTime<Local>
pub fn datetime_from_str(text: String) -> std::result::Result<DateTime<Local>, rusqlite::Error> {
    DateTime::parse_from_rfc3339(&text)
        .map(|dt| dt.with_timezone(&Local))
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
}

/// 将文本解析为 NaiveDate
pub fn naive_date_from_str(text: String) -> std::result::Result<NaiveDate, rusqlite::Error> {
    NaiveDate::parse_from_str(&text, "%Y-%m-%d").map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}