//! # 数据管理页面
//!
//! 数据管理模块的主页面，包含数据统计和功能入口

use crate::components::data_management::{DataExport, DataImport};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct DataManagementPageProps {
    pub show_back_button: bool,
    pub on_back: Option<EventHandler<()>>,
}

// 数据统计结构
#[derive(Debug, Clone, PartialEq)]
struct DataStatistics {
    total_tasks: u32,
    total_time_spent: u64,
    total_transactions: u32,
    total_notes: u32,
    database_size: String,
    last_backup: String,
}

impl Default for DataStatistics {
    fn default() -> Self {
        Self {
            total_tasks: 0,
            total_time_spent: 0,
            total_transactions: 0,
            total_notes: 0,
            database_size: "未知".to_string(),
            last_backup: "从未".to_string(),
        }
    }
}

// 功能卡片结构
#[derive(Debug, Clone, PartialEq)]
struct FeatureCard {
    id: String,
    icon: String,
    title: String,
    description: String,
    color: String,
    bg_color: String,
}

// 操作状态
#[derive(Debug, Clone, PartialEq)]
enum OperationStatus {
    None,
    Success(String),
    Error(String),
}

// 页面状态
#[derive(Debug, Clone, PartialEq)]
enum PageState {
    Overview,
    Export,
    Import,
    Backup,
    Sync,
    Cleanup,
}

#[component]
pub fn DataManagementPage(props: DataManagementPageProps) -> Element {
    // 状态管理
    let statistics = use_signal(|| DataStatistics::default());
    let loading = use_signal(|| false);
    let operation_status = use_signal(|| OperationStatus::None);
    let current_page = use_signal(|| PageState::Overview);

    // 功能卡片数据
    let features = use_memo(|| {
        vec![
            FeatureCard {
                id: "export".to_string(),
                icon: "📤".to_string(),
                title: "数据导出".to_string(),
                description: "导出任务、财务、笔记等数据".to_string(),
                color: "text-blue-600 dark:text-blue-400".to_string(),
                bg_color: "bg-blue-50 dark:bg-blue-900/20".to_string(),
            },
            FeatureCard {
                id: "import".to_string(),
                icon: "📥".to_string(),
                title: "数据导入".to_string(),
                description: "从备份文件导入数据".to_string(),
                color: "text-green-600 dark:text-green-400".to_string(),
                bg_color: "bg-green-50 dark:bg-green-900/20".to_string(),
            },
            FeatureCard {
                id: "backup".to_string(),
                icon: "💾".to_string(),
                title: "备份与恢复".to_string(),
                description: "创建备份和从备份恢复数据".to_string(),
                color: "text-purple-600 dark:text-purple-400".to_string(),
                bg_color: "bg-purple-50 dark:bg-purple-900/20".to_string(),
            },
            FeatureCard {
                id: "sync".to_string(),
                icon: "☁️".to_string(),
                title: "多端同步".to_string(),
                description: "配置 WebDAV 云同步".to_string(),
                color: "text-indigo-600 dark:text-indigo-400".to_string(),
                bg_color: "bg-indigo-50 dark:bg-indigo-900/20".to_string(),
            },
            FeatureCard {
                id: "cleanup".to_string(),
                icon: "🗑️".to_string(),
                title: "数据清理".to_string(),
                description: "永久删除所有数据（危险操作）".to_string(),
                color: "text-red-600 dark:text-red-400".to_string(),
                bg_color: "bg-red-50 dark:bg-red-900/20".to_string(),
            },
        ]
    });

    // 获取数据统计信息
    let fetch_statistics = {
        let mut statistics = statistics.clone();
        let mut loading = loading.clone();
        let mut operation_status = operation_status.clone();

        move || {
            spawn(async move {
                loading.set(true);
                operation_status.set(OperationStatus::None);

                // 模拟数据获取（将来会替换为真实的API调用）
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                match get_data_statistics().await {
                    Ok(stats) => {
                        statistics.set(stats);
                        log::info!("Data statistics loaded successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to load data statistics: {}", e);
                        operation_status.set(OperationStatus::Error(
                            "获取数据统计失败，请重试".to_string(),
                        ));
                    }
                }

                loading.set(false);
            });
        }
    };

    // 初始化加载数据
    use_effect(move || {
        if matches!(current_page.read().clone(), PageState::Overview) {
            fetch_statistics();
        }
    });

    // 处理功能卡片点击
    let handle_feature_click = {
        let mut current_page = current_page.clone();
        let mut operation_status = operation_status.clone();

        move |feature_id: String| match feature_id.as_str() {
            "export" => {
                current_page.set(PageState::Export);
                log::info!("Navigate to data export");
            }
            "import" => {
                current_page.set(PageState::Import);
                log::info!("Navigate to data import");
            }
            "backup" => {
                current_page.set(PageState::Backup);
                operation_status.set(OperationStatus::Success(
                    "备份恢复功能开发中...".to_string(),
                ));
                log::info!("Navigate to data backup");
            }
            "sync" => {
                current_page.set(PageState::Sync);
                operation_status.set(OperationStatus::Success(
                    "多端同步功能开发中...".to_string(),
                ));
                log::info!("Navigate to data sync");
            }
            "cleanup" => {
                current_page.set(PageState::Cleanup);
                operation_status.set(OperationStatus::Success(
                    "数据清理功能开发中...".to_string(),
                ));
                log::info!("Navigate to data cleanup");
            }
            _ => {
                operation_status.set(OperationStatus::Error("未知功能".to_string()));
            }
        }
    };

    // 处理返回到概览页面
    let mut handle_back_to_overview = {
        let mut current_page = current_page.clone();
        let mut operation_status = operation_status.clone();

        move || {
            current_page.set(PageState::Overview);
            operation_status.set(OperationStatus::None);
        }
    };

    // 格式化时间显示
    let format_time = |seconds: u64| {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        format!("{}小时{}分钟", hours, minutes)
    };

    // 格式化数字显示
    let format_number = |num: u32| {
        if num >= 1000 {
            format!("{:.1}K", num as f64 / 1000.0)
        } else {
            num.to_string()
        }
    };

    // 根据当前页面状态渲染内容
    let page_state = current_page.read().clone();
    match page_state {
        PageState::Overview => rsx! {
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
                            h1 { class: "text-2xl font-bold text-gray-900 dark:text-white",
                                "数据管理"
                            }
                        }
                        // 刷新按钮
                        button {
                            class: "flex items-center justify-center w-8 h-8 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors",
                            onclick: move |_| fetch_statistics(),
                            title: "刷新数据",
                            disabled: loading(),
                            if loading() { "⟳" } else { "🔄" }
                        }
                    }
                }

                // 可滚动内容区域
                div { class: "flex-1 overflow-y-auto py-4 px-4 md:px-6",
                    div { class: "max-w-6xl mx-auto space-y-6",

                        // 操作状态提示
                        match operation_status.read().clone() {
                            OperationStatus::Success(msg) => rsx! {
                                div { class: "p-4 bg-green-100 dark:bg-green-900/20 border border-green-300 dark:border-green-700 rounded-lg",
                                    p { class: "text-green-700 dark:text-green-300", "{msg}" }
                                }
                            },
                            OperationStatus::Error(msg) => rsx! {
                                div { class: "p-4 bg-red-100 dark:bg-red-900/20 border border-red-300 dark:border-red-700 rounded-lg",
                                    p { class: "text-red-700 dark:text-red-300", "{msg}" }
                                }
                            },
                            OperationStatus::None => rsx! { div {} }
                        }

                        // 数据统计卡片
                        div { class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700",
                            div { class: "p-6",
                                h2 { class: "text-lg font-semibold text-gray-900 dark:text-white mb-4",
                                    "📊 数据统计"
                                }

                                if loading() {
                                    div { class: "flex justify-center items-center h-24",
                                        div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" }
                                    }
                                } else {
                                    div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4",
                                        // 任务数量
                                        div { class: "text-center p-4 bg-gray-50 dark:bg-gray-700 rounded-lg",
                                            div { class: "text-2xl font-bold text-blue-600 dark:text-blue-400",
                                                "{format_number(statistics.read().total_tasks)}"
                                            }
                                            div { class: "text-sm text-gray-600 dark:text-gray-400",
                                                "任务数量"
                                            }
                                        }

                                        // 总时长
                                        div { class: "text-center p-4 bg-gray-50 dark:bg-gray-700 rounded-lg",
                                            div { class: "text-2xl font-bold text-green-600 dark:text-green-400",
                                                "{format_time(statistics.read().total_time_spent)}"
                                            }
                                            div { class: "text-sm text-gray-600 dark:text-gray-400",
                                                "总时长"
                                            }
                                        }

                                        // 交易数量
                                        div { class: "text-center p-4 bg-gray-50 dark:bg-gray-700 rounded-lg",
                                            div { class: "text-2xl font-bold text-purple-600 dark:text-purple-400",
                                                "{format_number(statistics.read().total_transactions)}"
                                            }
                                            div { class: "text-sm text-gray-600 dark:text-gray-400",
                                                "交易记录"
                                            }
                                        }

                                        // 笔记数量
                                        div { class: "text-center p-4 bg-gray-50 dark:bg-gray-700 rounded-lg",
                                            div { class: "text-2xl font-bold text-orange-600 dark:text-orange-400",
                                                "{format_number(statistics.read().total_notes)}"
                                            }
                                            div { class: "text-sm text-gray-600 dark:text-gray-400",
                                                "笔记数量"
                                            }
                                        }

                                        // 数据库大小
                                        div { class: "text-center p-4 bg-gray-50 dark:bg-gray-700 rounded-lg",
                                            div { class: "text-2xl font-bold text-indigo-600 dark:text-indigo-400",
                                                "{statistics.read().database_size}"
                                            }
                                            div { class: "text-sm text-gray-600 dark:text-gray-400",
                                                "数据库大小"
                                            }
                                        }

                                        // 最后备份
                                        div { class: "text-center p-4 bg-gray-50 dark:bg-gray-700 rounded-lg",
                                            div { class: "text-2xl font-bold text-red-600 dark:text-red-400",
                                                "{statistics.read().last_backup}"
                                            }
                                            div { class: "text-sm text-gray-600 dark:text-gray-400",
                                                "最后备份"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 功能卡片网格
                        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                            for feature in features.read().iter() {
                                button {
                                    key: "{feature.id}",
                                    class: "p-6 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-600 dark:hover:border-blue-400 transition-all duration-200 text-left group shadow-lg hover:shadow-xl",
                                    onclick: {
                                        let feature_id = feature.id.clone();
                                        let handle_click = handle_feature_click.clone();
                                        move |_| handle_click(feature_id.clone())
                                    },

                                    div { class: "flex items-center mb-4",
                                        div { class: "w-12 h-12 {feature.bg_color} rounded-lg flex items-center justify-center group-hover:scale-105 transition-transform",
                                            span { class: "text-2xl", "{feature.icon}" }
                                        }
                                    }
                                    h3 { class: "text-lg font-semibold text-gray-900 dark:text-white mb-2",
                                        "{feature.title}"
                                    }
                                    p { class: "text-sm text-gray-600 dark:text-gray-400",
                                        "{feature.description}"
                                    }
                                }
                            }
                        }

                        // 说明文本
                        div { class: "bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-700 rounded-lg p-4",
                            div { class: "flex items-start space-x-3",
                                div { class: "flex-shrink-0 mt-1",
                                    span { class: "text-blue-600 dark:text-blue-400", "💡" }
                                }
                                div { class: "text-sm text-blue-800 dark:text-blue-200",
                                    h4 { class: "font-semibold mb-2", "数据管理说明" }
                                    ul { class: "space-y-1",
                                        li { "• 数据导出：将您的数据导出为 JSON、CSV 或其他格式" }
                                        li { "• 数据导入：从之前的备份文件导入数据" }
                                        li { "• 备份与恢复：创建完整的数据备份，并在需要时恢复" }
                                        li { "• 多端同步：通过 WebDAV 在不同设备间同步数据" }
                                        li { "• 数据清理：永久删除所有数据，请谨慎操作" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },

        PageState::Export => rsx! {
            DataExport {
                show_back_button: true,
                on_back: move |_| handle_back_to_overview(),
            }
        },

        PageState::Import => rsx! {
            DataImport {
                show_back_button: true,
                on_back: move |_| handle_back_to_overview(),
            }
        },

        PageState::Backup => rsx! {
            div { class: "h-full flex flex-col",
                div { class: "flex-shrink-0 px-4 md:px-6 py-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800",
                    div { class: "flex items-center justify-between",
                        div { class: "flex items-center space-x-3",
                            button {
                                class: "flex items-center justify-center w-8 h-8 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors",
                                onclick: move |_| handle_back_to_overview(),
                                title: "返回",
                                "←"
                            }
                            h1 { class: "text-2xl font-bold text-gray-900 dark:text-white",
                                "数据备份"
                            }
                        }
                    }
                }
                div { class: "flex-1 p-6 text-center",
                    div { class: "text-6xl mb-4", "💾" }
                    h3 { class: "text-xl font-semibold text-gray-900 dark:text-white mb-2",
                        "数据备份功能"
                    }
                    p { class: "text-gray-600 dark:text-gray-400",
                        "备份恢复功能正在开发中..."
                    }
                }
            }
        },

        PageState::Sync => rsx! {
            div { class: "h-full flex flex-col",
                div { class: "flex-shrink-0 px-4 md:px-6 py-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800",
                    div { class: "flex items-center justify-between",
                        div { class: "flex items-center space-x-3",
                            button {
                                class: "flex items-center justify-center w-8 h-8 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors",
                                onclick: move |_| handle_back_to_overview(),
                                title: "返回",
                                "←"
                            }
                            h1 { class: "text-2xl font-bold text-gray-900 dark:text-white",
                                "多端同步"
                            }
                        }
                    }
                }
                div { class: "flex-1 p-6 text-center",
                    div { class: "text-6xl mb-4", "☁️" }
                    h3 { class: "text-xl font-semibold text-gray-900 dark:text-white mb-2",
                        "多端同步功能"
                    }
                    p { class: "text-gray-600 dark:text-gray-400",
                        "多端同步功能正在开发中..."
                    }
                }
            }
        },

        PageState::Cleanup => rsx! {
            div { class: "h-full flex flex-col",
                div { class: "flex-shrink-0 px-4 md:px-6 py-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800",
                    div { class: "flex items-center justify-between",
                        div { class: "flex items-center space-x-3",
                            button {
                                class: "flex items-center justify-center w-8 h-8 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors",
                                onclick: move |_| handle_back_to_overview(),
                                title: "返回",
                                "←"
                            }
                            h1 { class: "text-2xl font-bold text-gray-900 dark:text-white",
                                "数据清理"
                            }
                        }
                    }
                }
                div { class: "flex-1 p-6 text-center",
                    div { class: "text-6xl mb-4", "🗑️" }
                    h3 { class: "text-xl font-semibold text-gray-900 dark:text-white mb-2",
                        "数据清理功能"
                    }
                    p { class: "text-gray-600 dark:text-gray-400",
                        "数据清理功能正在开发中..."
                    }
                }
            }
        },
    }
}

// 模拟获取数据统计的异步函数
async fn get_data_statistics() -> Result<DataStatistics, Box<dyn std::error::Error>> {
    // 模拟网络延迟
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // 模拟数据（将来会替换为真实的数据库查询）
    Ok(DataStatistics {
        total_tasks: 156,
        total_time_spent: 87432, // 约24小时
        total_transactions: 89,
        total_notes: 43,
        database_size: "2.3MB".to_string(),
        last_backup: "2天前".to_string(),
    })
}
