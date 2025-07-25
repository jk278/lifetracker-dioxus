//! # LifeTracker 核心库
//!
//! 生活追踪应用的核心功能模块

// lib.rs 是一个 Rust 库（crate）的根文件。当你创建一个 Rust 库项目时，这个文件是默认的入口点。
// 一个 Rust 项目可以有一个 lib.rs（库），也可以有一个 main.rs（二进制可执行文件），或者两者都有。
// lib.rs 和 mod.rs 的主要区别：
// 1. lib.rs 是库的根模块（crate root），mod.rs 是目录的模块根文件（module root for a directory）。
// 2. lib.rs 位于项目根目录的 src/ 下，mod.rs 位于任何作为模块的目录中.
// 3. lib.rs 定义整个库的入口和公共接口，mod.rs 声明和组织当前目录下的子模块。
// 4. 一个库项目只有一个 lib.rs，而一个目录可以有多个 mod.rs（一个模块目录可以有一个 mod.rs）。
// 5. lib.rs 可包含模块、公共 API、类型、函数、常量和宏，而 mod.rs 不能。

// 导入一些常用的库和模块。
// once_cell::sync::Lazy 是一个线程安全的延迟初始化宏，用于创建单例实例。
// std::sync::Arc 是 Rust 中的智能指针，用于在多线程环境中共享数据。
// tokio::sync::RwLock 是 Rust 中的读写锁，用于在多线程环境中保护共享数据。
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

// 定义了库的公共 API，通过 pub 关键字声明的项（函数、结构体、枚举、模块等）将从这里暴露给其他 crate 使用。
// 它负责声明和组织库内部的所有模块。
pub mod config;
pub mod core;
pub mod errors;
pub mod storage;
pub mod sync;
pub mod utils;

// 重新导出常用类型
pub use errors::{AppError, Result};
pub use storage::database::Database;

/// 主题模式枚举
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ThemeMode {
    /// 浅色模式
    Light,
    /// 深色模式
    Dark,
    /// 跟随系统
    System,
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::System
    }
}

impl ThemeMode {
    /// 转换为字符串
    pub fn to_string(&self) -> String {
        match self {
            ThemeMode::Light => "light".to_string(),
            ThemeMode::Dark => "dark".to_string(),
            ThemeMode::System => "system".to_string(),
        }
    }
    
    /// 从字符串创建
    pub fn from_string(s: &str) -> Self {
        match s {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            "system" => ThemeMode::System,
            _ => ThemeMode::System,
        }
    }
    
    /// 判断是否为深色模式
    pub fn is_dark(&self) -> bool {
        match self {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => {
                // 使用config模块的系统主题检测
                config::theme::ThemeConfig::detect_system_theme() == "dark"
            }
        }
    }
}

/// 应用全局状态
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接
    pub database: Option<Arc<Database>>,
    /// 应用配置
    pub config: config::AppConfig,
    /// 初始化状态
    pub initialized: bool,
    /// 当前主题模式
    pub theme_mode: ThemeMode,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            database: None,
            config: config::AppConfig::default(),
            initialized: false,
            theme_mode: ThemeMode::default(),
        }
    }
}

impl AppState {
    /// 创建新的应用状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 同步初始化应用状态（避免runtime嵌套）
    pub fn initialize_sync(&mut self) -> Result<()> {
        log::info!("开始同步初始化应用状态");

        // 初始化数据库（同步方式）
        let database = Database::new("./data/lifetracker.db")?;
        database.run_migrations()?;

        // 加载配置并设置主题
        if let Ok(config_path) = config::get_default_config_path() {
            if let Ok(config_manager) = config::ConfigManager::new(config_path) {
                self.config = config_manager.config().clone();
                
                // 根据配置设置主题模式
                self.theme_mode = if self.config.ui.theme == "system" {
                    ThemeMode::System
                } else if self.config.ui.dark_mode {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                };
                
                log::info!("已加载配置，主题模式: {:?}", self.theme_mode);
            }
        }

        self.database = Some(Arc::new(database));
        self.initialized = true;

        log::info!("应用状态同步初始化完成");
        Ok(())
    }

    /// 获取数据库连接
    pub fn get_database(&self) -> Option<Arc<Database>> {
        self.database.clone()
    }
}

/// 全局应用状态实例（使用 Lazy 延迟初始化）
static APP_STATE: Lazy<RwLock<AppState>> = Lazy::new(|| RwLock::new(AppState::default()));

/// 同步初始化应用（避免runtime嵌套）
pub fn initialize_app_sync() -> Result<()> {
    let mut state = AppState::new();
    state.initialize_sync()?;

    // 使用 try_write 避免阻塞
    match APP_STATE.try_write() {
        Ok(mut app_state) => {
            *app_state = state;
            Ok(())
        }
        Err(_) => Err(AppError::System("无法获取应用状态写锁".to_string())),
    }
}

/// 获取全局应用状态（同步版本）
pub fn get_app_state_sync() -> AppState {
    match APP_STATE.try_read() {
        Ok(state) => state.clone(),
        Err(_) => {
            log::warn!("无法获取应用状态读锁，返回默认状态");
            AppState::default()
        }
    }
}

/// 异步获取全局应用状态（兼容接口）
pub async fn get_app_state() -> AppState {
    APP_STATE.read().await.clone()
}

/// 设置全局应用状态
pub async fn set_app_state(state: AppState) {
    *APP_STATE.write().await = state;
}

/// 获取当前主题模式
pub fn get_theme_mode() -> ThemeMode {
    get_app_state_sync().theme_mode
}

/// 设置主题模式
pub fn set_theme_mode(theme_mode: ThemeMode) -> Result<()> {
    match APP_STATE.try_write() {
        Ok(mut state) => {
            state.theme_mode = theme_mode;
            Ok(())
        }
        Err(_) => Err(AppError::System("无法获取应用状态写锁".to_string())),
    }
}

/// 切换主题模式
pub fn toggle_theme() -> Result<ThemeMode> {
    let current_theme = get_theme_mode();
    let new_theme = match current_theme {
        ThemeMode::Light => ThemeMode::Dark,
        ThemeMode::Dark => ThemeMode::Light,
        ThemeMode::System => {
            // 如果当前是系统模式，根据检测到的系统主题切换到相反模式
            if current_theme.is_dark() {
                ThemeMode::Light
            } else {
                ThemeMode::Dark
            }
        }
    };
    
    set_theme_mode(new_theme.clone())?;
    Ok(new_theme)
}

/// 关闭应用
pub async fn shutdown_app(_app_state: &AppState) -> Result<()> {
    log::info!("开始关闭应用");

    // 这里可以添加清理逻辑
    log::info!("应用关闭完成");
    Ok(())
}
