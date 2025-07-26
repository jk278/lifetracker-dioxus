//! # 主应用组件
//!
//! 定义应用的页面导航和基础布局，是整个 Dioxus 应用的入口点。

use super::app_state_provider::AppStateProvider;
use super::theme_provider::ThemeProvider;
use dioxus::prelude::*;

/// 页面枚举定义
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Page {
    Dashboard,
    Tasks,
    Financial,
    Diary,
    Habits,
    Settings,
}

impl Page {
    fn title(&self) -> &'static str {
        match self {
            Page::Dashboard => "📊 仪表板",
            Page::Tasks => "⏱️ 时间追踪",
            Page::Financial => "💰 财务管理",
            Page::Diary => "📝 日记",
            Page::Habits => "🎯 习惯打卡",
            Page::Settings => "⚙️ 设置",
        }
    }
}

/// 主应用组件
#[component]
pub fn App() -> Element {
    // 当前页面状态
    let current_page = use_signal(|| Page::Dashboard);

    // 使用Provider层级包装应用
    rsx! {
        // 引入Tailwind CSS
        document::Stylesheet { href: "/assets/tailwind.css" }

        AppStateProvider {
            ThemeProvider {
                AppContent { current_page }
            }
        }
    }
}

/// 应用内容组件 - 分离出来以便ThemeProvider包装
#[component]
fn AppContent(current_page: Signal<Page>) -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-gray-50 dark:bg-gray-900",
            // 导航栏
            nav {
                class: "bg-white dark:bg-gray-800 shadow-lg border-b border-gray-200 dark:border-gray-700 sticky top-0 z-50",
                div { class: "max-w-7xl mx-auto px-4",
                    div { class: "flex justify-between items-center h-16",
                        // Logo
                        div { class: "flex items-center",
                            button {
                                onclick: move |_| { current_page.set(Page::Dashboard); },
                                class: "text-xl font-bold text-gray-900 dark:text-white hover:text-blue-600 dark:hover:text-blue-400 transition-colors",
                                "📊 LifeTracker"
                            }
                            // Tailwind CSS 测试指示器
                            div { 
                                class: "ml-2 px-2 py-1 bg-green-500 text-white text-xs rounded-full",
                                "CSS✓"
                            }
                        }

                        // 导航菜单
                        div { class: "flex space-x-1",
                            for page in [Page::Dashboard, Page::Tasks, Page::Financial, Page::Diary, Page::Habits, Page::Settings] {
                                button {
                                    onclick: move |_| { current_page.set(page); },
                                    class: if *current_page.read() == page {
                                        "px-3 py-2 rounded-md text-sm font-medium text-blue-600 dark:text-blue-400 bg-blue-100 dark:bg-blue-900/50"
                                    } else {
                                        "px-3 py-2 rounded-md text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                                    },
                                    "{page.title()}"
                                }
                            }
                        }
                    }
                }
            }

            // 主内容区域
            main { class: "flex-1",
                match current_page() {
                    Page::Dashboard => rsx! { Dashboard {} },
                    Page::Tasks => rsx! { TaskManagement {} },
                    Page::Financial => rsx! { Financial {} },
                    Page::Diary => rsx! { Diary {} },
                    Page::Habits => rsx! { Habits {} },
                    Page::Settings => rsx! { Settings {} },
                }
            }
        }
    }
}

// 页面组件定义 - 直接使用原有的完整组件

/// 仪表盘组件
#[component]
fn Dashboard() -> Element {
    rsx! {
        super::dashboard::Dashboard {}
    }
}

/// 任务管理/时间追踪组件
#[component]
fn TaskManagement() -> Element {
    rsx! {
        super::timing::TimingPage {}
    }
}

/// 财务管理组件
#[component]
fn Financial() -> Element {
    rsx! {
        super::accounting::AccountingPage {}
    }
}

/// 日记组件
#[component]
fn Diary() -> Element {
    rsx! {
        super::diary::DiaryPage {}
    }
}

/// 习惯打卡组件
#[component]
fn Habits() -> Element {
    rsx! {
        super::habits::HabitsPage {}
    }
}

/// 设置组件
#[component]
fn Settings() -> Element {
    rsx! {
        super::settings::SettingsPage {
            show_back_button: false,
        }
    }
}
