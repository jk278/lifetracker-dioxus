//! # 计时器控制命令模块
//!
//! 负责处理计时器的启动、停止、暂停以及今日统计功能

use super::*;

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
    let storage = &state.storage;
    {
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
    let storage = &state.storage;

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

/// 恢复计时器
#[tauri::command]
pub async fn resume_timer(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<TimerStatusDto, String> {
    {
        let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
        timer.resume().map_err(|e| e.to_string())?;
    }

    // 只是恢复，不写入数据库，状态由TimerStatusDto反映
    log::info!("计时器已恢复");

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
        if let Ok(uuid) = Uuid::parse_str(task_id) {
            if let Ok(Some(task)) = state.storage.get_database().get_task_by_id(uuid) {
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
    let storage = &state.storage;

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
    let storage = &state.storage;
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
    let total_today_seconds = { get_today_total_seconds(&state.storage)? };

    let mut status_dto = get_timer_status(state).await?;

    status_dto.total_today_seconds = total_today_seconds;

    Ok(status_dto)
}
