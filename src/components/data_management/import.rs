//! # 数据导入组件
//!
//! 提供数据导入功能，支持多种数据格式

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct DataImportProps {
    pub show_back_button: bool,
    pub on_back: Option<EventHandler<()>>,
}

// 导入结果状态
#[derive(Debug, Clone, PartialEq)]
enum ImportResult {
    None,
    Success(String),
    Error(String),
}

// 支持的文件格式
#[derive(Debug, Clone, PartialEq)]
struct FileFormat {
    name: String,
    extension: String,
    description: String,
}

#[component]
pub fn DataImport(props: DataImportProps) -> Element {
    // 状态管理
    let is_importing = use_signal(|| false);
    let import_result = use_signal(|| ImportResult::None);
    let show_confirm_dialog = use_signal(|| false);
    let selected_file_path = use_signal(|| String::new());

    // 支持的文件格式
    let supported_formats = use_memo(|| {
        vec![
            FileFormat {
                name: "JSON".to_string(),
                extension: "json".to_string(),
                description: "结构化数据格式".to_string(),
            },
            FileFormat {
                name: "CSV".to_string(),
                extension: "csv".to_string(),
                description: "逗号分隔表格".to_string(),
            },
            FileFormat {
                name: "XML".to_string(),
                extension: "xml".to_string(),
                description: "标记语言格式".to_string(),
            },
        ]
    });

    // 处理文件选择
    let handle_file_selection = {
        let mut show_confirm_dialog = show_confirm_dialog.clone();
        let mut selected_file_path = selected_file_path.clone();

        move || {
            spawn(async move {
                // 模拟文件选择对话框
                // 在实际应用中，这里会调用系统文件选择器
                let file_path = simulate_file_selection().await;

                if let Some(path) = file_path {
                    selected_file_path.set(path);
                    show_confirm_dialog.set(true);
                }
            });
        }
    };

    // 确认导入
    let confirm_import = {
        let mut is_importing = is_importing.clone();
        let mut import_result = import_result.clone();
        let mut show_confirm_dialog = show_confirm_dialog.clone();
        let selected_file_path = selected_file_path.read().clone();

        move || {
            spawn(async move {
                is_importing.set(true);
                import_result.set(ImportResult::None);
                show_confirm_dialog.set(false);

                // 模拟导入过程
                tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

                match perform_import(&selected_file_path).await {
                    Ok(message) => {
                        log::info!("Import completed successfully: {}", message);
                        import_result.set(ImportResult::Success(message));
                    }
                    Err(e) => {
                        import_result.set(ImportResult::Error(format!("导入失败: {}", e)));
                        log::error!("Import failed: {}", e);
                    }
                }

                is_importing.set(false);
            });
        }
    };

    // 取消导入
    let mut cancel_import = {
        let mut show_confirm_dialog = show_confirm_dialog.clone();

        move || {
            show_confirm_dialog.set(false);
        }
    };

    rsx! {
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
                            "数据导入"
                        }
                    }
                }
            }

            // 可滚动内容区域
            div { class: "flex-1 overflow-y-auto py-4 px-4 md:px-6",
                div { class: "max-w-2xl mx-auto space-y-6",

                    // 导入注意事项
                    div { class: "bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-6",
                        div { class: "flex items-start",
                            span { class: "text-yellow-600 dark:text-yellow-400 mr-3 mt-0.5 flex-shrink-0", "ℹ️" }
                            div { class: "text-sm text-yellow-700 dark:text-yellow-300",
                                p { class: "font-medium mb-2", "导入注意事项：" }
                                ul { class: "list-disc list-inside space-y-1",
                                    li { "导入操作将覆盖现有数据" }
                                    li { "支持 JSON、CSV、XML 格式" }
                                    li { "建议在导入前先导出备份" }
                                    li { "大文件导入可能需要较长时间" }
                                }
                            }
                        }
                    }

                    // 导入操作
                    div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6",
                        div { class: "text-center space-y-4",
                            div { class: "flex justify-center",
                                div { class: "w-16 h-16 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center",
                                    span { class: "text-3xl text-blue-600 dark:text-blue-400", "📥" }
                                }
                            }

                            div {
                                h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2",
                                    "选择数据文件"
                                }
                                p { class: "text-sm text-gray-600 dark:text-gray-400 mb-4",
                                    "支持 JSON、CSV、XML 格式的数据文件"
                                }
                            }

                            button {
                                class: if is_importing() {
                                    "w-full px-6 py-3 rounded-lg font-medium text-white transition-colors bg-gray-400 cursor-not-allowed"
                                } else {
                                    "w-full px-6 py-3 rounded-lg font-medium text-white transition-colors bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                                },
                                disabled: is_importing(),
                                onclick: move |_| handle_file_selection(),

                                if is_importing() {
                                    span { class: "flex items-center justify-center",
                                        div { class: "animate-spin rounded-full h-5 w-5 border-b-2 border-white mr-3" }
                                        "导入中..."
                                    }
                                } else {
                                    span { class: "flex items-center justify-center",
                                        span { class: "mr-2", "📁" }
                                        "选择文件导入"
                                    }
                                }
                            }
                        }
                    }

                    // 导入结果
                    match import_result.read().clone() {
                        ImportResult::Success(message) => rsx! {
                            div { class: "bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4",
                                div { class: "flex items-start",
                                    span { class: "text-green-600 dark:text-green-400 mr-2 mt-0.5 flex-shrink-0", "✅" }
                                    p { class: "text-sm text-green-700 dark:text-green-300", "{message}" }
                                }
                            }
                        },
                        ImportResult::Error(message) => rsx! {
                            div { class: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4",
                                div { class: "flex items-start",
                                    span { class: "text-red-600 dark:text-red-400 mr-2 mt-0.5 flex-shrink-0", "❌" }
                                    p { class: "text-sm text-red-700 dark:text-red-300", "{message}" }
                                }
                            }
                        },
                        ImportResult::None => rsx! { div {} }
                    }

                    // 支持的文件格式说明
                    div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6",
                        h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                            "支持的文件格式"
                        }
                        div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                            for format in supported_formats.read().iter() {
                                div { class: "text-center p-4 bg-gray-50 dark:bg-gray-800 rounded-lg",
                                    div { class: "text-sm font-medium text-gray-900 dark:text-gray-100 mb-1",
                                        "{format.name}"
                                    }
                                    div { class: "text-xs text-gray-600 dark:text-gray-400",
                                        "{format.description}"
                                    }
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
                                h4 { class: "font-semibold mb-2", "导入说明" }
                                ul { class: "space-y-1",
                                    li { "• 请选择从本应用导出的数据文件" }
                                    li { "• 导入前建议先备份现有数据" }
                                    li { "• 导入过程中请勿关闭应用" }
                                    li { "• 大文件导入可能需要几分钟时间" }
                                    li { "• 导入完成后应用会自动刷新数据" }
                                }
                            }
                        }
                    }
                }
            }

            // 确认对话框
            if show_confirm_dialog() {
                div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                    div { class: "bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md p-6",
                        h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                            "确认导入"
                        }
                        p { class: "text-gray-600 dark:text-gray-400 mb-6",
                            "导入数据将覆盖现有数据，确定要继续吗？"
                        }
                        div { class: "flex justify-end space-x-3",
                            button {
                                class: "px-4 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200",
                                onclick: move |_| cancel_import(),
                                "取消"
                            }
                            button {
                                class: "px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700",
                                onclick: move |_| confirm_import(),
                                "确认导入"
                            }
                        }
                    }
                }
            }
        }
    }
}

// 模拟文件选择
async fn simulate_file_selection() -> Option<String> {
    // 模拟用户选择文件
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 模拟选择了一个文件
    Some("lifetracker-backup-20241215.json".to_string())
}

// 模拟导入函数
async fn perform_import(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 模拟导入过程
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    // 模拟不同的导入结果
    if file_path.contains("invalid") {
        return Err("文件格式不支持".into());
    }

    if file_path.contains("corrupted") {
        return Err("文件已损坏或格式错误".into());
    }

    // 模拟成功导入
    Ok(format!(
        "数据导入成功！\n文件: {}\n导入了 156 个任务、89 个交易记录和 43 个笔记",
        file_path
    ))
}
