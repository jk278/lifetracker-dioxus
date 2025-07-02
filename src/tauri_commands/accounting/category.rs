//! # 分类管理 Tauri 命令
//!
//! 负责交易分类相关的 Tauri 命令实现

use crate::tauri_commands::AppState;
use tauri::State;

use super::types::{
    CreateTransactionCategoryRequest, TransactionCategoryDto, UpdateTransactionCategoryRequest,
};

/// 获取所有交易分类
#[tauri::command]
pub async fn get_transaction_categories(
    state: State<'_, AppState>,
) -> Result<Vec<TransactionCategoryDto>, String> {
    log::debug!("[CMD] get_transaction_categories: 获取交易分类列表");

    // 暂时返回空列表，等待完整实现
    let categories = Vec::new();

    log::debug!("返回 {} 个交易分类", categories.len());
    Ok(categories)
}

/// 创建交易分类
#[tauri::command]
pub async fn create_transaction_category(
    state: State<'_, AppState>,
    request: CreateTransactionCategoryRequest,
) -> Result<TransactionCategoryDto, String> {
    log::info!(
        "[CMD] create_transaction_category: 创建交易分类 '{}'",
        request.name
    );

    // 暂时返回模拟数据，等待完整实现
    let category_dto = TransactionCategoryDto {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        transaction_type: request.transaction_type,
        color: request.color.unwrap_or("#4F46E5".to_string()),
        icon: request.icon,
        description: request.description,
        parent_id: request.parent_id,
        is_active: true,
        created_at: chrono::Local::now(),
        updated_at: None,
    };

    log::info!("交易分类创建成功: {}", category_dto.id);
    Ok(category_dto)
}

/// 删除交易分类
#[tauri::command]
pub async fn delete_transaction_category(
    state: State<'_, AppState>,
    category_id: String,
) -> Result<bool, String> {
    log::info!(
        "[CMD] delete_transaction_category: 删除交易分类 {}",
        category_id
    );

    // 暂时返回成功，等待完整实现
    Ok(true)
}
