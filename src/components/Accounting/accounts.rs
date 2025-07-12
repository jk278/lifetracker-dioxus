//! # 账户管理组件
//!
//! 显示账户列表、创建和编辑账户等功能

use dioxus::prelude::*;
use life_tracker::storage::{Account, AccountType};

/// 账户管理组件的属性
#[derive(Props, Clone, PartialEq)]
pub struct AccountsTabProps {
    /// 账户列表
    pub accounts: Vec<Account>,
    /// 创建账户回调
    pub on_create_account: EventHandler<()>,
}

/// 账户管理标签页组件
#[component]
pub fn AccountsTab(props: AccountsTabProps) -> Element {
    /// 格式化金额显示
    fn format_amount(amount: f64, currency: &str) -> String {
        match currency {
            "CNY" => format!("¥{:.2}", amount),
            "USD" => format!("${:.2}", amount),
            "EUR" => format!("€{:.2}", amount),
            _ => format!("{:.2} {}", amount, currency),
        }
    }

    /// 获取账户类型标签
    fn get_account_type_label(account_type: &AccountType) -> &'static str {
        match account_type {
            AccountType::Cash => "现金",
            AccountType::Bank => "银行账户",
            AccountType::CreditCard => "信用卡",
            AccountType::Investment => "投资账户",
            AccountType::Other => "其他",
        }
    }

    rsx! {
        div { class: "space-y-6",
            // 标题和添加按钮
            div { class: "flex justify-between items-center",
                h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100",
                    "账户管理"
                }
                button {
                    class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors",
                    onclick: move |_| props.on_create_account.call(()),
                    "添加账户"
                }
            }

            // 账户网格
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                for account in props.accounts.iter() {
                    div {
                        key: "{account.id}",
                        class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-sm hover:shadow-md transition-shadow p-6",

                        // 账户标题和默认标识
                        div { class: "flex items-center justify-between mb-4",
                            h4 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100",
                                "{account.name}"
                            }
                            if account.is_default {
                                span { class: "px-2 py-1 bg-blue-100 dark:bg-blue-900/20 text-blue-800 dark:text-blue-400 text-xs rounded-full",
                                    "默认"
                                }
                            }
                        }

                        // 账户类型
                        p { class: "text-sm text-gray-500 dark:text-gray-400 mb-2",
                            "{get_account_type_label(&account.account_type)}"
                        }

                        // 账户余额
                        p { class: "text-2xl font-bold text-gray-900 dark:text-gray-100",
                            "{format_amount(account.balance, &account.currency)}"
                        }

                        // 账户描述（可选）
                        if let Some(description) = &account.description {
                            p { class: "text-sm text-gray-500 dark:text-gray-400 mt-2",
                                "{description}"
                            }
                        }

                        // 账户状态指示器
                        if !account.is_active {
                            div { class: "mt-3 flex items-center text-sm text-gray-500 dark:text-gray-400",
                                span { class: "w-2 h-2 bg-red-500 rounded-full mr-2" }
                                "已停用"
                            }
                        }
                    }
                }
            }

            // 无账户状态
            if props.accounts.is_empty() {
                div { class: "text-center py-12",
                    div { class: "text-gray-400 text-6xl mb-4", "💳" }
                    h4 { class: "text-lg font-medium text-gray-900 dark:text-gray-100 mb-2", "暂无账户" }
                    p { class: "text-gray-600 dark:text-gray-400 mb-4", "点击上方按钮创建您的第一个账户" }
                    button {
                        class: "px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors",
                        onclick: move |_| props.on_create_account.call(()),
                        "创建账户"
                    }
                }
            }
        }
    }
}
