//! # CLI输出格式化模块
//!
//! 提供各种数据的格式化输出功能，支持多种输出格式

use crate::{
    cli::OutputFormat,
    errors::Result,
    storage::models::*,
    utils::truncate_string,
};
use colored::*;

/// 输出格式化器
pub struct OutputFormatter {
    /// 输出格式
    format: OutputFormat,
    /// 是否详细输出
    verbose: bool,
    /// 是否静默模式
    quiet: bool,
}

impl OutputFormatter {
    /// 创建新的输出格式化器
    pub fn new(format: OutputFormat, verbose: bool, quiet: bool) -> Self {
        Self {
            format,
            verbose,
            quiet,
        }
    }

    /// 格式化时间记录列表
    pub fn format_time_entries(&self, entries: &[TimeEntry]) -> Result<()> {
        if self.quiet {
            return Ok(());
        }

        match self.format {
            OutputFormat::Table => self.format_time_entries_table(entries),
            OutputFormat::Json => self.format_time_entries_json(entries),
            OutputFormat::Csv => self.format_time_entries_csv(entries),
            OutputFormat::Simple => self.format_time_entries_simple(entries),
        }
    }

    /// 格式化分类列表
    pub fn format_categories(&self, categories: &[CategoryModel]) -> Result<()> {
        if self.quiet {
            return Ok(());
        }

        match self.format {
            OutputFormat::Table => self.format_categories_table(categories),
            OutputFormat::Json => self.format_categories_json(categories),
            OutputFormat::Csv => self.format_categories_csv(categories),
            OutputFormat::Simple => self.format_categories_simple(categories),
        }
    }

    /// 格式化统计信息
    pub fn format_stats(&self, stats: &TimeStats, detailed: bool) -> Result<()> {
        if self.quiet {
            return Ok(());
        }

        match self.format {
            OutputFormat::Table => self.format_stats_table(stats, detailed),
            OutputFormat::Json => self.format_stats_json(stats),
            OutputFormat::Csv => self.format_stats_csv(stats),
            OutputFormat::Simple => self.format_stats_simple(stats, detailed),
        }
    }

    // ==================== 时间记录格式化 ====================

    /// 表格格式输出时间记录
    fn format_time_entries_table(&self, entries: &[TimeEntry]) -> Result<()> {
        if entries.is_empty() {
            println!("{}", "没有找到时间记录".yellow());
            return Ok(());
        }

        // 计算列宽
        let max_task_width = entries
            .iter()
            .map(|e| e.task_name.len())
            .max()
            .unwrap_or(10)
            .max(10)
            .min(30); // 限制最大宽度

        let max_category_width = 12;
        let max_duration_width = 10;
        let max_date_width = 16;

        // 打印表头
        println!(
            "{:<width_date$} {:<width_task$} {:<width_cat$} {:<width_dur$} {}",
            center_text("日期时间", max_date_width).bold(),
            center_text("任务名称", max_task_width).bold(),
            center_text("分类", max_category_width).bold(),
            center_text("持续时间", max_duration_width).bold(),
            "标签".bold(),
            width_date = max_date_width,
            width_task = max_task_width,
            width_cat = max_category_width,
            width_dur = max_duration_width,
        );

        println!(
            "{} {} {} {} {}",
            "-".repeat(max_date_width),
            "-".repeat(max_task_width),
            "-".repeat(max_category_width),
            "-".repeat(max_duration_width),
            "-".repeat(10),
        );

        // 打印数据行
        for entry in entries {
            let task_name = truncate_string(&entry.task_name, max_task_width);
            let category = "未分类"; // 简化处理
            let duration = if let Some(end) = entry.end_time {
                let dur = end.signed_duration_since(entry.start_time);
                format_duration(dur.num_seconds())
            } else {
                "进行中".to_string()
            };

            let tags_display = if entry.tags.is_empty() {
                "-".to_string()
            } else {
                truncate_string(&entry.tags.join(", "), 20)
            };

            println!(
                "{:<width_date$} {:<width_task$} {:<width_cat$} {:<width_dur$} {}",
                entry.start_time.format("%m-%d %H:%M"),
                task_name.cyan(),
                category.yellow(),
                duration.green(),
                tags_display.dimmed(),
                width_date = max_date_width,
                width_task = max_task_width,
                width_cat = max_category_width,
                width_dur = max_duration_width,
            );
        }

        // 使用show_progress_bar显示一个简单的统计
        let completed_count = entries.iter().filter(|e| e.end_time.is_some()).count();
        println!();
        self.show_progress_bar(completed_count, entries.len(), "完成进度");

        Ok(())
    }

    /// JSON格式输出时间记录
    fn format_time_entries_json(&self, entries: &[TimeEntry]) -> Result<()> {
        let json = serde_json::to_string_pretty(entries)?;
        println!("{}", json);
        Ok(())
    }

    /// CSV格式输出时间记录
    fn format_time_entries_csv(&self, entries: &[TimeEntry]) -> Result<()> {
        // 打印CSV头
        println!("日期,任务名称,分类,开始时间,结束时间,持续时间(秒),描述,标签");

        // 打印数据
        for entry in entries {
            let date = entry.start_time.format("%Y-%m-%d").to_string();
            let start_time = entry.start_time.format("%H:%M:%S").to_string();
            let end_time = entry
                .end_time
                .map(|t| t.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "进行中".to_string());
            let description = entry.description.as_deref().unwrap_or("");
            let tags = entry.tags.join(";");

            println!(
                "{},{},未分类,{},{},{},{},{}",
                date,
                escape_csv(&entry.task_name), // TODO: 查询分类名称
                start_time,
                end_time,
                entry.duration_seconds,
                escape_csv(description),
                escape_csv(&tags),
            );
        }

        Ok(())
    }

    /// 简洁格式输出时间记录
    fn format_time_entries_simple(&self, entries: &[TimeEntry]) -> Result<()> {
        if entries.is_empty() {
            println!("没有找到时间记录");
            return Ok(());
        }

        for entry in entries {
            let status_icon = if entry.is_running() {
                "●".green()
            } else {
                "○".white()
            };

            let date = entry.start_time.format("%m-%d %H:%M").to_string();
            let duration = format_duration(entry.duration_seconds);

            println!(
                "{} {} {} ({})",
                status_icon,
                date.blue(),
                entry.task_name.bold(),
                duration.yellow(),
            );

            if self.verbose {
                if let Some(desc) = &entry.description {
                    println!("   描述: {}", desc.dimmed());
                }
                if !entry.tags.is_empty() {
                    println!("   标签: {}", entry.tags.join(", ").dimmed());
                }
            }
        }

        Ok(())
    }

    // ==================== 分类格式化 ====================

    /// 表格格式输出分类
    fn format_categories_table(&self, categories: &[CategoryModel]) -> Result<()> {
        if categories.is_empty() {
            println!("{}", "没有找到分类".yellow());
            return Ok(());
        }

        // 计算列宽
        let max_name_width = categories
            .iter()
            .map(|c| c.name.len())
            .max()
            .unwrap_or(10)
            .max(10);

        let max_desc_width = 30;

        // 打印表头
        println!(
            "{:<width_name$} {:<width_desc$} {:<8} {:<10} {:<8}",
            "名称".bold(),
            "描述".bold(),
            "颜色".bold(),
            "图标".bold(),
            "状态".bold(),
            width_name = max_name_width,
            width_desc = max_desc_width,
        );

        println!(
            "{} {} {} {} {}",
            "-".repeat(max_name_width),
            "-".repeat(max_desc_width),
            "-".repeat(8),
            "-".repeat(10),
            "-".repeat(8),
        );

        // 打印数据行
        for category in categories {
            let description = category.description.as_deref().unwrap_or("-");
            let desc_display = if description.len() > max_desc_width {
                format!("{}...", &description[..max_desc_width - 3])
            } else {
                description.to_string()
            };

            let status = if category.is_active {
                "激活".green()
            } else {
                "停用".red()
            };

            println!(
                "{:<width_name$} {:<width_desc$} {:<8} {:<10} {}",
                category.name.bold(),
                desc_display,
                category.color.blue(),
                category.icon.yellow(),
                status,
                width_name = max_name_width,
                width_desc = max_desc_width,
            );
        }

        println!();
        println!("{} {}", "总计:".bold(), categories.len().to_string().cyan());

        Ok(())
    }

    /// JSON格式输出分类
    fn format_categories_json(&self, categories: &[CategoryModel]) -> Result<()> {
        let json = serde_json::to_string_pretty(categories)?;
        println!("{}", json);
        Ok(())
    }

    /// CSV格式输出分类
    fn format_categories_csv(&self, categories: &[CategoryModel]) -> Result<()> {
        println!("名称,描述,颜色,图标,状态,排序,创建时间");

        for category in categories {
            let description = category.description.as_deref().unwrap_or("");
            let status = if category.is_active {
                "激活"
            } else {
                "停用"
            };
            let created_at = category.created_at.format("%Y-%m-%d %H:%M:%S").to_string();

            println!(
                "{},{},{},{},{},{},{}",
                escape_csv(&category.name),
                escape_csv(description),
                category.color,
                category.icon,
                status,
                category.sort_order,
                created_at,
            );
        }

        Ok(())
    }

    /// 简洁格式输出分类
    fn format_categories_simple(&self, categories: &[CategoryModel]) -> Result<()> {
        if categories.is_empty() {
            println!("没有找到分类");
            return Ok(());
        }

        for category in categories {
            let status_icon = if category.is_active {
                "●".green()
            } else {
                "○".red()
            };

            println!(
                "{} {} {}",
                status_icon,
                category.name.bold(),
                category.color.blue(),
            );

            if self.verbose {
                if let Some(desc) = &category.description {
                    println!("   描述: {}", desc.dimmed());
                }
                println!(
                    "   图标: {} | 排序: {}",
                    category.icon.yellow(),
                    category.sort_order
                );
            }
        }

        Ok(())
    }

    // ==================== 统计信息格式化 ====================

    /// 表格格式输出统计信息
    fn format_stats_table(&self, stats: &TimeStats, detailed: bool) -> Result<()> {
        println!("{}", "=== 时间统计 ===".bold().blue());
        println!();

        // 基本统计
        println!(
            "{:<15} {}",
            "统计周期:".bold(),
            format!(
                "{} 到 {}",
                stats.start_date.format("%Y-%m-%d"),
                stats.end_date.format("%Y-%m-%d")
            )
            .cyan()
        );

        println!(
            "{:<15} {}",
            "总时长:".bold(),
            format_duration(stats.total_seconds).green()
        );

        println!(
            "{:<15} {}",
            "任务数量:".bold(),
            stats.task_count.to_string().yellow()
        );

        if stats.task_count > 0 {
            println!(
                "{:<15} {}",
                "平均时长:".bold(),
                format_duration(stats.average_seconds as i64).blue()
            );

            if detailed {
                println!(
                    "{:<15} {}",
                    "最长时长:".bold(),
                    format_duration(stats.max_seconds).green()
                );

                println!(
                    "{:<15} {}",
                    "最短时长:".bold(),
                    format_duration(stats.min_seconds).red()
                );
            }
        }

        // 效率指标
        if detailed && stats.task_count > 0 {
            println!();
            println!("{}", "=== 效率分析 ===".bold().blue());

            let days = (stats.end_date - stats.start_date).num_days() + 1;
            let daily_avg = stats.total_seconds as f64 / days as f64;

            println!(
                "{:<15} {}",
                "日均时长:".bold(),
                format_duration(daily_avg as i64).cyan()
            );

            let hourly_rate = stats.task_count as f64 / (stats.total_seconds as f64 / 3600.0);
            println!("{:<15} {:.2} 任务/小时", "任务效率:".bold(), hourly_rate);
        }

        Ok(())
    }

    /// JSON格式输出统计信息
    fn format_stats_json(&self, stats: &TimeStats) -> Result<()> {
        let json = serde_json::to_string_pretty(stats)?;
        println!("{}", json);
        Ok(())
    }

    /// CSV格式输出统计信息
    fn format_stats_csv(&self, stats: &TimeStats) -> Result<()> {
        println!("指标,值");
        println!("总时长(秒),{}", stats.total_seconds);
        println!("任务数量,{}", stats.task_count);
        println!("平均时长(秒),{:.0}", stats.average_seconds);
        println!("最长时长(秒),{}", stats.max_seconds);
        println!("最短时长(秒),{}", stats.min_seconds);
        Ok(())
    }

    /// 简洁格式输出统计信息
    fn format_stats_simple(&self, stats: &TimeStats, detailed: bool) -> Result<()> {
        use crate::utils::calculate_percentage;

        // 计算效率百分比（假设8小时为100%）
        let eight_hours = 8 * 3600;
        let efficiency = calculate_percentage(stats.total_seconds as f64, eight_hours as f64);

        println!(
            "📊 {} | {} | {} | {}",
            format!("总时长: {}", format_duration(stats.total_seconds)).green(),
            format!("任务数: {}", stats.task_count).yellow(),
            format!("平均: {}", format_duration(stats.average_seconds as i64)).blue(),
            format!("效率: {:.1}%", efficiency).cyan(),
        );

        if detailed && stats.task_count > 0 {
            println!(
                "   {} | {}",
                format!("最长: {}", format_duration(stats.max_seconds)).cyan(),
                format!("最短: {}", format_duration(stats.min_seconds)).magenta(),
            );

            // 使用utils中的format_percentage来显示时间分布
            let max_ratio =
                calculate_percentage(stats.max_seconds as f64, stats.total_seconds as f64);
            let min_ratio =
                calculate_percentage(stats.min_seconds as f64, stats.total_seconds as f64);
            println!("   分布: 最长占{:.1}%, 最短占{:.1}%", max_ratio, min_ratio);
        }

        Ok(())
    }

    // ==================== 进度条和图表 ====================

    /// 显示进度条
    pub fn show_progress_bar(&self, current: usize, total: usize, message: &str) {
        if self.quiet {
            return;
        }

        let percentage = if total > 0 {
            (current as f64 / total as f64 * 100.0) as usize
        } else {
            0
        };

        let bar_width = 30;
        let filled = (percentage * bar_width / 100).min(bar_width);
        let empty = bar_width - filled;

        let bar = format!(
            "[{}{}] {}% ({}/{})",
            "█".repeat(filled).green(),
            "░".repeat(empty).dimmed(),
            percentage,
            current,
            total
        );

        print!("\r{} {}", message, bar);
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        if current >= total {
            println!(); // 完成后换行
        }
    }

    /// 显示简单的条形图
    pub fn show_bar_chart(&self, data: &[(String, i64)], title: &str) -> Result<()> {
        if self.quiet || data.is_empty() {
            return Ok(());
        }

        println!("{}", title.bold().blue());
        println!();

        let max_value = data.iter().map(|(_, v)| *v).max().unwrap_or(1);
        let max_label_width = data.iter().map(|(l, _)| l.len()).max().unwrap_or(10);

        for (label, value) in data {
            let bar_length = if max_value > 0 {
                ((*value as f64 / max_value as f64) * 40.0) as usize
            } else {
                0
            };

            let bar = "█".repeat(bar_length);
            let percentage = if max_value > 0 {
                (*value as f64 / max_value as f64 * 100.0) as usize
            } else {
                0
            };

            println!(
                "{:<width$} {} {}% ({})",
                label.bold(),
                bar.green(),
                percentage,
                format_duration(*value),
                width = max_label_width
            );
        }

        Ok(())
    }
}

// ==================== 辅助函数 ====================

/// 格式化持续时间
fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", secs)
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

/// 截断文本
fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length.saturating_sub(3)])
    }
}

/// 居中文本
fn center_text(text: &str, width: usize) -> String {
    let text_len = text.len();
    if text_len >= width {
        return text.to_string();
    }

    let padding = width - text_len;
    let left_padding = padding / 2;
    let right_padding = padding - left_padding;

    format!(
        "{}{}{}",
        " ".repeat(left_padding),
        text,
        " ".repeat(right_padding)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m");
        assert_eq!(format_duration(150), "2m");
        assert_eq!(format_duration(3600), "1h 0m");
        assert_eq!(format_duration(3661), "1h 1m");
    }

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
        assert_eq!(escape_csv("with\nline"), "\"with\nline\"");
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("short", 10), "short");
        assert_eq!(truncate_text("this is a very long text", 10), "this is...");
        assert_eq!(truncate_text("exactly10c", 10), "exactly10c");
    }

    #[test]
    fn test_center_text() {
        assert_eq!(center_text("test", 10), "   test   ");
        assert_eq!(center_text("test", 9), "  test   ");
        assert_eq!(center_text("toolong", 5), "toolong");
    }

    #[test]
    fn test_output_formatter_creation() {
        let formatter = OutputFormatter::new(OutputFormat::Table, false, false);
        assert_eq!(formatter.format, OutputFormat::Table);
        assert!(!formatter.verbose);
        assert!(!formatter.quiet);
    }
}
