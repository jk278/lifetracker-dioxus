//! # Tauri 命令处理模块
//!
//! 提供前端调用的所有后端API命令

use crate::{config::AppConfig, core::Timer, storage::StorageManager};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_notification::NotificationExt;
use uuid::Uuid;

/// 应用状态
#[derive(Default)]
pub struct AppState {
    pub storage: Arc<Mutex<Option<StorageManager>>>,
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
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    log::debug!(
        "获取任务列表 - limit: {:?}, offset: {:?}, category_id: {:?}",
        limit,
        offset,
        category_id
    );

    // 从数据库获取任务
    let tasks = if let Some(cat_id_str) = category_id {
        if let Ok(cat_id) = Uuid::parse_str(&cat_id_str) {
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

    log::debug!("从数据库获取到 {} 个任务", tasks.len());

    // 转换为TaskDto
    let mut task_dtos = Vec::new();
    for task in tasks {
        // 获取分类名称
        let category_name = if let Some(cat_id) = task.category_id {
            match storage.get_database().get_category_by_id(cat_id) {
                Ok(Some(category)) => Some(category.name),
                _ => None,
            }
        } else {
            None
        };

        // 解析标签
        let tags: Vec<String> = serde_json::from_str(&task.tags).unwrap_or_default();

        task_dtos.push(TaskDto {
            id: task.id.to_string(),
            name: task.name,
            description: task.description,
            category_id: task.category_id.map(|id| id.to_string()),
            category_name,
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

    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_mut().ok_or("存储未初始化")?;

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
        tags: tags_json,
        due_date: None,
        is_completed: false,
        completed_at: None,
        created_at: chrono::Local::now(),
    };

    log::debug!("任务插入模型: {:?}", task_insert);

    // 插入到数据库
    let insert_result = storage
        .get_database()
        .insert_task(&task_insert)
        .map_err(|e| format!("创建任务失败: {}", e))?;

    log::info!(
        "任务创建成功: {} (ID: {}, 插入行ID: {})",
        request.name,
        task_id,
        insert_result
    );

    // 获取分类名称
    let category_name = if let Some(cat_id) = task_insert.category_id {
        match storage.get_database().get_category_by_id(cat_id) {
            Ok(Some(category)) => Some(category.name),
            _ => None,
        }
    } else {
        None
    };

    // 解析标签
    let tags: Vec<String> = serde_json::from_str(&task_insert.tags).unwrap_or_default();

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
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_mut().ok_or("存储未初始化")?;

    let task_uuid = Uuid::parse_str(&task_id).map_err(|_| "无效的任务ID格式".to_string())?;

    log::info!("更新任务: {}", task_id);

    // 验证任务是否存在
    let existing_task = storage
        .get_database()
        .get_task_by_id(task_uuid)
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
        tags: tags_json,
        due_date: None,
        is_completed: None,
        completed_at: None,
    };

    // 更新数据库
    storage
        .get_database()
        .update_task(task_uuid, &task_update)
        .map_err(|e| format!("更新任务失败: {}", e))?;

    log::info!("任务更新成功: {}", task_id);

    // 获取更新后的任务
    let updated_task = storage
        .get_database()
        .get_task_by_id(task_uuid)
        .map_err(|e| format!("获取更新后任务失败: {}", e))?
        .ok_or("更新后任务不存在".to_string())?;

    // 获取分类名称
    let category_name = if let Some(cat_id) = updated_task.category_id {
        match storage.get_database().get_category_by_id(cat_id) {
            Ok(Some(category)) => Some(category.name),
            _ => None,
        }
    } else {
        None
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
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_mut().ok_or("存储未初始化")?;

    let task_uuid = Uuid::parse_str(&task_id).map_err(|_| "无效的任务ID格式".to_string())?;

    log::info!("删除任务: {}", task_id);

    // 验证任务是否存在
    let _existing_task = storage
        .get_database()
        .get_task_by_id(task_uuid)
        .map_err(|e| format!("查询任务失败: {}", e))?
        .ok_or("任务不存在".to_string())?;

    // 删除任务
    storage
        .get_database()
        .delete_task(task_uuid)
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
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    // 验证任务是否存在
    let task_uuid = uuid::Uuid::parse_str(&task_id).map_err(|_| "无效的任务ID格式".to_string())?;
    let task = storage
        .get_database()
        .get_task_by_id(task_uuid)
        .map_err(|e| format!("查询任务失败: {}", e))?
        .ok_or("任务不存在".to_string())?;

    // 启动计时器
    timer
        .start()
        .map_err(|e| format!("启动计时器失败: {}", e))?;

    // 设置当前任务ID
    {
        let mut current_task = state.current_task_id.lock().map_err(|e| e.to_string())?;
        *current_task = Some(task_id.clone());
    }

    let start_time = Local::now();
    log::info!("开始计时任务: {} ({})", task.name, task_id);

    // 发送通知
    let _ = app_handle.emit("timer-started", &task_id);

    // 显示系统通知
    let _ = app_handle
        .notification()
        .builder()
        .title("计时器已启动")
        .body(&format!("开始记录任务: {}", task.name))
        .show();

    Ok(TimerStatusDto {
        state: "running".to_string(),
        current_task_id: Some(task_id),
        current_task_name: Some(task.name),
        start_time: Some(start_time),
        pause_time: None,
        elapsed_seconds: 0,
        total_today_seconds: get_today_total_seconds(&storage).unwrap_or(0),
    })
}

/// 停止计时
#[tauri::command]
pub async fn stop_timer(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<TimerStatusDto, String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    // 获取当前任务信息
    let current_task_id = {
        let current_task = state.current_task_id.lock().map_err(|e| e.to_string())?;
        current_task.clone()
    };

    // 停止计时器并获取时长
    let duration = timer.stop().map_err(|e| format!("停止计时器失败: {}", e))?;
    let elapsed_seconds = duration.num_seconds();

    // 如果有当前任务，保存时间记录
    if let Some(task_id_str) = &current_task_id {
        if let Ok(task_uuid) = uuid::Uuid::parse_str(task_id_str) {
            if let Ok(Some(task)) = storage.get_database().get_task_by_id(task_uuid) {
                // 创建时间记录
                let time_entry = crate::storage::models::TimeEntryInsert {
                    id: uuid::Uuid::new_v4(),
                    task_name: task.name.clone(),
                    category_id: task.category_id, // 使用任务的分类ID，而不是任务ID
                    start_time: Local::now() - duration, // 开始时间 = 当前时间 - 持续时间
                    end_time: Some(Local::now()),
                    duration_seconds: elapsed_seconds,
                    description: Some(format!("自动记录的计时会话")),
                    tags: vec![],
                    created_at: Local::now(),
                };

                log::info!(
                    "创建时间记录: 任务={}, 分类ID={:?}, 时长={}秒",
                    task.name,
                    task.category_id,
                    elapsed_seconds
                );

                // 保存时间记录
                if let Err(e) = storage.get_database().insert_time_entry(&time_entry) {
                    log::error!("保存时间记录失败: {}", e);
                } else {
                    log::info!("保存时间记录成功: {} - {}秒", task.name, elapsed_seconds);
                }

                // 更新任务的总时长
                let task_update = crate::storage::task_models::TaskUpdate {
                    name: None,
                    description: None,
                    category_id: None,
                    status: None,
                    priority: None,
                    estimated_duration_seconds: None,
                    total_duration_seconds: Some(task.total_duration_seconds + elapsed_seconds),
                    tags: None,
                    due_date: None,
                    is_completed: None,
                    completed_at: None,
                };

                if let Err(e) = storage.get_database().update_task(task_uuid, &task_update) {
                    log::error!("更新任务总时长失败: {}", e);
                }
            }
        }
    }

    // 清除当前任务ID
    {
        let mut current_task = state.current_task_id.lock().map_err(|e| e.to_string())?;
        *current_task = None;
    }

    log::info!("停止计时器，用时: {} 秒", elapsed_seconds);

    // 发送通知
    let _ = app_handle.emit("timer-stopped", elapsed_seconds);

    // 显示系统通知
    let _ = app_handle
        .notification()
        .builder()
        .title("计时器已停止")
        .body(&format!(
            "时间记录已保存: {}",
            format_duration_for_display(elapsed_seconds)
        ))
        .show();

    Ok(TimerStatusDto {
        state: "stopped".to_string(),
        current_task_id: None,
        current_task_name: None,
        start_time: None,
        pause_time: None,
        elapsed_seconds: 0,
        total_today_seconds: get_today_total_seconds(&storage).unwrap_or(0),
    })
}

/// 暂停/恢复计时
#[tauri::command]
pub async fn pause_timer(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<TimerStatusDto, String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    let (new_state, elapsed_seconds) = if timer.is_running() {
        // 暂停计时器
        timer
            .pause()
            .map_err(|e| format!("暂停计时器失败: {}", e))?;
        ("paused".to_string(), timer.get_elapsed().num_seconds())
    } else if timer.is_paused() {
        // 恢复计时器
        timer
            .resume()
            .map_err(|e| format!("恢复计时器失败: {}", e))?;
        ("running".to_string(), timer.get_elapsed().num_seconds())
    } else {
        return Err("计时器未运行".to_string());
    };

    log::info!("计时器状态变更为: {}", new_state);

    // 发送通知
    let _ = app_handle.emit("timer-paused", &new_state);

    Ok(TimerStatusDto {
        state: new_state,
        current_task_id: Some("current_task".to_string()), // TODO: 从状态中获取当前任务ID
        current_task_name: Some("当前任务".to_string()),   // TODO: 从存储中获取任务名称
        start_time: Some(Local::now()),
        pause_time: if timer.is_paused() {
            Some(Local::now())
        } else {
            None
        },
        elapsed_seconds,
        total_today_seconds: get_today_total_seconds(&storage).unwrap_or(0),
    })
}

/// 获取计时器状态
#[tauri::command]
pub async fn get_timer_status(state: State<'_, AppState>) -> Result<TimerStatusDto, String> {
    let timer = state.timer.lock().map_err(|e| e.to_string())?;
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    let state_str = if timer.is_running() {
        "running"
    } else if timer.is_paused() {
        "paused"
    } else {
        "stopped"
    };

    let elapsed_seconds = timer.get_elapsed().num_seconds();

    // 获取当前任务信息
    let current_task_id = state
        .current_task_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let current_task_name = if let Some(ref task_id) = current_task_id {
        if let Ok(task_uuid) = uuid::Uuid::parse_str(task_id) {
            storage
                .get_database()
                .get_task_by_id(task_uuid)
                .ok()
                .flatten()
                .map(|task| task.name)
        } else {
            None
        }
    } else {
        None
    };

    Ok(TimerStatusDto {
        state: state_str.to_string(),
        current_task_id,
        current_task_name,
        start_time: None, // TODO: 从计时器状态中获取开始时间
        pause_time: None, // TODO: 从计时器状态中获取暂停时间
        elapsed_seconds,
        total_today_seconds: get_today_total_seconds(&storage).unwrap_or(0),
    })
}

// 辅助函数：获取今日总时长
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

// 辅助函数：格式化时长显示
fn format_duration_for_display(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}时{}分{}秒", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}分{}秒", minutes, secs)
    } else {
        format!("{}秒", secs)
    }
}

/// 获取今日时间记录
#[tauri::command]
pub async fn get_today_time_entries(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    let today = Local::now().date_naive();

    // 获取今日时间记录
    let today_entries = storage
        .get_database()
        .get_time_entries_by_date_range(today, today)
        .map_err(|e| format!("查询今日时间记录失败: {}", e))?;

    // 转换为前端需要的格式
    let formatted_entries: Vec<serde_json::Value> = today_entries
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
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    match storage.get_database().get_all_time_entries() {
        Ok(entries) => {
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
        Err(e) => {
            log::error!("获取时间记录失败: {}", e);
            Err(format!("获取时间记录失败: {}", e))
        }
    }
}

/// 获取今日统计数据
#[tauri::command]
pub async fn get_today_stats(state: State<'_, AppState>) -> Result<TimerStatusDto, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    let today = Local::now().date_naive();

    // 获取今日时间记录
    let today_entries = storage
        .get_database()
        .get_time_entries_by_date_range(today, today)
        .map_err(|e| format!("查询今日时间记录失败: {}", e))?;

    // 计算今日统计
    let total_seconds: i64 = today_entries
        .iter()
        .map(|entry| entry.duration_seconds)
        .sum();

    // 方案A：获取今日有时间记录的不同任务数量
    let mut unique_task_names = std::collections::HashSet::new();
    for entry in &today_entries {
        unique_task_names.insert(&entry.task_name);
    }
    let unique_task_count = unique_task_names.len() as i64;

    // 方案B：时间记录条数
    let record_count = today_entries.len() as i64;

    // 方案C：今日创建的任务总数（需要查询tasks表）
    let created_tasks_count = match storage.get_database().get_all_tasks() {
        Ok(all_tasks) => {
            let today_start = today.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let today_end = today.and_hms_opt(23, 59, 59).unwrap().and_utc();

            all_tasks
                .iter()
                .filter(|task| task.created_at >= today_start && task.created_at <= today_end)
                .count() as i64
        }
        Err(_) => 0,
    };

    log::info!("今日任务数统计 - 方案A(不同任务): {}, 方案B(记录条数): {}, 方案C(今日创建): {}, 总时长: {}秒", 
        unique_task_count, record_count, created_tasks_count, total_seconds);

    // 当前采用方案B：时间记录条数
    let task_count_to_use = record_count;

    Ok(TimerStatusDto {
        state: "stopped".to_string(),
        current_task_id: None,
        current_task_name: None,
        start_time: None,
        pause_time: None,
        elapsed_seconds: task_count_to_use, // 复用这个字段传递任务数
        total_today_seconds: total_seconds,
    })
}

// ========== 分类管理命令 ==========

/// 获取所有分类
#[tauri::command]
pub async fn get_categories(state: State<'_, AppState>) -> Result<Vec<CategoryDto>, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    // TODO: 实现实际的分类查询逻辑
    let categories = vec![
        CategoryDto {
            id: "cat1".to_string(),
            name: "工作".to_string(),
            description: Some("工作相关任务".to_string()),
            color: "#007bff".to_string(),
            icon: Some("💼".to_string()),
            is_active: true,
            task_count: 5,
            total_duration_seconds: 18000, // 5小时
            created_at: Local::now(),
            updated_at: Local::now(),
        },
        CategoryDto {
            id: "cat2".to_string(),
            name: "学习".to_string(),
            description: Some("学习相关任务".to_string()),
            color: "#28a745".to_string(),
            icon: Some("📚".to_string()),
            is_active: true,
            task_count: 3,
            total_duration_seconds: 10800, // 3小时
            created_at: Local::now(),
            updated_at: Local::now(),
        },
    ];

    Ok(categories)
}

/// 创建新分类
#[tauri::command]
pub async fn create_category(
    state: State<'_, AppState>,
    request: CreateCategoryRequest,
) -> Result<CategoryDto, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    // TODO: 实现实际的分类创建逻辑
    let category = CategoryDto {
        id: Uuid::new_v4().to_string(),
        name: request.name,
        description: request.description,
        color: request.color.unwrap_or("#6c757d".to_string()),
        icon: request.icon,
        is_active: true,
        task_count: 0,
        total_duration_seconds: 0,
        created_at: Local::now(),
        updated_at: Local::now(),
    };

    Ok(category)
}

/// 更新分类
#[tauri::command]
pub async fn update_category(
    state: State<'_, AppState>,
    category_id: String,
    request: UpdateCategoryRequest,
) -> Result<CategoryDto, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    // TODO: 实现实际的分类更新逻辑
    let category = CategoryDto {
        id: category_id,
        name: request.name.unwrap_or("更新的分类".to_string()),
        description: request.description,
        color: request.color.unwrap_or("#6c757d".to_string()),
        icon: request.icon,
        is_active: request.is_active.unwrap_or(true),
        task_count: 0,
        total_duration_seconds: 0,
        created_at: Local::now(),
        updated_at: Local::now(),
    };

    Ok(category)
}

/// 删除分类
#[tauri::command]
pub async fn delete_category(
    state: State<'_, AppState>,
    category_id: String,
) -> Result<bool, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    // TODO: 实现实际的分类删除逻辑
    Ok(true)
}

// ========== 统计分析命令 ==========

/// 获取统计数据
#[tauri::command]
pub async fn get_statistics(
    state: State<'_, AppState>,
    period: Option<String>, // "today", "week", "month", "all"
) -> Result<StatisticsDto, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

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

// ========== 数据导入导出命令 ==========

/// 导出数据
#[tauri::command]
pub async fn export_data(
    state: State<'_, AppState>,
    format: String, // "json", "csv", "xml"
    file_path: String,
) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    // TODO: 实现实际的数据导出逻辑
    Ok(format!("数据已导出到: {}", file_path))
}

/// 导入数据
#[tauri::command]
pub async fn import_data(state: State<'_, AppState>, file_path: String) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let storage = storage.as_ref().ok_or("存储未初始化")?;

    // TODO: 实现实际的数据导入逻辑
    Ok(format!("数据已从文件导入: {}", file_path))
}

// ========== 配置管理命令 ==========

/// 获取应用配置
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// 更新应用配置
#[tauri::command]
pub async fn update_config(state: State<'_, AppState>, config: AppConfig) -> Result<bool, String> {
    let mut config_guard = state.config.lock().map_err(|e| e.to_string())?;
    *config_guard = config;

    // TODO: 保存配置到文件
    Ok(true)
}
