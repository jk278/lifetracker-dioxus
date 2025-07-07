//! # 交易管理 Tauri 命令
//!
//! 负责交易相关的 Tauri 命令实现

use crate::{
    storage::{TransactionStatus, TransactionType},
    tauri_commands::AppState,
};
use chrono::Local;
use log::{error, info};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};
use uuid::Uuid;

use super::types::{
    CreateTransactionRequest, TransactionDto, TransactionQueryRequest, UpdateTransactionRequest,
};

/// 获取交易列表
#[tauri::command]
pub async fn get_transactions(
    state: State<'_, AppState>,
    query: Option<TransactionQueryRequest>,
) -> Result<Vec<TransactionDto>, String> {
    let storage = &state.storage;

    let (transactions, accounts) = {
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

        let transaction_dto = TransactionDto {
            id: transaction.id.to_string(),
            transaction_type: format!("{:?}", transaction.transaction_type).to_lowercase(),
            amount: transaction.amount,
            currency: transaction.currency,
            description: transaction.description,
            account_id: transaction.account_id.to_string(),
            account_name,
            category_id: transaction.category_id.map(|id| id.to_string()),
            category_name: None,
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

/// 根据ID获取交易
#[tauri::command]
pub async fn get_transaction_by_id(
    state: State<'_, AppState>,
    transaction_id: String,
) -> Result<Option<TransactionDto>, String> {
    let uuid = Uuid::parse_str(&transaction_id).map_err(|_| "无效的交易ID")?;
    let storage = &state.storage;

    let transaction = storage
        .get_database()
        .get_transaction_by_id(uuid)
        .map_err(|e| e.to_string())?;

    if let Some(transaction) = transaction {
        // 获取账户信息
        let account = storage
            .get_database()
            .get_account_by_id(transaction.account_id)
            .map_err(|e| e.to_string())?;

        let to_account_name = if let Some(to_id) = transaction.to_account_id {
            storage
                .get_database()
                .get_account_by_id(to_id)
                .map_err(|e| e.to_string())?
                .map(|a| a.name)
        } else {
            None
        };

        let transaction_dto = TransactionDto {
            id: transaction.id.to_string(),
            transaction_type: format!("{:?}", transaction.transaction_type).to_lowercase(),
            amount: transaction.amount,
            currency: transaction.currency,
            description: transaction.description,
            account_id: transaction.account_id.to_string(),
            account_name: account.map(|a| a.name),
            category_id: transaction.category_id.map(|id| id.to_string()),
            category_name: None,
            to_account_id: transaction.to_account_id.map(|id| id.to_string()),
            to_account_name,
            status: format!("{:?}", transaction.status).to_lowercase(),
            transaction_date: transaction.transaction_date.format("%Y-%m-%d").to_string(),
            tags: transaction.tags,
            receipt_path: transaction.receipt_path,
            created_at: transaction.created_at,
            updated_at: transaction.updated_at,
        };

        Ok(Some(transaction_dto))
    } else {
        Ok(None)
    }
}

/// 创建交易
#[tauri::command]
pub async fn create_transaction(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    request: CreateTransactionRequest,
) -> Result<TransactionDto, String> {
    let storage = &state.storage;

    // 解析交易类型
    let transaction_type = match request.transaction_type.as_str() {
        "income" => TransactionType::Income,
        "expense" => TransactionType::Expense,
        "transfer" => TransactionType::Transfer,
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
        currency: "CNY".to_string(),
        description: request.description,
        account_id,
        category_id,
        to_account_id,
        status: TransactionStatus::Completed,
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
        TransactionType::Income => {
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
        TransactionType::Expense => {
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
        TransactionType::Transfer => {
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
        category_name: None,
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

    // 发送数据变化事件通知前端刷新
    if let Err(e) = app_handle.emit("data_changed", "transaction_created") {
        log::warn!("发送交易创建事件失败: {}", e);
    }

    Ok(transaction_dto)
}

/// 删除交易
#[tauri::command]
pub async fn delete_transaction(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    transaction_id: String,
) -> Result<bool, String> {
    let storage = &state.storage;
    let uuid = Uuid::parse_str(&transaction_id).map_err(|_| "无效的交易ID")?;

    storage
        .get_database()
        .delete_transaction(uuid)
        .map_err(|e| e.to_string())?;

    log::info!("交易删除成功: {}", transaction_id);

    // 发送数据变化事件通知前端刷新
    if let Err(e) = app_handle.emit("data_changed", "transaction_deleted") {
        log::warn!("发送交易删除事件失败: {}", e);
    }

    Ok(true)
}
