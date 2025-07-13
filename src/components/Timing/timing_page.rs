//! # 时间追踪主页面组件
//!
//! 包含标签页导航和路由逻辑

// use super::{CategoryManagement, StatisticsPlaceholder, TaskManagementContent, TimingDashboard}; // 暂时注释未使用的导入
use dioxus::prelude::*;
// use dioxus_free_icons::{icons::bs_icons::*, Icon}; // 暂时注释未使用的导入

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
            TimingTab::Statistics => "统计报告",
        }
    }

    pub fn icon(&self) -> Element {
        let icon_class = "w-5 h-5";
        match self {
            TimingTab::Dashboard => rsx! { span { class: icon_class, "🏠" } },
            TimingTab::Tasks => rsx! { span { class: icon_class, "📋" } },
            TimingTab::Categories => rsx! { span { class: icon_class, "🏷️" } },
            TimingTab::Statistics => rsx! { span { class: icon_class, "📊" } },
        }
    }
}

// 标签页列表（静态定义避免生命周期问题）
const TABS: [TimingTab; 4] = [
    TimingTab::Dashboard,
    TimingTab::Tasks,
    TimingTab::Categories,
    TimingTab::Statistics,
];

/// 时间追踪主页面组件
#[component]
pub fn TimingPage() -> Element {
    // 当前激活的标签页
    let mut active_tab = use_signal(|| TimingTab::Dashboard);


    rsx! {
        div {
            class: "min-h-screen bg-gradient-to-br from-slate-50 to-blue-50 dark:from-gray-900 dark:to-gray-800",

            // 现代化标签页导航
            div {
                class: "bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm shadow-lg border-b border-gray-200/50 dark:border-gray-700/50 sticky top-0 z-10",
                div {
                    class: "container mx-auto px-6",
                    nav {
                        class: "flex space-x-2",
                        for tab in TABS.iter() {
                            button {
                                key: "{tab:?}",
                                class: if *active_tab.read() == *tab {
                                    "group flex items-center space-x-3 py-4 px-6 font-medium text-sm transition-all duration-300 rounded-t-xl relative text-blue-600 dark:text-blue-400 bg-white dark:bg-gray-700 shadow-lg transform -translate-y-1"
                                } else {
                                    "group flex items-center space-x-3 py-4 px-6 font-medium text-sm transition-all duration-300 rounded-t-xl relative text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-white/50 dark:hover:bg-gray-700/50 hover:shadow-md hover:-translate-y-0.5"
                                },
                                onclick: move |_| active_tab.set(*tab),
                                
                                // 添加活跃状态指示器
                                if *active_tab.read() == *tab {
                                    div {
                                        class: "absolute bottom-0 left-1/2 transform -translate-x-1/2 w-12 h-1 bg-blue-500 rounded-t-full"
                                    }
                                }
                                
                                div {
                                    class: if *active_tab.read() == *tab {
                                        "transition-all duration-300 text-blue-500 scale-110"
                                    } else {
                                        "transition-all duration-300 text-gray-400 group-hover:text-gray-600 group-hover:scale-105"
                                    },
                                    span { 
                                        class: "w-5 h-5",
                                        match tab {
                                            TimingTab::Dashboard => "🏠",
                                            TimingTab::Tasks => "📋", 
                                            TimingTab::Categories => "🏷️",
                                            TimingTab::Statistics => "📊",
                                        }
                                    }
                                }
                                span { 
                                    class: "font-semibold",
                                    "{tab.label()}" 
                                }
                            }
                        }
                    }
                }
            }

            // 标签页内容 - 添加动画过渡
            div {
                class: "container mx-auto px-6 py-8",
                div {
                    class: "animate-fade-in",
                    match *active_tab.read() {
                        TimingTab::Dashboard => rsx! { TimingDashboard {} },
                        TimingTab::Tasks => rsx! { TaskManagementContent {} },
                        TimingTab::Categories => rsx! { CategoryManagement {} },
                        TimingTab::Statistics => rsx! { StatisticsPlaceholder {} },
                    }
                }
            }
        }

        // 添加自定义CSS动画
        style {
            r#"
            @keyframes fade-in {
                from { opacity: 0; transform: translateY(10px); }
                to { opacity: 1; transform: translateY(0); }
            }
            .animate-fade-in {
                animation: fade-in 0.3s ease-out;
            }
            "#
        }
    }
}
