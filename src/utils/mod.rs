//! # 工具模块
//!
//! 提供通用的工具函数和辅助功能

use crate::errors::Result;
use chrono::{DateTime, Duration, Local};
use std::collections::HashMap;
use uuid::Uuid;

pub mod date;
pub mod export;
pub mod format;
pub mod import;
pub mod validation;

/// 生成唯一ID
pub fn generate_id() -> Uuid {
    Uuid::new_v4()
}

/// 获取当前时间戳
pub fn current_timestamp() -> DateTime<Local> {
    Local::now()
}

/// 计算两个时间之间的持续时间
pub fn calculate_duration(start: DateTime<Local>, end: DateTime<Local>) -> Duration {
    end.signed_duration_since(start)
}

/// 格式化持续时间为人类可读格式
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// 解析持续时间字符串
pub fn parse_duration(duration_str: &str) -> Result<Duration> {
    let duration_str = duration_str.trim().to_lowercase();

    // 支持格式: "1h 30m", "90m", "5400s", "1.5h"
    if duration_str.contains('h') || duration_str.contains('m') || duration_str.contains('s') {
        parse_human_duration(&duration_str)
    } else {
        // 尝试解析为秒数
        let seconds: i64 = duration_str
            .parse()
            .map_err(|_| format!("无效的持续时间格式: {}", duration_str))?;
        Ok(Duration::seconds(seconds))
    }
}

/// 解析人类可读的持续时间格式
fn parse_human_duration(duration_str: &str) -> Result<Duration> {
    let mut total_seconds = 0i64;
    let mut current_number = String::new();

    for ch in duration_str.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            current_number.push(ch);
        } else if ch.is_alphabetic() {
            if !current_number.is_empty() {
                let value: f64 = current_number
                    .parse()
                    .map_err(|_| format!("无效的数字: {}", current_number))?;

                match ch {
                    'h' => total_seconds += (value * 3600.0) as i64,
                    'm' => total_seconds += (value * 60.0) as i64,
                    's' => total_seconds += value as i64,
                    _ => return Err(format!("不支持的时间单位: {}", ch).into()),
                }

                current_number.clear();
            }
        }
        // 忽略空格和其他字符
    }

    Ok(Duration::seconds(total_seconds))
}

/// 验证邮箱格式
pub fn validate_email(email: &str) -> bool {
    let email_regex =
        regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

/// 清理字符串（移除多余空格）
pub fn clean_string(s: &str) -> String {
    s.trim().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 截断字符串到指定长度
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// 计算百分比
pub fn calculate_percentage(part: f64, total: f64) -> f64 {
    if total == 0.0 {
        0.0
    } else {
        (part / total) * 100.0
    }
}

/// 格式化文件大小
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
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

/// 创建进度条字符串
pub fn create_progress_bar(current: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return "█".repeat(width);
    }

    let progress = (current as f64 / total as f64).min(1.0);
    let filled = (progress * width as f64) as usize;
    let empty = width.saturating_sub(filled);

    format!(
        "{}{}[{}/{}]",
        "█".repeat(filled),
        "░".repeat(empty),
        current,
        total
    )
}

/// 解析键值对字符串
pub fn parse_key_value_pairs(input: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            map.insert(
                key.trim().to_string(),
                value.trim().trim_matches('"').to_string(),
            );
        }
    }

    map
}

/// 安全地创建目录
pub fn ensure_dir_exists(path: &std::path::Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| format!("创建目录失败 {}: {}", path.display(), e))?;
    }
    Ok(())
}

/// 获取文件扩展名
pub fn get_file_extension(filename: &str) -> Option<&str> {
    std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
}

/// 生成随机字符串
pub fn generate_random_string(length: usize) -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// 应用标识符，与 tauri.conf.json 中的 identifier 保持一致
const APP_IDENTIFIER: &str = "com.lifetracker.app";

/// 获取应用数据目录
pub fn get_app_data_dir() -> Result<std::path::PathBuf> {
    // 使用与前端一致的目录标识符
    if let Some(data_dir) = dirs::data_dir() {
        return Ok(data_dir.join(APP_IDENTIFIER));
    }

    // Fallback：手动构建路径
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").map_err(|_| "无法获取APPDATA环境变量")?;
        return Ok(std::path::PathBuf::from(appdata).join(APP_IDENTIFIER));
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").map_err(|_| "无法获取HOME环境变量")?;
        return Ok(std::path::PathBuf::from(home)
            .join(".local")
            .join("share")
            .join(APP_IDENTIFIER));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2);
        assert_eq!(id1.to_string().len(), 36); // UUID v4 长度
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::seconds(30)), "30s");
        assert_eq!(format_duration(Duration::seconds(90)), "1m 30s");
        assert_eq!(format_duration(Duration::seconds(3661)), "1h 1m 1s");
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::seconds(30));
        assert_eq!(parse_duration("1m 30s").unwrap(), Duration::seconds(90));
        assert_eq!(parse_duration("1h").unwrap(), Duration::seconds(3600));
        assert_eq!(parse_duration("3600").unwrap(), Duration::seconds(3600));
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com"));
        assert!(validate_email("user.name+tag@domain.co.uk"));
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email("@domain.com"));
    }

    #[test]
    fn test_clean_string() {
        assert_eq!(clean_string("  hello   world  "), "hello world");
        assert_eq!(clean_string("\t\ntest\r\n"), "test");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
    }

    #[test]
    fn test_calculate_percentage() {
        assert_eq!(calculate_percentage(25.0, 100.0), 25.0);
        assert_eq!(calculate_percentage(1.0, 3.0), 33.333333333333336);
        assert_eq!(calculate_percentage(10.0, 0.0), 0.0);
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
    }

    #[test]
    fn test_create_progress_bar() {
        assert_eq!(create_progress_bar(5, 10, 10), "█████░░░░░[5/10]");
        assert_eq!(create_progress_bar(0, 10, 5), "░░░░░[0/10]");
        assert_eq!(create_progress_bar(10, 10, 5), "█████[10/10]");
    }

    #[test]
    fn test_parse_key_value_pairs() {
        let input = r#"
            # Comment
            key1=value1
            key2="quoted value"
            key3 = value with spaces
        "#;

        let map = parse_key_value_pairs(input);
        assert_eq!(map.get("key1"), Some(&"value1".to_string()));
        assert_eq!(map.get("key2"), Some(&"quoted value".to_string()));
        assert_eq!(map.get("key3"), Some(&"value with spaces".to_string()));
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("test.txt"), Some("txt"));
        assert_eq!(get_file_extension("archive.tar.gz"), Some("gz"));
        assert_eq!(get_file_extension("no_extension"), None);
    }

    #[test]
    fn test_generate_random_string() {
        let s1 = generate_random_string(10);
        let s2 = generate_random_string(10);
        assert_eq!(s1.len(), 10);
        assert_eq!(s2.len(), 10);
        assert_ne!(s1, s2); // 极小概率相同
    }
}
