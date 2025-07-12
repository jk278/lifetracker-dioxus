//! # 预算管理核心逻辑
//!
//! 负责预算的创建、更新、状态检查和警告等操作

use crate::errors::{AppError, Result};
use crate::storage::{Budget, BudgetPeriod};
use chrono::{Datelike, Local, NaiveDate};
use std::collections::HashMap;
use uuid::Uuid;

use super::types::{BudgetStatus, BudgetWarning, WarningSeverity};

/// 预算管理器
#[derive(Debug)]
pub struct BudgetManager {
    /// 默认货币
    default_currency: String,
    /// 预算使用情况缓存
    budget_usage: HashMap<Uuid, f64>,
}

impl BudgetManager {
    /// 创建新的预算管理器
    pub fn new() -> Self {
        Self {
            default_currency: "CNY".to_string(),
            budget_usage: HashMap::new(),
        }
    }

    /// 设置默认货币
    pub fn set_default_currency(&mut self, currency: String) {
        self.default_currency = currency;
    }

    /// 获取默认货币
    pub fn get_default_currency(&self) -> &str {
        &self.default_currency
    }

    /// 创建新预算
    pub fn create_budget(
        &mut self,
        name: String,
        category_id: Uuid,
        amount: f64,
        period: BudgetPeriod,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
        currency: Option<String>,
    ) -> Result<Budget> {
        if amount <= 0.0 {
            return Err(AppError::Validation("预算金额必须大于0".to_string()));
        }

        let currency = currency.unwrap_or_else(|| self.default_currency.clone());
        let mut budget = Budget::new(name, category_id, amount, currency, period, start_date);

        if let Some(end) = end_date {
            budget.end_date = Some(end);
        }

        // 初始化预算使用情况
        self.budget_usage.insert(budget.id, 0.0);

        log::info!("创建预算: {} ({})", budget.name, budget.id);
        Ok(budget)
    }

    /// 更新预算使用情况
    pub fn update_budget_usage(&mut self, budget_id: Uuid, spent_amount: f64) -> Result<()> {
        self.budget_usage.insert(budget_id, spent_amount);
        log::debug!("更新预算使用: {} = {}", budget_id, spent_amount);
        Ok(())
    }

    /// 增加预算使用金额
    pub fn add_budget_usage(&mut self, budget_id: Uuid, amount: f64) -> Result<()> {
        let current_usage = self.budget_usage.get(&budget_id).copied().unwrap_or(0.0);
        self.budget_usage.insert(budget_id, current_usage + amount);
        log::debug!(
            "增加预算使用: {} += {} (总计: {})",
            budget_id,
            amount,
            current_usage + amount
        );
        Ok(())
    }

    /// 减少预算使用金额
    pub fn subtract_budget_usage(&mut self, budget_id: Uuid, amount: f64) -> Result<()> {
        let current_usage = self.budget_usage.get(&budget_id).copied().unwrap_or(0.0);
        let new_usage = (current_usage - amount).max(0.0);
        self.budget_usage.insert(budget_id, new_usage);
        log::debug!(
            "减少预算使用: {} -= {} (总计: {})",
            budget_id,
            amount,
            new_usage
        );
        Ok(())
    }

    /// 获取预算使用情况
    pub fn get_budget_usage(&self, budget_id: Uuid) -> f64 {
        self.budget_usage.get(&budget_id).copied().unwrap_or(0.0)
    }

    /// 检查预算状态
    pub fn check_budget_status(&self, budget: &Budget) -> BudgetStatus {
        let spent = self.budget_usage.get(&budget.id).copied().unwrap_or(0.0);
        let usage_percentage = if budget.amount > 0.0 {
            spent / budget.amount * 100.0
        } else {
            0.0
        };

        if spent > budget.amount {
            BudgetStatus::OverBudget {
                spent,
                over_amount: spent - budget.amount,
            }
        } else if usage_percentage >= 90.0 {
            BudgetStatus::NearLimit {
                spent,
                usage_percentage,
            }
        } else if usage_percentage >= 75.0 {
            BudgetStatus::Warning {
                spent,
                usage_percentage,
            }
        } else {
            BudgetStatus::OnTrack {
                spent,
                usage_percentage,
            }
        }
    }

    /// 获取预算警告
    pub fn get_budget_warnings(&self, budgets: &[Budget]) -> Vec<BudgetWarning> {
        budgets
            .iter()
            .filter_map(|budget| {
                if !budget.is_active {
                    return None;
                }

                match self.check_budget_status(budget) {
                    BudgetStatus::OverBudget { over_amount, .. } => Some(BudgetWarning {
                        budget_id: budget.id,
                        budget_name: budget.name.clone(),
                        message: format!("预算已超支 {:.2} {}", over_amount, budget.currency),
                        severity: WarningSeverity::Critical,
                    }),
                    BudgetStatus::NearLimit {
                        usage_percentage, ..
                    } => Some(BudgetWarning {
                        budget_id: budget.id,
                        budget_name: budget.name.clone(),
                        message: format!("预算即将用尽 ({:.1}%)", usage_percentage),
                        severity: WarningSeverity::High,
                    }),
                    BudgetStatus::Warning {
                        usage_percentage, ..
                    } => Some(BudgetWarning {
                        budget_id: budget.id,
                        budget_name: budget.name.clone(),
                        message: format!("预算使用较多 ({:.1}%)", usage_percentage),
                        severity: WarningSeverity::Medium,
                    }),
                    _ => None,
                }
            })
            .collect()
    }

    /// 检查预算是否已过期
    pub fn is_budget_expired(&self, budget: &Budget) -> bool {
        // 如果有明确的结束日期，检查是否已过期
        if let Some(end_date) = budget.end_date {
            return Local::now().date_naive() > end_date;
        }

        // 基于预算周期判断是否过期
        match budget.period {
            BudgetPeriod::Daily => Local::now().date_naive() > budget.start_date,
            BudgetPeriod::Weekly => {
                let days_since_start = (Local::now().date_naive() - budget.start_date).num_days();
                days_since_start >= 7
            }
            BudgetPeriod::Monthly => {
                let current_date = Local::now().date_naive();
                current_date.year() != budget.start_date.year()
                    || current_date.month() != budget.start_date.month()
            }
            BudgetPeriod::Yearly => Local::now().date_naive().year() != budget.start_date.year(),
        }
    }

    /// 获取活跃预算
    pub fn get_active_budgets(&self, budgets: &[Budget]) -> Vec<Budget> {
        budgets
            .iter()
            .filter(|budget| budget.is_active && !self.is_budget_expired(budget))
            .cloned()
            .collect()
    }

    /// 获取过期预算
    pub fn get_expired_budgets(&self, budgets: &[Budget]) -> Vec<Budget> {
        budgets
            .iter()
            .filter(|budget| self.is_budget_expired(budget))
            .cloned()
            .collect()
    }

    /// 计算预算剩余金额
    pub fn calculate_remaining_amount(&self, budget: &Budget) -> f64 {
        let spent = self.get_budget_usage(budget.id);
        (budget.amount - spent).max(0.0)
    }

    /// 计算所有预算的总使用情况
    pub fn calculate_total_budget_usage(&self, budgets: &[Budget]) -> (f64, f64) {
        let total_budgeted = budgets.iter().map(|b| b.amount).sum();
        let total_spent = budgets.iter().map(|b| self.get_budget_usage(b.id)).sum();

        (total_budgeted, total_spent)
    }

    /// 重置预算使用情况（新周期开始时使用）
    pub fn reset_budget_usage(&mut self, budget_id: Uuid) -> Result<()> {
        self.budget_usage.insert(budget_id, 0.0);
        log::info!("重置预算使用情况: {}", budget_id);
        Ok(())
    }

    /// 批量重置预算使用情况
    pub fn batch_reset_budget_usage(&mut self, budget_ids: &[Uuid]) -> Result<()> {
        for budget_id in budget_ids {
            self.reset_budget_usage(*budget_id)?;
        }
        log::info!("批量重置了 {} 个预算的使用情况", budget_ids.len());
        Ok(())
    }

    /// 清空预算使用缓存
    pub fn clear_usage_cache(&mut self) {
        self.budget_usage.clear();
        log::debug!("清空预算使用缓存");
    }

    /// 重新加载预算使用缓存
    pub fn reload_usage_cache(&mut self, usage_data: HashMap<Uuid, f64>) {
        self.budget_usage = usage_data;
        log::debug!("重新加载了 {} 个预算的使用情况", self.budget_usage.len());
    }
}

impl Default for BudgetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_budget_manager_creation() {
        let manager = BudgetManager::new();
        assert_eq!(manager.get_default_currency(), "CNY");
        assert!(manager.budget_usage.is_empty());
    }

    #[test]
    fn test_create_budget() {
        let mut manager = BudgetManager::new();
        let category_id = Uuid::new_v4();

        let budget = manager
            .create_budget(
                "测试预算".to_string(),
                category_id,
                1000.0,
                BudgetPeriod::Monthly,
                Local::now().date_naive(),
                None,
                None,
            )
            .unwrap();

        assert_eq!(budget.name, "测试预算");
        assert_eq!(budget.amount, 1000.0);
        assert_eq!(manager.get_budget_usage(budget.id), 0.0);
    }

    #[test]
    fn test_invalid_budget_amount() {
        let mut manager = BudgetManager::new();
        let category_id = Uuid::new_v4();

        let result = manager.create_budget(
            "无效预算".to_string(),
            category_id,
            -100.0,
            BudgetPeriod::Monthly,
            Local::now().date_naive(),
            None,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_budget_status() {
        let mut manager = BudgetManager::new();
        let budget = Budget::new(
            "测试预算".to_string(),
            Uuid::new_v4(),
            1000.0,
            "CNY".to_string(),
            BudgetPeriod::Monthly,
            Local::now().date_naive(),
        );

        // 测试正常状态
        manager.update_budget_usage(budget.id, 300.0).unwrap();
        if let BudgetStatus::OnTrack {
            usage_percentage, ..
        } = manager.check_budget_status(&budget)
        {
            assert_eq!(usage_percentage, 30.0);
        } else {
            panic!("预期状态为 OnTrack");
        }

        // 测试警告状态
        manager.update_budget_usage(budget.id, 800.0).unwrap();
        if let BudgetStatus::Warning {
            usage_percentage, ..
        } = manager.check_budget_status(&budget)
        {
            assert_eq!(usage_percentage, 80.0);
        } else {
            panic!("预期状态为 Warning");
        }

        // 测试超预算状态
        manager.update_budget_usage(budget.id, 1200.0).unwrap();
        if let BudgetStatus::OverBudget { over_amount, .. } = manager.check_budget_status(&budget) {
            assert_eq!(over_amount, 200.0);
        } else {
            panic!("预期状态为 OverBudget");
        }
    }

    #[test]
    fn test_budget_usage_operations() {
        let mut manager = BudgetManager::new();
        let budget_id = Uuid::new_v4();

        // 设置初始使用情况
        manager.update_budget_usage(budget_id, 100.0).unwrap();
        assert_eq!(manager.get_budget_usage(budget_id), 100.0);

        // 增加使用金额
        manager.add_budget_usage(budget_id, 50.0).unwrap();
        assert_eq!(manager.get_budget_usage(budget_id), 150.0);

        // 减少使用金额
        manager.subtract_budget_usage(budget_id, 30.0).unwrap();
        assert_eq!(manager.get_budget_usage(budget_id), 120.0);

        // 减少到负数应该设为0
        manager.subtract_budget_usage(budget_id, 200.0).unwrap();
        assert_eq!(manager.get_budget_usage(budget_id), 0.0);
    }

    #[test]
    fn test_budget_warnings() {
        let mut manager = BudgetManager::new();

        let budget1 = Budget::new(
            "正常预算".to_string(),
            Uuid::new_v4(),
            1000.0,
            "CNY".to_string(),
            BudgetPeriod::Monthly,
            Local::now().date_naive(),
        );

        let budget2 = Budget::new(
            "超支预算".to_string(),
            Uuid::new_v4(),
            1000.0,
            "CNY".to_string(),
            BudgetPeriod::Monthly,
            Local::now().date_naive(),
        );

        // 设置使用情况
        manager.update_budget_usage(budget1.id, 300.0).unwrap(); // 30% - 正常
        manager.update_budget_usage(budget2.id, 1200.0).unwrap(); // 120% - 超支

        let budgets = vec![budget1, budget2];
        let warnings = manager.get_budget_warnings(&budgets);

        // 应该只有一个警告（超支的预算）
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("超支"));
    }
}
