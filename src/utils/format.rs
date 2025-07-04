//! # 格式化工具模块
//!
//! 提供各种数据格式化功能

use chrono::Duration;

/// 格式化持续时间（默认使用紧凑格式）
pub fn format_duration(duration: Duration) -> String {
    format_duration_compact(duration)
}

/// 格式化持续时间为详细格式
pub fn format_duration_detailed(duration: Duration) -> String {
    let total_seconds = duration.num_seconds().abs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    let mut parts = Vec::new();

    if days > 0 {
        parts.push(format!("{}天", days));
    }
    if hours > 0 {
        parts.push(format!("{}小时", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}分钟", minutes));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{}秒", seconds));
    }

    let result = parts.join("");
    if duration.num_seconds() < 0 {
        format!("-{}", result)
    } else {
        result
    }
}

/// 格式化持续时间为紧凑格式 (HH:MM:SS)
pub fn format_duration_compact(duration: Duration) -> String {
    let total_seconds = duration.num_seconds().abs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    let result = if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    };

    if duration.num_seconds() < 0 {
        format!("-{}", result)
    } else {
        result
    }
}

/// 格式化持续时间为小数小时格式
pub fn format_duration_decimal_hours(duration: Duration) -> String {
    let hours = duration.num_seconds() as f64 / 3600.0;
    format!("{:.2}h", hours)
}

/// 格式化数字为千分位格式
pub fn format_number_with_commas(number: i64) -> String {
    let mut result = String::new();
    let number_str = number.abs().to_string();
    let chars: Vec<char> = number_str.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }

    if number < 0 {
        format!("-{}", result)
    } else {
        result
    }
}

/// 格式化浮点数为千分位格式
pub fn format_float_with_commas(number: f64, decimals: usize) -> String {
    let formatted = format!("{:.prec$}", number.abs(), prec = decimals);
    let parts: Vec<&str> = formatted.split('.').collect();

    let integer_part = format_number_with_commas(parts[0].parse().unwrap_or(0));

    let result = if parts.len() > 1 && decimals > 0 {
        format!("{}.{}", integer_part, parts[1])
    } else {
        integer_part
    };

    if number < 0.0 {
        format!("-{}", result.trim_start_matches('-'))
    } else {
        result
    }
}

/// 格式化百分比
pub fn format_percentage(value: f64, decimals: usize) -> String {
    format!("{:.prec$}%", value, prec = decimals)
}

/// 格式化文件大小（字节）
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// 格式化内存大小
pub fn format_memory(bytes: u64) -> String {
    format_bytes(bytes)
}

/// 格式化货币
pub fn format_currency(amount: f64, currency: &str) -> String {
    format!("{} {:.2}", currency, amount)
}

/// 格式化表格列
pub struct TableFormatter {
    columns: Vec<ColumnConfig>,
}

#[derive(Debug, Clone)]
pub struct ColumnConfig {
    pub title: String,
    pub width: usize,
    pub alignment: Alignment,
}

#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

impl TableFormatter {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
        }
    }

    pub fn add_column(&mut self, title: &str, width: usize, alignment: Alignment) {
        self.columns.push(ColumnConfig {
            title: title.to_string(),
            width,
            alignment,
        });
    }

    pub fn format_header(&self) -> String {
        let mut header = String::new();
        header.push('|');

        for column in &self.columns {
            let formatted = self.format_cell(&column.title, column.width, column.alignment);
            header.push_str(&formatted);
            header.push('|');
        }

        header
    }

    pub fn format_separator(&self) -> String {
        let mut separator = String::new();
        separator.push('|');

        for column in &self.columns {
            let line = match column.alignment {
                Alignment::Left => format!("{:-<width$}", "", width = column.width),
                Alignment::Center => format!("{:-^width$}", "", width = column.width),
                Alignment::Right => format!("{:->width$}", "", width = column.width),
            };
            separator.push_str(&line);
            separator.push('|');
        }

        separator
    }

    pub fn format_row(&self, values: &[&str]) -> String {
        let mut row = String::new();
        row.push('|');

        for (i, column) in self.columns.iter().enumerate() {
            let value = values.get(i).unwrap_or(&"");
            let formatted = self.format_cell(value, column.width, column.alignment);
            row.push_str(&formatted);
            row.push('|');
        }

        row
    }

    fn format_cell(&self, content: &str, width: usize, alignment: Alignment) -> String {
        let truncated = if content.len() > width {
            if width > 3 {
                format!("{}...", &content[..width - 3])
            } else {
                content[..width].to_string()
            }
        } else {
            content.to_string()
        };

        match alignment {
            Alignment::Left => format!(" {:<width$} ", truncated, width = width - 2),
            Alignment::Center => format!(" {:^width$} ", truncated, width = width - 2),
            Alignment::Right => format!(" {:>width$} ", truncated, width = width - 2),
        }
    }
}

impl Default for TableFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// 格式化进度条
pub fn format_progress_bar(
    current: usize,
    total: usize,
    width: usize,
    style: ProgressStyle,
) -> String {
    if total == 0 {
        return format!("[{}] 0/0", "█".repeat(width));
    }

    let progress = (current as f64 / total as f64).min(1.0);
    let filled = (progress * width as f64) as usize;
    let empty = width.saturating_sub(filled);

    let (fill_char, empty_char) = match style {
        ProgressStyle::Block => ('█', '░'),
        ProgressStyle::Bar => ('=', '-'),
        ProgressStyle::Dot => ('●', '○'),
        ProgressStyle::Arrow => ('>', ' '),
    };

    format!(
        "[{}{}] {}/{} ({:.1}%)",
        fill_char.to_string().repeat(filled),
        empty_char.to_string().repeat(empty),
        current,
        total,
        progress * 100.0
    )
}

#[derive(Debug, Clone, Copy)]
pub enum ProgressStyle {
    Block,
    Bar,
    Dot,
    Arrow,
}

/// 格式化列表
pub fn format_list(items: &[String], style: ListStyle) -> String {
    let mut result = String::new();

    for (i, item) in items.iter().enumerate() {
        let prefix = match style {
            ListStyle::Bullet => "• ",
            ListStyle::Numbered => &format!("{}. ", i + 1),
            ListStyle::Dash => "- ",
            ListStyle::Arrow => "→ ",
        };
        result.push_str(&format!("{}{}", prefix, item));
        if i < items.len() - 1 {
            result.push('\n');
        }
    }

    result
}

#[derive(Debug, Clone, Copy)]
pub enum ListStyle {
    Bullet,
    Numbered,
    Dash,
    Arrow,
}

/// 格式化键值对
pub fn format_key_value_pairs(pairs: &[(String, String)], separator: &str) -> String {
    pairs
        .iter()
        .map(|(key, value)| format!("{}{}{}", key, separator, value))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 格式化多行文本的缩进
pub fn indent_text(text: &str, indent: usize) -> String {
    let indent_str = " ".repeat(indent);
    text.lines()
        .map(|line| format!("{}{}", indent_str, line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 格式化文本居中
pub fn center_text(text: &str, width: usize) -> String {
    if text.len() >= width {
        text.to_string()
    } else {
        let padding = width - text.len();
        let left_padding = padding / 2;
        let right_padding = padding - left_padding;
        format!(
            "{}{}{}",
            " ".repeat(left_padding),
            text,
            " ".repeat(right_padding)
        )
    }
}

/// 格式化文本框
pub fn format_text_box(text: &str, width: usize, style: BoxStyle) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let max_content_width = width.saturating_sub(4); // 减去边框和内边距

    let (top_left, top_right, bottom_left, bottom_right, horizontal, vertical) = match style {
        BoxStyle::Single => ('┌', '┐', '└', '┘', '─', '│'),
        BoxStyle::Double => ('╔', '╗', '╚', '╝', '═', '║'),
        BoxStyle::Rounded => ('╭', '╮', '╰', '╯', '─', '│'),
        BoxStyle::Thick => ('┏', '┓', '┗', '┛', '━', '┃'),
    };

    let mut result = String::new();

    // 顶部边框
    result.push(top_left);
    result.push_str(&horizontal.to_string().repeat(width - 2));
    result.push(top_right);
    result.push('\n');

    // 内容行
    for line in lines {
        result.push(vertical);
        result.push(' ');

        if line.len() > max_content_width {
            result.push_str(&line[..max_content_width]);
        } else {
            result.push_str(line);
            result.push_str(&" ".repeat(max_content_width - line.len()));
        }

        result.push(' ');
        result.push(vertical);
        result.push('\n');
    }

    // 底部边框
    result.push(bottom_left);
    result.push_str(&horizontal.to_string().repeat(width - 2));
    result.push(bottom_right);

    result
}

#[derive(Debug, Clone, Copy)]
pub enum BoxStyle {
    Single,
    Double,
    Rounded,
    Thick,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_format_duration_detailed() {
        assert_eq!(format_duration_detailed(Duration::seconds(30)), "30秒");
        assert_eq!(format_duration_detailed(Duration::seconds(90)), "1分钟30秒");
        assert_eq!(
            format_duration_detailed(Duration::seconds(3661)),
            "1小时1分钟1秒"
        );
        assert_eq!(
            format_duration_detailed(Duration::seconds(90061)),
            "1天1小时1分钟1秒"
        );
    }

    #[test]
    fn test_format_duration_compact() {
        assert_eq!(format_duration_compact(Duration::seconds(30)), "00:30");
        assert_eq!(format_duration_compact(Duration::seconds(90)), "01:30");
        assert_eq!(format_duration_compact(Duration::seconds(3661)), "01:01:01");
    }

    #[test]
    fn test_format_number_with_commas() {
        assert_eq!(format_number_with_commas(1234), "1,234");
        assert_eq!(format_number_with_commas(1234567), "1,234,567");
        assert_eq!(format_number_with_commas(-1234), "-1,234");
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(format_percentage(25.5, 1), "25.5%");
        assert_eq!(format_percentage(33.333, 2), "33.33%");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }

    #[test]
    fn test_table_formatter() {
        let mut formatter = TableFormatter::new();
        formatter.add_column("Name", 10, Alignment::Left);
        formatter.add_column("Age", 5, Alignment::Right);

        let header = formatter.format_header();
        assert!(header.contains("Name"));
        assert!(header.contains("Age"));

        let row = formatter.format_row(&["Alice", "25"]);
        assert!(row.contains("Alice"));
        assert!(row.contains("25"));
    }

    #[test]
    fn test_format_progress_bar() {
        let bar = format_progress_bar(5, 10, 10, ProgressStyle::Block);
        assert!(bar.contains("5/10"));
        assert!(bar.contains("50.0%"));
    }

    #[test]
    fn test_center_text() {
        assert_eq!(center_text("test", 10), "   test   ");
        assert_eq!(center_text("hello", 10), "  hello   ");
    }
}
