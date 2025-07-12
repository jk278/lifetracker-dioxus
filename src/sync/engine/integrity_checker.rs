//! # 数据完整性检查器
//!
//! 负责检查数据完整性、评估数据丢失风险、分类操作安全级别

use crate::errors::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 数据完整性统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataIntegrityStats {
    /// 任务数量
    pub tasks: usize,
    /// 时间记录数量
    pub time_entries: usize,
    /// 分类数量
    pub categories: usize,
    /// 账户数量
    pub accounts: usize,
    /// 交易数量
    pub transactions: usize,
    /// 数据大小（字节）
    pub data_size: usize,
    /// 关键字段完整性
    pub key_fields_integrity: HashMap<String, bool>,
    /// 数据关系完整性
    pub relationship_integrity: HashMap<String, bool>,
}

/// 数据丢失风险评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLossRiskAssessment {
    /// 风险级别
    pub risk_level: RiskLevel,
    /// 风险分数 (0-100)
    pub risk_score: u8,
    /// 潜在丢失的数据量
    pub potential_loss: DataLossInfo,
    /// 风险因素
    pub risk_factors: Vec<String>,
    /// 建议操作
    pub recommended_action: RecommendedAction,
}

/// 风险级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    /// 安全操作
    Safe,
    /// 需要确认
    NeedsConfirmation,
    /// 高风险
    HighRisk,
    /// 危险操作
    Dangerous,
}

/// 潜在数据丢失信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLossInfo {
    /// 可能丢失的任务数量
    pub tasks: usize,
    /// 可能丢失的时间记录数量
    pub time_entries: usize,
    /// 可能丢失的交易数量
    pub transactions: usize,
    /// 可能丢失的数据大小
    pub data_size: usize,
    /// 数据价值评估（基于记录数量和时间跨度）
    pub data_value_score: u8,
}

/// 推荐操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    /// 自动处理
    AutoProcess,
    /// 用户确认后处理
    UserConfirmation,
    /// 需要详细确认
    DetailedConfirmation,
    /// 强制备份后处理
    ForceBackupFirst,
    /// 拒绝操作
    RefuseOperation,
}

/// 冲突检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetectionResult {
    /// 是否存在冲突
    pub has_conflict: bool,
    /// 冲突类型
    pub conflict_type: ConflictType,
    /// 本地数据统计
    pub local_stats: DataIntegrityStats,
    /// 远程数据统计
    pub remote_stats: DataIntegrityStats,
    /// 风险评估
    pub risk_assessment: DataLossRiskAssessment,
    /// 详细说明
    pub description: String,
    /// 用户友好的说明
    pub user_message: String,
}

/// 冲突类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    /// 无冲突
    None,
    /// 数据量不匹配
    DataVolumeConflict,
    /// 数据完整性问题
    DataIntegrityConflict,
    /// 数据丢失风险
    DataLossRisk,
    /// 结构不兼容
    StructuralConflict,
    /// 时间戳冲突
    TimestampConflict,
}

/// 数据完整性检查器
pub struct DataIntegrityChecker;

impl DataIntegrityChecker {
    /// 创建新的数据完整性检查器
    pub fn new() -> Self {
        Self
    }

    /// 执行完整的冲突检测
    pub fn detect_conflicts(
        &self,
        local_data: &serde_json::Value,
        remote_data: &serde_json::Value,
    ) -> Result<ConflictDetectionResult> {
        log::info!("=== 开始执行完整的冲突检测 ===");

        // 1. 生成数据统计
        let local_stats = self.generate_data_stats(local_data)?;
        let remote_stats = self.generate_data_stats(remote_data)?;

        log::info!(
            "本地数据统计: 任务={}, 时间记录={}, 交易={}, 大小={}字节",
            local_stats.tasks,
            local_stats.time_entries,
            local_stats.transactions,
            local_stats.data_size
        );
        log::info!(
            "远程数据统计: 任务={}, 时间记录={}, 交易={}, 大小={}字节",
            remote_stats.tasks,
            remote_stats.time_entries,
            remote_stats.transactions,
            remote_stats.data_size
        );

        // 2. 检查数据完整性
        let integrity_issues = self.check_data_integrity(&local_stats, &remote_stats)?;

        // 3. 评估数据丢失风险
        let risk_assessment = self.assess_data_loss_risk(&local_stats, &remote_stats)?;

        // 4. 确定冲突类型
        let conflict_type =
            self.determine_conflict_type(&local_stats, &remote_stats, &risk_assessment)?;

        // 5. 生成用户友好的说明
        let (description, user_message) = self.generate_conflict_description(
            &conflict_type,
            &local_stats,
            &remote_stats,
            &risk_assessment,
        )?;

        let result = ConflictDetectionResult {
            has_conflict: conflict_type != ConflictType::None,
            conflict_type,
            local_stats,
            remote_stats,
            risk_assessment,
            description,
            user_message,
        };

        log::info!(
            "冲突检测结果: 冲突={}, 类型={:?}, 风险级别={:?}",
            result.has_conflict,
            result.conflict_type,
            result.risk_assessment.risk_level
        );

        Ok(result)
    }

    /// 生成数据统计信息
    fn generate_data_stats(&self, data: &serde_json::Value) -> Result<DataIntegrityStats> {
        let tasks = data
            .get("tasks")
            .and_then(|v| v.as_array())
            .map_or(0, |arr| arr.len());
        let time_entries = data
            .get("time_entries")
            .and_then(|v| v.as_array())
            .map_or(0, |arr| arr.len());
        let categories = data
            .get("categories")
            .and_then(|v| v.as_array())
            .map_or(0, |arr| arr.len());
        let accounts = data
            .get("accounts")
            .and_then(|v| v.as_array())
            .map_or(0, |arr| arr.len());
        let transactions = data
            .get("transactions")
            .and_then(|v| v.as_array())
            .map_or(0, |arr| arr.len());
        let data_size = data.to_string().len();

        // 检查关键字段完整性
        let mut key_fields_integrity = HashMap::new();
        key_fields_integrity.insert("tasks".to_string(), data.get("tasks").is_some());
        key_fields_integrity.insert(
            "time_entries".to_string(),
            data.get("time_entries").is_some(),
        );
        key_fields_integrity.insert("categories".to_string(), data.get("categories").is_some());
        key_fields_integrity.insert("accounts".to_string(), data.get("accounts").is_some());
        key_fields_integrity.insert(
            "transactions".to_string(),
            data.get("transactions").is_some(),
        );
        key_fields_integrity.insert("export_time".to_string(), data.get("export_time").is_some());

        // 检查数据关系完整性
        let mut relationship_integrity = HashMap::new();
        relationship_integrity.insert(
            "task_category_refs".to_string(),
            self.check_task_category_refs(data)?,
        );
        relationship_integrity.insert(
            "time_entry_task_refs".to_string(),
            self.check_time_entry_refs(data)?,
        );
        relationship_integrity.insert(
            "transaction_account_refs".to_string(),
            self.check_transaction_refs(data)?,
        );

        Ok(DataIntegrityStats {
            tasks,
            time_entries,
            categories,
            accounts,
            transactions,
            data_size,
            key_fields_integrity,
            relationship_integrity,
        })
    }

    /// 检查任务-分类引用完整性
    fn check_task_category_refs(&self, data: &serde_json::Value) -> Result<bool> {
        let empty_tasks = vec![];
        let empty_categories = vec![];
        let tasks = data
            .get("tasks")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_tasks);
        let categories = data
            .get("categories")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_categories);

        let category_ids: std::collections::HashSet<_> = categories
            .iter()
            .filter_map(|cat| cat.get("id").and_then(|id| id.as_str()))
            .collect();

        for task in tasks {
            if let Some(category_id) = task.get("category_id").and_then(|id| id.as_str()) {
                if !category_ids.contains(category_id) {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// 检查时间记录-任务引用完整性
    fn check_time_entry_refs(&self, data: &serde_json::Value) -> Result<bool> {
        let empty_time_entries = vec![];
        let empty_tasks = vec![];
        let time_entries = data
            .get("time_entries")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_time_entries);
        let tasks = data
            .get("tasks")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_tasks);

        let task_ids: std::collections::HashSet<_> = tasks
            .iter()
            .filter_map(|task| task.get("id").and_then(|id| id.as_str()))
            .collect();

        for entry in time_entries {
            if let Some(task_id) = entry.get("task_id").and_then(|id| id.as_str()) {
                if !task_ids.contains(task_id) {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// 检查交易-账户引用完整性
    fn check_transaction_refs(&self, data: &serde_json::Value) -> Result<bool> {
        let empty_transactions = vec![];
        let empty_accounts = vec![];
        let transactions = data
            .get("transactions")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_transactions);
        let accounts = data
            .get("accounts")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_accounts);

        let account_ids: std::collections::HashSet<_> = accounts
            .iter()
            .filter_map(|acc| acc.get("id").and_then(|id| id.as_str()))
            .collect();

        for transaction in transactions {
            if let Some(account_id) = transaction.get("account_id").and_then(|id| id.as_str()) {
                if !account_ids.contains(account_id) {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// 检查数据完整性
    fn check_data_integrity(
        &self,
        local_stats: &DataIntegrityStats,
        remote_stats: &DataIntegrityStats,
    ) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // 检查关键字段缺失
        for (field, &exists) in &local_stats.key_fields_integrity {
            if !exists {
                issues.push(format!("本地数据缺失关键字段: {}", field));
            }
        }

        for (field, &exists) in &remote_stats.key_fields_integrity {
            if !exists {
                issues.push(format!("远程数据缺失关键字段: {}", field));
            }
        }

        // 检查关系完整性
        for (relation, &valid) in &local_stats.relationship_integrity {
            if !valid {
                issues.push(format!("本地数据关系完整性问题: {}", relation));
            }
        }

        for (relation, &valid) in &remote_stats.relationship_integrity {
            if !valid {
                issues.push(format!("远程数据关系完整性问题: {}", relation));
            }
        }

        Ok(issues)
    }

    /// 评估数据丢失风险
    fn assess_data_loss_risk(
        &self,
        local_stats: &DataIntegrityStats,
        remote_stats: &DataIntegrityStats,
    ) -> Result<DataLossRiskAssessment> {
        let mut risk_factors = Vec::new();
        let mut risk_score = 0u8;

        // 计算数据量差异
        let local_total = local_stats.tasks + local_stats.time_entries + local_stats.transactions;
        let remote_total =
            remote_stats.tasks + remote_stats.time_entries + remote_stats.transactions;

        // 数据量差异风险评估
        if local_total == 0 && remote_total > 0 {
            risk_factors.push("本地数据为空，远程有数据".to_string());
            risk_score += 40;
        } else if remote_total == 0 && local_total > 0 {
            risk_factors.push("远程数据为空，本地有数据".to_string());
            risk_score += 10;
        } else if local_total > 0 && remote_total > 0 {
            let ratio = local_total as f64 / remote_total as f64;
            if ratio < 0.3 {
                risk_factors.push(format!(
                    "本地数据量显著少于远程数据 ({}% of remote)",
                    (ratio * 100.0) as u8
                ));
                risk_score += 60;
            } else if ratio < 0.6 {
                risk_factors.push(format!(
                    "本地数据量少于远程数据 ({}% of remote)",
                    (ratio * 100.0) as u8
                ));
                risk_score += 30;
            } else if ratio > 3.0 {
                risk_factors.push(format!(
                    "本地数据量显著多于远程数据 ({}% of remote)",
                    (ratio * 100.0) as u8
                ));
                risk_score += 20;
            }
        }

        // 数据价值评估
        let remote_value = self.calculate_data_value(remote_stats);
        let local_value = self.calculate_data_value(local_stats);

        if remote_value > local_value * 2 {
            risk_factors.push("远程数据价值显著高于本地数据".to_string());
            risk_score += 25;
        }

        // 数据完整性风险
        let local_integrity_issues = local_stats
            .relationship_integrity
            .values()
            .filter(|&&v| !v)
            .count();
        let remote_integrity_issues = remote_stats
            .relationship_integrity
            .values()
            .filter(|&&v| !v)
            .count();

        if local_integrity_issues > 0 {
            risk_factors.push(format!("本地数据完整性问题: {} 个", local_integrity_issues));
            risk_score += local_integrity_issues as u8 * 10;
        }

        if remote_integrity_issues > 0 {
            risk_factors.push(format!(
                "远程数据完整性问题: {} 个",
                remote_integrity_issues
            ));
            risk_score += remote_integrity_issues as u8 * 5;
        }

        // 限制风险分数最大值
        risk_score = risk_score.min(100);

        // 确定风险级别
        let risk_level = match risk_score {
            0..=20 => RiskLevel::Safe,
            21..=40 => RiskLevel::NeedsConfirmation,
            41..=70 => RiskLevel::HighRisk,
            71..=u8::MAX => RiskLevel::Dangerous,
        };

        // 计算潜在损失
        let potential_loss = if local_total < remote_total {
            DataLossInfo {
                tasks: remote_stats.tasks.saturating_sub(local_stats.tasks),
                time_entries: remote_stats
                    .time_entries
                    .saturating_sub(local_stats.time_entries),
                transactions: remote_stats
                    .transactions
                    .saturating_sub(local_stats.transactions),
                data_size: remote_stats.data_size.saturating_sub(local_stats.data_size),
                data_value_score: remote_value.saturating_sub(local_value),
            }
        } else {
            DataLossInfo {
                tasks: 0,
                time_entries: 0,
                transactions: 0,
                data_size: 0,
                data_value_score: 0,
            }
        };

        // 确定推荐操作
        let recommended_action = match risk_level {
            RiskLevel::Safe => RecommendedAction::AutoProcess,
            RiskLevel::NeedsConfirmation => RecommendedAction::UserConfirmation,
            RiskLevel::HighRisk => RecommendedAction::DetailedConfirmation,
            RiskLevel::Dangerous => RecommendedAction::ForceBackupFirst,
        };

        Ok(DataLossRiskAssessment {
            risk_level,
            risk_score,
            potential_loss,
            risk_factors,
            recommended_action,
        })
    }

    /// 计算数据价值分数
    fn calculate_data_value(&self, stats: &DataIntegrityStats) -> u8 {
        // 基于数据量和类型计算价值分数
        let mut value = 0u8;

        // 任务价值
        value += (stats.tasks.min(50) * 2) as u8;

        // 时间记录价值（时间数据很珍贵）
        value += (stats.time_entries.min(100) * 3) as u8;

        // 交易价值（财务数据很重要）
        value += (stats.transactions.min(100) * 4) as u8;

        // 分类和账户价值
        value += (stats.categories.min(20) * 1) as u8;
        value += (stats.accounts.min(20) * 2) as u8;

        value.min(100)
    }

    /// 确定冲突类型
    fn determine_conflict_type(
        &self,
        local_stats: &DataIntegrityStats,
        remote_stats: &DataIntegrityStats,
        risk_assessment: &DataLossRiskAssessment,
    ) -> Result<ConflictType> {
        // 1. 检查数据完整性问题
        let local_integrity_issues = local_stats
            .relationship_integrity
            .values()
            .filter(|&&v| !v)
            .count();
        let remote_integrity_issues = remote_stats
            .relationship_integrity
            .values()
            .filter(|&&v| !v)
            .count();

        if local_integrity_issues > 0 || remote_integrity_issues > 0 {
            return Ok(ConflictType::DataIntegrityConflict);
        }

        // 2. 检查数据丢失风险
        if risk_assessment.risk_level == RiskLevel::Dangerous
            || risk_assessment.risk_level == RiskLevel::HighRisk
        {
            return Ok(ConflictType::DataLossRisk);
        }

        // 3. 检查数据量冲突
        let local_total = local_stats.tasks + local_stats.time_entries + local_stats.transactions;
        let remote_total =
            remote_stats.tasks + remote_stats.time_entries + remote_stats.transactions;

        if local_total > 0 && remote_total > 0 {
            let ratio = local_total as f64 / remote_total as f64;
            if ratio < 0.5 || ratio > 2.0 {
                return Ok(ConflictType::DataVolumeConflict);
            }
        }

        // 4. 检查结构兼容性
        let local_keys: std::collections::HashSet<_> =
            local_stats.key_fields_integrity.keys().collect();
        let remote_keys: std::collections::HashSet<_> =
            remote_stats.key_fields_integrity.keys().collect();

        if !local_keys.is_subset(&remote_keys) || !remote_keys.is_subset(&local_keys) {
            return Ok(ConflictType::StructuralConflict);
        }

        // 5. 如果需要用户确认，标记为时间戳冲突
        if risk_assessment.risk_level == RiskLevel::NeedsConfirmation {
            return Ok(ConflictType::TimestampConflict);
        }

        // 6. 默认无冲突
        Ok(ConflictType::None)
    }

    /// 生成冲突描述
    fn generate_conflict_description(
        &self,
        conflict_type: &ConflictType,
        local_stats: &DataIntegrityStats,
        remote_stats: &DataIntegrityStats,
        risk_assessment: &DataLossRiskAssessment,
    ) -> Result<(String, String)> {
        match conflict_type {
            ConflictType::None => Ok((
                "数据同步检查通过，可以安全同步".to_string(),
                "数据检查通过，可以安全同步".to_string(),
            )),
            ConflictType::DataLossRisk => {
                let description = format!(
                    "检测到数据丢失风险！本地数据：{}个任务，{}个时间记录，{}个交易；远程数据：{}个任务，{}个时间记录，{}个交易。风险分数：{}/100",
                    local_stats.tasks, local_stats.time_entries, local_stats.transactions,
                    remote_stats.tasks, remote_stats.time_entries, remote_stats.transactions,
                    risk_assessment.risk_score
                );
                let user_message = format!(
                    "⚠️ 检测到数据丢失风险！\n\n您的本地数据（{}条记录）显著少于远程数据（{}条记录）。\n\n如果继续同步，可能会丢失：\n• {}个任务\n• {}个时间记录\n• {}个交易记录\n\n建议选择\"合并数据\"或\"使用远程数据\"。",
                    local_stats.tasks + local_stats.time_entries + local_stats.transactions,
                    remote_stats.tasks + remote_stats.time_entries + remote_stats.transactions,
                    risk_assessment.potential_loss.tasks,
                    risk_assessment.potential_loss.time_entries,
                    risk_assessment.potential_loss.transactions
                );
                Ok((description, user_message))
            }
            ConflictType::DataVolumeConflict => {
                let description = format!(
                    "数据量不匹配冲突。本地：{}条记录，远程：{}条记录",
                    local_stats.tasks + local_stats.time_entries + local_stats.transactions,
                    remote_stats.tasks + remote_stats.time_entries + remote_stats.transactions
                );
                let user_message = "检测到数据量差异，建议仔细确认同步方向".to_string();
                Ok((description, user_message))
            }
            ConflictType::DataIntegrityConflict => {
                let description = format!(
                    "数据完整性冲突。风险因素：{}",
                    risk_assessment.risk_factors.join(", ")
                );
                let user_message = "检测到数据完整性问题，建议检查数据后再同步".to_string();
                Ok((description, user_message))
            }
            ConflictType::StructuralConflict => {
                let description = "数据结构不兼容冲突".to_string();
                let user_message = "检测到数据结构差异，可能需要数据迁移".to_string();
                Ok((description, user_message))
            }
            ConflictType::TimestampConflict => {
                let description = "时间戳冲突，需要用户确认同步方向".to_string();
                let user_message = "本地和远程数据都有更新，请选择同步方向".to_string();
                Ok((description, user_message))
            }
        }
    }
}
