//! # 日记概览组件
//!
//! 显示日记统计信息和最近笔记

use dioxus::prelude::*;

/// 日记概览组件
#[component]
pub fn NotesOverview() -> Element {
    // 模拟统计数据
    let total_notes = 0;
    let favorite_notes = 0;
    let weekly_notes = 0;

    // 处理新建笔记
    let handle_new_note = move |_| {
        log::info!("创建新笔记");
        // TODO: 实现新建笔记逻辑
    };

    // 处理搜索
    let handle_search = move |_| {
        log::info!("搜索笔记");
        // TODO: 实现搜索逻辑
    };

    rsx! {
        div { class: "space-y-6",
            // 顶部工具栏
            div { class: "flex items-center justify-between",
                div { class: "flex items-center space-x-3",
                    span { class: "text-2xl", "📚" }
                    h1 { class: "text-xl font-bold text-gray-900 dark:text-white",
                        "笔记概览"
                    }
                }
                div { class: "flex items-center space-x-2",
                    button {
                        class: "p-2 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors",
                        onclick: handle_search,
                        span { class: "text-lg", "🔍" }
                    }
                    button {
                        class: "flex items-center space-x-2 px-3 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors",
                        onclick: handle_new_note,
                        span { class: "text-lg", "➕" }
                        span { class: "text-sm font-medium", "新建笔记" }
                    }
                }
            }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                // 总笔记
                div { class: "bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700",
                    div { class: "flex items-center justify-between",
                        div {
                            p { class: "text-sm font-medium text-gray-600 dark:text-gray-400",
                                "总笔记"
                            }
                            p { class: "text-2xl font-bold text-gray-900 dark:text-white",
                                "{total_notes}"
                            }
                        }
                        span { class: "text-2xl", "📚" }
                    }
                }

                // 收藏笔记
                div { class: "bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700",
                    div { class: "flex items-center justify-between",
                        div {
                            p { class: "text-sm font-medium text-gray-600 dark:text-gray-400",
                                "收藏笔记"
                            }
                            p { class: "text-2xl font-bold text-gray-900 dark:text-white",
                                "{favorite_notes}"
                            }
                        }
                        span { class: "text-2xl", "❤️" }
                    }
                }

                // 本周新增
                div { class: "bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700",
                    div { class: "flex items-center justify-between",
                        div {
                            p { class: "text-sm font-medium text-gray-600 dark:text-gray-400",
                                "本周新增"
                            }
                            p { class: "text-2xl font-bold text-gray-900 dark:text-white",
                                "{weekly_notes}"
                            }
                        }
                        span { class: "text-2xl", "📈" }
                    }
                }
            }

            // 最近笔记
            div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700",
                div { class: "p-6 border-b border-gray-200 dark:border-gray-700",
                    h2 { class: "text-lg font-semibold text-gray-900 dark:text-white",
                        "最近笔记"
                    }
                }
                div { class: "p-6",
                    div { class: "text-center py-8",
                        span { class: "text-4xl block mb-4", "📚" }
                        h3 { class: "text-lg font-medium text-gray-900 dark:text-white mb-2",
                            "还没有笔记"
                        }
                        p { class: "text-gray-600 dark:text-gray-400 mb-4",
                            "开始记录您的想法和灵感吧"
                        }
                        button {
                            class: "flex items-center space-x-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors mx-auto",
                            onclick: handle_new_note,
                            span { class: "text-lg", "➕" }
                            span { "创建第一篇笔记" }
                        }
                    }
                }
            }
        }
    }
}
