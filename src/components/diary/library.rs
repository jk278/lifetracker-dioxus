//! # 日记库组件
//!
//! 显示笔记列表，支持搜索和过滤功能

use dioxus::prelude::*;

/// 过滤器类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterType {
    All,
    Favorite,
    Archived,
}

impl FilterType {
    fn label(&self) -> &'static str {
        match self {
            FilterType::All => "全部",
            FilterType::Favorite => "收藏",
            FilterType::Archived => "归档",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            FilterType::All => "📚",
            FilterType::Favorite => "❤️",
            FilterType::Archived => "📦",
        }
    }
}

/// 视图模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Grid,
    List,
}

/// 模拟笔记数据
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub mood: Option<String>,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 日记库组件
#[component]
pub fn NotesLibrary() -> Element {
    // 状态管理
    let mut view_mode = use_signal(|| ViewMode::Grid);
    let mut search_query = use_signal(|| String::new());
    let mut selected_filter = use_signal(|| FilterType::All);

    // 模拟笔记数据
    let notes = use_signal(|| {
        vec![
            Note {
                id: "1".to_string(),
                title: "今日工作总结".to_string(),
                content: "今天完成了项目的主要功能开发，遇到了一些技术难题但都已解决..."
                    .to_string(),
                mood: Some("happy".to_string()),
                tags: vec!["工作".to_string(), "总结".to_string()],
                is_favorite: true,
                is_archived: false,
                created_at: "2024-01-15T10:30:00Z".to_string(),
                updated_at: "2024-01-15T18:45:00Z".to_string(),
            },
            Note {
                id: "2".to_string(),
                title: "学习笔记 - React Hooks".to_string(),
                content: "useCallback 和 useMemo 的区别与使用场景...".to_string(),
                mood: Some("excited".to_string()),
                tags: vec!["学习".to_string(), "React".to_string(), "前端".to_string()],
                is_favorite: false,
                is_archived: false,
                created_at: "2024-01-14T14:20:00Z".to_string(),
                updated_at: "2024-01-14T16:30:00Z".to_string(),
            },
        ]
    });

    // 过滤笔记
    let filtered_notes = use_memo(move || {
        let notes = notes.read();
        let search = search_query.read();
        let filter = selected_filter.read();

        notes
            .iter()
            .filter(|note| {
                // 过滤器检查
                match *filter {
                    FilterType::Favorite => note.is_favorite,
                    FilterType::Archived => note.is_archived,
                    FilterType::All => !note.is_archived, // 默认不显示归档
                }
            })
            .filter(|note| {
                // 搜索检查
                if search.is_empty() {
                    true
                } else {
                    let search_lower = search.to_lowercase();
                    note.title.to_lowercase().contains(&search_lower)
                        || note.content.to_lowercase().contains(&search_lower)
                }
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    // 获取心情表情
    let get_mood_emoji = move |mood: &str| -> &'static str {
        match mood {
            "happy" => "😊",
            "sad" => "😢",
            "neutral" => "😐",
            "excited" => "🤩",
            "stressed" => "😰",
            "relaxed" => "😌",
            "anxious" => "😟",
            "confident" => "😎",
            _ => "😐",
        }
    };

    // 格式化日期
    let format_date = move |date_str: &str| -> String {
        // 简单的日期格式化，实际应用中应该使用更好的日期库
        date_str.split('T').next().unwrap_or(date_str).to_string()
    };

    rsx! {
        div { class: "space-y-6",
            // 顶部工具栏
            div { class: "flex items-center justify-between",
                div { class: "flex items-center space-x-3",
                    span { class: "text-2xl", "📚" }
                    h1 { class: "text-xl font-bold text-gray-900 dark:text-white",
                        "笔记库"
                    }
                }
                div { class: "flex items-center space-x-2",
                    button {
                        class: "p-2 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors",
                        onclick: move |_| {
                            view_mode.set(match view_mode() {
                                ViewMode::Grid => ViewMode::List,
                                ViewMode::List => ViewMode::Grid,
                            });
                        },
                        span { class: "text-lg",
                            match view_mode() {
                                ViewMode::Grid => "📋",
                                ViewMode::List => "▦",
                            }
                        }
                    }
                }
            }

            // 搜索和过滤
            div { class: "flex flex-col sm:flex-row gap-4",
                // 搜索框
                div { class: "relative flex-1",
                    span { class: "absolute left-3 top-1/2 transform -translate-y-1/2 text-lg", "🔍" }
                    input {
                        r#type: "text",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value().clone()),
                        placeholder: "搜索笔记...",
                        class: "w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                    }
                }

                // 过滤器
                div { class: "flex space-x-2",
                    for filter in [FilterType::All, FilterType::Favorite, FilterType::Archived] {
                        button {
                            key: "{filter:?}",
                            class: if selected_filter() == filter {
                                "flex items-center space-x-2 px-3 py-2 rounded-lg text-sm font-medium bg-blue-600 text-white transition-colors"
                            } else {
                                "flex items-center space-x-2 px-3 py-2 rounded-lg text-sm font-medium bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                            },
                            onclick: move |_| selected_filter.set(filter),
                            span { class: "text-lg", "{filter.icon()}" }
                            span { "{filter.label()}" }
                        }
                    }
                }
            }

            // 笔记列表
            if filtered_notes().is_empty() {
                div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-12",
                    div { class: "text-center",
                        span { class: "text-4xl block mb-4", "📚" }
                        h3 { class: "text-lg font-medium text-gray-900 dark:text-white mb-2",
                            if search_query.read().is_empty() {
                                "还没有笔记"
                            } else {
                                "未找到相关笔记"
                            }
                        }
                        p { class: "text-gray-600 dark:text-gray-400",
                            if search_query.read().is_empty() {
                                "开始创建您的第一篇笔记"
                            } else {
                                "尝试调整搜索条件"
                            }
                        }
                    }
                }
            } else {
                div {
                    class: match view_mode() {
                        ViewMode::Grid => "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                        ViewMode::List => "space-y-4",
                    },
                    for note in filtered_notes() {
                        div {
                            key: "{note.id}",
                            class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 hover:shadow-md transition-shadow cursor-pointer",

                            // 笔记头部
                            div { class: "flex items-start justify-between mb-3",
                                h3 { class: "font-semibold text-gray-900 dark:text-white truncate flex-1",
                                    "{note.title}"
                                }
                                div { class: "flex items-center space-x-1 ml-2 flex-shrink-0",
                                    if let Some(mood) = &note.mood {
                                        span { class: "text-lg",
                                            "{get_mood_emoji(mood)}"
                                        }
                                    }
                                    if note.is_favorite {
                                        span { class: "text-lg text-red-500", "❤️" }
                                    }
                                    if note.is_archived {
                                        span { class: "text-lg text-gray-500", "📦" }
                                    }
                                }
                            }

                            // 笔记内容预览
                            p { class: "text-gray-600 dark:text-gray-300 text-sm mb-3 line-clamp-3",
                                "{note.content}"
                            }

                            // 标签
                            if !note.tags.is_empty() {
                                div { class: "flex flex-wrap gap-1 mb-3",
                                    for tag in note.tags.iter() {
                                        span {
                                            key: "{tag}",
                                            class: "px-2 py-1 bg-blue-100 dark:bg-blue-900/20 text-blue-800 dark:text-blue-400 text-xs rounded-full",
                                            "{tag}"
                                        }
                                    }
                                }
                            }

                            // 时间信息
                            div { class: "flex items-center text-xs text-gray-500 dark:text-gray-400",
                                span { class: "mr-1", "📅" }
                                span { "更新于 {format_date(&note.updated_at)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
