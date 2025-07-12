//! # 分类管理组件
//!
//! 包含分类的创建、编辑、删除和展示功能

use dioxus::prelude::*;
use life_tracker::get_app_state_sync;

/// 分类管理主组件
#[component]
pub fn CategoryManagement() -> Element {
    let mut categories = use_signal(|| Vec::<life_tracker::storage::models::CategoryModel>::new());
    let mut search_term = use_signal(|| String::new());
    let mut show_create_dialog = use_signal(|| false);

    // 获取分类列表
    let fetch_categories = use_callback(move |_| {
        spawn(async move {
            let app_state = life_tracker::get_app_state_sync();
            if let Some(database) = app_state.get_database() {
                match database.get_all_categories() {
                    Ok(category_list) => {
                        categories.set(category_list);
                        log::info!("Categories loaded successfully");
                    }
                    Err(e) => {
                        log::error!("获取分类失败: {}", e);
                    }
                }
            }
        });
    });

    // 初始化加载数据
    use_effect(move || {
        fetch_categories(());
    });

    // 过滤分类
    let filtered_categories: Vec<_> = categories
        .read()
        .iter()
        .filter(|category| {
            search_term.read().is_empty()
                || category
                    .name
                    .to_lowercase()
                    .contains(&search_term.read().to_lowercase())
                || category.description.as_ref().map_or(false, |desc| {
                    desc.to_lowercase()
                        .contains(&search_term.read().to_lowercase())
                })
        })
        .cloned()
        .collect();

    rsx! {
        div {
            class: "space-y-6",

            // 页面标题和操作按钮
            div {
                class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-6",
                div {
                    class: "flex items-center justify-between mb-4",
                    h2 { class: "text-2xl font-bold text-gray-900 dark:text-white", "分类管理" }
                    button {
                        class: "flex items-center space-x-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors shadow-md",
                        onclick: move |_| show_create_dialog.set(true),
                        span { class: "text-lg", "➕" }
                        span { "新建分类" }
                    }
                }

                // 搜索框
                div {
                    class: "relative",
                    span {
                        class: "absolute left-3 top-3 text-gray-400 dark:text-gray-500",
                        "🔍"
                    }
                    input {
                        r#type: "text",
                        class: "pl-10 pr-4 py-2 w-full border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors",
                        placeholder: "搜索分类名称或描述...",
                        value: "{search_term.read()}",
                        oninput: move |e| search_term.set(e.value()),
                    }
                }

                // 分类数量统计
                div {
                    class: "mt-4 text-sm text-gray-600 dark:text-gray-400",
                    "显示 {filtered_categories.len()} 个分类"
                }
            }

            // 分类列表
            if filtered_categories.is_empty() {
                div {
                    class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-12 text-center",
                    div { class: "text-6xl mb-4", "📁" }
                    h3 { class: "text-xl font-medium text-gray-900 dark:text-white mb-2", "暂无分类" }
                    p { class: "text-gray-600 dark:text-gray-400", "创建您的第一个分类来组织任务" }
                }
            } else {
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                    for category in filtered_categories.iter() {
                        CategoryCard {
                            key: "{category.id}",
                            category: category.clone()
                        }
                    }
                }
            }

            // 创建分类对话框
            if show_create_dialog() {
                CreateCategoryModal {
                    on_close: move |_| show_create_dialog.set(false),
                    on_created: move |_| {
                        show_create_dialog.set(false);
                        fetch_categories(());
                    }
                }
            }
        }
    }
}

/// 分类卡片组件
#[component]
fn CategoryCard(category: life_tracker::storage::models::CategoryModel) -> Element {
    let mut show_actions = use_signal(|| false);

    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 shadow-md hover:shadow-lg transition-all duration-200 p-6 relative",
            onmouseenter: move |_| show_actions.set(true),
            onmouseleave: move |_| show_actions.set(false),

            // 分类头部
            div {
                class: "flex items-center space-x-3 mb-4",
                div {
                    class: "w-12 h-12 rounded-xl flex items-center justify-center shadow-sm",
                    style: "background-color: {category.color}20;",
                    span {
                        style: "color: {category.color}; font-size: 24px;",
                        "{category.icon}"
                    }
                }
                div {
                    class: "flex-1 min-w-0",
                    h3 {
                        class: "text-lg font-semibold text-gray-900 dark:text-white truncate",
                        "{category.name}"
                    }
                    p {
                        class: "text-sm text-gray-500 dark:text-gray-400",
                        "0 个任务" // TODO: 从数据库获取实际任务数量
                    }
                }
            }

            // 分类描述
            if let Some(description) = &category.description {
                p {
                    class: "text-sm text-gray-600 dark:text-gray-300 mb-4 line-clamp-2",
                    "{description}"
                }
            }

            // 分类信息
            div {
                class: "flex items-center justify-between text-xs text-gray-500 dark:text-gray-400",
                div {
                    class: "px-2 py-1 rounded-full",
                    style: "background-color: {category.color}15; color: {category.color};",
                    "{category.name}"
                }
                div {
                    "创建于 {category.created_at.format(\"%Y-%m-%d\")}"
                }
            }

            // 操作按钮 (悬停时显示)
            if show_actions() {
                div {
                    class: "absolute top-4 right-4 flex space-x-1 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 p-1",

                                        button {
                        class: "p-2 text-gray-400 hover:text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded transition-colors",
                        title: "编辑分类",
                        onclick: {
                            let category_name = category.name.clone();
                            move |_| {
                                // TODO: 实现编辑功能
                                log::info!("编辑分类: {}", category_name);
                            }
                        },
                        "✏️"
                    }

                    button {
                        class: "p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors",
                        title: "删除分类",
                        onclick: {
                            let category_name = category.name.clone();
                            move |_| {
                                // TODO: 实现删除功能
                                log::info!("删除分类: {}", category_name);
                            }
                        },
                        "🗑️"
                    }
                }
            }
        }
    }
}

/// 创建分类模态框组件
#[component]
fn CreateCategoryModal(on_close: EventHandler<()>, on_created: EventHandler<()>) -> Element {
    let mut form_data = use_signal(|| CategoryFormData::default());
    let mut error_message = use_signal(|| None::<String>);

    // 预设颜色选项
    const PRESET_COLORS: [&str; 12] = [
        "#3B82F6", "#10B981", "#F59E0B", "#EF4444", "#8B5CF6", "#06B6D4", "#84CC16", "#F97316",
        "#EC4899", "#6366F1", "#14B8A6", "#DC2626",
    ];

    // 预设图标选项
    const PRESET_ICONS: [&str; 16] = [
        "💼", "📚", "🏃", "🎯", "🔧", "🎨", "📱", "💻", "🏠", "🍳", "✈️", "🎵", "📷", "🏥", "🛒",
        "📝",
    ];

    // 创建分类功能
    let mut create_category = move || {
        let data = form_data.read().clone();

        if data.name.trim().is_empty() {
            error_message.set(Some("分类名称不能为空".to_string()));
            return;
        }

        let app_state = get_app_state_sync();
        if let Some(database) = app_state.get_database() {
            let category = life_tracker::storage::models::CategoryInsert {
                id: uuid::Uuid::new_v4(),
                name: data.name.trim().to_string(),
                description: if data.description.trim().is_empty() {
                    None
                } else {
                    Some(data.description.trim().to_string())
                },
                color: data.color,
                icon: data.icon,
                daily_target_seconds: None,
                weekly_target_seconds: None,
                is_active: true,
                sort_order: 0,
                parent_id: None,
                created_at: chrono::Local::now(),
            };

            match database.insert_category(&category) {
                Ok(_) => {
                    log::info!("分类创建成功: {}", category.name);
                    on_created.call(());
                }
                Err(e) => {
                    log::error!("分类创建失败: {}", e);
                    error_message.set(Some(format!("分类创建失败: {}", e)));
                }
            }
        } else {
            error_message.set(Some("数据库未初始化".to_string()));
        }
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| on_close.call(()),

            div {
                class: "bg-white dark:bg-gray-800 rounded-xl p-6 max-w-md w-full mx-4 shadow-xl max-h-[90vh] overflow-y-auto",
                onclick: move |e| e.stop_propagation(),

                h3 {
                    class: "text-xl font-semibold text-gray-900 dark:text-white mb-6 text-center",
                    "创建新分类"
                }

                div {
                    class: "space-y-4",

                    // 分类名称
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "分类名称 *"
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            placeholder: "输入分类名称...",
                            value: "{form_data.read().name}",
                            oninput: move |e| {
                                let mut data = form_data.read().clone();
                                data.name = e.value();
                                form_data.set(data);
                            },
                            autofocus: true
                        }
                    }

                    // 分类描述
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "分类描述"
                        }
                        textarea {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            placeholder: "输入分类描述...",
                            rows: "3",
                            value: "{form_data.read().description}",
                            oninput: move |e| {
                                let mut data = form_data.read().clone();
                                data.description = e.value();
                                form_data.set(data);
                            }
                        }
                    }

                    // 颜色选择
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "分类颜色"
                        }
                        div {
                            class: "grid grid-cols-6 gap-2",
                            for color in PRESET_COLORS.iter() {
                                button {
                                    class: format!("w-8 h-8 rounded-lg border-2 transition-all {}",
                                        if form_data.read().color == *color {
                                            "border-gray-800 dark:border-white scale-110"
                                        } else {
                                            "border-gray-200 dark:border-gray-600 hover:scale-105"
                                        }
                                    ),
                                    style: "background-color: {color}",
                                    onclick: move |_| {
                                        let mut data = form_data.read().clone();
                                        data.color = color.to_string();
                                        form_data.set(data);
                                    }
                                }
                            }
                        }
                    }

                    // 图标选择
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "分类图标"
                        }
                        div {
                            class: "grid grid-cols-8 gap-2",
                            for icon in PRESET_ICONS.iter() {
                                button {
                                    class: format!("w-8 h-8 rounded-lg border-2 flex items-center justify-center text-lg transition-all {}",
                                        if form_data.read().icon == *icon {
                                            "border-blue-500 bg-blue-50 dark:bg-blue-900/20 scale-110"
                                        } else {
                                            "border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 hover:scale-105"
                                        }
                                    ),
                                    onclick: move |_| {
                                        let mut data = form_data.read().clone();
                                        data.icon = icon.to_string();
                                        form_data.set(data);
                                    },
                                    "{icon}"
                                }
                            }
                        }
                    }

                    // 预览区域
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "预览"
                        }
                        div {
                            class: "flex items-center space-x-3 p-3 rounded-lg border border-gray-200 dark:border-gray-600",
                            div {
                                class: "w-10 h-10 rounded-lg flex items-center justify-center",
                                style: "background-color: {form_data.read().color}20;",
                                span {
                                    style: "color: {form_data.read().color}; font-size: 20px;",
                                    "{form_data.read().icon}"
                                }
                            }
                            div {
                                h4 {
                                    class: "font-medium text-gray-900 dark:text-white",
                                    {
                                        let name = form_data.read().name.clone();
                                        if name.is_empty() {
                                            "分类名称".to_string()
                                        } else {
                                            name
                                        }
                                    }
                                }
                                p {
                                    class: "text-sm text-gray-500 dark:text-gray-400",
                                    {
                                        let desc = form_data.read().description.clone();
                                        if desc.is_empty() {
                                            "分类描述".to_string()
                                        } else {
                                            desc
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 错误信息
                    if let Some(error) = error_message.read().as_ref() {
                        div {
                            class: "bg-red-50 border border-red-200 text-red-700 px-3 py-2 rounded-lg text-sm",
                            "{error}"
                        }
                    }

                    // 按钮
                    div {
                        class: "flex justify-end space-x-3 pt-4",
                        button {
                            class: "px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors",
                            onclick: move |_| on_close.call(()),
                            "取消"
                        }
                        button {
                            class: format!("px-4 py-2 rounded-lg transition-colors {}",
                                if form_data.read().name.trim().is_empty() {
                                    "bg-gray-300 text-gray-500 cursor-not-allowed"
                                } else {
                                    "bg-blue-600 hover:bg-blue-700 text-white"
                                }
                            ),
                            disabled: form_data.read().name.trim().is_empty(),
                            onclick: move |_| {
                                if !form_data.read().name.trim().is_empty() {
                                    create_category();
                                }
                            },
                            "创建分类"
                        }
                    }
                }
            }
        }
    }
}

/// 分类表单数据
#[derive(Clone, Debug)]
struct CategoryFormData {
    name: String,
    description: String,
    color: String,
    icon: String,
}

impl Default for CategoryFormData {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            color: "#3B82F6".to_string(), // 默认蓝色
            icon: "📁".to_string(),       // 默认文件夹图标
        }
    }
}
