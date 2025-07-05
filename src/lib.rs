//! # LifeTracker 核心库
//!
//! 支持桌面端和移动端的共享核心逻辑

pub mod config;
pub mod core;
pub mod errors;
pub mod storage;
pub mod utils;

#[cfg(feature = "tauri")]
pub mod tauri_commands;

use tauri::Manager;

// 重新导出核心类型，方便外部使用
pub use config::{AppConfig, ConfigManager};
pub use core::{Analytics, AppCore, Category, CategoryColor, CategoryIcon, Task, Timer};
pub use errors::{AppError, ErrorHandler, ErrorSeverity, Result};
pub use storage::{models::CategoryModel, Database, DatabaseConfig, StorageManager, TimeEntry};

/// 移动端入口点
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 移动端直接运行完整应用
    init_logging();

    if let Err(e) = create_app_dirs() {
        eprintln!("创建应用目录失败: {}", e);
        std::process::exit(1);
    }

    #[cfg(feature = "tauri")]
    {
        if let Err(e) = create_app_builder_with_setup().run(tauri::generate_context!()) {
            eprintln!("运行应用失败: {}", e);
            std::process::exit(1);
        }
    }

    #[cfg(not(feature = "tauri"))]
    {
        eprintln!("Tauri 功能未启用，当前配置无可用界面");
        std::process::exit(1);
    }
}

/// 创建 Tauri 应用 Builder（桌面端可以基于此添加更多功能）
#[cfg(feature = "tauri")]
pub fn create_app_builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            tauri_commands::config::get_config,
            tauri_commands::task::get_tasks,
            tauri_commands::task::create_task,
            tauri_commands::task::update_task,
            tauri_commands::task::delete_task,
            tauri_commands::category::get_categories,
            tauri_commands::category::create_category,
            tauri_commands::category::update_category,
            tauri_commands::category::delete_category,
            tauri_commands::timer::get_timer_status,
            tauri_commands::timer::start_timer,
            tauri_commands::timer::pause_timer,
            tauri_commands::timer::stop_timer,
            tauri_commands::accounting::account::get_accounts,
            tauri_commands::accounting::account::get_account_by_id,
            tauri_commands::accounting::account::create_account,
            tauri_commands::accounting::account::update_account,
            tauri_commands::accounting::account::delete_account,
            tauri_commands::accounting::account::get_account_balance,
            tauri_commands::accounting::account::update_account_balance,
            tauri_commands::accounting::account::set_default_account,
            tauri_commands::accounting::transaction::get_transactions,
            tauri_commands::accounting::transaction::get_transaction_by_id,
            tauri_commands::accounting::transaction::create_transaction,
            tauri_commands::accounting::transaction::delete_transaction,
            tauri_commands::accounting::budget::get_budgets,
            tauri_commands::accounting::budget::create_budget,
            tauri_commands::accounting::budget::delete_budget,
            tauri_commands::accounting::category::get_transaction_categories,
            tauri_commands::accounting::category::create_transaction_category,
            tauri_commands::accounting::category::delete_transaction_category,
            tauri_commands::statistics::get_statistics,
            tauri_commands::statistics::get_financial_stats,
            tauri_commands::statistics::get_monthly_trend,
            tauri_commands::statistics::get_financial_trend,
            tauri_commands::data_io::export_data,
            tauri_commands::data_io::import_data,
            tauri_commands::data_io::backup_database,
            tauri_commands::data_io::restore_database,
        ])
}

/// 创建带有基础 setup 的 Tauri 应用 Builder
#[cfg(feature = "tauri")]
pub fn create_app_builder_with_setup() -> tauri::Builder<tauri::Wry> {
    create_app_builder().setup(|app| {
        log::info!("Tauri 应用初始化开始");

        // 显示主窗口 (在 Tauri v2 中 show() 是异步的，但在 setup 中我们可以忽略错误)
        if let Some(window) = app.get_webview_window("main") {
            // show() 在 Tauri v2 中是异步方法，但在 setup 中我们不能使用 await
            // 所以我们使用 tauri 的内部 API 或者忽略这个错误
            // 通常窗口默认是可见的，所以这行可能不是必需的
            // let _ = window.show();
        }

        // 初始化应用状态
        let app_state = create_app_state()?;
        app.manage(app_state);

        log::info!("Tauri 应用初始化完成");
        Ok(())
    })
}

/// 为桌面端提供的便捷运行函数（可以添加桌面端特有功能）
#[cfg(feature = "tauri")]
pub fn run_desktop_app<F>(customize_builder: F) -> Result<()>
where
    F: FnOnce(tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry>,
{
    // 初始化日志
    init_logging();

    // 创建应用目录
    if let Err(e) = create_app_dirs() {
        return Err(format!("创建应用目录失败: {}", e).into());
    }

    // 创建基础 builder（无 setup），然后让桌面端自定义
    let builder = create_app_builder();
    let customized_builder = customize_builder(builder);

    customized_builder
        .run(tauri::generate_context!())
        .map_err(|e| format!("运行应用失败: {}", e).into())
}

/// 创建应用状态（供桌面端 setup 使用）
#[cfg(feature = "tauri")]
pub fn create_app_state() -> Result<crate::tauri_commands::AppState> {
    use std::sync::{Arc, Mutex};

    // 创建配置
    let config = Arc::new(Mutex::new(AppConfig::default()));

    // 创建存储管理器
    let app_config = AppConfig::default();
    let db_config = crate::storage::DatabaseConfig {
        database_path: app_config.data.database_path.to_string_lossy().to_string(),
        enable_wal: true,
        pool_size: 10,
        timeout_seconds: 30,
    };
    let mut storage = Arc::new(StorageManager::new(db_config)?);

    // 初始化存储系统
    if let Some(storage_mut) = Arc::get_mut(&mut storage) {
        storage_mut.initialize()?;
    }

    // 创建计时器
    let timer = Arc::new(Mutex::new(Timer::new()));

    // 创建当前任务ID
    let current_task_id = Arc::new(Mutex::new(None));

    let app_state = crate::tauri_commands::AppState {
        storage,
        timer,
        config,
        current_task_id,
    };

    Ok(app_state)
}

/// 初始化日志系统
fn init_logging() {
    use env_logger::{Builder, Env};

    let env = Env::default()
        .filter_or("RUST_LOG", "info")
        .write_style_or("RUST_LOG_STYLE", "always");

    Builder::from_env(env).format_timestamp_micros().init();
}

/// 创建应用目录
fn create_app_dirs() -> Result<()> {
    let app_config = AppConfig::default();
    let data_dir = app_config
        .data
        .database_path
        .parent()
        .ok_or("无法获取数据目录")?;

    std::fs::create_dir_all(data_dir).map_err(|e| format!("创建数据目录失败: {}", e))?;

    Ok(())
}
