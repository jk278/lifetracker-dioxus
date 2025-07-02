//! # 账户管理 Tauri 命令
//!
//! 负责账户相关的 Tauri 命令实现

use crate::{storage::AccountType, tauri_commands::AppState};
use chrono::Local;
use log::{error, info};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use super::types::{AccountDto, CreateAccountRequest, UpdateAccountRequest};

/// 获取所有账户
#[tauri::command]
pub async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<AccountDto>, String> {
    log::debug!("[CMD] get_accounts: Attempting to get accounts.");
    let storage = &state.storage;

    let accounts_from_db = storage
        .get_database()
        .get_all_accounts()
        .map_err(|e| e.to_string())?;

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

/// 根据ID获取账户
#[tauri::command]
pub async fn get_account_by_id(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<Option<AccountDto>, String> {
    let uuid = Uuid::parse_str(&account_id).map_err(|_| "无效的账户ID")?;
    let storage = &state.storage;

    let account = storage
        .get_database()
        .get_account_by_id(uuid)
        .map_err(|e| e.to_string())?;

    let account_dto = account.map(|account| AccountDto {
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
    });

    Ok(account_dto)
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
    let storage = &state.storage;

    // 解析账户类型
    let account_type = match request.account_type.as_str() {
        "cash" => AccountType::Cash,
        "bank" => AccountType::Bank,
        "creditcard" | "credit_card" => AccountType::CreditCard,
        "investment" => AccountType::Investment,
        "other" => AccountType::Other,
        _ => return Err("无效的账户类型".to_string()),
    };

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

    storage
        .get_database()
        .insert_account(&account_insert)
        .map_err(|e| e.to_string())?;

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
    let uuid = Uuid::parse_str(&account_id).map_err(|_| "无效的账户ID")?;
    let storage = &state.storage;

    // 构建更新请求
    let account_type = if let Some(type_str) = &request.account_type {
        Some(match type_str.as_str() {
            "cash" => AccountType::Cash,
            "bank" => AccountType::Bank,
            "creditcard" | "credit_card" => AccountType::CreditCard,
            "investment" => AccountType::Investment,
            "other" => AccountType::Other,
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
    let uuid = Uuid::parse_str(&account_id).map_err(|_| "无效的账户ID")?;
    let storage = &state.storage;

    // 检查账户是否有关联交易
    let transactions = storage
        .get_database()
        .get_transactions_by_account(uuid)
        .map_err(|e| e.to_string())?;

    if !transactions.is_empty() {
        return Err("无法删除有关联交易的账户".to_string());
    }

    storage
        .get_database()
        .delete_account(uuid)
        .map_err(|e| e.to_string())?;

    log::info!("账户删除成功: {}", account_id);
    Ok(true)
}

/// 获取账户余额
#[tauri::command]
pub async fn get_account_balance(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<f64, String> {
    let uuid = Uuid::parse_str(&account_id).map_err(|_| "无效的账户ID")?;
    let storage = &state.storage;

    let account = storage
        .get_database()
        .get_account_by_id(uuid)
        .map_err(|e| e.to_string())?
        .ok_or("账户不存在")?;

    Ok(account.balance)
}

/// 更新账户余额
#[tauri::command]
pub async fn update_account_balance(
    state: State<'_, AppState>,
    account_id: String,
    new_balance: f64,
) -> Result<bool, String> {
    let uuid = Uuid::parse_str(&account_id).map_err(|_| "无效的账户ID")?;
    let storage = &state.storage;

    storage
        .get_database()
        .update_account_balance(uuid, new_balance)
        .map_err(|e| e.to_string())?;

    log::info!("账户余额更新成功: {} = {}", account_id, new_balance);
    Ok(true)
}

/// 设置默认账户
#[tauri::command]
pub async fn set_default_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<bool, String> {
    let uuid = Uuid::parse_str(&account_id).map_err(|_| "无效的账户ID")?;
    let storage = &state.storage;

    // 首先取消所有账户的默认状态
    let all_accounts = storage
        .get_database()
        .get_all_accounts()
        .map_err(|e| e.to_string())?;

    for account in all_accounts {
        if account.is_default {
            let update = crate::storage::AccountUpdate {
                name: None,
                account_type: None,
                currency: None,
                description: None,
                is_active: None,
                is_default: Some(false),
            };

            storage
                .get_database()
                .update_account(account.id, &update)
                .map_err(|e| e.to_string())?;
        }
    }

    // 设置新的默认账户
    let update = crate::storage::AccountUpdate {
        name: None,
        account_type: None,
        currency: None,
        description: None,
        is_active: None,
        is_default: Some(true),
    };

    storage
        .get_database()
        .update_account(uuid, &update)
        .map_err(|e| e.to_string())?;

    log::info!("设置默认账户成功: {}", account_id);
    Ok(true)
}
