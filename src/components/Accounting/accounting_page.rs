//! # 财务管理主页面组件
//!
//! 财务管理模块的主入口，包含标签页导航

// use super::{
//     AccountsTab, OverviewTab, StatsTab, TransactionsTab,
// }; // 暂时注释未使用的导入
use dioxus::prelude::*;
// use dioxus_free_icons::{icons::bs_icons::*, Icon}; // 暂时注释未使用的导入
use life_tracker::storage::{Account, FinancialStats, Transaction};

/// 财务管理主页面组件
#[component]
pub fn AccountingPage() -> Element {
    // 状态管理
    let mut active_tab = use_signal(|| "overview");
    let mut accounts = use_signal(|| Vec::<Account>::new());
    let mut transactions = use_signal(|| Vec::<Transaction>::new());
    let mut financial_stats = use_signal(|| None::<FinancialStats>);
    let error = use_signal(|| None::<String>);

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
    let handle_edit_transaction = move |transaction_id: uuid::Uuid| {
        // 这里处理编辑交易逻辑
        // 现在暂时只是打印日志
        log::info!("Edit transaction: {}", transaction_id);
    };

    // 关闭弹窗
    let close_create_account = move |_| {
        is_create_account_open.set(false);
    };

    let close_create_transaction = move |_| {
        is_create_transaction_open.set(false);
    };


    // 预计算复杂的 class 字符串
    let overview_tab_class = if *active_tab.read() == "overview" {
        "group flex items-center space-x-3 py-4 px-6 font-medium text-sm transition-all duration-300 rounded-t-xl relative text-emerald-600 dark:text-emerald-400 bg-white dark:bg-gray-700 shadow-lg transform -translate-y-1"
    } else {
        "group flex items-center space-x-3 py-4 px-6 font-medium text-sm transition-all duration-300 rounded-t-xl relative text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-white/50 dark:hover:bg-gray-700/50 hover:shadow-md hover:-translate-y-0.5"
    };

    let accounts_tab_class = if *active_tab.read() == "accounts" {
        "group flex items-center space-x-3 py-4 px-6 font-medium text-sm transition-all duration-300 rounded-t-xl relative text-emerald-600 dark:text-emerald-400 bg-white dark:bg-gray-700 shadow-lg transform -translate-y-1"
    } else {
        "group flex items-center space-x-3 py-4 px-6 font-medium text-sm transition-all duration-300 rounded-t-xl relative text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-white/50 dark:hover:bg-gray-700/50 hover:shadow-md hover:-translate-y-0.5"
    };

    let transactions_tab_class = if *active_tab.read() == "transactions" {
        "group flex items-center space-x-3 py-4 px-6 font-medium text-sm transition-all duration-300 rounded-t-xl relative text-emerald-600 dark:text-emerald-400 bg-white dark:bg-gray-700 shadow-lg transform -translate-y-1"
    } else {
        "group flex items-center space-x-3 py-4 px-6 font-medium text-sm transition-all duration-300 rounded-t-xl relative text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-white/50 dark:hover:bg-gray-700/50 hover:shadow-md hover:-translate-y-0.5"
    };

    let stats_tab_class = if *active_tab.read() == "stats" {
        "group flex items-center space-x-3 py-4 px-6 font-medium text-sm transition-all duration-300 rounded-t-xl relative text-emerald-600 dark:text-emerald-400 bg-white dark:bg-gray-700 shadow-lg transform -translate-y-1"
    } else {
        "group flex items-center space-x-3 py-4 px-6 font-medium text-sm transition-all duration-300 rounded-t-xl relative text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-white/50 dark:hover:bg-gray-700/50 hover:shadow-md hover:-translate-y-0.5"
    };

    rsx! {
        div { 
            class: "min-h-screen bg-gradient-to-br from-emerald-50 via-green-50 to-teal-100 dark:from-gray-900 dark:via-gray-800 dark:to-gray-700",
            
            // 现代化标签页导航
            div { 
                class: "bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm shadow-lg border-b border-emerald-200/50 dark:border-emerald-700/50 sticky top-0 z-10",
                div { 
                    class: "container mx-auto px-6",
                    nav { 
                        class: "flex space-x-2",
                        
                        // 财务概览标签
                        button {
                            class: overview_tab_class,
                            onclick: move |_| active_tab.set("overview"),
                            
                            if *active_tab.read() == "overview" {
                                div {
                                    class: "absolute bottom-0 left-1/2 transform -translate-x-1/2 w-12 h-1 bg-emerald-500 rounded-t-full"
                                }
                            }
                            
                            div {
                                class: if *active_tab.read() == "overview" {
                                    "transition-all duration-300 text-emerald-500 scale-110"
                                } else {
                                    "transition-all duration-300 text-gray-400 group-hover:text-gray-600 group-hover:scale-105"
                                },
                                span { class: "w-5 h-5 text-xl", "🏠" }
                            }
                            span { 
                                class: "font-semibold",
                                "财务概览" 
                            }
                        }

                        // 账户管理标签
                        button {
                            class: accounts_tab_class,
                            onclick: move |_| active_tab.set("accounts"),
                            
                            if *active_tab.read() == "accounts" {
                                div {
                                    class: "absolute bottom-0 left-1/2 transform -translate-x-1/2 w-12 h-1 bg-emerald-500 rounded-t-full"
                                }
                            }
                            
                            div {
                                class: if *active_tab.read() == "accounts" {
                                    "transition-all duration-300 text-emerald-500 scale-110"
                                } else {
                                    "transition-all duration-300 text-gray-400 group-hover:text-gray-600 group-hover:scale-105"
                                },
                                span { class: "w-5 h-5 text-xl", "💰" }
                            }
                            span { 
                                class: "font-semibold",
                                "账户管理" 
                            }
                        }

                        // 交易记录标签
                        button {
                            class: transactions_tab_class,
                            onclick: move |_| active_tab.set("transactions"),
                            
                            if *active_tab.read() == "transactions" {
                                div {
                                    class: "absolute bottom-0 left-1/2 transform -translate-x-1/2 w-12 h-1 bg-emerald-500 rounded-t-full"
                                }
                            }
                            
                            div {
                                class: if *active_tab.read() == "transactions" {
                                    "transition-all duration-300 text-emerald-500 scale-110"
                                } else {
                                    "transition-all duration-300 text-gray-400 group-hover:text-gray-600 group-hover:scale-105"
                                },
                                span { class: "w-5 h-5 text-xl", "💱" }
                            }
                            span { 
                                class: "font-semibold",
                                "交易记录" 
                            }
                        }

                        // 统计分析标签
                        button {
                            class: stats_tab_class,
                            onclick: move |_| active_tab.set("stats"),
                            
                            if *active_tab.read() == "stats" {
                                div {
                                    class: "absolute bottom-0 left-1/2 transform -translate-x-1/2 w-12 h-1 bg-emerald-500 rounded-t-full"
                                }
                            }
                            
                            div {
                                class: if *active_tab.read() == "stats" {
                                    "transition-all duration-300 text-emerald-500 scale-110"
                                } else {
                                    "transition-all duration-300 text-gray-400 group-hover:text-gray-600 group-hover:scale-105"
                                },
                                span { class: "w-5 h-5 text-xl", "📊" }
                            }
                            span { 
                                class: "font-semibold",
                                "统计分析" 
                            }
                        }
                    }
                }
            }

            // 错误提示
            if let Some(error_msg) = error.read().as_ref() {
                div { class: "mx-4 md:mx-6 mt-4 p-4 bg-red-100 dark:bg-red-900 border border-red-300 dark:border-red-700 rounded-lg",
                    p { class: "text-red-700 dark:text-red-300", "{error_msg}" }
                }
            }

            // 内容区域 - 添加动画过渡
            div { class: "container mx-auto px-6 py-8",
                div {
                    class: "animate-fade-in",
                    match active_tab.read().as_ref() {
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
                            StatsTab {}
                        },
                        _ => rsx! {
                            div { "未知页面" }
                        },
                    }
                }
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
