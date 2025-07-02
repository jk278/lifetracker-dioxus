//! # 记账功能命令类型定义
//!
//! 定义各种请求和响应结构体

use serde::{Deserialize, Serialize};

// ========== 账户管理请求结构体 ==========

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

// ========== 交易管理请求结构体 ==========

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

// ========== 预算管理请求结构体 ==========

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

// ========== 分类管理请求结构体 ==========

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

// ========== 响应结构体 ==========

/// 账户信息响应
#[derive(Debug, Serialize)]
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
    pub created_at: chrono::DateTime<chrono::Local>,
    pub updated_at: Option<chrono::DateTime<chrono::Local>>,
}

/// 交易信息响应
#[derive(Debug, Serialize)]
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
    pub created_at: chrono::DateTime<chrono::Local>,
    pub updated_at: Option<chrono::DateTime<chrono::Local>>,
}

/// 预算信息响应
#[derive(Debug, Serialize)]
pub struct BudgetDto {
    pub id: String,
    pub name: String,
    pub category_id: String,
    pub amount: f64,
    pub currency: String,
    pub period: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub spent_amount: f64,
    pub usage_percentage: f64,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub updated_at: Option<chrono::DateTime<chrono::Local>>,
}

/// 交易分类信息响应
#[derive(Debug, Serialize)]
pub struct TransactionCategoryDto {
    pub id: String,
    pub name: String,
    pub transaction_type: String,
    pub color: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub updated_at: Option<chrono::DateTime<chrono::Local>>,
}
