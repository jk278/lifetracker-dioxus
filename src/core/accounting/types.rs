//! # 记账功能共享类型定义
//!
//! 定义记账系统中各模块共享的类型和枚举

use uuid::Uuid;

/// 预算状态枚举
#[derive(Debug, Clone)]
pub enum BudgetStatus {
    /// 在预算范围内
    OnTrack { spent: f64, usage_percentage: f64 },
    /// 使用较多（75-90%）
    Warning { spent: f64, usage_percentage: f64 },
    /// 接近限额（90%+）
    NearLimit { spent: f64, usage_percentage: f64 },
    /// 超预算
    OverBudget { spent: f64, over_amount: f64 },
}

/// 预算警告
#[derive(Debug, Clone)]
pub struct BudgetWarning {
    pub budget_id: Uuid,
    pub budget_name: String,
    pub message: String,
    pub severity: WarningSeverity,
}

/// 警告严重程度
#[derive(Debug, Clone)]
pub enum WarningSeverity {
    Low,
    Medium,
    High,
    Critical,
}
