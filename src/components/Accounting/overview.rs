//! # 财务概览组件
//!
//! 显示财务统计卡片和最近交易记录

use dioxus::prelude::*;
use life_tracker::storage::{Account, FinancialStats, Transaction, TransactionType};

/// 财务概览组件的属性
#[derive(Props, Clone, PartialEq)]
pub struct OverviewTabProps {
    /// 账户列表
    pub accounts: Vec<Account>,
    /// 财务统计数据
    pub financial_stats: Option<FinancialStats>,
    /// 交易记录列表
    pub transactions: Vec<Transaction>,
}

/// 财务概览标签页组件
#[component]
pub fn OverviewTab(props: OverviewTabProps) -> Element {
    /// 格式化金额显示
    fn format_amount(amount: f64, currency: Option<&str>) -> String {
        let currency = currency.unwrap_or("CNY");
        match currency {
            "CNY" => format!("¥{:.2}", amount),
            "USD" => format!("${:.2}", amount),
            "EUR" => format!("€{:.2}", amount),
            _ => format!("{:.2} {}", amount, currency),
        }
    }

    /// 计算总余额
    fn calculate_total_balance(accounts: &[Account]) -> f64 {
        accounts.iter().map(|acc| acc.balance).sum()
    }

    /// 获取交易类型颜色
    fn get_transaction_type_color(transaction_type: &TransactionType) -> &'static str {
        match transaction_type {
            TransactionType::Income => "bg-green-500",
            TransactionType::Expense => "bg-red-500",
            TransactionType::Transfer => "bg-blue-500",
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
            // 标题
            div { class: "flex justify-between items-center mb-2",
                h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100",
                    "财务概览"
                }
            }

            // 统计卡片区
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                // 总余额卡片
                div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6",
                    div { class: "flex items-center",
                        div { class: "flex-shrink-0 text-3xl", "💰" }
                        div { class: "ml-4",
                            p { class: "text-sm font-medium text-gray-500 dark:text-gray-400",
                                "总余额"
                            }
                            p { class: "text-2xl font-semibold text-gray-900 dark:text-white",
                                "{format_amount(calculate_total_balance(&props.accounts), Some(\"CNY\"))}"
                            }
                        }
                    }
                }

                // 本月收入卡片
                div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6",
                    div { class: "flex items-center",
                        div { class: "flex-shrink-0 text-3xl", "📈" }
                        div { class: "ml-4",
                            p { class: "text-sm font-medium text-gray-500 dark:text-gray-400",
                                "本月收入"
                            }
                            p { class: "text-2xl font-semibold text-gray-900 dark:text-white",
                                if let Some(stats) = &props.financial_stats {
                                    "{format_amount(stats.total_income, Some(&stats.currency))}"
                                } else {
                                    "¥0.00"
                                }
                            }
                        }
                    }
                }

                // 本月支出卡片
                div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6",
                    div { class: "flex items-center",
                        div { class: "flex-shrink-0 text-3xl", "📉" }
                        div { class: "ml-4",
                            p { class: "text-sm font-medium text-gray-500 dark:text-gray-400",
                                "本月支出"
                            }
                            p { class: "text-2xl font-semibold text-gray-900 dark:text-white",
                                if let Some(stats) = &props.financial_stats {
                                    "{format_amount(stats.total_expense, Some(&stats.currency))}"
                                } else {
                                    "¥0.00"
                                }
                            }
                        }
                    }
                }

                // 净收入卡片
                div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6",
                    div { class: "flex items-center",
                        div { class: "flex-shrink-0 text-3xl", "💎" }
                        div { class: "ml-4",
                            p { class: "text-sm font-medium text-gray-500 dark:text-gray-400",
                                "净收入"
                            }
                            p { class: "text-2xl font-semibold text-gray-900 dark:text-white",
                                if let Some(stats) = &props.financial_stats {
                                    "{format_amount(stats.net_income, Some(&stats.currency))}"
                                } else {
                                    "¥0.00"
                                }
                            }
                        }
                    }
                }
            }

            // 最近交易卡片
            div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6",
                h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                    "最近交易"
                }
                div { class: "space-y-4",
                    // 显示最近5笔交易
                    for transaction in props.transactions.iter().take(5) {
                        div {
                            key: "{transaction.id}",
                            class: "flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-lg",

                            // 交易信息
                            div { class: "flex items-center space-x-4",
                                // 交易类型指示器
                                div { class: "w-3 h-3 rounded-full {get_transaction_type_color(&transaction.transaction_type)}" }

                                // 交易详情
                                div {
                                    p { class: "font-medium text-gray-900 dark:text-gray-100",
                                        "{transaction.description}"
                                    }
                                    p { class: "text-sm text-gray-500 dark:text-gray-400",
                                        "账户 • {transaction.transaction_date}"
                                    }
                                }
                            }

                            // 交易金额
                            div { class: "text-lg font-semibold {get_transaction_amount_color(&transaction.transaction_type)}",
                                "{get_transaction_amount_prefix(&transaction.transaction_type)}{format_amount(transaction.amount, Some(&transaction.currency))}"
                            }
                        }
                    }
                }

                // 无交易数据状态
                if props.transactions.is_empty() {
                    div { class: "text-center py-8",
                        div { class: "text-gray-400 text-4xl mb-4", "📊" }
                        p { class: "text-gray-500 dark:text-gray-400", "暂无交易记录" }
                    }
                }
            }
        }
    }
}
