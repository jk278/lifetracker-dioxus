//! # 数据导入导出命令模块
//!
//! 负责处理数据的导入和导出功能

use super::*;
use crate::utils::export::{create_export_data, DataExporter, ExportFormat, ExportOptions};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

// ========== 数据导入导出命令 ==========

/// 导出数据
#[tauri::command]
pub async fn export_data(
    state: State<'_, AppState>,
    format: String, // "json", "csv", "xml", "html", "markdown"
    file_path: String,
    options: Option<ExportOptionsRequest>,
) -> Result<String, String> {
    log::info!("导出数据到: {}，格式: {}", file_path, format);

    // 解析导出格式
    let export_format = ExportFormat::from_extension(&format)
        .ok_or_else(|| format!("不支持的导出格式: {}", format))?;

    // 构建导出选项
    let export_options = build_export_options(export_format, options);

    // 执行导出
    match perform_data_export(&state, export_options, &file_path).await {
        Ok(summary) => {
            log::info!("数据导出成功: {}", summary);
            Ok(summary)
        }
        Err(e) => {
            log::error!("数据导出失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 导出选项请求结构
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExportOptionsRequest {
    pub include_categories: Option<bool>,
    pub include_statistics: Option<bool>,
    pub start_date: Option<String>, // ISO 8601 格式
    pub end_date: Option<String>,   // ISO 8601 格式
    pub category_filter: Option<Vec<String>>,
    pub group_by_date: Option<bool>,
    pub group_by_category: Option<bool>,
    pub include_metadata: Option<bool>,
}

/// 构建导出选项
fn build_export_options(
    format: ExportFormat,
    request: Option<ExportOptionsRequest>,
) -> ExportOptions {
    let mut options = ExportOptions {
        format,
        ..Default::default()
    };

    if let Some(req) = request {
        if let Some(include_categories) = req.include_categories {
            options.include_categories = include_categories;
        }
        if let Some(include_statistics) = req.include_statistics {
            options.include_statistics = include_statistics;
        }
        if let Some(group_by_date) = req.group_by_date {
            options.group_by_date = group_by_date;
        }
        if let Some(group_by_category) = req.group_by_category {
            options.group_by_category = group_by_category;
        }
        if let Some(include_metadata) = req.include_metadata {
            options.include_metadata = include_metadata;
        }
        if let Some(category_filter) = req.category_filter {
            options.category_filter = Some(category_filter);
        }

        // 解析日期范围
        if let (Some(start_str), Some(end_str)) = (req.start_date, req.end_date) {
            if let (Ok(start), Ok(end)) = (
                DateTime::parse_from_rfc3339(&start_str),
                DateTime::parse_from_rfc3339(&end_str),
            ) {
                options.date_range = Some((start.with_timezone(&Local), end.with_timezone(&Local)));
            }
        }
    }

    options
}

/// 执行数据导出
async fn perform_data_export(
    state: &State<'_, AppState>,
    options: ExportOptions,
    file_path: &str,
) -> anyhow::Result<String> {
    // 获取数据库连接
    let storage = &state.storage;
    let database = storage.get_database();

    // 收集导出数据
    let export_data = gather_export_data(database, &options).await?;

    // 创建导出器并执行导出
    let exporter = DataExporter::new(options);
    exporter.export_to_file(&export_data, file_path)?;

    // 返回导出摘要
    let summary = format!(
        "成功导出 {} 条时间记录和 {} 个分类到文件: {}",
        export_data.time_entries.len(),
        export_data.categories.len(),
        file_path
    );

    Ok(summary)
}

/// 收集导出数据
async fn gather_export_data(
    database: &crate::storage::Database,
    options: &ExportOptions,
) -> anyhow::Result<crate::utils::export::ExportData> {
    // 获取所有时间记录
    let mut time_entries = database
        .get_all_time_entries()
        .map_err(|e| anyhow::anyhow!("获取时间记录失败: {}", e))?;

    // 应用日期范围过滤
    if let Some((start, end)) = options.date_range {
        time_entries.retain(|entry| entry.start_time >= start && entry.start_time <= end);
    }

    // 应用分类过滤
    if let Some(category_filter) = &options.category_filter {
        let uncategorized = "uncategorized".to_string();
        time_entries.retain(|entry| {
            if let Some(category_id) = &entry.category_id {
                category_filter.contains(&category_id.to_string())
            } else {
                category_filter.contains(&uncategorized)
            }
        });
    }

    // 获取分类数据
    use crate::core::category::{CategoryColor, CategoryIcon};
    use chrono::Duration;

    let categories = if options.include_categories {
        database
            .get_all_categories()
            .map_err(|e| anyhow::anyhow!("获取分类失败: {}", e))?
            .into_iter()
            .map(|cat| crate::core::Category {
                id: cat.id,
                name: cat.name,
                description: cat.description,
                color: CategoryColor::from_hex(&cat.color),
                icon: CategoryIcon::Other, // TODO: 根据存储值映射实际图标
                created_at: cat.created_at,
                updated_at: cat.updated_at.unwrap_or(cat.created_at),
                daily_target: cat.daily_target_seconds.map(Duration::seconds),
                weekly_target: cat.weekly_target_seconds.map(Duration::seconds),
                target_duration: None,
                is_active: cat.is_active,
                sort_order: cat.sort_order,
                parent_id: cat.parent_id,
            })
            .collect()
    } else {
        Vec::new()
    };

    // 创建导出数据
    let export_data = create_export_data(time_entries, categories, options);

    Ok(export_data)
}

/// 导入数据
#[tauri::command]
pub async fn import_data(
    _state: State<'_, AppState>,
    app_handle: AppHandle,
    file_path: String,
) -> Result<String, String> {
    log::info!("开始导入数据: {}", file_path);

    // 检查文件是否存在
    if !std::path::Path::new(&file_path).exists() {
        return Err("文件不存在".to_string());
    }

    // TODO: 实现数据导入逻辑
    // 1. 读取文件内容
    // 2. 解析数据格式
    // 3. 验证数据完整性
    // 4. 执行数据导入
    // 5. 处理冲突和重复数据

    // 暂时返回成功消息
    let summary = "数据导入功能正在开发中".to_string();
    log::info!("数据导入完成: {}", summary);

    // 发送数据变化事件
    if let Err(e) = app_handle.emit("data_changed", "data_imported") {
        log::warn!("发送数据导入事件失败: {}", e);
    }

    Ok(summary)
}

/// 备份数据库
#[tauri::command]
pub async fn backup_database(
    state: State<'_, AppState>,
    dest_path: String,
) -> Result<String, String> {
    log::info!("开始备份数据库到: {}", dest_path);

    let storage = &state.storage;
    storage
        .backup_database(&dest_path)
        .map_err(|e| format!("数据库备份失败: {}", e))?;

    log::info!("数据库备份完成");
    Ok("数据库备份成功".to_string())
}

/// 恢复数据库
#[tauri::command]
pub async fn restore_database(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    src_path: String,
) -> Result<String, String> {
    log::info!("开始恢复数据库: {}", src_path);

    // 检查备份文件是否存在
    if !std::path::Path::new(&src_path).exists() {
        return Err("备份文件不存在".to_string());
    }

    let storage = &state.storage;
    storage
        .restore_database_from_backup(&src_path)
        .map_err(|e| format!("数据库恢复失败: {}", e))?;

    log::info!("数据库恢复完成");

    // 发送数据变化事件
    if let Err(e) = app_handle.emit("data_changed", "database_restored") {
        log::warn!("发送数据库恢复事件失败: {}", e);
    }

    Ok("数据库恢复成功".to_string())
}

/// 清空所有数据
#[tauri::command]
pub async fn clear_all_data(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    log::warn!("开始清空所有数据");

    let storage = &state.storage;

    // 使用SQL直接清空表数据
    if let Ok(conn) = storage.get_database().get_connection() {
        let conn = conn.get_raw_connection();
        let conn = conn.lock().unwrap();

        // 按依赖关系顺序删除所有数据
        let tables = [
            "time_entries",
            "transactions",
            "tasks",
            "categories",
            "accounts",
        ];
        for table in &tables {
            if let Err(e) = conn.execute(&format!("DELETE FROM {}", table), rusqlite::params![]) {
                log::error!("清除表 {} 失败: {}", table, e);
                return Err(format!("清除失败，请重试: {}", e));
            }
        }

        // 重置自增ID
        if let Err(e) = conn.execute("DELETE FROM sqlite_sequence", rusqlite::params![]) {
            log::debug!("清理sqlite_sequence表失败（可能不存在）: {}", e);
        }
    }

    log::info!("所有数据已清空");

    // 发送数据变化事件
    if let Err(e) = app_handle.emit("data_changed", "all_data_cleared") {
        log::warn!("发送数据清空事件失败: {}", e);
    }

    Ok("所有数据已清空".to_string())
}

// ========== 数据统计功能 ==========

/// 数据管理统计信息 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStatisticsDto {
    pub total_tasks: u32,
    pub total_time_spent: i64,
    pub total_transactions: u32,
    pub total_notes: u32,
    pub database_size: String,
    pub last_backup: String,
}

/// 获取数据管理统计信息
#[tauri::command]
pub async fn get_data_statistics(state: State<'_, AppState>) -> Result<DataStatisticsDto, String> {
    log::debug!("[CMD] get_data_statistics: Starting data statistics collection");

    let storage = &state.storage;

    // 获取任务总数
    let total_tasks = storage
        .get_database()
        .get_all_tasks()
        .map_err(|e| e.to_string())?
        .len() as u32;
    log::debug!("[CMD] get_data_statistics: Total tasks: {}", total_tasks);

    // 获取总时间记录（以秒为单位）
    let total_time_spent = storage
        .get_database()
        .get_all_time_entries()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|entry| entry.duration_seconds)
        .sum::<i64>();
    log::debug!(
        "[CMD] get_data_statistics: Total time spent: {} seconds",
        total_time_spent
    );

    // 获取交易总数
    let total_transactions = storage
        .get_database()
        .get_all_transactions()
        .map_err(|e| e.to_string())?
        .len() as u32;
    log::debug!(
        "[CMD] get_data_statistics: Total transactions: {}",
        total_transactions
    );

    // 获取笔记总数 (暂时使用0，因为还没有实现笔记功能)
    let total_notes = 0u32;
    log::debug!("[CMD] get_data_statistics: Total notes: {}", total_notes);

    // 获取数据库大小
    let database_size = match storage.get_database_stats() {
        Ok(stats) => stats.get_formatted_size(),
        Err(e) => {
            log::warn!(
                "[CMD] get_data_statistics: Could not get database size: {}",
                e
            );
            "未知".to_string()
        }
    };
    log::debug!(
        "[CMD] get_data_statistics: Database size: {}",
        database_size
    );

    // 获取最后备份时间 (暂时使用固定值，实际应该从配置或文件系统获取)
    let last_backup = "从未".to_string();
    log::debug!("[CMD] get_data_statistics: Last backup: {}", last_backup);

    let stats = DataStatisticsDto {
        total_tasks,
        total_time_spent,
        total_transactions,
        total_notes,
        database_size,
        last_backup,
    };

    log::debug!("[CMD] get_data_statistics: Statistics collected successfully");
    Ok(stats)
}
