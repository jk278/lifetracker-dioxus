//! # TimeTracker - 个人时间追踪器
//!
//! 这是一个用Rust和Tauri开发的现代化时间追踪应用，提供Web界面。
//! 主要功能包括任务计时、数据统计、分类管理等。

// 公共模块声明
pub mod config; // 配置管理
pub mod core; // 核心业务逻辑
pub mod errors; // 错误处理
                // #[cfg(feature = "gui")]
                // pub mod gui; // 图形界面模块（已弃用，使用 Tauri 替代）
pub mod storage; // 数据存储层
pub mod utils; // 工具函数

// 重新导出核心类型，方便外部使用
pub use config::{AppConfig, ConfigManager};
pub use core::{Analytics, AppCore, Category, CategoryColor, CategoryIcon, Task, Timer};
pub use errors::{AppError, ErrorHandler, ErrorSeverity, Result};
pub use storage::{models::CategoryModel, Database, DatabaseConfig, StorageManager, TimeEntry};

// 应用程序信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// 应用程序信息结构
#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: String,
    pub homepage: String,
    pub repository: String,
    pub license: String,
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            name: NAME.to_string(),
            version: VERSION.to_string(),
            description: DESCRIPTION.to_string(),
            authors: AUTHORS.to_string(),
            homepage: "https://github.com/username/time-tracker".to_string(),
            repository: "https://github.com/username/time-tracker".to_string(),
            license: "MIT".to_string(),
        }
    }
}

/// 应用程序构建器
pub struct AppBuilder {
    config_path: Option<std::path::PathBuf>,
    database_path: Option<std::path::PathBuf>,
    log_level: Option<log::LevelFilter>,
}

impl AppBuilder {
    /// 创建新的应用程序构建器
    pub fn new() -> Self {
        Self {
            config_path: None,
            database_path: None,
            log_level: None,
        }
    }

    /// 设置配置文件路径
    pub fn with_config_path<P: Into<std::path::PathBuf>>(mut self, path: P) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// 设置数据库路径
    pub fn with_database_path<P: Into<std::path::PathBuf>>(mut self, path: P) -> Self {
        self.database_path = Some(path.into());
        self
    }

    /// 设置日志级别
    pub fn with_log_level(mut self, level: log::LevelFilter) -> Self {
        self.log_level = Some(level);
        self
    }

    /// 构建应用程序实例
    pub fn build(self) -> Result<App> {
        // 初始化日志（如果尚未初始化）
        if let Some(level) = self.log_level {
            if env_logger::Builder::from_default_env()
                .filter_level(level)
                .try_init()
                .is_err()
            {
                // 日志已经初始化，继续执行
                log::debug!("日志系统已经初始化");
            }
        }

        // 获取应用程序目录
        let app_dir = get_app_directory()?;

        // 设置配置路径
        let config_path = self
            .config_path
            .unwrap_or_else(|| app_dir.join("config.toml"));

        // 设置数据库路径
        let database_path = self
            .database_path
            .unwrap_or_else(|| app_dir.join("timetracker.db"));

        // 创建配置管理器
        let config_manager = ConfigManager::new(config_path)?;

        // 创建数据库连接
        let database = Database::new(&database_path)?;

        // 创建错误处理器
        let error_handler = ErrorHandler::new()
            .with_logging(true)
            .with_stack_trace(cfg!(debug_assertions));

        Ok(App {
            info: AppInfo::default(),
            config_manager,
            database,
            error_handler,
        })
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 主应用程序结构
pub struct App {
    pub info: AppInfo,
    pub config_manager: ConfigManager,
    pub database: Database,
    pub error_handler: ErrorHandler,
}

impl App {
    /// 创建应用程序构建器
    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    /// 获取应用程序信息
    pub fn info(&self) -> &AppInfo {
        &self.info
    }

    /// 获取配置管理器
    pub fn config(&self) -> &ConfigManager {
        &self.config_manager
    }

    /// 获取数据库连接
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// 获取错误处理器
    pub fn error_handler(&self) -> &ErrorHandler {
        &self.error_handler
    }

    /// 清理应用程序资源
    pub fn cleanup(&mut self) -> Result<()> {
        log::info!("清理应用程序资源");

        // 保存配置
        self.config_manager.save()?;

        log::info!("应用程序清理完成");
        Ok(())
    }
}

/// 获取应用程序目录
pub fn get_app_directory() -> Result<std::path::PathBuf> {
    let app_dir = if cfg!(target_os = "windows") {
        dirs::config_dir()
            .ok_or_else(|| AppError::System("无法获取配置目录".to_string()))?
            .join("TimeTracker")
    } else {
        dirs::home_dir()
            .ok_or_else(|| AppError::System("无法获取用户目录".to_string()))?
            .join(".timetracker")
    };

    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).map_err(|e| AppError::Io(e))?;
    }

    Ok(app_dir)
}

/// 初始化应用程序
pub fn init() -> Result<App> {
    App::builder()
        .with_log_level(log::LevelFilter::Info)
        .build()
}

/// 初始化应用程序（带自定义配置）
pub fn init_with_config(config_path: &std::path::Path) -> Result<App> {
    App::builder()
        .with_config_path(config_path)
        .with_log_level(log::LevelFilter::Info)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(!NAME.is_empty());
        assert!(!DESCRIPTION.is_empty());
    }

    #[test]
    fn test_app_info() {
        let info = AppInfo::default();
        assert_eq!(info.name, NAME);
        assert_eq!(info.version, VERSION);
        assert_eq!(info.description, DESCRIPTION);
    }

    #[test]
    fn test_app_builder() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        let db_path = temp_dir.path().join("test.db");

        let app = App::builder()
            .with_config_path(&config_path)
            .with_database_path(&db_path)
            .with_log_level(log::LevelFilter::Debug)
            .build();

        assert!(app.is_ok());
    }

    #[test]
    fn test_get_app_directory() {
        let app_dir = get_app_directory();
        assert!(app_dir.is_ok());

        let app_dir = app_dir.unwrap();
        assert!(app_dir.exists());
    }
}
