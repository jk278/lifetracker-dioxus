//! # 任务数据模型
//!
//! 定义任务相关的数据结构和数据库模型

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 任务数据库模型
///
/// 用于表示数据库中的任务记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskModel {
    /// 唯一标识符
    pub id: Uuid,
    /// 任务名称
    pub name: String,
    /// 任务描述（可选）
    pub description: Option<String>,
    /// 分类ID（可选）
    pub category_id: Option<Uuid>,
    /// 任务状态
    pub status: String,
    /// 优先级
    pub priority: String,
    /// 预估时长（秒）（可选）
    pub estimated_duration_seconds: Option<i64>,
    /// 实际总时长（秒）
    pub total_duration_seconds: i64,
    /// 标签（JSON数组字符串）
    pub tags: String,
    /// 截止日期（可选）
    pub due_date: Option<DateTime<Local>>,
    /// 是否已完成
    pub is_completed: bool,
    /// 完成时间（可选）
    pub completed_at: Option<DateTime<Local>>,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 更新时间（可选）
    pub updated_at: Option<DateTime<Local>>,
}

/// 任务插入模型
///
/// 用于插入新的任务到数据库
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInsert {
    /// 唯一标识符
    pub id: Uuid,
    /// 任务名称
    pub name: String,
    /// 任务描述（可选）
    pub description: Option<String>,
    /// 分类ID（可选）
    pub category_id: Option<Uuid>,
    /// 任务状态
    pub status: String,
    /// 优先级
    pub priority: String,
    /// 预估时长（秒）（可选）
    pub estimated_duration_seconds: Option<i64>,
    /// 实际总时长（秒）
    pub total_duration_seconds: i64,
    /// 标签（JSON数组字符串）
    pub tags: String,
    /// 截止日期（可选）
    pub due_date: Option<DateTime<Local>>,
    /// 是否已完成
    pub is_completed: bool,
    /// 完成时间（可选）
    pub completed_at: Option<DateTime<Local>>,
    /// 创建时间
    pub created_at: DateTime<Local>,
}

/// 任务更新模型
///
/// 用于更新现有的任务
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskUpdate {
    /// 任务名称（可选）
    pub name: Option<String>,
    /// 任务描述（可选）
    pub description: Option<Option<String>>,
    /// 分类ID（可选）
    pub category_id: Option<Option<Uuid>>,
    /// 任务状态（可选）
    pub status: Option<String>,
    /// 优先级（可选）
    pub priority: Option<String>,
    /// 预估时长（秒）（可选）
    pub estimated_duration_seconds: Option<Option<i64>>,
    /// 实际总时长（秒）（可选）
    pub total_duration_seconds: Option<i64>,
    /// 标签（可选）
    pub tags: Option<String>,
    /// 截止日期（可选）
    pub due_date: Option<Option<DateTime<Local>>>,
    /// 是否已完成（可选）
    pub is_completed: Option<bool>,
    /// 完成时间（可选）
    pub completed_at: Option<Option<DateTime<Local>>>,
}

impl TaskModel {
    /// 创建新的任务模型
    pub fn new(name: String, description: Option<String>, category_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            category_id,
            status: "pending".to_string(),
            priority: "medium".to_string(),
            estimated_duration_seconds: None,
            total_duration_seconds: 0,
            tags: "[]".to_string(),
            due_date: None,
            is_completed: false,
            completed_at: None,
            created_at: Local::now(),
            updated_at: None,
        }
    }

    /// 标记任务为已完成
    pub fn mark_completed(&mut self) {
        self.is_completed = true;
        self.completed_at = Some(Local::now());
        self.updated_at = Some(Local::now());
        self.status = "completed".to_string();
    }

    /// 更新任务总时长
    pub fn update_duration(&mut self, additional_seconds: i64) {
        self.total_duration_seconds += additional_seconds;
        self.updated_at = Some(Local::now());
    }

    /// 获取格式化的总时长
    pub fn formatted_duration(&self) -> String {
        let hours = self.total_duration_seconds / 3600;
        let minutes = (self.total_duration_seconds % 3600) / 60;
        let seconds = self.total_duration_seconds % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }

    /// 检查任务是否逾期
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            !self.is_completed && Local::now() > due_date
        } else {
            false
        }
    }
}

impl From<TaskInsert> for TaskModel {
    fn from(insert: TaskInsert) -> Self {
        Self {
            id: insert.id,
            name: insert.name,
            description: insert.description,
            category_id: insert.category_id,
            status: insert.status,
            priority: insert.priority,
            estimated_duration_seconds: insert.estimated_duration_seconds,
            total_duration_seconds: insert.total_duration_seconds,
            tags: insert.tags,
            due_date: insert.due_date,
            is_completed: insert.is_completed,
            completed_at: insert.completed_at,
            created_at: insert.created_at,
            updated_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_model_new() {
        let task = TaskModel::new(
            "测试任务".to_string(),
            Some("这是一个测试任务".to_string()),
            None,
        );

        assert_eq!(task.name, "测试任务");
        assert_eq!(task.description, Some("这是一个测试任务".to_string()));
        assert_eq!(task.status, "pending");
        assert_eq!(task.priority, "medium");
        assert!(!task.is_completed);
        assert_eq!(task.total_duration_seconds, 0);
    }

    #[test]
    fn test_task_mark_completed() {
        let mut task = TaskModel::new("测试任务".to_string(), None, None);

        assert!(!task.is_completed);
        assert!(task.completed_at.is_none());

        task.mark_completed();

        assert!(task.is_completed);
        assert!(task.completed_at.is_some());
        assert_eq!(task.status, "completed");
    }

    #[test]
    fn test_task_update_duration() {
        let mut task = TaskModel::new("测试任务".to_string(), None, None);

        assert_eq!(task.total_duration_seconds, 0);

        task.update_duration(3600); // 1小时
        assert_eq!(task.total_duration_seconds, 3600);

        task.update_duration(1800); // 30分钟
        assert_eq!(task.total_duration_seconds, 5400);
    }

    #[test]
    fn test_formatted_duration() {
        let mut task = TaskModel::new("测试任务".to_string(), None, None);

        // 测试小于1小时的格式
        task.total_duration_seconds = 1830; // 30分30秒
        assert_eq!(task.formatted_duration(), "30:30");

        // 测试大于1小时的格式
        task.total_duration_seconds = 3661; // 1小时1分1秒
        assert_eq!(task.formatted_duration(), "01:01:01");
    }
}
