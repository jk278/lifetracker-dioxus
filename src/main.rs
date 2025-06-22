//! # TimeTracker 主程序
//!
//! 一个功能强大的时间跟踪和管理工具
//! 支持CLI和GUI两种界面模式

// Windows 子系统配置：GUI应用不显示控制台窗口
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions), feature = "gui"),
    windows_subsystem = "windows"
)]

mod cli;
mod config;
mod core;
mod errors;
mod gui;
mod storage;
mod utils;

use std::process;

/// 应用程序错误类型
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// 主程序入口点
#[tokio::main]
async fn main() {
    // 初始化日志系统
    init_logging();

    // 创建应用程序目录
    if let Err(e) = create_app_dirs() {
        handle_startup_error(&format!("创建应用目录失败: {}", e));
        process::exit(1);
    }

    // 检查应用程序依赖
    if let Err(e) = check_dependencies() {
        handle_startup_error(&format!("依赖检查失败: {}", e));
        process::exit(1);
    }

    // 简单检查命令行参数
    let args: Vec<String> = std::env::args().collect();

    let result = if args.len() > 1
        && (args.contains(&"--gui".to_string()) || args.contains(&"-g".to_string()))
    {
        // GUI模式：直接启动GUI
        log::info!("启动GUI模式");
        run_gui_mode().await
    } else if args.len() == 1 {
        // 没有参数，默认启动GUI
        log::info!("默认启动GUI模式");
        run_gui_mode().await
    } else {
        // CLI模式：使用clap解析参数
        log::info!("启动CLI模式");
        run_cli_mode().await
    };

    // 清理应用程序
    cleanup();

    // 处理运行结果
    if let Err(e) = result {
        handle_runtime_error(&format!("运行时错误: {}", e));
        process::exit(1);
    }
}

/// 初始化日志系统
fn init_logging() {
    use env_logger::{Builder, Target};
    use log::LevelFilter;
    use std::io::Write;

    let mut builder = Builder::from_default_env();

    // 在GUI发布模式下，将日志写入文件
    #[cfg(all(not(debug_assertions), feature = "gui"))]
    {
        use std::fs::OpenOptions;

        // 创建日志目录
        if let Err(_) = std::fs::create_dir_all("data/logs") {
            // 如果无法创建目录，回退到当前目录
        }

        let log_file_path = "data/logs/timetracker.log";

        if let Ok(log_file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)
        {
            builder
                .target(Target::Pipe(Box::new(log_file)))
                .filter_level(LevelFilter::Info);
        } else {
            // 如果无法创建日志文件，使用内存目标（不输出）
            builder
                .target(Target::Stderr)
                .filter_level(LevelFilter::Off);
        }
    }

    // 在调试模式或CLI模式下，输出到控制台
    #[cfg(any(debug_assertions, not(feature = "gui")))]
    {
        builder
            .target(Target::Stdout)
            .filter_level(LevelFilter::Info);
    }

    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] [{}:{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();

    log::info!("TimeTracker 启动");
}

/// 运行GUI模式
async fn run_gui_mode() -> Result<()> {
    // 使用GUI模块的run_gui函数
    gui::run_gui(None)?;
    Ok(())
}

/// 运行CLI模式
async fn run_cli_mode() -> Result<()> {
    // 使用CLI模块的run_cli函数
    cli::run_cli()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
}

/// 应用程序配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 数据库路径
    pub database_path: String,
    /// 配置文件路径
    pub config_path: String,
    /// 日志级别
    pub log_level: String,
    /// 是否启用调试模式
    pub debug_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        // 获取应用数据目录，失败时使用./data目录
        let app_dir =
            crate::utils::get_app_data_dir().unwrap_or_else(|_| std::path::PathBuf::from("./data"));

        Self {
            database_path: app_dir.join("timetracker.db").to_string_lossy().to_string(),
            config_path: app_dir.join("config.toml").to_string_lossy().to_string(),
            log_level: "info".to_string(),
            debug_mode: false,
        }
    }
}

/// 获取应用程序配置
pub fn get_app_config() -> AppConfig {
    let mut config = AppConfig::default();

    // 从环境变量读取配置
    if let Ok(db_path) = std::env::var("TIMETRACKER_DB_PATH") {
        config.database_path = db_path;
    }

    if let Ok(config_path) = std::env::var("TIMETRACKER_CONFIG_PATH") {
        config.config_path = config_path;
    }

    if let Ok(log_level) = std::env::var("TIMETRACKER_LOG_LEVEL") {
        config.log_level = log_level;
    }

    if std::env::var("TIMETRACKER_DEBUG").is_ok() {
        config.debug_mode = true;
    }

    config
}

/// 创建应用程序目录
pub fn create_app_dirs() -> Result<()> {
    use std::fs;

    // 在开发环境下，优先使用本地data目录
    let is_dev = cfg!(debug_assertions) || std::env::var("CARGO").is_ok();

    let base_dirs = if is_dev {
        // 开发环境：使用项目内的data目录，避免污染根目录
        vec![
            "data",
            "data/config",
            "data/logs",
            "data/backups",
            "data/exports",
        ]
    } else {
        // 生产环境：使用标准目录结构
        vec!["data", "config", "logs", "backups", "exports"]
    };

    for dir in base_dirs {
        if let Err(e) = fs::create_dir_all(dir) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(format!("创建目录 {} 失败: {}", dir, e).into());
            }
        }
    }

    Ok(())
}

/// 检查应用程序依赖
pub fn check_dependencies() -> Result<()> {
    // 检查SQLite是否可用
    match rusqlite::Connection::open(":memory:") {
        Ok(_) => log::info!("SQLite 检查通过"),
        Err(e) => return Err(format!("SQLite 不可用: {}", e).into()),
    }

    // 检查文件系统权限
    match std::fs::File::create("test_write_permission") {
        Ok(_) => {
            let _ = std::fs::remove_file("test_write_permission");
            log::info!("文件系统权限检查通过");
        }
        Err(e) => return Err(format!("文件系统权限不足: {}", e).into()),
    }

    Ok(())
}

/// 应用程序清理
pub fn cleanup() {
    log::info!("TimeTracker 正在关闭");

    // 清理临时文件
    let temp_files = ["test_write_permission", ".timetracker.lock"];

    for file in temp_files {
        if std::path::Path::new(file).exists() {
            if let Err(e) = std::fs::remove_file(file) {
                log::warn!("清理临时文件 {} 失败: {}", file, e);
            }
        }
    }

    log::info!("TimeTracker 已关闭");
}

/// 处理启动时错误
fn handle_startup_error(message: &str) {
    log::error!("{}", message);

    // 在调试模式或CLI模式下，输出到控制台
    #[cfg(debug_assertions)]
    eprintln!("{}", message);

    // 在Windows GUI发布版本中，写入错误文件
    #[cfg(all(target_os = "windows", not(debug_assertions)))]
    {
        if let Err(_) = std::fs::write("timetracker_error.log", message) {
            // 如果无法写入文件，尝试显示消息框
            #[cfg(feature = "gui")]
            show_error_dialog("启动错误", message);
        }
    }
}

/// 处理运行时错误
fn handle_runtime_error(message: &str) {
    log::error!("{}", message);

    // 在调试模式下，输出到控制台
    #[cfg(debug_assertions)]
    eprintln!("{}", message);

    // 在GUI模式下，尝试显示错误对话框
    #[cfg(feature = "gui")]
    show_error_dialog("运行时错误", message);
}

/// 显示错误对话框（仅在GUI模式下）
#[cfg(feature = "gui")]
fn show_error_dialog(title: &str, message: &str) {
    // 这里可以实现一个简单的错误对话框
    // 或者将错误传递给GUI系统处理
    log::error!("GUI错误对话框: {} - {}", title, message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        // 数据库路径现在应该包含完整路径，而不是简单的文件名
        assert!(config.database_path.ends_with("timetracker.db"));
        assert!(config.config_path.ends_with("config.toml"));
        assert_eq!(config.log_level, "info");
        assert!(!config.debug_mode);
    }

    #[test]
    fn test_get_app_config() {
        let config = get_app_config();
        assert!(!config.database_path.is_empty());
        assert!(!config.config_path.is_empty());
    }

    #[tokio::test]
    async fn test_check_dependencies() {
        assert!(check_dependencies().is_ok());
    }
}
