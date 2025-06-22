//! # 计时器模块
//!
//! 提供精确的时间计算和状态管理功能

use crate::errors::{AppError, Result};
use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};

/// 计时器状态枚举
///
/// 表示计时器的当前状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum TimerState {
    /// 停止状态 - 计时器未启动
    #[default]
    Stopped,
    /// 运行状态 - 正在计时
    Running {
        /// 开始时间
        start_time: DateTime<Local>,
        /// 累计暂停时长
        paused_duration: Duration,
    },
    /// 暂停状态 - 计时暂停
    Paused {
        /// 开始时间
        start_time: DateTime<Local>,
        /// 暂停开始时间
        pause_start: DateTime<Local>,
        /// 累计暂停时长
        paused_duration: Duration,
    },
}


/// 计时器核心结构体
///
/// 负责管理时间计算和状态转换
#[derive(Debug, Clone)]
pub struct Timer {
    /// 当前状态
    state: TimerState,
}

impl Timer {
    /// 创建新的计时器实例
    ///
    /// # 示例
    /// ```
    /// use time_tracker::core::Timer;
    ///
    /// let timer = Timer::new();
    /// assert_eq!(timer.get_state(), &TimerState::Stopped);
    /// ```
    pub fn new() -> Self {
        Self {
            state: TimerState::Stopped,
        }
    }

    /// 启动计时器
    ///
    /// 将计时器从停止状态转换为运行状态
    ///
    /// # 错误
    /// 如果计时器已经在运行，返回错误
    pub fn start(&mut self) -> Result<()> {
        match self.state {
            TimerState::Stopped => {
                self.state = TimerState::Running {
                    start_time: Local::now(),
                    paused_duration: Duration::zero(),
                };
                log::debug!("计时器启动");
                Ok(())
            }
            _ => Err(AppError::TimerState(
                "计时器已经在运行或暂停状态".to_string(),
            )),
        }
    }

    /// 暂停计时器
    ///
    /// 将计时器从运行状态转换为暂停状态
    ///
    /// # 错误
    /// 如果计时器不在运行状态，返回错误
    pub fn pause(&mut self) -> Result<()> {
        match self.state {
            TimerState::Running {
                start_time,
                paused_duration,
            } => {
                self.state = TimerState::Paused {
                    start_time,
                    pause_start: Local::now(),
                    paused_duration,
                };
                log::debug!("计时器暂停");
                Ok(())
            }
            _ => Err(AppError::TimerState("计时器不在运行状态".to_string())),
        }
    }

    /// 恢复计时器
    ///
    /// 将计时器从暂停状态转换为运行状态
    ///
    /// # 错误
    /// 如果计时器不在暂停状态，返回错误
    pub fn resume(&mut self) -> Result<()> {
        match self.state {
            TimerState::Paused {
                start_time,
                pause_start,
                paused_duration,
            } => {
                let additional_pause = Local::now() - pause_start;
                self.state = TimerState::Running {
                    start_time,
                    paused_duration: paused_duration + additional_pause,
                };
                log::debug!("计时器恢复");
                Ok(())
            }
            _ => Err(AppError::TimerState("计时器不在暂停状态".to_string())),
        }
    }

    /// 停止计时器并返回总时长
    ///
    /// 将计时器重置为停止状态，并返回本次计时的总时长
    ///
    /// # 返回
    /// 返回计时的总时长（不包括暂停时间）
    pub fn stop(&mut self) -> Result<Duration> {
        let duration = match self.state {
            TimerState::Running {
                start_time,
                paused_duration,
            } => {
                let total_time = Local::now() - start_time;
                total_time - paused_duration
            }
            TimerState::Paused {
                start_time,
                pause_start,
                paused_duration,
            } => {
                let time_until_pause = pause_start - start_time;
                time_until_pause - paused_duration
            }
            TimerState::Stopped => {
                return Err(AppError::TimerState("计时器已经停止".to_string()));
            }
        };

        self.state = TimerState::Stopped;
        log::debug!("计时器停止，总时长: {:?}", duration);
        Ok(duration)
    }

    /// 获取当前已计时时长
    ///
    /// 返回当前已经计时的时长（不包括暂停时间）
    /// 如果计时器停止，返回零时长
    pub fn get_elapsed(&self) -> Duration {
        match &self.state {
            TimerState::Running {
                start_time,
                paused_duration,
            } => {
                let total_time = Local::now() - *start_time;
                total_time - *paused_duration
            }
            TimerState::Paused {
                start_time,
                pause_start,
                paused_duration,
            } => {
                let time_until_pause = *pause_start - *start_time;
                time_until_pause - *paused_duration
            }
            TimerState::Stopped => Duration::zero(),
        }
    }

    /// 获取计时器当前状态
    pub fn get_state(&self) -> &TimerState {
        &self.state
    }

    /// 获取计时器当前状态（别名）
    pub fn state(&self) -> &TimerState {
        &self.state
    }

    /// 检查计时器是否正在运行
    pub fn is_running(&self) -> bool {
        matches!(self.state, TimerState::Running { .. })
    }

    /// 检查计时器是否暂停
    pub fn is_paused(&self) -> bool {
        matches!(self.state, TimerState::Paused { .. })
    }

    /// 检查计时器是否停止
    pub fn is_stopped(&self) -> bool {
        matches!(self.state, TimerState::Stopped)
    }

    /// 重置计时器到初始状态
    pub fn reset(&mut self) {
        self.state = TimerState::Stopped;
        log::debug!("计时器重置");
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_timer_creation() {
        let timer = Timer::new();
        assert!(timer.is_stopped());
        assert_eq!(timer.get_elapsed(), Duration::zero());
    }

    #[test]
    fn test_timer_start_stop() {
        let mut timer = Timer::new();

        // 启动计时器
        assert!(timer.start().is_ok());
        assert!(timer.is_running());

        // 等待一小段时间
        thread::sleep(StdDuration::from_millis(10));

        // 停止计时器
        let duration = timer.stop().unwrap();
        assert!(timer.is_stopped());
        assert!(duration > Duration::zero());
    }

    #[test]
    fn test_timer_pause_resume() {
        let mut timer = Timer::new();

        // 启动并暂停
        timer.start().unwrap();
        thread::sleep(StdDuration::from_millis(10));
        timer.pause().unwrap();
        assert!(timer.is_paused());

        // 恢复并停止
        timer.resume().unwrap();
        assert!(timer.is_running());

        let duration = timer.stop().unwrap();
        assert!(duration > Duration::zero());
    }

    #[test]
    fn test_timer_state_errors() {
        let mut timer = Timer::new();

        // 停止状态下不能暂停
        assert!(timer.pause().is_err());

        // 停止状态下不能恢复
        assert!(timer.resume().is_err());

        // 停止状态下不能再次停止
        assert!(timer.stop().is_err());

        // 启动后不能再次启动
        timer.start().unwrap();
        assert!(timer.start().is_err());
    }

    #[test]
    fn test_timer_reset() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        timer.reset();
        assert!(timer.is_stopped());
    }
}
