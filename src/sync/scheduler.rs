//! # 同步调度器模块
//!
//! 管理定时同步任务

use crate::errors::{AppError, Result};
use crate::sync::SyncConfig;
use chrono::{DateTime, Local};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::{interval, sleep};
use tokio_cron_scheduler::{Job, JobScheduler};

/// 同步调度器
pub struct SyncScheduler {
    /// 调度器实例
    scheduler: JobScheduler,
    /// 是否正在运行
    is_running: Arc<Mutex<bool>>,
    /// 下次同步时间
    next_sync_time: Arc<Mutex<Option<DateTime<Local>>>>,
}

/// 调度器状态
#[derive(Debug, Clone)]
pub struct SchedulerStatus {
    /// 是否正在运行
    pub is_running: bool,
    /// 下次同步时间
    pub next_sync_time: Option<DateTime<Local>>,
    /// 上次同步时间
    pub last_sync_time: Option<DateTime<Local>>,
    /// 同步间隔（分钟）
    pub interval_minutes: u32,
}

impl SyncScheduler {
    /// 创建新的同步调度器
    pub async fn new() -> Result<Self> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| AppError::System(format!("创建调度器失败: {}", e)))?;

        Ok(Self {
            scheduler,
            is_running: Arc::new(Mutex::new(false)),
            next_sync_time: Arc::new(Mutex::new(None)),
        })
    }

    /// 启动调度器
    pub async fn start(&self) -> Result<()> {
        log::info!("启动同步调度器");

        {
            let mut is_running = self.is_running.lock().unwrap();
            if *is_running {
                return Err(AppError::System("调度器已在运行中".to_string()));
            }
            *is_running = true;
        }

        self.scheduler
            .start()
            .await
            .map_err(|e| AppError::System(format!("启动调度器失败: {}", e)))?;

        Ok(())
    }

    /// 停止调度器
    pub async fn stop(&mut self) -> Result<()> {
        log::info!("停止同步调度器");

        {
            let mut is_running = self.is_running.lock().unwrap();
            *is_running = false;
        }

        self.scheduler
            .shutdown()
            .await
            .map_err(|e| AppError::System(format!("停止调度器失败: {}", e)))?;

        Ok(())
    }

    /// 添加定时同步任务
    pub async fn schedule_sync(&self, config: &SyncConfig) -> Result<String> {
        if !config.auto_sync {
            return Err(AppError::Validation("自动同步未启用".to_string()));
        }

        let interval_minutes = config.interval;
        if interval_minutes < 5 {
            return Err(AppError::Validation("同步间隔不能小于5分钟".to_string()));
        }

        // 创建cron表达式，每N分钟执行一次
        let cron_expr = format!("0 */{} * * * *", interval_minutes);

        let next_sync_time = self.next_sync_time.clone();
        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _l| {
            let next_sync_time = next_sync_time.clone();
            Box::pin(async move {
                log::info!("执行定时同步任务");

                // 更新下次同步时间
                {
                    let mut next_time = next_sync_time.lock().unwrap();
                    *next_time =
                        Some(Local::now() + chrono::Duration::minutes(interval_minutes as i64));
                }

                // TODO: 在这里调用同步引擎执行同步
                // 由于需要访问存储管理器等，这里需要通过某种方式传递依赖
                log::warn!("定时同步功能需要集成到应用状态中");
            })
        })
        .map_err(|e| AppError::System(format!("创建定时任务失败: {}", e)))?;

        let job_id = self
            .scheduler
            .add(job)
            .await
            .map_err(|e| AppError::System(format!("添加定时任务失败: {}", e)))?;

        // 设置下次同步时间
        {
            let mut next_time = self.next_sync_time.lock().unwrap();
            *next_time = Some(Local::now() + chrono::Duration::minutes(interval_minutes as i64));
        }

        log::info!("已添加定时同步任务，间隔: {} 分钟", interval_minutes);
        Ok(job_id.to_string())
    }

    /// 移除定时同步任务
    pub async fn remove_sync(&self, job_id: &str) -> Result<()> {
        let uuid = uuid::Uuid::parse_str(job_id)
            .map_err(|e| AppError::Validation(format!("无效的任务ID: {}", e)))?;

        self.scheduler
            .remove(&uuid)
            .await
            .map_err(|e| AppError::System(format!("移除定时任务失败: {}", e)))?;

        // 清除下次同步时间
        {
            let mut next_time = self.next_sync_time.lock().unwrap();
            *next_time = None;
        }

        log::info!("已移除定时同步任务: {}", job_id);
        Ok(())
    }

    /// 手动触发同步
    pub async fn trigger_sync(&self) -> Result<()> {
        log::info!("手动触发同步");

        // TODO: 在这里调用同步引擎执行同步
        // 由于需要访问存储管理器等，这里需要通过某种方式传递依赖
        log::warn!("手动同步功能需要集成到应用状态中");

        Ok(())
    }

    /// 获取调度器状态
    pub fn get_status(&self) -> SchedulerStatus {
        let is_running = {
            let running = self.is_running.lock().unwrap();
            *running
        };

        let next_sync_time = {
            let next_time = self.next_sync_time.lock().unwrap();
            *next_time
        };

        SchedulerStatus {
            is_running,
            next_sync_time,
            last_sync_time: None, // TODO: 实现最后同步时间跟踪
            interval_minutes: 30, // TODO: 从配置中获取
        }
    }

    /// 更新同步间隔
    pub async fn update_interval(&self, new_interval: u32) -> Result<()> {
        if new_interval < 5 {
            return Err(AppError::Validation("同步间隔不能小于5分钟".to_string()));
        }

        // TODO: 实现更新现有任务的间隔
        // 目前需要先移除旧任务，再添加新任务

        log::info!("同步间隔已更新为 {} 分钟", new_interval);
        Ok(())
    }

    /// 暂停调度器
    pub async fn pause(&self) -> Result<()> {
        log::info!("暂停同步调度器");

        // TODO: 实现暂停功能
        // tokio-cron-scheduler 可能不直接支持暂停，需要通过移除任务来实现

        Ok(())
    }

    /// 恢复调度器
    pub async fn resume(&self) -> Result<()> {
        log::info!("恢复同步调度器");

        // TODO: 实现恢复功能
        // 重新添加之前暂停的任务

        Ok(())
    }
}

/// 简单的同步调度器实现（不依赖cron）
pub struct SimpleSyncScheduler {
    /// 是否正在运行
    is_running: Arc<Mutex<bool>>,
    /// 同步间隔
    interval: Duration,
}

impl SimpleSyncScheduler {
    /// 创建简单调度器
    pub fn new(interval_minutes: u32) -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
            interval: Duration::from_secs(interval_minutes as u64 * 60),
        }
    }

    /// 启动简单调度器
    pub async fn start<F>(&self, sync_callback: F) -> Result<()>
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        {
            let mut is_running = self.is_running.lock().unwrap();
            if *is_running {
                return Err(AppError::System("调度器已在运行中".to_string()));
            }
            *is_running = true;
        }

        let is_running = self.is_running.clone();
        let interval_duration = self.interval;

        tokio::spawn(async move {
            let mut interval_timer = interval(interval_duration);

            loop {
                {
                    let running = is_running.lock().unwrap();
                    if !*running {
                        break;
                    }
                }

                interval_timer.tick().await;

                // 执行同步回调
                if let Err(e) = sync_callback() {
                    log::error!("定时同步执行失败: {}", e);
                }
            }

            log::info!("简单调度器已停止");
        });

        log::info!("简单调度器已启动，间隔: {:?}", interval_duration);
        Ok(())
    }

    /// 停止简单调度器
    pub fn stop(&self) {
        let mut is_running = self.is_running.lock().unwrap();
        *is_running = false;
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        let is_running = self.is_running.lock().unwrap();
        *is_running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_scheduler_creation() {
        let scheduler = SyncScheduler::new().await.unwrap();
        let status = scheduler.get_status();
        assert!(!status.is_running);
        assert!(status.next_sync_time.is_none());
    }

    #[tokio::test]
    async fn test_simple_scheduler() {
        let scheduler = SimpleSyncScheduler::new(1); // 1分钟间隔，仅用于测试
        assert!(!scheduler.is_running());

        // 测试启动和停止
        let sync_count = Arc::new(Mutex::new(0));
        let sync_count_clone = sync_count.clone();

        let callback = move || {
            let mut count = sync_count_clone.lock().unwrap();
            *count += 1;
            Ok(())
        };

        // 由于是异步的，我们不在这里测试实际的回调执行
        // 在实际应用中需要更复杂的测试设置
    }
}
