//! # 任务管理命令模块
//!
//! 负责处理任务的创建、读取、更新、删除操作

use super::*;

// ========== 请求结构体 ==========

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
    let storage = &state.storage;

    // 在作用域内获取数据，然后立即释放锁
    let (tasks, category_names, db_access_time) = {
        let start_time = std::time::Instant::now();

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
    let storage = &state.storage;

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
    let storage = &state.storage;

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
    let uuid = Uuid::parse_str(&task_id).map_err(|_| "无效的任务ID格式".to_string())?;
    let storage = &state.storage;

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
