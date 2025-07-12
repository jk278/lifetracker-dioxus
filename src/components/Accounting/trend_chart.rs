//! # 财务趋势图表组件
//!
//! 显示收支趋势图表

use dioxus::prelude::*;
use life_tracker::storage::{TrendData, TrendGranularity};

#[derive(Props, Clone, PartialEq)]
pub struct FinancialTrendChartProps {
    pub data: Vec<TrendData>,
    pub show_income: bool,
    pub show_expense: bool,
    pub granularity: TrendGranularity,
    pub format_amount: fn(f64, Option<&str>) -> String,
}

#[component]
pub fn FinancialTrendChart(props: FinancialTrendChartProps) -> Element {
    // 响应式检测
    let mut is_mobile = use_signal(|| false);

    // 检查屏幕尺寸
    use_effect(move || {
        // TODO: 实现移动端检测
        if false {
            // 暂时禁用移动端检测
            let update_mobile = move |_| {
                // TODO: 实现移动端检测
                is_mobile.set(false);
            };

            // 初始检查
            update_mobile(());

            // TODO: 实现resize事件监听器
            // let closure = wasm_bindgen::closure::Closure::wrap(
            //     Box::new(update_mobile) as Box<dyn Fn()>,
            // );
            // let _ = window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref());
            // closure.forget();
        }
    });

    // 如果没有数据，显示空状态
    if props.data.is_empty() {
        return rsx! {
            div { class: "flex items-center justify-center h-full bg-gray-50 dark:bg-gray-800 rounded-lg",
                div { class: "text-center",
                    div { class: "text-gray-400 dark:text-gray-500 text-lg mb-2",
                        "📊"
                    }
                    p { class: "text-gray-500 dark:text-gray-400",
                        "暂无趋势数据"
                    }
                }
            }
        };
    }

    // 计算最大值用于缩放
    let max_value = props
        .data
        .iter()
        .map(|d| (d.income as f64).max(d.expense as f64))
        .fold(0.0f64, |a, b| a.max(b));

    // 格式化标签显示
    let format_label = |label: &str| -> String {
        match props.granularity {
            TrendGranularity::Day => {
                // MM-DD格式
                label.split('-').skip(1).collect::<Vec<_>>().join("-")
            }
            TrendGranularity::Week => {
                // W27格式
                label.replace("W", "W")
            }
            TrendGranularity::Month => {
                // YYYY-MM -> MM月
                if let Some(month) = label.split('-').nth(1) {
                    format!("{}月", month)
                } else {
                    label.to_string()
                }
            }
        }
    };

    rsx! {
        div { class: "h-full w-full",
            div { class: "relative h-full p-4",
                // 图表区域
                div { class: "flex items-end justify-between h-full space-x-1",
                    for (index, data) in props.data.iter().enumerate() {
                        div {
                            key: "{index}",
                            class: "flex-1 flex flex-col items-center space-y-1",

                            // 工具提示容器
                            div {
                                class: "relative group flex flex-col items-center flex-1 w-full",

                                // 柱状图容器
                                div { class: "flex items-end justify-center w-full h-full space-x-1",

                                    // 收入柱
                                    if props.show_income {
                                        div {
                                            class: "bg-green-500 dark:bg-green-400 rounded-t-sm transition-all duration-300 hover:bg-green-600 dark:hover:bg-green-300",
                                            style: format!(
                                                "height: {}%; width: {}px; min-height: 4px;",
                                                if max_value > 0.0 { (data.income / max_value * 100.0) as i32 } else { 0 },
                                                if *is_mobile.read() { 12 } else { 20 }
                                            ),
                                            title: format!("收入: {}", (props.format_amount)(data.income, None))
                                        }
                                    }

                                    // 支出柱
                                    if props.show_expense {
                                        div {
                                            class: "bg-red-500 dark:bg-red-400 rounded-t-sm transition-all duration-300 hover:bg-red-600 dark:hover:bg-red-300",
                                            style: format!(
                                                "height: {}%; width: {}px; min-height: 4px;",
                                                if max_value > 0.0 { (data.expense / max_value * 100.0) as i32 } else { 0 },
                                                if *is_mobile.read() { 12 } else { 20 }
                                            ),
                                            title: format!("支出: {}", (props.format_amount)(data.expense, None))
                                        }
                                    }
                                }

                                // 悬浮工具提示
                                div {
                                    class: "absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none",
                                    div { class: "bg-gray-800 text-white text-xs rounded px-2 py-1 shadow-lg whitespace-nowrap",
                                        div { class: "font-medium mb-1", "{format_label(&data.label)}" }
                                        if props.show_income {
                                            div { class: "text-green-400", "收入: {(props.format_amount)(data.income, None)}" }
                                        }
                                        if props.show_expense {
                                            div { class: "text-red-400", "支出: {(props.format_amount)(data.expense, None)}" }
                                        }
                                    }
                                }
                            }

                            // X轴标签
                            div {
                                class: format!(
                                    "text-gray-600 dark:text-gray-400 text-center mt-2 {}",
                                    if *is_mobile.read() { "text-xs" } else { "text-sm" }
                                ),
                                style: "writing-mode: horizontal-tb;",
                                "{format_label(&data.label)}"
                            }
                        }
                    }
                }

                // Y轴标签 (显示在左侧)
                div { class: "absolute left-0 top-0 h-full flex flex-col justify-between text-xs text-gray-500 dark:text-gray-400 pr-2",
                    div { "{(props.format_amount)(max_value, None)}" }
                    div { "{(props.format_amount)(max_value * 0.75, None)}" }
                    div { "{(props.format_amount)(max_value * 0.5, None)}" }
                    div { "{(props.format_amount)(max_value * 0.25, None)}" }
                    div { "0" }
                }
            }

            // 图例
            div { class: "flex justify-center mt-4 space-x-4",
                if props.show_income {
                    div { class: "flex items-center space-x-2",
                        div { class: "w-3 h-3 bg-green-500 dark:bg-green-400 rounded-sm" }
                        span { class: "text-sm text-gray-700 dark:text-gray-300", "收入" }
                    }
                }
                if props.show_expense {
                    div { class: "flex items-center space-x-2",
                        div { class: "w-3 h-3 bg-red-500 dark:bg-red-400 rounded-sm" }
                        span { class: "text-sm text-gray-700 dark:text-gray-300", "支出" }
                    }
                }
            }
        }
    }
}
