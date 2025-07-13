//! # 时间追踪主页面组件
//!
//! 包含标签页导航和路由逻辑

use dioxus::prelude::*;

/// 时间追踪页面的子标签
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TimingTab {
    Dashboard,
    Tasks,
    Categories,
    Statistics,
}

impl TimingTab {
    pub fn label(&self) -> &'static str {
        match self {
            TimingTab::Dashboard => "仪表板",
            TimingTab::Tasks => "任务管理",
            TimingTab::Categories => "分类管理",
            TimingTab::Statistics => "统计分析",
        }
    }
}

/// 时间追踪主页面组件
#[component]
pub fn TimingPage() -> Element {
    // 当前活动的标签页
    let mut active_tab = use_signal(|| TimingTab::Dashboard);

    rsx! {
        div {
            class: "min-h-screen bg-gradient-to-br from-slate-50 to-blue-50 dark:from-gray-900 dark:to-gray-800",

            div {
                class: "max-w-7xl mx-auto",

                // 页面标题
                header {
                    class: "bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm shadow-lg border-b border-slate-200/50 dark:border-slate-700/50 mb-6",
                    div {
                        class: "px-6 py-4",
                        h1 {
                            class: "text-3xl font-bold text-gray-800 dark:text-white mb-4",
                            "⏱️ 时间追踪"
                        }

                        // 标签页导航
                        nav {
                            class: "flex space-x-1 bg-gray-100 dark:bg-gray-700 rounded-lg p-1",
                            for tab in [TimingTab::Dashboard, TimingTab::Tasks, TimingTab::Categories, TimingTab::Statistics] {
                                button {
                                    key: "{tab:?}",
                                    class: format!("px-4 py-2 rounded-md text-sm font-medium transition-all duration-200 {}",
                                        if *active_tab.read() == tab {
                                            "bg-white dark:bg-gray-600 text-blue-600 dark:text-blue-400 shadow-sm"
                                        } else {
                                            "text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-white/50 dark:hover:bg-gray-600/50"
                                        }
                                    ),
                                    onclick: move |_| active_tab.set(tab),
                                    "{tab.label()}"
                                }
                            }
                        }
                    }
                }

                // 内容区域
                div {
                    class: "px-6 pb-6",
                    match *active_tab.read() {
                        TimingTab::Dashboard => rsx! {
                            div {
                                class: "space-y-6",
                                
                                // 快速统计卡片
                                div {
                                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                                    for (title, value, icon, color) in [
                                        ("今日工作时间", "2h 30m", "⏰", "blue"),
                                        ("本周工作时间", "18h 45m", "📅", "green"),
                                        ("活跃任务", "3", "📋", "yellow"),
                                        ("完成任务", "12", "✅", "purple")
                                    ] {
                                        div {
                                            class: format!("bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 border-l-4 border-{}-500", color),
                                            div {
                                                class: "flex items-center justify-between",
                                                div {
                                                    p { class: "text-sm font-medium text-gray-600 dark:text-gray-400 mb-1", "{title}" }
                                                    p { class: "text-2xl font-bold text-gray-900 dark:text-white", "{value}" }
                                                }
                                                span { class: "text-3xl", "{icon}" }
                                            }
                                        }
                                    }
                                }

                                // 当前任务和快速操作
                                div {
                                    class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                                    
                                    // 当前任务
                                    div {
                                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6",
                                        h3 { class: "text-lg font-semibold text-gray-900 dark:text-white mb-4", "🎯 当前任务" }
                                        div {
                                            class: "text-center py-8",
                                            p { class: "text-gray-500 dark:text-gray-400 mb-4", "暂无活动任务" }
                                            button {
                                                class: "px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors",
                                                onclick: move |_| active_tab.set(TimingTab::Tasks),
                                                "开始新任务"
                                            }
                                        }
                                    }

                                    // 快速操作
                                    div {
                                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6",
                                        h3 { class: "text-lg font-semibold text-gray-900 dark:text-white mb-4", "⚡ 快速操作" }
                                        div {
                                            class: "space-y-3",
                                            button {
                                                class: "w-full px-4 py-3 text-left bg-gray-50 dark:bg-gray-700 hover:bg-gray-100 dark:hover:bg-gray-600 rounded-lg transition-colors",
                                                onclick: move |_| active_tab.set(TimingTab::Tasks),
                                                div {
                                                    class: "flex items-center",
                                                    span { class: "text-xl mr-3", "📝" }
                                                    span { class: "text-gray-900 dark:text-white", "创建新任务" }
                                                }
                                            }
                                            button {
                                                class: "w-full px-4 py-3 text-left bg-gray-50 dark:bg-gray-700 hover:bg-gray-100 dark:hover:bg-gray-600 rounded-lg transition-colors",
                                                onclick: move |_| active_tab.set(TimingTab::Categories),
                                                div {
                                                    class: "flex items-center",
                                                    span { class: "text-xl mr-3", "🏷️" }
                                                    span { class: "text-gray-900 dark:text-white", "管理分类" }
                                                }
                                            }
                                            button {
                                                class: "w-full px-4 py-3 text-left bg-gray-50 dark:bg-gray-700 hover:bg-gray-100 dark:hover:bg-gray-600 rounded-lg transition-colors",
                                                onclick: move |_| active_tab.set(TimingTab::Statistics),
                                                div {
                                                    class: "flex items-center",
                                                    span { class: "text-xl mr-3", "📊" }
                                                    span { class: "text-gray-900 dark:text-white", "查看统计" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        TimingTab::Tasks => rsx! {
                            div {
                                class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6",
                                h2 { class: "text-xl font-bold mb-4 text-gray-900 dark:text-white", "📋 任务管理" }
                                div {
                                    class: "text-center py-12",
                                    div { class: "text-6xl mb-4", "📝" }
                                    p { class: "text-lg text-gray-600 dark:text-gray-300 mb-6", "任务管理功能开发中..." }
                                    p { class: "text-sm text-gray-500 dark:text-gray-400", "即将支持任务创建、编辑、删除和时间跟踪功能" }
                                }
                            }
                        },
                        TimingTab::Categories => rsx! {
                            div {
                                class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6",
                                h2 { class: "text-xl font-bold mb-4 text-gray-900 dark:text-white", "🏷️ 分类管理" }
                                div {
                                    class: "text-center py-12",
                                    div { class: "text-6xl mb-4", "🗂️" }
                                    p { class: "text-lg text-gray-600 dark:text-gray-300 mb-6", "分类管理功能开发中..." }
                                    p { class: "text-sm text-gray-500 dark:text-gray-400", "即将支持任务分类的创建、编辑和颜色设置功能" }
                                }
                            }
                        },
                        TimingTab::Statistics => rsx! {
                            div {
                                class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6",
                                h2 { class: "text-xl font-bold mb-4 text-gray-900 dark:text-white", "📊 统计分析" }
                                div {
                                    class: "text-center py-12",
                                    div { class: "text-6xl mb-4", "📈" }
                                    p { class: "text-lg text-gray-600 dark:text-gray-300 mb-6", "统计分析功能开发中..." }
                                    p { class: "text-sm text-gray-500 dark:text-gray-400", "即将支持时间统计图表、效率分析和报告导出功能" }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}
