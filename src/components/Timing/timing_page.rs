//! # 时间追踪主页面组件
//!
//! 包含标签页导航和路由逻辑

use super::{CategoryManagement, StatisticsPlaceholder, TaskManagementContent, TimingDashboard};
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
            TimingTab::Statistics => "统计报告",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            TimingTab::Dashboard => "🏠",
            TimingTab::Tasks => "📋",
            TimingTab::Categories => "🏷️",
            TimingTab::Statistics => "📊",
        }
    }
}

/// 时间追踪主页面组件
#[component]
pub fn TimingPage() -> Element {
    // 当前激活的标签页
    let mut active_tab = use_signal(|| TimingTab::Dashboard);

    // 标签页列表（静态定义避免生命周期问题）
    const TABS: [TimingTab; 4] = [
        TimingTab::Dashboard,
        TimingTab::Tasks,
        TimingTab::Categories,
        TimingTab::Statistics,
    ];

    rsx! {
        div {
            class: "min-h-screen bg-gray-50 dark:bg-gray-900",

            // 现代化标签页导航
            div {
                class: "bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700",
                div {
                    class: "container mx-auto px-4",
                    nav {
                        class: "flex space-x-1",
                        for tab in TABS.iter() {
                            button {
                                key: "{tab:?}",
                                class: format!("flex items-center space-x-2 py-4 px-6 font-medium text-sm transition-all duration-200 {}",
                                    if *active_tab.read() == *tab {
                                        "text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400"
                                    } else {
                                        "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 rounded-t-lg"
                                    }
                                ),
                                onclick: move |_| active_tab.set(*tab),
                                span { class: "text-lg", "{tab.icon()}" }
                                span { "{tab.label()}" }
                            }
                        }
                    }
                }
            }

            // 标签页内容
            div {
                class: "container mx-auto px-4 py-8",
                match *active_tab.read() {
                    TimingTab::Dashboard => {
                        rsx! { TimingDashboard {} }
                    }
                    TimingTab::Tasks => {
                        rsx! { TaskManagementContent {} }
                    }
                    TimingTab::Categories => {
                        rsx! { CategoryManagement {} }
                    }
                    TimingTab::Statistics => {
                        rsx! { StatisticsPlaceholder {} }
                    }
                }
            }
        }
    }
}
