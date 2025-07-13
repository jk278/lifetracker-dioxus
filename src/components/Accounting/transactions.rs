//! # 交易记录组件
//!
//! 显示交易列表、创建和编辑交易等功能

use dioxus::prelude::*;
use life_tracker::storage::{Transaction, TransactionType};

/// 交易记录组件的属性
#[derive(Props, Clone, PartialEq)]
pub struct TransactionsTabProps {
    /// 交易记录列表
    pub transactions: Vec<Transaction>,
    /// 创建交易回调
    pub on_create_transaction: EventHandler<()>,
    /// 编辑交易回调（传递交易ID）
    pub on_edit_transaction: EventHandler<uuid::Uuid>,
}

/// 交易记录标签页组件
#[component]
pub fn TransactionsTab(props: TransactionsTabProps) -> Element {
    // 复制数据的简单方法避免生命周期问题
    let transactions = props.transactions.clone();
    
    /// 格式化金额显示
    fn format_amount(amount: f64, currency: &str) -> String {
        match currency {
            "CNY" => format!("¥{:.2}", amount),
            "USD" => format!("${:.2}", amount),
            "EUR" => format!("€{:.2}", amount),
            _ => format!("{:.2} {}", amount, currency),
        }
    }

    /// 获取交易类型标签
    fn get_transaction_type_label(transaction_type: &TransactionType) -> &'static str {
        match transaction_type {
            TransactionType::Income => "收入",
            TransactionType::Expense => "支出",
            TransactionType::Transfer => "转账",
        }
    }

    /// 获取交易类型颜色样式
    fn get_transaction_type_color(transaction_type: &TransactionType) -> &'static str {
        match transaction_type {
            TransactionType::Income => {
                "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
            }
            TransactionType::Expense => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
            TransactionType::Transfer => {
                "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200"
            }
        }
    }

    /// 获取交易金额颜色
    fn get_transaction_amount_color(transaction_type: &TransactionType) -> &'static str {
        match transaction_type {
            TransactionType::Income => "text-green-600 dark:text-green-400",
            TransactionType::Expense => "text-red-600 dark:text-red-400",
            TransactionType::Transfer => "text-blue-600 dark:text-blue-400",
        }
    }

    /// 获取交易金额前缀
    fn get_transaction_amount_prefix(transaction_type: &TransactionType) -> &'static str {
        match transaction_type {
            TransactionType::Income => "+",
            TransactionType::Expense => "-",
            TransactionType::Transfer => "",
        }
    }

    rsx! {
        div { class: "space-y-6",
            // 标题和添加按钮
            div { class: "flex justify-between items-center",
                h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100",
                    "交易记录"
                }
                button {
                    class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors",
                    onclick: move |_| props.on_create_transaction.call(()),
                    "添加交易"
                }
            }

            // 大屏表格布局
            div { class: "hidden md:block bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 overflow-hidden",
                div { class: "overflow-x-auto",
                    table { class: "w-full",
                        thead { class: "bg-gray-50 dark:bg-gray-800",
                            tr {
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider",
                                    "类型"
                                }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider",
                                    "描述"
                                }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider",
                                    "账户"
                                }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider",
                                    "金额"
                                }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider",
                                    "日期"
                                }
                                th { class: "px-6 py-3 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider text-right",
                                    "操作"
                                }
                            }
                        }
                        tbody { class: "divide-y divide-gray-200 dark:divide-gray-700",
                            for transaction in &transactions {
                                tr {
                                    key: "{transaction.id}",
                                    class: "hover:bg-gray-50 dark:hover:bg-gray-800",

                                    // 交易类型
                                    td { class: "px-6 py-4 whitespace-nowrap",
                                        span { class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {get_transaction_type_color(&transaction.transaction_type)}",
                                            "{get_transaction_type_label(&transaction.transaction_type)}"
                                        }
                                    }

                                    // 描述
                                    td { class: "px-6 py-4",
                                        div { class: "text-sm font-medium text-gray-900 dark:text-gray-100",
                                            "{transaction.description}"
                                        }
                                    }

                                    // 账户
                                    td { class: "px-6 py-4",
                                        div { class: "text-sm text-gray-900 dark:text-gray-100",
                                            "账户 {transaction.account_id}"
                                        }
                                        if transaction.to_account_id.is_some() {
                                            div { class: "text-xs text-gray-500 dark:text-gray-400",
                                                "→ 目标账户"
                                            }
                                        }
                                    }

                                    // 金额
                                    td { class: "px-6 py-4",
                                        div { class: "text-sm font-medium {get_transaction_amount_color(&transaction.transaction_type)}",
                                            "{get_transaction_amount_prefix(&transaction.transaction_type)}{format_amount(transaction.amount, &transaction.currency)}"
                                        }
                                    }

                                    // 日期
                                    td { class: "px-6 py-4 text-sm text-gray-500 dark:text-gray-400",
                                        "{transaction.transaction_date}"
                                    }

                                    // 操作
                                    td { class: "px-6 py-4 whitespace-nowrap text-right",
                                        button {
                                            class: "text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 text-sm transition-colors",
                                            onclick: {
                                                let id = transaction.id;
                                                move |_| props.on_edit_transaction.call(id)
                                            },
                                            "编辑"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 小屏卡片布局
            div { class: "md:hidden space-y-4",
                for transaction in &transactions {
                    div {
                        key: "{transaction.id}",
                        class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-4",

                        // 第一行：类型标签 + 金额 + 编辑按钮
                        div { class: "flex justify-between items-center mb-2",
                            div { class: "flex items-center space-x-3",
                                span { class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {get_transaction_type_color(&transaction.transaction_type)}",
                                    "{get_transaction_type_label(&transaction.transaction_type)}"
                                }
                                div { class: "text-lg font-semibold {get_transaction_amount_color(&transaction.transaction_type)}",
                                    "{get_transaction_amount_prefix(&transaction.transaction_type)}{format_amount(transaction.amount, &transaction.currency)}"
                                }
                            }
                            button {
                                class: "text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 text-sm transition-colors px-3 py-1 rounded-md hover:bg-blue-50 dark:hover:bg-blue-900/20",
                                onclick: {
                                    let id = transaction.id;
                                    move |_| props.on_edit_transaction.call(id)
                                },
                                "编辑"
                            }
                        }

                        // 第二行：描述 + 账户信息 + 日期
                        div { class: "flex justify-between items-center text-sm",
                            div { class: "flex items-center space-x-2 flex-1 min-w-0",
                                span { class: "font-medium text-gray-900 dark:text-gray-100 truncate",
                                    "{transaction.description}"
                                }
                                span { class: "text-gray-500 dark:text-gray-400", "•" }
                                span { class: "text-gray-700 dark:text-gray-300",
                                    "账户 {transaction.account_id}"
                                }
                                if transaction.to_account_id.is_some() {
                                    span { class: "text-gray-500 dark:text-gray-400",
                                        "→ 目标账户"
                                    }
                                }
                            }
                            div { class: "text-xs text-gray-500 dark:text-gray-400 ml-2 flex-shrink-0",
                                "{transaction.transaction_date}"
                            }
                        }
                    }
                }
            }

            // 无数据状态
            if transactions.is_empty() {
                div { class: "text-center py-12",
                    div { class: "text-gray-400 text-6xl mb-4", "📊" }
                    h4 { class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-2", "暂无交易记录" }
                    p { class: "text-gray-600 dark:text-gray-400 mb-4", "点击上方按钮添加您的第一笔交易" }
                    button {
                        class: "px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors",
                        onclick: move |_| props.on_create_transaction.call(()),
                        "添加交易"
                    }
                }
            }
        }
    }
}
