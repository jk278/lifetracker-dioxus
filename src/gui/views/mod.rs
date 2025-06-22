//! # GUI视图模块
//!
//! 定义TimeTracker应用程序的各个界面视图

mod about;
mod categories;
mod dashboard;
mod settings;
mod statistics;
mod tasks;

pub use about::AboutView;
pub use categories::CategoriesView;
pub use dashboard::DashboardView;
pub use settings::SettingsView;
pub use statistics::StatisticsView;
pub use tasks::TasksView;

use crate::gui::AppState;
use eframe::egui;

/// 视图特征
pub trait View {
    /// 渲染视图
    fn render(&mut self, ui: &mut egui::Ui, state: &mut AppState);

    /// 获取视图标题
    fn title(&self) -> &str;

    /// 视图是否需要刷新
    fn needs_refresh(&self) -> bool {
        false
    }

    /// 刷新视图数据
    fn refresh(&mut self, _state: &mut AppState) {
        // 默认实现为空
    }

    /// 处理键盘快捷键
    fn handle_shortcut(&mut self, _ctx: &egui::Context, _state: &mut AppState) -> bool {
        false // 默认不处理快捷键
    }

    /// 视图初始化
    fn initialize(&mut self, _state: &mut AppState) {
        // 默认实现为空
    }

    /// 视图清理
    fn cleanup(&mut self, _state: &mut AppState) {
        // 默认实现为空
    }
}

/// 视图状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewState {
    /// 正常状态
    Normal,
    /// 加载中
    Loading,
    /// 错误状态
    Error,
    /// 空数据状态
    Empty,
}

/// 视图配置
#[derive(Debug, Clone)]
pub struct ViewConfig {
    /// 是否显示工具栏
    pub show_toolbar: bool,
    /// 是否显示搜索框
    pub show_search: bool,
    /// 是否显示筛选器
    pub show_filters: bool,
    /// 是否显示分页
    pub show_pagination: bool,
    /// 每页显示数量
    pub items_per_page: usize,
    /// 是否自动刷新
    pub auto_refresh: bool,
    /// 刷新间隔（秒）
    pub refresh_interval: u64,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            show_toolbar: true,
            show_search: true,
            show_filters: true,
            show_pagination: true,
            items_per_page: 20,
            auto_refresh: false,
            refresh_interval: 30,
        }
    }
}

/// 通用视图组件
pub mod common {
    use super::*;
    use crate::gui::theme::Theme;

    /// 渲染加载指示器
    pub fn render_loading(ui: &mut egui::Ui, message: &str) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.spinner();
            ui.add_space(10.0);
            ui.label(message);
        });
    }

    /// 渲染错误消息
    pub fn render_error(ui: &mut egui::Ui, error: &str, theme: &Theme) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.colored_label(
                theme.get_color(crate::gui::theme::ColorType::Error),
                "⚠ 错误",
            );
            ui.add_space(10.0);
            ui.label(error);
            ui.add_space(10.0);
            if ui.button("重试").clicked() {
                // TODO: 实现重试逻辑
            }
        });
    }

    /// 渲染空数据状态
    pub fn render_empty(ui: &mut egui::Ui, message: &str, action_text: Option<&str>) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label("📭");
            ui.add_space(10.0);
            ui.label(message);

            if let Some(action) = action_text {
                ui.add_space(10.0);
                if ui.button(action).clicked() {
                    // TODO: 实现创建操作
                }
            }
        });
    }

    /// 渲染工具栏
    pub fn render_toolbar<F>(
        ui: &mut egui::Ui,
        title: &str,
        show_search: bool,
        search_text: &mut String,
        add_buttons: F,
    ) where
        F: FnOnce(&mut egui::Ui),
    {
        ui.horizontal(|ui| {
            // 标题
            ui.heading(title);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 自定义按钮
                add_buttons(ui);

                // 搜索框
                if show_search {
                    ui.add_space(10.0);
                    ui.add(
                        egui::TextEdit::singleline(search_text)
                            .hint_text("搜索...")
                            .desired_width(200.0),
                    );
                }
            });
        });

        ui.separator();
    }

    /// 渲染分页控件
    pub fn render_pagination(
        ui: &mut egui::Ui,
        current_page: &mut usize,
        total_pages: usize,
        total_items: usize,
        items_per_page: usize,
    ) {
        if total_pages <= 1 {
            return;
        }

        ui.separator();
        ui.horizontal(|ui| {
            // 页面信息
            let start_item = *current_page * items_per_page + 1;
            let end_item = ((*current_page + 1) * items_per_page).min(total_items);
            ui.label(format!(
                "显示 {}-{} 项，共 {} 项",
                start_item, end_item, total_items
            ));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 下一页按钮
                ui.add_enabled_ui(*current_page < total_pages - 1, |ui| {
                    if ui.button("下一页 >").clicked() {
                        *current_page += 1;
                    }
                });

                // 页码显示
                ui.label(format!("{} / {}", *current_page + 1, total_pages));

                // 上一页按钮
                ui.add_enabled_ui(*current_page > 0, |ui| {
                    if ui.button("< 上一页").clicked() {
                        *current_page -= 1;
                    }
                });
            });
        });
    }

    /// 渲染筛选器面板
    pub fn render_filters<F>(ui: &mut egui::Ui, show_filters: &mut bool, add_filters: F)
    where
        F: FnOnce(&mut egui::Ui),
    {
        ui.horizontal(|ui| {
            if ui
                .button(if *show_filters {
                    "隐藏筛选"
                } else {
                    "显示筛选"
                })
                .clicked()
            {
                *show_filters = !*show_filters;
            }
        });

        if *show_filters {
            ui.separator();
            egui::CollapsingHeader::new("筛选条件")
                .default_open(true)
                .show(ui, |ui| {
                    add_filters(ui);
                });
        }
    }

    /// 渲染数据表格
    pub fn render_table<T, F>(ui: &mut egui::Ui, items: &[T], headers: &[&str], render_row: F)
    where
        F: Fn(&mut egui::Ui, &T, usize),
    {
        if items.is_empty() {
            render_empty(ui, "暂无数据", None);
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                egui::Grid::new("data_table")
                    .striped(true)
                    .num_columns(headers.len())
                    .show(ui, |ui| {
                        // 渲染表头
                        for header in headers {
                            ui.strong(*header);
                        }
                        ui.end_row();

                        // 渲染数据行
                        for (index, item) in items.iter().enumerate() {
                            render_row(ui, item, index);
                            ui.end_row();
                        }
                    });
            });
    }

    /// 渲染卡片列表
    pub fn render_card_list<T, F>(ui: &mut egui::Ui, items: &[T], render_card: F)
    where
        F: Fn(&mut egui::Ui, &T, usize),
    {
        if items.is_empty() {
            render_empty(ui, "暂无数据", None);
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for (index, item) in items.iter().enumerate() {
                    egui::Frame::none()
                        .fill(ui.visuals().faint_bg_color)
                        .rounding(4.0)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            render_card(ui, item, index);
                        });

                    ui.add_space(4.0);
                }
            });
    }

    /// 渲染统计卡片
    pub fn render_stat_card(
        ui: &mut egui::Ui,
        title: &str,
        value: &str,
        icon: &str,
        color: egui::Color32,
    ) {
        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(color, icon);
                    ui.vertical(|ui| {
                        ui.label(title);
                        ui.heading(value);
                    });
                });
            });
    }

    /// 渲染进度环
    pub fn render_progress_ring(ui: &mut egui::Ui, progress: f32, size: f32, color: egui::Color32) {
        let (response, painter) =
            ui.allocate_painter(egui::Vec2::splat(size), egui::Sense::hover());

        let center = response.rect.center();
        let radius = size * 0.4;
        let stroke_width = size * 0.1;

        // 绘制背景圆环
        painter.circle_stroke(
            center,
            radius,
            egui::Stroke::new(stroke_width, ui.visuals().weak_text_color()),
        );

        // 绘制进度圆环
        if progress > 0.0 {
            let _angle = progress * 2.0 * std::f32::consts::PI;
            painter.circle_stroke(center, radius, egui::Stroke::new(stroke_width, color));
        }

        // 绘制进度文本
        let text = format!("{:.0}%", progress * 100.0);
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::default(),
            ui.visuals().text_color(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_config_default() {
        let config = ViewConfig::default();
        assert!(config.show_toolbar);
        assert!(config.show_search);
        assert!(config.show_filters);
        assert!(config.show_pagination);
        assert_eq!(config.items_per_page, 20);
        assert!(!config.auto_refresh);
        assert_eq!(config.refresh_interval, 30);
    }

    #[test]
    fn test_view_state() {
        assert_eq!(ViewState::Normal, ViewState::Normal);
        assert_ne!(ViewState::Normal, ViewState::Loading);
    }
}
