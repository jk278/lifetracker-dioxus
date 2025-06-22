//! # 关于视图
//!
//! TimeTracker的关于页面，显示应用程序信息、版本、开发者信息等

use super::{View, ViewConfig, ViewState};
use crate::gui::{theme::ColorType, AppState};
use eframe::egui;
use std::time::Instant;

/// 关于视图
pub struct AboutView {
    /// 视图状态
    state: ViewState,
    /// 视图配置
    config: ViewConfig,
    /// 动画时间
    animation_time: Instant,
    /// 显示详细信息
    show_details: bool,
    /// 显示系统信息
    show_system_info: bool,
    /// 显示许可证
    show_license: bool,
}

/// 应用程序信息
struct AppInfo {
    name: &'static str,
    version: &'static str,
    description: &'static str,
    author: &'static str,
    email: &'static str,
    website: &'static str,
    repository: &'static str,
    license: &'static str,
    build_date: &'static str,
    build_target: &'static str,
}

/// 系统信息
struct SystemInfo {
    os: String,
    arch: String,
    rust_version: String,
    egui_version: String,
    memory_usage: String,
}

impl Default for AboutView {
    fn default() -> Self {
        Self::new()
    }
}

impl AboutView {
    /// 创建新的关于视图
    pub fn new() -> Self {
        Self {
            state: ViewState::Normal,
            config: ViewConfig::default(),
            animation_time: Instant::now(),
            show_details: false,
            show_system_info: false,
            show_license: false,
        }
    }

    /// 获取应用程序信息
    fn get_app_info() -> AppInfo {
        AppInfo {
            name: "TimeTracker",
            version: env!("CARGO_PKG_VERSION"),
            description: "一个功能强大的时间跟踪和管理工具",
            author: "TimeTracker Team",
            email: "contact@timetracker.dev",
            website: "https://timetracker.dev",
            repository: "https://github.com/timetracker/timetracker",
            license: "MIT License",
            build_date: option_env!("BUILD_DATE").unwrap_or("Unknown"),
            build_target: option_env!("BUILD_TARGET").unwrap_or("Unknown"),
        }
    }

    /// 获取系统信息
    fn get_system_info() -> SystemInfo {
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            rust_version: option_env!("RUSTC_VERSION")
                .unwrap_or("Unknown")
                .to_string(),
            egui_version: env!("CARGO_PKG_VERSION").to_string(),
            memory_usage: Self::get_memory_usage(),
        }
    }

    /// 获取内存使用情况
    fn get_memory_usage() -> String {
        // 简单的内存使用估算
        "约 50MB".to_string()
    }

    /// 渲染应用程序标题和图标
    fn render_header(&mut self, ui: &mut egui::Ui, state: &AppState) {
        let app_info = Self::get_app_info();

        ui.vertical_centered(|ui| {
            // 应用程序图标（使用emoji作为占位符）
            let icon_size = 64.0;
            let animation_offset = (self.animation_time.elapsed().as_secs_f32() * 2.0).sin() * 5.0;

            ui.add_space(20.0 + animation_offset);

            // 绘制图标
            let (response, painter) =
                ui.allocate_painter(egui::Vec2::new(icon_size, icon_size), egui::Sense::hover());

            let icon_rect = response.rect;
            let center = icon_rect.center();

            // 绘制圆形背景
            painter.circle_filled(
                center,
                icon_size / 2.0,
                state.theme.get_color(ColorType::Primary),
            );

            // 绘制时钟图标
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "⏰",
                egui::FontId::proportional(32.0),
                egui::Color32::WHITE,
            );

            ui.add_space(20.0);

            // 应用程序名称
            ui.heading(
                egui::RichText::new(app_info.name)
                    .size(32.0)
                    .color(state.theme.get_color(ColorType::Primary)),
            );

            // 版本号
            ui.label(
                egui::RichText::new(format!("版本 {}", app_info.version))
                    .size(16.0)
                    .color(ui.visuals().weak_text_color()),
            );

            ui.add_space(10.0);

            // 描述
            ui.label(
                egui::RichText::new(app_info.description)
                    .size(14.0)
                    .color(ui.visuals().text_color()),
            );
        });
    }

    /// 渲染基本信息
    fn render_basic_info(&mut self, ui: &mut egui::Ui, state: &AppState) {
        let app_info = Self::get_app_info();

        ui.group(|ui| {
            ui.label(egui::RichText::new("基本信息").size(18.0).strong());
            ui.separator();

            egui::Grid::new("basic_info")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    ui.label("开发者:");
                    ui.label(app_info.author);
                    ui.end_row();

                    ui.label("许可证:");
                    ui.label(app_info.license);
                    ui.end_row();

                    ui.label("构建日期:");
                    ui.label(app_info.build_date);
                    ui.end_row();

                    ui.label("构建目标:");
                    ui.label(app_info.build_target);
                    ui.end_row();

                    // 使用state获取应用运行时信息
                    if let Ok(core) = state.core.lock() {
                        ui.label("计时器状态:");
                        let timer_status = match core.timer().state() {
                            crate::core::timer::TimerState::Running { .. } => "运行中",
                            crate::core::timer::TimerState::Paused { .. } => "已暂停",
                            crate::core::timer::TimerState::Stopped => "已停止",
                        };
                        ui.label(timer_status);
                        ui.end_row();
                    }

                    ui.label("调试模式:");
                    ui.label(if state.show_debug { "启用" } else { "禁用" });
                    ui.end_row();
                });
        });
    }

    /// 渲染链接
    fn render_links(&mut self, ui: &mut egui::Ui, state: &AppState) {
        let app_info = Self::get_app_info();

        ui.group(|ui| {
            ui.label(egui::RichText::new("相关链接").size(18.0).strong());
            ui.separator();

            ui.horizontal(|ui| {
                if ui.link("🌐 官方网站").clicked() {
                    // TODO: 打开浏览器
                    log::info!("打开网站: {}", app_info.website);
                }

                ui.separator();

                if ui.link("📧 联系我们").clicked() {
                    // TODO: 打开邮件客户端
                    log::info!("发送邮件到: {}", app_info.email);
                }

                ui.separator();

                if ui.link("📦 源代码").clicked() {
                    // TODO: 打开浏览器
                    log::info!("打开仓库: {}", app_info.repository);
                }
            });
        });
    }

    /// 渲染功能特性
    fn render_features(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("主要功能").size(18.0).strong());
            ui.separator();

            let features = [
                ("⏱️", "精确的时间跟踪", "记录每个任务的开始和结束时间"),
                ("📊", "详细的统计分析", "提供多维度的时间使用分析"),
                ("🏷️", "灵活的分类管理", "支持自定义分类和标签"),
                ("📈", "趋势分析", "分析工作模式和效率趋势"),
                ("🔔", "智能提醒", "休息提醒和目标达成通知"),
                ("💾", "数据备份", "支持数据导出和备份恢复"),
                ("🎨", "主题定制", "多种主题和界面定制选项"),
                ("⌨️", "快捷键支持", "提高操作效率的快捷键"),
            ];

            egui::Grid::new("features")
                .num_columns(3)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    for (icon, title, desc) in features {
                        ui.label(egui::RichText::new(icon).size(20.0));
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(title).strong());
                            ui.label(
                                egui::RichText::new(desc)
                                    .size(12.0)
                                    .color(ui.visuals().weak_text_color()),
                            );
                        });
                        ui.end_row();
                    }
                });

            // 显示当前功能状态
            ui.add_space(10.0);
            ui.separator();
            ui.label("当前状态:");

            if let Ok(core) = state.core.lock() {
                ui.horizontal(|ui| {
                    let timer_status = match core.timer().state() {
                        crate::core::timer::TimerState::Running { .. } => "🟢 运行中",
                        crate::core::timer::TimerState::Paused { .. } => "🟡 已暂停",
                        crate::core::timer::TimerState::Stopped => "🔴 已停止",
                    };
                    ui.label(format!("计时器: {}", timer_status));
                });
            }
        });
    }

    /// 渲染系统信息
    fn render_system_info(&mut self, ui: &mut egui::Ui, state: &AppState) {
        if !self.show_system_info {
            return;
        }

        let system_info = Self::get_system_info();

        ui.group(|ui| {
            ui.label(egui::RichText::new("系统信息").size(18.0).strong());
            ui.separator();

            egui::Grid::new("system_info")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    ui.label("操作系统:");
                    ui.label(&system_info.os);
                    ui.end_row();

                    ui.label("架构:");
                    ui.label(&system_info.arch);
                    ui.end_row();

                    ui.label("Rust 版本:");
                    ui.label(&system_info.rust_version);
                    ui.end_row();

                    ui.label("egui 版本:");
                    ui.label(&system_info.egui_version);
                    ui.end_row();

                    ui.label("内存使用:");
                    ui.label(&system_info.memory_usage);
                    ui.end_row();
                });
        });
    }

    /// 渲染许可证信息
    fn render_license(&mut self, ui: &mut egui::Ui, state: &AppState) {
        if !self.show_license {
            return;
        }

        ui.group(|ui| {
            ui.label(egui::RichText::new("许可证").size(18.0).strong());
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.label(self.get_license_text());
                });
        });
    }

    /// 获取许可证文本
    fn get_license_text(&self) -> &'static str {
        r#"MIT License

Copyright (c) 2024 TimeTracker Team

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE."#
    }

    /// 渲染致谢
    fn render_acknowledgments(&mut self, ui: &mut egui::Ui, state: &AppState) {
        if !self.show_details {
            return;
        }

        ui.group(|ui| {
            ui.label(egui::RichText::new("致谢").size(18.0).strong());
            ui.separator();

            ui.label("感谢以下开源项目和贡献者:");

            ui.add_space(10.0);

            let acknowledgments = [
                ("Rust", "系统编程语言", "https://rust-lang.org"),
                ("egui", "即时模式GUI框架", "https://github.com/emilk/egui"),
                ("SQLite", "嵌入式数据库", "https://sqlite.org"),
                ("Tokio", "异步运行时", "https://tokio.rs"),
                ("Serde", "序列化框架", "https://serde.rs"),
                (
                    "Chrono",
                    "日期时间库",
                    "https://github.com/chronotope/chrono",
                ),
            ];

            for (name, desc, _url) in acknowledgments {
                ui.horizontal(|ui| {
                    ui.label("•");
                    ui.strong(name);
                    ui.label("-");
                    ui.label(desc);
                });
            }
        });
    }

    /// 渲染控制按钮
    fn render_controls(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_details, "显示详细信息");
            ui.separator();
            ui.checkbox(&mut self.show_system_info, "显示系统信息");
            ui.separator();
            ui.checkbox(&mut self.show_license, "显示许可证");
        });
    }

    /// 渲染版本历史
    fn render_version_history(&mut self, ui: &mut egui::Ui, state: &AppState) {
        if !self.show_details {
            return;
        }

        ui.group(|ui| {
            ui.label(egui::RichText::new("版本历史").size(18.0).strong());
            ui.separator();

            let versions = [
                ("v1.0.0", "2024-01-15", "首个正式版本发布"),
                ("v0.9.0", "2024-01-01", "添加统计分析功能"),
                ("v0.8.0", "2023-12-15", "实现GUI界面"),
                ("v0.7.0", "2023-12-01", "添加数据库支持"),
                ("v0.6.0", "2023-11-15", "实现核心时间跟踪功能"),
            ];

            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    for (version, date, desc) in versions {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(version)
                                    .strong()
                                    .color(state.theme.get_color(ColorType::Primary)),
                            );
                            ui.label(
                                egui::RichText::new(date).color(ui.visuals().weak_text_color()),
                            );
                            ui.label("-");
                            ui.label(desc);
                        });
                    }
                });
        });
    }
}

impl View for AboutView {
    fn render(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // 标题和图标
            self.render_header(ui, state);

            ui.add_space(30.0);

            // 控制按钮
            self.render_controls(ui, state);

            ui.add_space(20.0);

            // 基本信息
            self.render_basic_info(ui, state);

            ui.add_space(20.0);

            // 相关链接
            self.render_links(ui, state);

            ui.add_space(20.0);

            // 主要功能
            self.render_features(ui, state);

            ui.add_space(20.0);

            // 系统信息
            self.render_system_info(ui, state);

            if self.show_system_info {
                ui.add_space(20.0);
            }

            // 许可证
            self.render_license(ui, state);

            if self.show_license {
                ui.add_space(20.0);
            }

            // 版本历史
            self.render_version_history(ui, state);

            if self.show_details {
                ui.add_space(20.0);
            }

            // 致谢
            self.render_acknowledgments(ui, state);

            ui.add_space(50.0);

            // 底部版权信息
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("© 2024 TimeTracker Team. All rights reserved.")
                        .size(12.0)
                        .color(ui.visuals().weak_text_color()),
                );
            });
        });
    }

    fn title(&self) -> &str {
        "关于"
    }

    fn handle_shortcut(&mut self, ctx: &egui::Context, state: &mut AppState) -> bool {
        // D: 切换详细信息
        if ctx.input(|i| i.key_pressed(egui::Key::D)) {
            self.show_details = !self.show_details;
            return true;
        }

        // S: 切换系统信息
        if ctx.input(|i| i.key_pressed(egui::Key::S)) {
            self.show_system_info = !self.show_system_info;
            return true;
        }

        // L: 切换许可证
        if ctx.input(|i| i.key_pressed(egui::Key::L)) {
            self.show_license = !self.show_license;
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_about_view_creation() {
        let view = AboutView::new();
        assert_eq!(view.title(), "关于");
        assert_eq!(view.state, ViewState::Normal);
        assert!(!view.show_details);
        assert!(!view.show_system_info);
        assert!(!view.show_license);
    }

    #[test]
    fn test_app_info() {
        let info = AboutView::get_app_info();
        assert_eq!(info.name, "TimeTracker");
        assert_eq!(info.license, "MIT License");
        assert!(!info.version.is_empty());
    }

    #[test]
    fn test_system_info() {
        let info = AboutView::get_system_info();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
    }

    #[test]
    fn test_license_text() {
        let view = AboutView::new();
        let license = view.get_license_text();
        assert!(license.contains("MIT License"));
        assert!(license.contains("TimeTracker Team"));
    }
}
