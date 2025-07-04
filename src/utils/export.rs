//! # 数据导出工具模块
//!
//! 提供各种格式的数据导出功能

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::core::Category;
use crate::storage::models::{CategoryModel, TimeEntry};
use anyhow::Result;

/// 导出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
    Xml,
    Html,
    Markdown,
    Pdf,
}

impl ExportFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            "xml" => Some(Self::Xml),
            "html" | "htm" => Some(Self::Html),
            "md" | "markdown" => Some(Self::Markdown),
            "pdf" => Some(Self::Pdf),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Xml => "xml",
            Self::Html => "html",
            Self::Markdown => "md",
            Self::Pdf => "pdf",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::Csv => "text/csv",
            Self::Xml => "application/xml",
            Self::Html => "text/html",
            Self::Markdown => "text/markdown",
            Self::Pdf => "application/pdf",
        }
    }
}

/// 导出选项
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_categories: bool,
    pub include_statistics: bool,
    pub date_range: Option<(DateTime<Local>, DateTime<Local>)>,
    pub category_filter: Option<Vec<String>>,
    pub group_by_date: bool,
    pub group_by_category: bool,
    pub include_metadata: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            include_categories: true,
            include_statistics: true,
            date_range: None,
            category_filter: None,
            group_by_date: false,
            group_by_category: false,
            include_metadata: true,
        }
    }
}

/// 导出数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub metadata: ExportMetadata,
    pub categories: Vec<Category>,
    pub time_entries: Vec<TimeEntry>,
    pub statistics: Option<ExportStatistics>,
}

/// 导出元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub export_time: DateTime<Local>,
    pub version: String,
    pub total_entries: usize,
    pub total_categories: usize,
    pub date_range: Option<(DateTime<Local>, DateTime<Local>)>,
    pub filters_applied: Vec<String>,
}

/// 导出统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStatistics {
    pub total_time: chrono::Duration,
    pub average_session_time: chrono::Duration,
    pub category_breakdown: HashMap<String, chrono::Duration>,
    pub daily_totals: HashMap<String, chrono::Duration>,
    pub most_productive_day: Option<String>,
    pub most_used_category: Option<String>,
}

/// 数据导出器
pub struct DataExporter {
    options: ExportOptions,
}

impl DataExporter {
    pub fn new(options: ExportOptions) -> Self {
        Self { options }
    }

    /// 导出数据到文件
    pub fn export_to_file<P: AsRef<Path>>(&self, data: &ExportData, file_path: P) -> Result<()> {
        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);

        match self.options.format {
            ExportFormat::Json => self.export_json(data, &mut writer)?,
            ExportFormat::Csv => self.export_csv(data, &mut writer)?,
            ExportFormat::Xml => self.export_xml(data, &mut writer)?,
            ExportFormat::Html => self.export_html(data, &mut writer)?,
            ExportFormat::Markdown => self.export_markdown(data, &mut writer)?,
            ExportFormat::Pdf => return Err(anyhow::Error::msg("PDF导出暂未实现")),
        }

        writer.flush()?;
        Ok(())
    }

    /// 导出为JSON格式
    fn export_json<W: Write>(&self, data: &ExportData, writer: &mut W) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        writer.write_all(json.as_bytes())?;
        Ok(())
    }

    /// 导出为CSV格式
    fn export_csv<W: Write>(&self, data: &ExportData, writer: &mut W) -> Result<()> {
        // CSV头部
        writeln!(writer, "ID,任务名称,分类,开始时间,结束时间,持续时间,描述")?;

        // 创建分类映射
        let category_map: HashMap<String, String> = data
            .categories
            .iter()
            .map(|c| (c.id.to_string(), c.name.clone()))
            .collect();

        // 导出时间记录
        for entry in &data.time_entries {
            let category_name = category_map
                .get(
                    &entry
                        .category_id
                        .as_ref()
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                )
                .map(|s| s.as_str())
                .unwrap_or("未知分类");

            let end_time = entry
                .end_time
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "进行中".to_string());

            let duration = if let Some(end) = entry.end_time {
                let dur = end.signed_duration_since(entry.start_time);
                crate::utils::format_duration(dur)
            } else {
                "进行中".to_string()
            };

            writeln!(
                writer,
                "{},{},{},{},{},{},{}",
                escape_csv(&entry.id.to_string()),
                escape_csv(&entry.task_name),
                escape_csv(category_name),
                entry.start_time.format("%Y-%m-%d %H:%M:%S"),
                end_time,
                duration,
                escape_csv(entry.description.as_deref().unwrap_or(""))
            )?;
        }

        Ok(())
    }

    /// 导出为XML格式
    fn export_xml<W: Write>(&self, data: &ExportData, writer: &mut W) -> Result<()> {
        writeln!(writer, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        writeln!(writer, "<timetracker_export>")?;

        // 元数据
        if self.options.include_metadata {
            writeln!(writer, "  <metadata>")?;
            writeln!(
                writer,
                "    <export_time>{}</export_time>",
                data.metadata.export_time.format("%Y-%m-%d %H:%M:%S")
            )?;
            writeln!(
                writer,
                "    <version>{}</version>",
                escape_xml(&data.metadata.version)
            )?;
            writeln!(
                writer,
                "    <total_entries>{}</total_entries>",
                data.metadata.total_entries
            )?;
            writeln!(
                writer,
                "    <total_categories>{}</total_categories>",
                data.metadata.total_categories
            )?;
            writeln!(writer, "  </metadata>")?;
        }

        // 分类
        if self.options.include_categories {
            writeln!(writer, "  <categories>")?;
            for category in &data.categories {
                writeln!(writer, "    <category>")?;
                writeln!(
                    writer,
                    "      <id>{}</id>",
                    escape_xml(&category.id.to_string())
                )?;
                writeln!(writer, "      <name>{}</name>", escape_xml(&category.name))?;
                if let Some(ref desc) = category.description {
                    writeln!(
                        writer,
                        "      <description>{}</description>",
                        escape_xml(desc)
                    )?;
                }
                writeln!(
                    writer,
                    "      <color>{}</color>",
                    escape_xml(&category.color.to_hex())
                )?;
                writeln!(
                    writer,
                    "      <created_at>{}</created_at>",
                    category.created_at.format("%Y-%m-%d %H:%M:%S")
                )?;
                writeln!(writer, "    </category>")?;
            }
            writeln!(writer, "  </categories>")?;
        }

        // 时间记录
        writeln!(writer, "  <time_entries>")?;
        for entry in &data.time_entries {
            writeln!(writer, "    <time_entry>")?;
            writeln!(
                writer,
                "      <id>{}</id>",
                escape_xml(&entry.id.to_string())
            )?;
            writeln!(
                writer,
                "      <task_name>{}</task_name>",
                escape_xml(&entry.task_name)
            )?;
            writeln!(
                writer,
                "      <category_id>{}</category_id>",
                escape_xml(
                    &entry
                        .category_id
                        .as_ref()
                        .map(|id| id.to_string())
                        .unwrap_or_default()
                )
            )?;
            writeln!(
                writer,
                "      <start_time>{}</start_time>",
                entry.start_time.format("%Y-%m-%d %H:%M:%S")
            )?;
            if let Some(end_time) = entry.end_time {
                writeln!(
                    writer,
                    "      <end_time>{}</end_time>",
                    end_time.format("%Y-%m-%d %H:%M:%S")
                )?;
            }
            if let Some(ref desc) = entry.description {
                writeln!(
                    writer,
                    "      <description>{}</description>",
                    escape_xml(desc)
                )?;
            }
            writeln!(writer, "    </time_entry>")?;
        }
        writeln!(writer, "  </time_entries>")?;

        writeln!(writer, "</timetracker_export>")?;
        Ok(())
    }

    /// 导出为HTML格式
    fn export_html<W: Write>(&self, data: &ExportData, writer: &mut W) -> Result<()> {
        writeln!(writer, "<!DOCTYPE html>")?;
        writeln!(writer, "<html lang=\"zh-CN\">")?;
        writeln!(writer, "<head>")?;
        writeln!(writer, "  <meta charset=\"UTF-8\">")?;
        writeln!(
            writer,
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">"
        )?;
        writeln!(writer, "  <title>TimeTracker 导出报告</title>")?;
        writeln!(writer, "  <style>")?;
        writeln!(
            writer,
            "    body {{ font-family: Arial, sans-serif; margin: 20px; }}"
        )?;
        writeln!(
            writer,
            "    table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}"
        )?;
        writeln!(
            writer,
            "    th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}"
        )?;
        writeln!(writer, "    th {{ background-color: #f2f2f2; }}")?;
        writeln!(
            writer,
            "    .metadata {{ background-color: #f9f9f9; padding: 15px; margin: 20px 0; }}"
        )?;
        writeln!(
            writer,
            "    .statistics {{ background-color: #e9f7ef; padding: 15px; margin: 20px 0; }}"
        )?;
        writeln!(writer, "  </style>")?;
        writeln!(writer, "</head>")?;
        writeln!(writer, "<body>")?;

        writeln!(writer, "  <h1>TimeTracker 导出报告</h1>")?;

        // 元数据
        if self.options.include_metadata {
            writeln!(writer, "  <div class=\"metadata\">")?;
            writeln!(writer, "    <h2>导出信息</h2>")?;
            writeln!(
                writer,
                "    <p><strong>导出时间:</strong> {}</p>",
                data.metadata.export_time.format("%Y-%m-%d %H:%M:%S")
            )?;
            writeln!(
                writer,
                "    <p><strong>版本:</strong> {}</p>",
                escape_html(&data.metadata.version)
            )?;
            writeln!(
                writer,
                "    <p><strong>记录总数:</strong> {}</p>",
                data.metadata.total_entries
            )?;
            writeln!(
                writer,
                "    <p><strong>分类总数:</strong> {}</p>",
                data.metadata.total_categories
            )?;
            writeln!(writer, "  </div>")?;
        }

        // 统计信息
        if let Some(ref stats) = data.statistics {
            writeln!(writer, "  <div class=\"statistics\">")?;
            writeln!(writer, "    <h2>统计信息</h2>")?;
            writeln!(
                writer,
                "    <p><strong>总时间:</strong> {}</p>",
                crate::utils::format_duration(stats.total_time)
            )?;
            writeln!(
                writer,
                "    <p><strong>平均会话时间:</strong> {}</p>",
                crate::utils::format_duration(stats.average_session_time)
            )?;
            if let Some(ref day) = stats.most_productive_day {
                writeln!(
                    writer,
                    "    <p><strong>最高效的一天:</strong> {}</p>",
                    escape_html(day)
                )?;
            }
            if let Some(ref category) = stats.most_used_category {
                writeln!(
                    writer,
                    "    <p><strong>最常用分类:</strong> {}</p>",
                    escape_html(category)
                )?;
            }
            writeln!(writer, "  </div>")?;
        }

        // 分类表格
        if self.options.include_categories && !data.categories.is_empty() {
            writeln!(writer, "  <h2>分类列表</h2>")?;
            writeln!(writer, "  <table>")?;
            writeln!(writer, "    <thead>")?;
            writeln!(
                writer,
                "      <tr><th>名称</th><th>描述</th><th>颜色</th><th>创建时间</th></tr>"
            )?;
            writeln!(writer, "    </thead>")?;
            writeln!(writer, "    <tbody>")?;

            for category in &data.categories {
                writeln!(writer, "      <tr>")?;
                writeln!(writer, "        <td>{}</td>", escape_html(&category.name))?;
                writeln!(
                    writer,
                    "        <td>{}</td>",
                    escape_html(category.description.as_deref().unwrap_or(""))
                )?;
                writeln!(
                    writer,
                    "        <td style=\"background-color: {}\">{}</td>",
                    category.color,
                    escape_html(&category.color.to_hex())
                )?;
                writeln!(
                    writer,
                    "        <td>{}</td>",
                    category.created_at.format("%Y-%m-%d %H:%M")
                )?;
                writeln!(writer, "      </tr>")?;
            }

            writeln!(writer, "    </tbody>")?;
            writeln!(writer, "  </table>")?;
        }

        // 时间记录表格
        writeln!(writer, "  <h2>时间记录</h2>")?;
        writeln!(writer, "  <table>")?;
        writeln!(writer, "    <thead>")?;
        writeln!(writer, "      <tr><th>任务名称</th><th>分类</th><th>开始时间</th><th>结束时间</th><th>持续时间</th><th>描述</th></tr>")?;
        writeln!(writer, "    </thead>")?;
        writeln!(writer, "    <tbody>")?;

        let category_map: HashMap<String, String> = data
            .categories
            .iter()
            .map(|c| (c.id.to_string(), c.name.clone()))
            .collect();

        for entry in &data.time_entries {
            let category_name = category_map
                .get(
                    &entry
                        .category_id
                        .as_ref()
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                )
                .map(|s| s.as_str())
                .unwrap_or("未知分类");

            let end_time = entry
                .end_time
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "进行中".to_string());

            let duration = if let Some(end) = entry.end_time {
                let dur = end.signed_duration_since(entry.start_time);
                crate::utils::format_duration(dur)
            } else {
                "进行中".to_string()
            };

            writeln!(writer, "      <tr>")?;
            writeln!(writer, "        <td>{}</td>", escape_html(&entry.task_name))?;
            writeln!(writer, "        <td>{}</td>", escape_html(category_name))?;
            writeln!(
                writer,
                "        <td>{}</td>",
                entry.start_time.format("%Y-%m-%d %H:%M:%S")
            )?;
            writeln!(writer, "        <td>{}</td>", end_time)?;
            writeln!(writer, "        <td>{}</td>", duration)?;
            writeln!(
                writer,
                "        <td>{}</td>",
                escape_html(entry.description.as_deref().unwrap_or(""))
            )?
        }

        writeln!(writer, "    </tbody>")?;
        writeln!(writer, "  </table>")?;
        writeln!(writer, "</body>")?;
        writeln!(writer, "</html>")?;

        Ok(())
    }

    /// 导出为Markdown格式
    fn export_markdown<W: Write>(&self, data: &ExportData, writer: &mut W) -> Result<()> {
        writeln!(writer, "# TimeTracker 导出报告")?;
        writeln!(writer)?;

        // 元数据
        if self.options.include_metadata {
            writeln!(writer, "## 导出信息")?;
            writeln!(writer)?;
            writeln!(
                writer,
                "- **导出时间:** {}",
                data.metadata.export_time.format("%Y-%m-%d %H:%M:%S")
            )?;
            writeln!(writer, "- **版本:** {}", data.metadata.version)?;
            writeln!(writer, "- **记录总数:** {}", data.metadata.total_entries)?;
            writeln!(writer, "- **分类总数:** {}", data.metadata.total_categories)?;
            writeln!(writer)?;
        }

        // 统计信息
        if let Some(ref stats) = data.statistics {
            writeln!(writer, "## 统计信息")?;
            writeln!(writer)?;
            writeln!(
                writer,
                "- **总时间:** {}",
                crate::utils::format_duration(stats.total_time)
            )?;
            writeln!(
                writer,
                "- **平均会话时间:** {}",
                crate::utils::format_duration(stats.average_session_time)
            )?;
            if let Some(ref day) = stats.most_productive_day {
                writeln!(writer, "- **最高效的一天:** {}", day)?;
            }
            if let Some(ref category) = stats.most_used_category {
                writeln!(writer, "- **最常用分类:** {}", category)?;
            }
            writeln!(writer)?;
        }

        // 分类列表
        if self.options.include_categories && !data.categories.is_empty() {
            writeln!(writer, "## 分类列表")?;
            writeln!(writer)?;
            writeln!(writer, "| 名称 | 描述 | 颜色 | 创建时间 |")?;
            writeln!(writer, "|------|------|------|----------|")?;

            for category in &data.categories {
                writeln!(
                    writer,
                    "| {} | {} | {} | {} |",
                    escape_markdown(&category.name),
                    escape_markdown(category.description.as_deref().unwrap_or("")),
                    category.color,
                    category.created_at.format("%Y-%m-%d %H:%M")
                )?;
            }
            writeln!(writer)?;
        }

        // 时间记录
        writeln!(writer, "## 时间记录")?;
        writeln!(writer)?;
        writeln!(
            writer,
            "| 任务名称 | 分类 | 开始时间 | 结束时间 | 持续时间 | 描述 |"
        )?;
        writeln!(
            writer,
            "|----------|------|----------|----------|----------|------|"
        )?;

        let category_map: HashMap<String, String> = data
            .categories
            .iter()
            .map(|c| (c.id.to_string(), c.name.clone()))
            .collect();

        for entry in &data.time_entries {
            let category_name = category_map
                .get(
                    &entry
                        .category_id
                        .as_ref()
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                )
                .map(|s| s.as_str())
                .unwrap_or("未知分类");

            let end_time = entry
                .end_time
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "进行中".to_string());

            let duration = if let Some(end) = entry.end_time {
                let dur = end.signed_duration_since(entry.start_time);
                crate::utils::format_duration(dur)
            } else {
                "进行中".to_string()
            };

            writeln!(
                writer,
                "| {} | {} | {} | {} | {} | {} |",
                escape_markdown(&entry.task_name),
                escape_markdown(category_name),
                entry.start_time.format("%Y-%m-%d %H:%M:%S"),
                end_time,
                duration,
                escape_markdown(entry.description.as_deref().unwrap_or(""))
            )?;
        }

        Ok(())
    }
}

/// 转义CSV字段
fn escape_csv(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// 转义XML内容
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// 转义HTML内容
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// 转义Markdown内容
fn escape_markdown(text: &str) -> String {
    text.replace('|', "\\|")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

/// 计算导出统计信息
pub fn calculate_export_statistics(
    time_entries: &[TimeEntry],
    categories: &[Category],
) -> ExportStatistics {
    let mut total_time = chrono::Duration::zero();
    let mut category_breakdown = HashMap::new();
    let mut daily_totals = HashMap::new();
    let mut completed_entries = 0;

    // 创建分类映射
    let category_map: HashMap<String, String> = categories
        .iter()
        .map(|c| (c.id.to_string(), c.name.clone()))
        .collect();

    for entry in time_entries {
        if let Some(end_time) = entry.end_time {
            let duration = end_time.signed_duration_since(entry.start_time);
            total_time += duration;
            completed_entries += 1;

            // 按分类统计
            let category_name = if let Some(category_id) = &entry.category_id {
                category_map
                    .get(&category_id.to_string())
                    .cloned()
                    .unwrap_or_else(|| "未知分类".to_string())
            } else {
                "未分类".to_string()
            };

            *category_breakdown
                .entry(category_name)
                .or_insert(chrono::Duration::zero()) += duration;

            // 按日期统计
            let date_key = entry.start_time.format("%Y-%m-%d").to_string();
            *daily_totals
                .entry(date_key)
                .or_insert(chrono::Duration::zero()) += duration;
        }
    }

    let average_session_time = if completed_entries > 0 {
        total_time / completed_entries
    } else {
        chrono::Duration::zero()
    };

    let most_productive_day = daily_totals
        .iter()
        .max_by_key(|(_, duration)| *duration)
        .map(|(date, _)| date.clone());

    let most_used_category = category_breakdown
        .iter()
        .max_by_key(|(_, duration)| *duration)
        .map(|(category, _)| category.clone());

    ExportStatistics {
        total_time,
        average_session_time,
        category_breakdown,
        daily_totals,
        most_productive_day,
        most_used_category,
    }
}

/// 创建导出数据
pub fn create_export_data(
    time_entries: Vec<TimeEntry>,
    categories: Vec<Category>,
    options: &ExportOptions,
) -> ExportData {
    let statistics = if options.include_statistics {
        Some(calculate_export_statistics(&time_entries, &categories))
    } else {
        None
    };

    let mut filters_applied = Vec::new();

    if options.date_range.is_some() {
        filters_applied.push("日期范围过滤".to_string());
    }

    if options.category_filter.is_some() {
        filters_applied.push("分类过滤".to_string());
    }

    let metadata = ExportMetadata {
        export_time: Local::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        total_entries: time_entries.len(),
        total_categories: categories.len(),
        date_range: options.date_range,
        filters_applied,
    };

    ExportData {
        metadata,
        categories,
        time_entries,
        statistics,
    }
}

/// 导出数据到JSON文件
pub fn export_to_json<P: AsRef<Path>>(data: &ExportData, file_path: P) -> Result<()> {
    let options = ExportOptions {
        format: ExportFormat::Json,
        ..Default::default()
    };
    let exporter = DataExporter::new(options);
    exporter.export_to_file(data, file_path)
}

/// 从时间记录和分类创建导出数据
pub fn create_simple_export_data(
    time_entries: Vec<TimeEntry>,
    categories: Vec<CategoryModel>,
) -> ExportData {
    // 转换CategoryModel为Category
    let converted_categories: Vec<Category> = categories
        .into_iter()
        .map(|cm| {
            use crate::core::category::{CategoryColor, CategoryIcon};

            Category {
                id: cm.id,
                name: cm.name,
                description: cm.description,
                color: CategoryColor::from_hex(&cm.color),
                icon: CategoryIcon::Work, // 默认图标，可以后续根据 cm.icon 字段解析
                created_at: cm.created_at,
                updated_at: cm.updated_at.unwrap_or_else(|| Local::now()),
                daily_target: cm
                    .daily_target_seconds
                    .map(|s| chrono::Duration::seconds(s)),
                weekly_target: cm
                    .weekly_target_seconds
                    .map(|s| chrono::Duration::seconds(s)),
                target_duration: None,
                is_active: cm.is_active,
                sort_order: cm.sort_order,
                parent_id: cm.parent_id,
            }
        })
        .collect();

    let metadata = ExportMetadata {
        export_time: Local::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        total_entries: time_entries.len(),
        total_categories: converted_categories.len(),
        date_range: None,
        filters_applied: vec![],
    };

    ExportData {
        metadata,
        categories: converted_categories,
        time_entries,
        statistics: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::io::Cursor;

    fn create_test_data() -> ExportData {
        use crate::core::category::{CategoryColor, CategoryIcon};
        use uuid::Uuid;

        let category_id = Uuid::new_v4();
        let categories = vec![Category {
            id: category_id,
            name: "工作".to_string(),
            description: Some("工作相关任务".to_string()),
            color: CategoryColor::Red,
            icon: CategoryIcon::Work,
            created_at: Local.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
            updated_at: Local.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
            daily_target: None,
            weekly_target: None,
            target_duration: None,
            is_active: true,
            sort_order: 0,
            parent_id: None,
        }];

        let time_entries = vec![TimeEntry {
            id: Uuid::new_v4(),
            task_name: "编程".to_string(),
            category_id: Some(category_id),
            start_time: Local.with_ymd_and_hms(2023, 1, 1, 9, 0, 0).unwrap(),
            end_time: Some(Local.with_ymd_and_hms(2023, 1, 1, 10, 0, 0).unwrap()),
            duration_seconds: 3600,
            description: Some("学习Rust".to_string()),
            tags: vec![],
            created_at: Local.with_ymd_and_hms(2023, 1, 1, 9, 0, 0).unwrap(),
            updated_at: Some(Local.with_ymd_and_hms(2023, 1, 1, 10, 0, 0).unwrap()),
        }];

        let metadata = ExportMetadata {
            export_time: Local::now(),
            version: "1.0.0".to_string(),
            total_entries: 1,
            total_categories: 1,
            date_range: None,
            filters_applied: vec![],
        };

        ExportData {
            metadata,
            categories,
            time_entries,
            statistics: None,
        }
    }

    #[test]
    fn test_export_format_from_extension() {
        assert_eq!(
            ExportFormat::from_extension("json"),
            Some(ExportFormat::Json)
        );
        assert_eq!(ExportFormat::from_extension("CSV"), Some(ExportFormat::Csv));
        assert_eq!(ExportFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_export_json() {
        let data = create_test_data();
        let options = ExportOptions {
            format: ExportFormat::Json,
            ..Default::default()
        };
        let exporter = DataExporter::new(options);

        let mut buffer = Cursor::new(Vec::new());
        exporter.export_json(&data, &mut buffer).unwrap();

        let result = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(result.contains("编程"));
        assert!(result.contains("工作"));
    }

    #[test]
    fn test_export_csv() {
        let data = create_test_data();
        let options = ExportOptions {
            format: ExportFormat::Csv,
            ..Default::default()
        };
        let exporter = DataExporter::new(options);

        let mut buffer = Cursor::new(Vec::new());
        exporter.export_csv(&data, &mut buffer).unwrap();

        let result = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(result.contains("任务名称"));
        assert!(result.contains("编程"));
        assert!(result.contains("工作"));
    }

    #[test]
    fn test_escape_functions() {
        assert_eq!(escape_csv("test,with,commas"), "\"test,with,commas\"");
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_markdown("test|with|pipes"), "test\\|with\\|pipes");
    }
}
