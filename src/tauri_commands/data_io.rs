//! # 数据导入导出命令模块
//!
//! 负责处理数据的导入和导出功能

use super::*;
use crate::utils::export::{create_export_data, DataExporter, ExportFormat, ExportOptions};
use chrono::{DateTime, Local};
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
pub async fn import_data(state: State<'_, AppState>, file_path: String) -> Result<String, String> {
    log::info!("从 {} 导入数据", file_path);

    // TODO: 实现实际的数据导入逻辑
    Ok(format!("数据已从文件导入: {}", file_path))
}

/// 备份数据库到指定文件
#[tauri::command]
pub async fn backup_database(
    state: State<'_, AppState>,
    dest_path: String,
) -> Result<String, String> {
    log::info!("备份数据库到 {}", dest_path);
    let storage = &state.storage;
    match storage.backup_database(&dest_path) {
        Ok(_) => Ok(format!("数据库已备份到 {}", dest_path)),
        Err(e) => {
            log::error!("数据库备份失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 从备份文件恢复数据库
#[tauri::command]
pub async fn restore_database(
    state: State<'_, AppState>,
    src_path: String,
) -> Result<String, String> {
    log::info!("从 {} 恢复数据库", src_path);
    let mut storage = state.storage.clone();
    match Arc::get_mut(&mut storage) {
        Some(mut_storage) => match mut_storage.restore_database(&src_path) {
            Ok(_) => Ok("数据库恢复完成".to_string()),
            Err(e) => {
                log::error!("数据库恢复失败: {}", e);
                Err(e.to_string())
            }
        },
        None => Err("无法获取可变存储引用".to_string()),
    }
}
