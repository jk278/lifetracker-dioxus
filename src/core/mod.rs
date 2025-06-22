//! # 核心业务逻辑模块
//!
//! 包含时间追踪器的核心功能实现：
//! - 计时器逻辑
//! - 任务管理
//! - 分类管理
//! - 数据分析

pub mod analytics;
pub mod category; // 分类管理
pub mod task; // 任务管理
pub mod timer; // 计时器核心逻辑 // 数据分析和统计

// 重新导出主要类型
pub use analytics::{Analytics, AnalyticsReport};
pub use category::{Category, CategoryColor, CategoryIcon, CategoryManager};
pub use task::{Priority, Task, TaskManager, TaskStatus};
pub use timer::{Timer, TimerState};

use crate::errors::{AppError, Result};
use chrono::Duration;

/// 应用程序核心状态管理器
///
/// 协调各个模块之间的交互，维护应用程序的全局状态
#[derive(Debug)]
pub struct AppCore {
    /// 计时器实例
    pub timer: Timer,
    /// 任务管理器
    pub task_manager: TaskManager,
    /// 分类管理器
    pub category_manager: CategoryManager,
    /// 数据分析器
    pub analytics: Analytics,
    /// 当前活动任务ID
    current_task_id: Option<uuid::Uuid>,
}

impl AppCore {
    /// 创建新的应用核心实例
    pub fn new() -> Self {
        Self {
            timer: Timer::new(),
            task_manager: TaskManager::new(),
            category_manager: CategoryManager::new(),
            analytics: Analytics::new(),
            current_task_id: None,
        }
    }

    /// 开始新任务计时
    ///
    /// # 参数
    /// * `task_name` - 任务名称
    /// * `category_id` - 分类ID（可选）
    /// * `description` - 任务描述（可选）
    pub fn start_task(
        &mut self,
        task_name: String,
        category_id: Option<uuid::Uuid>,
        description: Option<String>,
    ) -> Result<uuid::Uuid> {
        // 如果有正在进行的任务，先停止它
        if let Some(_current_id) = self.current_task_id {
            self.stop_current_task()?;
        }

        // 创建新任务
        let task = Task::new(task_name, category_id, description);
        let task_id = task.id;

        // 添加到任务管理器
        self.task_manager.add_task(task)?;

        // 启动计时器
        self.timer.start()?;
        self.current_task_id = Some(task_id);

        log::info!("开始任务: {}", task_id);
        Ok(task_id)
    }

    /// 暂停当前任务
    pub fn pause_current_task(&mut self) -> Result<()> {
        if self.current_task_id.is_none() {
            return Err(AppError::TimerState("没有正在进行的任务".to_string()));
        }

        self.timer.pause()?;
        log::info!("暂停当前任务");
        Ok(())
    }

    /// 恢复当前任务
    pub fn resume_current_task(&mut self) -> Result<()> {
        if self.current_task_id.is_none() {
            return Err(AppError::TimerState("没有暂停的任务".to_string()));
        }

        self.timer.resume()?;
        log::info!("恢复当前任务");
        Ok(())
    }

    /// 停止当前任务
    pub fn stop_current_task(&mut self) -> Result<Duration> {
        let task_id = self
            .current_task_id
            .ok_or_else(|| AppError::TimerState("没有正在进行的任务".to_string()))?;

        // 停止计时器并获取总时长
        let duration = self.timer.stop()?;

        // 更新任务状态
        self.task_manager.complete_task(task_id, duration)?;

        self.current_task_id = None;
        log::info!("停止任务: {}, 用时: {:?}", task_id, duration);
        Ok(duration)
    }

    /// 获取当前任务信息
    pub fn get_current_task(&self) -> Option<&Task> {
        self.current_task_id
            .and_then(|id| self.task_manager.get_task(id))
    }

    /// 获取计时器引用
    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    /// 获取计时器状态
    pub fn get_timer_state(&self) -> &TimerState {
        self.timer.get_state()
    }

    /// 获取当前计时时长
    pub fn get_current_duration(&self) -> Duration {
        self.timer.get_elapsed()
    }

    /// 暂停任务（别名方法）
    pub fn pause_task(&mut self) -> Result<()> {
        self.pause_current_task()
    }

    /// 恢复任务（别名方法）
    pub fn resume_task(&mut self) -> Result<()> {
        self.resume_current_task()
    }

    /// 停止任务（别名方法）
    pub fn stop_task(&mut self) -> Result<Duration> {
        self.stop_current_task()
    }

    /// 获取已用时长（别名方法）
    pub fn elapsed(&self) -> Duration {
        self.timer.get_elapsed()
    }

    /// 检查是否有活动任务
    pub fn has_active_task(&self) -> bool {
        self.current_task_id.is_some()
    }

    /// 获取所有任务
    pub fn get_tasks(&self) -> Result<Vec<Task>> {
        Ok(self
            .task_manager
            .get_all_tasks()
            .into_iter()
            .cloned()
            .collect())
    }

    /// 获取所有分类
    pub fn get_categories(&self) -> Result<Vec<Category>> {
        Ok(self
            .category_manager
            .get_all_categories()
            .into_iter()
            .cloned()
            .collect())
    }

    /// 获取应用配置
    pub fn config(&self) -> &crate::config::AppConfig {
        // 这里需要一个默认配置引用，暂时返回静态配置
        // 在实际应用中，应该从配置管理器获取
        static DEFAULT_CONFIG: std::sync::OnceLock<crate::config::AppConfig> =
            std::sync::OnceLock::new();
        DEFAULT_CONFIG.get_or_init(crate::config::AppConfig::default)
    }

    /// 更新应用配置
    pub fn update_config(&mut self, config: crate::config::AppConfig) -> Result<()> {
        // 在实际应用中，这里应该更新配置管理器
        // 目前只是一个占位实现，但我们可以记录配置的一些关键信息
        log::info!(
            "配置已更新 - 主题: {}, 自动开始计时: {}, 默认分类: {:?}",
            config.ui.theme,
            config.general.auto_start_timer,
            config.general.default_category_id
        );

        // 记录工作提醒设置
        if let Some(work_interval) = config.general.work_reminder_interval {
            log::info!("应用工作提醒设置: {} 分钟", work_interval);
        }

        if let Some(break_interval) = config.general.break_reminder_interval {
            log::info!("应用休息提醒设置: {} 分钟", break_interval);
        }

        // 如果有通知设置，记录通知配置
        if config.notifications.enabled {
            log::info!(
                "通知已启用 - 任务开始: {}, 任务结束: {}, 工作提醒: {}, 休息提醒: {}",
                config.notifications.notify_task_start,
                config.notifications.notify_task_end,
                config.notifications.notify_work_time,
                config.notifications.notify_break_time
            );
        }

        // 记录数据配置
        log::info!(
            "数据配置 - 自动备份: {}, 备份间隔: {} 天, 数据库路径: {:?}",
            config.data.auto_backup,
            config.data.backup_interval,
            config.data.database_path
        );

        Ok(())
    }

    /// 创建新任务
    pub fn create_task(
        &mut self,
        name: String,
        description: String,
        category_id: Option<uuid::Uuid>,
        priority: Priority,
        estimated_duration: Option<Duration>,
        tags: Vec<String>,
        due_date: Option<chrono::NaiveDate>,
    ) -> Result<uuid::Uuid> {
        let mut task = Task::new(name, category_id, Some(description));
        task.priority = priority;
        task.estimated_duration = estimated_duration;
        task.tags = tags;
        // 如果有截止日期，添加到任务描述或者将来的due_date字段中
        if let Some(due_date) = due_date {
            // 暂时将截止日期添加到标签中，直到Task结构支持due_date字段
            task.tags
                .push(format!("截止日期:{}", due_date.format("%Y-%m-%d")));
        }

        let task_id = task.id;
        self.task_manager.add_task(task)?;

        log::info!("创建任务: {}, 截止日期: {:?}", task_id, due_date);
        Ok(task_id)
    }

    /// 更新任务
    pub fn update_task(
        &mut self,
        task_id: uuid::Uuid,
        name: Option<String>,
        description: Option<String>,
        category_id: Option<uuid::Uuid>,
        priority: Option<Priority>,
        estimated_duration: Option<Duration>,
        tags: Option<Vec<String>>,
        due_date: Option<chrono::NaiveDate>,
    ) -> Result<()> {
        let task = self
            .task_manager
            .get_task_mut(task_id)
            .ok_or_else(|| AppError::TaskNotFound(task_id.to_string()))?;

        if let Some(name) = name {
            task.name = name;
        }
        if let Some(description) = description {
            task.description = Some(description);
        }
        if let Some(category_id) = category_id {
            task.category_id = Some(category_id);
        }
        if let Some(priority) = priority {
            task.priority = priority;
        }
        if let Some(estimated_duration) = estimated_duration {
            task.estimated_duration = Some(estimated_duration);
        }
        if let Some(mut tags) = tags {
            // 如果有截止日期，添加到标签中
            if let Some(due_date) = due_date {
                // 移除现有的截止日期标签
                tags.retain(|tag| !tag.starts_with("截止日期:"));
                // 添加新的截止日期标签
                tags.push(format!("截止日期:{}", due_date.format("%Y-%m-%d")));
            }
            task.tags = tags;
        } else if let Some(due_date) = due_date {
            // 只更新截止日期时，保留现有标签并更新截止日期
            task.tags.retain(|tag| !tag.starts_with("截止日期:"));
            task.tags
                .push(format!("截止日期:{}", due_date.format("%Y-%m-%d")));
        }

        log::info!("更新任务: {}, 截止日期: {:?}", task_id, due_date);
        Ok(())
    }

    /// 删除任务
    pub fn delete_task(&mut self, task_id: uuid::Uuid) -> Result<()> {
        self.task_manager.remove_task(task_id)?;

        // 如果删除的是当前任务，清除当前任务ID
        if self.current_task_id == Some(task_id) {
            self.current_task_id = None;
            self.timer.stop()?;
        }

        log::info!("删除任务: {}", task_id);
        Ok(())
    }

    /// 开始指定任务计时
    pub fn start_task_by_id(&mut self, task_id: uuid::Uuid) -> Result<()> {
        // 检查任务是否存在
        if self.task_manager.get_task(task_id).is_none() {
            return Err(AppError::TaskNotFound(format!("任务 {} 不存在", task_id)));
        }

        // 如果有正在进行的任务，先停止它
        if let Some(current_id) = self.current_task_id {
            if current_id != task_id {
                self.stop_current_task()?;
            } else {
                return Err(AppError::TimerState("任务已经在运行".to_string()));
            }
        }

        // 启动计时器
        self.timer.start()?;
        self.current_task_id = Some(task_id);

        log::info!("开始任务: {}", task_id);
        Ok(())
    }

    /// 完成任务
    pub fn complete_task(&mut self, task_id: uuid::Uuid) -> Result<()> {
        // 获取任务
        let _task = self
            .task_manager
            .get_task(task_id)
            .ok_or_else(|| AppError::TaskNotFound(format!("任务 {} 不存在", task_id)))?;

        // 如果是当前任务，先停止计时器
        if self.current_task_id == Some(task_id) {
            let duration = self.stop_current_task()?;
            self.task_manager.complete_task(task_id, duration)?;
        } else {
            // 直接标记为完成
            self.task_manager.complete_task(task_id, Duration::zero())?;
        }

        log::info!("完成任务: {}", task_id);
        Ok(())
    }

    /// 生成分析报告
    pub fn generate_analytics_report(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<crate::core::analytics::AnalyticsReport> {
        self.analytics.generate_report(start_date, end_date)
    }

    /// 分析趋势
    pub fn analyze_trends(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<crate::core::analytics::TrendAnalysis> {
        // 获取期间的每日统计数据
        let report = self.analytics.generate_report(start_date, end_date)?;
        Ok(report.trends)
    }
}

impl Default for AppCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_core_creation() {
        let core = AppCore::new();
        assert!(!core.has_active_task());
        assert_eq!(core.get_current_duration(), Duration::zero());
    }

    #[test]
    fn test_start_and_stop_task() {
        let mut core = AppCore::new();

        // 开始任务
        let task_id = core
            .start_task(
                "测试任务".to_string(),
                None,
                Some("这是一个测试任务".to_string()),
            )
            .unwrap();

        assert!(core.has_active_task());
        assert_eq!(core.current_task_id, Some(task_id));

        // 停止任务
        let duration = core.stop_current_task().unwrap();
        assert!(!core.has_active_task());
        assert!(duration >= Duration::zero());
    }
}
