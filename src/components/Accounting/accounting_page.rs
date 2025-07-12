//! # 财务管理主页面组件
//!
//! 财务管理模块的主入口，包含标签页导航

use super::{
    AccountsTab, AccountsTabProps, OverviewTab, OverviewTabProps, StatsTab, TransactionsTab,
    TransactionsTabProps,
};
use dioxus::prelude::*;
use life_tracker::storage::{Account, FinancialStats, Transaction};

/// 财务管理主页面组件
#[component]
pub fn AccountingPage() -> Element {
    // 状态管理
    let mut active_tab = use_signal(|| "overview");
    let mut accounts = use_signal(|| Vec::<Account>::new());
    let mut transactions = use_signal(|| Vec::<Transaction>::new());
    let mut financial_stats = use_signal(|| None::<FinancialStats>);
    let mut error = use_signal(|| None::<String>);

    // 弹窗状态
    let mut is_create_account_open = use_signal(|| false);
    let mut is_create_transaction_open = use_signal(|| false);

    // 模拟数据获取（将来会替换为真实的API调用）
    let _fetch_data = use_resource(move || async move {
        // 这里将来会调用真实的数据获取函数
        // 现在使用模拟数据
        accounts.set(vec![]);
        transactions.set(vec![]);
        financial_stats.set(None);
    });

    // 处理创建账户
    let handle_create_account = move |_| {
        is_create_account_open.set(true);
    };

    // 处理创建交易
    let handle_create_transaction = move |_| {
        is_create_transaction_open.set(true);
    };

    // 处理编辑交易
    let handle_edit_transaction = move |_transaction: Transaction| {
        // 这里处理编辑交易逻辑
        // 现在暂时只是打印日志
        log::info!("Edit transaction: {}", _transaction.id);
    };

    // 关闭弹窗
    let close_create_account = move |_| {
        is_create_account_open.set(false);
    };

    let close_create_transaction = move |_| {
        is_create_transaction_open.set(false);
    };

    // 渲染活动标签页内容
    let render_active_tab = move || match active_tab.read().as_ref() {
        "overview" => rsx! {
            OverviewTab {
                accounts: accounts.read().clone(),
                financial_stats: financial_stats.read().clone(),
                transactions: transactions.read().clone(),
            }
        },
        "accounts" => rsx! {
            AccountsTab {
                accounts: accounts.read().clone(),
                on_create_account: handle_create_account,
            }
        },
        "transactions" => rsx! {
            TransactionsTab {
                transactions: transactions.read().clone(),
                on_create_transaction: handle_create_transaction,
                on_edit_transaction: handle_edit_transaction,
            }
        },
        "stats" => rsx! {
            StatsTab {
                format_amount: |amount, _currency| format!("￥{:.2}", amount),
            }
        },
        _ => rsx! {
            div { "未知页面" }
        },
    };

    rsx! {
        div { class: "flex flex-col h-full",
            // 标签页导航
            div { class: "flex-shrink-0 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 overflow-x-auto sticky top-0 z-10 pt-2 md:pt-4",
                div { class: "flex px-0 md:px-6",
                    // 概览标签
                    div { class: "relative",
                        button {
                            class: if *active_tab.read() == "overview" {
                                "px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap text-blue-600"
                            } else {
                                "px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                            },
                            onclick: move |_| active_tab.set("overview"),
                            "财务概览"
                        }
                        // 选中指示器
                        div { class: if *active_tab.read() == "overview" {
                            "absolute bottom-0 left-1/2 transform -translate-x-1/2 h-0.5 bg-blue-600 transition-all duration-300 ease-out w-8 opacity-100"
                        } else {
                            "absolute bottom-0 left-1/2 transform -translate-x-1/2 h-0.5 bg-blue-600 transition-all duration-300 ease-out w-0 opacity-0"
                        } }
                    }

                    // 账户标签
                    div { class: "relative",
                        button {
                            class: if *active_tab.read() == "accounts" {
                                "px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap text-blue-600"
                            } else {
                                "px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                            },
                            onclick: move |_| active_tab.set("accounts"),
                            "账户管理"
                        }
                        div { class: if *active_tab.read() == "accounts" {
                            "absolute bottom-0 left-1/2 transform -translate-x-1/2 h-0.5 bg-blue-600 transition-all duration-300 ease-out w-8 opacity-100"
                        } else {
                            "absolute bottom-0 left-1/2 transform -translate-x-1/2 h-0.5 bg-blue-600 transition-all duration-300 ease-out w-0 opacity-0"
                        } }
                    }

                    // 交易标签
                    div { class: "relative",
                        button {
                            class: if *active_tab.read() == "transactions" {
                                "px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap text-blue-600"
                            } else {
                                "px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                            },
                            onclick: move |_| active_tab.set("transactions"),
                            "交易明细"
                        }
                        div { class: if *active_tab.read() == "transactions" {
                            "absolute bottom-0 left-1/2 transform -translate-x-1/2 h-0.5 bg-blue-600 transition-all duration-300 ease-out w-8 opacity-100"
                        } else {
                            "absolute bottom-0 left-1/2 transform -translate-x-1/2 h-0.5 bg-blue-600 transition-all duration-300 ease-out w-0 opacity-0"
                        } }
                    }

                    // 统计标签
                    div { class: "relative",
                        button {
                            class: if *active_tab.read() == "stats" {
                                "px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap text-blue-600"
                            } else {
                                "px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                            },
                            onclick: move |_| active_tab.set("stats"),
                            "统计分析"
                        }
                        div { class: if *active_tab.read() == "stats" {
                            "absolute bottom-0 left-1/2 transform -translate-x-1/2 h-0.5 bg-blue-600 transition-all duration-300 ease-out w-8 opacity-100"
                        } else {
                            "absolute bottom-0 left-1/2 transform -translate-x-1/2 h-0.5 bg-blue-600 transition-all duration-300 ease-out w-0 opacity-0"
                        } }
                    }
                }
            }

            // 错误提示
            if let Some(error_msg) = error.read().as_ref() {
                div { class: "mx-4 md:mx-6 mt-4 p-4 bg-red-100 dark:bg-red-900 border border-red-300 dark:border-red-700 rounded-lg",
                    p { class: "text-red-700 dark:text-red-300", "{error_msg}" }
                }
            }

            // 内容区域
            div { class: "flex-1 overflow-y-auto py-4 px-4 md:px-6",
                {render_active_tab()}
            }

            // 创建账户弹窗
            if is_create_account_open() {
                div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                    onclick: close_create_account,

                    div { class: "bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md p-6",
                        onclick: move |e| e.stop_propagation(),

                        h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                            "创建账户"
                        }

                        div { class: "space-y-4",
                            p { class: "text-gray-600 dark:text-gray-400", "创建账户功能开发中..." }
                        }

                        div { class: "flex justify-end space-x-3 mt-6",
                            button {
                                class: "px-4 py-2 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors",
                                onclick: close_create_account,
                                "取消"
                            }
                            button {
                                class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors",
                                onclick: close_create_account,
                                "创建"
                            }
                        }
                    }
                }
            }

            // 创建交易弹窗
            if is_create_transaction_open() {
                div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                    onclick: close_create_transaction,

                    div { class: "bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md p-6",
                        onclick: move |e| e.stop_propagation(),

                        h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                            "创建交易"
                        }

                        div { class: "space-y-4",
                            p { class: "text-gray-600 dark:text-gray-400", "创建交易功能开发中..." }
                        }

                        div { class: "flex justify-end space-x-3 mt-6",
                            button {
                                class: "px-4 py-2 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors",
                                onclick: close_create_transaction,
                                "取消"
                            }
                            button {
                                class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors",
                                onclick: close_create_transaction,
                                "创建"
                            }
                        }
                    }
                }
            }
        }
    }
}
