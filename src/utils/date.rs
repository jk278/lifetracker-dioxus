//! # 日期时间工具模块
//!
//! 提供日期时间相关的工具函数

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, Weekday};
use std::collections::HashMap;

/// 日期范围
#[derive(Debug, Clone, PartialEq)]
pub struct DateRange {
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

impl DateRange {
    /// 创建新的日期范围
    pub fn new(start: DateTime<Local>, end: DateTime<Local>) -> Self {
        Self { start, end }
    }

    /// 检查日期是否在范围内
    pub fn contains(&self, date: DateTime<Local>) -> bool {
        date >= self.start && date <= self.end
    }

    /// 获取范围内的天数
    pub fn days(&self) -> i64 {
        self.end.signed_duration_since(self.start).num_days() + 1
    }

    /// 获取范围内的小时数
    pub fn hours(&self) -> i64 {
        self.end.signed_duration_since(self.start).num_hours()
    }
}

/// 获取今天的日期范围（00:00:00 到 23:59:59）
pub fn today_range() -> DateRange {
    let now = Local::now();
    let start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();
    let end = now
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();
    DateRange::new(start, end)
}

/// 获取昨天的日期范围
pub fn yesterday_range() -> DateRange {
    let yesterday = Local::now() - Duration::days(1);
    let start = yesterday
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();
    let end = yesterday
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();
    DateRange::new(start, end)
}

/// 获取本周的日期范围（周一到周日）
pub fn this_week_range() -> DateRange {
    let now = Local::now();
    let weekday = now.weekday();
    let days_from_monday = weekday.num_days_from_monday() as i64;

    let monday = now - Duration::days(days_from_monday);
    let start = monday
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();

    let sunday = monday + Duration::days(6);
    let end = sunday
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();

    DateRange::new(start, end)
}

/// 获取上周的日期范围
pub fn last_week_range() -> DateRange {
    let this_week = this_week_range();
    let start = this_week.start - Duration::days(7);
    let end = this_week.end - Duration::days(7);
    DateRange::new(start, end)
}

/// 获取本月的日期范围
pub fn this_month_range() -> DateRange {
    let now = Local::now();
    let start = now
        .date_naive()
        .with_day(1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();

    let next_month = if now.month() == 12 {
        now.with_year(now.year() + 1)
            .unwrap()
            .with_month(1)
            .unwrap()
    } else {
        now.with_month(now.month() + 1).unwrap()
    };

    let end = (next_month - Duration::days(1))
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();

    DateRange::new(start, end)
}

/// 获取上月的日期范围
pub fn last_month_range() -> DateRange {
    let now = Local::now();
    let last_month = if now.month() == 1 {
        now.with_year(now.year() - 1)
            .unwrap()
            .with_month(12)
            .unwrap()
    } else {
        now.with_month(now.month() - 1).unwrap()
    };

    let start = last_month
        .date_naive()
        .with_day(1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();

    let end = now
        .date_naive()
        .with_day(1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        - Duration::seconds(1);

    DateRange::new(start, end)
}

/// 获取最近N天的日期范围
pub fn last_n_days_range(n: i64) -> DateRange {
    let now = Local::now();
    let start = (now - Duration::days(n - 1))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();
    let end = now
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();
    DateRange::new(start, end)
}

/// 获取最近N周的日期范围
pub fn last_n_weeks_range(n: i64) -> DateRange {
    let this_week = this_week_range();
    let start = this_week.start - Duration::weeks(n - 1);
    DateRange::new(start, this_week.end)
}

/// 获取最近N个月的日期范围
pub fn last_n_months_range(n: i32) -> DateRange {
    let now = Local::now();
    let start_month = if now.month() as i32 - n < 0 {
        let year_diff = (n - now.month() as i32 - 1) / 12 + 1;
        let month = 12 - ((n - now.month() as i32 - 1) % 12);
        now.with_year(now.year() - year_diff)
            .unwrap()
            .with_month(month as u32)
            .unwrap()
    } else {
        now.with_month((now.month() as i32 - n + 1) as u32).unwrap()
    };

    let start = start_month
        .date_naive()
        .with_day(1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();

    let end = now
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();

    DateRange::new(start, end)
}

/// 解析日期字符串
pub fn parse_date(date_str: &str) -> Result<NaiveDate, chrono::ParseError> {
    // 支持多种日期格式
    let formats = [
        "%Y-%m-%d", "%Y/%m/%d", "%d/%m/%Y", "%d-%m-%Y", "%Y.%m.%d", "%d.%m.%Y",
    ];

    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(date);
        }
    }

    // 如果都失败了，返回第一个格式的错误
    NaiveDate::parse_from_str(date_str, formats[0])
}

/// 解析日期时间字符串
pub fn parse_datetime(datetime_str: &str) -> Result<NaiveDateTime, chrono::ParseError> {
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d %H:%M",
        "%d/%m/%Y %H:%M:%S",
        "%d/%m/%Y %H:%M",
    ];

    for format in &formats {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(datetime_str, format) {
            return Ok(datetime);
        }
    }

    NaiveDateTime::parse_from_str(datetime_str, formats[0])
}

/// 格式化日期为字符串
pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// 格式化日期时间为字符串
pub fn format_datetime(datetime: DateTime<Local>) -> String {
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 格式化日期时间为简短格式
pub fn format_datetime_short(datetime: DateTime<Local>) -> String {
    let now = Local::now();
    let date = datetime.date_naive();
    let today = now.date_naive();

    if date == today {
        datetime.format("%H:%M").to_string()
    } else if date == today - Duration::days(1) {
        format!("昨天 {}", datetime.format("%H:%M"))
    } else if date.year() == today.year() {
        datetime.format("%m-%d %H:%M").to_string()
    } else {
        datetime.format("%Y-%m-%d %H:%M").to_string()
    }
}

/// 获取相对时间描述
pub fn relative_time(datetime: DateTime<Local>) -> String {
    let now = Local::now();
    let duration = now.signed_duration_since(datetime);

    if duration.num_seconds() < 60 {
        "刚刚".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}分钟前", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}小时前", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}天前", duration.num_days())
    } else if duration.num_weeks() < 4 {
        format!("{}周前", duration.num_weeks())
    } else {
        format_date(datetime.date_naive())
    }
}

/// 获取工作日列表（周一到周五）
pub fn get_weekdays() -> Vec<Weekday> {
    vec![
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
    ]
}

/// 检查是否为工作日
pub fn is_weekday(date: NaiveDate) -> bool {
    let weekday = date.weekday();
    matches!(
        weekday,
        Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri
    )
}

/// 检查是否为周末
pub fn is_weekend(date: NaiveDate) -> bool {
    !is_weekday(date)
}

/// 获取月份名称
pub fn get_month_name(month: u32) -> &'static str {
    match month {
        1 => "一月",
        2 => "二月",
        3 => "三月",
        4 => "四月",
        5 => "五月",
        6 => "六月",
        7 => "七月",
        8 => "八月",
        9 => "九月",
        10 => "十月",
        11 => "十一月",
        12 => "十二月",
        _ => "未知",
    }
}

/// 获取星期名称
pub fn get_weekday_name(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "周一",
        Weekday::Tue => "周二",
        Weekday::Wed => "周三",
        Weekday::Thu => "周四",
        Weekday::Fri => "周五",
        Weekday::Sat => "周六",
        Weekday::Sun => "周日",
    }
}

/// 计算两个日期之间的工作日数量
pub fn count_weekdays_between(start: NaiveDate, end: NaiveDate) -> i64 {
    let mut count = 0;
    let mut current = start;

    while current <= end {
        if is_weekday(current) {
            count += 1;
        }
        current = current.succ_opt().unwrap_or(current);
    }

    count
}

/// 获取日期范围内的所有日期
pub fn get_dates_in_range(range: &DateRange) -> Vec<NaiveDate> {
    let mut dates = Vec::new();
    let mut current = range.start.date_naive();
    let end_date = range.end.date_naive();

    while current <= end_date {
        dates.push(current);
        current = current.succ_opt().unwrap_or(current);
    }

    dates
}

/// 按周分组日期
pub fn group_dates_by_week(dates: &[NaiveDate]) -> HashMap<String, Vec<NaiveDate>> {
    let mut groups = HashMap::new();

    for &date in dates {
        let monday = date - Duration::days(date.weekday().num_days_from_monday() as i64);
        let week_key = format!(
            "{} (第{}周)",
            monday.format("%Y-%m-%d"),
            monday.iso_week().week()
        );
        groups.entry(week_key).or_insert_with(Vec::new).push(date);
    }

    groups
}

/// 按月分组日期
pub fn group_dates_by_month(dates: &[NaiveDate]) -> HashMap<String, Vec<NaiveDate>> {
    let mut groups = HashMap::new();

    for &date in dates {
        let month_key = format!("{}-{:02}", date.year(), date.month());
        groups.entry(month_key).or_insert_with(Vec::new).push(date);
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_date_range() {
        let start = Local.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        let end = Local.with_ymd_and_hms(2023, 1, 3, 23, 59, 59).unwrap();
        let range = DateRange::new(start, end);

        assert_eq!(range.days(), 3);
        assert!(range.contains(Local.with_ymd_and_hms(2023, 1, 2, 12, 0, 0).unwrap()));
        assert!(!range.contains(Local.with_ymd_and_hms(2023, 1, 4, 0, 0, 0).unwrap()));
    }

    #[test]
    fn test_parse_date() {
        assert!(parse_date("2023-01-01").is_ok());
        assert!(parse_date("2023/01/01").is_ok());
        assert!(parse_date("01/01/2023").is_ok());
        assert!(parse_date("invalid").is_err());
    }

    #[test]
    fn test_format_date() {
        let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        assert_eq!(format_date(date), "2023-01-01");
    }

    #[test]
    fn test_is_weekday() {
        let monday = NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(); // 2023-01-02 是周一
        let saturday = NaiveDate::from_ymd_opt(2023, 1, 7).unwrap(); // 2023-01-07 是周六

        assert!(is_weekday(monday));
        assert!(!is_weekday(saturday));
        assert!(is_weekend(saturday));
        assert!(!is_weekend(monday));
    }

    #[test]
    fn test_get_month_name() {
        assert_eq!(get_month_name(1), "一月");
        assert_eq!(get_month_name(12), "十二月");
        assert_eq!(get_month_name(13), "未知");
    }

    #[test]
    fn test_get_weekday_name() {
        assert_eq!(get_weekday_name(Weekday::Mon), "周一");
        assert_eq!(get_weekday_name(Weekday::Sun), "周日");
    }
}
