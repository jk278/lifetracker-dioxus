//! # 统计视图
//!
//! TimeTracker的统计分析界面，用于查看时间跟踪的各种统计数据和图表

use super::{common, View, ViewConfig, ViewState};
use crate::{
    core::{analytics::TrendAnalysis, AnalyticsReport},
    gui::{gui_utils, theme::ColorType, AppState},
    storage::models::*,
};
use chrono::{Datelike, Duration as ChronoDuration, Local, NaiveDate};
use eframe::egui;
use std::time::{Duration, Instant};

/// 统计视图
pub struct StatisticsView {
    /// 视图状态
    state: ViewState,
    /// 视图配置
    config: ViewConfig,
    /// 统计时间范围
    date_range: DateRange,
    /// 统计类型
    stats_type: StatsType,
    /// 图表类型
    chart_type: ChartType,
    /// 每日统计数据
    daily_stats: Vec<DailyStats>,
    /// 每周统计数据
    weekly_stats: Vec<WeeklyStats>,
    /// 每月统计数据
    monthly_stats: Vec<MonthlyStats>,
    /// 分类统计数据
    category_stats: Vec<CategoryStats>,
    /// 分析报告
    analytics_report: Option<AnalyticsReport>,
    /// 趋势分析
    trend_analysis: Option<TrendAnalysis>,
    /// 上次数据刷新时间
    last_refresh: Instant,
    /// 是否显示详细信息
    show_details: bool,
    /// 选中的日期
    selected_date: Option<NaiveDate>,
    /// 自定义日期范围
    custom_start_date: NaiveDate,
    custom_end_date: NaiveDate,
    /// 是否显示自定义日期选择器
    show_date_picker: bool,
}

/// 统计时间范围
#[derive(Debug, Clone, Copy, PartialEq)]
enum DateRange {
    /// 今天
    Today,
    /// 本周
    ThisWeek,
    /// 本月
    ThisMonth,
    /// 最近7天
    Last7Days,
    /// 最近30天
    Last30Days,
    /// 最近90天
    Last90Days,
    /// 自定义范围
    Custom,
}

/// 统计类型
#[derive(Debug, Clone, Copy, PartialEq)]
enum StatsType {
    /// 概览
    Overview,
    /// 每日统计
    Daily,
    /// 每周统计
    Weekly,
    /// 每月统计
    Monthly,
    /// 分类统计
    Category,
    /// 趋势分析
    Trend,
}

/// 图表类型
#[derive(Debug, Clone, Copy, PartialEq)]
enum ChartType {
    /// 柱状图
    Bar,
    /// 折线图
    Line,
    /// 饼图
    Pie,
    /// 面积图
    Area,
}

impl Default for StatisticsView {
    fn default() -> Self {
        Self::new()
    }
}

impl StatisticsView {
    /// 创建新的统计视图
    pub fn new() -> Self {
        let today = Local::now().date_naive();

        Self {
            state: ViewState::Normal,
            config: ViewConfig {
                auto_refresh: true,
                refresh_interval: 30, // 30秒刷新一次
                ..ViewConfig::default()
            },
            date_range: DateRange::Last7Days,
            stats_type: StatsType::Overview,
            chart_type: ChartType::Bar,
            daily_stats: Vec::new(),
            weekly_stats: Vec::new(),
            monthly_stats: Vec::new(),
            category_stats: Vec::new(),
            analytics_report: None,
            trend_analysis: None,
            last_refresh: Instant::now(),
            show_details: false,
            selected_date: None,
            custom_start_date: today - ChronoDuration::days(7),
            custom_end_date: today,
            show_date_picker: false,
        }
    }

    /// 获取日期范围
    fn get_date_range(&self) -> (NaiveDate, NaiveDate) {
        let today = Local::now().date_naive();

        match self.date_range {
            DateRange::Today => (today, today),
            DateRange::ThisWeek => {
                let days_since_monday = today.weekday().num_days_from_monday();
                let week_start = today - ChronoDuration::days(days_since_monday as i64);
                (week_start, today)
            }
            DateRange::ThisMonth => {
                let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
                (month_start, today)
            }
            DateRange::Last7Days => (today - ChronoDuration::days(6), today),
            DateRange::Last30Days => (today - ChronoDuration::days(29), today),
            DateRange::Last90Days => (today - ChronoDuration::days(89), today),
            DateRange::Custom => (self.custom_start_date, self.custom_end_date),
        }
    }

    /// 刷新统计数据
    fn refresh_data(&mut self, state: &mut AppState) {
        self.state = ViewState::Loading;

        let (start_date, end_date) = self.get_date_range();

        if let Ok(storage) = state.storage.lock() {
            // 获取每日统计
            match storage.get_daily_stats_range(start_date, end_date) {
                Ok(stats) => self.daily_stats = stats,
                Err(e) => {
                    log::error!("获取每日统计失败: {}", e);
                    self.state = ViewState::Error;
                    return;
                }
            }

            // 获取每周统计
            match storage.get_weekly_stats_range(start_date, end_date) {
                Ok(stats) => self.weekly_stats = stats,
                Err(e) => {
                    log::error!("获取每周统计失败: {}", e);
                }
            }

            // 获取每月统计
            match storage.get_monthly_stats_range(start_date, end_date) {
                Ok(stats) => self.monthly_stats = stats,
                Err(e) => {
                    log::error!("获取每月统计失败: {}", e);
                }
            }

            // 获取分类统计
            match storage.get_category_stats(start_date, end_date) {
                Ok(stats) => self.category_stats = stats,
                Err(e) => {
                    log::error!("获取分类统计失败: {}", e);
                }
            }
        }

        // 生成分析报告
        if let Ok(core) = state.core.lock() {
            match core.generate_analytics_report(start_date, end_date) {
                Ok(report) => self.analytics_report = Some(report),
                Err(e) => {
                    log::error!("生成分析报告失败: {}", e);
                }
            }

            // 生成趋势分析
            match core.analyze_trends(start_date, end_date) {
                Ok(analysis) => self.trend_analysis = Some(analysis),
                Err(e) => {
                    log::error!("生成趋势分析失败: {}", e);
                }
            }
        }

        self.state = ViewState::Normal;
        self.last_refresh = Instant::now();
    }

    /// 渲染工具栏
    fn render_toolbar(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.horizontal(|ui| {
            // 时间范围选择
            ui.label("时间范围:");
            egui::ComboBox::from_id_source("date_range")
                .selected_text(match self.date_range {
                    DateRange::Today => "今天",
                    DateRange::ThisWeek => "本周",
                    DateRange::ThisMonth => "本月",
                    DateRange::Last7Days => "最近7天",
                    DateRange::Last30Days => "最近30天",
                    DateRange::Last90Days => "最近90天",
                    DateRange::Custom => "自定义",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.date_range, DateRange::Today, "今天");
                    ui.selectable_value(&mut self.date_range, DateRange::ThisWeek, "本周");
                    ui.selectable_value(&mut self.date_range, DateRange::ThisMonth, "本月");
                    ui.selectable_value(&mut self.date_range, DateRange::Last7Days, "最近7天");
                    ui.selectable_value(&mut self.date_range, DateRange::Last30Days, "最近30天");
                    ui.selectable_value(&mut self.date_range, DateRange::Last90Days, "最近90天");
                    ui.selectable_value(&mut self.date_range, DateRange::Custom, "自定义");
                });

            // 自定义日期范围按钮
            if self.date_range == DateRange::Custom && ui.button("📅 选择日期").clicked() {
                self.show_date_picker = true;
            }

            ui.separator();

            // 统计类型选择
            ui.label("统计类型:");
            egui::ComboBox::from_id_source("stats_type")
                .selected_text(match self.stats_type {
                    StatsType::Overview => "概览",
                    StatsType::Daily => "每日",
                    StatsType::Weekly => "每周",
                    StatsType::Monthly => "每月",
                    StatsType::Category => "分类",
                    StatsType::Trend => "趋势",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.stats_type, StatsType::Overview, "概览");
                    ui.selectable_value(&mut self.stats_type, StatsType::Daily, "每日");
                    ui.selectable_value(&mut self.stats_type, StatsType::Weekly, "每周");
                    ui.selectable_value(&mut self.stats_type, StatsType::Monthly, "每月");
                    ui.selectable_value(&mut self.stats_type, StatsType::Category, "分类");
                    ui.selectable_value(&mut self.stats_type, StatsType::Trend, "趋势");
                });

            ui.separator();

            // 图表类型选择
            ui.label("图表类型:");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.chart_type, ChartType::Bar, "📊");
                ui.selectable_value(&mut self.chart_type, ChartType::Line, "📈");
                ui.selectable_value(&mut self.chart_type, ChartType::Pie, "🥧");
                ui.selectable_value(&mut self.chart_type, ChartType::Area, "📉");
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 刷新按钮
                if ui.button("🔄").on_hover_text("刷新").clicked() {
                    self.refresh_data(state);
                }

                // 详细信息切换
                ui.checkbox(&mut self.show_details, "详细信息");
            });
        });
    }

    /// 渲染概览统计
    fn render_overview(&mut self, ui: &mut egui::Ui, state: &AppState) {
        let (start_date, end_date) = self.get_date_range();

        ui.heading("统计概览");
        ui.separator();

        // 计算总体统计
        let total_time: i64 = self.daily_stats.iter().map(|s| s.stats.total_seconds).sum();
        let total_tasks: usize = self
            .daily_stats
            .iter()
            .map(|s| s.stats.task_count as usize)
            .sum();
        let avg_daily_time = if !self.daily_stats.is_empty() {
            total_time / self.daily_stats.len() as i64
        } else {
            0
        };

        // 统计卡片
        ui.horizontal(|ui| {
            common::render_stat_card(
                ui,
                "总时长",
                &gui_utils::format_duration(total_time),
                "⏱",
                state.theme.get_color(ColorType::Primary),
            );

            ui.add_space(10.0);

            common::render_stat_card(
                ui,
                "总任务数",
                &total_tasks.to_string(),
                "📝",
                state.theme.get_color(ColorType::Info),
            );

            ui.add_space(10.0);

            common::render_stat_card(
                ui,
                "日均时长",
                &gui_utils::format_duration(avg_daily_time),
                "📊",
                state.theme.get_color(ColorType::Success),
            );

            ui.add_space(10.0);

            common::render_stat_card(
                ui,
                "活跃天数",
                &self
                    .daily_stats
                    .iter()
                    .filter(|s| s.stats.total_seconds > 0)
                    .count()
                    .to_string(),
                "📅",
                state.theme.get_color(ColorType::Warning),
            );
        });

        ui.add_space(20.0);

        // 分析报告
        if let Some(report) = &self.analytics_report {
            ui.heading("分析报告");
            ui.separator();

            ui.label(format!(
                "统计期间: {} 至 {}",
                start_date.format("%Y-%m-%d"),
                end_date.format("%Y-%m-%d")
            ));

            ui.add_space(10.0);

            egui::Grid::new("analytics_grid")
                .num_columns(2)
                .spacing([20.0, 5.0])
                .show(ui, |ui| {
                    ui.label("最高效日期:");
                    if let Some(best_day) = report
                        .daily_stats
                        .iter()
                        .max_by_key(|stats| stats.efficiency_score as i32)
                    {
                        ui.label(best_day.date.format("%Y-%m-%d").to_string());
                    } else {
                        ui.label("无数据");
                    }
                    ui.end_row();

                    ui.label("高峰时段:");
                    if !report.trends.peak_hours.is_empty() {
                        let peak_start = report.trends.peak_hours[0];
                        let peak_end = peak_start + 1;
                        ui.label(format!("{}:00-{}:00", peak_start, peak_end));
                    } else {
                        ui.label("无数据");
                    }
                    ui.end_row();

                    ui.label("最活跃分类:");
                    if let Some(most_active_category) = report.summary.get_most_active_category() {
                        ui.label(format!("{:?}", most_active_category)); // 临时显示UUID
                    } else {
                        ui.label("无数据");
                    }
                    ui.end_row();

                    ui.label("平均任务时长:");
                    ui.label(gui_utils::format_duration(
                        report.summary.average_session.num_seconds(),
                    ));
                    ui.end_row();

                    ui.label("完成率:");
                    let completion_rate = if report.summary.task_count > 0 {
                        report.summary.completed_tasks as f32 / report.summary.task_count as f32
                    } else {
                        0.0
                    };
                    ui.label(format!("{:.1}%", completion_rate * 100.0));
                    ui.end_row();
                });

            if self.show_details && !report.recommendations.is_empty() {
                ui.separator();
                ui.heading("建议");

                for suggestion in &report.recommendations {
                    ui.horizontal(|ui| {
                        ui.label("💡");
                        ui.label(suggestion);
                    });
                }
            }
        }
    }

    /// 渲染每日统计
    fn render_daily_stats(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.heading("每日统计");
        ui.separator();

        if self.daily_stats.is_empty() {
            common::render_empty(ui, "暂无每日统计数据", None);
            return;
        }

        // 图表区域
        self.render_daily_chart(ui, state);

        ui.add_space(20.0);

        // 详细数据表格
        if self.show_details {
            ui.heading("详细数据");
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    egui::Grid::new("daily_stats_grid")
                        .num_columns(4)
                        .spacing([10.0, 5.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // 表头
                            ui.strong("日期");
                            ui.strong("总时长");
                            ui.strong("任务数");
                            ui.strong("平均时长");
                            ui.end_row();

                            // 数据行
                            for stat in &self.daily_stats {
                                let is_selected = self.selected_date == Some(stat.date);

                                if ui
                                    .selectable_label(
                                        is_selected,
                                        stat.date.format("%m-%d").to_string(),
                                    )
                                    .clicked()
                                {
                                    self.selected_date =
                                        if is_selected { None } else { Some(stat.date) };
                                }

                                ui.label(gui_utils::format_duration(stat.stats.total_seconds));
                                ui.label(stat.stats.task_count.to_string());
                                ui.label(gui_utils::format_duration(
                                    stat.stats.average_seconds as i64,
                                ));
                                ui.end_row();
                            }
                        });
                });
        }
    }

    /// 渲染每日图表
    fn render_daily_chart(&self, ui: &mut egui::Ui, state: &AppState) {
        let chart_height = 200.0;
        let chart_width = ui.available_width();

        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(chart_width, chart_height),
            egui::Sense::hover(),
        );

        let chart_rect = response.rect;
        let margin = 40.0;
        let plot_rect = egui::Rect::from_min_size(
            chart_rect.min + egui::Vec2::new(margin, margin),
            chart_rect.size() - egui::Vec2::new(margin * 2.0, margin * 2.0),
        );

        if self.daily_stats.is_empty() {
            return;
        }

        // 绘制背景
        painter.rect_filled(chart_rect, 4.0, ui.visuals().extreme_bg_color);

        // 计算数据范围
        let max_time = self
            .daily_stats
            .iter()
            .map(|s| s.stats.total_seconds)
            .max()
            .unwrap_or(1) as f32;

        let data_count = self.daily_stats.len();
        let bar_width = plot_rect.width() / data_count as f32 * 0.8;
        let bar_spacing = plot_rect.width() / data_count as f32;

        match self.chart_type {
            ChartType::Bar => {
                // 绘制柱状图
                for (i, stat) in self.daily_stats.iter().enumerate() {
                    let x = plot_rect.min.x + i as f32 * bar_spacing + bar_spacing * 0.1;
                    let height = (stat.stats.total_seconds as f32 / max_time) * plot_rect.height();
                    let y = plot_rect.max.y - height;

                    let bar_rect = egui::Rect::from_min_size(
                        egui::Pos2::new(x, y),
                        egui::Vec2::new(bar_width, height),
                    );

                    painter.rect_filled(bar_rect, 2.0, state.theme.get_color(ColorType::Primary));

                    // 绘制日期标签
                    painter.text(
                        egui::Pos2::new(x + bar_width / 2.0, plot_rect.max.y + 10.0),
                        egui::Align2::CENTER_TOP,
                        stat.date.format("%m-%d").to_string(),
                        egui::FontId::proportional(10.0),
                        ui.visuals().text_color(),
                    );
                }
            }
            ChartType::Line => {
                // 绘制折线图
                let points: Vec<egui::Pos2> = self
                    .daily_stats
                    .iter()
                    .enumerate()
                    .map(|(i, stat)| {
                        let x = plot_rect.min.x + i as f32 * bar_spacing + bar_spacing / 2.0;
                        let height =
                            (stat.stats.total_seconds as f32 / max_time) * plot_rect.height();
                        let y = plot_rect.max.y - height;
                        egui::Pos2::new(x, y)
                    })
                    .collect();

                // 绘制线条
                for window in points.windows(2) {
                    painter.line_segment(
                        [window[0], window[1]],
                        egui::Stroke::new(2.0, state.theme.get_color(ColorType::Primary)),
                    );
                }

                // 绘制点
                for point in points {
                    painter.circle_filled(point, 4.0, state.theme.get_color(ColorType::Primary));
                }
            }
            _ => {
                // 其他图表类型暂时使用柱状图
                // TODO: 实现饼图和面积图
            }
        }

        // 绘制坐标轴
        painter.line_segment(
            [plot_rect.left_bottom(), plot_rect.right_bottom()],
            egui::Stroke::new(1.0, ui.visuals().text_color()),
        );
        painter.line_segment(
            [plot_rect.left_bottom(), plot_rect.left_top()],
            egui::Stroke::new(1.0, ui.visuals().text_color()),
        );
    }

    /// 渲染分类统计
    fn render_category_stats(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.heading("分类统计");
        ui.separator();

        if self.category_stats.is_empty() {
            common::render_empty(ui, "暂无分类统计数据", None);
            return;
        }

        // 计算总时间
        let total_time: i64 = self.category_stats.iter().map(|s| s.total_seconds).sum();

        if total_time > 0 {
            for stat in &self.category_stats {
                let percentage = (stat.total_seconds as f32 / total_time as f32) * 100.0;

                ui.horizontal(|ui| {
                    // 分类名称
                    ui.strong(&stat.category_name);

                    // 进度条
                    let progress_bar = egui::ProgressBar::new(percentage / 100.0)
                        .text(format!("{:.1}%", percentage))
                        .fill(state.theme.get_color(ColorType::Primary));
                    ui.add_sized([200.0, 20.0], progress_bar);

                    // 统计信息
                    ui.label(format!(
                        "{} | {} 任务",
                        gui_utils::format_duration(stat.total_seconds),
                        stat.task_count
                    ));
                });

                if self.show_details {
                    ui.indent("category_details", |ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "平均时长: {}",
                                gui_utils::format_duration(stat.average_seconds as i64)
                            ));
                            ui.label("|");
                            ui.label(format!("任务数: {}", stat.task_count));
                        });
                    });
                }

                ui.add_space(5.0);
            }
        }
    }

    /// 渲染趋势分析
    fn render_trend_analysis(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.heading("趋势分析");
        ui.separator();

        if let Some(analysis) = self.trend_analysis.as_ref() {
            let trend_color = if analysis.time_trend > 0.0 {
                state.theme.get_color(ColorType::Success)
            } else if analysis.time_trend < 0.0 {
                state.theme.get_color(ColorType::Warning)
            } else {
                state.theme.get_color(ColorType::Secondary)
            };

            ui.horizontal(|ui| {
                ui.colored_label(trend_color, format!("{:+.1}%", analysis.time_trend * 100.0));
                ui.label("时间趋势");
            });

            ui.label(format!(
                "效率趋势: {:+.1}%",
                analysis.efficiency_trend * 100.0
            ));

            ui.label(format!(
                "预测下周时长: {}",
                gui_utils::format_duration(
                    (analysis.time_trend * 40.0 * 3600.0) as i64 // 假设基于趋势预测
                ),
            ));

            // 显示高峰时间段
            if !analysis.peak_hours.is_empty() {
                ui.label("高峰时段:");
                for hour in &analysis.peak_hours {
                    ui.label(format!("{}:00", hour));
                }
            }
        } else {
            common::render_empty(ui, "暂无趋势分析数据", None);
        }
    }

    /// 渲染日期选择器对话框
    fn render_date_picker(&mut self, ctx: &egui::Context, state: &mut AppState) {
        if !self.show_date_picker {
            return;
        }

        egui::Window::new("选择日期范围")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("开始日期:");
                    // TODO: 实现日期选择器组件
                    ui.label(self.custom_start_date.format("%Y-%m-%d").to_string());
                });

                ui.horizontal(|ui| {
                    ui.label("结束日期:");
                    // TODO: 实现日期选择器组件
                    ui.label(self.custom_end_date.format("%Y-%m-%d").to_string());
                });

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    if ui.button("确定").clicked() {
                        self.show_date_picker = false;
                        self.refresh_data(state);
                    }

                    if ui.button("取消").clicked() {
                        self.show_date_picker = false;
                    }
                });
            });
    }
}

impl View for StatisticsView {
    fn render(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        // 检查是否需要刷新数据
        if self.config.auto_refresh
            && self.last_refresh.elapsed() >= Duration::from_secs(self.config.refresh_interval)
        {
            self.refresh_data(state);
        }

        match self.state {
            ViewState::Loading => {
                common::render_loading(ui, "加载统计数据...");
                return;
            }
            ViewState::Error => {
                common::render_error(ui, "加载数据失败", &state.theme);
                return;
            }
            _ => {}
        }

        ui.vertical(|ui| {
            // 工具栏
            self.render_toolbar(ui, state);

            ui.add_space(10.0);

            // 根据统计类型渲染不同内容
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.stats_type {
                    StatsType::Overview => self.render_overview(ui, state),
                    StatsType::Daily => self.render_daily_stats(ui, state),
                    StatsType::Weekly => {
                        // TODO: 实现每周统计
                        ui.label("每周统计功能开发中...");
                    }
                    StatsType::Monthly => {
                        // TODO: 实现每月统计
                        ui.label("每月统计功能开发中...");
                    }
                    StatsType::Category => self.render_category_stats(ui, state),
                    StatsType::Trend => self.render_trend_analysis(ui, state),
                }
            });
        });

        // 渲染日期选择器
        self.render_date_picker(ui.ctx(), state);
    }

    fn title(&self) -> &str {
        "统计分析"
    }

    fn needs_refresh(&self) -> bool {
        self.config.auto_refresh
            && self.last_refresh.elapsed() >= Duration::from_secs(self.config.refresh_interval)
    }

    fn refresh(&mut self, state: &mut AppState) {
        self.refresh_data(state);
    }

    fn handle_shortcut(&mut self, ctx: &egui::Context, state: &mut AppState) -> bool {
        // F5: 刷新
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.refresh_data(state);
            return true;
        }

        // 1-6: 切换统计类型
        if ctx.input(|i| i.key_pressed(egui::Key::Num1)) {
            self.stats_type = StatsType::Overview;
            return true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num2)) {
            self.stats_type = StatsType::Daily;
            return true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num3)) {
            self.stats_type = StatsType::Weekly;
            return true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num4)) {
            self.stats_type = StatsType::Monthly;
            return true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num5)) {
            self.stats_type = StatsType::Category;
            return true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num6)) {
            self.stats_type = StatsType::Trend;
            return true;
        }

        // D: 切换详细信息
        if ctx.input(|i| i.key_pressed(egui::Key::D)) {
            self.show_details = !self.show_details;
            return true;
        }

        false
    }

    fn initialize(&mut self, state: &mut AppState) {
        self.refresh_data(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_view_creation() {
        let view = StatisticsView::new();
        assert_eq!(view.title(), "统计分析");
        assert_eq!(view.state, ViewState::Normal);
        assert!(view.config.auto_refresh);
        assert_eq!(view.config.refresh_interval, 30);
        assert_eq!(view.date_range, DateRange::Last7Days);
        assert_eq!(view.stats_type, StatsType::Overview);
    }

    #[test]
    fn test_date_range() {
        assert_eq!(DateRange::Today, DateRange::Today);
        assert_ne!(DateRange::Today, DateRange::ThisWeek);
    }

    #[test]
    fn test_stats_type() {
        assert_eq!(StatsType::Overview, StatsType::Overview);
        assert_ne!(StatsType::Overview, StatsType::Daily);
    }
}
