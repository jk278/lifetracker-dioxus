//! # Tauri 命令处理模块
//!
//! 提供前端调用的所有后端API命令
//!
//! 本模块采用分治策略将原本2000+行的单一文件重构为多个职责单一的子模块：
//! - task: 任务管理CRUD
//! - timer: 计时器控制
//! - category: 分类管理
//! - statistics: 统计分析
//! - data_io: 数据导入导出
//! - config: 应用配置管理
//! - accounting: 财务管理（账户、交易、预算）

use crate::{config::AppConfig, core::Timer, storage::StorageManager};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;
use tokio::{
    sync::Mutex as AsyncMutex,
    time::{timeout, Duration},
};
use uuid::Uuid;

// ========== 核心应用状态 ==========

/// 应用状态  
#[derive(Debug)]
pub struct AppState {
    pub storage: Arc<StorageManager>,
    pub timer: Arc<Mutex<Timer>>,
    pub config: Arc<Mutex<AppConfig>>,
    pub current_task_id: Arc<Mutex<Option<String>>>,
}

// ========== 数据传输对象 (DTOs) ==========

/// 任务数据传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub start_time: Option<DateTime<Local>>,
    pub end_time: Option<DateTime<Local>>,
    pub duration_seconds: i64,
    pub is_active: bool,
    pub tags: Vec<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

/// 分类数据传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub icon: Option<String>,
    pub is_active: bool,
    pub task_count: u32,
    pub total_duration_seconds: i64,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

/// 计时器状态传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerStatusDto {
    pub state: String, // "running", "paused", "stopped"
    pub current_task_id: Option<String>,
    pub current_task_name: Option<String>,
    pub start_time: Option<DateTime<Local>>,
    pub pause_time: Option<DateTime<Local>>,
    pub elapsed_seconds: i64,
    pub total_today_seconds: i64,
}

/// 统计数据传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsDto {
    pub today: DayStatDto,
    pub this_week: PeriodStatDto,
    pub this_month: PeriodStatDto,
    pub all_time: PeriodStatDto,
    pub category_stats: Vec<CategoryStatDto>,
    pub daily_trend: Vec<DailyTrendDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayStatDto {
    pub date: String,
    pub total_seconds: i64,
    pub task_count: u32,
    pub active_categories: u32,
    pub most_productive_hour: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodStatDto {
    pub total_seconds: i64,
    pub task_count: u32,
    pub active_days: u32,
    pub average_daily_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStatDto {
    pub category_id: String,
    pub category_name: String,
    pub total_seconds: i64,
    pub task_count: u32,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyTrendDto {
    pub date: String,
    pub total_seconds: i64,
    pub task_count: u32,
}

// 注意：记账相关的DTO已经移到 accounting/types.rs 模块中

/// 财务统计数据传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialStatsDto {
    pub total_income: f64,
    pub total_expense: f64,
    pub net_income: f64,
    pub account_balance: f64,
    pub transaction_count: i64,
    pub period_start: String,
    pub period_end: String,
    pub currency: String,
}

// ========== 公共辅助函数 ==========

/// 从数据库计算今日总时长
pub fn get_today_total_seconds(storage: &StorageManager) -> Result<i64, String> {
    let today = Local::now().date_naive();
    log::debug!("查询今日时间记录，日期: {}", today);

    match storage
        .get_database()
        .get_time_entries_by_date_range(today, today)
    {
        Ok(entries) => {
            log::debug!("查询到 {} 条今日时间记录", entries.len());
            let total_seconds: i64 = entries
                .iter()
                .map(|entry| {
                    log::debug!(
                        "时间记录: {} - {}秒",
                        entry.task_name,
                        entry.duration_seconds
                    );
                    entry.duration_seconds
                })
                .sum();
            log::debug!("今日总时长: {}秒", total_seconds);
            Ok(total_seconds)
        }
        Err(e) => {
            log::error!("查询今日时间记录失败: {}", e);
            // 返回默认值而不是错误，避免阻塞整个流程
            Ok(0)
        }
    }
}

// ========== 子模块声明 ==========

pub mod accounting;
pub mod category;
pub mod config;
pub mod data_io;
pub mod statistics;
pub mod task;
pub mod timer;

// ========== 重新导出给外部使用 ==========

pub use accounting::*;
pub use category::*;
pub use config::*;
pub use data_io::*;
pub use statistics::*;
pub use task::*;
pub use timer::*;
