//! # 数据模型定义
//!
//! 定义数据库表对应的Rust结构体和相关类型

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ==================== 时间记录模型 ====================

/// 时间记录完整模型
///
/// 对应数据库中的 time_entries 表
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeEntry {
    /// 唯一标识符
    pub id: Uuid,
    /// 任务名称
    pub task_name: String,
    /// 分类ID（可选）
    pub category_id: Option<Uuid>,
    /// 开始时间
    pub start_time: DateTime<Local>,
    /// 结束时间（可选，正在进行的任务为None）
    pub end_time: Option<DateTime<Local>>,
    /// 持续时间（秒）
    pub duration_seconds: i64,
    /// 任务描述（可选）
    pub description: Option<String>,
    /// 标签列表
    pub tags: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 更新时间（可选）
    pub updated_at: Option<DateTime<Local>>,
}

/// 时间记录插入模型
///
/// 用于插入新的时间记录到数据库
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntryInsert {
    /// 唯一标识符
    pub id: Uuid,
    /// 任务名称
    pub task_name: String,
    /// 分类ID（可选）
    pub category_id: Option<Uuid>,
    /// 开始时间
    pub start_time: DateTime<Local>,
    /// 结束时间（可选）
    pub end_time: Option<DateTime<Local>>,
    /// 持续时间（秒）
    pub duration_seconds: i64,
    /// 任务描述（可选）
    pub description: Option<String>,
    /// 标签列表
    pub tags: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Local>,
}

/// 时间记录更新模型
///
/// 用于更新现有的时间记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntryUpdate {
    /// 任务名称（可选）
    pub task_name: Option<String>,
    /// 分类ID（可选）
    pub category_id: Option<Option<Uuid>>,
    /// 开始时间（可选）
    pub start_time: Option<DateTime<Local>>,
    /// 结束时间（可选）
    pub end_time: Option<Option<DateTime<Local>>>,
    /// 持续时间（秒）（可选）
    pub duration_seconds: Option<i64>,
    /// 任务描述（可选）
    pub description: Option<Option<String>>,
    /// 标签列表（可选）
    pub tags: Option<Vec<String>>,
}

// ==================== 分类模型 ====================

/// 分类完整模型
///
/// 对应数据库中的 categories 表
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryModel {
    /// 唯一标识符
    pub id: Uuid,
    /// 分类名称
    pub name: String,
    /// 分类描述（可选）
    pub description: Option<String>,
    /// 分类颜色（十六进制颜色代码）
    pub color: String,
    /// 分类图标
    pub icon: String,
    /// 每日目标时长（秒）（可选）
    pub daily_target_seconds: Option<i64>,
    /// 每周目标时长（秒）（可选）
    pub weekly_target_seconds: Option<i64>,
    /// 是否激活
    pub is_active: bool,
    /// 排序顺序
    pub sort_order: i32,
    /// 父分类ID（用于层级分类）（可选）
    pub parent_id: Option<Uuid>,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 更新时间（可选）
    pub updated_at: Option<DateTime<Local>>,
}

/// 分类插入模型
///
/// 用于插入新的分类到数据库
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInsert {
    /// 唯一标识符
    pub id: Uuid,
    /// 分类名称
    pub name: String,
    /// 分类描述（可选）
    pub description: Option<String>,
    /// 分类颜色（十六进制颜色代码）
    pub color: String,
    /// 分类图标
    pub icon: String,
    /// 每日目标时长（秒）（可选）
    pub daily_target_seconds: Option<i64>,
    /// 每周目标时长（秒）（可选）
    pub weekly_target_seconds: Option<i64>,
    /// 是否激活
    pub is_active: bool,
    /// 排序顺序
    pub sort_order: i32,
    /// 父分类ID（用于层级分类）（可选）
    pub parent_id: Option<Uuid>,
    /// 创建时间
    pub created_at: DateTime<Local>,
}

/// 分类更新模型
///
/// 用于更新现有的分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryUpdate {
    /// 分类名称（可选）
    pub name: Option<String>,
    /// 分类描述（可选）
    pub description: Option<Option<String>>,
    /// 分类颜色（可选）
    pub color: Option<String>,
    /// 分类图标（可选）
    pub icon: Option<String>,
    /// 每日目标时长（秒）（可选）
    pub daily_target_seconds: Option<Option<i64>>,
    /// 每周目标时长（秒）（可选）
    pub weekly_target_seconds: Option<Option<i64>>,
    /// 是否激活（可选）
    pub is_active: Option<bool>,
    /// 排序顺序（可选）
    pub sort_order: Option<i32>,
    /// 父分类ID（可选）
    pub parent_id: Option<Option<Uuid>>,
}

// ==================== 统计模型 ====================

/// 时间统计模型
///
/// 用于表示各种时间统计数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeStats {
    /// 总时长（秒）
    pub total_seconds: i64,
    /// 任务数量
    pub task_count: i64,
    /// 平均时长（秒）
    pub average_seconds: f64,
    /// 最长时长（秒）
    pub max_seconds: i64,
    /// 最短时长（秒）
    pub min_seconds: i64,
    /// 统计日期范围开始
    pub start_date: DateTime<Local>,
    /// 统计日期范围结束
    pub end_date: DateTime<Local>,
}

/// 数据库时间统计模型
///
/// 用于数据库查询结果的时间统计
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseTimeStats {
    /// 总时长（秒）
    pub total_seconds: i64,
    /// 任务数量
    pub task_count: i64,
    /// 平均时长（秒）
    pub average_seconds: f64,
    /// 最长时长（秒）
    pub max_seconds: i64,
    /// 最短时长（秒）
    pub min_seconds: i64,
    /// 元数据信息
    pub metadata: StatisticsMetadata,
    /// 额外统计信息
    pub statistics: AdditionalStatistics,
}

/// 分类统计信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryStats {
    pub category_id: Uuid,
    pub category_name: String,
    pub task_count: usize,
    pub total_seconds: i64,
    pub average_seconds: f64,
    pub last_used: Option<DateTime<Local>>,
}

/// 导出元数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportMetadata {
    pub generated_at: DateTime<Local>,
    pub source: String,
    pub version: String,
}

/// 导出统计信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportStatistics {
    pub median_seconds: i64,
    pub standard_deviation: f64,
    pub completion_rate: f64,
}

/// 统计元数据（别名）
pub type StatisticsMetadata = ExportMetadata;

/// 附加统计信息（别名）
pub type AdditionalStatistics = ExportStatistics;

/// 每日统计模型
///
/// 用于表示每日的时间统计
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DailyStats {
    /// 日期
    pub date: chrono::NaiveDate,
    /// 该日期的时间统计
    pub stats: DatabaseTimeStats,
    /// 按分类的统计
    pub category_stats: Vec<CategoryStats>,
}

/// 每周统计模型
///
/// 用于表示每周的时间统计
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeeklyStats {
    /// 周开始日期
    pub week_start: chrono::NaiveDate,
    /// 周结束日期
    pub week_end: chrono::NaiveDate,
    /// 该周的时间统计
    pub stats: DatabaseTimeStats,
    /// 每日统计
    pub daily_stats: Vec<DailyStats>,
    /// 按分类的统计
    pub category_stats: Vec<CategoryStats>,
}

/// 每月统计模型
///
/// 用于表示每月的时间统计
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonthlyStats {
    /// 年份
    pub year: i32,
    /// 月份
    pub month: u32,
    /// 该月的时间统计
    pub stats: DatabaseTimeStats,
    /// 每日统计
    pub daily_stats: Vec<DailyStats>,
    /// 每周统计
    pub weekly_stats: Vec<WeeklyStats>,
    /// 按分类的统计
    pub category_stats: Vec<CategoryStats>,
}

// ==================== 查询参数模型 ====================

/// 时间记录查询参数
///
/// 用于构建复杂的查询条件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeEntryQuery {
    /// 分类ID过滤
    pub category_id: Option<Uuid>,
    /// 开始时间范围
    pub start_time_from: Option<DateTime<Local>>,
    /// 结束时间范围
    pub start_time_to: Option<DateTime<Local>>,
    /// 任务名称搜索（模糊匹配）
    pub task_name_search: Option<String>,
    /// 标签过滤
    pub tags: Option<Vec<String>>,
    /// 最小持续时间（秒）
    pub min_duration_seconds: Option<i64>,
    /// 最大持续时间（秒）
    pub max_duration_seconds: Option<i64>,
    /// 排序字段
    pub sort_by: Option<TimeEntrySortBy>,
    /// 排序方向
    pub sort_order: Option<SortOrder>,
    /// 分页：偏移量
    pub offset: Option<i64>,
    /// 分页：限制数量
    pub limit: Option<i64>,
}

/// 时间记录排序字段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeEntrySortBy {
    /// 按开始时间排序
    StartTime,
    /// 按持续时间排序
    Duration,
    /// 按任务名称排序
    TaskName,
    /// 按创建时间排序
    CreatedAt,
    /// 按更新时间排序
    UpdatedAt,
}

/// 排序方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortOrder {
    /// 升序
    Asc,
    /// 降序
    Desc,
}

/// 分类查询参数
///
/// 用于构建分类查询条件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryQuery {
    /// 父分类ID过滤
    pub parent_id: Option<Option<Uuid>>,
    /// 是否激活过滤
    pub is_active: Option<bool>,
    /// 名称搜索（模糊匹配）
    pub name_search: Option<String>,
    /// 排序字段
    pub sort_by: Option<CategorySortBy>,
    /// 排序方向
    pub sort_order: Option<SortOrder>,
}

/// 分类排序字段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CategorySortBy {
    /// 按排序顺序排序
    SortOrder,
    /// 按名称排序
    Name,
    /// 按创建时间排序
    CreatedAt,
    /// 按更新时间排序
    UpdatedAt,
}

// ==================== 类型转换实现 ====================

/// 从核心Category转换为数据库CategoryInsert
impl From<crate::core::Category> for CategoryInsert {
    fn from(category: crate::core::Category) -> Self {
        Self {
            id: category.id,
            name: category.name,
            description: category.description,
            color: category.color.to_hex(),
            icon: category.icon.to_emoji().to_string(),
            daily_target_seconds: category.daily_target.map(|d| d.num_seconds()),
            weekly_target_seconds: category.weekly_target.map(|d| d.num_seconds()),
            is_active: category.is_active,
            sort_order: category.sort_order,
            parent_id: category.parent_id,
            created_at: category.created_at,
        }
    }
}

/// 从数据库CategoryModel转换为核心Category
impl From<CategoryModel> for crate::core::Category {
    fn from(model: CategoryModel) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            color: crate::core::CategoryColor::from_hex(&model.color),
            icon: crate::core::CategoryIcon::Other, // 简化处理，可后续完善
            created_at: model.created_at,
            updated_at: model.updated_at.unwrap_or(model.created_at),
            daily_target: model.daily_target_seconds.map(chrono::Duration::seconds),
            weekly_target: model.weekly_target_seconds.map(chrono::Duration::seconds),
            target_duration: model.daily_target_seconds.map(chrono::Duration::seconds),
            is_active: model.is_active,
            sort_order: model.sort_order,
            parent_id: model.parent_id,
        }
    }
}

/// 从核心Category转换为数据库CategoryModel
impl From<crate::core::Category> for CategoryModel {
    fn from(category: crate::core::Category) -> Self {
        Self {
            id: category.id,
            name: category.name,
            description: category.description,
            color: category.color.to_hex(),
            icon: category.icon.to_emoji().to_string(),
            daily_target_seconds: category.daily_target.map(|d| d.num_seconds()),
            weekly_target_seconds: category.weekly_target.map(|d| d.num_seconds()),
            is_active: category.is_active,
            sort_order: category.sort_order,
            parent_id: category.parent_id,
            created_at: category.created_at,
            updated_at: Some(category.updated_at),
        }
    }
}

/// 从TimeEntry转换为TimeEntryInsert  
impl From<TimeEntry> for TimeEntryInsert {
    fn from(entry: TimeEntry) -> Self {
        Self {
            id: entry.id,
            task_name: entry.task_name,
            category_id: entry.category_id,
            start_time: entry.start_time,
            end_time: entry.end_time,
            duration_seconds: entry.duration_seconds,
            description: entry.description,
            tags: entry.tags,
            created_at: entry.created_at,
        }
    }
}

// ==================== 实现方法 ====================

impl TimeEntry {
    /// 创建新的时间记录
    pub fn new(task_name: String, category_id: Option<Uuid>, start_time: DateTime<Local>) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_name,
            category_id,
            start_time,
            end_time: None,
            duration_seconds: 0,
            description: None,
            tags: Vec::new(),
            created_at: Local::now(),
            updated_at: None,
        }
    }

    /// 结束时间记录
    pub fn finish(&mut self, end_time: DateTime<Local>) {
        self.end_time = Some(end_time);
        self.duration_seconds = (end_time - self.start_time).num_seconds();
        self.updated_at = Some(Local::now());
    }

    /// 检查是否正在进行中
    pub fn is_running(&self) -> bool {
        self.end_time.is_none()
    }

    /// 获取格式化的持续时间
    pub fn formatted_duration(&self) -> String {
        let hours = self.duration_seconds / 3600;
        let minutes = (self.duration_seconds % 3600) / 60;
        let seconds = self.duration_seconds % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Some(Local::now());
        }
    }

    /// 移除标签
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Some(Local::now());
        }
    }
}

impl CategoryModel {
    /// 创建新的分类
    pub fn new(name: String, color: String, icon: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            color,
            icon,
            daily_target_seconds: None,
            weekly_target_seconds: None,
            is_active: true,
            sort_order: 0,
            parent_id: None,
            created_at: Local::now(),
            updated_at: None,
        }
    }

    /// 检查是否为根分类
    pub fn is_root_category(&self) -> bool {
        self.parent_id.is_none()
    }

    /// 设置每日目标
    pub fn set_daily_target(&mut self, seconds: Option<i64>) {
        self.daily_target_seconds = seconds;
        self.updated_at = Some(Local::now());
    }

    /// 设置每周目标
    pub fn set_weekly_target(&mut self, seconds: Option<i64>) {
        self.weekly_target_seconds = seconds;
        self.updated_at = Some(Local::now());
    }

    /// 激活/停用分类
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
        self.updated_at = Some(Local::now());
    }
}

impl DatabaseTimeStats {
    /// 创建空的统计数据
    pub fn empty() -> Self {
        Self {
            total_seconds: 0,
            task_count: 0,
            average_seconds: 0.0,
            max_seconds: 0,
            min_seconds: 0,
            metadata: StatisticsMetadata {
                generated_at: chrono::Local::now(),
                source: "time-tracker".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            statistics: AdditionalStatistics {
                median_seconds: 0,
                standard_deviation: 0.0,
                completion_rate: 0.0,
            },
        }
    }

    /// 获取格式化的总时长
    pub fn formatted_total_duration(&self) -> String {
        let hours = self.total_seconds / 3600;
        let minutes = (self.total_seconds % 3600) / 60;

        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }

    /// 获取格式化的平均时长
    pub fn formatted_average_duration(&self) -> String {
        let avg_seconds = self.average_seconds as i64;
        let hours = avg_seconds / 3600;
        let minutes = (avg_seconds % 3600) / 60;

        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }
}

// ==================== 默认实现 ====================

impl Default for TimeEntrySortBy {
    fn default() -> Self {
        Self::StartTime
    }
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Desc
    }
}

impl Default for CategorySortBy {
    fn default() -> Self {
        Self::SortOrder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_time_entry_creation() {
        let entry = TimeEntry::new("测试任务".to_string(), None, Local::now());

        assert_eq!(entry.task_name, "测试任务");
        assert!(entry.is_running());
        assert_eq!(entry.duration_seconds, 0);
    }

    #[test]
    fn test_time_entry_finish() {
        let mut entry = TimeEntry::new("测试任务".to_string(), None, Local::now());

        let end_time = entry.start_time + Duration::hours(1);
        entry.finish(end_time);

        assert!(!entry.is_running());
        assert_eq!(entry.duration_seconds, 3600);
        assert!(entry.updated_at.is_some());
    }

    #[test]
    fn test_time_entry_tags() {
        let mut entry = TimeEntry::new("测试任务".to_string(), None, Local::now());

        entry.add_tag("工作".to_string());
        entry.add_tag("重要".to_string());
        assert_eq!(entry.tags.len(), 2);

        // 重复添加不会增加
        entry.add_tag("工作".to_string());
        assert_eq!(entry.tags.len(), 2);

        entry.remove_tag("工作");
        assert_eq!(entry.tags.len(), 1);
        assert_eq!(entry.tags[0], "重要");
    }

    #[test]
    fn test_category_creation() {
        let category = CategoryModel::new(
            "工作".to_string(),
            "#FF5722".to_string(),
            "work".to_string(),
        );

        assert_eq!(category.name, "工作");
        assert_eq!(category.color, "#FF5722");
        assert_eq!(category.icon, "work");
        assert!(category.is_active);
        assert!(category.is_root_category());
    }

    #[test]
    fn test_category_targets() {
        let mut category = CategoryModel::new(
            "工作".to_string(),
            "#FF5722".to_string(),
            "work".to_string(),
        );

        category.set_daily_target(Some(8 * 3600)); // 8小时
        category.set_weekly_target(Some(40 * 3600)); // 40小时

        assert_eq!(category.daily_target_seconds, Some(8 * 3600));
        assert_eq!(category.weekly_target_seconds, Some(40 * 3600));
        assert!(category.updated_at.is_some());
    }

    #[test]
    fn test_time_stats_formatting() {
        let stats = DatabaseTimeStats {
            total_seconds: 7320, // 2小时2分钟
            task_count: 3,
            average_seconds: 2440.0, // 40分钟40秒
            max_seconds: 3600,
            min_seconds: 1800,
            metadata: StatisticsMetadata {
                generated_at: Local::now(),
                source: "test".to_string(),
                version: "1.0.0".to_string(),
            },
            statistics: AdditionalStatistics {
                median_seconds: 2440,
                standard_deviation: 100.0,
                completion_rate: 0.85,
            },
        };

        assert_eq!(stats.formatted_total_duration(), "2h 2m");
        assert_eq!(stats.formatted_average_duration(), "40m");
    }

    #[test]
    fn test_formatted_duration() {
        let mut entry = TimeEntry::new("测试任务".to_string(), None, Local::now());

        entry.duration_seconds = 3661; // 1小时1分钟1秒
        assert_eq!(entry.formatted_duration(), "1h 1m 1s");

        entry.duration_seconds = 61; // 1分钟1秒
        assert_eq!(entry.formatted_duration(), "1m 1s");

        entry.duration_seconds = 30; // 30秒
        assert_eq!(entry.formatted_duration(), "30s");
    }
}

/// 表统计信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableStats {
    /// 表名
    pub table_name: String,
    /// 记录数量
    pub record_count: u64,
}

/// 数据库统计信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseStats {
    pub database_size: u64,
    pub page_count: usize,
    pub page_size: usize,
    pub table_stats: Vec<TableStats>,
    pub size_mb: f64,
    pub last_updated: chrono::DateTime<chrono::Local>,
}

// ==================== 笔记模型 ====================

/// 笔记完整模型
///
/// 对应数据库中的 notes 表
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    /// 唯一标识符
    pub id: Uuid,
    /// 笔记标题
    pub title: String,
    /// 笔记内容
    pub content: String,
    /// 心情状态（可选）
    pub mood: Option<String>,
    /// 标签列表
    pub tags: Vec<String>,
    /// 是否收藏
    pub is_favorite: bool,
    /// 是否归档
    pub is_archived: bool,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 更新时间
    pub updated_at: DateTime<Local>,
}

/// 笔记插入模型
///
/// 用于插入新的笔记到数据库
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteInsert {
    /// 唯一标识符
    pub id: Uuid,
    /// 笔记标题
    pub title: String,
    /// 笔记内容
    pub content: String,
    /// 心情状态（可选）
    pub mood: Option<String>,
    /// 标签列表
    pub tags: Vec<String>,
    /// 是否收藏
    pub is_favorite: bool,
    /// 是否归档
    pub is_archived: bool,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 更新时间
    pub updated_at: DateTime<Local>,
}

/// 笔记更新模型
///
/// 用于更新现有的笔记
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteUpdate {
    /// 笔记标题（可选）
    pub title: Option<String>,
    /// 笔记内容（可选）
    pub content: Option<String>,
    /// 心情状态（可选）
    pub mood: Option<Option<String>>,
    /// 标签列表（可选）
    pub tags: Option<Vec<String>>,
    /// 是否收藏（可选）
    pub is_favorite: Option<bool>,
    /// 是否归档（可选）
    pub is_archived: Option<bool>,
    /// 更新时间
    pub updated_at: DateTime<Local>,
}

/// 笔记查询模型
///
/// 用于查询和过滤笔记
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoteQuery {
    /// 标题或内容搜索（模糊匹配）
    pub search: Option<String>,
    /// 标签过滤
    pub tags: Option<Vec<String>>,
    /// 心情过滤
    pub mood: Option<String>,
    /// 是否收藏过滤
    pub is_favorite: Option<bool>,
    /// 是否归档过滤
    pub is_archived: Option<bool>,
    /// 创建时间范围开始
    pub created_from: Option<DateTime<Local>>,
    /// 创建时间范围结束
    pub created_to: Option<DateTime<Local>>,
    /// 排序字段
    pub sort_by: Option<NoteSortBy>,
    /// 排序方向
    pub sort_order: Option<SortOrder>,
    /// 分页：偏移量
    pub offset: Option<i64>,
    /// 分页：限制数量
    pub limit: Option<i64>,
}

/// 笔记排序字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoteSortBy {
    /// 按创建时间排序
    CreatedAt,
    /// 按更新时间排序
    UpdatedAt,
    /// 按标题排序
    Title,
    /// 按心情排序
    Mood,
}

/// 笔记统计模型
///
/// 用于表示笔记相关的统计数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NoteStats {
    /// 总笔记数
    pub total_notes: i64,
    /// 收藏笔记数
    pub favorite_notes: i64,
    /// 归档笔记数
    pub archived_notes: i64,
    /// 本周笔记数
    pub notes_this_week: i64,
    /// 本月笔记数
    pub notes_this_month: i64,
    /// 最常用标签
    pub most_used_tags: Vec<TagStats>,
    /// 心情分布
    pub mood_distribution: Vec<MoodStats>,
    /// 每日笔记趋势
    pub daily_notes_trend: Vec<DailyNoteStats>,
}

/// 标签统计
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagStats {
    /// 标签名称
    pub tag: String,
    /// 使用次数
    pub count: i64,
    /// 使用百分比
    pub percentage: f64,
}

/// 心情统计
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MoodStats {
    /// 心情类型
    pub mood: String,
    /// 次数
    pub count: i64,
    /// 百分比
    pub percentage: f64,
}

/// 每日笔记统计
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DailyNoteStats {
    /// 日期
    pub date: chrono::NaiveDate,
    /// 笔记数量
    pub count: i64,
}

// ==================== 笔记模型实现 ====================

impl Note {
    /// 创建新的笔记
    pub fn new(title: String, content: String) -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4(),
            title,
            content,
            mood: None,
            tags: Vec::new(),
            is_favorite: false,
            is_archived: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// 设置心情
    pub fn set_mood(&mut self, mood: Option<String>) {
        self.mood = mood;
        self.updated_at = Local::now();
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Local::now();
        }
    }

    /// 移除标签
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Local::now();
        }
    }

    /// 设置收藏状态
    pub fn set_favorite(&mut self, is_favorite: bool) {
        self.is_favorite = is_favorite;
        self.updated_at = Local::now();
    }

    /// 设置归档状态
    pub fn set_archived(&mut self, is_archived: bool) {
        self.is_archived = is_archived;
        self.updated_at = Local::now();
    }

    /// 更新内容
    pub fn update_content(&mut self, title: Option<String>, content: Option<String>) {
        if let Some(title) = title {
            self.title = title;
        }
        if let Some(content) = content {
            self.content = content;
        }
        self.updated_at = Local::now();
    }

    /// 获取内容预览（前100个字符）
    pub fn content_preview(&self) -> String {
        if self.content.len() > 100 {
            format!("{}...", &self.content[..100])
        } else {
            self.content.clone()
        }
    }

    /// 检查是否包含标签
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// 获取格式化的创建时间
    pub fn formatted_created_at(&self) -> String {
        self.created_at.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// 获取格式化的更新时间
    pub fn formatted_updated_at(&self) -> String {
        self.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

impl From<Note> for NoteInsert {
    fn from(note: Note) -> Self {
        Self {
            id: note.id,
            title: note.title,
            content: note.content,
            mood: note.mood,
            tags: note.tags,
            is_favorite: note.is_favorite,
            is_archived: note.is_archived,
            created_at: note.created_at,
            updated_at: note.updated_at,
        }
    }
}

impl Default for NoteSortBy {
    fn default() -> Self {
        Self::UpdatedAt
    }
}

impl NoteStats {
    /// 创建空的统计数据
    pub fn empty() -> Self {
        Self {
            total_notes: 0,
            favorite_notes: 0,
            archived_notes: 0,
            notes_this_week: 0,
            notes_this_month: 0,
            most_used_tags: Vec::new(),
            mood_distribution: Vec::new(),
            daily_notes_trend: Vec::new(),
        }
    }

    /// 计算收藏率
    pub fn favorite_rate(&self) -> f64 {
        if self.total_notes == 0 {
            0.0
        } else {
            (self.favorite_notes as f64 / self.total_notes as f64) * 100.0
        }
    }

    /// 计算归档率
    pub fn archive_rate(&self) -> f64 {
        if self.total_notes == 0 {
            0.0
        } else {
            (self.archived_notes as f64 / self.total_notes as f64) * 100.0
        }
    }
}
