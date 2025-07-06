//! # 错误处理模块
//!
//! 定义应用程序的错误类型和错误处理功能

use chrono::ParseError as ChronoParseError;
use csv::Error as CsvError;
use rusqlite::Error as SqliteError;
use serde_json::Error as JsonError;
use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::num::{ParseFloatError, ParseIntError};

/// 配置错误类型
#[derive(Debug)]
pub struct ConfigError {
    pub message: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "配置错误: {}", self.message)
    }
}

impl StdError for ConfigError {}

/// 应用程序错误类型
#[derive(Debug)]
pub enum AppError {
    /// IO错误
    Io(io::Error),
    /// 数据库错误
    Database(SqliteError),
    /// JSON序列化/反序列化错误
    Json(JsonError),
    /// CSV处理错误
    Csv(CsvError),
    /// 配置错误
    Config(ConfigError),
    /// 日期时间解析错误
    DateTimeParse(ChronoParseError),
    /// 数字解析错误
    ParseInt(ParseIntError),
    /// 浮点数解析错误
    ParseFloat(ParseFloatError),
    /// 验证错误
    Validation(String),
    /// 业务逻辑错误
    Business(String),
    /// 网络错误
    Network(String),
    /// 权限错误
    Permission(String),
    /// 资源未找到错误
    NotFound(String),
    /// 任务未找到错误
    TaskNotFound(String),
    /// 分类未找到错误
    CategoryNotFound(String),
    /// 输入无效错误
    InvalidInput(String),
    /// 资源已存在错误
    AlreadyExists(String),
    /// 操作超时错误
    Timeout(String),
    /// 系统错误
    System(String),
    /// 未知错误
    Unknown(String),
    /// GUI错误
    GuiError(String),
    /// 计时器状态错误
    TimerState(String),
    /// 存储层错误
    Storage(String),
    /// 加密错误
    Crypto(String),
    /// 同步错误
    Sync(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "IO错误: {}", err),
            AppError::Database(err) => write!(f, "数据库错误: {}", err),
            AppError::Json(err) => write!(f, "JSON错误: {}", err),
            AppError::Csv(err) => write!(f, "CSV错误: {}", err),
            AppError::Config(err) => write!(f, "配置错误: {}", err),
            AppError::DateTimeParse(err) => write!(f, "日期时间解析错误: {}", err),
            AppError::ParseInt(err) => write!(f, "整数解析错误: {}", err),
            AppError::ParseFloat(err) => write!(f, "浮点数解析错误: {}", err),
            AppError::Validation(msg) => write!(f, "验证错误: {}", msg),
            AppError::Business(msg) => write!(f, "业务错误: {}", msg),
            AppError::Network(msg) => write!(f, "网络错误: {}", msg),
            AppError::Permission(msg) => write!(f, "权限错误: {}", msg),
            AppError::NotFound(msg) => write!(f, "未找到: {}", msg),
            AppError::TaskNotFound(msg) => write!(f, "任务未找到: {}", msg),
            AppError::CategoryNotFound(msg) => write!(f, "分类未找到: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "输入无效: {}", msg),
            AppError::AlreadyExists(msg) => write!(f, "已存在: {}", msg),
            AppError::Timeout(msg) => write!(f, "超时: {}", msg),
            AppError::System(msg) => write!(f, "系统错误: {}", msg),
            AppError::Unknown(msg) => write!(f, "未知错误: {}", msg),
            AppError::GuiError(msg) => write!(f, "GUI错误: {}", msg),
            AppError::TimerState(msg) => write!(f, "计时器状态错误: {}", msg),
            AppError::Storage(msg) => write!(f, "存储层错误: {}", msg),
            AppError::Crypto(msg) => write!(f, "加密错误: {}", msg),
            AppError::Sync(msg) => write!(f, "同步错误: {}", msg),
        }
    }
}

impl StdError for AppError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            AppError::Database(err) => Some(err),
            AppError::Json(err) => Some(err),
            AppError::Csv(err) => Some(err),
            AppError::Config(err) => Some(err),
            AppError::DateTimeParse(err) => Some(err),
            AppError::ParseInt(err) => Some(err),
            AppError::ParseFloat(err) => Some(err),
            _ => None,
        }
    }
}

// 错误转换实现
impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<SqliteError> for AppError {
    fn from(err: SqliteError) -> Self {
        AppError::Database(err)
    }
}

impl From<JsonError> for AppError {
    fn from(err: JsonError) -> Self {
        AppError::Json(err)
    }
}

impl From<CsvError> for AppError {
    fn from(err: CsvError) -> Self {
        AppError::Csv(err)
    }
}

impl From<ConfigError> for AppError {
    fn from(err: ConfigError) -> Self {
        AppError::Config(err)
    }
}

impl From<ChronoParseError> for AppError {
    fn from(err: ChronoParseError) -> Self {
        AppError::DateTimeParse(err)
    }
}

impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self {
        AppError::ParseInt(err)
    }
}

impl From<ParseFloatError> for AppError {
    fn from(err: ParseFloatError) -> Self {
        AppError::ParseFloat(err)
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Unknown(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Unknown(msg.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::System(err.to_string())
    }
}

/// 应用程序结果类型
pub type Result<T> = std::result::Result<T, AppError>;

/// 错误上下文trait
pub trait ErrorContext<T> {
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;

    fn context(self, msg: &str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<AppError>,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let original_error = e.into();
            let context = f();
            AppError::Unknown(format!("{}: {}", context, original_error))
        })
    }

    fn context(self, msg: &str) -> Result<T> {
        self.with_context(|| msg.to_string())
    }
}

/// 错误处理器
pub struct ErrorHandler {
    log_errors: bool,
    show_stack_trace: bool,
}

impl ErrorHandler {
    /// 创建新的错误处理器
    pub fn new() -> Self {
        Self {
            log_errors: true,
            show_stack_trace: false,
        }
    }

    /// 设置是否记录错误日志
    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.log_errors = enabled;
        self
    }

    /// 设置是否显示堆栈跟踪
    pub fn with_stack_trace(mut self, enabled: bool) -> Self {
        self.show_stack_trace = enabled;
        self
    }

    /// 处理错误
    pub fn handle_error(&self, error: &AppError) {
        if self.log_errors {
            log::error!("{}", error);
            if self.show_stack_trace {
                log::error!("Stack trace: {:?}", error);
            }
        }
    }

    /// 获取用户友好的错误消息
    pub fn user_friendly_message(&self, error: &AppError) -> String {
        match error {
            AppError::Io(_) => "文件操作失败，请检查文件权限或磁盘空间".to_string(),
            AppError::Database(_) => "数据库操作失败，请重试或联系技术支持".to_string(),
            AppError::Json(_) => "数据格式错误，请检查输入内容".to_string(),
            AppError::Config(_) => "配置文件错误，请检查配置设置".to_string(),
            AppError::Network(_) => "网络连接失败，请检查网络设置".to_string(),
            AppError::Permission(_) => "权限不足，请联系管理员".to_string(),
            AppError::TaskNotFound(_) => "指定的任务不存在".to_string(),
            AppError::CategoryNotFound(_) => "指定的分类不存在".to_string(),
            AppError::InvalidInput(_) => "输入内容无效，请重新输入".to_string(),
            AppError::TimerState(_) => "计时器状态错误，请重新开始计时".to_string(),
            AppError::GuiError(_) => "界面显示错误，请重启应用程序".to_string(),
            AppError::Crypto(_) => "加密操作失败，请检查配置或重试".to_string(),
            AppError::Sync(_) => "数据同步失败，请检查网络连接和同步设置".to_string(),
            _ => format!("操作失败: {}", error),
        }
    }

    /// 获取错误严重程度
    pub fn error_severity(&self, error: &AppError) -> ErrorSeverity {
        match error {
            AppError::Io(_) | AppError::Database(_) => ErrorSeverity::Critical,
            AppError::Network(_) | AppError::Permission(_) => ErrorSeverity::Error,
            AppError::Validation(_) | AppError::InvalidInput(_) => ErrorSeverity::Warning,
            AppError::TaskNotFound(_) | AppError::CategoryNotFound(_) => ErrorSeverity::Info,
            AppError::TimerState(_) => ErrorSeverity::Warning,
            AppError::GuiError(_) => ErrorSeverity::Error,
            AppError::Crypto(_) => ErrorSeverity::Error,
            AppError::Sync(_) => ErrorSeverity::Warning,
            _ => ErrorSeverity::Error,
        }
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// 错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
    Fatal,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "信息"),
            ErrorSeverity::Warning => write!(f, "警告"),
            ErrorSeverity::Error => write!(f, "错误"),
            ErrorSeverity::Critical => write!(f, "严重"),
            ErrorSeverity::Fatal => write!(f, "致命"),
        }
    }
}

/// 错误恢复策略
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// 重试操作
    Retry { max_attempts: u32, delay_ms: u64 },
    /// 使用默认值
    UseDefault,
    /// 跳过操作
    Skip,
    /// 终止程序
    Abort,
    /// 自定义恢复函数
    Custom(fn() -> Result<()>),
}

/// 错误恢复器
pub struct ErrorRecovery {
    strategies: std::collections::HashMap<String, RecoveryStrategy>,
}

impl ErrorRecovery {
    /// 创建新的错误恢复器
    pub fn new() -> Self {
        Self {
            strategies: std::collections::HashMap::new(),
        }
    }

    /// 注册恢复策略
    pub fn register_strategy(&mut self, error_type: &str, strategy: RecoveryStrategy) {
        self.strategies.insert(error_type.to_string(), strategy);
    }

    /// 尝试恢复错误
    pub fn try_recover(&self, error: &AppError) -> Option<RecoveryStrategy> {
        let error_type = match error {
            AppError::Io(_) => "io",
            AppError::Database(_) => "database",
            AppError::Network(_) => "network",
            AppError::Timeout(_) => "timeout",
            AppError::Crypto(_) => "crypto",
            AppError::Sync(_) => "sync",
            _ => "unknown",
        };

        self.strategies.get(error_type).cloned()
    }
}

impl Default for ErrorRecovery {
    fn default() -> Self {
        let mut recovery = Self::new();

        // 注册默认恢复策略
        recovery.register_strategy(
            "network",
            RecoveryStrategy::Retry {
                max_attempts: 3,
                delay_ms: 1000,
            },
        );
        recovery.register_strategy(
            "timeout",
            RecoveryStrategy::Retry {
                max_attempts: 2,
                delay_ms: 2000,
            },
        );
        recovery.register_strategy(
            "database",
            RecoveryStrategy::Retry {
                max_attempts: 2,
                delay_ms: 500,
            },
        );
        recovery.register_strategy("crypto", RecoveryStrategy::Skip);
        recovery.register_strategy(
            "sync",
            RecoveryStrategy::Retry {
                max_attempts: 3,
                delay_ms: 2000,
            },
        );

        recovery
    }
}

/// 便捷宏：创建验证错误
#[macro_export]
macro_rules! validation_error {
    ($msg:expr) => {
        $crate::errors::AppError::Validation($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::errors::AppError::Validation(format!($fmt, $($arg)*))
    };
}

/// 便捷宏：创建业务错误
#[macro_export]
macro_rules! business_error {
    ($msg:expr) => {
        $crate::errors::AppError::Business($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::errors::AppError::Business(format!($fmt, $($arg)*))
    };
}

/// 便捷宏：创建未找到错误
#[macro_export]
macro_rules! not_found_error {
    ($msg:expr) => {
        $crate::errors::AppError::NotFound($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::errors::AppError::NotFound(format!($fmt, $($arg)*))
    };
}

/// 便捷宏：创建已存在错误
#[macro_export]
macro_rules! already_exists_error {
    ($msg:expr) => {
        $crate::errors::AppError::AlreadyExists($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::errors::AppError::AlreadyExists(format!($fmt, $($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display() {
        let error = AppError::Validation("测试验证错误".to_string());
        assert_eq!(error.to_string(), "验证错误: 测试验证错误");

        let io_error = AppError::Io(io::Error::new(io::ErrorKind::NotFound, "文件未找到"));
        assert!(io_error.to_string().contains("IO错误"));
    }

    #[test]
    fn test_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "权限被拒绝");
        let app_error: AppError = io_error.into();

        match app_error {
            AppError::Io(_) => {}
            _ => panic!("错误转换失败"),
        }
    }

    #[test]
    fn test_error_context() {
        let result: std::result::Result<(), io::Error> =
            Err(io::Error::new(io::ErrorKind::NotFound, "文件未找到"));
        let with_context = result.context("读取配置文件时");

        assert!(with_context.is_err());
        assert!(with_context
            .unwrap_err()
            .to_string()
            .contains("读取配置文件时"));
    }

    #[test]
    fn test_error_handler() {
        let handler = ErrorHandler::new()
            .with_logging(true)
            .with_stack_trace(true);
        let error = AppError::Validation("测试错误".to_string());

        let message = handler.user_friendly_message(&error);
        assert!(message.contains("输入验证失败"));

        let severity = handler.error_severity(&error);
        assert_eq!(severity, ErrorSeverity::Warning);
    }

    #[test]
    fn test_error_recovery() {
        let mut recovery = ErrorRecovery::new();
        recovery.register_strategy("test", RecoveryStrategy::UseDefault);

        let strategy = recovery.strategies.get("test").unwrap();
        match strategy {
            RecoveryStrategy::UseDefault => {}
            _ => panic!("恢复策略不匹配"),
        }
    }

    #[test]
    fn test_error_macros() {
        let validation_err = validation_error!("测试验证错误");
        match validation_err {
            AppError::Validation(msg) => assert_eq!(msg, "测试验证错误"),
            _ => panic!("宏生成的错误类型不正确"),
        }

        let business_err = business_error!("业务错误: {}", "测试");
        match business_err {
            AppError::Business(msg) => assert_eq!(msg, "业务错误: 测试"),
            _ => panic!("宏生成的错误类型不正确"),
        }
    }

    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Info < ErrorSeverity::Warning);
        assert!(ErrorSeverity::Warning < ErrorSeverity::Error);
        assert!(ErrorSeverity::Error < ErrorSeverity::Critical);
        assert!(ErrorSeverity::Critical < ErrorSeverity::Fatal);
    }
}
