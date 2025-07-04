//! # LifeTracker 主程序
//!
//! LifeTracker 是一个综合性的生活追踪和管理工具，支持时间追踪、财务记录、
//! 日记写作、习惯打卡等功能。
//!
//! ## 主要功能
//! - 时间追踪和任务管理
//! - 财务记录和统计
//! - 日记写作功能
//! - 习惯追踪和打卡
//! - 数据统计和可视化
//! - 数据导入导出
//!
//! ## 架构
//! - 前端：React + TypeScript + Tailwind CSS
//! - 后端：Rust + Tauri + SQLite
//! - 跨平台：Windows/macOS/Linux

// Windows 子系统配置：GUI应用不显示控制台窗口
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod config;
mod core;
mod errors;
mod storage;
mod utils;

#[cfg(feature = "tauri")]
mod tauri_commands;

use std::process;

#[cfg(feature = "tauri")]
use tauri::window::Color;

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

    // 启动Tauri模式
    let result = {
        #[cfg(feature = "tauri")]
        {
            log::info!("启动Tauri模式");
            run_tauri_mode().await
        }
        #[cfg(not(feature = "tauri"))]
        {
            eprintln!("Tauri功能未启用，当前配置无可用界面");
            process::exit(1);
        }
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
    use env_logger::{Builder, Env};
    use log::LevelFilter;

    let env = Env::default()
        .filter_or("RUST_LOG", "info") // 将默认日志级别恢复为 info
        .write_style_or("RUST_LOG_STYLE", "always");

    Builder::from_env(env).format_timestamp_micros().init();
}

/// 运行Tauri模式
#[cfg(feature = "tauri")]
async fn run_tauri_mode() -> Result<()> {
    use tauri::Manager;

    // 创建Tauri应用
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // 应用初始化
            log::info!("Tauri应用初始化开始");

            // 检测系统主题并设置正确的窗口背景色（与Tailwind颜色完全一致）
            let is_dark_theme =
                crate::config::theme::ThemeConfig::get_initial_theme_class() == "dark";
            let bg_color = if is_dark_theme {
                Color(15, 20, 25, 255) // 暗色模式背景 #0f1419
            } else {
                Color(249, 250, 251, 255) // 亮色模式背景 #f9fafb (gray-50)
            };

            if let Some(window) = app.get_webview_window("main") {
                // 设置背景色避免启动闪烁和拖拽残影
                if let Err(e) = window.set_background_color(Some(bg_color)) {
                    log::warn!("设置窗口背景色失败: {}", e);
                } else {
                    log::info!("窗口背景色已设置为: {:?}", bg_color);
                }

                // 显示窗口
                if let Err(e) = window.show() {
                    log::warn!("显示窗口失败: {}", e);
                } else {
                    log::info!("窗口已显示");
                }
            }

            // 创建应用配置
            let app_config = get_app_config();
            log::info!("数据库路径: {}", app_config.database_path);

            // 初始化存储管理器
            let storage_config = crate::storage::DatabaseConfig {
                database_path: app_config.database_path.clone(),
                ..Default::default()
            };

            let storage_manager = match crate::storage::StorageManager::new(storage_config) {
                Ok(mut sm) => {
                    // 初始化数据库
                    if let Err(e) = sm.initialize() {
                        log::error!("数据库初始化失败: {}", e);
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("数据库初始化失败: {}", e),
                        )));
                    }
                    log::info!("数据库初始化成功");
                    sm
                }
                Err(e) => {
                    log::error!("存储管理器创建失败: {}", e);
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("存储管理器创建失败: {}", e),
                    )));
                }
            };

            // 创建计时器
            let timer = crate::core::Timer::new();

            // 创建应用状态
            let app_state = tauri_commands::AppState {
                storage: std::sync::Arc::new(storage_manager),
                timer: std::sync::Arc::new(std::sync::Mutex::new(timer)),
                config: std::sync::Arc::new(std::sync::Mutex::new(
                    crate::config::AppConfig::default(),
                )),
                current_task_id: std::sync::Arc::new(std::sync::Mutex::new(None)),
            };

            // 注册应用状态
            app.manage(app_state);

            // 设置系统托盘
            if let Err(e) = setup_system_tray(app) {
                log::error!("设置系统托盘失败: {}", e);
            }

            // 设置托盘菜单事件处理
            app.on_menu_event(move |app, event| match event.id().as_ref() {
                "start_timer" => {
                    log::info!("从托盘开始计时");
                    // TODO: 调用实际的开始计时命令
                }
                "pause_timer" => {
                    log::info!("从托盘暂停计时");
                    // TODO: 调用实际的暂停计时命令
                }
                "stop_timer" => {
                    log::info!("从托盘停止计时");
                    // TODO: 调用实际的停止计时命令
                }
                "show_window" => {
                    log::info!("托盘菜单：显示主窗口");
                    if let Some(window) = app.get_webview_window("main") {
                        let is_visible = window.is_visible().unwrap_or(false);
                        let is_minimized = window.is_minimized().unwrap_or(false);

                        if !is_visible || is_minimized {
                            let _ = window.show();
                            let _ = window.unminimize();
                        }

                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    log::info!("托盘菜单：退出应用");
                    app.exit(0);
                }
                _ => {}
            });

            log::info!("Tauri应用初始化完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tauri_commands::get_tasks,
            tauri_commands::create_task,
            tauri_commands::update_task,
            tauri_commands::delete_task,
            tauri_commands::start_timer,
            tauri_commands::stop_timer,
            tauri_commands::pause_timer,
            tauri_commands::resume_timer,
            tauri_commands::get_timer_status,
            tauri_commands::get_today_time_entries,
            tauri_commands::debug_get_time_entries,
            tauri_commands::get_today_stats,
            tauri_commands::get_categories,
            tauri_commands::create_category,
            tauri_commands::update_category,
            tauri_commands::delete_category,
            tauri_commands::get_statistics,
            tauri_commands::export_data,
            tauri_commands::import_data,
            tauri_commands::get_config,
            tauri_commands::update_config,
            tauri_commands::set_window_theme,
            // 记账功能命令
            tauri_commands::get_accounts,
            tauri_commands::create_account,
            tauri_commands::update_account,
            tauri_commands::delete_account,
            tauri_commands::get_transactions,
            tauri_commands::create_transaction,
            tauri_commands::delete_transaction,
            tauri_commands::get_financial_stats,
            tauri_commands::get_monthly_trend,
            tauri_commands::get_financial_trend,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Tauri运行失败: {}", e),
            ))
        })?;

    Ok(())
}

/// 设置系统托盘
#[cfg(feature = "tauri")]
fn setup_system_tray(app: &tauri::App) -> Result<()> {
    use tauri::{
        menu::{Menu, MenuItem, PredefinedMenuItem},
        tray::{TrayIcon, TrayIconBuilder, TrayIconEvent},
        Manager,
    };

    // 创建托盘菜单
    let start_item = MenuItem::with_id(app, "start_timer", "开始计时", true, None::<&str>)?;
    let pause_item = MenuItem::with_id(app, "pause_timer", "暂停计时", true, None::<&str>)?;
    let stop_item = MenuItem::with_id(app, "stop_timer", "停止计时", true, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show_window", "显示主窗口", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show_item,
            &PredefinedMenuItem::separator(app)?,
            &start_item,
            &pause_item,
            &stop_item,
            &PredefinedMenuItem::separator(app)?,
            &quit_item,
        ],
    )?;

    // 创建托盘图标
    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false) // 左键不自动弹菜单，由我们手动控制
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                // 只处理左键点击，右键点击由菜单处理
                if button == tauri::tray::MouseButton::Left {
                    if let Some(window) = tray.app_handle().get_webview_window("main") {
                        // 简化处理：直接显示并聚焦窗口
                        let is_visible = window.is_visible().unwrap_or(false);
                        let is_minimized = window.is_minimized().unwrap_or(false);

                        if !is_visible || is_minimized {
                            // 窗口隐藏或最小化：显示并聚焦
                            let _ = window.show();
                            let _ = window.unminimize();
                        }

                        // 始终聚焦窗口
                        let _ = window.set_focus();
                        log::info!("托盘左键点击：窗口已聚焦");
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
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
            database_path: app_dir.join("lifetracker.db").to_string_lossy().to_string(),
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
    if let Ok(db_path) = std::env::var("LIFETRACKER_DB_PATH") {
        config.database_path = db_path;
    }

    if let Ok(config_path) = std::env::var("LIFETRACKER_CONFIG_PATH") {
        config.config_path = config_path;
    }

    if let Ok(log_level) = std::env::var("LIFETRACKER_LOG_LEVEL") {
        config.log_level = log_level;
    }

    if std::env::var("LIFETRACKER_DEBUG").is_ok() {
        config.debug_mode = true;
    }

    config
}

/// 创建应用程序目录
pub fn create_app_dirs() -> Result<()> {
    use std::fs;

    // 统一使用 utils::get_app_data_dir()，在开发环境下它仍会返回 ./data
    let app_dir = crate::utils::get_app_data_dir()?;

    // 需要创建的子目录
    let sub_dirs = ["", "config", "logs", "backups", "exports"];

    for sub in &sub_dirs {
        let dir = app_dir.join(sub);
        if let Err(e) = fs::create_dir_all(&dir) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(format!("创建目录 {} 失败: {}", dir.display(), e).into());
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
    log::info!("LifeTracker 正在关闭");

    // 清理临时文件
    let temp_files = ["test_write_permission", ".lifetracker.lock"];

    for file in temp_files {
        if std::path::Path::new(file).exists() {
            if let Err(e) = std::fs::remove_file(file) {
                log::warn!("清理临时文件 {} 失败: {}", file, e);
            }
        }
    }

    log::info!("LifeTracker 已关闭");
}

/// 处理启动时错误
fn handle_startup_error(message: &str) {
    log::error!("{}", message);

    // 在调试模式或CLI模式下，输出到控制台
    #[cfg(debug_assertions)]
    eprintln!("{}", message);

    // 在Windows发布版本中，写入错误文件
    #[cfg(all(target_os = "windows", not(debug_assertions)))]
    {
        if let Err(_) = std::fs::write("lifetracker_error.log", message) {
            // 写入文件失败时记录日志
            log::warn!("无法写入错误日志文件");
        }
    }
}

/// 处理运行时错误
fn handle_runtime_error(message: &str) {
    log::error!("{}", message);

    // 在调试模式下，输出到控制台
    #[cfg(debug_assertions)]
    eprintln!("{}", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        // 数据库路径现在应该包含完整路径，而不是简单的文件名
        assert!(config.database_path.ends_with("lifetracker.db"));
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
