//! # 仪表板视图
//!
//! TimeTracker的主要仪表板界面，显示当前状态和概览信息

use super::{common, View, ViewConfig, ViewState};
use crate::{
    core::TimerState,
    gui::{gui_utils, theme::ColorType, AppState},
    storage::models::*,
    utils::{current_timestamp, generate_id},
};
use chrono::Local;
use eframe::egui;
use std::time::{Duration, Instant};

/// 仪表板视图
pub struct DashboardView {
    /// 视图状态
    state: ViewState,
    /// 视图配置
    config: ViewConfig,
    /// 当前任务名称
    current_task_name: String,
    /// 当前任务描述
    current_task_description: String,
    /// 选中的分类ID
    selected_category_id: Option<uuid::Uuid>,
    /// 今日统计数据
    today_stats: Option<TimeStats>,
    /// 最近任务列表
    recent_tasks: Vec<TimeEntry>,
    /// 活跃分类统计
    category_stats: Vec<CategoryStats>,
    /// 上次数据刷新时间
    last_refresh: Instant,
    /// 是否显示快速开始对话框
    show_quick_start: bool,
    /// 是否显示今日详情
    show_today_details: bool,
    /// 计时器显示格式
    timer_format: TimerFormat,
}

/// 计时器显示格式
#[derive(Debug, Clone, Copy, PartialEq)]
enum TimerFormat {
    /// HH:MM:SS
    Full,
    /// HH:MM
    Compact,
    /// 数字格式（秒）
    Seconds,
}

impl Default for DashboardView {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardView {
    /// 创建新的仪表板视图
    pub fn new() -> Self {
        Self {
            state: ViewState::Normal,
            config: ViewConfig {
                auto_refresh: true,
                refresh_interval: 5, // 5秒刷新一次
                ..ViewConfig::default()
            },
            current_task_name: String::new(),
            current_task_description: String::new(),
            selected_category_id: None,
            today_stats: None,
            recent_tasks: Vec::new(),
            category_stats: Vec::new(),
            last_refresh: Instant::now(),
            show_quick_start: false,
            show_today_details: false,
            timer_format: TimerFormat::Full,
        }
    }

    /// 刷新仪表板数据
    fn refresh_data(&mut self, state: &mut AppState) {
        self.state = ViewState::Loading;

        // 获取今日统计
        if let Ok(storage) = state.storage.lock() {
            // 检查数据库完整性（定期检查）
            if self.last_refresh.elapsed() > Duration::from_secs(300) {
                if let Err(e) = storage.check_integrity() {
                    log::warn!("数据库完整性检查失败: {}", e);
                }
            }
            let today = Local::now().date_naive();

            // 获取今日时间统计
            match storage.get_daily_stats_range(today, today) {
                Ok(stats) => {
                    if let Some(daily_stat) = stats.first() {
                        // 从DatabaseTimeStats转换为TimeStats
                        let time_stats = crate::storage::models::TimeStats {
                            total_seconds: daily_stat.stats.total_seconds,
                            task_count: daily_stat.stats.task_count,
                            average_seconds: daily_stat.stats.average_seconds,
                            max_seconds: daily_stat.stats.max_seconds,
                            min_seconds: daily_stat.stats.min_seconds,
                            start_date: chrono::Local::now(),
                            end_date: chrono::Local::now(),
                        };
                        self.today_stats = Some(time_stats);
                    }
                }
                Err(e) => {
                    log::error!("获取今日统计失败: {}", e);
                    self.state = ViewState::Error;
                    return;
                }
            }

            // 获取最近任务
            match storage.get_recent_time_entries(10) {
                Ok(entries) => self.recent_tasks = entries,
                Err(e) => {
                    log::error!("获取最近任务失败: {}", e);
                }
            }

            // 获取分类统计
            match storage.get_category_stats(today, today) {
                Ok(stats) => self.category_stats = stats,
                Err(e) => {
                    log::error!("获取分类统计失败: {}", e);
                }
            }
        }

        self.state = ViewState::Normal;
        self.last_refresh = Instant::now();
    }

    /// 开始新任务
    fn start_new_task(&mut self, state: &mut AppState) {
        if self.current_task_name.trim().is_empty() {
            return;
        }

        // 生成新的任务ID
        let task_id = generate_id();
        let start_time = current_timestamp();

        if let Ok(mut core) = state.core.lock() {
            match core.start_task(
                self.current_task_name.clone(),
                self.selected_category_id,
                Some(self.current_task_description.clone()),
            ) {
                Ok(_) => {
                    // 记录任务开始信息
                    log::info!(
                        "开始新任务: {} (ID: {}, 时间: {})",
                        self.current_task_name,
                        task_id,
                        start_time.format("%H:%M:%S")
                    );

                    // 清空输入
                    self.current_task_name.clear();
                    self.current_task_description.clear();
                    self.show_quick_start = false;
                }
                Err(e) => {
                    log::error!("开始任务失败: {}", e);
                }
            }
        }
    }

    /// 暂停当前任务
    fn pause_current_task(&mut self, state: &mut AppState) {
        if let Ok(mut core) = state.core.lock() {
            if let Err(e) = core.pause_current_task() {
                log::error!("暂停任务失败: {}", e);
            }
        }
    }

    /// 恢复当前任务
    fn resume_current_task(&mut self, state: &mut AppState) {
        if let Ok(mut core) = state.core.lock() {
            if let Err(e) = core.resume_current_task() {
                log::error!("恢复任务失败: {}", e);
            }
        }
    }

    /// 停止当前任务
    fn stop_current_task(&mut self, state: &mut AppState) {
        if let Ok(mut core) = state.core.lock() {
            if let Err(e) = core.stop_current_task() {
                log::error!("停止任务失败: {}", e);
            }
        }
    }

    /// 渲染计时器区域
    fn render_timer_section(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(8.0)
            .inner_margin(20.0)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    // 计时器状态和时间显示
                    let (timer_state, current_duration, current_task) =
                        if let Ok(core) = state.core.lock() {
                            let timer_state = core.get_timer_state().clone();
                            let current_duration = core.get_current_duration().num_seconds();
                            let current_task = core.get_current_task().cloned();
                            (timer_state, current_duration, current_task)
                        } else {
                            (TimerState::Stopped, 0, None)
                        };

                    // 状态指示器
                    ui.horizontal(|ui| {
                        let (status_text, status_color) = match &timer_state {
                            TimerState::Stopped => {
                                ("已停止", state.theme.get_color(ColorType::Secondary))
                            }
                            TimerState::Running { .. } => {
                                ("运行中", state.theme.get_color(ColorType::Success))
                            }
                            TimerState::Paused { .. } => {
                                ("已暂停", state.theme.get_color(ColorType::Warning))
                            }
                        };

                        ui.colored_label(status_color, status_text);

                        ui.separator();

                        // 当前时长显示
                        ui.label(format!(
                            "时长: {}",
                            gui_utils::format_duration(current_duration)
                        ));

                        ui.separator();

                        // 当前任务显示
                        if let Some(task) = current_task {
                            ui.label(format!("任务: {}", task.name));
                        } else {
                            ui.label("无当前任务");
                        }
                    });

                    ui.add_space(10.0);

                    // 控制按钮
                    let mut action = None;
                    ui.horizontal(|ui| match &timer_state {
                        TimerState::Stopped => {
                            if ui.button("开始").clicked() {
                                action = Some("start");
                            }
                        }
                        TimerState::Running { .. } => {
                            if ui.button("暂停").clicked() {
                                action = Some("pause");
                            }
                        }
                        TimerState::Paused { .. } => {
                            if ui.button("继续").clicked() {
                                action = Some("resume");
                            }
                            if ui.button("停止").clicked() {
                                action = Some("stop");
                            }
                        }
                    });

                    // 在UI闭包外执行动作
                    match action {
                        Some("start") => self.start_new_task(state),
                        Some("pause") => self.pause_current_task(state),
                        Some("resume") => self.resume_current_task(state),
                        Some("stop") => self.stop_current_task(state),
                        _ => {}
                    }
                });
            });
    }

    /// 渲染今日统计
    fn render_today_stats(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.heading("今日统计");
        ui.separator();

        if let Some(stats) = &self.today_stats {
            // 统计卡片
            ui.horizontal(|ui| {
                // 总时长
                common::render_stat_card(
                    ui,
                    "总时长",
                    &gui_utils::format_duration(stats.total_seconds),
                    "⏱",
                    state.theme.get_color(ColorType::Primary),
                );

                ui.add_space(10.0);

                // 任务数量
                common::render_stat_card(
                    ui,
                    "任务数量",
                    &stats.task_count.to_string(),
                    "📝",
                    state.theme.get_color(ColorType::Info),
                );

                ui.add_space(10.0);

                // 平均时长
                common::render_stat_card(
                    ui,
                    "平均时长",
                    &gui_utils::format_duration(stats.average_seconds as i64),
                    "📊",
                    state.theme.get_color(ColorType::Success),
                );
            });

            ui.add_space(10.0);

            // 详细信息按钮
            if ui
                .button(if self.show_today_details {
                    "隐藏详情"
                } else {
                    "显示详情"
                })
                .clicked()
            {
                self.show_today_details = !self.show_today_details;
            }

            if self.show_today_details {
                ui.separator();

                egui::Grid::new("today_details")
                    .num_columns(2)
                    .spacing([10.0, 5.0])
                    .show(ui, |ui| {
                        ui.label("最长任务:");
                        ui.label(gui_utils::format_duration(stats.max_seconds));
                        ui.end_row();

                        ui.label("最短任务:");
                        ui.label(gui_utils::format_duration(stats.min_seconds));
                        ui.end_row();

                        ui.label("统计时间:");
                        ui.label(stats.start_date.format("%Y-%m-%d").to_string());
                        ui.end_row();
                    });
            }
        } else {
            common::render_empty(ui, "暂无今日数据", Some("开始第一个任务"));
        }
    }

    /// 渲染最近任务
    fn render_recent_tasks(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.heading("最近任务");
        ui.separator();

        if self.recent_tasks.is_empty() {
            common::render_empty(ui, "暂无最近任务", None);
            return;
        }

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for (index, entry) in self.recent_tasks.iter().enumerate() {
                    if index > 0 {
                        ui.separator();
                    }

                    ui.horizontal(|ui| {
                        // 状态指示器
                        let status_color = if entry.is_running() {
                            state.theme.get_color(ColorType::Success)
                        } else {
                            ui.visuals().weak_text_color()
                        };

                        ui.colored_label(status_color, "●");

                        // 任务信息
                        ui.vertical(|ui| {
                            ui.strong(&entry.task_name);

                            ui.horizontal(|ui| {
                                ui.label(entry.start_time.format("%H:%M").to_string());
                                ui.label("|");
                                ui.label(gui_utils::format_duration(entry.duration_seconds));

                                if !entry.tags.is_empty() {
                                    ui.label("|");
                                    ui.colored_label(
                                        state.theme.get_color(ColorType::Info),
                                        entry.tags.join(", "),
                                    );
                                }
                            });
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("📋").on_hover_text("复制任务").clicked() {
                                self.current_task_name = entry.task_name.clone();
                                self.current_task_description =
                                    entry.description.clone().unwrap_or_default();
                                self.show_quick_start = true;
                            }
                        });
                    });
                }
            });
    }

    /// 渲染分类统计
    fn render_category_stats(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.heading("分类统计");
        ui.separator();

        if self.category_stats.is_empty() {
            common::render_empty(ui, "暂无分类数据", None);
            return;
        }

        // 计算总时间
        let total_time: i64 = self.category_stats.iter().map(|s| s.total_seconds).sum();

        if total_time > 0 {
            for stat in &self.category_stats {
                let percentage = (stat.total_seconds as f32 / total_time as f32) * 100.0;
                if percentage > 5.0 {
                    // 只显示占比超过5%的分类
                    ui.horizontal(|ui| {
                        ui.label(&stat.category_name);
                        ui.label(gui_utils::format_duration(stat.total_seconds));
                        ui.label(format!("{:.1}%", percentage));
                    });
                }
            }
        }
    }

    /// 渲染快速开始对话框
    fn render_quick_start_dialog(&mut self, ctx: &egui::Context, state: &mut AppState) {
        if !self.show_quick_start {
            return;
        }

        egui::Window::new("开始新任务")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // 任务名称
                    ui.label("任务名称:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.current_task_name)
                            .hint_text("输入任务名称...")
                            .desired_width(300.0),
                    );

                    ui.add_space(10.0);

                    // 任务描述
                    ui.label("任务描述 (可选):");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.current_task_description)
                            .hint_text("输入任务描述...")
                            .desired_width(300.0)
                            .desired_rows(3),
                    );

                    ui.add_space(10.0);

                    // 分类选择
                    ui.label("分类:");
                    egui::ComboBox::from_label("")
                        .selected_text("选择分类")
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_category_id, None, "无分类");
                            // TODO: 加载分类列表
                        });

                    ui.add_space(20.0);

                    // 按钮
                    ui.horizontal(|ui| {
                        if ui.button("开始计时").clicked() {
                            self.start_new_task(state);
                        }

                        if ui.button("取消").clicked() {
                            self.show_quick_start = false;
                        }
                    });
                });
            });
    }
}

impl View for DashboardView {
    fn render(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        // 检查是否需要刷新数据
        if self.config.auto_refresh
            && self.last_refresh.elapsed() >= Duration::from_secs(self.config.refresh_interval)
        {
            self.refresh_data(state);
        }

        match self.state {
            ViewState::Loading => {
                common::render_loading(ui, "加载仪表板数据...");
                return;
            }
            ViewState::Error => {
                common::render_error(ui, "加载数据失败", &state.theme);
                return;
            }
            _ => {}
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            // 计时器区域
            self.render_timer_section(ui, state);

            ui.add_space(20.0);

            // 统计信息区域
            ui.columns(2, |columns| {
                // 左列：今日统计和最近任务
                columns[0].vertical(|ui| {
                    self.render_today_stats(ui, state);

                    ui.add_space(20.0);

                    self.render_recent_tasks(ui, state);
                });

                // 右列：分类统计
                columns[1].vertical(|ui| {
                    self.render_category_stats(ui, state);
                });
            });
        });

        // 渲染快速开始对话框
        self.render_quick_start_dialog(ui.ctx(), state);
    }

    fn title(&self) -> &str {
        "仪表板"
    }

    fn needs_refresh(&self) -> bool {
        self.config.auto_refresh
            && self.last_refresh.elapsed() >= Duration::from_secs(self.config.refresh_interval)
    }

    fn refresh(&mut self, state: &mut AppState) {
        self.refresh_data(state);
    }

    fn handle_shortcut(&mut self, ctx: &egui::Context, state: &mut AppState) -> bool {
        // 处理快捷键
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            // 空格键：开始/暂停/继续
            let timer_state = if let Ok(core) = state.core.lock() {
                core.get_timer_state().clone()
            } else {
                TimerState::Stopped
            };

            match timer_state {
                TimerState::Running { .. } => self.pause_current_task(state),
                TimerState::Paused { .. } => self.resume_current_task(state),
                TimerState::Stopped => {
                    // 已停止状态不需要处理
                }
            }
            return true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            // ESC键：停止计时或关闭对话框
            if self.show_quick_start {
                self.show_quick_start = false;
            } else {
                self.stop_current_task(state);
            }
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
    fn test_dashboard_view_creation() {
        let view = DashboardView::new();
        assert_eq!(view.title(), "仪表板");
        assert_eq!(view.state, ViewState::Normal);
        assert!(view.config.auto_refresh);
        assert_eq!(view.config.refresh_interval, 5);
    }

    #[test]
    fn test_timer_format() {
        assert_eq!(TimerFormat::Full, TimerFormat::Full);
        assert_ne!(TimerFormat::Full, TimerFormat::Compact);
    }
}
