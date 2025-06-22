//! # 数据验证工具模块
//!
//! 提供各种数据验证功能

use chrono::{DateTime, Duration, Local};
use regex::Regex;
use std::collections::HashSet;

/// 验证结果
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: &str) {
        self.is_valid = false;
        self.errors.push(error.to_string());
    }

    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
            self.errors.extend(other.errors);
        }
    }

    pub fn first_error(&self) -> Option<&String> {
        self.errors.first()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证器trait
pub trait Validator<T> {
    fn validate(&self, value: &T) -> ValidationResult;
}

/// 字符串验证器
pub struct StringValidator {
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<Regex>,
    required: bool,
    forbidden_chars: HashSet<char>,
    allowed_chars: Option<HashSet<char>>,
}

impl StringValidator {
    pub fn new() -> Self {
        Self {
            min_length: None,
            max_length: None,
            pattern: None,
            required: false,
            forbidden_chars: HashSet::new(),
            allowed_chars: None,
        }
    }

    pub fn min_length(mut self, min: usize) -> Self {
        self.min_length = Some(min);
        self
    }

    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    pub fn pattern(mut self, pattern: &str) -> Result<Self, regex::Error> {
        self.pattern = Some(Regex::new(pattern)?);
        Ok(self)
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn forbid_chars(mut self, chars: &[char]) -> Self {
        self.forbidden_chars.extend(chars);
        self
    }

    pub fn allow_only_chars(mut self, chars: &[char]) -> Self {
        self.allowed_chars = Some(chars.iter().cloned().collect());
        self
    }
}

impl Default for StringValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator<String> for StringValidator {
    fn validate(&self, value: &String) -> ValidationResult {
        let mut result = ValidationResult::new();

        // 检查是否为空且必需
        if self.required && value.trim().is_empty() {
            result.add_error("此字段为必填项");
            return result;
        }

        // 如果不是必需且为空，则跳过其他验证
        if !self.required && value.trim().is_empty() {
            return result;
        }

        // 检查最小长度
        if let Some(min) = self.min_length {
            if value.len() < min {
                result.add_error(&format!("长度不能少于{}个字符", min));
            }
        }

        // 检查最大长度
        if let Some(max) = self.max_length {
            if value.len() > max {
                result.add_error(&format!("长度不能超过{}个字符", max));
            }
        }

        // 检查正则表达式
        if let Some(ref pattern) = self.pattern {
            if !pattern.is_match(value) {
                result.add_error("格式不正确");
            }
        }

        // 检查禁用字符
        for ch in value.chars() {
            if self.forbidden_chars.contains(&ch) {
                result.add_error(&format!("不能包含字符: {}", ch));
                break;
            }
        }

        // 检查允许字符
        if let Some(ref allowed) = self.allowed_chars {
            for ch in value.chars() {
                if !allowed.contains(&ch) {
                    result.add_error(&format!("包含不允许的字符: {}", ch));
                    break;
                }
            }
        }

        result
    }
}

/// 数字验证器
pub struct NumberValidator<T> {
    min_value: Option<T>,
    max_value: Option<T>,
    required: bool,
}

impl<T> NumberValidator<T>
where
    T: PartialOrd + Copy,
{
    pub fn new() -> Self {
        Self {
            min_value: None,
            max_value: None,
            required: false,
        }
    }

    pub fn min_value(mut self, min: T) -> Self {
        self.min_value = Some(min);
        self
    }

    pub fn max_value(mut self, max: T) -> Self {
        self.max_value = Some(max);
        self
    }

    pub fn range(mut self, min: T, max: T) -> Self {
        self.min_value = Some(min);
        self.max_value = Some(max);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

impl<T> Default for NumberValidator<T>
where
    T: PartialOrd + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Validator<Option<T>> for NumberValidator<T>
where
    T: PartialOrd + Copy + std::fmt::Display,
{
    fn validate(&self, value: &Option<T>) -> ValidationResult {
        let mut result = ValidationResult::new();

        match value {
            None => {
                if self.required {
                    result.add_error("此字段为必填项");
                }
            }
            Some(val) => {
                if let Some(min) = self.min_value {
                    if *val < min {
                        result.add_error(&format!("值不能小于{}", min));
                    }
                }

                if let Some(max) = self.max_value {
                    if *val > max {
                        result.add_error(&format!("值不能大于{}", max));
                    }
                }
            }
        }

        result
    }
}

/// 日期时间验证器
pub struct DateTimeValidator {
    min_date: Option<DateTime<Local>>,
    max_date: Option<DateTime<Local>>,
    required: bool,
    allow_future: bool,
    allow_past: bool,
}

impl DateTimeValidator {
    pub fn new() -> Self {
        Self {
            min_date: None,
            max_date: None,
            required: false,
            allow_future: true,
            allow_past: true,
        }
    }

    pub fn min_date(mut self, min: DateTime<Local>) -> Self {
        self.min_date = Some(min);
        self
    }

    pub fn max_date(mut self, max: DateTime<Local>) -> Self {
        self.max_date = Some(max);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn no_future(mut self) -> Self {
        self.allow_future = false;
        self
    }

    pub fn no_past(mut self) -> Self {
        self.allow_past = false;
        self
    }

    pub fn today_only(mut self) -> Self {
        let now = Local::now();
        let today_start = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();
        let today_end = now
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();

        self.min_date = Some(today_start);
        self.max_date = Some(today_end);
        self
    }
}

impl Default for DateTimeValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator<Option<DateTime<Local>>> for DateTimeValidator {
    fn validate(&self, value: &Option<DateTime<Local>>) -> ValidationResult {
        let mut result = ValidationResult::new();

        match value {
            None => {
                if self.required {
                    result.add_error("此字段为必填项");
                }
            }
            Some(datetime) => {
                let now = Local::now();

                // 检查是否允许未来时间
                if !self.allow_future && *datetime > now {
                    result.add_error("不能选择未来时间");
                }

                // 检查是否允许过去时间
                if !self.allow_past && *datetime < now {
                    result.add_error("不能选择过去时间");
                }

                // 检查最小日期
                if let Some(min) = self.min_date {
                    if *datetime < min {
                        result.add_error(&format!("日期不能早于{}", min.format("%Y-%m-%d %H:%M")));
                    }
                }

                // 检查最大日期
                if let Some(max) = self.max_date {
                    if *datetime > max {
                        result.add_error(&format!("日期不能晚于{}", max.format("%Y-%m-%d %H:%M")));
                    }
                }
            }
        }

        result
    }
}

/// 持续时间验证器
pub struct DurationValidator {
    min_duration: Option<Duration>,
    max_duration: Option<Duration>,
    required: bool,
}

impl DurationValidator {
    pub fn new() -> Self {
        Self {
            min_duration: None,
            max_duration: None,
            required: false,
        }
    }

    pub fn min_duration(mut self, min: Duration) -> Self {
        self.min_duration = Some(min);
        self
    }

    pub fn max_duration(mut self, max: Duration) -> Self {
        self.max_duration = Some(max);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

impl Default for DurationValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator<Option<Duration>> for DurationValidator {
    fn validate(&self, value: &Option<Duration>) -> ValidationResult {
        let mut result = ValidationResult::new();

        match value {
            None => {
                if self.required {
                    result.add_error("此字段为必填项");
                }
            }
            Some(duration) => {
                if let Some(min) = self.min_duration {
                    if *duration < min {
                        result.add_error(&format!(
                            "持续时间不能少于{}",
                            crate::utils::format_duration(min)
                        ));
                    }
                }

                if let Some(max) = self.max_duration {
                    if *duration > max {
                        result.add_error(&format!(
                            "持续时间不能超过{}",
                            crate::utils::format_duration(max)
                        ));
                    }
                }
            }
        }

        result
    }
}

/// 邮箱验证器
pub struct EmailValidator {
    required: bool,
}

impl EmailValidator {
    pub fn new() -> Self {
        Self { required: false }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

impl Default for EmailValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator<String> for EmailValidator {
    fn validate(&self, value: &String) -> ValidationResult {
        let mut result = ValidationResult::new();

        if value.trim().is_empty() {
            if self.required {
                result.add_error("邮箱地址为必填项");
            }
            return result;
        }

        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

        if !email_regex.is_match(value) {
            result.add_error("邮箱地址格式不正确");
        }

        result
    }
}

/// URL验证器
pub struct UrlValidator {
    required: bool,
    allowed_schemes: HashSet<String>,
}

impl UrlValidator {
    pub fn new() -> Self {
        let mut allowed_schemes = HashSet::new();
        allowed_schemes.insert("http".to_string());
        allowed_schemes.insert("https".to_string());

        Self {
            required: false,
            allowed_schemes,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn allow_schemes(mut self, schemes: &[&str]) -> Self {
        self.allowed_schemes = schemes.iter().map(|s| s.to_string()).collect();
        self
    }
}

impl Default for UrlValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator<String> for UrlValidator {
    fn validate(&self, value: &String) -> ValidationResult {
        let mut result = ValidationResult::new();

        if value.trim().is_empty() {
            if self.required {
                result.add_error("URL为必填项");
            }
            return result;
        }

        // 简单的URL验证
        let url_regex = Regex::new(r"^[a-zA-Z][a-zA-Z0-9+.-]*://[^\s]+$").unwrap();

        if !url_regex.is_match(value) {
            result.add_error("URL格式不正确");
            return result;
        }

        // 检查协议
        if let Some(scheme_end) = value.find("://") {
            let scheme = &value[..scheme_end].to_lowercase();
            if !self.allowed_schemes.contains(scheme) {
                result.add_error(&format!("不支持的协议: {}", scheme));
            }
        }

        result
    }
}

/// 组合验证器
pub struct CompositeValidator<T> {
    validators: Vec<Box<dyn Validator<T>>>,
}

impl<T> CompositeValidator<T> {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    pub fn add_validator(mut self, validator: Box<dyn Validator<T>>) -> Self {
        self.validators.push(validator);
        self
    }
}

impl<T> Default for CompositeValidator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Validator<T> for CompositeValidator<T> {
    fn validate(&self, value: &T) -> ValidationResult {
        let mut result = ValidationResult::new();

        for validator in &self.validators {
            let validation_result = validator.validate(value);
            result.merge(validation_result);
        }

        result
    }
}

/// 便捷验证函数
pub fn validate_task_name(name: &str) -> ValidationResult {
    StringValidator::new()
        .required()
        .min_length(1)
        .max_length(100)
        .forbid_chars(&['<', '>', '"', '\\', '/', '|', '?', '*'])
        .validate(&name.to_string())
}

pub fn validate_category_name(name: &str) -> ValidationResult {
    StringValidator::new()
        .required()
        .min_length(1)
        .max_length(50)
        .forbid_chars(&['<', '>', '"', '\\', '/', '|', '?', '*'])
        .validate(&name.to_string())
}

pub fn validate_description(description: &str) -> ValidationResult {
    StringValidator::new()
        .max_length(500)
        .validate(&description.to_string())
}

pub fn validate_time_entry_duration(
    start: DateTime<Local>,
    end: Option<DateTime<Local>>,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    if let Some(end_time) = end {
        if end_time <= start {
            result.add_error("结束时间必须晚于开始时间");
        }

        let duration = end_time.signed_duration_since(start);
        if duration > Duration::hours(24) {
            result.add_error("单次记录时间不能超过24小时");
        }
    }

    // 检查开始时间不能是未来
    let now = Local::now();
    if start > now {
        result.add_error("开始时间不能是未来时间");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_string_validator() {
        let validator = StringValidator::new()
            .required()
            .min_length(3)
            .max_length(10);

        // 有效字符串
        let result = validator.validate(&"hello".to_string());
        assert!(result.is_valid);

        // 空字符串
        let result = validator.validate(&"".to_string());
        assert!(!result.is_valid);
        assert!(result.errors[0].contains("必填项"));

        // 太短
        let result = validator.validate(&"hi".to_string());
        assert!(!result.is_valid);
        assert!(result.errors[0].contains("不能少于"));

        // 太长
        let result = validator.validate(&"hello world!".to_string());
        assert!(!result.is_valid);
        assert!(result.errors[0].contains("不能超过"));
    }

    #[test]
    fn test_number_validator() {
        let validator = NumberValidator::new().required().range(1, 100);

        // 有效数字
        let result = validator.validate(&Some(50));
        assert!(result.is_valid);

        // 空值
        let result = validator.validate(&None);
        assert!(!result.is_valid);

        // 太小
        let result = validator.validate(&Some(0));
        assert!(!result.is_valid);

        // 太大
        let result = validator.validate(&Some(101));
        assert!(!result.is_valid);
    }

    #[test]
    fn test_email_validator() {
        let validator = EmailValidator::new().required();

        // 有效邮箱
        let result = validator.validate(&"test@example.com".to_string());
        assert!(result.is_valid);

        // 无效邮箱
        let result = validator.validate(&"invalid-email".to_string());
        assert!(!result.is_valid);

        // 空邮箱
        let result = validator.validate(&"".to_string());
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_task_name() {
        // 有效任务名
        let result = validate_task_name("学习Rust");
        assert!(result.is_valid);

        // 空任务名
        let result = validate_task_name("");
        assert!(!result.is_valid);

        // 包含非法字符
        let result = validate_task_name("任务<test>");
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_time_entry_duration() {
        let start = Local.with_ymd_and_hms(2023, 1, 1, 10, 0, 0).unwrap();
        let end = Local.with_ymd_and_hms(2023, 1, 1, 11, 0, 0).unwrap();

        // 有效时间范围
        let result = validate_time_entry_duration(start, Some(end));
        assert!(result.is_valid);

        // 结束时间早于开始时间
        let result = validate_time_entry_duration(end, Some(start));
        assert!(!result.is_valid);
    }
}
