//! # 分类视图
//!
//! TimeTracker的分类管理界面，用于查看、创建、编辑和管理任务分类

use super::{common, View, ViewConfig, ViewState};
use crate::{
    core::{Category, CategoryColor, CategoryIcon},
    gui::{gui_utils, theme::ColorType, AppState},
    storage::models::*,
};
use chrono::{Duration as ChronoDuration, Local};
use eframe::egui;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// 分类视图
pub struct CategoriesView {
    /// 视图状态
    state: ViewState,
    /// 视图配置
    config: ViewConfig,
    /// 分类列表
    categories: Vec<Category>,
    /// 分类树结构
    category_tree: Vec<CategoryTreeNode>,
    /// 搜索文本
    search_text: String,
    /// 排序方式
    sort_by: CategorySortBy,
    /// 排序方向
    sort_ascending: bool,
    /// 选中的分类ID
    selected_category_id: Option<Uuid>,
    /// 是否显示分类详情
    show_category_details: bool,
    /// 是否显示创建分类对话框
    show_create_dialog: bool,
    /// 是否显示编辑分类对话框
    show_edit_dialog: bool,
    /// 是否显示删除确认对话框
    show_delete_dialog: bool,
    /// 新分类表单
    new_category_form: CategoryForm,
    /// 编辑分类表单
    edit_category_form: CategoryForm,
    /// 分类统计数据
    category_stats: Vec<CategoryStats>,
    /// 上次数据刷新时间
    last_refresh: Instant,
    /// 是否显示树形视图
    show_tree_view: bool,
    /// 展开的节点
    expanded_nodes: std::collections::HashSet<Uuid>,
}

/// 分类排序方式
#[derive(Debug, Clone, Copy, PartialEq)]
enum CategorySortBy {
    /// 按名称排序
    Name,
    /// 按创建时间排序
    CreatedAt,
    /// 按更新时间排序
    UpdatedAt,
    /// 按排序顺序
    SortOrder,
    /// 按任务数量排序
    TaskCount,
    /// 按总时长排序
    TotalTime,
}

/// 分类表单
#[derive(Debug, Clone, PartialEq)]
struct CategoryForm {
    /// 分类名称
    name: String,
    /// 分类描述
    description: String,
    /// 分类颜色
    color: CategoryColor,
    /// 分类图标
    icon: CategoryIcon,
    /// 父分类ID
    parent_id: Option<Uuid>,
    /// 目标时长（小时）
    target_hours: f32,
    /// 是否激活
    is_active: bool,
    /// 排序顺序
    sort_order: i32,
}

impl Default for CategoryForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            color: CategoryColor::Blue,
            icon: CategoryIcon::Other,
            parent_id: None,
            target_hours: 0.0,
            is_active: true,
            sort_order: 0,
        }
    }
}

/// 分类树节点
#[derive(Debug, Clone)]
struct CategoryTreeNode {
    /// 分类信息
    category: Category,
    /// 子分类
    children: Vec<CategoryTreeNode>,
    /// 层级深度
    depth: usize,
    /// 统计信息
    stats: Option<CategoryStats>,
}

impl Default for CategoriesView {
    fn default() -> Self {
        Self::new()
    }
}

impl CategoriesView {
    /// 创建新的分类视图
    pub fn new() -> Self {
        Self {
            state: ViewState::Normal,
            config: ViewConfig {
                auto_refresh: true,
                refresh_interval: 15, // 15秒刷新一次
                ..ViewConfig::default()
            },
            categories: Vec::new(),
            category_tree: Vec::new(),
            search_text: String::new(),
            sort_by: CategorySortBy::SortOrder,
            sort_ascending: true,
            selected_category_id: None,
            show_category_details: false,
            show_create_dialog: false,
            show_edit_dialog: false,
            show_delete_dialog: false,
            new_category_form: CategoryForm::default(),
            edit_category_form: CategoryForm::default(),
            category_stats: Vec::new(),
            last_refresh: Instant::now(),
            show_tree_view: true,
            expanded_nodes: std::collections::HashSet::new(),
        }
    }

    /// 刷新分类数据
    fn refresh_data(&mut self, state: &AppState) {
        self.state = ViewState::Loading;

        if let Ok(core) = state.core.lock() {
            // 获取分类列表
            match core.get_categories() {
                Ok(categories) => {
                    self.categories = self.apply_filters_and_sort(categories);
                    self.build_category_tree();
                }
                Err(e) => {
                    log::error!("获取分类列表失败: {}", e);
                    self.state = ViewState::Error;
                    return;
                }
            }

            // 获取分类统计
            let today = Local::now().date_naive();
            let week_start = today - ChronoDuration::days(7);

            if let Ok(storage) = state.storage.lock() {
                match storage.get_category_stats(week_start, today) {
                    Ok(stats) => self.category_stats = stats,
                    Err(e) => {
                        log::error!("获取分类统计失败: {}", e);
                    }
                }
            }
        }

        self.state = ViewState::Normal;
        self.last_refresh = Instant::now();
    }

    /// 应用筛选和排序
    fn apply_filters_and_sort(&self, mut categories: Vec<Category>) -> Vec<Category> {
        // 应用搜索筛选
        if !self.search_text.is_empty() {
            let search_lower = self.search_text.to_lowercase();
            categories.retain(|category| {
                category.name.to_lowercase().contains(&search_lower)
                    || category
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&search_lower))
                        .unwrap_or(false)
            });
        }

        // 应用排序
        categories.sort_by(|a, b| {
            let ordering = match self.sort_by {
                CategorySortBy::Name => a.name.cmp(&b.name),
                CategorySortBy::CreatedAt => a.created_at.cmp(&b.created_at),
                CategorySortBy::UpdatedAt => a.updated_at.cmp(&b.updated_at),
                CategorySortBy::SortOrder => a.sort_order.cmp(&b.sort_order),
                CategorySortBy::TaskCount => {
                    let a_count = self.get_category_task_count(a.id);
                    let b_count = self.get_category_task_count(b.id);
                    a_count.cmp(&b_count)
                }
                CategorySortBy::TotalTime => {
                    let a_time = self.get_category_total_time(a.id);
                    let b_time = self.get_category_total_time(b.id);
                    a_time.cmp(&b_time)
                }
            };

            if self.sort_ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });

        categories
    }

    /// 构建分类树
    fn build_category_tree(&mut self) {
        self.category_tree.clear();

        // 找到根分类（没有父分类的分类）
        let root_categories: Vec<_> = self
            .categories
            .iter()
            .filter(|c| c.parent_id.is_none())
            .cloned()
            .collect();

        for root_category in root_categories {
            let node = self.build_tree_node(root_category, 0);
            self.category_tree.push(node);
        }
    }

    /// 构建树节点
    fn build_tree_node(&self, category: Category, depth: usize) -> CategoryTreeNode {
        let children: Vec<_> = self
            .categories
            .iter()
            .filter(|c| c.parent_id == Some(category.id))
            .map(|c| self.build_tree_node(c.clone(), depth + 1))
            .collect();

        let stats = self
            .category_stats
            .iter()
            .find(|s| s.category_id == category.id)
            .cloned();

        CategoryTreeNode {
            category,
            children,
            depth,
            stats,
        }
    }

    /// 获取分类任务数量
    fn get_category_task_count(&self, category_id: Uuid) -> usize {
        // 获取分类的任务统计
        self.category_stats
            .iter()
            .find(|s| s.category_id == category_id)
            .map(|s| s.task_count)
            .unwrap_or(0)
    }

    /// 获取分类总时长
    fn get_category_total_time(&self, category_id: Uuid) -> i64 {
        // 获取分类的总时间
        self.category_stats
            .iter()
            .find(|s| s.category_id == category_id)
            .map(|s| s.total_seconds)
            .unwrap_or(0)
    }

    /// 创建新分类
    fn create_category(&mut self, state: &AppState) {
        if let Ok(mut core) = state.core.lock() {
            let result = core.category_manager.create_category(
                self.new_category_form.name.clone(),
                if self.new_category_form.description.is_empty() {
                    None
                } else {
                    Some(self.new_category_form.description.clone())
                },
                Some(self.new_category_form.color.clone()),
                Some(self.new_category_form.icon),
            );

            match result {
                Ok(_) => {
                    self.new_category_form = CategoryForm::default();
                    self.show_create_dialog = false;
                    self.refresh_data(state);
                    log::info!("分类创建成功");
                }
                Err(e) => {
                    log::error!("创建分类失败: {}", e);
                }
            }
        }
    }

    /// 更新分类
    fn update_category(&mut self, state: &AppState) {
        if let Some(category_id) = self.selected_category_id {
            if let Ok(mut core) = state.core.lock() {
                let target_duration = if self.edit_category_form.target_hours > 0.0 {
                    Some(ChronoDuration::minutes(
                        (self.edit_category_form.target_hours * 60.0) as i64,
                    ))
                } else {
                    None
                };

                // 记录目标时长信息
                if let Some(duration) = target_duration {
                    log::info!("设置分类目标时长: {} 分钟", duration.num_minutes());
                } else {
                    log::info!("清除分类目标时长");
                }

                match core.category_manager.update_category(
                    category_id,
                    Some(self.edit_category_form.name.clone()),
                    Some(self.edit_category_form.description.clone()),
                    Some(self.edit_category_form.color.clone()),
                    Some(self.edit_category_form.icon),
                ) {
                    Ok(_) => {
                        self.show_edit_dialog = false;
                        self.refresh_data(state);
                        log::info!("分类更新成功");
                    }
                    Err(e) => {
                        log::error!("更新分类失败: {}", e);
                    }
                }
            }
        }
    }

    /// 删除分类
    fn delete_category(&mut self, state: &AppState) {
        if let Some(category_id) = self.selected_category_id {
            if let Ok(mut core) = state.core.lock() {
                match core.category_manager.remove_category(category_id) {
                    Ok(_) => {
                        self.show_delete_dialog = false;
                        self.selected_category_id = None;
                        self.refresh_data(state);
                        log::info!("分类删除成功");
                    }
                    Err(e) => {
                        log::error!("删除分类失败: {}", e);
                    }
                }
            }
        }
    }

    /// 渲染工具栏
    fn render_toolbar(&mut self, ui: &mut egui::Ui, state: &AppState) {
        ui.horizontal(|ui| {
            // 创建分类按钮
            if ui.button("➕ 新建分类").clicked() {
                self.new_category_form = CategoryForm::default();
                self.show_create_dialog = true;
            }

            ui.separator();

            // 视图切换
            ui.label("视图:");
            ui.selectable_value(&mut self.show_tree_view, true, "🌳 树形");
            ui.selectable_value(&mut self.show_tree_view, false, "📋 列表");

            ui.separator();

            // 搜索框
            ui.label("搜索:");
            if ui
                .add(
                    egui::TextEdit::singleline(&mut self.search_text)
                        .hint_text("搜索分类...")
                        .desired_width(200.0),
                )
                .changed()
            {
                self.refresh_data(state);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 刷新按钮
                if ui.button("🔄").on_hover_text("刷新").clicked() {
                    self.refresh_data(state);
                }

                // 排序选项
                ui.label("排序:");
                egui::ComboBox::from_id_source("category_sort")
                    .selected_text(match self.sort_by {
                        CategorySortBy::Name => "名称",
                        CategorySortBy::CreatedAt => "创建时间",
                        CategorySortBy::UpdatedAt => "更新时间",
                        CategorySortBy::SortOrder => "排序",
                        CategorySortBy::TaskCount => "任务数",
                        CategorySortBy::TotalTime => "总时长",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.sort_by, CategorySortBy::Name, "名称");
                        ui.selectable_value(
                            &mut self.sort_by,
                            CategorySortBy::CreatedAt,
                            "创建时间",
                        );
                        ui.selectable_value(
                            &mut self.sort_by,
                            CategorySortBy::UpdatedAt,
                            "更新时间",
                        );
                        ui.selectable_value(&mut self.sort_by, CategorySortBy::SortOrder, "排序");
                        ui.selectable_value(&mut self.sort_by, CategorySortBy::TaskCount, "任务数");
                        ui.selectable_value(&mut self.sort_by, CategorySortBy::TotalTime, "总时长");
                    });

                if ui
                    .button(if self.sort_ascending { "↑" } else { "↓" })
                    .clicked()
                {
                    self.sort_ascending = !self.sort_ascending;
                    self.refresh_data(state);
                }
            });
        });
    }

    /// 渲染分类列表
    fn render_category_list(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        if self.categories.is_empty() {
            common::render_empty(ui, "暂无分类", Some("创建第一个分类"));
            return;
        }

        if self.show_tree_view {
            self.render_tree_view(ui, state);
        } else {
            self.render_list_view(ui, state);
        }
    }

    /// 渲染树形视图
    fn render_tree_view(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for node in &self.category_tree.clone() {
                self.render_tree_node(ui, node, state);
            }
        });
    }

    /// 渲染树节点
    fn render_tree_node(
        &mut self,
        ui: &mut egui::Ui,
        node: &CategoryTreeNode,
        state: &mut AppState,
    ) {
        let indent = node.depth as f32 * 20.0;

        ui.horizontal(|ui| {
            ui.add_space(indent);

            // 展开/折叠按钮
            if !node.children.is_empty() {
                let is_expanded = self.expanded_nodes.contains(&node.category.id);
                let button_text = if is_expanded { "▼" } else { "▶" };

                if ui.small_button(button_text).clicked() {
                    if is_expanded {
                        self.expanded_nodes.remove(&node.category.id);
                    } else {
                        self.expanded_nodes.insert(node.category.id);
                    }
                }
            } else {
                ui.add_space(20.0);
            }

            // 分类信息
            self.render_category_item(ui, &node.category, node.stats.as_ref(), state);
        });

        // 渲染子节点
        if self.expanded_nodes.contains(&node.category.id) {
            for child in &node.children {
                self.render_tree_node(ui, child, state);
            }
        }
    }

    /// 渲染列表视图
    fn render_list_view(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let categories = self.categories.clone();
            for category in &categories {
                let stats = self
                    .category_stats
                    .iter()
                    .find(|s| s.category_id == category.id)
                    .cloned();

                self.render_category_item(ui, category, stats.as_ref(), state);
                ui.separator();
            }
        });
    }

    /// 渲染分类项
    fn render_category_item(
        &mut self,
        ui: &mut egui::Ui,
        category: &Category,
        stats: Option<&CategoryStats>,
        state: &mut AppState,
    ) {
        ui.horizontal(|ui| {
            // 选择框
            let mut is_selected = self.selected_category_id == Some(category.id);
            if ui.checkbox(&mut is_selected, "").changed() {
                self.selected_category_id = if is_selected { Some(category.id) } else { None };
            }

            // 颜色指示器
            let color = self.get_category_color(category.color.clone());
            ui.colored_label(color, "●");

            // 图标
            ui.label(self.get_category_icon_text(category.icon));

            // 分类信息
            ui.vertical(|ui| {
                // 分类名称
                ui.strong(&category.name);

                // 分类描述
                if let Some(description) = &category.description {
                    if !description.is_empty() {
                        ui.label(
                            egui::RichText::new(description)
                                .size(12.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                    }
                }

                // 统计信息
                if let Some(stats) = stats {
                    ui.horizontal(|ui| {
                        ui.label(format!("任务: {}", stats.task_count));
                        ui.label("|");
                        ui.label(format!(
                            "时长: {}",
                            gui_utils::format_duration(stats.total_seconds)
                        ));

                        if let Some(target) = category.target_duration {
                            ui.label("|");
                            let progress = if target.num_seconds() > 0 {
                                (stats.total_seconds as f32 / target.num_seconds() as f32) * 100.0
                            } else {
                                0.0
                            };
                            ui.label(format!("进度: {:.1}%", progress));
                        }
                    });
                }

                // 更新时间
                ui.label(
                    egui::RichText::new(category.updated_at.format("%Y-%m-%d %H:%M").to_string())
                        .size(10.0)
                        .color(ui.visuals().weak_text_color()),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 操作按钮
                if ui.small_button("🗑").on_hover_text("删除").clicked() {
                    self.selected_category_id = Some(category.id);
                    self.show_delete_dialog = true;
                }

                if ui.small_button("✏").on_hover_text("编辑").clicked() {
                    self.selected_category_id = Some(category.id);
                    self.edit_category_form = CategoryForm {
                        name: category.name.clone(),
                        description: category.description.clone().unwrap_or_default(),
                        color: category.color.clone(),
                        icon: category.icon,
                        parent_id: category.parent_id,
                        target_hours: category
                            .target_duration
                            .map(|d| d.num_minutes() as f32 / 60.0)
                            .unwrap_or(0.0),
                        is_active: category.is_active,
                        sort_order: category.sort_order,
                    };
                    self.show_edit_dialog = true;
                }

                // 激活状态
                if category.is_active {
                    ui.colored_label(state.theme.get_color(ColorType::Success), "✓");
                } else {
                    ui.colored_label(state.theme.get_color(ColorType::Error), "✗");
                }
            });
        });
    }

    /// 获取分类颜色
    fn get_category_color(&self, color: CategoryColor) -> egui::Color32 {
        match color {
            CategoryColor::Red => egui::Color32::from_rgb(255, 99, 99),
            CategoryColor::Green => egui::Color32::from_rgb(99, 255, 99),
            CategoryColor::Blue => egui::Color32::from_rgb(99, 99, 255),
            CategoryColor::Yellow => egui::Color32::from_rgb(255, 255, 99),
            CategoryColor::Purple => egui::Color32::from_rgb(255, 99, 255),
            CategoryColor::Orange => egui::Color32::from_rgb(255, 165, 0),
            CategoryColor::Pink => egui::Color32::from_rgb(255, 192, 203),
            CategoryColor::Cyan => egui::Color32::from_rgb(0, 255, 255),
            CategoryColor::Gray => egui::Color32::from_rgb(128, 128, 128),
            CategoryColor::Custom(hex) => {
                // 简单的十六进制颜色解析
                let hex = hex.trim_start_matches('#');
                if hex.len() >= 6 {
                    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
                    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
                    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
                    egui::Color32::from_rgb(r, g, b)
                } else {
                    egui::Color32::from_rgb(128, 128, 128)
                }
            }
        }
    }

    /// 获取分类图标文本
    fn get_category_icon_text(&self, icon: CategoryIcon) -> &'static str {
        match icon {
            CategoryIcon::Work => "💼",
            CategoryIcon::Study => "📚",
            CategoryIcon::Personal => "👤",
            CategoryIcon::Health => "🏥",
            CategoryIcon::Entertainment => "🎮",
            CategoryIcon::Travel => "✈️",
            CategoryIcon::Shopping => "🛒",
            CategoryIcon::Food => "🍽️",
            CategoryIcon::Exercise => "🏃",
            CategoryIcon::Meeting => "👥",
            CategoryIcon::Project => "📋",
            CategoryIcon::Research => "🔬",
            CategoryIcon::Writing => "✍️",
            CategoryIcon::Design => "🎨",
            CategoryIcon::Development => "💻",
            CategoryIcon::Other => "📁",
            CategoryIcon::Household => "🏠",
            CategoryIcon::Social => "👥",
            CategoryIcon::Creative => "🎨",
        }
    }

    /// 渲染对话框
    fn render_dialogs(&mut self, ctx: &egui::Context, state: &mut AppState) {
        // 创建分类对话框
        if self.show_create_dialog {
            egui::Window::new("创建分类")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    // 提取表单引用避免借用冲突
                    let mut form = self.new_category_form.clone();
                    Self::render_category_form_static(ui, &mut form, "新建分类");

                    // 更新表单数据
                    self.new_category_form = form;

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if ui.button("创建").clicked() {
                            self.create_category(state);
                        }

                        if ui.button("取消").clicked() {
                            self.show_create_dialog = false;
                        }
                    });
                });
        }

        // 编辑分类对话框
        if self.show_edit_dialog {
            egui::Window::new("编辑分类")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    // 提取表单引用避免借用冲突
                    let mut form = self.edit_category_form.clone();
                    Self::render_category_form_static(ui, &mut form, "编辑分类");

                    // 更新表单数据
                    self.edit_category_form = form;

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if ui.button("保存").clicked() {
                            self.update_category(state);
                        }

                        if ui.button("取消").clicked() {
                            self.show_edit_dialog = false;
                        }
                    });
                });
        }

        // 删除确认对话框
        if self.show_delete_dialog {
            egui::Window::new("确认删除")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("确定要删除这个分类吗？此操作不可撤销。");
                    ui.label("注意：删除分类会影响相关的任务。");

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if ui.button("删除").clicked() {
                            self.delete_category(state);
                        }

                        if ui.button("取消").clicked() {
                            self.show_delete_dialog = false;
                        }
                    });
                });
        }
    }

    /// 静态方法渲染分类表单，避免借用冲突
    fn render_category_form_static(ui: &mut egui::Ui, form: &mut CategoryForm, title: &str) {
        ui.heading(title);
        ui.separator();

        // 分类名称
        ui.horizontal(|ui| {
            ui.label("名称:");
            ui.add(
                egui::TextEdit::singleline(&mut form.name)
                    .hint_text("输入分类名称...")
                    .desired_width(300.0),
            );
        });

        ui.add_space(5.0);

        // 分类描述
        ui.horizontal(|ui| {
            ui.label("描述:");
            ui.add(
                egui::TextEdit::multiline(&mut form.description)
                    .hint_text("输入分类描述...")
                    .desired_width(300.0)
                    .desired_rows(3),
            );
        });

        ui.add_space(5.0);

        // 分类颜色
        ui.horizontal(|ui| {
            ui.label("颜色:");
            egui::ComboBox::from_id_source("category_color")
                .selected_text(format!("{:?}", form.color))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut form.color, CategoryColor::Red, "红色");
                    ui.selectable_value(&mut form.color, CategoryColor::Green, "绿色");
                    ui.selectable_value(&mut form.color, CategoryColor::Blue, "蓝色");
                    ui.selectable_value(&mut form.color, CategoryColor::Yellow, "黄色");
                    ui.selectable_value(&mut form.color, CategoryColor::Purple, "紫色");
                    ui.selectable_value(&mut form.color, CategoryColor::Orange, "橙色");
                    ui.selectable_value(&mut form.color, CategoryColor::Pink, "粉色");
                    ui.selectable_value(&mut form.color, CategoryColor::Cyan, "青色");
                    ui.selectable_value(&mut form.color, CategoryColor::Gray, "灰色");
                });
        });

        ui.add_space(5.0);

        // 分类图标
        ui.horizontal(|ui| {
            ui.label("图标:");
            egui::ComboBox::from_id_source("category_icon")
                .selected_text(Self::get_category_icon_text_static(form.icon))
                .show_ui(ui, |ui| {
                    let icons = [
                        (CategoryIcon::Work, "💼 工作"),
                        (CategoryIcon::Study, "📚 学习"),
                        (CategoryIcon::Personal, "👤 个人"),
                        (CategoryIcon::Health, "🏥 健康"),
                        (CategoryIcon::Entertainment, "🎮 娱乐"),
                        (CategoryIcon::Travel, "✈️ 旅行"),
                        (CategoryIcon::Shopping, "🛒 购物"),
                        (CategoryIcon::Food, "🍽️ 饮食"),
                        (CategoryIcon::Exercise, "🏃 运动"),
                        (CategoryIcon::Meeting, "👥 会议"),
                        (CategoryIcon::Project, "📋 项目"),
                        (CategoryIcon::Research, "🔬 研究"),
                        (CategoryIcon::Writing, "✍️ 写作"),
                        (CategoryIcon::Design, "🎨 设计"),
                        (CategoryIcon::Development, "💻 开发"),
                        (CategoryIcon::Household, "🏠 家务"),
                        (CategoryIcon::Social, "👥 社交"),
                        (CategoryIcon::Creative, "🎨 创意"),
                        (CategoryIcon::Other, "📁 其他"),
                    ];

                    for (icon, label) in icons {
                        ui.selectable_value(&mut form.icon, icon, label);
                    }
                });
        });

        ui.add_space(5.0);

        // 目标时长
        ui.horizontal(|ui| {
            ui.label("目标时长(小时):");
            ui.add(
                egui::DragValue::new(&mut form.target_hours)
                    .range(0.0..=24.0)
                    .suffix("小时"),
            );
        });

        ui.add_space(5.0);

        // 排序顺序
        ui.horizontal(|ui| {
            ui.label("排序顺序:");
            ui.add(egui::DragValue::new(&mut form.sort_order).range(0..=1000));
        });

        ui.add_space(5.0);

        // 激活状态
        ui.checkbox(&mut form.is_active, "激活");
    }

    /// 静态方法获取分类图标文本
    fn get_category_icon_text_static(icon: CategoryIcon) -> &'static str {
        match icon {
            CategoryIcon::Work => "💼",
            CategoryIcon::Study => "📚",
            CategoryIcon::Personal => "👤",
            CategoryIcon::Health => "🏥",
            CategoryIcon::Entertainment => "🎮",
            CategoryIcon::Travel => "✈️",
            CategoryIcon::Shopping => "🛒",
            CategoryIcon::Food => "🍽️",
            CategoryIcon::Exercise => "🏃",
            CategoryIcon::Meeting => "👥",
            CategoryIcon::Project => "📋",
            CategoryIcon::Research => "🔬",
            CategoryIcon::Writing => "✍️",
            CategoryIcon::Design => "🎨",
            CategoryIcon::Development => "💻",
            CategoryIcon::Other => "📁",
            CategoryIcon::Household => "🏠",
            CategoryIcon::Social => "👥",
            CategoryIcon::Creative => "��",
        }
    }
}

impl View for CategoriesView {
    fn render(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        // 检查是否需要刷新数据
        if self.config.auto_refresh
            && self.last_refresh.elapsed() >= Duration::from_secs(self.config.refresh_interval)
        {
            self.refresh_data(state);
        }

        match self.state {
            ViewState::Loading => {
                common::render_loading(ui, "加载分类数据...");
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

            // 分类列表
            self.render_category_list(ui, state);
        });

        // 渲染对话框
        self.render_dialogs(ui.ctx(), state);
    }

    fn title(&self) -> &str {
        "分类管理"
    }

    fn needs_refresh(&self) -> bool {
        self.config.auto_refresh
            && self.last_refresh.elapsed() >= Duration::from_secs(self.config.refresh_interval)
    }

    fn refresh(&mut self, state: &mut AppState) {
        self.refresh_data(state);
    }

    fn handle_shortcut(&mut self, ctx: &egui::Context, state: &mut AppState) -> bool {
        // Ctrl+N: 新建分类
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::N)) {
            self.new_category_form = CategoryForm::default();
            self.show_create_dialog = true;
            return true;
        }

        // Delete: 删除选中分类
        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) && self.selected_category_id.is_some() {
            self.show_delete_dialog = true;
            return true;
        }

        // F5: 刷新
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.refresh_data(state);
            return true;
        }

        // Tab: 切换视图
        if ctx.input(|i| i.key_pressed(egui::Key::Tab)) {
            self.show_tree_view = !self.show_tree_view;
            return true;
        }

        false
    }

    fn initialize(&mut self, state: &mut AppState) {
        self.refresh_data(state);
        // 默认展开所有根节点
        for category in &self.categories {
            if category.parent_id.is_none() {
                self.expanded_nodes.insert(category.id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categories_view_creation() {
        let view = CategoriesView::new();
        assert_eq!(view.title(), "分类管理");
        assert_eq!(view.state, ViewState::Normal);
        assert!(view.config.auto_refresh);
        assert_eq!(view.config.refresh_interval, 15);
        assert!(view.show_tree_view);
    }

    #[test]
    fn test_category_sort_by() {
        assert_eq!(CategorySortBy::Name, CategorySortBy::Name);
        assert_ne!(CategorySortBy::Name, CategorySortBy::CreatedAt);
    }

    #[test]
    fn test_category_form_default() {
        let form = CategoryForm::default();
        assert!(form.name.is_empty());
        assert!(form.description.is_empty());
        assert!(form.is_active);
        assert_eq!(form.sort_order, 0);
    }
}
