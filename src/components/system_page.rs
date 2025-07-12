//! # 系统管理页面
//!
//! 系统管理模块的主页面，提供数据管理、设置、关于三个主要功能的导航入口

use crate::components::{AboutPage, DataManagementPage, SettingsPage};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SystemPageProps {
    pub show_back_button: bool,
    pub on_back: Option<EventHandler<()>>,
}

// 系统页面项结构
#[derive(Debug, Clone, PartialEq)]
struct SystemItem {
    id: String,
    name: String,
    icon: String,
    description: String,
}

// 页面状态枚举
#[derive(Debug, Clone, PartialEq)]
enum SystemPageState {
    Overview,
    Settings,
    About,
    DataManagement,
}

#[component]
pub fn SystemPage(props: SystemPageProps) -> Element {
    // 当前页面状态
    let current_page = use_signal(|| SystemPageState::Overview);

    // 系统管理项目
    let system_items = use_memo(|| {
        vec![
            SystemItem {
                id: "data".to_string(),
                name: "数据管理".to_string(),
                icon: "💾".to_string(),
                description: "导入导出、备份恢复".to_string(),
            },
            SystemItem {
                id: "settings".to_string(),
                name: "应用设置".to_string(),
                icon: "⚙️".to_string(),
                description: "主题、偏好设置".to_string(),
            },
            SystemItem {
                id: "about".to_string(),
                name: "关于应用".to_string(),
                icon: "ℹ️".to_string(),
                description: "版本信息、许可证".to_string(),
            },
        ]
    });

    // 处理导航到子页面
    let handle_navigate_to_sub_page = {
        let mut current_page = current_page.clone();
        move |sub_page_id: String| match sub_page_id.as_str() {
            "data" => current_page.set(SystemPageState::DataManagement),
            "settings" => current_page.set(SystemPageState::Settings),
            "about" => current_page.set(SystemPageState::About),
            _ => current_page.set(SystemPageState::Overview),
        }
    };

    // 处理返回到概览页面
    let mut handle_back_to_overview = {
        let mut current_page = current_page.clone();
        move || {
            current_page.set(SystemPageState::Overview);
        }
    };

    // 渲染系统页面概览
    let render_overview = || {
        rsx! {
            div { class: "h-full p-6 overflow-y-auto",
                div { class: "max-w-4xl mx-auto",

                    // 页面标题
                    div { class: "mb-8",
                        h1 { class: "text-2xl font-bold text-gray-900 dark:text-white mb-2",
                            "系统管理"
                        }
                        p { class: "text-gray-600 dark:text-gray-300",
                            "管理应用数据、设置和查看相关信息"
                        }
                    }

                    // 选项卡网格
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                        for item in system_items.read().iter() {
                            button {
                                key: "{item.id}",
                                class: "p-6 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-600 dark:hover:border-blue-400 transition-all duration-200 text-left group shadow-lg hover:shadow-xl",
                                onclick: {
                                    let item_id = item.id.clone();
                                    let handle_navigate = handle_navigate_to_sub_page.clone();
                                    move |_| handle_navigate(item_id.clone())
                                },

                                div { class: "flex items-center mb-3",
                                    div { class: "w-10 h-10 bg-blue-100 dark:bg-blue-900/20 rounded-lg flex items-center justify-center group-hover:bg-blue-200 dark:group-hover:bg-blue-900/30 transition-colors",
                                        span { class: "text-xl", "{item.icon}" }
                                    }
                                }
                                h3 { class: "text-lg font-semibold text-gray-900 dark:text-white mb-2",
                                    "{item.name}"
                                }
                                p { class: "text-sm text-gray-600 dark:text-gray-300",
                                    "{item.description}"
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    // 根据当前页面状态渲染不同内容
    let page_state = current_page.read().clone();
    match page_state {
        SystemPageState::Overview => rsx! {
            div { class: "h-full flex flex-col",

                // 固定顶部导航栏
                div { class: "flex-shrink-0 px-4 md:px-6 py-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800",
                    div { class: "flex items-center justify-between",
                        div { class: "flex items-center space-x-3",
                            // 返回按钮
                            if props.show_back_button {
                                button {
                                    class: "flex items-center justify-center w-8 h-8 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors",
                                    onclick: move |_| {
                                        if let Some(handler) = &props.on_back {
                                            handler.call(());
                                        }
                                    },
                                    title: "返回",
                                    "←"
                                }
                            }
                            h2 { class: "text-2xl font-bold text-gray-900 dark:text-white",
                                "系统"
                            }
                        }
                    }
                }

                // 概览内容
                div { class: "flex-1",
                    {render_overview()}
                }
            }
        },

        SystemPageState::Settings => rsx! {
            SettingsPage {
                show_back_button: true,
                on_back: move |_| handle_back_to_overview(),
            }
        },

        SystemPageState::About => rsx! {
            AboutPage {
                show_back_button: true,
                on_back: move |_| handle_back_to_overview(),
            }
        },

        SystemPageState::DataManagement => rsx! {
            DataManagementPage {
                show_back_button: true,
                on_back: move |_| handle_back_to_overview(),
            }
        },
    }
}
