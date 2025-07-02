//! # 分类管理核心逻辑
//!
//! 负责交易分类的创建、更新、层级管理等操作

use crate::errors::{AppError, Result};
use crate::storage::{TransactionCategory, TransactionType};
use std::collections::HashMap;
use uuid::Uuid;

/// 分类管理器
#[derive(Debug)]
pub struct CategoryManager {
    /// 默认货币（虽然分类不直接使用货币，但保持一致性）
    default_currency: String,
}

impl CategoryManager {
    /// 创建新的分类管理器
    pub fn new() -> Self {
        Self {
            default_currency: "CNY".to_string(),
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

    /// 创建交易分类
    pub fn create_transaction_category(
        &self,
        name: String,
        transaction_type: TransactionType,
        color: String,
        icon: Option<String>,
        parent_id: Option<Uuid>,
        description: Option<String>,
    ) -> Result<TransactionCategory> {
        if name.trim().is_empty() {
            return Err(AppError::Validation("分类名称不能为空".to_string()));
        }

        let mut category = TransactionCategory::new(name, transaction_type, color);

        if let Some(desc) = description {
            category.description = Some(desc);
        }

        if let Some(icon_name) = icon {
            category.icon = Some(icon_name);
        }

        if let Some(parent) = parent_id {
            category.parent_id = Some(parent);
        }

        log::info!("创建交易分类: {} ({})", category.name, category.id);
        Ok(category)
    }

    /// 验证分类层级
    pub fn validate_category_hierarchy(
        &self,
        category_id: Uuid,
        parent_id: Uuid,
        categories: &[TransactionCategory],
    ) -> Result<()> {
        // 防止循环引用
        if category_id == parent_id {
            return Err(AppError::Validation("分类不能以自己为父分类".to_string()));
        }

        // 检查是否会形成循环
        let mut current_parent = Some(parent_id);
        let mut depth = 0;
        const MAX_DEPTH: i32 = 10; // 防止无限循环

        while let Some(parent) = current_parent {
            depth += 1;
            if depth > MAX_DEPTH {
                return Err(AppError::Validation("分类层级过深".to_string()));
            }

            if parent == category_id {
                return Err(AppError::Validation("分类层级不能形成循环".to_string()));
            }

            current_parent = categories
                .iter()
                .find(|c| c.id == parent)
                .and_then(|c| c.parent_id);
        }

        Ok(())
    }

    /// 获取分类的完整路径
    pub fn get_category_path(
        &self,
        category_id: Uuid,
        categories: &[TransactionCategory],
    ) -> Result<Vec<String>> {
        let mut path = Vec::new();
        let mut current_id = Some(category_id);

        while let Some(id) = current_id {
            if let Some(category) = categories.iter().find(|c| c.id == id) {
                path.insert(0, category.name.clone());
                current_id = category.parent_id;
            } else {
                break;
            }
        }

        if path.is_empty() {
            return Err(AppError::Validation("分类不存在".to_string()));
        }

        Ok(path)
    }

    /// 获取分类的子分类
    pub fn get_subcategories(
        &self,
        parent_id: Uuid,
        categories: &[TransactionCategory],
    ) -> Vec<TransactionCategory> {
        categories
            .iter()
            .filter(|c| c.parent_id == Some(parent_id) && c.is_active)
            .cloned()
            .collect()
    }

    /// 获取顶级分类
    pub fn get_root_categories(
        &self,
        categories: &[TransactionCategory],
        transaction_type: Option<TransactionType>,
    ) -> Vec<TransactionCategory> {
        categories
            .iter()
            .filter(|c| {
                c.parent_id.is_none()
                    && c.is_active
                    && transaction_type.map_or(true, |tt| c.transaction_type == tt)
            })
            .cloned()
            .collect()
    }

    /// 构建分类树
    pub fn build_category_tree(
        &self,
        categories: &[TransactionCategory],
        transaction_type: Option<TransactionType>,
    ) -> Vec<CategoryNode> {
        let root_categories = self.get_root_categories(categories, transaction_type);

        root_categories
            .into_iter()
            .map(|category| self.build_category_node(category, categories))
            .collect()
    }

    /// 构建单个分类节点
    fn build_category_node(
        &self,
        category: TransactionCategory,
        all_categories: &[TransactionCategory],
    ) -> CategoryNode {
        let subcategories = self.get_subcategories(category.id, all_categories);
        let children = subcategories
            .into_iter()
            .map(|sub| self.build_category_node(sub, all_categories))
            .collect();

        CategoryNode { category, children }
    }

    /// 获取分类使用统计
    pub fn get_category_usage_stats(
        &self,
        category_id: Uuid,
        transactions: &[crate::storage::Transaction],
    ) -> CategoryUsageStats {
        let transactions_in_category: Vec<_> = transactions
            .iter()
            .filter(|t| t.category_id == Some(category_id))
            .collect();

        let transaction_count = transactions_in_category.len();
        let total_amount = transactions_in_category.iter().map(|t| t.amount).sum();

        let latest_transaction_date = transactions_in_category
            .iter()
            .map(|t| t.transaction_date)
            .max();

        CategoryUsageStats {
            category_id,
            transaction_count,
            total_amount,
            latest_transaction_date,
        }
    }

    /// 验证分类删除是否安全
    pub fn validate_category_deletion(
        &self,
        category_id: Uuid,
        categories: &[TransactionCategory],
        transactions: &[crate::storage::Transaction],
    ) -> Result<()> {
        // 检查是否有子分类
        let has_subcategories = categories
            .iter()
            .any(|c| c.parent_id == Some(category_id) && c.is_active);

        if has_subcategories {
            return Err(AppError::Validation("不能删除有子分类的分类".to_string()));
        }

        // 检查是否有关联的交易
        let has_transactions = transactions
            .iter()
            .any(|t| t.category_id == Some(category_id));

        if has_transactions {
            return Err(AppError::Validation("不能删除有关联交易的分类".to_string()));
        }

        Ok(())
    }

    /// 迁移分类下的所有交易到新分类
    pub fn migrate_category_transactions(
        &self,
        from_category_id: Uuid,
        to_category_id: Uuid,
        categories: &[TransactionCategory],
    ) -> Result<()> {
        // 验证目标分类存在
        let target_exists = categories
            .iter()
            .any(|c| c.id == to_category_id && c.is_active);

        if !target_exists {
            return Err(AppError::Validation("目标分类不存在".to_string()));
        }

        // 验证分类类型匹配
        let source_type = categories
            .iter()
            .find(|c| c.id == from_category_id)
            .map(|c| c.transaction_type);

        let target_type = categories
            .iter()
            .find(|c| c.id == to_category_id)
            .map(|c| c.transaction_type);

        if source_type != target_type {
            return Err(AppError::Validation(
                "源分类和目标分类的交易类型不匹配".to_string(),
            ));
        }

        log::info!(
            "准备迁移分类 {} 的交易到分类 {}",
            from_category_id,
            to_category_id
        );
        Ok(())
    }

    /// 按交易类型分组分类
    pub fn group_categories_by_type(
        &self,
        categories: &[TransactionCategory],
    ) -> std::collections::HashMap<TransactionType, Vec<TransactionCategory>> {
        let mut groups = std::collections::HashMap::new();

        for category in categories {
            if category.is_active {
                groups
                    .entry(category.transaction_type)
                    .or_insert_with(Vec::new)
                    .push(category.clone());
            }
        }

        groups
    }

    /// 搜索分类
    pub fn search_categories(
        &self,
        query: &str,
        categories: &[TransactionCategory],
        transaction_type: Option<TransactionType>,
    ) -> Vec<TransactionCategory> {
        let query_lower = query.to_lowercase();

        categories
            .iter()
            .filter(|c| {
                c.is_active
                    && c.name.to_lowercase().contains(&query_lower)
                    && transaction_type.map_or(true, |tt| c.transaction_type == tt)
            })
            .cloned()
            .collect()
    }

    /// 获取分类的深度
    pub fn get_category_depth(
        &self,
        category_id: Uuid,
        categories: &[TransactionCategory],
    ) -> Result<usize> {
        let mut depth = 0;
        let mut current_id = Some(category_id);

        while let Some(id) = current_id {
            if let Some(category) = categories.iter().find(|c| c.id == id) {
                if category.parent_id.is_some() {
                    depth += 1;
                }
                current_id = category.parent_id;
            } else {
                return Err(AppError::Validation("分类不存在".to_string()));
            }
        }

        Ok(depth)
    }
}

impl Default for CategoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 分类树节点
#[derive(Debug, Clone)]
pub struct CategoryNode {
    pub category: TransactionCategory,
    pub children: Vec<CategoryNode>,
}

/// 分类使用统计
#[derive(Debug)]
pub struct CategoryUsageStats {
    pub category_id: Uuid,
    pub transaction_count: usize,
    pub total_amount: f64,
    pub latest_transaction_date: Option<chrono::NaiveDate>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_category_manager_creation() {
        let manager = CategoryManager::new();
        assert_eq!(manager.get_default_currency(), "CNY");
    }

    #[test]
    fn test_create_category() {
        let manager = CategoryManager::new();

        let category = manager
            .create_transaction_category(
                "餐饮".to_string(),
                TransactionType::Expense,
                "#FF5722".to_string(),
                Some("restaurant".to_string()),
                None,
                Some("餐饮支出".to_string()),
            )
            .unwrap();

        assert_eq!(category.name, "餐饮");
        assert_eq!(category.transaction_type, TransactionType::Expense);
        assert_eq!(category.color, "#FF5722");
        assert_eq!(category.icon, Some("restaurant".to_string()));
    }

    #[test]
    fn test_invalid_category_name() {
        let manager = CategoryManager::new();

        let result = manager.create_transaction_category(
            "".to_string(),
            TransactionType::Expense,
            "#FF5722".to_string(),
            None,
            None,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_category_hierarchy_validation() {
        let manager = CategoryManager::new();
        let category_id = Uuid::new_v4();
        let parent_id = Uuid::new_v4();

        // 测试自己作为父分类
        let result = manager.validate_category_hierarchy(category_id, category_id, &[]);
        assert!(result.is_err());

        // 测试正常情况
        let categories = vec![TransactionCategory::new(
            "父分类".to_string(),
            TransactionType::Expense,
            "#FF0000".to_string(),
        )];

        let result = manager.validate_category_hierarchy(category_id, parent_id, &categories);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_root_categories() {
        let manager = CategoryManager::new();

        let mut root_category = TransactionCategory::new(
            "根分类".to_string(),
            TransactionType::Expense,
            "#FF0000".to_string(),
        );
        root_category.parent_id = None;

        let mut sub_category = TransactionCategory::new(
            "子分类".to_string(),
            TransactionType::Expense,
            "#00FF00".to_string(),
        );
        sub_category.parent_id = Some(root_category.id);

        let categories = vec![root_category.clone(), sub_category];
        let roots = manager.get_root_categories(&categories, Some(TransactionType::Expense));

        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].id, root_category.id);
    }

    #[test]
    fn test_category_path() {
        let manager = CategoryManager::new();

        let mut root = TransactionCategory::new(
            "生活".to_string(),
            TransactionType::Expense,
            "#FF0000".to_string(),
        );
        root.parent_id = None;

        let mut sub = TransactionCategory::new(
            "餐饮".to_string(),
            TransactionType::Expense,
            "#00FF00".to_string(),
        );
        sub.parent_id = Some(root.id);

        let categories = vec![root, sub.clone()];
        let path = manager.get_category_path(sub.id, &categories).unwrap();

        assert_eq!(path, vec!["生活".to_string(), "餐饮".to_string()]);
    }

    #[test]
    fn test_category_search() {
        let manager = CategoryManager::new();

        let categories = vec![
            TransactionCategory::new(
                "餐饮美食".to_string(),
                TransactionType::Expense,
                "#FF0000".to_string(),
            ),
            TransactionCategory::new(
                "交通出行".to_string(),
                TransactionType::Expense,
                "#00FF00".to_string(),
            ),
            TransactionCategory::new(
                "工资收入".to_string(),
                TransactionType::Income,
                "#0000FF".to_string(),
            ),
        ];

        let results = manager.search_categories("餐", &categories, Some(TransactionType::Expense));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "餐饮美食");

        let results = manager.search_categories("收入", &categories, Some(TransactionType::Income));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "工资收入");
    }
}
