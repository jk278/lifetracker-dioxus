//! # 任务管理模块
//!
//! 提供任务的创建、管理和状态跟踪功能

use crate::errors::{AppError, Result};
use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 任务优先级枚举
///
/// 表示任务的优先级级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum Priority {
    /// 低优先级
    Low,
    /// 中等优先级
    #[default]
    Medium,
    /// 高优先级
    High,
    /// 紧急优先级
    Urgent,
}

/// 任务状态枚举
///
/// 表示任务的当前状态
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize, Default)]
pub enum TaskStatus {
    /// 活动状态 - 正在进行的任务
    #[default]
    Active,
    /// 暂停状态 - 暂时停止的任务
    Paused,
    /// 完成状态 - 已完成的任务
    Completed,
    /// 取消状态 - 被取消的任务
    Cancelled,
}

/// 任务结构体
///
/// 表示一个时间追踪任务的完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务唯一标识符
    pub id: Uuid,
    /// 任务名称
    pub name: String,
    /// 任务描述
    pub description: Option<String>,
    /// 分类ID
    pub category_id: Option<Uuid>,
    /// 任务状态
    pub status: TaskStatus,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 开始时间
    pub started_at: Option<DateTime<Local>>,
    /// 完成时间
    pub completed_at: Option<DateTime<Local>>,
    /// 总计时时长
    pub total_duration: Duration,
    /// 任务标签
    pub tags: Vec<String>,
    /// 优先级
    pub priority: Priority,
    /// 预估时长
    pub estimated_duration: Option<Duration>,
}

impl Task {
    /// 创建新任务
    ///
    /// # 参数
    /// * `name` - 任务名称
    /// * `category_id` - 分类ID（可选）
    /// * `description` - 任务描述（可选）
    ///
    /// # 示例
    /// ```
    /// use time_tracker::core::Task;
    ///
    /// let task = Task::new(
    ///     "学习Rust".to_string(),
    ///     None,
    ///     Some("学习Rust编程语言".to_string())
    /// );
    /// ```
    pub fn new(name: String, category_id: Option<Uuid>, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            category_id,
            status: TaskStatus::Active,
            created_at: Local::now(),
            started_at: None,
            completed_at: None,
            total_duration: Duration::zero(),
            tags: Vec::new(),
            priority: Priority::Medium, // 默认中等优先级
            estimated_duration: None,
        }
    }

    /// 开始任务
    pub fn start(&mut self) -> Result<()> {
        match self.status {
            TaskStatus::Active | TaskStatus::Paused => {
                self.started_at = Some(Local::now());
                self.status = TaskStatus::Active;
                log::debug!("任务开始: {}", self.name);
                Ok(())
            }
            _ => Err(AppError::TimerState(format!(
                "任务 '{}' 无法开始，当前状态: {:?}",
                self.name, self.status
            ))),
        }
    }

    /// 暂停任务
    pub fn pause(&mut self) -> Result<()> {
        match self.status {
            TaskStatus::Active => {
                self.status = TaskStatus::Paused;
                log::debug!("任务暂停: {}", self.name);
                Ok(())
            }
            _ => Err(AppError::TimerState(format!(
                "任务 '{}' 无法暂停，当前状态: {:?}",
                self.name, self.status
            ))),
        }
    }

    /// 完成任务
    ///
    /// # 参数
    /// * `duration` - 本次计时的时长
    pub fn complete(&mut self, duration: Duration) -> Result<()> {
        self.total_duration += duration;
        self.completed_at = Some(Local::now());
        self.status = TaskStatus::Completed;
        log::info!("任务完成: {}, 总时长: {:?}", self.name, self.total_duration);
        Ok(())
    }

    /// 取消任务
    pub fn cancel(&mut self) -> Result<()> {
        self.status = TaskStatus::Cancelled;
        log::debug!("任务取消: {}", self.name);
        Ok(())
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// 移除标签
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// 设置任务优先级
    ///
    /// # 参数
    /// * `priority` - 优先级
    pub fn set_priority(&mut self, priority: Priority) -> Result<()> {
        self.priority = priority;
        Ok(())
    }

    /// 设置预估时长
    pub fn set_estimated_duration(&mut self, duration: Duration) {
        self.estimated_duration = Some(duration);
    }

    /// 检查任务是否活动
    pub fn is_active(&self) -> bool {
        self.status == TaskStatus::Active
    }

    /// 检查任务是否暂停
    pub fn is_paused(&self) -> bool {
        self.status == TaskStatus::Paused
    }

    /// 检查任务是否完成
    pub fn is_completed(&self) -> bool {
        self.status == TaskStatus::Completed
    }

    /// 获取任务进度百分比（基于预估时长）
    pub fn get_progress_percentage(&self) -> Option<f32> {
        self.estimated_duration.map(|estimated| {
            let progress =
                self.total_duration.num_seconds() as f32 / estimated.num_seconds() as f32;
            (progress * 100.0).min(100.0)
        })
    }
}

/// 任务管理器
///
/// 负责管理所有任务的生命周期
#[derive(Debug, Clone)]
pub struct TaskManager {
    /// 任务存储
    tasks: HashMap<Uuid, Task>,
    /// 活动任务ID
    active_task_id: Option<Uuid>,
}

impl TaskManager {
    /// 创建新的任务管理器
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            active_task_id: None,
        }
    }

    /// 添加任务
    pub fn add_task(&mut self, task: Task) -> Result<()> {
        let task_id = task.id;
        self.tasks.insert(task_id, task);
        log::debug!("添加任务: {}", task_id);
        Ok(())
    }

    /// 获取任务
    pub fn get_task(&self, task_id: Uuid) -> Option<&Task> {
        self.tasks.get(&task_id)
    }

    /// 获取可变任务引用
    pub fn get_task_mut(&mut self, task_id: Uuid) -> Option<&mut Task> {
        self.tasks.get_mut(&task_id)
    }

    /// 删除任务
    pub fn remove_task(&mut self, task_id: Uuid) -> Result<Task> {
        self.tasks
            .remove(&task_id)
            .ok_or_else(|| AppError::TaskNotFound(task_id.to_string()))
    }

    /// 完成任务
    pub fn complete_task(&mut self, task_id: Uuid, duration: Duration) -> Result<()> {
        let task = self
            .get_task_mut(task_id)
            .ok_or_else(|| AppError::TaskNotFound(task_id.to_string()))?;

        task.complete(duration)?;

        // 如果是当前活动任务，清除活动状态
        if self.active_task_id == Some(task_id) {
            self.active_task_id = None;
        }

        Ok(())
    }

    /// 设置活动任务
    pub fn set_active_task(&mut self, task_id: Uuid) -> Result<()> {
        if !self.tasks.contains_key(&task_id) {
            return Err(AppError::TaskNotFound(task_id.to_string()));
        }

        self.active_task_id = Some(task_id);
        Ok(())
    }

    /// 获取活动任务
    pub fn get_active_task(&self) -> Option<&Task> {
        self.active_task_id.and_then(|id| self.tasks.get(&id))
    }

    /// 获取所有任务
    pub fn get_all_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    /// 按状态筛选任务
    pub fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.status == status)
            .collect()
    }

    /// 按分类筛选任务
    pub fn get_tasks_by_category(&self, category_id: Uuid) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.category_id == Some(category_id))
            .collect()
    }

    /// 搜索任务
    pub fn search_tasks(&self, query: &str) -> Vec<&Task> {
        let query_lower = query.to_lowercase();
        self.tasks
            .values()
            .filter(|task| {
                task.name.to_lowercase().contains(&query_lower)
                    || task
                        .description
                        .as_ref()
                        .is_some_and(|desc| desc.to_lowercase().contains(&query_lower))
                    || task
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// 获取任务总数
    pub fn get_task_count(&self) -> usize {
        self.tasks.len()
    }

    /// 清空所有任务
    pub fn clear(&mut self) {
        self.tasks.clear();
        self.active_task_id = None;
        log::debug!("清空所有任务");
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "测试任务".to_string(),
            None,
            Some("这是一个测试任务".to_string()),
        );

        assert_eq!(task.name, "测试任务");
        assert_eq!(task.description, Some("这是一个测试任务".to_string()));
        assert_eq!(task.status, TaskStatus::Active);
        assert_eq!(task.total_duration, Duration::zero());
    }

    #[test]
    fn test_task_lifecycle() {
        let mut task = Task::new("测试".to_string(), None, None);

        // 开始任务
        assert!(task.start().is_ok());
        assert!(task.is_active());

        // 暂停任务
        assert!(task.pause().is_ok());
        assert!(task.is_paused());

        // 恢复任务
        assert!(task.start().is_ok());
        assert!(task.is_active());

        // 完成任务
        let duration = Duration::minutes(30);
        assert!(task.complete(duration).is_ok());
        assert!(task.is_completed());
        assert_eq!(task.total_duration, duration);
    }

    #[test]
    fn test_task_manager() {
        let mut manager = TaskManager::new();

        // 添加任务
        let task = Task::new("测试任务".to_string(), None, None);
        let task_id = task.id;
        manager.add_task(task).unwrap();

        // 获取任务
        assert!(manager.get_task(task_id).is_some());
        assert_eq!(manager.get_task_count(), 1);

        // 设置活动任务
        manager.set_active_task(task_id).unwrap();
        assert!(manager.get_active_task().is_some());

        // 完成任务
        manager
            .complete_task(task_id, Duration::minutes(15))
            .unwrap();
        assert!(manager.get_active_task().is_none());
    }

    #[test]
    fn test_task_tags_and_priority() {
        let mut task = Task::new("测试".to_string(), None, None);

        // 添加标签
        task.add_tag("重要".to_string());
        task.add_tag("紧急".to_string());
        assert_eq!(task.tags.len(), 2);

        // 移除标签
        task.remove_tag("重要");
        assert_eq!(task.tags.len(), 1);

        // 设置优先级
        assert!(task.set_priority(Priority::Urgent).is_ok());
        assert_eq!(task.priority, Priority::Urgent);
    }
}
