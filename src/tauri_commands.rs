//! # Tauri 命令处理模块
//!
//! 提供前端调用的所有后端API命令

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

/// 应用状态
#[derive(Default)]
pub struct AppState {
    pub storage: Arc<AsyncMutex<Option<StorageManager>>>,
    pub timer: Arc<Mutex<Timer>>,
    pub config: Arc<Mutex<AppConfig>>,
    pub current_task_id: Arc<Mutex<Option<String>>>,
}

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

/// 创建任务请求
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// 更新任务请求
#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// 创建分类请求
#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

/// 更新分类请求
#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub is_active: Option<bool>,
}

// ========== 任务管理命令 ==========

/// 获取所有任务
#[tauri::command]
pub async fn get_tasks(
    state: State<'_, AppState>,
    limit: Option<u32>,
    offset: Option<u32>,
    category_id: Option<String>,
) -> Result<Vec<TaskDto>, String> {
    log::debug!(
        "获取任务列表 - limit: {:?}, offset: {:?}, category_id: {:?}",
        limit,
        offset,
        category_id
    );

    // 在作用域内获取数据，然后立即释放锁
    let (tasks, category_names, db_access_time) = {
        let start_time = std::time::Instant::now();
        let storage_guard = state.storage.lock().await;
        let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

        // 从数据库获取任务
        let tasks_result = if let Some(cat_id_str) = &category_id {
            if let Ok(cat_id) = Uuid::parse_str(cat_id_str) {
                storage
                    .get_database()
                    .get_tasks_by_category(cat_id)
                    .map_err(|e| e.to_string())?
            } else {
                return Err("无效的分类ID".to_string());
            }
        } else {
            storage
                .get_database()
                .get_all_tasks()
                .map_err(|e| e.to_string())?
        };

        // 获取分类信息，用于填充分类名称
        let mut category_names = std::collections::HashMap::new();
        for task in &tasks_result {
            if let Some(cat_id) = task.category_id {
                if !category_names.contains_key(&cat_id) {
                    if let Ok(Some(category)) = storage.get_database().get_category_by_id(cat_id) {
                        category_names.insert(cat_id, category.name);
                    }
                }
            }
        }

        let elapsed = start_time.elapsed();
        (tasks_result, category_names, elapsed)
    };

    log::debug!(
        "从数据库获取到 {} 个任务，耗时 {:?}",
        tasks.len(),
        db_access_time
    );

    // 锁已释放，现在可以安全地进行数据转换
    let mut task_dtos = Vec::new();
    for task in tasks {
        // 解析标签
        let tags: Vec<String> = serde_json::from_str(&task.tags).unwrap_or_default();

        task_dtos.push(TaskDto {
            id: task.id.to_string(),
            name: task.name,
            description: task.description,
            category_id: task.category_id.map(|id| id.to_string()),
            category_name: task
                .category_id
                .and_then(|id| category_names.get(&id).cloned()),
            start_time: None, // 任务开始时间由计时器管理
            end_time: None,   // 任务结束时间由计时器管理
            duration_seconds: task.total_duration_seconds,
            is_active: false, // 激活状态由计时器管理
            tags,
            created_at: task.created_at,
            updated_at: task.updated_at.unwrap_or(task.created_at),
        });
    }

    log::debug!("转换为 {} 个TaskDto", task_dtos.len());

    // 应用限制和偏移
    let start = offset.unwrap_or(0) as usize;
    let end = if let Some(limit) = limit {
        std::cmp::min(start + limit as usize, task_dtos.len())
    } else {
        task_dtos.len()
    };

    let result = if start < task_dtos.len() {
        task_dtos[start..end].to_vec()
    } else {
        Vec::new()
    };

    log::debug!("返回 {} 个任务", result.len());
    Ok(result)
}

/// 创建新任务
#[tauri::command]
pub async fn create_task(
    state: State<'_, AppState>,
    request: CreateTaskRequest,
) -> Result<TaskDto, String> {
    log::info!("创建任务: {}", request.name);
    log::debug!("创建任务请求: {:?}", request);

    // 创建任务插入模型
    let task_id = Uuid::new_v4();
    let tags_json = serde_json::to_string(&request.tags.unwrap_or_default())
        .map_err(|e| format!("标签序列化失败: {}", e))?;

    let task_insert = crate::storage::TaskInsert {
        id: task_id,
        name: request.name.clone(),
        description: request.description.clone(),
        category_id: request
            .category_id
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok()),
        status: "pending".to_string(),
        priority: "medium".to_string(),
        estimated_duration_seconds: None,
        total_duration_seconds: 0,
        tags: tags_json.clone(),
        due_date: None,
        is_completed: false,
        completed_at: None,
        created_at: chrono::Local::now(),
    };

    log::debug!("任务插入模型: {:?}", task_insert);

    // 在作用域内执行数据库操作，然后立即释放锁
    let (insert_result, category_name) = {
        let storage_guard = state.storage.lock().await;
        let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

        // 验证分类ID（如果存在）
        if let Some(category_id_str) = &request.category_id {
            if let Ok(category_id) = Uuid::parse_str(category_id_str) {
                match storage.get_database().get_category_by_id(category_id) {
                    Ok(None) => return Err("指定的分类不存在".to_string()),
                    Err(e) => return Err(format!("验证分类失败: {}", e)),
                    _ => {}
                }
            } else {
                return Err("无效的分类ID格式".to_string());
            }
        }

        // 插入到数据库
        let row_id = storage
            .get_database()
            .insert_task(&task_insert)
            .map_err(|e| format!("创建任务失败: {}", e))?;

        // 获取分类名称
        let cat_name = if let Some(cat_id) = task_insert.category_id {
            match storage.get_database().get_category_by_id(cat_id) {
                Ok(Some(category)) => Some(category.name),
                _ => None,
            }
        } else {
            None
        };

        (row_id, cat_name)
    };

    log::info!(
        "任务创建成功: {} (ID: {}, 插入行ID: {})",
        request.name,
        task_id,
        insert_result
    );

    // 解析标签
    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

    // 返回创建的任务
    let task_dto = TaskDto {
        id: task_id.to_string(),
        name: task_insert.name,
        description: task_insert.description,
        category_id: task_insert.category_id.map(|id| id.to_string()),
        category_name,
        start_time: None,
        end_time: None,
        duration_seconds: task_insert.total_duration_seconds,
        is_active: false,
        tags,
        created_at: task_insert.created_at,
        updated_at: task_insert.created_at,
    };

    log::debug!("返回TaskDto: {:?}", task_dto);
    Ok(task_dto)
}

/// 更新任务
#[tauri::command]
pub async fn update_task(
    state: State<'_, AppState>,
    task_id: String,
    request: UpdateTaskRequest,
) -> Result<TaskDto, String> {
    log::info!("更新任务: {}", task_id);

    let uuid = Uuid::parse_str(&task_id).map_err(|_| "无效的任务ID格式".to_string())?;

    // 创建更新模型
    let tags_json = if let Some(tags) = &request.tags {
        Some(serde_json::to_string(tags).map_err(|e| format!("标签序列化失败: {}", e))?)
    } else {
        None
    };

    let task_update = crate::storage::TaskUpdate {
        name: request.name.clone(),
        description: Some(request.description.clone()),
        category_id: Some(
            request
                .category_id
                .as_ref()
                .and_then(|s| Uuid::parse_str(s).ok()),
        ),
        status: None,   // 保持原有状态
        priority: None, // 保持原有优先级
        estimated_duration_seconds: None,
        total_duration_seconds: None, // 保持原有时长
        tags: tags_json.clone(),
        due_date: None,
        is_completed: None,
        completed_at: None,
    };

    // 在作用域内执行数据库操作，然后立即释放锁
    let (updated_task, category_name) = {
        let storage_guard = state.storage.lock().await;
        let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

        // 验证任务是否存在
        let existing_task = storage
            .get_database()
            .get_task_by_id(uuid)
            .map_err(|e| format!("查询任务失败: {}", e))?
            .ok_or("任务不存在".to_string())?;

        // 验证分类ID（如果提供）
        if let Some(category_id_str) = &request.category_id {
            if let Ok(category_id) = Uuid::parse_str(category_id_str) {
                match storage.get_database().get_category_by_id(category_id) {
                    Ok(None) => return Err("指定的分类不存在".to_string()),
                    Err(e) => return Err(format!("验证分类失败: {}", e)),
                    _ => {}
                }
            } else {
                return Err("无效的分类ID格式".to_string());
            }
        }

        // 更新数据库
        storage
            .get_database()
            .update_task(uuid, &task_update)
            .map_err(|e| format!("更新任务失败: {}", e))?;

        log::info!("任务更新成功: {}", task_id);

        // 获取更新后的任务
        let task = storage
            .get_database()
            .get_task_by_id(uuid)
            .map_err(|e| format!("获取更新后任务失败: {}", e))?
            .ok_or("更新后任务不存在".to_string())?;

        // 获取分类名称
        let cat_name = if let Some(cat_id) = task.category_id {
            match storage.get_database().get_category_by_id(cat_id) {
                Ok(Some(category)) => Some(category.name),
                _ => None,
            }
        } else {
            None
        };

        (task, cat_name)
    };

    // 解析标签
    let tags: Vec<String> = serde_json::from_str(&updated_task.tags).unwrap_or_default();

    // 返回更新后的任务
    Ok(TaskDto {
        id: updated_task.id.to_string(),
        name: updated_task.name,
        description: updated_task.description,
        category_id: updated_task.category_id.map(|id| id.to_string()),
        category_name,
        start_time: None,
        end_time: None,
        duration_seconds: updated_task.total_duration_seconds,
        is_active: false,
        tags,
        created_at: updated_task.created_at,
        updated_at: updated_task.updated_at.unwrap_or(updated_task.created_at),
    })
}

/// 删除任务
#[tauri::command]
pub async fn delete_task(state: State<'_, AppState>, task_id: String) -> Result<bool, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    let uuid = Uuid::parse_str(&task_id).map_err(|_| "无效的任务ID格式".to_string())?;

    log::info!("删除任务: {}", task_id);

    // 验证任务是否存在
    let _existing_task = storage
        .get_database()
        .get_task_by_id(uuid)
        .map_err(|e| format!("查询任务失败: {}", e))?
        .ok_or("任务不存在".to_string())?;

    // 删除任务
    storage
        .get_database()
        .delete_task(uuid)
        .map_err(|e| format!("删除任务失败: {}", e))?;

    log::info!("任务删除成功: {}", task_id);

    Ok(true)
}

// ========== 计时器控制命令 ==========

/// 开始计时
#[tauri::command]
pub async fn start_timer(
    state: State<'_, AppState>,
    task_id: String,
    app_handle: AppHandle,
) -> Result<TimerStatusDto, String> {
    // 1. 验证任务是否存在
    let task_uuid = Uuid::parse_str(&task_id).map_err(|_| "无效的任务ID")?;
    {
        let storage_guard = state.storage.lock().await;
        let storage = storage_guard.as_ref().ok_or("存储未初始化")?;
        storage
            .get_database()
            .get_task_by_id(task_uuid)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "任务不存在".to_string())?;
    }

    // 2. 启动计时器核心逻辑
    {
        let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
        timer.start().map_err(|e| e.to_string())?;
    }

    // 3. 更新当前任务ID
    *state.current_task_id.lock().map_err(|e| e.to_string())? = Some(task_id.clone());

    // 4. (可选) 更新任务状态为 'in_progress'
    {
        let storage_guard = state.storage.lock().await;
        let storage = storage_guard.as_ref().ok_or("存储未初始化")?;
        let task_update = crate::storage::TaskUpdate {
            status: Some("in_progress".to_string()),
            ..Default::default()
        };
        storage
            .get_database()
            .update_task(task_uuid, &task_update)
            .map_err(|e| e.to_string())?;
    }

    // 5. 获取并返回最新状态
    let status_dto = get_timer_status(state).await?;
    app_handle
        .emit("timer_status_changed", &status_dto)
        .map_err(|e| e.to_string())?;
    Ok(status_dto)
}

/// 停止计时器
#[tauri::command]
pub async fn stop_timer(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<TimerStatusDto, String> {
    let duration;
    let task_id;

    // 1. 停止计时器并获取会话信息
    {
        let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
        duration = timer.stop().map_err(|e| e.to_string())?;
    }

    // 2. 获取并清除当前任务ID
    {
        task_id = state
            .current_task_id
            .lock()
            .map_err(|e| e.to_string())?
            .take();
    }

    // 3. 如果有当前任务，保存时间记录
    if let Some(task_id_str) = task_id {
        let task_uuid = Uuid::parse_str(&task_id_str).map_err(|_| "无效的任务ID")?;
        let storage_guard = state.storage.lock().await;
        let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

        // 从数据库获取任务，以填充 `task_name` 和 `category_id`
        let task = storage
            .get_database()
            .get_task_by_id(task_uuid)
            .map_err(|e| format!("查询任务失败: {}", e))?
            .ok_or("任务不存在")?;

        // 创建时间记录
        let time_entry = crate::storage::models::TimeEntryInsert {
            id: Uuid::new_v4(),
            task_name: task.name.clone(),
            category_id: task.category_id,
            start_time: Local::now() - duration,
            end_time: Some(Local::now()),
            duration_seconds: duration.num_seconds(),
            description: None,
            tags: vec![],
            created_at: Local::now(),
        };

        // 插入时间记录
        storage
            .get_database()
            .insert_time_entry(&time_entry)
            .map_err(|e| format!("保存时间记录失败: {}", e))?;

        log::info!("时间记录已保存，任务: {}", task.name);
    }

    // 4. 获取并返回最新状态
    let status_dto = get_timer_status(state).await?;
    app_handle
        .emit("timer_status_changed", &status_dto)
        .map_err(|e| e.to_string())?;
    Ok(status_dto)
}

/// 暂停计时器
#[tauri::command]
pub async fn pause_timer(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<TimerStatusDto, String> {
    {
        let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
        timer.pause().map_err(|e| e.to_string())?;
    }

    // 只是暂停，不写入数据库，状态由TimerStatusDto反映
    log::info!("计时器已暂停");

    // 获取并返回当前状态
    let status_dto = get_timer_status(state).await?;
    app_handle
        .emit("timer_status_changed", &status_dto)
        .map_err(|e| e.to_string())?;
    Ok(status_dto)
}

/// 获取当前计时器状态
#[tauri::command]
pub async fn get_timer_status(state: State<'_, AppState>) -> Result<TimerStatusDto, String> {
    let (state_str, start_time, pause_time, elapsed_seconds) = {
        let timer = state.timer.lock().unwrap();
        let status = timer.get_state();
        let elapsed_seconds = timer.get_elapsed().num_seconds();

        let (state_str, start_time, pause_time) = match status {
            crate::core::TimerState::Running { start_time, .. } => {
                ("running", Some(*start_time), None)
            }
            crate::core::TimerState::Paused {
                start_time,
                pause_start,
                ..
            } => ("paused", Some(*start_time), Some(*pause_start)),
            crate::core::TimerState::Stopped => ("stopped", None, None),
        };
        (state_str, start_time, pause_time, elapsed_seconds)
    };

    let current_task_id = state.current_task_id.lock().unwrap().clone();
    let mut current_task_name = None;

    if let Some(task_id) = &current_task_id {
        let storage_guard = state.storage.lock().await;
        let storage = storage_guard.as_ref().ok_or("存储未初始化")?;
        if let Ok(uuid) = Uuid::parse_str(task_id) {
            if let Ok(Some(task)) = storage.get_database().get_task_by_id(uuid) {
                current_task_name = Some(task.name);
            }
        }
    }

    Ok(TimerStatusDto {
        state: state_str.to_string(),
        current_task_id,
        current_task_name,
        start_time,
        pause_time,
        elapsed_seconds,
        total_today_seconds: 0, // 这个字段由 get_today_stats 填充
    })
}

/// 获取今日时间记录
#[tauri::command]
pub async fn get_today_time_entries(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    log::debug!("获取今日时间记录");
    let today = Local::now().date_naive();

    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    let entries = storage
        .get_database()
        .get_time_entries_by_date_range(today, today)
        .map_err(|e| format!("查询今日时间记录失败: {}", e))?;

    // 转换为前端需要的格式
    let formatted_entries: Vec<serde_json::Value> = entries
        .iter()
        .map(|entry| {
            serde_json::json!({
                "id": entry.id,
                "task_name": entry.task_name,
                "start_time": entry.start_time.format("%H:%M:%S").to_string(),
                "end_time": entry.end_time.as_ref().map(|dt| dt.format("%H:%M:%S").to_string()),
                "duration_seconds": entry.duration_seconds
            })
        })
        .collect();

    log::info!("返回今日时间记录 {} 条", formatted_entries.len());
    Ok(formatted_entries)
}

/// 调试命令：获取所有时间记录
#[tauri::command]
pub async fn debug_get_time_entries(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    let entries = storage
        .get_database()
        .get_all_time_entries()
        .map_err(|e| e.to_string())?;

    let debug_info: Vec<String> = entries
        .iter()
        .map(|entry| {
            format!(
                "ID: {}, 任务: {}, 开始: {}, 结束: {:?}, 时长: {}秒",
                entry.id,
                entry.task_name,
                entry.start_time.format("%Y-%m-%d %H:%M:%S"),
                entry
                    .end_time
                    .as_ref()
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                entry.duration_seconds
            )
        })
        .collect();

    log::info!("数据库中共有 {} 条时间记录", entries.len());
    Ok(debug_info)
}

/// 获取今日统计信息
#[tauri::command]
pub async fn get_today_stats(state: State<'_, AppState>) -> Result<TimerStatusDto, String> {
    let total_today_seconds = {
        let storage_guard = state.storage.lock().await;
        let storage = storage_guard.as_ref().ok_or("存储未初始化")?;
        get_today_total_seconds(storage)?
    };

    let mut status_dto = get_timer_status(state).await?;

    status_dto.total_today_seconds = total_today_seconds;

    Ok(status_dto)
}

// ========== 分类管理命令 ==========

/// 获取所有分类
#[tauri::command]
pub async fn get_categories(state: State<'_, AppState>) -> Result<Vec<CategoryDto>, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    log::debug!("获取分类列表");

    let categories = storage
        .get_database()
        .get_all_categories()
        .map_err(|e| format!("获取分类失败: {}", e))?;

    // 获取每个分类的任务数量统计
    let task_counts = storage
        .get_database()
        .get_category_task_counts()
        .map_err(|e| format!("获取任务统计失败: {}", e))?;

    // 获取每个分类的时长统计
    let duration_stats = storage
        .get_database()
        .get_category_duration_stats()
        .map_err(|e| format!("获取时长统计失败: {}", e))?;

    // 转换为 CategoryDto
    let mut category_dtos = Vec::new();
    for category in categories {
        let task_count = task_counts.get(&category.id).unwrap_or(&0);
        let total_duration = duration_stats.get(&category.id).unwrap_or(&0);

        let category_dto = CategoryDto {
            id: category.id.to_string(),
            name: category.name,
            description: category.description,
            color: category.color,
            icon: Some(category.icon),
            is_active: category.is_active,
            task_count: *task_count as u32,
            total_duration_seconds: *total_duration,
            created_at: category.created_at,
            updated_at: category.updated_at.unwrap_or(category.created_at),
        };

        category_dtos.push(category_dto);
    }

    log::debug!("获取到 {} 个分类", category_dtos.len());
    Ok(category_dtos)
}

/// 创建新分类
#[tauri::command]
pub async fn create_category(
    state: State<'_, AppState>,
    request: CreateCategoryRequest,
) -> Result<CategoryDto, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    use crate::storage::models::CategoryInsert;

    let category_id = Uuid::new_v4();
    let created_at = Local::now();

    let category_insert = CategoryInsert {
        id: category_id,
        name: request.name.clone(),
        description: request.description.clone(),
        color: request.color.clone().unwrap_or("#6c757d".to_string()),
        icon: request.icon.clone().unwrap_or("folder".to_string()),
        daily_target_seconds: None,
        weekly_target_seconds: None,
        is_active: true,
        sort_order: 0,
        parent_id: None,
        created_at,
    };

    // 插入到数据库
    storage
        .get_database()
        .insert_category(&category_insert)
        .map_err(|e| format!("创建分类失败: {}", e))?;

    // 返回创建的分类
    let category_dto = CategoryDto {
        id: category_id.to_string(),
        name: request.name,
        description: request.description,
        color: request.color.unwrap_or("#6c757d".to_string()),
        icon: request.icon,
        is_active: true,
        task_count: 0,
        total_duration_seconds: 0,
        created_at,
        updated_at: created_at,
    };

    log::debug!("创建分类成功: {}", category_dto.name);
    Ok(category_dto)
}

/// 更新分类
#[tauri::command]
pub async fn update_category(
    state: State<'_, AppState>,
    category_id: String,
    request: UpdateCategoryRequest,
) -> Result<CategoryDto, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    use crate::storage::models::CategoryInsert;

    let uuid = Uuid::parse_str(&category_id).map_err(|e| format!("无效的分类ID: {}", e))?;

    log::debug!("更新分类: {}", category_id);

    // 获取现有分类
    let existing_category = storage
        .get_database()
        .get_category_by_id(uuid)
        .map_err(|e| format!("获取分类失败: {}", e))?
        .ok_or("分类不存在")?;

    // 构建更新数据
    let category_insert = CategoryInsert {
        id: uuid,
        name: request.name.unwrap_or(existing_category.name.clone()),
        description: request
            .description
            .or(existing_category.description.clone()),
        color: request.color.unwrap_or(existing_category.color.clone()),
        icon: request.icon.unwrap_or(existing_category.icon.clone()),
        daily_target_seconds: existing_category.daily_target_seconds,
        weekly_target_seconds: existing_category.weekly_target_seconds,
        is_active: request.is_active.unwrap_or(existing_category.is_active),
        sort_order: existing_category.sort_order,
        parent_id: existing_category.parent_id,
        created_at: existing_category.created_at,
    };

    // 更新数据库
    storage
        .get_database()
        .update_category(uuid, &category_insert)
        .map_err(|e| format!("更新分类失败: {}", e))?;

    // 获取任务数量和时长统计
    let task_counts = storage
        .get_database()
        .get_category_task_counts()
        .map_err(|e| format!("获取任务统计失败: {}", e))?;

    let duration_stats = storage
        .get_database()
        .get_category_duration_stats()
        .map_err(|e| format!("获取时长统计失败: {}", e))?;

    let task_count = task_counts.get(&uuid).unwrap_or(&0);
    let total_duration = duration_stats.get(&uuid).unwrap_or(&0);

    // 返回更新后的分类
    let category_dto = CategoryDto {
        id: category_id,
        name: category_insert.name,
        description: category_insert.description,
        color: category_insert.color,
        icon: Some(category_insert.icon),
        is_active: category_insert.is_active,
        task_count: *task_count as u32,
        total_duration_seconds: *total_duration,
        created_at: category_insert.created_at,
        updated_at: Local::now(),
    };

    log::debug!("更新分类成功: {}", category_dto.name);
    Ok(category_dto)
}

/// 删除分类
#[tauri::command]
pub async fn delete_category(
    state: State<'_, AppState>,
    category_id: String,
) -> Result<bool, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    let uuid = Uuid::parse_str(&category_id).map_err(|e| format!("无效的分类ID: {}", e))?;

    log::debug!("删除分类: {}", category_id);

    // 检查分类是否存在
    let category = storage
        .get_database()
        .get_category_by_id(uuid)
        .map_err(|e| format!("获取分类失败: {}", e))?
        .ok_or("分类不存在")?;

    // 删除分类
    storage
        .get_database()
        .delete_category(uuid)
        .map_err(|e| format!("删除分类失败: {}", e))?;

    log::debug!("删除分类成功: {}", category.name);
    Ok(true)
}

// ========== 统计分析命令 ==========

/// 获取统计数据
#[tauri::command]
pub async fn get_statistics(
    state: State<'_, AppState>,
    period: Option<String>, // "today", "week", "month", "all"
) -> Result<StatisticsDto, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    // TODO: 实现实际的统计数据查询逻辑
    let stats = StatisticsDto {
        today: DayStatDto {
            date: Local::now().format("%Y-%m-%d").to_string(),
            total_seconds: 7200, // 2小时
            task_count: 3,
            active_categories: 2,
            most_productive_hour: Some(14), // 下午2点
        },
        this_week: PeriodStatDto {
            total_seconds: 36000, // 10小时
            task_count: 15,
            active_days: 5,
            average_daily_seconds: 7200,
        },
        this_month: PeriodStatDto {
            total_seconds: 144000, // 40小时
            task_count: 60,
            active_days: 20,
            average_daily_seconds: 7200,
        },
        all_time: PeriodStatDto {
            total_seconds: 720000, // 200小时
            task_count: 300,
            active_days: 100,
            average_daily_seconds: 7200,
        },
        category_stats: vec![
            CategoryStatDto {
                category_id: "cat1".to_string(),
                category_name: "工作".to_string(),
                total_seconds: 18000,
                task_count: 10,
                percentage: 60.0,
            },
            CategoryStatDto {
                category_id: "cat2".to_string(),
                category_name: "学习".to_string(),
                total_seconds: 12000,
                task_count: 8,
                percentage: 40.0,
            },
        ],
        daily_trend: vec![], // TODO: 添加7天的趋势数据
    };

    Ok(stats)
}

/// 获取财务统计
#[tauri::command]
pub async fn get_financial_stats(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<FinancialStatsDto, String> {
    log::debug!(
        "[CMD] get_financial_stats: Attempting for {} to {}",
        start_date,
        end_date
    );

    let start_date_naive = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|_| "无效的开始日期格式")?;
    let end_date_naive = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|_| "无效的结束日期格式")?;

    let stats_from_db = {
        let storage_guard = state.storage.lock().await;
        let storage = storage_guard.as_ref().ok_or("存储未初始化")?;
        log::debug!("[CMD] get_financial_stats: Lock acquired, fetching from DB.");

        storage
            .get_database()
            .get_transaction_statistics(start_date_naive, end_date_naive)
            .map_err(|e| e.to_string())
    }?;
    log::debug!("[CMD] get_financial_stats: Lock released, mapping DTO.");

    let stats_dto = FinancialStatsDto {
        total_income: stats_from_db.total_income,
        total_expense: stats_from_db.total_expense,
        net_income: stats_from_db.net_income,
        account_balance: stats_from_db.account_balance,
        transaction_count: stats_from_db.transaction_count,
        period_start: stats_from_db.period_start.format("%Y-%m-%d").to_string(),
        period_end: stats_from_db.period_end.format("%Y-%m-%d").to_string(),
        currency: stats_from_db.currency,
    };

    log::debug!("财务统计获取成功");
    Ok(stats_dto)
}

/// 从数据库计算今日总时长
fn get_today_total_seconds(storage: &StorageManager) -> Result<i64, String> {
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

// ========== 数据导入导出命令 ==========

/// 导出数据
#[tauri::command]
pub async fn export_data(
    state: State<'_, AppState>,
    format: String, // "json", "csv", "xml"
    file_path: String,
) -> Result<String, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    log::info!("导出数据到: {}，格式: {}", file_path, format);

    let result = match format.as_str() {
        "json" => {
            // TODO: 实现实际的数据导出逻辑
            Ok(format!("数据已导出到: {}", file_path))
        }
        "csv" => {
            // TODO: 实现CSV格式导出逻辑
            Err("CSV格式导出功能待实现".to_string())
        }
        "xml" => {
            // TODO: 实现XML格式导出逻辑
            Err("XML格式导出功能待实现".to_string())
        }
        _ => Err(format!("不支持的导出格式: {}", format)),
    };

    result
}

/// 导入数据
#[tauri::command]
pub async fn import_data(state: State<'_, AppState>, file_path: String) -> Result<String, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    log::info!("从 {} 导入数据", file_path);

    // TODO: 实现实际的数据导入逻辑
    Ok(format!("数据已从文件导入: {}", file_path))
}

// ========== 配置管理命令 ==========

/// 获取应用配置
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().unwrap().clone();
    Ok(config)
}

/// 更新应用配置
#[tauri::command]
pub async fn update_config(state: State<'_, AppState>, config: AppConfig) -> Result<bool, String> {
    let mut state_config = state.config.lock().unwrap();
    *state_config = config;
    Ok(true)
}

/// 设置窗口主题背景色
#[tauri::command]
pub async fn set_window_theme(app_handle: AppHandle, is_dark: bool) -> Result<(), String> {
    use tauri::window::Color;

    let bg_color = if is_dark {
        Color(0, 0, 0, 255) // 暗色模式纯黑 #000000
    } else {
        Color(249, 250, 251, 255) // 亮色模式背景 #f9fafb (gray-50)
    };

    if let Some(window) = app_handle.get_webview_window("main") {
        window
            .set_background_color(Some(bg_color))
            .map_err(|e| format!("设置窗口背景色失败: {}", e))?;

        log::info!("窗口背景色已更新为: {:?} (暗色模式: {})", bg_color, is_dark);
    }

    Ok(())
}

// ========== 记账功能命令 ==========

/// 账户数据传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDto {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub balance: f64,
    pub initial_balance: f64,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_default: bool,
    pub created_at: DateTime<Local>,
    pub updated_at: Option<DateTime<Local>>,
}

/// 交易数据传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDto {
    pub id: String,
    pub transaction_type: String,
    pub amount: f64,
    pub currency: String,
    pub description: String,
    pub account_id: String,
    pub account_name: Option<String>,
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub to_account_id: Option<String>,
    pub to_account_name: Option<String>,
    pub status: String,
    pub transaction_date: String,
    pub tags: Vec<String>,
    pub receipt_path: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: Option<DateTime<Local>>,
}

/// 交易分类数据传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCategoryDto {
    pub id: String,
    pub name: String,
    pub transaction_type: String,
    pub color: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Local>,
    pub updated_at: Option<DateTime<Local>>,
}

/// 预算数据传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetDto {
    pub id: String,
    pub name: String,
    pub category_id: String,
    pub category_name: Option<String>,
    pub amount: f64,
    pub currency: String,
    pub period: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub is_active: bool,
    pub description: Option<String>,
    pub spent_amount: f64,
    pub usage_percentage: f64,
    pub status: String,
    pub created_at: DateTime<Local>,
    pub updated_at: Option<DateTime<Local>>,
}

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

/// 创建账户请求
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub account_type: String,
    pub currency: Option<String>,
    pub initial_balance: Option<f64>,
    pub description: Option<String>,
    pub is_default: Option<bool>,
}

/// 更新账户请求
#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub account_type: Option<String>,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub is_default: Option<bool>,
}

/// 创建交易请求
#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub transaction_type: String,
    pub amount: f64,
    pub description: String,
    pub account_id: String,
    pub category_id: Option<String>,
    pub to_account_id: Option<String>,
    pub transaction_date: Option<String>,
    pub tags: Option<Vec<String>>,
    pub receipt_path: Option<String>,
}

/// 更新交易请求
#[derive(Debug, Deserialize)]
pub struct UpdateTransactionRequest {
    pub transaction_type: Option<String>,
    pub amount: Option<f64>,
    pub description: Option<String>,
    pub account_id: Option<String>,
    pub category_id: Option<String>,
    pub to_account_id: Option<String>,
    pub transaction_date: Option<String>,
    pub tags: Option<Vec<String>>,
    pub receipt_path: Option<String>,
    pub status: Option<String>,
}

/// 创建交易分类请求
#[derive(Debug, Deserialize)]
pub struct CreateTransactionCategoryRequest {
    pub name: String,
    pub transaction_type: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<String>,
}

/// 更新交易分类请求
#[derive(Debug, Deserialize)]
pub struct UpdateTransactionCategoryRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub is_active: Option<bool>,
}

/// 创建预算请求
#[derive(Debug, Deserialize)]
pub struct CreateBudgetRequest {
    pub name: String,
    pub category_id: String,
    pub amount: f64,
    pub period: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub currency: Option<String>,
    pub description: Option<String>,
}

/// 更新预算请求
#[derive(Debug, Deserialize)]
pub struct UpdateBudgetRequest {
    pub name: Option<String>,
    pub amount: Option<f64>,
    pub period: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

/// 交易查询请求
#[derive(Debug, Deserialize)]
pub struct TransactionQueryRequest {
    pub account_id: Option<String>,
    pub category_id: Option<String>,
    pub transaction_type: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub search: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

// ========== 账户管理命令 ==========

/// 获取所有账户
#[tauri::command]
pub async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<AccountDto>, String> {
    log::debug!("[CMD] get_accounts: Attempting to get accounts.");

    // 在作用域内获取数据，然后立即释放锁
    let accounts_from_db = {
        let storage_guard = state.storage.lock().await;
        let storage = storage_guard.as_ref().ok_or("存储未初始化")?;
        log::debug!("[CMD] get_accounts: Lock acquired, fetching from DB.");

        storage
            .get_database()
            .get_all_accounts()
            .map_err(|e| e.to_string())?
    };
    log::debug!("[CMD] get_accounts: Lock released, mapping DTOs.");

    let account_dtos: Vec<AccountDto> = accounts_from_db
        .into_iter()
        .map(|account| AccountDto {
            id: account.id.to_string(),
            name: account.name,
            account_type: format!("{:?}", account.account_type).to_lowercase(),
            currency: account.currency,
            balance: account.balance,
            initial_balance: account.initial_balance,
            description: account.description,
            is_active: account.is_active,
            is_default: account.is_default,
            created_at: account.created_at,
            updated_at: account.updated_at,
        })
        .collect();

    log::debug!("返回 {} 个账户", account_dtos.len());
    Ok(account_dtos)
}

/// 创建账户
#[tauri::command]
pub async fn create_account(
    state: State<'_, AppState>,
    request: CreateAccountRequest,
) -> Result<AccountDto, String> {
    log::info!(
        "[CMD] create_account: Received request for name '{}'",
        request.name
    );

    // 解析账户类型
    let account_type = match request.account_type.as_str() {
        "cash" => crate::storage::AccountType::Cash,
        "bank" => crate::storage::AccountType::Bank,
        "creditcard" | "credit_card" => crate::storage::AccountType::CreditCard,
        "investment" => crate::storage::AccountType::Investment,
        "other" => crate::storage::AccountType::Other,
        _ => return Err("无效的账户类型".to_string()),
    };
    log::info!("[CMD] create_account: Parsed account type.");

    let created_at = Local::now();
    let account_id = Uuid::new_v4();

    let account_insert = crate::storage::AccountInsert {
        id: account_id,
        name: request.name.clone(),
        account_type,
        currency: request.currency.unwrap_or_else(|| "CNY".to_string()),
        balance: request.initial_balance.unwrap_or(0.0),
        initial_balance: request.initial_balance.unwrap_or(0.0),
        description: request.description.clone(),
        is_active: true,
        is_default: request.is_default.unwrap_or(false),
        created_at,
    };
    log::info!("[CMD] create_account: Prepared account insert data.");

    // 在作用域内执行数据库操作，然后立即释放锁
    let insert_result = {
        let storage_guard = state.storage.lock().await;
        log::info!("[CMD] create_account: Acquired storage lock.");
        let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

        // 如果设置为默认账户，先取消其他账户的默认状态
        if account_insert.is_default {
            // 这里需要额外的逻辑来处理默认账户，暂时跳过
            log::warn!("设置默认账户功能待实现");
        }

        let result = storage.get_database().insert_account(&account_insert);

        log::info!(
            "[CMD] create_account: Database insert result: {:?}",
            result
                .as_ref()
                .map(|_| "Success")
                .map_err(|e| e.to_string())
        );

        result.map_err(|e| e.to_string())?
    };
    log::info!("[CMD] create_account: Database lock released.");

    let account_dto = AccountDto {
        id: account_id.to_string(),
        name: account_insert.name,
        account_type: format!("{:?}", account_insert.account_type).to_lowercase(),
        currency: account_insert.currency,
        balance: account_insert.balance,
        initial_balance: account_insert.initial_balance,
        description: account_insert.description,
        is_active: account_insert.is_active,
        is_default: account_insert.is_default,
        created_at: account_insert.created_at,
        updated_at: None,
    };

    log::info!("账户创建成功: {}", account_dto.id);
    Ok(account_dto)
}

/// 更新账户
#[tauri::command]
pub async fn update_account(
    state: State<'_, AppState>,
    account_id: String,
    request: UpdateAccountRequest,
) -> Result<AccountDto, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    let uuid = Uuid::parse_str(&account_id).map_err(|_| "无效的账户ID")?;

    log::debug!("更新账户: {}", account_id);

    // 构建更新请求
    let account_type = if let Some(type_str) = &request.account_type {
        Some(match type_str.as_str() {
            "cash" => crate::storage::AccountType::Cash,
            "bank" => crate::storage::AccountType::Bank,
            "creditcard" | "credit_card" => crate::storage::AccountType::CreditCard,
            "investment" => crate::storage::AccountType::Investment,
            "other" => crate::storage::AccountType::Other,
            _ => return Err("无效的账户类型".to_string()),
        })
    } else {
        None
    };

    let account_update = crate::storage::AccountUpdate {
        name: request.name,
        account_type,
        currency: request.currency,
        description: Some(request.description.clone()),
        is_active: request.is_active,
        is_default: request.is_default,
    };

    storage
        .get_database()
        .update_account(uuid, &account_update)
        .map_err(|e| e.to_string())?;

    // 获取更新后的账户
    let updated_account = storage
        .get_database()
        .get_account_by_id(uuid)
        .map_err(|e| e.to_string())?
        .ok_or("账户不存在")?;

    let account_dto = AccountDto {
        id: updated_account.id.to_string(),
        name: updated_account.name,
        account_type: format!("{:?}", updated_account.account_type).to_lowercase(),
        currency: updated_account.currency,
        balance: updated_account.balance,
        initial_balance: updated_account.initial_balance,
        description: updated_account.description,
        is_active: updated_account.is_active,
        is_default: updated_account.is_default,
        created_at: updated_account.created_at,
        updated_at: updated_account.updated_at,
    };

    log::info!("账户更新成功: {}", account_id);
    Ok(account_dto)
}

/// 删除账户
#[tauri::command]
pub async fn delete_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<bool, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    let uuid = Uuid::parse_str(&account_id).map_err(|_| "无效的账户ID")?;

    log::debug!("删除账户: {}", account_id);

    storage
        .get_database()
        .delete_account(uuid)
        .map_err(|e| e.to_string())?;

    log::info!("账户删除成功: {}", account_id);
    Ok(true)
}

// ========== 交易管理命令 ==========

/// 获取交易列表
#[tauri::command]
pub async fn get_transactions(
    state: State<'_, AppState>,
    query: Option<TransactionQueryRequest>,
) -> Result<Vec<TransactionDto>, String> {
    log::debug!("[CMD] get_transactions: Attempting to get transactions.");

    let (transactions, accounts) = {
        let storage_guard = state.storage.lock().await;
        let storage = storage_guard.as_ref().ok_or("存储未初始化")?;
        log::debug!("[CMD] get_transactions: Lock acquired, fetching from DB.");

        let transactions_res = if let Some(q) = query {
            if let (Some(start), Some(end)) = (&q.start_date, &q.end_date) {
                let start_date = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d")
                    .map_err(|_| "无效的开始日期格式")?;
                let end_date = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d")
                    .map_err(|_| "无效的结束日期格式")?;

                storage
                    .get_database()
                    .get_transactions_by_date_range(start_date, end_date)
            } else if let Some(account_id_str) = &q.account_id {
                let account_id = Uuid::parse_str(account_id_str).map_err(|_| "无效的账户ID")?;
                storage
                    .get_database()
                    .get_transactions_by_account(account_id)
            } else {
                storage.get_database().get_all_transactions()
            }
        } else {
            storage.get_database().get_all_transactions()
        }
        .map_err(|e| e.to_string())?;

        let accounts_res = storage
            .get_database()
            .get_all_accounts()
            .map_err(|e| e.to_string())?;

        (transactions_res, accounts_res)
    };
    log::debug!("[CMD] get_transactions: Lock released, mapping DTOs.");

    let mut transaction_dtos = Vec::new();
    for transaction in transactions {
        // 获取账户名称
        let account_name = accounts
            .iter()
            .find(|a| a.id == transaction.account_id)
            .map(|a| a.name.clone());

        let to_account_name = if let Some(to_id) = transaction.to_account_id {
            accounts
                .iter()
                .find(|a| a.id == to_id)
                .map(|a| a.name.clone())
        } else {
            None
        };

        // TODO: 获取分类名称（需要实现交易分类查询）
        let category_name = None;

        let transaction_dto = TransactionDto {
            id: transaction.id.to_string(),
            transaction_type: format!("{:?}", transaction.transaction_type).to_lowercase(),
            amount: transaction.amount,
            currency: transaction.currency,
            description: transaction.description,
            account_id: transaction.account_id.to_string(),
            account_name,
            category_id: transaction.category_id.map(|id| id.to_string()),
            category_name,
            to_account_id: transaction.to_account_id.map(|id| id.to_string()),
            to_account_name,
            status: format!("{:?}", transaction.status).to_lowercase(),
            transaction_date: transaction.transaction_date.format("%Y-%m-%d").to_string(),
            tags: transaction.tags,
            receipt_path: transaction.receipt_path,
            created_at: transaction.created_at,
            updated_at: transaction.updated_at,
        };

        transaction_dtos.push(transaction_dto);
    }

    log::debug!("返回 {} 个交易记录", transaction_dtos.len());
    Ok(transaction_dtos)
}

/// 创建交易
#[tauri::command]
pub async fn create_transaction(
    state: State<'_, AppState>,
    request: CreateTransactionRequest,
) -> Result<TransactionDto, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    log::debug!("创建交易: {}", request.description);

    // 解析交易类型
    let transaction_type = match request.transaction_type.as_str() {
        "income" => crate::storage::TransactionType::Income,
        "expense" => crate::storage::TransactionType::Expense,
        "transfer" => crate::storage::TransactionType::Transfer,
        _ => return Err("无效的交易类型".to_string()),
    };

    // 解析日期
    let transaction_date = if let Some(date_str) = &request.transaction_date {
        chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| "无效的日期格式")?
    } else {
        Local::now().date_naive()
    };

    let account_id = Uuid::parse_str(&request.account_id).map_err(|_| "无效的账户ID")?;
    let category_id = if let Some(cat_id_str) = &request.category_id {
        Some(Uuid::parse_str(cat_id_str).map_err(|_| "无效的分类ID")?)
    } else {
        None
    };
    let to_account_id = if let Some(to_id_str) = &request.to_account_id {
        Some(Uuid::parse_str(to_id_str).map_err(|_| "无效的目标账户ID")?)
    } else {
        None
    };

    let transaction_insert = crate::storage::TransactionInsert {
        id: Uuid::new_v4(),
        transaction_type,
        amount: request.amount,
        currency: "CNY".to_string(), // 暂时使用默认货币
        description: request.description,
        account_id,
        category_id,
        to_account_id,
        status: crate::storage::TransactionStatus::Completed,
        transaction_date,
        tags: request.tags.unwrap_or_default(),
        receipt_path: request.receipt_path,
        created_at: Local::now(),
    };

    storage
        .get_database()
        .insert_transaction(&transaction_insert)
        .map_err(|e| e.to_string())?;

    // 更新账户余额
    match transaction_type {
        crate::storage::TransactionType::Income => {
            let current_account = storage
                .get_database()
                .get_account_by_id(account_id)
                .map_err(|e| e.to_string())?
                .ok_or("账户不存在")?;

            let new_balance = current_account.balance + request.amount;
            storage
                .get_database()
                .update_account_balance(account_id, new_balance)
                .map_err(|e| e.to_string())?;
        }
        crate::storage::TransactionType::Expense => {
            let current_account = storage
                .get_database()
                .get_account_by_id(account_id)
                .map_err(|e| e.to_string())?
                .ok_or("账户不存在")?;

            let new_balance = current_account.balance - request.amount;
            storage
                .get_database()
                .update_account_balance(account_id, new_balance)
                .map_err(|e| e.to_string())?;
        }
        crate::storage::TransactionType::Transfer => {
            if let Some(to_id) = to_account_id {
                // 从源账户扣减
                let source_account = storage
                    .get_database()
                    .get_account_by_id(account_id)
                    .map_err(|e| e.to_string())?
                    .ok_or("源账户不存在")?;

                let source_new_balance = source_account.balance - request.amount;
                storage
                    .get_database()
                    .update_account_balance(account_id, source_new_balance)
                    .map_err(|e| e.to_string())?;

                // 向目标账户增加
                let target_account = storage
                    .get_database()
                    .get_account_by_id(to_id)
                    .map_err(|e| e.to_string())?
                    .ok_or("目标账户不存在")?;

                let target_new_balance = target_account.balance + request.amount;
                storage
                    .get_database()
                    .update_account_balance(to_id, target_new_balance)
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    // 获取账户名称用于返回
    let account = storage
        .get_database()
        .get_account_by_id(account_id)
        .map_err(|e| e.to_string())?
        .ok_or("账户不存在")?;

    let to_account_name = if let Some(to_id) = to_account_id {
        storage
            .get_database()
            .get_account_by_id(to_id)
            .map_err(|e| e.to_string())?
            .map(|a| a.name)
    } else {
        None
    };

    let transaction_dto = TransactionDto {
        id: transaction_insert.id.to_string(),
        transaction_type: format!("{:?}", transaction_insert.transaction_type).to_lowercase(),
        amount: transaction_insert.amount,
        currency: transaction_insert.currency,
        description: transaction_insert.description,
        account_id: transaction_insert.account_id.to_string(),
        account_name: Some(account.name),
        category_id: transaction_insert.category_id.map(|id| id.to_string()),
        category_name: None, // TODO: 查询分类名称
        to_account_id: transaction_insert.to_account_id.map(|id| id.to_string()),
        to_account_name,
        status: format!("{:?}", transaction_insert.status).to_lowercase(),
        transaction_date: transaction_insert
            .transaction_date
            .format("%Y-%m-%d")
            .to_string(),
        tags: transaction_insert.tags,
        receipt_path: transaction_insert.receipt_path,
        created_at: transaction_insert.created_at,
        updated_at: None,
    };

    log::info!("交易创建成功: {}", transaction_dto.id);
    Ok(transaction_dto)
}

/// 更新交易
#[tauri::command]
pub async fn update_transaction(
    state: State<'_, AppState>,
    transaction_id: String,
    request: UpdateTransactionRequest,
) -> Result<TransactionDto, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    let uuid = Uuid::parse_str(&transaction_id).map_err(|_| "无效的交易ID")?;

    log::debug!("更新交易: {}", transaction_id);

    // 构建更新请求
    let transaction_type = if let Some(type_str) = &request.transaction_type {
        Some(match type_str.as_str() {
            "income" => crate::storage::TransactionType::Income,
            "expense" => crate::storage::TransactionType::Expense,
            "transfer" => crate::storage::TransactionType::Transfer,
            _ => return Err("无效的交易类型".to_string()),
        })
    } else {
        None
    };

    let status = if let Some(status_str) = &request.status {
        Some(match status_str.as_str() {
            "pending" => crate::storage::TransactionStatus::Pending,
            "completed" => crate::storage::TransactionStatus::Completed,
            "cancelled" => crate::storage::TransactionStatus::Cancelled,
            _ => return Err("无效的交易状态".to_string()),
        })
    } else {
        None
    };

    let transaction_date = if let Some(date_str) = &request.transaction_date {
        Some(
            chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|_| "无效的日期格式")?,
        )
    } else {
        None
    };

    let account_id = if let Some(id_str) = &request.account_id {
        Some(Uuid::parse_str(id_str).map_err(|_| "无效的账户ID")?)
    } else {
        None
    };

    let category_id = if let Some(cat_id_str) = &request.category_id {
        Some(Some(
            Uuid::parse_str(cat_id_str).map_err(|_| "无效的分类ID")?,
        ))
    } else {
        None
    };

    let to_account_id = if let Some(to_id_str) = &request.to_account_id {
        Some(Some(
            Uuid::parse_str(to_id_str).map_err(|_| "无效的目标账户ID")?,
        ))
    } else {
        None
    };

    let transaction_update = crate::storage::TransactionUpdate {
        transaction_type,
        amount: request.amount,
        currency: None, // 暂不支持修改货币
        description: request.description.clone(),
        account_id,
        category_id,
        to_account_id,
        status,
        transaction_date,
        tags: request.tags.clone(),
        receipt_path: Some(request.receipt_path.clone()),
    };

    storage
        .get_database()
        .update_transaction(uuid, &transaction_update)
        .map_err(|e| e.to_string())?;

    // 获取更新后的交易（需要重新实现，因为数据库方法可能不返回完整对象）
    // 这里暂时返回一个简单的响应
    let mock_dto = TransactionDto {
        id: transaction_id.clone(),
        transaction_type: request.transaction_type.unwrap_or_default(),
        amount: request.amount.unwrap_or(0.0),
        currency: "CNY".to_string(),
        description: request.description.unwrap_or_default(),
        account_id: request.account_id.unwrap_or_default(),
        account_name: None,
        category_id: None,
        category_name: None,
        to_account_id: None,
        to_account_name: None,
        status: request.status.unwrap_or("completed".to_string()),
        transaction_date: request
            .transaction_date
            .unwrap_or_else(|| Local::now().date_naive().format("%Y-%m-%d").to_string()),
        tags: request.tags.unwrap_or_default(),
        receipt_path: request.receipt_path,
        created_at: Local::now(),
        updated_at: Some(Local::now()),
    };

    log::info!("交易更新成功: {}", transaction_id);
    Ok(mock_dto)
}

/// 删除交易
#[tauri::command]
pub async fn delete_transaction(
    state: State<'_, AppState>,
    transaction_id: String,
) -> Result<bool, String> {
    let storage_guard = state.storage.lock().await;
    let storage = storage_guard.as_ref().ok_or("存储未初始化")?;

    let uuid = Uuid::parse_str(&transaction_id).map_err(|_| "无效的交易ID")?;

    log::debug!("删除交易: {}", transaction_id);

    storage
        .get_database()
        .delete_transaction(uuid)
        .map_err(|e| e.to_string())?;

    log::info!("交易删除成功: {}", transaction_id);
    Ok(true)
}
