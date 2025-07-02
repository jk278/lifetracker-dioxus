//! # 记账功能命令模块
//!
//! 负责处理财务管理相关功能：账户管理、交易记录、预算管理

use super::*;

// ========== 请求结构体 ==========

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
        "cash" => crate::storage::AccountType::Cash,
        "bank" => crate::storage::AccountType::Bank,
        "creditcard" | "credit_card" => crate::storage::AccountType::CreditCard,
        "investment" => crate::storage::AccountType::Investment,
        "other" => crate::storage::AccountType::Other,
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
    let uuid = Uuid::parse_str(&account_id).map_err(|_| "无效的账户ID")?;
    let storage = &state.storage;

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

/// 创建交易
#[tauri::command]
pub async fn create_transaction(
    state: State<'_, AppState>,
    request: CreateTransactionRequest,
) -> Result<TransactionDto, String> {
    let storage = &state.storage;

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
        currency: "CNY".to_string(),
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
    Ok(transaction_dto)
}

/// 更新交易
#[tauri::command]
pub async fn update_transaction(
    state: State<'_, AppState>,
    transaction_id: String,
    request: UpdateTransactionRequest,
) -> Result<TransactionDto, String> {
    let storage = &state.storage;
    let uuid = Uuid::parse_str(&transaction_id).map_err(|_| "无效的交易ID")?;

    // 这里暂时返回一个简单的响应，实际实现需要更完整的逻辑
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
    let storage = &state.storage;
    let uuid = Uuid::parse_str(&transaction_id).map_err(|_| "无效的交易ID")?;

    storage
        .get_database()
        .delete_transaction(uuid)
        .map_err(|e| e.to_string())?;

    log::info!("交易删除成功: {}", transaction_id);
    Ok(true)
}
