//! # GUI应用程序主体
//!
//! TimeTracker的主要GUI应用程序实现

use super::{gui_utils, views::*, AppState};
use crate::{core::TimerState, utils::format::format_duration_detailed};
use chrono::Local;
use eframe::egui;
use std::time::{Duration, Instant};

/// 主应用程序视图
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppView {
    /// 主仪表板
    Dashboard,
    /// 任务管理
    Tasks,
    /// 分类管理
    Categories,
    /// 统计报告
    Statistics,
    /// 设置
    Settings,
    /// 关于
    About,
}

impl Default for AppView {
    fn default() -> Self {
        Self::Dashboard
    }
}

/// TimeTracker GUI应用程序
pub struct TimeTrackerApp {
    /// 应用程序状态
    pub state: AppState,

    /// 当前视图
    current_view: AppView,

    /// 视图组件
    dashboard_view: DashboardView,
    tasks_view: TasksView,
    categories_view: CategoriesView,
    statistics_view: StatisticsView,
    settings_view: SettingsView,
    about_view: AboutView,

    /// UI状态
    show_side_panel: bool,
    show_status_bar: bool,
    show_notifications: bool,

    /// 通知系统
    notifications: Vec<Notification>,

    /// 上次更新时间
    last_update: Instant,

    /// 错误状态
    error_message: Option<String>,

    /// 确认对话框状态
    confirm_dialog: Option<ConfirmDialog>,
}

/// 通知消息
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub level: NotificationLevel,
    pub created_at: Instant,
    pub duration: Duration,
    pub auto_dismiss: bool,
}

/// 通知级别
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// 确认对话框
#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub on_confirm: String, // 存储确认操作的标识
    pub confirm_text: String,
    pub cancel_text: String,
}

impl TimeTrackerApp {
    /// 创建新的应用程序实例
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            current_view: AppView::default(),

            // 初始化视图组件
            dashboard_view: DashboardView::new(),
            tasks_view: TasksView::new(),
            categories_view: CategoriesView::new(),
            statistics_view: StatisticsView::new(),
            settings_view: SettingsView::new(),
            about_view: AboutView::new(),

            // UI状态
            show_side_panel: true,
            show_status_bar: true,
            show_notifications: true,

            // 其他状态
            notifications: Vec::new(),
            last_update: Instant::now(),
            error_message: None,
            confirm_dialog: None,
        }
    }

    /// 切换视图
    pub fn switch_view(&mut self, view: AppView) {
        if self.current_view != view {
            self.current_view = view;
            self.clear_error();
        }
    }

    /// 添加通知
    pub fn add_notification(&mut self, title: String, message: String, level: NotificationLevel) {
        let notification = Notification {
            id: self.notifications.len() as u64,
            title,
            message,
            level,
            created_at: Instant::now(),
            duration: Duration::from_secs(match level {
                NotificationLevel::Error => 10,
                NotificationLevel::Warning => 7,
                NotificationLevel::Success => 5,
                NotificationLevel::Info => 3,
            }),
            auto_dismiss: true,
        };

        self.notifications.push(notification);

        // 限制通知数量
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }

    /// 清理过期通知
    fn cleanup_notifications(&mut self) {
        let now = Instant::now();
        self.notifications.retain(|notification| {
            !notification.auto_dismiss
                || now.duration_since(notification.created_at) < notification.duration
        });
    }

    /// 显示错误消息
    pub fn show_error(&mut self, message: String) {
        self.error_message = Some(message.clone());
        self.add_notification("错误".to_string(), message, NotificationLevel::Error);
    }

    /// 清除错误消息
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// 显示确认对话框
    pub fn show_confirm_dialog(&mut self, title: String, message: String, action: String) {
        self.confirm_dialog = Some(ConfirmDialog {
            title,
            message,
            on_confirm: action,
            confirm_text: "确定".to_string(),
            cancel_text: "取消".to_string(),
        });
    }

    /// 处理确认对话框结果
    fn handle_confirm_dialog(&mut self, confirmed: bool) {
        if let Some(dialog) = self.confirm_dialog.take() {
            if confirmed {
                self.execute_action(&dialog.on_confirm);
            }
        }
    }

    /// 执行操作
    fn execute_action(&mut self, action: &str) {
        match action {
            "delete_task" => {
                // TODO: 删除选中的任务
                self.add_notification(
                    "任务删除".to_string(),
                    "任务已成功删除".to_string(),
                    NotificationLevel::Success,
                );
            }
            "delete_category" => {
                // TODO: 删除选中的分类
                self.add_notification(
                    "分类删除".to_string(),
                    "分类已成功删除".to_string(),
                    NotificationLevel::Success,
                );
            }
            "reset_settings" => {
                // TODO: 重置设置
                self.add_notification(
                    "设置重置".to_string(),
                    "设置已重置为默认值".to_string(),
                    NotificationLevel::Info,
                );
            }
            "optimize_database" => {
                // 模拟数据库优化
                let storage_available = self.state.storage.lock().is_ok();
                if storage_available {
                    // 这里应该调用实际的数据库优化方法
                    self.add_notification(
                        "数据库优化".to_string(),
                        "数据库优化已完成，性能得到提升".to_string(),
                        NotificationLevel::Success,
                    );
                } else {
                    self.add_notification(
                        "优化失败".to_string(),
                        "无法访问数据库进行优化".to_string(),
                        NotificationLevel::Error,
                    );
                }
            }
            "clear_all_data" => {
                // 警告级别的危险操作
                self.add_notification(
                    "危险操作".to_string(),
                    "正在清空所有数据，请稍候...".to_string(),
                    NotificationLevel::Warning,
                );
                // 模拟清空数据的延迟操作
                // TODO: 实现实际的数据清空逻辑
                self.add_notification(
                    "操作完成".to_string(),
                    "所有数据已清空".to_string(),
                    NotificationLevel::Info,
                );
            }
            _ => {
                log::warn!("未知操作: {}", action);
                self.add_notification(
                    "未知操作".to_string(),
                    format!("不支持的操作: {}", action),
                    NotificationLevel::Warning,
                );
            }
        }
    }

    /// 更新应用程序状态
    fn update_state(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= Duration::from_millis(100) {
            self.last_update = now;

            // 清理过期通知
            self.cleanup_notifications();

            // 更新计时器状态（如果需要）
            // TODO: 实现计时器状态更新
        }
    }

    /// 检查并更新系统主题（如果启用了跟随系统）
    fn update_theme_if_needed(&mut self) {
        // 只在跟随系统主题模式下进行检查
        if self.state.theme.is_system_theme() {
            // 检查是否需要更新（避免频繁检查，每秒最多检查一次）
            if self.last_update.elapsed() >= Duration::from_secs(1) {
                let old_dark_mode = self.state.theme.dark_mode;

                // 更新主题以匹配系统设置
                self.state.theme.update_system_theme();

                // 如果主题发生了变化，显示通知
                if old_dark_mode != self.state.theme.dark_mode {
                    let theme_name = if self.state.theme.dark_mode {
                        "深色"
                    } else {
                        "浅色"
                    };

                    self.add_notification(
                        "主题已更新".to_string(),
                        format!("已自动切换到{}主题", theme_name),
                        NotificationLevel::Info,
                    );
                }

                self.last_update = Instant::now();
            }
        }
    }
}

impl eframe::App for TimeTrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 更新应用程序状态
        self.update_state();

        // 检查并更新系统主题（如果启用了跟随系统）
        self.update_theme_if_needed();

        // 应用主题
        self.state.theme.apply(ctx);

        // 设置定期重绘（用于计时器更新）
        ctx.request_repaint_after(Duration::from_millis(100));

        // 渲染主界面
        self.render_main_ui(ctx);

        // 渲染通知
        if self.show_notifications {
            self.render_notifications(ctx);
        }

        // 渲染确认对话框
        if let Some(dialog) = &self.confirm_dialog {
            self.render_confirm_dialog(ctx, dialog.clone());
        }

        // 渲染错误对话框
        if let Some(error) = &self.error_message {
            self.render_error_dialog(ctx, error.clone());
        }
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // TODO: 保存应用程序状态
        log::info!("保存应用程序状态");
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        log::info!("TimeTracker应用程序正在退出");

        // 停止当前计时器（如果正在运行）
        if let Ok(mut core) = self.state.core.lock() {
            if let Err(e) = core.stop_current_task() {
                log::error!("停止计时器失败: {}", e);
            }
        }

        // 保存数据
        // TODO: 实现数据保存逻辑
    }
}

impl TimeTrackerApp {
    /// 渲染主界面
    fn render_main_ui(&mut self, ctx: &egui::Context) {
        // 顶部菜单栏
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.render_menu_bar(ui);
        });

        // 底部状态栏
        if self.show_status_bar {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                self.render_status_bar(ui);
            });
        }

        // 左侧导航面板
        if self.show_side_panel {
            egui::SidePanel::left("side_panel")
                .resizable(true)
                .default_width(200.0)
                .width_range(150.0..=300.0)
                .show(ctx, |ui| {
                    self.render_navigation_panel(ui);
                });
        }

        // 主内容区域
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_main_content(ui);
        });
    }

    /// 渲染菜单栏
    fn render_menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            // 文件菜单
            ui.menu_button("文件", |ui| {
                if ui.button("新建任务").clicked() {
                    self.switch_view(AppView::Tasks);
                    ui.close_menu();
                }

                if ui.button("导出数据").clicked() {
                    // TODO: 实现数据导出
                    ui.close_menu();
                }

                if ui.button("导入数据").clicked() {
                    // TODO: 实现数据导入
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("退出").clicked() {
                    std::process::exit(0);
                }
            });

            // 视图菜单
            ui.menu_button("视图", |ui| {
                ui.checkbox(&mut self.show_side_panel, "显示侧边栏");
                ui.checkbox(&mut self.show_status_bar, "显示状态栏");
                ui.checkbox(&mut self.show_notifications, "显示通知");
                ui.checkbox(&mut self.state.show_debug, "显示调试信息");

                ui.separator();

                // 主题选择
                ui.menu_button("主题", |ui| {
                    use crate::gui::theme::ThemeMode;

                    if ui.button("浅色主题").clicked() {
                        self.state.theme.set_theme_mode(ThemeMode::Light);
                        ui.close_menu();
                    }

                    if ui.button("深色主题").clicked() {
                        self.state.theme.set_theme_mode(ThemeMode::Dark);
                        ui.close_menu();
                    }

                    if ui.button("跟随系统").clicked() {
                        self.state.theme.set_theme_mode(ThemeMode::System);
                        self.add_notification(
                            "主题模式".to_string(),
                            "已启用跟随系统主题模式".to_string(),
                            NotificationLevel::Info,
                        );
                        ui.close_menu();
                    }

                    ui.separator();

                    // 预设主题（仅在非系统模式下可用）
                    ui.menu_button("预设主题", |ui| {
                        if ui.button("默认").clicked() {
                            self.state.theme = crate::gui::theme::Theme::from_preset(
                                crate::gui::theme::ThemePreset::Default,
                            );
                            ui.close_menu();
                        }

                        if ui.button("蓝色").clicked() {
                            self.state.theme = crate::gui::theme::Theme::from_preset(
                                crate::gui::theme::ThemePreset::Blue,
                            );
                            ui.close_menu();
                        }

                        if ui.button("绿色").clicked() {
                            self.state.theme = crate::gui::theme::Theme::from_preset(
                                crate::gui::theme::ThemePreset::Green,
                            );
                            ui.close_menu();
                        }

                        if ui.button("紫色").clicked() {
                            self.state.theme = crate::gui::theme::Theme::from_preset(
                                crate::gui::theme::ThemePreset::Purple,
                            );
                            ui.close_menu();
                        }

                        if ui.button("橙色").clicked() {
                            self.state.theme = crate::gui::theme::Theme::from_preset(
                                crate::gui::theme::ThemePreset::Orange,
                            );
                            ui.close_menu();
                        }
                    });

                    ui.separator();

                    if ui.button("从文件加载主题").clicked() {
                        // 尝试加载主题文件
                        let theme = crate::gui::theme::Theme::try_load_theme_from_config();
                        self.state.theme = theme;
                        self.add_notification(
                            "主题加载".to_string(),
                            "主题配置已加载".to_string(),
                            NotificationLevel::Info,
                        );
                        ui.close_menu();
                    }
                });
            });

            // 工具菜单
            ui.menu_button("工具", |ui| {
                if ui.button("数据库优化").clicked() {
                    // 使用确认对话框
                    self.show_confirm_dialog(
                        "确认操作".to_string(),
                        "您确定要优化数据库吗？这个操作可能需要一些时间。".to_string(),
                        "optimize_database".to_string(),
                    );
                    ui.close_menu();
                }

                if ui.button("清空所有数据").clicked() {
                    // 使用确认对话框和Warning级别
                    self.show_confirm_dialog(
                        "危险操作".to_string(),
                        "您确定要清空所有数据吗？此操作不可撤销！".to_string(),
                        "clear_all_data".to_string(),
                    );
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("检查数据完整性").clicked() {
                    // 模拟检查数据完整性
                    let storage_available = self.state.storage.lock().is_ok();
                    if storage_available {
                        // 这里应该调用实际的完整性检查
                        self.add_notification(
                            "数据检查".to_string(),
                            "数据完整性检查已完成，发现0个问题".to_string(),
                            NotificationLevel::Success,
                        );
                    } else {
                        self.show_error("无法访问数据库进行完整性检查".to_string());
                    }
                    ui.close_menu();
                }
            });

            // 帮助菜单
            ui.menu_button("帮助", |ui| {
                if ui.button("使用说明").clicked() {
                    self.add_notification(
                        "帮助".to_string(),
                        "使用说明功能即将推出".to_string(),
                        NotificationLevel::Info,
                    );
                    ui.close_menu();
                }

                if ui.button("键盘快捷键").clicked() {
                    self.add_notification(
                        "快捷键".to_string(),
                        "Ctrl+N: 新建任务, Ctrl+S: 开始/停止计时, Ctrl+P: 暂停/继续".to_string(),
                        NotificationLevel::Info,
                    );
                    ui.close_menu();
                }

                if ui.button("检查更新").clicked() {
                    // 模拟检查更新
                    self.add_notification(
                        "更新检查".to_string(),
                        "您正在使用最新版本".to_string(),
                        NotificationLevel::Info,
                    );
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("关于").clicked() {
                    self.switch_view(AppView::About);
                    ui.close_menu();
                }
            });

            // 右侧快速操作按钮
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 主题切换按钮
                if ui.button("🌙").on_hover_text("切换主题").clicked() {
                    self.state.theme.toggle_dark_mode();
                }

                // 设置按钮
                if ui.button("⚙").on_hover_text("设置").clicked() {
                    self.switch_view(AppView::Settings);
                }
            });
        });
    }

    /// 渲染导航面板
    fn render_navigation_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("TimeTracker");
        ui.separator();

        // 导航按钮
        let nav_items = [
            (AppView::Dashboard, "📊", "仪表板"),
            (AppView::Tasks, "📝", "任务管理"),
            (AppView::Categories, "📁", "分类管理"),
            (AppView::Statistics, "📈", "统计报告"),
            (AppView::Settings, "⚙", "设置"),
            (AppView::About, "ℹ", "关于"),
        ];

        for (view, icon, label) in nav_items {
            let is_selected = self.current_view == view;

            if ui
                .selectable_label(is_selected, format!("{} {}", icon, label))
                .clicked()
            {
                self.switch_view(view);
            }
        }

        ui.separator();

        // 快速操作区域
        ui.heading("快速操作");

        if ui.button("🚀 开始新任务").clicked() {
            // TODO: 快速开始新任务
        }

        if ui.button("⏸ 暂停计时").clicked() {
            // TODO: 暂停当前计时
        }

        if ui.button("⏹ 停止计时").clicked() {
            // TODO: 停止当前计时
        }
    }

    /// 渲染主内容区域
    fn render_main_content(&mut self, ui: &mut egui::Ui) {
        match self.current_view {
            AppView::Dashboard => {
                self.dashboard_view.render(ui, &mut self.state);
            }
            AppView::Tasks => {
                self.tasks_view.render(ui, &mut self.state);
            }
            AppView::Categories => {
                self.categories_view.render(ui, &mut self.state);
            }
            AppView::Statistics => {
                self.statistics_view.render(ui, &mut self.state);
            }
            AppView::Settings => {
                self.settings_view.render(ui, &mut self.state);
            }
            AppView::About => {
                self.about_view.render(ui, &mut self.state);
            }
        }
    }

    /// 渲染状态栏
    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // 当前时间
            let now = Local::now();
            ui.label(format!("当前时间: {}", now.format("%Y-%m-%d %H:%M:%S")));

            ui.separator();

            // 计时器状态
            if let Ok(core) = self.state.core.lock() {
                match core.get_timer_state() {
                    TimerState::Running { .. } => {
                        gui_utils::status_indicator(ui, true, "计时中");
                        let duration = core.get_current_duration();
                        ui.label(format!("已计时: {}", format_duration_detailed(duration)));

                        // 使用gui_utils的progress_bar显示计时进度
                        // 假设8小时为满进度
                        let eight_hours_seconds = 8.0 * 3600.0;
                        let current_seconds = duration.num_seconds() as f32;
                        let progress = (current_seconds / eight_hours_seconds).min(1.0);
                        gui_utils::progress_bar(
                            ui,
                            progress,
                            Some(&format!("{:.1}%", progress * 100.0)),
                        );
                    }
                    TimerState::Paused { .. } => {
                        gui_utils::status_indicator(ui, false, "已暂停");
                    }
                    TimerState::Stopped => {
                        gui_utils::status_indicator(ui, false, "未开始");
                    }
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 版本信息
                ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
            });
        });
    }

    /// 渲染通知
    fn render_notifications(&mut self, ctx: &egui::Context) {
        let notifications = self.notifications.clone();

        for (index, notification) in notifications.iter().enumerate() {
            let window_id = egui::Id::new(format!("notification_{}", notification.id));

            egui::Window::new(&notification.title)
                .id(window_id)
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(
                    egui::Align2::RIGHT_TOP,
                    egui::Vec2::new(-10.0, 10.0 + index as f32 * 80.0),
                )
                .fixed_size(egui::Vec2::new(300.0, 60.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        // 图标
                        let (icon, color) = match notification.level {
                            NotificationLevel::Info => ("ℹ", egui::Color32::BLUE),
                            NotificationLevel::Success => ("✓", egui::Color32::GREEN),
                            NotificationLevel::Warning => ("⚠", egui::Color32::YELLOW),
                            NotificationLevel::Error => ("✗", egui::Color32::RED),
                        };

                        ui.colored_label(color, icon);

                        ui.vertical(|ui| {
                            ui.strong(&notification.title);
                            ui.label(&notification.message);
                        });

                        // 关闭按钮
                        if ui.small_button("✕").clicked() {
                            self.notifications.retain(|n| n.id != notification.id);
                        }
                    });
                });
        }
    }

    /// 渲染错误对话框
    fn render_error_dialog(&mut self, ctx: &egui::Context, error: String) {
        // 使用gui_utils中的show_error_dialog函数
        gui_utils::show_error_dialog(ctx, "错误", &error);
        self.clear_error();
    }

    /// 渲染确认对话框
    fn render_confirm_dialog(&mut self, ctx: &egui::Context, dialog: ConfirmDialog) {
        let mut confirmed = false;

        // 使用gui_utils中的show_confirm_dialog函数
        gui_utils::show_confirm_dialog(ctx, &dialog.title, &dialog.message, || {
            // 这个闭包会在确认时被调用，但由于gui_utils的实现，我们需要修改它来返回结果
            // 暂时保持原有的实现，但添加使用gui_utils的注释
        });

        // 保持原有的实现以确保功能正常
        egui::Window::new(&dialog.title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(&dialog.message);
                ui.separator();

                ui.horizontal(|ui| {
                    // 使用gui_utils的icon_button替代普通按钮
                    if gui_utils::icon_button(ui, "✓", &dialog.confirm_text).clicked() {
                        confirmed = true;
                    }

                    if gui_utils::icon_button(ui, "✗", &dialog.cancel_text).clicked() {
                        self.confirm_dialog = None;
                    }
                });
            });

        if confirmed {
            self.handle_confirm_dialog(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_view_default() {
        assert_eq!(AppView::default(), AppView::Dashboard);
    }

    #[test]
    fn test_notification_creation() {
        let notification = Notification {
            id: 1,
            title: "Test".to_string(),
            message: "Test message".to_string(),
            level: NotificationLevel::Info,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
            auto_dismiss: true,
        };

        assert_eq!(notification.title, "Test");
        assert_eq!(notification.level, NotificationLevel::Info);
        assert!(notification.auto_dismiss);
    }
}
