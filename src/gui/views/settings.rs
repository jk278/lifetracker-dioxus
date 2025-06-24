//! # 设置视图
//!
//! TimeTracker的设置配置界面，用于管理应用程序的各种选项和偏好设置

use super::{View, ViewConfig, ViewState};
use crate::{
    config::AppConfig,
    gui::{
        theme::{ColorType, Theme, ThemeMode, ThemePreset},
        AppState,
    },
};
use eframe::egui;
use std::time::{Duration, Instant};

/// 设置视图
pub struct SettingsView {
    /// 视图状态
    state: ViewState,
    /// 视图配置
    config: ViewConfig,
    /// 设置分类
    category: SettingsCategory,
    /// 临时配置（用于编辑）
    temp_config: AppConfig,
    /// 临时主题配置
    temp_theme: Theme,
    /// 是否有未保存的更改
    has_changes: bool,
    /// 上次保存时间
    last_save: Option<Instant>,
    /// 显示确认对话框
    show_confirm_dialog: bool,
    /// 确认对话框类型
    confirm_type: ConfirmType,
    /// 错误消息
    error_message: Option<String>,
    /// 成功消息
    success_message: Option<String>,
    /// 消息显示时间
    message_time: Option<Instant>,
    /// 导入/导出路径
    import_export_path: String,
    /// 备份路径
    backup_path: String,
}

/// 设置分类
#[derive(Debug, Clone, Copy, PartialEq)]
enum SettingsCategory {
    /// 常规设置
    General,
    /// 外观设置
    Appearance,
    /// 通知设置
    Notifications,
    /// 数据管理
    Data,
    /// 快捷键设置
    Shortcuts,
    /// 高级设置
    Advanced,
}

/// 确认对话框类型
#[derive(Debug, Clone, Copy, PartialEq)]
enum ConfirmType {
    /// 重置设置
    Reset,
    /// 清除数据
    ClearData,
    /// 导入数据
    ImportData,
    /// 恢复备份
    RestoreBackup,
}

impl Default for SettingsView {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsView {
    /// 创建新的设置视图
    pub fn new() -> Self {
        Self {
            state: ViewState::Normal,
            config: ViewConfig::default(),
            category: SettingsCategory::General,
            temp_config: AppConfig::default(),
            temp_theme: Theme::default(),
            has_changes: false,
            last_save: None,
            show_confirm_dialog: false,
            confirm_type: ConfirmType::Reset,
            error_message: None,
            success_message: None,
            message_time: None,
            import_export_path: String::new(),
            backup_path: String::new(),
        }
    }

    /// 加载当前配置
    fn load_config(&mut self, state: &AppState) {
        if let Ok(core) = state.core.lock() {
            self.temp_config = core.config().clone();
        }
        self.temp_theme = state.theme.clone();
        self.has_changes = false;
    }

    /// 保存配置
    fn save_config(&mut self, state: &mut AppState) {
        // 保存核心配置
        if let Ok(mut core) = state.core.lock() {
            if let Err(e) = core.update_config(self.temp_config.clone()) {
                self.show_error(&format!("保存配置失败: {}", e));
                return;
            }
        }

        // 保存主题配置
        state.theme = self.temp_theme.clone();
        if let Err(e) = state.theme.save_to_config_dir() {
            log::warn!("保存主题配置失败: {}", e);
            self.show_error(&format!("保存主题配置失败: {}", e));
            return;
        }

        self.has_changes = false;
        self.last_save = Some(Instant::now());
        self.show_success("设置已保存");
    }

    /// 重置配置
    fn reset_config(&mut self, state: &mut AppState) {
        self.temp_config = AppConfig::default();
        self.temp_theme = Theme::default();
        self.save_config(state);
        self.show_success("设置已重置为默认值");
    }

    /// 显示错误消息
    fn show_error(&mut self, message: &str) {
        self.error_message = Some(message.to_string());
        self.success_message = None;
        self.message_time = Some(Instant::now());
    }

    /// 显示成功消息
    fn show_success(&mut self, message: &str) {
        self.success_message = Some(message.to_string());
        self.error_message = None;
        self.message_time = Some(Instant::now());
    }

    /// 清除消息
    fn clear_messages(&mut self) {
        if let Some(time) = self.message_time {
            if time.elapsed() >= Duration::from_secs(3) {
                self.error_message = None;
                self.success_message = None;
                self.message_time = None;
            }
        }
    }

    /// 渲染侧边栏
    fn render_sidebar(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.vertical(|ui| {
            ui.heading("设置分类");
            ui.separator();

            let categories = [
                (SettingsCategory::General, "🔧 常规"),
                (SettingsCategory::Appearance, "🎨 外观"),
                (SettingsCategory::Notifications, "🔔 通知"),
                (SettingsCategory::Data, "💾 数据"),
                (SettingsCategory::Shortcuts, "⌨️ 快捷键"),
                (SettingsCategory::Advanced, "⚙️ 高级"),
            ];

            for (cat, label) in categories {
                let is_selected = self.category == cat;
                if ui.selectable_label(is_selected, label).clicked() {
                    self.category = cat;
                }
            }

            ui.add_space(20.0);

            // 保存/重置按钮
            ui.vertical(|ui| {
                if ui.button("💾 保存设置").clicked() {
                    self.save_config(state);
                }

                ui.add_space(5.0);

                if ui.button("🔄 重置设置").clicked() {
                    self.show_confirm_dialog = true;
                    self.confirm_type = ConfirmType::Reset;
                }

                ui.add_space(10.0);

                // 显示保存状态
                if self.has_changes {
                    ui.colored_label(
                        state.theme.get_color(ColorType::Warning),
                        "⚠ 有未保存的更改",
                    );
                } else if let Some(save_time) = self.last_save {
                    if save_time.elapsed() < Duration::from_secs(5) {
                        ui.colored_label(state.theme.get_color(ColorType::Success), "✓ 已保存");
                    }
                }
            });
        });
    }

    /// 渲染常规设置
    fn render_general_settings(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.heading("常规设置");
        ui.separator();

        egui::Grid::new("general_settings")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                // 应用名称
                ui.label("应用名称:");
                if ui
                    .text_edit_singleline(&mut self.temp_config.general.language)
                    .changed()
                {
                    self.has_changes = true;
                }
                ui.end_row();

                // 默认分类
                ui.label("默认分类:");
                if ui
                    .text_edit_singleline(
                        self.temp_config
                            .general
                            .default_category_id
                            .get_or_insert_with(|| "".to_string()),
                    )
                    .changed()
                {
                    self.has_changes = true;
                }
                ui.end_row();

                // 自动保存间隔(分钟)
                ui.label("自动保存间隔(分钟):");
                if ui
                    .add(
                        egui::DragValue::new(
                            self.temp_config
                                .general
                                .work_reminder_interval
                                .get_or_insert(5),
                        )
                        .range(1..=60)
                        .suffix("分钟"),
                    )
                    .changed()
                {
                    self.has_changes = true;
                }
                ui.end_row();

                // 启动时恢复
                ui.label("启动时恢复:");
                if ui
                    .checkbox(&mut self.temp_config.general.auto_start_timer, "")
                    .changed()
                {
                    self.has_changes = true;
                }
                ui.end_row();

                // 最小化到托盘
                ui.label("最小化到托盘:");
                if ui
                    .checkbox(&mut self.temp_config.general.minimize_to_tray, "")
                    .changed()
                {
                    self.has_changes = true;
                }
                ui.end_row();

                // 退出时确认
                ui.label("退出时确认:");
                if ui
                    .checkbox(&mut self.temp_config.general.auto_start, "")
                    .changed()
                {
                    self.has_changes = true;
                }
                ui.end_row();

                // 语言
                ui.label("语言:");
                egui::ComboBox::from_id_source("language")
                    .selected_text(&self.temp_config.general.language)
                    .show_ui(ui, |ui| {
                        let languages = [("zh-CN", "中文"), ("en-US", "English")];
                        for (code, name) in languages {
                            if ui
                                .selectable_value(
                                    &mut self.temp_config.general.language,
                                    code.to_string(),
                                    name,
                                )
                                .clicked()
                            {
                                self.has_changes = true;
                            }
                        }
                    });
                ui.end_row();
            });
    }

    /// 渲染外观设置
    fn render_appearance_settings(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.heading("外观设置");
        ui.separator();

        egui::Grid::new("appearance_settings")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                // 主题选择
                ui.label("主题:");
                ui.horizontal(|ui| {
                    let current_mode = self.temp_theme.get_theme_mode();

                    if ui
                        .selectable_value(&mut self.temp_theme.theme_mode, ThemeMode::Light, "浅色")
                        .clicked()
                    {
                        if current_mode != ThemeMode::Light {
                            self.temp_theme.set_theme_mode(ThemeMode::Light);
                            self.has_changes = true;
                        }
                    }

                    if ui
                        .selectable_value(&mut self.temp_theme.theme_mode, ThemeMode::Dark, "深色")
                        .clicked()
                    {
                        if current_mode != ThemeMode::Dark {
                            self.temp_theme.set_theme_mode(ThemeMode::Dark);
                            self.has_changes = true;
                        }
                    }

                    if ui
                        .selectable_value(
                            &mut self.temp_theme.theme_mode,
                            ThemeMode::System,
                            "跟随系统",
                        )
                        .clicked()
                    {
                        if current_mode != ThemeMode::System {
                            self.temp_theme.set_theme_mode(ThemeMode::System);
                            self.has_changes = true;
                        }
                    }
                });
                ui.end_row();

                // 当前主题状态显示
                if self.temp_theme.get_theme_mode() == ThemeMode::System {
                    ui.label("当前状态:");
                    let system_dark = Theme::detect_system_dark_mode();
                    let status_text = if system_dark {
                        "🌙 系统当前为深色模式"
                    } else {
                        "☀️ 系统当前为浅色模式"
                    };
                    ui.label(
                        egui::RichText::new(status_text).color(ui.visuals().weak_text_color()),
                    );
                    ui.end_row();
                }

                // 预设主题
                ui.label("预设主题:");
                egui::ComboBox::from_id_source("theme_preset")
                    .selected_text("选择预设")
                    .show_ui(ui, |ui| {
                        if ui.button("默认").clicked() {
                            self.temp_theme = Theme::from_preset(ThemePreset::Default);
                            self.has_changes = true;
                        }
                        if ui.button("蓝色").clicked() {
                            self.temp_theme = Theme::from_preset(ThemePreset::Blue);
                            self.has_changes = true;
                        }
                        if ui.button("绿色").clicked() {
                            self.temp_theme = Theme::from_preset(ThemePreset::Green);
                            self.has_changes = true;
                        }
                        if ui.button("紫色").clicked() {
                            self.temp_theme = Theme::from_preset(ThemePreset::Purple);
                            self.has_changes = true;
                        }
                    });
                ui.end_row();

                // 配置文件路径显示
                ui.label("配置文件:");
                ui.horizontal(|ui| {
                    let config_path = Theme::get_config_file_path();
                    ui.label(
                        egui::RichText::new(&config_path).color(ui.visuals().weak_text_color()),
                    );
                    if ui
                        .small_button("📋")
                        .on_hover_text("复制路径到剪贴板")
                        .clicked()
                    {
                        ui.ctx().copy_text(config_path);
                        self.show_success("配置文件路径已复制到剪贴板");
                    }
                });
                ui.end_row();

                // 主色调
                ui.label("主色调:");
                let mut color = [
                    self.temp_theme.primary_color.r() as f32 / 255.0,
                    self.temp_theme.primary_color.g() as f32 / 255.0,
                    self.temp_theme.primary_color.b() as f32 / 255.0,
                ];
                if ui.color_edit_button_rgb(&mut color).changed() {
                    self.temp_theme.primary_color.primary = [
                        (color[0] * 255.0) as u8,
                        (color[1] * 255.0) as u8,
                        (color[2] * 255.0) as u8,
                    ];
                    self.has_changes = true;
                }
                ui.end_row();

                // 字体大小
                ui.label("字体大小:");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.temp_config.ui.font_size)
                            .range(8.0..=24.0)
                            .suffix("px"),
                    )
                    .changed()
                {
                    self.has_changes = true;
                }
                ui.end_row();

                // 窗口透明度
                ui.label("窗口透明度:");
                if ui
                    .add(
                        egui::Slider::new(&mut self.temp_config.ui.opacity, 0.5..=1.0)
                            .text("透明度")
                            .show_value(true),
                    )
                    .changed()
                {
                    self.has_changes = true;
                }
                ui.end_row();

                // 动画效果
                ui.label("启用动画:");
                if ui
                    .checkbox(&mut self.temp_theme.animations.enabled, "")
                    .changed()
                {
                    self.has_changes = true;
                }
                ui.end_row();

                // 动画速度
                if self.temp_theme.animations.enabled {
                    ui.label("动画速度:");
                    if ui
                        .add(
                            egui::Slider::new(&mut self.temp_theme.animations.duration_ms, 1..=5)
                                .text("倍速"),
                        )
                        .changed()
                    {
                        self.has_changes = true;
                    }
                    ui.end_row();
                }
            });
    }

    /// 渲染通知设置
    fn render_notification_settings(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.heading("通知设置");
        ui.separator();

        egui::Grid::new("notification_settings")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                // 启用通知
                ui.label("启用通知:");
                if ui
                    .checkbox(&mut self.temp_config.notifications.enabled, "")
                    .changed()
                {
                    self.has_changes = true;
                }
                ui.end_row();

                if self.temp_config.notifications.enabled {
                    ui.indent("notifications", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("任务完成通知:");
                            if ui
                                .checkbox(&mut self.temp_config.notifications.notify_task_end, "")
                                .changed()
                            {
                                self.has_changes = true;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("休息提醒:");
                            if ui
                                .checkbox(&mut self.temp_config.notifications.notify_break_time, "")
                                .changed()
                            {
                                self.has_changes = true;
                            }
                        });

                        if self.temp_config.notifications.notify_break_time {
                            ui.horizontal(|ui| {
                                ui.label("提醒间隔（分钟）:");
                                if ui
                                    .add(
                                        egui::DragValue::new(
                                            self.temp_config
                                                .general
                                                .break_reminder_interval
                                                .get_or_insert(30),
                                        )
                                        .range(1..=120)
                                        .suffix("分钟"),
                                    )
                                    .changed()
                                {
                                    self.has_changes = true;
                                }
                            });
                        }

                        ui.horizontal(|ui| {
                            ui.label("每日目标提醒:");
                            if ui
                                .checkbox(&mut self.temp_config.notifications.notify_work_time, "")
                                .changed()
                            {
                                self.has_changes = true;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("通知声音:");
                            if ui
                                .checkbox(
                                    &mut self.temp_config.notifications.sound_notifications,
                                    "",
                                )
                                .changed()
                            {
                                self.has_changes = true;
                            }
                        });
                    });
                }
            });
    }

    /// 渲染数据管理设置
    fn render_data_settings(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.heading("数据管理");
        ui.separator();

        // 数据库信息
        ui.group(|ui| {
            ui.label("数据库信息");
            ui.separator();

            // 暂时禁用数据库统计信息获取以调试问题
            ui.label("数据库统计信息暂时不可用");
            ui.label("(正在调试数据库连接问题)")
        });

        ui.add_space(20.0);

        // 备份和恢复
        ui.group(|ui| {
            ui.label("备份和恢复");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("备份路径:");
                ui.text_edit_singleline(&mut self.backup_path);
                if ui.button("📁").clicked() {
                    // TODO: 打开文件选择对话框
                }
            });

            ui.horizontal(|ui| {
                if ui.button("💾 创建备份").clicked() {
                    self.create_backup(state);
                }

                if ui.button("📥 恢复备份").clicked() {
                    self.show_confirm_dialog = true;
                    self.confirm_type = ConfirmType::RestoreBackup;
                }
            });
        });

        ui.add_space(20.0);

        // 导入和导出
        ui.group(|ui| {
            ui.label("导入和导出");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("文件路径:");
                ui.text_edit_singleline(&mut self.import_export_path);
                if ui.button("📁").clicked() {
                    // TODO: 打开文件选择对话框
                }
            });

            ui.horizontal(|ui| {
                if ui.button("📤 导出数据").clicked() {
                    self.export_data(state);
                }

                if ui.button("📥 导入数据").clicked() {
                    self.show_confirm_dialog = true;
                    self.confirm_type = ConfirmType::ImportData;
                }
            });
        });

        ui.add_space(20.0);

        // 危险操作
        ui.group(|ui| {
            ui.colored_label(state.theme.get_color(ColorType::Error), "危险操作");
            ui.separator();

            if ui.button("🗑️ 清除所有数据").clicked() {
                self.show_confirm_dialog = true;
                self.confirm_type = ConfirmType::ClearData;
            }

            if ui.button("🔧 优化数据库").clicked() {
                self.optimize_database(state);
            }
        });
    }

    /// 渲染快捷键设置
    fn render_shortcuts_settings(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.heading("快捷键设置");
        ui.separator();

        ui.label("快捷键功能开发中...");

        // TODO: 实现快捷键设置
        egui::Grid::new("shortcuts")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                ui.label("开始/停止任务:");
                ui.label("Ctrl+Space");
                ui.end_row();

                ui.label("暂停/恢复任务:");
                ui.label("Ctrl+P");
                ui.end_row();

                ui.label("新建任务:");
                ui.label("Ctrl+N");
                ui.end_row();

                ui.label("刷新:");
                ui.label("F5");
                ui.end_row();

                ui.label("设置:");
                ui.label("Ctrl+,");
                ui.end_row();
            });
    }

    /// 渲染高级设置
    fn render_advanced_settings(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.heading("高级设置");
        ui.separator();

        egui::Grid::new("advanced_settings")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                // 调试模式
                ui.label("调试模式:");
                if ui
                    .checkbox(&mut self.temp_config.advanced.debug_mode, "")
                    .changed()
                {
                    self.has_changes = true;
                }
                ui.end_row();

                // 日志级别
                ui.label("日志级别:");
                egui::ComboBox::from_id_source("log_level")
                    .selected_text(&self.temp_config.advanced.log_level)
                    .show_ui(ui, |ui| {
                        let levels = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
                        for level in levels {
                            if ui
                                .selectable_value(
                                    &mut self.temp_config.advanced.log_level,
                                    level.to_string(),
                                    level,
                                )
                                .clicked()
                            {
                                self.has_changes = true;
                            }
                        }
                    });
                ui.end_row();

                // 性能监控
                ui.label("性能监控:");
                if ui
                    .checkbox(&mut self.temp_config.advanced.performance_monitoring, "")
                    .changed()
                {
                    self.has_changes = true;
                }
                ui.end_row();

                // 数据库连接池大小
                ui.label("数据库连接池大小:");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.temp_config.advanced.db_pool_size)
                            .range(1..=20),
                    )
                    .changed()
                {
                    self.has_changes = true;
                }
                ui.end_row();

                // 缓存大小（临时删除，因为配置中没有这个字段）
                // ui.label("缓存大小(MB):");
                // if ui
                //     .add(
                //         egui::DragValue::new(&mut self.temp_config.cache_size_mb)
                //             .range(10..=500)
                //     )
                //     .changed()
                // {
                //     self.has_changes = true;
                // }
            });
    }

    /// 创建备份
    fn create_backup(&mut self, state: &mut AppState) {
        if let Ok(storage) = state.storage.lock() {
            match storage.create_backup(&self.backup_path) {
                Ok(_) => self.show_success("备份创建成功"),
                Err(e) => self.show_error(&format!("备份创建失败: {}", e)),
            }
        }
    }

    /// 恢复备份
    fn restore_backup(&mut self, state: &mut AppState) {
        if let Ok(mut storage) = state.storage.lock() {
            match storage.restore_backup(&self.backup_path) {
                Ok(_) => {
                    self.show_success("备份恢复成功");
                }
                Err(e) => {
                    self.show_error(&format!("备份恢复失败: {}", e));
                }
            }
        }
    }

    /// 导出数据
    fn export_data(&mut self, state: &mut AppState) {
        if let Ok(storage) = state.storage.lock() {
            match storage.export_data(&self.import_export_path) {
                Ok(_) => self.show_success("数据导出成功"),
                Err(e) => self.show_error(&format!("数据导出失败: {}", e)),
            }
        }
    }

    /// 导入数据
    fn import_data(&mut self, state: &mut AppState) {
        if let Ok(mut storage) = state.storage.lock() {
            match storage.import_data(&self.import_export_path) {
                Ok(_) => {
                    self.show_success("数据导入成功");
                }
                Err(e) => {
                    self.show_error(&format!("数据导入失败: {}", e));
                }
            }
        }
    }

    /// 清除所有数据
    fn clear_all_data(&mut self, state: &mut AppState) {
        if let Ok(mut storage) = state.storage.lock() {
            match storage.clear_all_data() {
                Ok(_) => {
                    self.show_success("数据清除成功");
                }
                Err(e) => {
                    self.show_error(&format!("数据清除失败: {}", e));
                }
            }
        }
    }

    /// 优化数据库
    fn optimize_database(&mut self, state: &mut AppState) {
        if let Ok(storage) = state.storage.lock() {
            match storage.optimize_database() {
                Ok(_) => self.show_success("数据库优化完成"),
                Err(e) => self.show_error(&format!("数据库优化失败: {}", e)),
            }
        }
    }

    /// 渲染确认对话框
    fn render_confirm_dialog(&mut self, ctx: &egui::Context, state: &mut AppState) {
        if !self.show_confirm_dialog {
            return;
        }

        let (title, message, action) = match self.confirm_type {
            ConfirmType::Reset => (
                "重置设置",
                "确定要重置所有设置为默认值吗？此操作不可撤销。",
                "重置",
            ),
            ConfirmType::ClearData => (
                "清除数据",
                "确定要清除所有数据吗？此操作不可撤销，建议先创建备份。",
                "清除",
            ),
            ConfirmType::ImportData => ("导入数据", "确定要导入数据吗？这将覆盖现有数据。", "导入"),
            ConfirmType::RestoreBackup => {
                ("恢复备份", "确定要恢复备份吗？这将覆盖现有数据。", "恢复")
            }
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(message);

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    if ui.button(action).clicked() {
                        match self.confirm_type {
                            ConfirmType::Reset => self.reset_config(state),
                            ConfirmType::ClearData => self.clear_all_data(state),
                            ConfirmType::ImportData => self.import_data(state),
                            ConfirmType::RestoreBackup => self.restore_backup(state),
                        }
                        self.show_confirm_dialog = false;
                    }

                    if ui.button("取消").clicked() {
                        self.show_confirm_dialog = false;
                    }
                });
            });
    }

    /// 渲染消息
    fn render_messages(&mut self, ui: &mut egui::Ui, state: &AppState) {
        if let Some(error) = &self.error_message {
            ui.colored_label(
                state.theme.get_color(ColorType::Error),
                format!("❌ {}", error),
            );
        }

        if let Some(success) = &self.success_message {
            ui.colored_label(
                state.theme.get_color(ColorType::Success),
                format!("✅ {}", success),
            );
        }
    }
}

impl View for SettingsView {
    fn render(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        // 清除过期消息
        self.clear_messages();

        ui.horizontal(|ui| {
            // 侧边栏
            ui.allocate_ui_with_layout(
                egui::Vec2::new(200.0, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    self.render_sidebar(ui, state);
                },
            );

            ui.separator();

            // 主内容区域
            ui.allocate_ui_with_layout(
                egui::Vec2::new(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        match self.category {
                            SettingsCategory::General => self.render_general_settings(ui, state),
                            SettingsCategory::Appearance => {
                                self.render_appearance_settings(ui, state)
                            }
                            SettingsCategory::Notifications => {
                                self.render_notification_settings(ui, state)
                            }
                            SettingsCategory::Data => self.render_data_settings(ui, state),
                            SettingsCategory::Shortcuts => {
                                self.render_shortcuts_settings(ui, state)
                            }
                            SettingsCategory::Advanced => self.render_advanced_settings(ui, state),
                        }

                        ui.add_space(20.0);

                        // 渲染消息
                        self.render_messages(ui, state);
                    });
                },
            );
        });

        // 渲染确认对话框
        self.render_confirm_dialog(ui.ctx(), state);
    }

    fn title(&self) -> &str {
        "设置"
    }

    fn initialize(&mut self, state: &mut AppState) {
        self.load_config(state);
    }

    fn handle_shortcut(&mut self, ctx: &egui::Context, state: &mut AppState) -> bool {
        // Ctrl+S: 保存设置
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            self.save_config(state);
            return true;
        }

        // Ctrl+R: 重置设置
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::R)) {
            self.show_confirm_dialog = true;
            self.confirm_type = ConfirmType::Reset;
            return true;
        }

        // 1-6: 切换设置分类
        if ctx.input(|i| i.key_pressed(egui::Key::Num1)) {
            self.category = SettingsCategory::General;
            return true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num2)) {
            self.category = SettingsCategory::Appearance;
            return true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num3)) {
            self.category = SettingsCategory::Notifications;
            return true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num4)) {
            self.category = SettingsCategory::Data;
            return true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num5)) {
            self.category = SettingsCategory::Shortcuts;
            return true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num6)) {
            self.category = SettingsCategory::Advanced;
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_view_creation() {
        let view = SettingsView::new();
        assert_eq!(view.title(), "设置");
        assert_eq!(view.state, ViewState::Normal);
        assert_eq!(view.category, SettingsCategory::General);
        assert!(!view.has_changes);
    }

    #[test]
    fn test_settings_category() {
        assert_eq!(SettingsCategory::General, SettingsCategory::General);
        assert_ne!(SettingsCategory::General, SettingsCategory::Appearance);
    }

    #[test]
    fn test_confirm_type() {
        assert_eq!(ConfirmType::Reset, ConfirmType::Reset);
        assert_ne!(ConfirmType::Reset, ConfirmType::ClearData);
    }
}
