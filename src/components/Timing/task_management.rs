//! # 任务管理组件
//!
//! 包含任务CRUD操作、列表展示、模态框等功能

use dioxus::prelude::*;
use life_tracker::get_app_state_sync;
use life_tracker::storage::models::CategoryModel;
use life_tracker::storage::task_models::TaskModel;
use std::collections::HashSet;

/// 任务表单数据
#[derive(Clone, Debug, Default)]
pub struct TaskFormData {
    pub name: String,
    pub description: String,
    pub category_id: Option<uuid::Uuid>,
    pub tags: String, // 逗号分隔的标签
    pub priority: String,
}

/// 任务管理主组件
#[component]
pub fn TaskManagementContent() -> Element {
    // 状态管理
    let mut show_create_form = use_signal(|| false);
    let mut editing_task = use_signal(|| None::<TaskModel>);
    let mut error_message = use_signal(|| None::<String>);
    let mut success_message = use_signal(|| None::<String>);
    let mut search_query = use_signal(String::new);
    let mut selected_category = use_signal(|| None::<uuid::Uuid>);
    let mut expanded_tasks = use_signal(|| HashSet::<uuid::Uuid>::new());

    // 同步获取应用状态和数据
    let app_state = get_app_state_sync();

    // 获取任务列表
    let tasks = if let Some(database) = app_state.get_database() {
        match database.get_all_tasks() {
            Ok(tasks) => tasks,
            Err(e) => {
                log::error!("获取任务失败: {}", e);
                error_message.set(Some(format!("获取任务失败: {}", e)));
                Vec::new()
            }
        }
    } else {
        error_message.set(Some("数据库未初始化".to_string()));
        Vec::new()
    };

    // 获取分类列表
    let categories = if let Some(database) = app_state.get_database() {
        match database.get_all_categories() {
            Ok(categories) => categories,
            Err(e) => {
                log::error!("获取分类失败: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    // 过滤任务
    let filtered_tasks: Vec<TaskModel> = tasks
        .into_iter()
        .filter(|task| {
            // 名称搜索过滤
            let name_match = search_query.read().is_empty()
                || task
                    .name
                    .to_lowercase()
                    .contains(&search_query.read().to_lowercase())
                || task.description.as_ref().map_or(false, |desc| {
                    desc.to_lowercase()
                        .contains(&search_query.read().to_lowercase())
                });

            // 分类过滤
            let category_match =
                selected_category.read().is_none() || task.category_id == *selected_category.read();

            name_match && category_match
        })
        .collect();

    // 创建任务功能
    let mut create_task = move |form_data: TaskFormData| {
        if form_data.name.trim().is_empty() {
            error_message.set(Some("任务名称不能为空".to_string()));
            return;
        }

        let app_state = get_app_state_sync();
        if let Some(database) = app_state.get_database() {
            // 解析标签
            let tags_json = if form_data.tags.trim().is_empty() {
                "[]".to_string()
            } else {
                let tags: Vec<String> = form_data
                    .tags
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string())
            };

            // 创建任务数据
            let task = life_tracker::storage::task_models::TaskInsert {
                id: uuid::Uuid::new_v4(),
                name: form_data.name.trim().to_string(),
                description: if form_data.description.trim().is_empty() {
                    None
                } else {
                    Some(form_data.description.trim().to_string())
                },
                category_id: form_data.category_id,
                status: "pending".to_string(),
                priority: form_data.priority,
                estimated_duration_seconds: None,
                total_duration_seconds: 0,
                tags: tags_json,
                due_date: None,
                is_completed: false,
                completed_at: None,
                created_at: chrono::Local::now(),
            };

            // 执行数据库插入
            match database.insert_task(&task) {
                Ok(_) => {
                    log::info!("任务创建成功: {}", task.name);
                    success_message.set(Some("任务创建成功".to_string()));
                    error_message.set(None);
                    show_create_form.set(false);
                }
                Err(e) => {
                    log::error!("任务创建失败: {}", e);
                    error_message.set(Some(format!("任务创建失败: {}", e)));
                    success_message.set(None);
                }
            }
        } else {
            error_message.set(Some("数据库未初始化".to_string()));
        }
    };

    // 更新任务功能
    let mut update_task = move |task_id: uuid::Uuid, form_data: TaskFormData| {
        if form_data.name.trim().is_empty() {
            error_message.set(Some("任务名称不能为空".to_string()));
            return;
        }

        let app_state = get_app_state_sync();
        if let Some(database) = app_state.get_database() {
            // 解析标签
            let tags_json = if form_data.tags.trim().is_empty() {
                "[]".to_string()
            } else {
                let tags: Vec<String> = form_data
                    .tags
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string())
            };

            let update = life_tracker::storage::task_models::TaskUpdate {
                name: Some(form_data.name.trim().to_string()),
                description: Some(if form_data.description.trim().is_empty() {
                    None
                } else {
                    Some(form_data.description.trim().to_string())
                }),
                category_id: Some(form_data.category_id),
                priority: Some(form_data.priority),
                tags: Some(tags_json),
                ..Default::default()
            };

            match database.update_task(task_id, &update) {
                Ok(_) => {
                    log::info!("任务更新成功: {}", task_id);
                    success_message.set(Some("任务更新成功".to_string()));
                    error_message.set(None);
                    editing_task.set(None);
                }
                Err(e) => {
                    log::error!("任务更新失败: {}", e);
                    error_message.set(Some(format!("任务更新失败: {}", e)));
                    success_message.set(None);
                }
            }
        } else {
            error_message.set(Some("数据库未初始化".to_string()));
        }
    };

    // 删除任务功能
    let mut delete_task = move |task_id: uuid::Uuid| {
        let app_state = get_app_state_sync();
        if let Some(database) = app_state.get_database() {
            match database.delete_task(task_id) {
                Ok(_) => {
                    log::info!("任务删除成功: {}", task_id);
                    success_message.set(Some("任务删除成功".to_string()));
                    error_message.set(None);
                }
                Err(e) => {
                    log::error!("任务删除失败: {}", e);
                    error_message.set(Some(format!("任务删除失败: {}", e)));
                    success_message.set(None);
                }
            }
        } else {
            error_message.set(Some("数据库未初始化".to_string()));
        }
    };

    // 切换任务展开状态
    let mut toggle_expanded = move |task_id: uuid::Uuid| {
        let mut expanded = expanded_tasks.write();
        if expanded.contains(&task_id) {
            expanded.remove(&task_id);
        } else {
            expanded.insert(task_id);
        }
    };

    rsx! {
        div {
            class: "space-y-6",

            // 页面标题和操作
            div {
                class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-6",
                div {
                    class: "flex items-center justify-between mb-4",
                    h2 { class: "text-2xl font-bold text-gray-900 dark:text-white", "任务管理" }
                    button {
                        class: "flex items-center space-x-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors shadow-md",
                        onclick: move |_| {
                            show_create_form.set(true);
                            error_message.set(None);
                            success_message.set(None);
                        },
                        span { class: "text-lg", "➕" }
                        span { "新建任务" }
                    }
                }

                // 搜索和过滤栏
                div {
                    class: "flex flex-col sm:flex-row gap-4 mb-4",

                    // 搜索框
                    div {
                        class: "flex-1",
                        input {
                            r#type: "text",
                            class: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            placeholder: "搜索任务名称或描述...",
                            value: "{search_query.read()}",
                            oninput: move |e| search_query.set(e.value())
                        }
                    }

                    // 分类过滤器
                    div {
                        class: "sm:w-48",
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            onchange: move |e| {
                                if e.value().is_empty() {
                                    selected_category.set(None);
                                } else if let Ok(uuid) = uuid::Uuid::parse_str(&e.value()) {
                                    selected_category.set(Some(uuid));
                                }
                            },
                            option { value: "", "所有分类" }
                            for category in categories.iter() {
                                option {
                                    value: "{category.id}",
                                    selected: selected_category.read().as_ref().map(|id| *id == category.id).unwrap_or(false),
                                    "{category.name}"
                                }
                            }
                        }
                    }

                    // 清除过滤器按钮
                    if !search_query.read().is_empty() || selected_category.read().is_some() {
                        button {
                            class: "px-3 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors",
                            onclick: move |_| {
                                search_query.set(String::new());
                                selected_category.set(None);
                            },
                            "🔄 清除"
                        }
                    }
                }

                // 任务数量统计
                div {
                    class: "text-sm text-gray-600 dark:text-gray-400",
                    "显示 {filtered_tasks.len()} 个任务"
                }
            }

            // 成功消息显示
            if let Some(success) = success_message.read().as_ref() {
                div {
                    class: "bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg flex items-center space-x-2",
                    span { class: "text-lg", "✅" }
                    span { "{success}" }
                    button {
                        class: "ml-auto text-green-500 hover:text-green-700",
                        onclick: move |_| success_message.set(None),
                        "×"
                    }
                }
            }

            // 错误信息显示
            if let Some(error) = error_message.read().as_ref() {
                div {
                    class: "bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg flex items-center space-x-2",
                    span { class: "text-lg", "❌" }
                    span { "{error}" }
                    button {
                        class: "ml-auto text-red-500 hover:text-red-700",
                        onclick: move |_| error_message.set(None),
                        "×"
                    }
                }
            }

            // 任务列表
            EnhancedTaskList {
                tasks: filtered_tasks.clone(),
                categories: categories.clone(),
                expanded_tasks: expanded_tasks.read().clone(),
                on_delete: move |task_id| delete_task(task_id),
                on_edit: move |task| editing_task.set(Some(task)),
                on_toggle_expanded: move |task_id| toggle_expanded(task_id)
            }

            // 任务创建表单模态框
            if show_create_form() {
                EnhancedTaskModal {
                    task: None,
                    categories: categories.clone(),
                    on_submit: move |form_data| create_task(form_data),
                    on_cancel: move |_| {
                        show_create_form.set(false);
                        error_message.set(None);
                        success_message.set(None);
                    }
                }
            }

            // 任务编辑表单模态框
            if let Some(task) = editing_task.read().clone() {
                EnhancedTaskModal {
                    task: Some(task.clone()),
                    categories: categories.clone(),
                    on_submit: move |form_data| update_task(task.id, form_data),
                    on_cancel: move |_| {
                        editing_task.set(None);
                        error_message.set(None);
                        success_message.set(None);
                    }
                }
            }
        }
    }
}

/// 增强版任务列表组件
#[component]
fn EnhancedTaskList(
    tasks: Vec<TaskModel>,
    categories: Vec<life_tracker::storage::models::CategoryModel>,
    expanded_tasks: HashSet<uuid::Uuid>,
    on_delete: EventHandler<uuid::Uuid>,
    on_edit: EventHandler<TaskModel>,
    on_toggle_expanded: EventHandler<uuid::Uuid>,
) -> Element {
    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 overflow-hidden",

            // 列表头部
            div {
                class: "px-6 py-4 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-750",
                h3 { class: "text-lg font-semibold text-gray-900 dark:text-white", "任务列表" }
            }

            // 任务列表内容
            if tasks.is_empty() {
                div {
                    class: "p-12 text-center",
                    div { class: "text-6xl mb-4", "📝" }
                    h4 { class: "text-lg font-medium text-gray-900 dark:text-white mb-2", "暂无任务" }
                    p { class: "text-gray-600 dark:text-gray-400", "点击上方按钮创建第一个任务" }
                }
            } else {
                div {
                    class: "divide-y divide-gray-200 dark:divide-gray-700",
                    for task in tasks.iter() {
                        EnhancedTaskItem {
                            key: "{task.id}",
                            task: task.clone(),
                            categories: categories.clone(),
                            is_expanded: expanded_tasks.contains(&task.id),
                            on_delete: on_delete,
                            on_edit: on_edit,
                            on_toggle_expanded: on_toggle_expanded
                        }
                    }
                }
            }
        }
    }
}

/// 增强版单个任务项组件
#[component]
fn EnhancedTaskItem(
    task: TaskModel,
    categories: Vec<CategoryModel>,
    is_expanded: bool,
    on_delete: EventHandler<uuid::Uuid>,
    on_edit: EventHandler<TaskModel>,
    on_toggle_expanded: EventHandler<uuid::Uuid>,
) -> Element {
    let mut show_delete_confirm = use_signal(|| false);

    // 状态颜色和图标
    let (status_color, status_icon) = match task.status.as_str() {
        "completed" => ("text-green-600 dark:text-green-400", "✅"),
        "in_progress" => ("text-blue-600 dark:text-blue-400", "🔄"),
        "pending" => ("text-yellow-600 dark:text-yellow-400", "⏳"),
        _ => ("text-gray-600 dark:text-gray-400", "📋"),
    };

    // 优先级颜色
    let priority_color = match task.priority.as_str() {
        "high" => "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400",
        "medium" => "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400",
        "low" => "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400",
        _ => "bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400",
    };

    // 获取分类信息
    let category = task
        .category_id
        .and_then(|id| categories.iter().find(|cat| cat.id == id));

    // 解析标签
    let tags: Vec<String> = serde_json::from_str(&task.tags).unwrap_or_default();

    // 预计算CSS类名
    let _status_class = format!(
        "text-sm font-medium px-2 py-1 rounded-full bg-gray-100 dark:bg-gray-600 {}",
        status_color
    );
    let _priority_class = format!(
        "flex items-center space-x-1 px-2 py-1 rounded-full text-xs font-medium {}",
        priority_color
    );

    rsx! {
        div {
            class: "p-4 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors",

            div {
                class: "flex items-start justify-between",

                // 任务信息
                div {
                    class: "flex-1 min-w-0",

                    // 任务名称和状态
                    div {
                        class: "flex items-center space-x-3 mb-2",
                        span { class: "text-lg", "{status_icon}" }
                        h4 {
                            class: "text-lg font-medium text-gray-900 dark:text-white truncate",
                            "{task.name}"
                        }
                        span {
                            class: "text-sm font-medium px-2 py-1 rounded-full bg-gray-100 dark:bg-gray-600 {status_color}",
                            "{task.status}"
                        }
                    }

                    // 任务详情
                    div {
                        class: "flex flex-wrap items-center gap-3 text-sm text-gray-600 dark:text-gray-400",

                        // 分类
                        if let Some(cat) = category {
                            span {
                                class: "flex items-center space-x-1 px-2 py-1 rounded-full text-xs font-medium",
                                style: "background-color: {cat.color}20; color: {cat.color};",
                                span { "{cat.icon}" }
                                span { "{cat.name}" }
                            }
                        }

                        // 时长
                        span {
                            class: "flex items-center space-x-1",
                            span { "⏱️" }
                            span { "{task.total_duration_seconds / 60}分钟" }
                        }

                        // 优先级
                        span {
                            class: "flex items-center space-x-1 px-2 py-1 rounded-full text-xs font-medium {priority_color}",
                            span { "🎯" }
                            span { "{task.priority}" }
                        }

                        // 创建时间
                        span {
                            class: "flex items-center space-x-1",
                            span { "📅" }
                            span { {task.created_at.format("%m-%d %H:%M").to_string()} }
                        }
                    }

                    // 标签
                    if !tags.is_empty() {
                        div {
                            class: "mt-2 flex flex-wrap gap-1",
                            for tag in tags.iter() {
                                span {
                                    class: "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400",
                                    "#{tag}"
                                }
                            }
                        }
                    }

                    // 任务描述（如果有）
                    if let Some(description) = &task.description {
                        div {
                            class: "mt-2 text-sm text-gray-600 dark:text-gray-400",
                            "{description}"
                        }
                    }
                }

                // 操作按钮
                div {
                    class: "flex items-center space-x-2 ml-4",

                    // 详情按钮
                    button {
                        class: "p-2 text-gray-400 hover:text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded-lg transition-colors",
                        onclick: move |_| on_toggle_expanded.call(task.id),
                        title: "查看详情",
                        if is_expanded { "📖" } else { "👁️" }
                    }

                    // 编辑按钮
                    button {
                        class: "p-2 text-gray-400 hover:text-green-600 hover:bg-green-50 dark:hover:bg-green-900/20 rounded-lg transition-colors",
                        onclick: move |_| on_edit.call(task.clone()),
                        title: "编辑任务",
                        "✏️"
                    }

                    // 删除按钮
                    button {
                        class: "p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors",
                        onclick: move |_| show_delete_confirm.set(true),
                        title: "删除任务",
                        "🗑️"
                    }
                }
            }

            // 详细信息展开
            if is_expanded {
                div {
                    class: "mt-4 pt-4 border-t border-gray-200 dark:border-gray-700",
                    div {
                        class: "grid grid-cols-2 gap-4 text-sm",

                        div {
                            span { class: "font-medium text-gray-700 dark:text-gray-300", "任务ID: " }
                            span { class: "text-gray-600 dark:text-gray-400 font-mono text-xs", "{task.id}" }
                        }

                        if let Some(due_date) = &task.due_date {
                            div {
                                span { class: "font-medium text-gray-700 dark:text-gray-300", "截止日期: " }
                                span {
                                    class: "text-gray-600 dark:text-gray-400",
                                    {due_date.format("%Y-%m-%d %H:%M").to_string()}
                                }
                            }
                        }

                        if let Some(updated_at) = &task.updated_at {
                            div {
                                span { class: "font-medium text-gray-700 dark:text-gray-300", "最后更新: " }
                                span {
                                    class: "text-gray-600 dark:text-gray-400",
                                    {updated_at.format("%Y-%m-%d %H:%M").to_string()}
                                }
                            }
                        }
                    }
                }
            }

            // 删除确认对话框
            if show_delete_confirm() {
                div {
                    class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                    onclick: move |_| show_delete_confirm.set(false),

                    div {
                        class: "bg-white dark:bg-gray-800 rounded-xl p-6 max-w-sm w-full mx-4 shadow-xl",
                        onclick: move |e| e.stop_propagation(),

                        div {
                            class: "text-center",
                            div { class: "text-4xl mb-4", "⚠️" }
                            h3 { class: "text-lg font-semibold text-gray-900 dark:text-white mb-2", "确认删除" }
                            p { class: "text-gray-600 dark:text-gray-400 mb-6", "确定要删除任务 \"{task.name}\" 吗？此操作无法撤销。" }

                            div {
                                class: "flex space-x-3",
                                button {
                                    class: "flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors",
                                    onclick: move |_| show_delete_confirm.set(false),
                                    "取消"
                                }
                                button {
                                    class: "flex-1 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors",
                                    onclick: move |_| {
                                        on_delete.call(task.id);
                                        show_delete_confirm.set(false);
                                    },
                                    "删除"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 增强版任务模态框组件
#[component]
fn EnhancedTaskModal(
    task: Option<TaskModel>, // None表示创建，Some表示编辑
    categories: Vec<life_tracker::storage::models::CategoryModel>,
    on_submit: EventHandler<TaskFormData>,
    on_cancel: EventHandler<()>,
) -> Element {
    // 表单状态
    let mut form_data = use_signal(|| {
        if let Some(ref task) = task {
            // 编辑模式：填充现有数据
            let tags_vec: Vec<String> = serde_json::from_str(&task.tags).unwrap_or_default();
            TaskFormData {
                name: task.name.clone(),
                description: task.description.clone().unwrap_or_default(),
                category_id: task.category_id,
                tags: tags_vec.join(", "),
                priority: task.priority.clone(),
            }
        } else {
            // 创建模式：默认值
            TaskFormData {
                name: String::new(),
                description: String::new(),
                category_id: None,
                tags: String::new(),
                priority: "medium".to_string(),
            }
        }
    });

    let is_editing = task.is_some();
    let title = if is_editing {
        "编辑任务"
    } else {
        "创建新任务"
    };
    let submit_text = if is_editing {
        "更新任务"
    } else {
        "创建任务"
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| on_cancel.call(()),

            div {
                class: "bg-white dark:bg-gray-800 rounded-xl p-6 max-w-lg w-full mx-4 shadow-xl max-h-[90vh] overflow-y-auto",
                onclick: move |e| e.stop_propagation(),

                h3 {
                    class: "text-xl font-semibold text-gray-900 dark:text-white mb-6 text-center",
                    "{title}"
                }

                div {
                    class: "space-y-4",

                    // 任务名称
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "任务名称 *"
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            placeholder: "输入任务名称...",
                            value: "{form_data.read().name}",
                            oninput: move |e| {
                                let mut data = form_data.read().clone();
                                data.name = e.value();
                                form_data.set(data);
                            },
                            autofocus: true
                        }
                    }

                    // 任务描述
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "任务描述"
                        }
                        textarea {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            placeholder: "输入任务描述...",
                            rows: "3",
                            value: "{form_data.read().description}",
                            oninput: move |e| {
                                let mut data = form_data.read().clone();
                                data.description = e.value();
                                form_data.set(data);
                            }
                        }
                    }

                    // 分类选择
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "分类"
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            onchange: move |e| {
                                let mut data = form_data.read().clone();
                                data.category_id = if e.value().is_empty() {
                                    None
                                } else {
                                    uuid::Uuid::parse_str(&e.value()).ok()
                                };
                                form_data.set(data);
                            },
                            option { value: "", "无分类" }
                            for category in categories.iter() {
                                option {
                                    value: "{category.id}",
                                    selected: form_data.read().category_id.map(|id| id == category.id).unwrap_or(false),
                                    "{category.name}"
                                }
                            }
                        }
                    }

                    // 优先级选择
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "优先级"
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            value: "{form_data.read().priority}",
                            onchange: move |e| {
                                let mut data = form_data.read().clone();
                                data.priority = e.value();
                                form_data.set(data);
                            },
                            option { value: "low", "低优先级" }
                            option { value: "medium", "中优先级" }
                            option { value: "high", "高优先级" }
                        }
                    }

                    // 标签输入
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "标签 (用逗号分隔)"
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            placeholder: "例如: 工作, 重要, 紧急",
                            value: "{form_data.read().tags}",
                            oninput: move |e| {
                                let mut data = form_data.read().clone();
                                data.tags = e.value();
                                form_data.set(data);
                            }
                        }
                    }

                    // 按钮
                    div {
                        class: "flex justify-end space-x-3 pt-4",
                        button {
                            class: "px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors",
                            onclick: move |_| on_cancel.call(()),
                            "取消"
                        }
                        button {
                            class: {
                                let base_class = "px-4 py-2 rounded-lg transition-colors";
                                if form_data.read().name.trim().is_empty() {
                                    format!("{} bg-gray-300 text-gray-500 cursor-not-allowed", base_class)
                                } else {
                                    format!("{} bg-blue-600 hover:bg-blue-700 text-white", base_class)
                                }
                            },
                            disabled: form_data.read().name.trim().is_empty(),
                            onclick: move |_| {
                                if !form_data.read().name.trim().is_empty() {
                                    on_submit.call(form_data.read().clone());
                                }
                            },
                            "{submit_text}"
                        }
                    }
                }
            }
        }
    }
}
