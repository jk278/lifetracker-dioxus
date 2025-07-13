//! # 数据导出组件
//!
//! 提供数据导出功能，支持多种格式和选项

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct DataExportProps {
    pub show_back_button: bool,
    pub on_back: Option<EventHandler<()>>,
}

// 导出选项结构
#[derive(Debug, Clone, PartialEq)]
struct ExportOptions {
    include_categories: bool,
    include_statistics: bool,
    include_metadata: bool,
    group_by_date: bool,
    group_by_category: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_categories: true,
            include_statistics: true,
            include_metadata: true,
            group_by_date: false,
            group_by_category: false,
        }
    }
}

// 日期范围结构
#[derive(Debug, Clone, PartialEq)]
struct DateRange {
    start: String,
    end: String,
}

impl Default for DateRange {
    fn default() -> Self {
        Self {
            start: String::new(),
            end: String::new(),
        }
    }
}

// 导出结果状态
#[derive(Debug, Clone, PartialEq)]
enum ExportResult {
    None,
    Success(String),
    Error(String),
}

// 导出格式选项
#[derive(Debug, Clone, PartialEq)]
struct ExportFormat {
    value: String,
    label: String,
    description: String,
}

#[component]
pub fn DataExport(props: DataExportProps) -> Element {
    // 状态管理
    let mut is_exporting = use_signal(|| false);
    let mut export_format = use_signal(|| "json".to_string());
    let mut export_options = use_signal(|| ExportOptions::default());
    let mut date_range = use_signal(|| DateRange::default());
    let mut export_result = use_signal(|| ExportResult::None);

    // 导出格式选项
    let export_formats = use_memo(|| {
        vec![
            ExportFormat {
                value: "json".to_string(),
                label: "JSON".to_string(),
                description: "结构化数据".to_string(),
            },
            ExportFormat {
                value: "csv".to_string(),
                label: "CSV".to_string(),
                description: "表格数据".to_string(),
            },
            ExportFormat {
                value: "xml".to_string(),
                label: "XML".to_string(),
                description: "标记语言".to_string(),
            },
            ExportFormat {
                value: "html".to_string(),
                label: "HTML".to_string(),
                description: "网页格式".to_string(),
            },
            ExportFormat {
                value: "markdown".to_string(),
                label: "Markdown".to_string(),
                description: "文档格式".to_string(),
            },
        ]
    });



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
                            "数据导出"
                        }
                    }
                }
            }

            // 可滚动内容区域
            div { class: "flex-1 overflow-y-auto py-4 px-4 md:px-6",
                div { class: "max-w-2xl mx-auto space-y-6",

                    // 导出格式选择
                    div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6",
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3",
                            span { class: "mr-1", "📄" }
                            "导出格式"
                        }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                            value: export_format.read().clone(),
                            onchange: move |e| export_format.set(e.value()),
                            for format in export_formats.read().iter() {
                                option { value: "{format.value}", "{format.label} - {format.description}" }
                            }
                        }
                    }

                    // 日期范围选择
                    div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6",
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3",
                            span { class: "mr-1", "📅" }
                            "日期范围（可选）"
                        }
                        div { class: "grid grid-cols-2 gap-4",
                            div {
                                label { class: "block text-xs text-gray-500 dark:text-gray-400 mb-1",
                                    "开始日期"
                                }
                                input {
                                    r#type: "date",
                                    class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    value: date_range.read().start.clone(),
                                    onchange: move |e| {
                                        let mut range = date_range.read().clone();
                                        range.start = e.value();
                                        date_range.set(range);
                                    }
                                }
                            }
                            div {
                                label { class: "block text-xs text-gray-500 dark:text-gray-400 mb-1",
                                    "结束日期"
                                }
                                input {
                                    r#type: "date",
                                    class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    value: date_range.read().end.clone(),
                                    onchange: move |e| {
                                        let mut range = date_range.read().clone();
                                        range.end = e.value();
                                        date_range.set(range);
                                    }
                                }
                            }
                        }
                    }

                    // 导出选项
                    div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6",
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3",
                            span { class: "mr-1", "⚙️" }
                            "导出选项"
                        }
                        div { class: "space-y-3",
                            // 包含分类信息
                            label { class: "flex items-center",
                                input {
                                    r#type: "checkbox",
                                    class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                                    checked: export_options.read().include_categories,
                                    onchange: move |e| {
                                        let mut options = export_options.read().clone();
                                        options.include_categories = e.value() == "true";
                                        export_options.set(options);
                                    }
                                }
                                span { class: "ml-2 text-sm text-gray-700 dark:text-gray-300",
                                    "包含分类信息"
                                }
                            }

                            // 包含统计数据
                            label { class: "flex items-center",
                                input {
                                    r#type: "checkbox",
                                    class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                                    checked: export_options.read().include_statistics,
                                    onchange: move |e| {
                                        let mut options = export_options.read().clone();
                                        options.include_statistics = e.value() == "true";
                                        export_options.set(options);
                                    }
                                }
                                span { class: "ml-2 text-sm text-gray-700 dark:text-gray-300",
                                    "包含统计数据"
                                }
                            }

                            // 包含元数据
                            label { class: "flex items-center",
                                input {
                                    r#type: "checkbox",
                                    class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                                    checked: export_options.read().include_metadata,
                                    onchange: move |e| {
                                        let mut options = export_options.read().clone();
                                        options.include_metadata = e.value() == "true";
                                        export_options.set(options);
                                    }
                                }
                                span { class: "ml-2 text-sm text-gray-700 dark:text-gray-300",
                                    "包含元数据"
                                }
                            }

                            // 按日期分组
                            label { class: "flex items-center",
                                input {
                                    r#type: "checkbox",
                                    class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                                    checked: export_options.read().group_by_date,
                                    onchange: move |e| {
                                        let mut options = export_options.read().clone();
                                        options.group_by_date = e.value() == "true";
                                        export_options.set(options);
                                    }
                                }
                                span { class: "ml-2 text-sm text-gray-700 dark:text-gray-300",
                                    "按日期分组"
                                }
                            }

                            // 按分类分组
                            label { class: "flex items-center",
                                input {
                                    r#type: "checkbox",
                                    class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                                    checked: export_options.read().group_by_category,
                                    onchange: move |e| {
                                        let mut options = export_options.read().clone();
                                        options.group_by_category = e.value() == "true";
                                        export_options.set(options);
                                    }
                                }
                                span { class: "ml-2 text-sm text-gray-700 dark:text-gray-300",
                                    "按分类分组"
                                }
                            }
                        }
                    }

                    // 导出按钮
                    div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6",
                        button {
                            class: if is_exporting() {
                                "w-full px-4 py-2 rounded-md font-medium text-white transition-colors bg-gray-400 cursor-not-allowed"
                            } else {
                                "w-full px-4 py-2 rounded-md font-medium text-white transition-colors bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2"
                            },
                            disabled: is_exporting(),
                            onclick: move |_| {
                                let mut is_exporting = is_exporting.clone();
                                let mut export_result = export_result.clone();
                                let export_format = export_format.read().clone();
                                let export_options = export_options.read().clone();
                                let date_range = date_range.read().clone();
                                
                                spawn(async move {
                                    is_exporting.set(true);
                                    export_result.set(ExportResult::None);

                                    // 模拟导出过程
                                    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

                                    match perform_export(&export_format, &export_options, &date_range).await {
                                        Ok(message) => {
                                            log::info!("Export completed successfully: {}", message);
                                            export_result.set(ExportResult::Success(message));
                                        }
                                        Err(e) => {
                                            export_result.set(ExportResult::Error(format!("导出失败: {}", e)));
                                            log::error!("Export failed: {}", e);
                                        }
                                    }

                                    is_exporting.set(false);
                                });
                            },

                            if is_exporting() {
                                span { class: "flex items-center justify-center",
                                    div { class: "animate-spin rounded-full h-5 w-5 border-b-2 border-white mr-3" }
                                    "导出中..."
                                }
                            } else {
                                span { class: "flex items-center justify-center",
                                    span { class: "mr-2", "📤" }
                                    "开始导出"
                                }
                            }
                        }
                    }

                    // 导出结果
                    match export_result.read().clone() {
                        ExportResult::Success(message) => rsx! {
                            div { class: "bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4",
                                div { class: "flex items-start",
                                    span { class: "text-green-600 dark:text-green-400 mr-2 mt-0.5 flex-shrink-0", "✅" }
                                    p { class: "text-sm text-green-700 dark:text-green-300", "{message}" }
                                }
                            }
                        },
                        ExportResult::Error(message) => rsx! {
                            div { class: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4",
                                div { class: "flex items-start",
                                    span { class: "text-red-600 dark:text-red-400 mr-2 mt-0.5 flex-shrink-0", "❌" }
                                    p { class: "text-sm text-red-700 dark:text-red-300", "{message}" }
                                }
                            }
                        },
                        ExportResult::None => rsx! { div {} }
                    }

                    // 说明文本
                    div { class: "bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-700 rounded-lg p-4",
                        div { class: "flex items-start space-x-3",
                            div { class: "flex-shrink-0 mt-1",
                                span { class: "text-blue-600 dark:text-blue-400", "💡" }
                            }
                            div { class: "text-sm text-blue-800 dark:text-blue-200",
                                h4 { class: "font-semibold mb-2", "导出说明" }
                                ul { class: "space-y-1",
                                    li { "• 选择合适的导出格式以满足您的需求" }
                                    li { "• 日期范围可用于筛选特定时间段的数据" }
                                    li { "• 导出选项可以自定义包含的数据类型" }
                                    li { "• 支持按日期或分类对数据进行分组" }
                                    li { "• 导出的文件将保存到您选择的位置" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// 模拟导出函数
async fn perform_export(
    format: &str,
    options: &ExportOptions,
    date_range: &DateRange,
) -> Result<String, Box<dyn std::error::Error>> {
    // 模拟导出过程
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // 模拟数据处理
    let mut export_info = Vec::new();
    export_info.push(format!("导出格式: {}", format.to_uppercase()));

    if options.include_categories {
        export_info.push("包含分类信息".to_string());
    }
    if options.include_statistics {
        export_info.push("包含统计数据".to_string());
    }
    if options.include_metadata {
        export_info.push("包含元数据".to_string());
    }

    if !date_range.start.is_empty() && !date_range.end.is_empty() {
        export_info.push(format!(
            "日期范围: {} 至 {}",
            date_range.start, date_range.end
        ));
    }

    let filename = format!(
        "lifetracker-export-{}.{}",
        chrono::Utc::now().format("%Y%m%d_%H%M%S"),
        format
    );

    Ok(format!(
        "数据导出成功！\n文件名: {}\n导出内容: {}",
        filename,
        export_info.join(", ")
    ))
}
