//! # 分类管理命令模块
//!
//! 负责处理分类的创建、读取、更新、删除操作

use super::*;

// ========== 请求结构体 ==========

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

// ========== 分类管理命令 ==========

/// 获取所有分类
#[tauri::command]
pub async fn get_categories(state: State<'_, AppState>) -> Result<Vec<CategoryDto>, String> {
    log::debug!("获取分类列表");
    let storage = &state.storage;

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
    use crate::storage::models::CategoryInsert;
    let storage = &state.storage;

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
    use crate::storage::models::CategoryInsert;
    let storage = &state.storage;

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
    let uuid = Uuid::parse_str(&category_id).map_err(|e| format!("无效的分类ID: {}", e))?;
    let storage = &state.storage;

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
