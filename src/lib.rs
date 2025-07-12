//! # LifeTracker 核心库
//!
//! 生活追踪应用的核心功能模块

use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod config;
pub mod core;
pub mod errors;
pub mod storage;
pub mod sync;
pub mod utils;

// 重新导出常用类型
pub use errors::{AppError, Result};
pub use storage::database::Database;

/// 应用全局状态
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接
    pub database: Option<Arc<Database>>,
    /// 应用配置
    pub config: config::AppConfig,
    /// 初始化状态
    pub initialized: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            database: None,
            config: config::AppConfig::default(),
            initialized: false,
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

/// 关闭应用
pub async fn shutdown_app(_app_state: &AppState) -> Result<()> {
    log::info!("开始关闭应用");

    // 这里可以添加清理逻辑
    log::info!("应用关闭完成");
    Ok(())
}
