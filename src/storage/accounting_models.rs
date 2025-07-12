//! # 记账功能数据模型定义
//!
//! 定义记账相关的数据库表对应的Rust结构体和相关类型

use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ==================== 枚举类型 ====================

/// 交易类型枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// 收入
    Income,
    /// 支出
    Expense,
    /// 转账
    Transfer,
}

/// 交易状态枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    /// 待处理
    Pending,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
}

/// 账户类型枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// 现金
    Cash,
    /// 银行账户
    Bank,
    /// 信用卡
    CreditCard,
    /// 投资账户
    Investment,
    /// 其他
    Other,
}

/// 预算周期枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BudgetPeriod {
    /// 每日
    Daily,
    /// 每周
    Weekly,
    /// 每月
    Monthly,
    /// 每年
    Yearly,
}

/// 趋势数据粒度枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrendGranularity {
    /// 按天
    Day,
    /// 按周
    Week,
    /// 按月
    Month,
}

// ==================== 账户模型 ====================

/// 账户完整模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    /// 唯一标识符
    pub id: Uuid,
    /// 账户名称
    pub name: String,
    /// 账户类型
    pub account_type: AccountType,
    /// 货币类型（如 USD, CNY）
    pub currency: String,
    /// 当前余额
    pub balance: f64,
    /// 初始余额
    pub initial_balance: f64,
    /// 账户描述
    pub description: Option<String>,
    /// 是否激活
    pub is_active: bool,
    /// 是否为默认账户
    pub is_default: bool,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 更新时间
    pub updated_at: Option<DateTime<Local>>,
}

/// 账户插入模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInsert {
    pub id: Uuid,
    pub name: String,
    pub account_type: AccountType,
    pub currency: String,
    pub balance: f64,
    pub initial_balance: f64,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_default: bool,
    pub created_at: DateTime<Local>,
}

/// 账户更新模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUpdate {
    pub name: Option<String>,
    pub account_type: Option<AccountType>,
    pub currency: Option<String>,
    pub description: Option<Option<String>>,
    pub is_active: Option<bool>,
    pub is_default: Option<bool>,
}

// ==================== 交易模型 ====================

/// 交易记录完整模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// 唯一标识符
    pub id: Uuid,
    /// 交易类型
    pub transaction_type: TransactionType,
    /// 交易金额
    pub amount: f64,
    /// 货币类型
    pub currency: String,
    /// 交易描述
    pub description: String,
    /// 账户ID
    pub account_id: Uuid,
    /// 分类ID（可选）
    pub category_id: Option<Uuid>,
    /// 转账目标账户ID（仅转账时使用）
    pub to_account_id: Option<Uuid>,
    /// 交易状态
    pub status: TransactionStatus,
    /// 交易日期
    pub transaction_date: NaiveDate,
    /// 标签列表
    pub tags: Vec<String>,
    /// 收据文件路径（可选）
    pub receipt_path: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 更新时间
    pub updated_at: Option<DateTime<Local>>,
}

/// 交易记录插入模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInsert {
    pub id: Uuid,
    pub transaction_type: TransactionType,
    pub amount: f64,
    pub currency: String,
    pub description: String,
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub to_account_id: Option<Uuid>,
    pub status: TransactionStatus,
    pub transaction_date: NaiveDate,
    pub tags: Vec<String>,
    pub receipt_path: Option<String>,
    pub created_at: DateTime<Local>,
}

/// 交易记录更新模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionUpdate {
    pub transaction_type: Option<TransactionType>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub account_id: Option<Uuid>,
    pub category_id: Option<Option<Uuid>>,
    pub to_account_id: Option<Option<Uuid>>,
    pub status: Option<TransactionStatus>,
    pub transaction_date: Option<NaiveDate>,
    pub tags: Option<Vec<String>>,
    pub receipt_path: Option<Option<String>>,
}

// ==================== 交易分类模型 ====================

/// 交易分类完整模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionCategory {
    /// 唯一标识符
    pub id: Uuid,
    /// 分类名称
    pub name: String,
    /// 交易类型（收入/支出/转账）
    pub transaction_type: TransactionType,
    /// 分类描述
    pub description: Option<String>,
    /// 分类颜色（十六进制）
    pub color: String,
    /// 分类图标
    pub icon: Option<String>,
    /// 父分类ID（层级分类）
    pub parent_id: Option<Uuid>,
    /// 是否激活
    pub is_active: bool,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 更新时间
    pub updated_at: Option<DateTime<Local>>,
}

/// 交易分类插入模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCategoryInsert {
    pub id: Uuid,
    pub name: String,
    pub transaction_type: TransactionType,
    pub description: Option<String>,
    pub color: String,
    pub icon: Option<String>,
    pub parent_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Local>,
}

/// 交易分类更新模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCategoryUpdate {
    pub name: Option<String>,
    pub transaction_type: Option<TransactionType>,
    pub description: Option<Option<String>>,
    pub color: Option<String>,
    pub icon: Option<Option<String>>,
    pub parent_id: Option<Option<Uuid>>,
    pub is_active: Option<bool>,
}

// ==================== 预算模型 ====================

/// 预算完整模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Budget {
    /// 唯一标识符
    pub id: Uuid,
    /// 预算名称
    pub name: String,
    /// 关联的分类ID
    pub category_id: Uuid,
    /// 预算金额
    pub amount: f64,
    /// 货币类型
    pub currency: String,
    /// 预算周期
    pub period: BudgetPeriod,
    /// 开始日期
    pub start_date: NaiveDate,
    /// 结束日期（可选）
    pub end_date: Option<NaiveDate>,
    /// 已花费金额
    pub spent_amount: f64,
    /// 剩余金额
    pub remaining_amount: f64,
    /// 是否激活
    pub is_active: bool,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 更新时间
    pub updated_at: Option<DateTime<Local>>,
}

/// 预算插入模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetInsert {
    pub id: Uuid,
    pub name: String,
    pub category_id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub period: BudgetPeriod,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub spent_amount: f64,
    pub remaining_amount: f64,
    pub is_active: bool,
    pub created_at: DateTime<Local>,
}

/// 预算更新模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetUpdate {
    pub name: Option<String>,
    pub category_id: Option<Uuid>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub period: Option<BudgetPeriod>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<Option<NaiveDate>>,
    pub is_active: Option<bool>,
}

// ==================== 统计模型 ====================

/// 财务统计模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinancialStats {
    /// 总收入
    pub total_income: f64,
    /// 总支出
    pub total_expense: f64,
    /// 净收入
    pub net_income: f64,
    /// 账户余额总计
    pub account_balance: f64,
    /// 交易总数
    pub transaction_count: i64,
    /// 统计期间开始
    pub period_start: NaiveDate,
    /// 统计期间结束
    pub period_end: NaiveDate,
    /// 货币类型
    pub currency: String,
}

/// 分类统计模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryBreakdown {
    /// 分类ID
    pub category_id: Uuid,
    /// 分类名称
    pub category_name: String,
    /// 金额
    pub amount: f64,
    /// 百分比
    pub percentage: f64,
    /// 交易数量
    pub transaction_count: i64,
}

/// 月度趋势模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonthlyTrend {
    /// 月份（YYYY-MM格式）
    pub month: String,
    /// 收入
    pub income: f64,
    /// 支出
    pub expense: f64,
    /// 净收入
    pub net: f64,
}

/// 账户余额模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountBalance {
    /// 账户ID
    pub account_id: Uuid,
    /// 账户名称
    pub account_name: String,
    /// 余额
    pub balance: f64,
    /// 货币类型
    pub currency: String,
}

/// 财务报表模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinancialReport {
    /// 统计期间
    pub period: (NaiveDate, NaiveDate),
    /// 汇总统计
    pub summary: FinancialStats,
    /// 收入分析
    pub income_breakdown: Vec<CategoryBreakdown>,
    /// 支出分析
    pub expense_breakdown: Vec<CategoryBreakdown>,
    /// 月度趋势
    pub monthly_trend: Vec<MonthlyTrend>,
    /// 账户余额
    pub account_balances: Vec<AccountBalance>,
}

/// 趋势数据模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrendData {
    /// 时间标签（如"2024-01", "W27", "01-15"）
    pub label: String,
    /// 收入金额
    pub income: f64,
    /// 支出金额
    pub expense: f64,
}

// ==================== 查询模型 ====================

/// 交易查询参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionQuery {
    pub account_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transaction_type: Option<TransactionType>,
    pub status: Option<TransactionStatus>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

// ==================== 实现方法 ====================

impl Account {
    /// 创建新账户
    pub fn new(
        name: String,
        account_type: AccountType,
        currency: String,
        initial_balance: f64,
    ) -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4(),
            name,
            account_type,
            currency,
            balance: initial_balance,
            initial_balance,
            description: None,
            is_active: true,
            is_default: false,
            created_at: now,
            updated_at: None,
        }
    }

    /// 更新余额
    pub fn update_balance(&mut self, amount: f64) {
        self.balance += amount;
        self.updated_at = Some(Local::now());
    }

    /// 设置为默认账户
    pub fn set_as_default(&mut self) {
        self.is_default = true;
        self.updated_at = Some(Local::now());
    }

    /// 格式化余额显示
    pub fn formatted_balance(&self) -> String {
        format!("{:.2} {}", self.balance, self.currency)
    }
}

impl Transaction {
    /// 创建新交易
    pub fn new(
        transaction_type: TransactionType,
        amount: f64,
        currency: String,
        description: String,
        account_id: Uuid,
        transaction_date: NaiveDate,
    ) -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4(),
            transaction_type,
            amount,
            currency,
            description,
            account_id,
            category_id: None,
            to_account_id: None,
            status: TransactionStatus::Completed,
            transaction_date,
            tags: Vec::new(),
            receipt_path: None,
            created_at: now,
            updated_at: None,
        }
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Some(Local::now());
        }
    }

    /// 设置分类
    pub fn set_category(&mut self, category_id: Uuid) {
        self.category_id = Some(category_id);
        self.updated_at = Some(Local::now());
    }

    /// 格式化金额显示
    pub fn formatted_amount(&self) -> String {
        let sign = match self.transaction_type {
            TransactionType::Income => "+",
            TransactionType::Expense => "-",
            TransactionType::Transfer => "",
        };
        format!("{}{:.2} {}", sign, self.amount, self.currency)
    }
}

impl TransactionCategory {
    /// 创建新分类
    pub fn new(name: String, transaction_type: TransactionType, color: String) -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4(),
            name,
            transaction_type,
            description: None,
            color,
            icon: None,
            parent_id: None,
            is_active: true,
            created_at: now,
            updated_at: None,
        }
    }

    /// 检查是否为根分类
    pub fn is_root_category(&self) -> bool {
        self.parent_id.is_none()
    }
}

impl Budget {
    /// 创建新预算
    pub fn new(
        name: String,
        category_id: Uuid,
        amount: f64,
        currency: String,
        period: BudgetPeriod,
        start_date: NaiveDate,
    ) -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4(),
            name,
            category_id,
            amount,
            currency,
            period,
            start_date,
            end_date: None,
            spent_amount: 0.0,
            remaining_amount: amount,
            is_active: true,
            created_at: now,
            updated_at: None,
        }
    }

    /// 更新已花费金额
    pub fn update_spent_amount(&mut self, spent: f64) {
        self.spent_amount = spent;
        self.remaining_amount = self.amount - spent;
        self.updated_at = Some(Local::now());
    }

    /// 获取使用百分比
    pub fn usage_percentage(&self) -> f64 {
        if self.amount > 0.0 {
            (self.spent_amount / self.amount * 100.0).min(100.0)
        } else {
            0.0
        }
    }

    /// 检查是否超预算
    pub fn is_over_budget(&self) -> bool {
        self.spent_amount > self.amount
    }
}

// ==================== 字符串转换实现 ====================

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Income => write!(f, "收入"),
            TransactionType::Expense => write!(f, "支出"),
            TransactionType::Transfer => write!(f, "转账"),
        }
    }
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "待处理"),
            TransactionStatus::Completed => write!(f, "已完成"),
            TransactionStatus::Cancelled => write!(f, "已取消"),
        }
    }
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::Cash => write!(f, "现金"),
            AccountType::Bank => write!(f, "银行账户"),
            AccountType::CreditCard => write!(f, "信用卡"),
            AccountType::Investment => write!(f, "投资账户"),
            AccountType::Other => write!(f, "其他"),
        }
    }
}

impl std::fmt::Display for BudgetPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetPeriod::Daily => write!(f, "每日"),
            BudgetPeriod::Weekly => write!(f, "每周"),
            BudgetPeriod::Monthly => write!(f, "每月"),
            BudgetPeriod::Yearly => write!(f, "每年"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account::new(
            "测试账户".to_string(),
            AccountType::Bank,
            "CNY".to_string(),
            1000.0,
        );

        assert_eq!(account.name, "测试账户");
        assert_eq!(account.account_type, AccountType::Bank);
        assert_eq!(account.balance, 1000.0);
        assert_eq!(account.initial_balance, 1000.0);
        assert!(account.is_active);
        assert!(!account.is_default);
    }

    #[test]
    fn test_transaction_creation() {
        let transaction = Transaction::new(
            TransactionType::Expense,
            100.0,
            "CNY".to_string(),
            "测试支出".to_string(),
            Uuid::new_v4(),
            chrono::Local::now().date_naive(),
        );

        assert_eq!(transaction.transaction_type, TransactionType::Expense);
        assert_eq!(transaction.amount, 100.0);
        assert_eq!(transaction.description, "测试支出");
        assert_eq!(transaction.status, TransactionStatus::Completed);
    }

    #[test]
    fn test_budget_usage_calculation() {
        let mut budget = Budget::new(
            "测试预算".to_string(),
            Uuid::new_v4(),
            1000.0,
            "CNY".to_string(),
            BudgetPeriod::Monthly,
            chrono::Local::now().date_naive(),
        );

        budget.update_spent_amount(300.0);
        assert_eq!(budget.usage_percentage(), 30.0);
        assert!(!budget.is_over_budget());

        budget.update_spent_amount(1200.0);
        assert!(budget.is_over_budget());
    }
}
