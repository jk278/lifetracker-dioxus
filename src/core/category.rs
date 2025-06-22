//! # 分类管理模块
//!
//! 提供任务分类的创建、管理和组织功能

use crate::errors::{AppError, Result};
use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// 分类颜色预设
///
/// 提供常用的分类颜色选择
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum CategoryColor {
    /// 红色 - 紧急重要
    Red,
    /// 橙色 - 工作相关
    Orange,
    /// 黄色 - 学习成长
    Yellow,
    /// 绿色 - 健康生活
    Green,
    /// 蓝色 - 个人项目
    #[default]
    Blue,
    /// 紫色 - 创意娱乐
    Purple,
    /// 粉色 - 个人生活
    Pink,
    /// 青色 - 沟通交流
    Cyan,
    /// 灰色 - 其他杂项
    Gray,
    /// 自定义颜色 (十六进制)
    Custom(String),
}

impl fmt::Display for CategoryColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl CategoryColor {
    /// 获取颜色的十六进制值
    pub fn to_hex(&self) -> String {
        match self {
            CategoryColor::Red => "#F44336".to_string(),
            CategoryColor::Orange => "#FF9800".to_string(),
            CategoryColor::Yellow => "#FFEB3B".to_string(),
            CategoryColor::Green => "#4CAF50".to_string(),
            CategoryColor::Blue => "#2196F3".to_string(),
            CategoryColor::Purple => "#9C27B0".to_string(),
            CategoryColor::Pink => "#E91E63".to_string(),
            CategoryColor::Cyan => "#00BCD4".to_string(),
            CategoryColor::Gray => "#9E9E9E".to_string(),
            CategoryColor::Custom(hex) => hex.clone(),
        }
    }

    /// 从十六进制字符串创建颜色
    pub fn from_hex(hex: &str) -> Self {
        match hex {
            "#F44336" => CategoryColor::Red,
            "#FF9800" => CategoryColor::Orange,
            "#FFEB3B" => CategoryColor::Yellow,
            "#4CAF50" => CategoryColor::Green,
            "#2196F3" => CategoryColor::Blue,
            "#9C27B0" => CategoryColor::Purple,
            "#E91E63" => CategoryColor::Pink,
            "#00BCD4" => CategoryColor::Cyan,
            "#9E9E9E" => CategoryColor::Gray,
            _ => CategoryColor::Custom(hex.to_string()),
        }
    }

    /// 检查颜色字符串是否有效
    pub fn is_valid_hex(hex: &str) -> bool {
        hex.starts_with('#') && hex.len() == 7
    }

    /// 获取颜色字符串的长度
    pub fn hex_len(&self) -> usize {
        self.to_hex().len()
    }
}


/// 分类图标枚举
///
/// 提供常用的分类图标选择
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
#[derive(Default)]
pub enum CategoryIcon {
    /// 工作
    Work,
    /// 学习
    Study,
    /// 个人
    Personal,
    /// 运动
    Exercise,
    /// 娱乐
    Entertainment,
    /// 家务
    Household,
    /// 社交
    Social,
    /// 购物
    Shopping,
    /// 旅行
    Travel,
    /// 健康
    Health,
    /// 创意
    Creative,
    /// 食物
    Food,
    /// 会议
    Meeting,
    /// 项目
    Project,
    /// 研究
    Research,
    /// 写作
    Writing,
    /// 设计
    Design,
    /// 开发
    Development,
    /// 其他
    #[default]
    Other,
}

impl CategoryIcon {
    /// 获取图标的Unicode字符
    pub fn to_emoji(&self) -> &'static str {
        match self {
            CategoryIcon::Work => "💼",
            CategoryIcon::Study => "📚",
            CategoryIcon::Personal => "👤",
            CategoryIcon::Exercise => "🏃",
            CategoryIcon::Entertainment => "🎮",
            CategoryIcon::Household => "🏠",
            CategoryIcon::Social => "👥",
            CategoryIcon::Shopping => "🛒",
            CategoryIcon::Travel => "✈️",
            CategoryIcon::Health => "🏥",
            CategoryIcon::Creative => "🎨",
            CategoryIcon::Food => "🍽️",
            CategoryIcon::Meeting => "👥",
            CategoryIcon::Project => "📋",
            CategoryIcon::Research => "🔬",
            CategoryIcon::Writing => "✍️",
            CategoryIcon::Design => "🎨",
            CategoryIcon::Development => "💻",
            CategoryIcon::Other => "📁",
        }
    }
}


/// 分类结构体
///
/// 表示一个任务分类的完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// 分类唯一标识符
    pub id: Uuid,
    /// 分类名称
    pub name: String,
    /// 分类描述
    pub description: Option<String>,
    /// 分类颜色
    pub color: CategoryColor,
    /// 分类图标
    pub icon: CategoryIcon,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 更新时间
    pub updated_at: DateTime<Local>,
    /// 每日目标时长
    pub daily_target: Option<Duration>,
    /// 每周目标时长
    pub weekly_target: Option<Duration>,
    /// 目标时长（用于计算进度）
    pub target_duration: Option<Duration>,
    /// 是否启用
    pub is_active: bool,
    /// 排序顺序
    pub sort_order: i32,
    /// 父分类ID（用于层级分类）
    pub parent_id: Option<Uuid>,
}

impl Category {
    /// 创建新分类
    ///
    /// # 参数
    /// * `name` - 分类名称
    /// * `description` - 分类描述（可选）
    /// * `color` - 分类颜色（可选，默认蓝色）
    /// * `icon` - 分类图标（可选，默认其他）
    ///
    /// # 示例
    /// ```
    /// use time_tracker::core::{Category, CategoryColor, CategoryIcon};
    ///
    /// let category = Category::new(
    ///     "工作".to_string(),
    ///     Some("日常工作任务".to_string()),
    ///     Some(CategoryColor::Orange),
    ///     Some(CategoryIcon::Work)
    /// );
    /// ```
    pub fn new(
        name: String,
        description: Option<String>,
        color: Option<CategoryColor>,
        icon: Option<CategoryIcon>,
    ) -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            color: color.unwrap_or_default(),
            icon: icon.unwrap_or_default(),
            created_at: now,
            updated_at: now,
            daily_target: None,
            weekly_target: None,
            target_duration: None,
            is_active: true,
            sort_order: 0,
            parent_id: None,
        }
    }

    /// 更新分类信息
    pub fn update(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        color: Option<CategoryColor>,
        icon: Option<CategoryIcon>,
    ) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(description) = description {
            self.description = Some(description);
        }
        if let Some(color) = color {
            self.color = color;
        }
        if let Some(icon) = icon {
            self.icon = icon;
        }
        self.updated_at = Local::now();
        log::debug!("分类更新: {}", self.name);
    }

    /// 设置每日目标
    pub fn set_daily_target(&mut self, target: Duration) {
        self.daily_target = Some(target);
        self.updated_at = Local::now();
    }

    /// 设置每周目标
    pub fn set_weekly_target(&mut self, target: Duration) {
        self.weekly_target = Some(target);
        self.updated_at = Local::now();
    }

    /// 设置父分类
    pub fn set_parent(&mut self, parent_id: Option<Uuid>) {
        self.parent_id = parent_id;
        self.updated_at = Local::now();
    }

    /// 启用/禁用分类
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
        self.updated_at = Local::now();
    }

    /// 设置排序顺序
    pub fn set_sort_order(&mut self, order: i32) {
        self.sort_order = order;
        self.updated_at = Local::now();
    }

    /// 检查是否为根分类
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// 检查是否为子分类
    pub fn is_child(&self) -> bool {
        self.parent_id.is_some()
    }
}

/// 分类管理器
///
/// 负责管理所有分类的生命周期和层级关系
#[derive(Debug, Clone)]
pub struct CategoryManager {
    /// 分类存储
    categories: HashMap<Uuid, Category>,
    /// 默认分类ID
    default_category_id: Option<Uuid>,
}

impl CategoryManager {
    /// 创建新的分类管理器
    pub fn new() -> Self {
        let mut manager = Self {
            categories: HashMap::new(),
            default_category_id: None,
        };

        // 创建默认分类
        manager.create_default_categories();
        manager
    }

    /// 创建默认分类
    fn create_default_categories(&mut self) {
        let default_categories = vec![
            (
                "工作",
                "工作相关任务",
                CategoryColor::Orange,
                CategoryIcon::Work,
            ),
            (
                "学习",
                "学习和成长",
                CategoryColor::Yellow,
                CategoryIcon::Study,
            ),
            (
                "运动",
                "健身和运动",
                CategoryColor::Green,
                CategoryIcon::Exercise,
            ),
            (
                "娱乐",
                "休闲娱乐",
                CategoryColor::Purple,
                CategoryIcon::Entertainment,
            ),
            (
                "其他",
                "其他杂项任务",
                CategoryColor::Gray,
                CategoryIcon::Other,
            ),
        ];

        for (i, (name, desc, color, icon)) in default_categories.into_iter().enumerate() {
            let mut category = Category::new(
                name.to_string(),
                Some(desc.to_string()),
                Some(color),
                Some(icon),
            );
            category.sort_order = i as i32;

            // 设置第一个为默认分类
            if i == 0 {
                self.default_category_id = Some(category.id);
            }

            self.categories.insert(category.id, category);
        }

        log::debug!("创建了 {} 个默认分类", self.categories.len());
    }

    /// 添加分类
    pub fn add_category(&mut self, category: Category) -> Result<Uuid> {
        let category_id = category.id;
        self.categories.insert(category_id, category);
        log::debug!("添加分类: {}", category_id);
        Ok(category_id)
    }

    /// 创建新分类
    pub fn create_category(
        &mut self,
        name: String,
        description: Option<String>,
        color: Option<CategoryColor>,
        icon: Option<CategoryIcon>,
    ) -> Result<Uuid> {
        // 检查名称是否已存在
        if self.categories.values().any(|c| c.name == name) {
            return Err(AppError::CategoryNotFound(format!(
                "分类名称 '{}' 已存在",
                name
            )));
        }

        let category = Category::new(name, description, color, icon);
        self.add_category(category)
    }

    /// 获取分类
    pub fn get_category(&self, category_id: Uuid) -> Option<&Category> {
        self.categories.get(&category_id)
    }

    /// 获取可变分类引用
    pub fn get_category_mut(&mut self, category_id: Uuid) -> Option<&mut Category> {
        self.categories.get_mut(&category_id)
    }

    /// 删除分类
    pub fn remove_category(&mut self, category_id: Uuid) -> Result<Category> {
        // 不能删除默认分类
        if Some(category_id) == self.default_category_id {
            return Err(AppError::CategoryNotFound("不能删除默认分类".to_string()));
        }

        self.categories
            .remove(&category_id)
            .ok_or_else(|| AppError::CategoryNotFound(category_id.to_string()))
    }

    /// 更新分类
    pub fn update_category(
        &mut self,
        category_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        color: Option<CategoryColor>,
        icon: Option<CategoryIcon>,
    ) -> Result<()> {
        let category = self
            .get_category_mut(category_id)
            .ok_or_else(|| AppError::CategoryNotFound(category_id.to_string()))?;

        category.update(name, description, color, icon);
        Ok(())
    }

    /// 获取所有分类
    pub fn get_all_categories(&self) -> Vec<&Category> {
        let mut categories: Vec<&Category> = self.categories.values().collect();
        categories.sort_by_key(|c| c.sort_order);
        categories
    }

    /// 获取活动分类
    pub fn get_active_categories(&self) -> Vec<&Category> {
        let mut categories: Vec<&Category> =
            self.categories.values().filter(|c| c.is_active).collect();
        categories.sort_by_key(|c| c.sort_order);
        categories
    }

    /// 获取根分类
    pub fn get_root_categories(&self) -> Vec<&Category> {
        let mut categories: Vec<&Category> = self
            .categories
            .values()
            .filter(|c| c.is_root() && c.is_active)
            .collect();
        categories.sort_by_key(|c| c.sort_order);
        categories
    }

    /// 获取子分类
    pub fn get_child_categories(&self, parent_id: Uuid) -> Vec<&Category> {
        let mut categories: Vec<&Category> = self
            .categories
            .values()
            .filter(|c| c.parent_id == Some(parent_id) && c.is_active)
            .collect();
        categories.sort_by_key(|c| c.sort_order);
        categories
    }

    /// 搜索分类
    pub fn search_categories(&self, query: &str) -> Vec<&Category> {
        let query_lower = query.to_lowercase();
        self.categories
            .values()
            .filter(|category| {
                category.name.to_lowercase().contains(&query_lower)
                    || category
                        .description
                        .as_ref().is_some_and(|desc| desc.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// 获取默认分类
    pub fn get_default_category(&self) -> Option<&Category> {
        self.default_category_id
            .and_then(|id| self.categories.get(&id))
    }

    /// 设置默认分类
    pub fn set_default_category(&mut self, category_id: Uuid) -> Result<()> {
        if !self.categories.contains_key(&category_id) {
            return Err(AppError::CategoryNotFound(category_id.to_string()));
        }

        self.default_category_id = Some(category_id);
        Ok(())
    }

    /// 获取分类总数
    pub fn get_category_count(&self) -> usize {
        self.categories.len()
    }

    /// 重新排序分类
    pub fn reorder_categories(&mut self, category_orders: Vec<(Uuid, i32)>) -> Result<()> {
        for (category_id, order) in category_orders {
            if let Some(category) = self.get_category_mut(category_id) {
                category.set_sort_order(order);
            }
        }
        Ok(())
    }
}

impl Default for CategoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_creation() {
        let category = Category::new(
            "测试分类".to_string(),
            Some("这是一个测试分类".to_string()),
            Some(CategoryColor::Red),
            Some(CategoryIcon::Work),
        );

        assert_eq!(category.name, "测试分类");
        assert_eq!(category.color.to_hex(), "#F44336");
        assert_eq!(category.icon.to_emoji(), "💼");
        assert!(category.is_active);
        assert!(category.is_root());
    }

    #[test]
    fn test_category_manager() {
        let mut manager = CategoryManager::new();

        // 检查默认分类
        assert!(manager.get_category_count() > 0);
        assert!(manager.get_default_category().is_some());

        // 创建新分类
        let category_id = manager
            .create_category(
                "新分类".to_string(),
                None,
                Some(CategoryColor::Green),
                Some(CategoryIcon::Study),
            )
            .unwrap();

        // 获取分类
        let category = manager.get_category(category_id).unwrap();
        assert_eq!(category.name, "新分类");

        // 更新分类
        manager
            .update_category(
                category_id,
                Some("更新的分类".to_string()),
                None,
                None,
                None,
            )
            .unwrap();

        let updated_category = manager.get_category(category_id).unwrap();
        assert_eq!(updated_category.name, "更新的分类");
    }

    #[test]
    fn test_category_colors() {
        assert_eq!(CategoryColor::Red.to_hex(), "#F44336");
        assert_eq!(CategoryColor::from_hex("#F44336"), CategoryColor::Red);

        let custom = CategoryColor::Custom("#123456".to_string());
        assert_eq!(custom.to_hex(), "#123456");
    }

    #[test]
    fn test_category_hierarchy() {
        let mut manager = CategoryManager::new();

        // 创建父分类
        let parent_id = manager
            .create_category("父分类".to_string(), None, None, None)
            .unwrap();

        // 创建子分类
        let mut child_category = Category::new("子分类".to_string(), None, None, None);
        child_category.set_parent(Some(parent_id));
        let child_id = manager.add_category(child_category).unwrap();

        // 验证层级关系
        let parent = manager.get_category(parent_id).unwrap();
        let child = manager.get_category(child_id).unwrap();

        assert!(parent.is_root());
        assert!(child.is_child());
        assert_eq!(child.parent_id, Some(parent_id));

        // 获取子分类
        let children = manager.get_child_categories(parent_id);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, child_id);
    }
}
