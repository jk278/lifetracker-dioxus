//! # 数据分析模块
//!
//! 提供时间数据的统计分析和报告生成功能

use super::Task;
use crate::errors::{AppError, Result};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 时间统计数据
///
/// 表示特定时间段内的统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStats {
    /// 统计日期
    pub date: NaiveDate,
    /// 按分类统计的时间
    pub category_stats: HashMap<Uuid, Duration>,
    /// 总计时间
    pub total_time: Duration,
    /// 任务数量
    pub task_count: usize,
    /// 完成任务数量
    pub completed_tasks: usize,
    /// 效率评分 (0.0-1.0)
    pub efficiency_score: f32,
    /// 最长单次计时
    pub longest_session: Duration,
    /// 平均单次计时
    pub average_session: Duration,
}

impl TimeStats {
    /// 创建新的时间统计
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            category_stats: HashMap::new(),
            total_time: Duration::zero(),
            task_count: 0,
            completed_tasks: 0,
            efficiency_score: 0.0,
            longest_session: Duration::zero(),
            average_session: Duration::zero(),
        }
    }

    /// 添加分类时间
    pub fn add_category_time(&mut self, category_id: Uuid, duration: Duration) {
        *self
            .category_stats
            .entry(category_id)
            .or_insert(Duration::zero()) += duration;
        self.total_time += duration;
    }

    /// 获取分类时间占比
    pub fn get_category_percentage(&self, category_id: Uuid) -> f32 {
        if self.total_time.is_zero() {
            return 0.0;
        }

        let zero_duration = Duration::zero();
        let category_time = self
            .category_stats
            .get(&category_id)
            .unwrap_or(&zero_duration);

        (category_time.num_seconds() as f32 / self.total_time.num_seconds() as f32) * 100.0
    }

    /// 获取最活跃的分类
    pub fn get_most_active_category(&self) -> Option<Uuid> {
        self.category_stats
            .iter()
            .max_by_key(|(_, duration)| duration.num_seconds())
            .map(|(id, _)| *id)
    }

    /// 计算效率评分
    pub fn calculate_efficiency(&mut self, target_hours: f32) {
        let actual_hours = self.total_time.num_seconds() as f32 / 3600.0;
        let completion_rate = if self.task_count > 0 {
            self.completed_tasks as f32 / self.task_count as f32
        } else {
            0.0
        };

        // 效率评分 = (实际时间/目标时间) * 完成率
        let time_efficiency = if target_hours > 0.0 {
            (actual_hours / target_hours).min(1.0)
        } else {
            0.0
        };

        self.efficiency_score = (time_efficiency * 0.6 + completion_rate * 0.4).min(1.0);
    }
}

/// 分析报告
///
/// 包含详细的时间分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    /// 报告生成时间
    pub generated_at: DateTime<Local>,
    /// 分析时间段
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    /// 每日统计
    pub daily_stats: Vec<TimeStats>,
    /// 总体统计
    pub summary: TimeStats,
    /// 趋势分析
    pub trends: TrendAnalysis,
    /// 建议
    pub recommendations: Vec<String>,
}

/// 趋势分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// 时间趋势 (正数表示增长)
    pub time_trend: f32,
    /// 效率趋势
    pub efficiency_trend: f32,
    /// 最活跃的时间段
    pub peak_hours: Vec<u32>,
    /// 最活跃的星期几
    pub peak_weekdays: Vec<u32>,
    /// 分类趋势
    pub category_trends: HashMap<Uuid, f32>,
}

impl Default for TrendAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl TrendAnalysis {
    /// 创建新的趋势分析
    pub fn new() -> Self {
        Self {
            time_trend: 0.0,
            efficiency_trend: 0.0,
            peak_hours: Vec::new(),
            peak_weekdays: Vec::new(),
            category_trends: HashMap::new(),
        }
    }
}

/// 数据分析器
///
/// 负责处理时间数据的分析和报告生成
#[derive(Debug, Clone)]
pub struct Analytics {
    /// 历史统计数据
    historical_stats: HashMap<NaiveDate, TimeStats>,
}

impl Analytics {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self {
            historical_stats: HashMap::new(),
        }
    }

    /// 添加任务数据到统计
    pub fn add_task_data(&mut self, task: &Task) {
        if !task.is_completed() {
            return;
        }

        let date = task.completed_at.unwrap_or_else(Local::now).date_naive();

        let stats = self
            .historical_stats
            .entry(date)
            .or_insert_with(|| TimeStats::new(date));

        // 添加分类时间
        if let Some(category_id) = task.category_id {
            stats.add_category_time(category_id, task.total_duration);
        }

        // 更新任务统计
        stats.task_count += 1;
        stats.completed_tasks += 1;

        // 更新会话统计
        if task.total_duration > stats.longest_session {
            stats.longest_session = task.total_duration;
        }

        // 重新计算平均时间
        stats.average_session =
            Duration::seconds(stats.total_time.num_seconds() / stats.completed_tasks as i64);

        log::debug!(
            "添加任务数据到统计: {} - {:?}",
            task.name,
            task.total_duration
        );
    }

    /// 获取指定日期的统计
    pub fn get_daily_stats(&self, date: NaiveDate) -> Option<&TimeStats> {
        self.historical_stats.get(&date)
    }

    /// 获取今日统计
    pub fn get_today_stats(&self) -> Option<&TimeStats> {
        let today = Local::now().date_naive();
        self.get_daily_stats(today)
    }

    /// 获取本周统计
    pub fn get_weekly_stats(&self, week_start: NaiveDate) -> TimeStats {
        let mut weekly_stats = TimeStats::new(week_start);

        for i in 0..7 {
            let date = week_start + chrono::Duration::days(i);
            if let Some(daily_stats) = self.get_daily_stats(date) {
                // 合并每日统计
                for (category_id, duration) in &daily_stats.category_stats {
                    weekly_stats.add_category_time(*category_id, *duration);
                }
                weekly_stats.task_count += daily_stats.task_count;
                weekly_stats.completed_tasks += daily_stats.completed_tasks;

                if daily_stats.longest_session > weekly_stats.longest_session {
                    weekly_stats.longest_session = daily_stats.longest_session;
                }
            }
        }

        // 重新计算平均时间
        if weekly_stats.completed_tasks > 0 {
            weekly_stats.average_session = Duration::seconds(
                weekly_stats.total_time.num_seconds() / weekly_stats.completed_tasks as i64,
            );
        }

        weekly_stats
    }

    /// 获取本月统计
    pub fn get_monthly_stats(&self, year: i32, month: u32) -> TimeStats {
        let month_start =
            NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_else(|| Local::now().date_naive());
        let mut monthly_stats = TimeStats::new(month_start);

        // 获取该月的所有日期
        let days_in_month = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
        }
        .day();

        for day in 1..=days_in_month {
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                if let Some(daily_stats) = self.get_daily_stats(date) {
                    // 合并每日统计
                    for (category_id, duration) in &daily_stats.category_stats {
                        monthly_stats.add_category_time(*category_id, *duration);
                    }
                    monthly_stats.task_count += daily_stats.task_count;
                    monthly_stats.completed_tasks += daily_stats.completed_tasks;

                    if daily_stats.longest_session > monthly_stats.longest_session {
                        monthly_stats.longest_session = daily_stats.longest_session;
                    }
                }
            }
        }

        // 重新计算平均时间
        if monthly_stats.completed_tasks > 0 {
            monthly_stats.average_session = Duration::seconds(
                monthly_stats.total_time.num_seconds() / monthly_stats.completed_tasks as i64,
            );
        }

        monthly_stats
    }

    /// 生成分析报告
    pub fn generate_report(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<AnalyticsReport> {
        if start_date > end_date {
            return Err(AppError::TimerState("开始日期不能晚于结束日期".to_string()));
        }

        let mut daily_stats = Vec::new();
        let mut summary = TimeStats::new(start_date);

        // 收集指定时间段的每日统计
        let mut current_date = start_date;
        while current_date <= end_date {
            if let Some(stats) = self.get_daily_stats(current_date) {
                daily_stats.push(stats.clone());

                // 合并到总体统计
                for (category_id, duration) in &stats.category_stats {
                    summary.add_category_time(*category_id, *duration);
                }
                summary.task_count += stats.task_count;
                summary.completed_tasks += stats.completed_tasks;

                if stats.longest_session > summary.longest_session {
                    summary.longest_session = stats.longest_session;
                }
            }
            current_date += chrono::Duration::days(1);
        }

        // 计算平均时间
        if summary.completed_tasks > 0 {
            summary.average_session = Duration::seconds(
                summary.total_time.num_seconds() / summary.completed_tasks as i64,
            );
        }

        // 生成趋势分析
        let trends = self.analyze_trends(&daily_stats);

        // 生成建议
        let recommendations = self.generate_recommendations(&summary, &trends);

        Ok(AnalyticsReport {
            generated_at: Local::now(),
            period_start: start_date,
            period_end: end_date,
            daily_stats,
            summary,
            trends,
            recommendations,
        })
    }

    /// 分析趋势
    pub fn analyze_trends(&self, daily_stats: &[TimeStats]) -> TrendAnalysis {
        let mut trends = TrendAnalysis::new();

        if daily_stats.len() < 2 {
            return trends;
        }

        // 计算时间趋势
        let first_half = &daily_stats[..daily_stats.len() / 2];
        let second_half = &daily_stats[daily_stats.len() / 2..];

        let first_avg = first_half
            .iter()
            .map(|s| s.total_time.num_seconds())
            .sum::<i64>() as f32
            / first_half.len() as f32;

        let second_avg = second_half
            .iter()
            .map(|s| s.total_time.num_seconds())
            .sum::<i64>() as f32
            / second_half.len() as f32;

        trends.time_trend = if first_avg > 0.0 {
            (second_avg - first_avg) / first_avg * 100.0
        } else {
            0.0
        };

        // 计算效率趋势
        let first_eff_avg =
            first_half.iter().map(|s| s.efficiency_score).sum::<f32>() / first_half.len() as f32;

        let second_eff_avg =
            second_half.iter().map(|s| s.efficiency_score).sum::<f32>() / second_half.len() as f32;

        trends.efficiency_trend = (second_eff_avg - first_eff_avg) * 100.0;

        trends
    }

    /// 生成建议
    fn generate_recommendations(&self, summary: &TimeStats, trends: &TrendAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();

        // 基于总时间的建议
        let total_hours = summary.total_time.num_seconds() as f32 / 3600.0;
        if total_hours < 4.0 {
            recommendations.push("建议增加每日工作时间，提高生产力".to_string());
        } else if total_hours > 12.0 {
            recommendations.push("工作时间较长，注意劳逸结合".to_string());
        }

        // 基于效率的建议
        if summary.efficiency_score < 0.5 {
            recommendations.push("效率偏低，建议优化时间管理方法".to_string());
        }

        // 基于趋势的建议
        if trends.time_trend < -10.0 {
            recommendations.push("工作时间呈下降趋势，需要关注".to_string());
        } else if trends.time_trend > 20.0 {
            recommendations.push("工作时间增长较快，注意可持续性".to_string());
        }

        if trends.efficiency_trend < -10.0 {
            recommendations.push("效率呈下降趋势，建议调整工作方式".to_string());
        }

        // 基于任务完成率的建议
        let completion_rate = if summary.task_count > 0 {
            summary.completed_tasks as f32 / summary.task_count as f32
        } else {
            0.0
        };

        if completion_rate < 0.7 {
            recommendations.push("任务完成率偏低，建议合理规划任务量".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("时间管理表现良好，继续保持！".to_string());
        }

        recommendations
    }

    /// 清除历史数据
    pub fn clear_history(&mut self) {
        self.historical_stats.clear();
        log::debug!("清除历史统计数据");
    }

    /// 获取统计数据总数
    pub fn get_stats_count(&self) -> usize {
        self.historical_stats.len()
    }
}

impl Default for Analytics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Task;

    #[test]
    fn test_time_stats_creation() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let stats = TimeStats::new(date);

        assert_eq!(stats.date, date);
        assert_eq!(stats.total_time, Duration::zero());
        assert_eq!(stats.task_count, 0);
    }

    #[test]
    fn test_analytics_add_task() {
        let mut analytics = Analytics::new();

        // 创建完成的任务
        let mut task = Task::new("测试任务".to_string(), Some(Uuid::new_v4()), None);
        task.complete(Duration::hours(2)).unwrap();

        // 添加到分析器
        analytics.add_task_data(&task);

        // 验证统计数据
        let today = Local::now().date_naive();
        let stats = analytics.get_daily_stats(today).unwrap();
        assert_eq!(stats.total_time, Duration::hours(2));
        assert_eq!(stats.completed_tasks, 1);
    }

    #[test]
    fn test_weekly_stats() {
        let mut analytics = Analytics::new();
        let category_id = Uuid::new_v4();

        // 添加一周的数据
        for i in 0..7 {
            let mut task = Task::new(format!("任务{}", i), Some(category_id), None);
            task.complete(Duration::hours(1)).unwrap();

            // 手动设置完成时间为不同日期
            let date = NaiveDate::from_ymd_opt(2024, 1, 1 + i).unwrap();
            let mut stats = TimeStats::new(date);
            stats.add_category_time(category_id, Duration::hours(1));
            stats.task_count = 1;
            stats.completed_tasks = 1;
            analytics.historical_stats.insert(date, stats);
        }

        // 获取周统计
        let week_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let weekly_stats = analytics.get_weekly_stats(week_start);

        assert_eq!(weekly_stats.total_time, Duration::hours(7));
        assert_eq!(weekly_stats.completed_tasks, 7);
    }

    #[test]
    fn test_generate_report() {
        let mut analytics = Analytics::new();
        let category_id = Uuid::new_v4();

        // 添加测试数据
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut stats = TimeStats::new(date);
        stats.add_category_time(category_id, Duration::hours(8));
        stats.task_count = 3;
        stats.completed_tasks = 2;
        stats.efficiency_score = 0.8;
        analytics.historical_stats.insert(date, stats);

        // 生成报告
        let report = analytics.generate_report(date, date).unwrap();

        assert_eq!(report.period_start, date);
        assert_eq!(report.period_end, date);
        assert_eq!(report.summary.total_time, Duration::hours(8));
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_category_percentage() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut stats = TimeStats::new(date);

        let category1 = Uuid::new_v4();
        let category2 = Uuid::new_v4();

        stats.add_category_time(category1, Duration::hours(3));
        stats.add_category_time(category2, Duration::hours(1));

        assert_eq!(stats.get_category_percentage(category1), 75.0);
        assert_eq!(stats.get_category_percentage(category2), 25.0);
        assert_eq!(stats.get_most_active_category(), Some(category1));
    }
}
