//! # 数据验证器模块
//!
//! 负责验证数据的完整性、格式和一致性

use super::types::*;
use crate::errors::{AppError, Result};
use std::collections::HashSet;

/// 数据验证器
pub struct DataValidator;

impl DataValidator {
    /// 创建新的数据验证器
    pub fn new() -> Self {
        Self
    }

    /// 验证数据完整性
    pub async fn verify_data_integrity(
        &self,
        data: &serde_json::Value,
    ) -> Result<DataIntegrityReport> {
        let mut report = DataIntegrityReport::new();

        // 验证任务数据
        if let Some(tasks) = data.get("tasks") {
            if let Some(tasks_array) = tasks.as_array() {
                for (i, task) in tasks_array.iter().enumerate() {
                    if let Err(e) = self.validate_task_format(task) {
                        report.add_error(format!("任务 {} 格式错误: {}", i, e));
                    }
                }
                report.task_count = tasks_array.len();
            }
        }

        // 验证分类数据
        if let Some(categories) = data.get("categories") {
            if let Some(categories_array) = categories.as_array() {
                for (i, category) in categories_array.iter().enumerate() {
                    if let Err(e) = self.validate_category_format(category) {
                        report.add_error(format!("分类 {} 格式错误: {}", i, e));
                    }
                }
                report.category_count = categories_array.len();
            }
        }

        // 验证时间记录数据
        if let Some(time_entries) = data.get("time_entries") {
            if let Some(time_entries_array) = time_entries.as_array() {
                for (i, entry) in time_entries_array.iter().enumerate() {
                    if let Err(e) = self.validate_time_entry_format(entry) {
                        report.add_error(format!("时间记录 {} 格式错误: {}", i, e));
                    }
                }
                report.time_entry_count = time_entries_array.len();
            }
        }

        // 验证账户数据
        if let Some(accounts) = data.get("accounts") {
            if let Some(accounts_array) = accounts.as_array() {
                for (i, account) in accounts_array.iter().enumerate() {
                    if let Err(e) = self.validate_account_format(account) {
                        report.add_error(format!("账户 {} 格式错误: {}", i, e));
                    }
                }
                report.account_count = accounts_array.len();
            }
        }

        // 验证交易数据
        if let Some(transactions) = data.get("transactions") {
            if let Some(transactions_array) = transactions.as_array() {
                for (i, transaction) in transactions_array.iter().enumerate() {
                    if let Err(e) = self.validate_transaction_format(transaction) {
                        report.add_error(format!("交易 {} 格式错误: {}", i, e));
                    }
                }
                report.transaction_count = transactions_array.len();
            }
        }

        // 验证引用完整性
        self.verify_reference_integrity(data, &mut report)?;

        report.is_valid = report.errors.is_empty();

        Ok(report)
    }

    /// 验证引用完整性
    fn verify_reference_integrity(
        &self,
        data: &serde_json::Value,
        report: &mut DataIntegrityReport,
    ) -> Result<()> {
        // 收集所有分类ID
        let mut category_ids = HashSet::new();
        if let Some(categories) = data.get("categories") {
            if let Some(categories_array) = categories.as_array() {
                for category in categories_array {
                    if let Some(id) = category.get("id").and_then(|v| v.as_str()) {
                        category_ids.insert(id.to_string());
                    }
                }
            }
        }

        // 收集所有任务ID
        let mut task_ids = HashSet::new();
        if let Some(tasks) = data.get("tasks") {
            if let Some(tasks_array) = tasks.as_array() {
                for task in tasks_array {
                    if let Some(id) = task.get("id").and_then(|v| v.as_str()) {
                        task_ids.insert(id.to_string());
                    }
                }
            }
        }

        // 收集所有账户ID
        let mut account_ids = HashSet::new();
        if let Some(accounts) = data.get("accounts") {
            if let Some(accounts_array) = accounts.as_array() {
                for account in accounts_array {
                    if let Some(id) = account.get("id").and_then(|v| v.as_str()) {
                        account_ids.insert(id.to_string());
                    }
                }
            }
        }

        // 验证时间记录中的任务和分类引用
        if let Some(time_entries) = data.get("time_entries") {
            if let Some(time_entries_array) = time_entries.as_array() {
                for entry in time_entries_array {
                    // 检查任务引用
                    if let Some(task_id) = entry.get("task_id").and_then(|v| v.as_str()) {
                        if !task_ids.contains(task_id) {
                            report.add_error(format!("时间记录引用了不存在的任务ID: {}", task_id));
                        }
                    }

                    // 检查分类引用（如果有的话）
                    if let Some(category_id) = entry.get("category_id").and_then(|v| v.as_str()) {
                        if !category_ids.contains(category_id) {
                            report.add_error(format!(
                                "时间记录引用了不存在的分类ID: {}",
                                category_id
                            ));
                        }
                    }
                }
            }
        }

        // 验证交易中的账户引用
        if let Some(transactions) = data.get("transactions") {
            if let Some(transactions_array) = transactions.as_array() {
                for transaction in transactions_array {
                    if let Some(account_id) = transaction.get("account_id").and_then(|v| v.as_str())
                    {
                        if !account_ids.contains(account_id) {
                            report.add_error(format!("交易引用了不存在的账户ID: {}", account_id));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 验证任务格式
    pub fn validate_task_format(&self, task: &serde_json::Value) -> Result<()> {
        if !task.is_object() {
            return Err(AppError::Sync("任务不是对象格式".to_string()));
        }

        // 检查必要字段
        let required_fields = ["id", "name", "created_at"];
        for field in &required_fields {
            if task.get(field).is_none() {
                return Err(AppError::Sync(format!("缺少必要字段: {}", field)));
            }
        }

        // 验证ID格式
        if let Some(id) = task.get("id") {
            if let Some(id_str) = id.as_str() {
                if uuid::Uuid::parse_str(id_str).is_err() {
                    return Err(AppError::Sync("ID格式无效".to_string()));
                }
            }
        }

        // 验证名称不为空
        if let Some(name) = task.get("name") {
            if let Some(name_str) = name.as_str() {
                if name_str.trim().is_empty() {
                    return Err(AppError::Sync("任务名称不能为空".to_string()));
                }
            }
        }

        // 验证时间格式
        if let Some(created_at) = task.get("created_at") {
            if let Some(time_str) = created_at.as_str() {
                if chrono::DateTime::parse_from_rfc3339(time_str).is_err() {
                    return Err(AppError::Sync("创建时间格式无效".to_string()));
                }
            }
        }

        Ok(())
    }

    /// 验证分类格式
    pub fn validate_category_format(&self, category: &serde_json::Value) -> Result<()> {
        if !category.is_object() {
            return Err(AppError::Sync("分类不是对象格式".to_string()));
        }

        // 检查必要字段
        let required_fields = ["id", "name", "color", "created_at"];
        for field in &required_fields {
            if category.get(field).is_none() {
                return Err(AppError::Sync(format!("缺少必要字段: {}", field)));
            }
        }

        // 验证ID格式
        if let Some(id) = category.get("id") {
            if let Some(id_str) = id.as_str() {
                if uuid::Uuid::parse_str(id_str).is_err() {
                    return Err(AppError::Sync("分类ID格式无效".to_string()));
                }
            }
        }

        // 验证名称不为空
        if let Some(name) = category.get("name") {
            if let Some(name_str) = name.as_str() {
                if name_str.trim().is_empty() {
                    return Err(AppError::Sync("分类名称不能为空".to_string()));
                }
            }
        }

        // 验证颜色格式
        if let Some(color) = category.get("color") {
            if let Some(color_str) = color.as_str() {
                if !color_str.starts_with('#') || color_str.len() != 7 {
                    return Err(AppError::Sync(
                        "分类颜色格式无效，应为#RRGGBB格式".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// 验证时间记录格式
    pub fn validate_time_entry_format(&self, entry: &serde_json::Value) -> Result<()> {
        let required_fields = ["id", "task_name", "start_time"];

        for field in &required_fields {
            if !entry.get(field).is_some() {
                return Err(AppError::Sync(format!("时间记录缺少必要字段: {}", field)));
            }
        }

        // 验证ID格式
        if let Some(id) = entry.get("id") {
            if let Some(id_str) = id.as_str() {
                if uuid::Uuid::parse_str(id_str).is_err() {
                    return Err(AppError::Sync("时间记录ID格式无效".to_string()));
                }
            }
        }

        // 验证任务名称不为空
        if let Some(task_name) = entry.get("task_name") {
            if let Some(task_name_str) = task_name.as_str() {
                if task_name_str.trim().is_empty() {
                    return Err(AppError::Sync("任务名称不能为空".to_string()));
                }
            }
        }

        // 验证开始时间格式
        if let Some(start_time) = entry.get("start_time") {
            if let Some(time_str) = start_time.as_str() {
                if chrono::DateTime::parse_from_rfc3339(time_str).is_err() {
                    return Err(AppError::Sync("开始时间格式无效".to_string()));
                }
            }
        }

        // 验证结束时间格式（如果存在）
        if let Some(end_time) = entry.get("end_time") {
            if let Some(time_str) = end_time.as_str() {
                if chrono::DateTime::parse_from_rfc3339(time_str).is_err() {
                    return Err(AppError::Sync("结束时间格式无效".to_string()));
                }
            }

            // 验证时间逻辑
            if let (Some(start_str), Some(end_str)) = (
                entry.get("start_time").and_then(|v| v.as_str()),
                entry.get("end_time").and_then(|v| v.as_str()),
            ) {
                if let (Ok(start), Ok(end)) = (
                    chrono::DateTime::parse_from_rfc3339(start_str),
                    chrono::DateTime::parse_from_rfc3339(end_str),
                ) {
                    if end <= start {
                        return Err(AppError::Sync("结束时间必须晚于开始时间".to_string()));
                    }
                }
            }
        }

        Ok(())
    }

    /// 验证账户格式
    fn validate_account_format(&self, account: &serde_json::Value) -> Result<()> {
        if !account.is_object() {
            return Err(AppError::Sync("账户不是对象格式".to_string()));
        }

        let required_fields = ["id", "name", "account_type", "created_at"];
        for field in &required_fields {
            if account.get(field).is_none() {
                return Err(AppError::Sync(format!("账户缺少必要字段: {}", field)));
            }
        }

        // 验证ID格式
        if let Some(id) = account.get("id") {
            if let Some(id_str) = id.as_str() {
                if uuid::Uuid::parse_str(id_str).is_err() {
                    return Err(AppError::Sync("账户ID格式无效".to_string()));
                }
            }
        }

        // 验证名称不为空
        if let Some(name) = account.get("name") {
            if let Some(name_str) = name.as_str() {
                if name_str.trim().is_empty() {
                    return Err(AppError::Sync("账户名称不能为空".to_string()));
                }
            }
        }

        // 验证账户类型
        if let Some(account_type) = account.get("account_type") {
            if let Some(type_str) = account_type.as_str() {
                let valid_types = ["cash", "bank", "credit_card", "investment", "other"];
                if !valid_types.contains(&type_str) {
                    return Err(AppError::Sync(format!(
                        "账户类型无效: {}，有效类型: {:?}",
                        type_str, valid_types
                    )));
                }
            }
        }

        Ok(())
    }

    /// 验证交易格式
    fn validate_transaction_format(&self, transaction: &serde_json::Value) -> Result<()> {
        if !transaction.is_object() {
            return Err(AppError::Sync("交易不是对象格式".to_string()));
        }

        let required_fields = [
            "id",
            "account_id",
            "amount",
            "transaction_type",
            "transaction_date",
            "description",
        ];
        for field in &required_fields {
            if transaction.get(field).is_none() {
                return Err(AppError::Sync(format!("交易缺少必要字段: {}", field)));
            }
        }

        // 验证ID格式
        if let Some(id) = transaction.get("id") {
            if let Some(id_str) = id.as_str() {
                if uuid::Uuid::parse_str(id_str).is_err() {
                    return Err(AppError::Sync("交易ID格式无效".to_string()));
                }
            }
        }

        // 验证账户ID格式
        if let Some(account_id) = transaction.get("account_id") {
            if let Some(account_id_str) = account_id.as_str() {
                if uuid::Uuid::parse_str(account_id_str).is_err() {
                    return Err(AppError::Sync("交易账户ID格式无效".to_string()));
                }
            }
        }

        // 验证金额
        if let Some(amount) = transaction.get("amount") {
            if !amount.is_number() {
                return Err(AppError::Sync("交易金额必须是数字".to_string()));
            }
        }

        // 验证交易类型
        if let Some(transaction_type) = transaction.get("transaction_type") {
            if let Some(type_str) = transaction_type.as_str() {
                let valid_types = ["income", "expense", "transfer"];
                if !valid_types.contains(&type_str) {
                    return Err(AppError::Sync(format!(
                        "交易类型无效: {}，有效类型: {:?}",
                        type_str, valid_types
                    )));
                }
            }
        }

        // 验证日期格式
        if let Some(date) = transaction.get("transaction_date") {
            if let Some(date_str) = date.as_str() {
                if chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err() {
                    return Err(AppError::Sync(
                        "交易日期格式无效，应为YYYY-MM-DD".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// 验证导入数据的完整性
    pub fn validate_data(&self, data: &[u8]) -> Result<bool> {
        log::info!("验证数据完整性，大小: {} 字节", data.len());

        // 1. 检查数据是否为空
        if data.is_empty() {
            return Err(AppError::Sync("导入数据为空".to_string()));
        }

        // 2. 检查数据大小是否合理
        if data.len() > 100 * 1024 * 1024 {
            // 100MB限制
            return Err(AppError::Sync("导入数据过大，超过100MB限制".to_string()));
        }

        // 3. 尝试解析JSON数据
        let parsed_data: serde_json::Value = serde_json::from_slice(data)
            .map_err(|e| AppError::Sync(format!("数据解析失败: {}", e)))?;

        // 4. 检查数据结构
        if !parsed_data.is_object() {
            return Err(AppError::Sync(
                "数据格式错误，不是有效的JSON对象".to_string(),
            ));
        }

        // 5. 验证版本信息
        if let Some(version) = parsed_data.get("version") {
            if let Some(version_str) = version.as_str() {
                if !self.is_compatible_version(version_str) {
                    return Err(AppError::Sync(format!("数据版本不兼容: {}", version_str)));
                }
            }
        }

        // 6. 验证导出时间
        if let Some(export_time) = parsed_data.get("export_time") {
            if export_time.as_str().is_none() {
                log::warn!("导出时间格式无效");
            }
        }

        // 7. 验证必要字段存在
        let required_fields = [
            "tasks",
            "categories",
            "time_entries",
            "transactions",
            "accounts",
        ];
        for field in &required_fields {
            if let Some(field_data) = parsed_data.get(field) {
                if !field_data.is_array() {
                    return Err(AppError::Sync(format!("字段 {} 不是数组格式", field)));
                }
            }
        }

        // 8. 验证数据记录数量
        let total_records = self.count_total_records(&parsed_data)?;
        if total_records > 1_000_000 {
            // 100万条记录限制
            return Err(AppError::Sync(format!(
                "数据记录过多: {} 条，超过100万条限制",
                total_records
            )));
        }

        // 9. 快速验证数据格式
        self.validate_data_format(&parsed_data)?;

        log::info!("数据验证通过，包含 {} 条记录", total_records);
        Ok(true)
    }

    /// 验证数据格式
    fn validate_data_format(&self, data: &serde_json::Value) -> Result<()> {
        // 验证任务数据格式
        if let Some(tasks) = data.get("tasks") {
            if let Some(tasks_array) = tasks.as_array() {
                for (i, task) in tasks_array.iter().enumerate() {
                    if let Err(e) = self.validate_task_format(task) {
                        return Err(AppError::Sync(format!("任务 {} 格式错误: {}", i, e)));
                    }
                }
            }
        }

        // 验证分类数据格式
        if let Some(categories) = data.get("categories") {
            if let Some(categories_array) = categories.as_array() {
                for (i, category) in categories_array.iter().enumerate() {
                    if let Err(e) = self.validate_category_format(category) {
                        return Err(AppError::Sync(format!("分类 {} 格式错误: {}", i, e)));
                    }
                }
            }
        }

        // 验证时间记录数据格式
        if let Some(time_entries) = data.get("time_entries") {
            if let Some(time_entries_array) = time_entries.as_array() {
                for (i, entry) in time_entries_array.iter().enumerate() {
                    if let Err(e) = self.validate_time_entry_format(entry) {
                        return Err(AppError::Sync(format!("时间记录 {} 格式错误: {}", i, e)));
                    }
                }
            }
        }

        Ok(())
    }

    /// 计算总记录数
    fn count_total_records(&self, data: &serde_json::Value) -> Result<usize> {
        let mut total = 0;

        if let Some(tasks) = data.get("tasks") {
            if let Some(tasks_array) = tasks.as_array() {
                total += tasks_array.len();
            }
        }

        if let Some(categories) = data.get("categories") {
            if let Some(categories_array) = categories.as_array() {
                total += categories_array.len();
            }
        }

        if let Some(time_entries) = data.get("time_entries") {
            if let Some(time_entries_array) = time_entries.as_array() {
                total += time_entries_array.len();
            }
        }

        if let Some(transactions) = data.get("transactions") {
            if let Some(transactions_array) = transactions.as_array() {
                total += transactions_array.len();
            }
        }

        if let Some(accounts) = data.get("accounts") {
            if let Some(accounts_array) = accounts.as_array() {
                total += accounts_array.len();
            }
        }

        Ok(total)
    }

    /// 检查版本兼容性
    fn is_compatible_version(&self, version: &str) -> bool {
        // 当前版本
        let current_version = env!("CARGO_PKG_VERSION");

        // 简单的版本检查，实际应该使用更精确的版本比较
        if version == current_version {
            return true;
        }

        // 允许同一大版本的不同小版本
        let current_major = current_version.split('.').next().unwrap_or("0");
        let import_major = version.split('.').next().unwrap_or("0");

        current_major == import_major
    }

    /// 验证数据一致性
    pub fn validate_consistency(&self, data: &serde_json::Value) -> Result<Vec<String>> {
        let mut inconsistencies = Vec::new();

        // 检查数据字段一致性
        self.check_field_consistency(data, &mut inconsistencies);

        // 检查业务逻辑一致性
        self.check_business_logic(data, &mut inconsistencies);

        // 检查数据关系一致性
        self.check_relationship_consistency(data, &mut inconsistencies);

        Ok(inconsistencies)
    }

    /// 检查字段一致性
    fn check_field_consistency(&self, data: &serde_json::Value, inconsistencies: &mut Vec<String>) {
        // 检查必要字段是否存在
        let expected_fields = [
            "tasks",
            "categories",
            "time_entries",
            "accounts",
            "transactions",
        ];
        for field in &expected_fields {
            if data.get(field).is_none() {
                inconsistencies.push(format!("缺少必要字段: {}", field));
            }
        }

        // 检查字段类型
        for field in &expected_fields {
            if let Some(value) = data.get(field) {
                if !value.is_array() {
                    inconsistencies.push(format!("字段 {} 应该是数组类型", field));
                }
            }
        }
    }

    /// 检查业务逻辑一致性
    fn check_business_logic(&self, data: &serde_json::Value, inconsistencies: &mut Vec<String>) {
        // 检查时间记录的时间逻辑
        if let Some(time_entries) = data.get("time_entries").and_then(|v| v.as_array()) {
            for (i, entry) in time_entries.iter().enumerate() {
                if let (Some(start_str), Some(end_str)) = (
                    entry.get("start_time").and_then(|v| v.as_str()),
                    entry.get("end_time").and_then(|v| v.as_str()),
                ) {
                    if let (Ok(start), Ok(end)) = (
                        chrono::DateTime::parse_from_rfc3339(start_str),
                        chrono::DateTime::parse_from_rfc3339(end_str),
                    ) {
                        if end <= start {
                            inconsistencies
                                .push(format!("时间记录 {} 的结束时间不能早于或等于开始时间", i));
                        }

                        // 检查时长是否合理（不超过24小时）
                        let duration = end.signed_duration_since(start);
                        if duration.num_hours() > 24 {
                            inconsistencies
                                .push(format!("时间记录 {} 的时长超过24小时，可能不合理", i));
                        }
                    }
                }
            }
        }

        // 检查交易金额逻辑
        if let Some(transactions) = data.get("transactions").and_then(|v| v.as_array()) {
            for (i, transaction) in transactions.iter().enumerate() {
                if let Some(amount) = transaction.get("amount").and_then(|v| v.as_f64()) {
                    if amount == 0.0 {
                        inconsistencies.push(format!("交易 {} 的金额为0，可能不合理", i));
                    }
                }
            }
        }
    }

    /// 检查数据关系一致性
    fn check_relationship_consistency(
        &self,
        data: &serde_json::Value,
        inconsistencies: &mut Vec<String>,
    ) {
        // 这个方法基本上和 verify_reference_integrity 类似，但返回错误列表而不是修改报告
        let mut category_ids = HashSet::new();
        if let Some(categories) = data.get("categories").and_then(|v| v.as_array()) {
            for category in categories {
                if let Some(id) = category.get("id").and_then(|v| v.as_str()) {
                    category_ids.insert(id.to_string());
                }
            }
        }

        let mut task_ids = HashSet::new();
        if let Some(tasks) = data.get("tasks").and_then(|v| v.as_array()) {
            for task in tasks {
                if let Some(id) = task.get("id").and_then(|v| v.as_str()) {
                    task_ids.insert(id.to_string());
                }
            }
        }

        let mut account_ids = HashSet::new();
        if let Some(accounts) = data.get("accounts").and_then(|v| v.as_array()) {
            for account in accounts {
                if let Some(id) = account.get("id").and_then(|v| v.as_str()) {
                    account_ids.insert(id.to_string());
                }
            }
        }

        // 检查时间记录引用
        if let Some(time_entries) = data.get("time_entries").and_then(|v| v.as_array()) {
            for (i, entry) in time_entries.iter().enumerate() {
                if let Some(task_id) = entry.get("task_id").and_then(|v| v.as_str()) {
                    if !task_ids.contains(task_id) {
                        inconsistencies
                            .push(format!("时间记录 {} 引用了不存在的任务ID: {}", i, task_id));
                    }
                }
            }
        }

        // 检查交易引用
        if let Some(transactions) = data.get("transactions").and_then(|v| v.as_array()) {
            for (i, transaction) in transactions.iter().enumerate() {
                if let Some(account_id) = transaction.get("account_id").and_then(|v| v.as_str()) {
                    if !account_ids.contains(account_id) {
                        inconsistencies
                            .push(format!("交易 {} 引用了不存在的账户ID: {}", i, account_id));
                    }
                }
            }
        }
    }
}

impl Default for DataValidator {
    fn default() -> Self {
        Self::new()
    }
}
