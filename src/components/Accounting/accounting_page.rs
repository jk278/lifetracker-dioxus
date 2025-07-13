//! # 财务管理主页面组件
//!
//! 财务管理模块的主入口，包含标签页导航

use super::super::common::{Button, ButtonVariant, Card};
use dioxus::prelude::*;

/// 财务管理主页面组件
#[component]
pub fn AccountingPage() -> Element {
    // 添加状态管理
    let mut active_tab = use_signal(|| "overview");

    // 数据加载
    let accounts_data = use_resource(move || async move {
        // 模拟数据加载
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok::<Vec<(&str, &str, &str)>, String>(vec![
            ("checking", "活期存款", "¥10,000.00"),
            ("savings", "定期存款", "¥50,000.00"),
            ("credit", "信用卡", "-¥2,500.00"),
        ])
    });

    rsx! {
        div {
            class: "min-h-screen bg-gradient-to-br from-emerald-50 via-green-50 to-teal-100 dark:from-gray-900 dark:via-gray-800 dark:to-gray-700",

            // 添加标签页导航
            div {
                class: "bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm shadow-lg border-b border-emerald-200/50 dark:border-emerald-700/50 sticky top-0 z-10",
                div {
                    class: "container mx-auto px-6",
                    nav {
                        class: "flex space-x-2",

                        Button {
                            variant: if *active_tab.read() == "overview" { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                            onclick: move |_| active_tab.set("overview"),
                            class: "py-4 px-6",
                            "财务概览"
                        }

                        Button {
                            variant: if *active_tab.read() == "accounts" { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                            onclick: move |_| active_tab.set("accounts"),
                            class: "py-4 px-6",
                            "账户管理"
                        }

                        Button {
                            variant: if *active_tab.read() == "transactions" { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                            onclick: move |_| active_tab.set("transactions"),
                            class: "py-4 px-6",
                            "交易记录"
                        }

                        Button {
                            variant: if *active_tab.read() == "stats" { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                            onclick: move |_| active_tab.set("stats"),
                            class: "py-4 px-6",
                            "统计分析"
                        }
                    }
                }
            }

            // 内容区域
            div {
                class: "container mx-auto px-6 py-8",

                match active_tab.read().as_ref() {
                    "overview" => rsx! {
                        div {
                            class: "space-y-6",
                            // 快速统计卡片
                            div {
                                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                                for (index, (title, value, icon)) in [
                                    ("总资产", "¥0.00", "💰"),
                                    ("总负债", "¥0.00", "💳"),
                                    ("净资产", "¥0.00", "📈"),
                                    ("本月支出", "¥0.00", "💸")
                                ].iter().enumerate() {
                                    div {
                                        key: "{index}",
                                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 border border-gray-200 dark:border-gray-700",
                                        div {
                                            class: "flex items-center justify-between",
                                            div {
                                                p { class: "text-sm font-medium text-gray-600 dark:text-gray-400", "{title}" }
                                                p { class: "text-2xl font-bold text-gray-900 dark:text-white", "{value}" }
                                            }
                                            span { class: "text-3xl", "{icon}" }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "accounts" => rsx! {
                        div {
                            class: "space-y-6",
                            h2 { class: "text-2xl font-bold text-gray-900 dark:text-white mb-6", "账户管理" }

                            match &*accounts_data.read_unchecked() {
                                Some(Ok(accounts)) => rsx! {
                                    div {
                                        class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                                        for (index, (account_type, name, balance)) in accounts.iter().enumerate() {
                                            Card {
                                                key: "{index}",
                                                hover: true,
                                                shadow: true,
                                                class: format!("p-6 border {}",
                                                    if balance.contains("-") {
                                                        "border-red-200 dark:border-red-800"
                                                    } else {
                                                        "border-green-200 dark:border-green-800"
                                                    }
                                                ),
                                                onclick: {
                                                    let name = name.to_string();
                                                    move |_| {
                                                        // 测试复杂的事件处理器
                                                        log::info!("Account clicked: {}", name);
                                                    }
                                                },
                                                div {
                                                    class: "flex items-center justify-between mb-3",
                                                    h3 { class: "text-lg font-semibold text-gray-900 dark:text-white", "{name}" }
                                                    span {
                                                        class: "text-xs px-2 py-1 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300",
                                                        "{account_type}"
                                                    }
                                                }
                                                p {
                                                    class: format!("text-2xl font-bold {}",
                                                        if balance.contains("-") {
                                                            "text-red-600 dark:text-red-400"
                                                        } else {
                                                            "text-green-600 dark:text-green-400"
                                                        }
                                                    ),
                                                    "{balance}"
                                                }
                                            }
                                        }

                                        // 添加新账户按钮
                                        div {
                                            class: "bg-gray-50 dark:bg-gray-700 rounded-lg border-2 border-dashed border-gray-300 dark:border-gray-600 p-6 flex items-center justify-center hover:border-emerald-400 transition-colors cursor-pointer",
                                            onclick: move |_| {
                                                // 这里可以打开新建账户弹窗
                                                log::info!("Add new account clicked");
                                            },
                                            div {
                                                class: "text-center",
                                                div {
                                                    class: "text-3xl mb-2 text-gray-400 dark:text-gray-500",
                                                    "+"
                                                }
                                                p {
                                                    class: "text-gray-600 dark:text-gray-400 text-sm font-medium",
                                                    "添加新账户"
                                                }
                                            }
                                        }
                                    }
                                },
                                Some(Err(e)) => rsx! {
                                    div {
                                        class: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6",
                                        p { class: "text-red-700 dark:text-red-300", "加载账户数据失败: {e:?}" }
                                    }
                                },
                                None => rsx! {
                                    div {
                                        class: "bg-gray-50 dark:bg-gray-800 rounded-lg p-6",
                                        div { class: "animate-pulse flex space-x-4",
                                            div { class: "rounded-full bg-gray-300 dark:bg-gray-600 h-12 w-12" }
                                            div { class: "flex-1 space-y-2 py-1",
                                                div { class: "h-4 bg-gray-300 dark:bg-gray-600 rounded w-3/4" }
                                                div { class: "h-4 bg-gray-300 dark:bg-gray-600 rounded w-1/2" }
                                            }
                                        }
                                        p { class: "text-center text-gray-500 dark:text-gray-400 mt-4", "正在加载账户数据..." }
                                    }
                                }
                            }
                        }
                    },
                    "transactions" => rsx! {
                        div {
                            class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6",
                            h2 { class: "text-xl font-bold mb-4", "交易记录" }
                            p { class: "text-gray-600 dark:text-gray-300", "交易记录功能开发中..." }
                        }
                    },
                    "stats" => rsx! {
                        div {
                            class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6",
                            h2 { class: "text-xl font-bold mb-4", "统计分析" }
                            p { class: "text-gray-600 dark:text-gray-300", "统计分析功能开发中..." }
                        }
                    },
                    _ => rsx! {
                        div { "未知页面" }
                    },
                }
            }
        }
    }
}
