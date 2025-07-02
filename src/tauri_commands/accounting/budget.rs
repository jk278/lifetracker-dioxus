//! # 预算管理 Tauri 命令
//!
//! 负责预算相关的 Tauri 命令实现

use crate::tauri_commands::AppState;
use tauri::State;

use super::types::{BudgetDto, CreateBudgetRequest, UpdateBudgetRequest};

/// 获取所有预算
#[tauri::command]
pub async fn get_budgets(state: State<'_, AppState>) -> Result<Vec<BudgetDto>, String> {
    log::debug!("[CMD] get_budgets: 获取预算列表");

    // 暂时返回空列表，等待完整实现
    let budgets = Vec::new();

    log::debug!("返回 {} 个预算", budgets.len());
    Ok(budgets)
}

/// 创建预算
#[tauri::command]
pub async fn create_budget(
    state: State<'_, AppState>,
    request: CreateBudgetRequest,
) -> Result<BudgetDto, String> {
    log::info!("[CMD] create_budget: 创建预算 '{}'", request.name);

    // 暂时返回模拟数据，等待完整实现
    let budget_dto = BudgetDto {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        category_id: request.category_id,
        amount: request.amount,
        currency: request.currency.unwrap_or("CNY".to_string()),
        period: request.period,
        start_date: request.start_date,
        end_date: request.end_date,
        description: request.description,
        is_active: true,
        spent_amount: 0.0,
        usage_percentage: 0.0,
        created_at: chrono::Local::now(),
        updated_at: None,
    };

    log::info!("预算创建成功: {}", budget_dto.id);
    Ok(budget_dto)
}

/// 删除预算
#[tauri::command]
pub async fn delete_budget(state: State<'_, AppState>, budget_id: String) -> Result<bool, String> {
    log::info!("[CMD] delete_budget: 删除预算 {}", budget_id);

    // 暂时返回成功，等待完整实现
    Ok(true)
}
