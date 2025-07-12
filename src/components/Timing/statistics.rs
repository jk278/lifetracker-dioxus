//! # 统计报告组件
//!
//! 包含时间追踪的统计分析和图表展示功能

use dioxus::prelude::*;
use life_tracker::get_app_state_sync;

/// 统计报告主组件
#[component]
pub fn StatisticsPlaceholder() -> Element {
    let mut selected_period = use_signal(|| "today".to_string());
    let mut show_chart_type = use_signal(|| "overview".to_string());

    // 获取统计数据
    let stats = get_mock_statistics();

    rsx! {
        div {
            class: "space-y-6",

            // 页面标题和控制栏
            div {
                class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-6",
                div {
                    class: "flex items-center justify-between mb-6",
                    h2 { class: "text-2xl font-bold text-gray-900 dark:text-white", "统计报告" }
                    div {
                        class: "flex items-center space-x-3",

                        // 时间范围选择
                        select {
                            class: "px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white",
                            value: "{selected_period.read()}",
                            onchange: move |e| selected_period.set(e.value()),
                            option { value: "today", "今天" }
                            option { value: "week", "本周" }
                            option { value: "month", "本月" }
                            option { value: "year", "今年" }
                        }

                        // 图表类型选择
                        select {
                            class: "px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white",
                            value: "{show_chart_type.read()}",
                            onchange: move |e| show_chart_type.set(e.value()),
                            option { value: "overview", "概览" }
                            option { value: "category", "分类统计" }
                            option { value: "trend", "趋势分析" }
                            option { value: "productivity", "效率分析" }
                        }
                    }
                }

                // 快速统计卡片
                div {
                    class: "grid grid-cols-1 md:grid-cols-4 gap-4",

                    StatCard {
                        title: "总时长",
                        value: "{stats.total_hours}小时",
                        icon: "⏱️",
                        color: "blue",
                        change: "+12%"
                    }

                    StatCard {
                        title: "任务数",
                        value: "{stats.total_tasks}个",
                        icon: "📋",
                        color: "green",
                        change: "+5%"
                    }

                    StatCard {
                        title: "平均效率",
                        value: "{stats.efficiency}%",
                        icon: "📈",
                        color: "purple",
                        change: "+8%"
                    }

                    StatCard {
                        title: "完成率",
                        value: "{stats.completion_rate}%",
                        icon: "✅",
                        color: "emerald",
                        change: "+3%"
                    }
                }
            }

            // 主要内容区域
            match show_chart_type.read().as_str() {
                "overview" => rsx! { OverviewChart { period: selected_period.read().clone() } },
                "category" => rsx! { CategoryChart { period: selected_period.read().clone() } },
                "trend" => rsx! { TrendChart { period: selected_period.read().clone() } },
                "productivity" => rsx! { ProductivityChart { period: selected_period.read().clone() } },
                _ => rsx! { OverviewChart { period: selected_period.read().clone() } }
            }

            // 详细报告
            DetailedReport { period: selected_period.read().clone() }
        }
    }
}

/// 统计卡片组件
#[component]
fn StatCard(
    title: &'static str,
    value: String,
    icon: &'static str,
    color: &'static str,
    change: &'static str,
) -> Element {
    let color_classes = match color {
        "blue" => "from-blue-500 to-blue-600",
        "green" => "from-green-500 to-green-600",
        "purple" => "from-purple-500 to-purple-600",
        "emerald" => "from-emerald-500 to-emerald-600",
        _ => "from-gray-500 to-gray-600",
    };

    rsx! {
        div {
            class: "bg-gradient-to-r {color_classes} rounded-xl p-6 text-white shadow-lg hover:shadow-xl transition-shadow",

            div {
                class: "flex items-center justify-between mb-4",
                div {
                    class: "text-2xl opacity-80",
                    "{icon}"
                }
                div {
                    class: "text-sm bg-white bg-opacity-20 px-2 py-1 rounded-full",
                    "{change}"
                }
            }

            div {
                h3 {
                    class: "text-sm font-medium opacity-80 mb-1",
                    "{title}"
                }
                p {
                    class: "text-2xl font-bold",
                    "{value}"
                }
            }
        }
    }
}

/// 概览图表组件
#[component]
fn OverviewChart(period: String) -> Element {
    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-6",

            h3 {
                class: "text-lg font-semibold text-gray-900 dark:text-white mb-6",
                "时间分布概览 - {period_to_chinese(&period)}"
            }

            div {
                class: "h-64 flex items-center justify-center text-gray-500 dark:text-gray-400",
                div {
                    class: "text-center",
                    div { class: "text-6xl mb-4", "📊" }
                    p { class: "text-lg font-medium mb-2", "图表功能开发中" }
                    p { class: "text-sm", "将显示时间分布的饼图或柱状图" }
                }
            }
        }
    }
}

/// 分类统计图表组件
#[component]
fn CategoryChart(period: String) -> Element {
    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-6",

            h3 {
                class: "text-lg font-semibold text-gray-900 dark:text-white mb-6",
                "分类统计 - {period_to_chinese(&period)}"
            }

            div {
                class: "h-64 flex items-center justify-center text-gray-500 dark:text-gray-400",
                div {
                    class: "text-center",
                    div { class: "text-6xl mb-4", "🏷️" }
                    p { class: "text-lg font-medium mb-2", "分类统计开发中" }
                    p { class: "text-sm", "将显示各分类的时间占比" }
                }
            }
        }
    }
}

/// 趋势分析图表组件
#[component]
fn TrendChart(period: String) -> Element {
    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-6",

            h3 {
                class: "text-lg font-semibold text-gray-900 dark:text-white mb-6",
                "趋势分析 - {period_to_chinese(&period)}"
            }

            div {
                class: "h-64 flex items-center justify-center text-gray-500 dark:text-gray-400",
                div {
                    class: "text-center",
                    div { class: "text-6xl mb-4", "📈" }
                    p { class: "text-lg font-medium mb-2", "趋势分析开发中" }
                    p { class: "text-sm", "将显示时间使用的趋势变化" }
                }
            }
        }
    }
}

/// 效率分析图表组件
#[component]
fn ProductivityChart(period: String) -> Element {
    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-6",

            h3 {
                class: "text-lg font-semibold text-gray-900 dark:text-white mb-6",
                "效率分析 - {period_to_chinese(&period)}"
            }

            div {
                class: "h-64 flex items-center justify-center text-gray-500 dark:text-gray-400",
                div {
                    class: "text-center",
                    div { class: "text-6xl mb-4", "🎯" }
                    p { class: "text-lg font-medium mb-2", "效率分析开发中" }
                    p { class: "text-sm", "将显示工作效率和专注度分析" }
                }
            }
        }
    }
}

/// 详细报告组件
#[component]
fn DetailedReport(period: String) -> Element {
    let report_data = get_mock_report_data();

    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-6",

            h3 {
                class: "text-lg font-semibold text-gray-900 dark:text-white mb-6",
                "详细报告 - {period_to_chinese(&period)}"
            }

            div {
                class: "grid grid-cols-1 lg:grid-cols-2 gap-6",

                // 左侧：热力图
                div {
                    h4 {
                        class: "text-base font-medium text-gray-900 dark:text-white mb-4",
                        "工作时间热力图"
                    }
                    div {
                        class: "h-32 bg-gray-50 dark:bg-gray-700 rounded-lg flex items-center justify-center text-gray-500 dark:text-gray-400",
                        "热力图开发中..."
                    }
                }

                // 右侧：任务排行
                div {
                    h4 {
                        class: "text-base font-medium text-gray-900 dark:text-white mb-4",
                        "任务时间排行"
                    }
                    div {
                        class: "space-y-3",
                        for (index, task) in report_data.top_tasks.iter().enumerate() {
                            div {
                                class: "flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg",
                                div {
                                    class: "flex items-center space-x-3",
                                    span {
                                        class: "w-6 h-6 bg-blue-500 text-white text-xs rounded-full flex items-center justify-center font-bold",
                                        "{index + 1}"
                                    }
                                    span {
                                        class: "text-gray-900 dark:text-white font-medium",
                                        "{task.name}"
                                    }
                                }
                                span {
                                    class: "text-gray-600 dark:text-gray-400 text-sm",
                                    "{task.duration}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 获取模拟统计数据
fn get_mock_statistics() -> StatisticsData {
    StatisticsData {
        total_hours: 8.5,
        total_tasks: 12,
        efficiency: 85,
        completion_rate: 92,
    }
}

/// 获取模拟报告数据
fn get_mock_report_data() -> ReportData {
    ReportData {
        top_tasks: vec![
            TaskSummary {
                name: "项目开发".to_string(),
                duration: "3.5小时".to_string(),
            },
            TaskSummary {
                name: "代码审查".to_string(),
                duration: "2.0小时".to_string(),
            },
            TaskSummary {
                name: "会议讨论".to_string(),
                duration: "1.5小时".to_string(),
            },
            TaskSummary {
                name: "文档编写".to_string(),
                duration: "1.2小时".to_string(),
            },
            TaskSummary {
                name: "学习研究".to_string(),
                duration: "0.8小时".to_string(),
            },
        ],
    }
}

/// 转换时间段为中文
fn period_to_chinese(period: &str) -> &'static str {
    match period {
        "today" => "今天",
        "week" => "本周",
        "month" => "本月",
        "year" => "今年",
        _ => "今天",
    }
}

/// 统计数据结构
#[derive(Clone, Debug)]
struct StatisticsData {
    total_hours: f64,
    total_tasks: u32,
    efficiency: u32,
    completion_rate: u32,
}

/// 报告数据结构
#[derive(Clone, Debug)]
struct ReportData {
    top_tasks: Vec<TaskSummary>,
}

/// 任务摘要结构
#[derive(Clone, Debug)]
struct TaskSummary {
    name: String,
    duration: String,
}
