//! # 财务统计组件
//!
//! 显示财务统计分析和趋势图表

use crate::components::accounting::FinancialTrendChart;
use chrono::Datelike;
use dioxus::prelude::*;
use life_tracker::storage::{FinancialStats, TrendData, TrendGranularity};

#[derive(Props, Clone, PartialEq)]
pub struct StatsTabProps {
    pub financial_stats: Option<FinancialStats>,
    pub format_amount: fn(f64, Option<&str>) -> String,
}

#[component]
pub fn StatsTab(props: StatsTabProps) -> Element {
    // 趋势数据状态
    let trend_data = use_signal(|| Vec::<TrendData>::new());
    let trend_loading = use_signal(|| false);
    let trend_error = use_signal(|| None::<String>);

    // 图表显示控制 (从localStorage恢复)
    let show_income = use_signal(|| {
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            if let Ok(Some(saved)) = storage.get_item("financial-trend-show-income") {
                return saved.parse::<bool>().unwrap_or(true);
            }
        }
        true
    });

    let show_expense = use_signal(|| {
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            if let Ok(Some(saved)) = storage.get_item("financial-trend-show-expense") {
                return saved.parse::<bool>().unwrap_or(true);
            }
        }
        true
    });

    // 趋势类型
    let trend_type = use_signal(|| {
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            if let Ok(Some(saved)) = storage.get_item("financial-trend-granularity") {
                return serde_json::from_str::<TrendGranularity>(&saved)
                    .unwrap_or(TrendGranularity::Month);
            }
        }
        TrendGranularity::Month
    });

    // 获取趋势数据
    let fetch_trend_data = {
        let mut trend_data = trend_data.clone();
        let mut trend_loading = trend_loading.clone();
        let mut trend_error = trend_error.clone();
        let trend_type = trend_type.read().clone();

        move || {
            spawn(async move {
                trend_loading.set(true);
                trend_error.set(None);

                match get_financial_trend_data(trend_type).await {
                    Ok(data) => {
                        trend_data.set(data);
                        log::info!("Financial trend data loaded successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to fetch trend data: {}", e);
                        trend_error.set(Some("获取趋势数据失败".to_string()));
                    }
                }

                trend_loading.set(false);
            });
        }
    };

    // 初始化加载数据
    use_effect(move || {
        fetch_trend_data();
    });

    // 处理显示收入变化
    let mut handle_show_income_change = {
        let mut show_income = show_income.clone();
        move |checked: bool| {
            show_income.set(checked);
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let _ = storage.set_item("financial-trend-show-income", &checked.to_string());
            }
        }
    };

    // 处理显示支出变化
    let mut handle_show_expense_change = {
        let mut show_expense = show_expense.clone();
        move |checked: bool| {
            show_expense.set(checked);
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let _ = storage.set_item("financial-trend-show-expense", &checked.to_string());
            }
        }
    };

    // 处理趋势类型变化
    let mut handle_trend_type_change = {
        let mut trend_type = trend_type.clone();
        move |new_type: TrendGranularity| {
            trend_type.set(new_type);
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let value = serde_json::to_string(&new_type).unwrap_or_default();
                let _ = storage.set_item("financial-trend-granularity", &value);
            }
            fetch_trend_data();
        }
    };

    rsx! {
        div { class: "space-y-6",
            h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100",
                "财务统计"
            }

            if let Some(financial_stats) = &props.financial_stats {
                div { class: "space-y-6",

                    // 统计卡片
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",

                        // 总收入
                        div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6",
                            h4 { class: "text-sm font-medium text-gray-500 dark:text-gray-400",
                                "总收入"
                            }
                            p { class: "text-2xl font-bold text-green-600 dark:text-green-400",
                                "{(props.format_amount)(financial_stats.total_income, None)}"
                            }
                        }

                        // 总支出
                        div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6",
                            h4 { class: "text-sm font-medium text-gray-500 dark:text-gray-400",
                                "总支出"
                            }
                            p { class: "text-2xl font-bold text-red-600 dark:text-red-400",
                                "{(props.format_amount)(financial_stats.total_expense, None)}"
                            }
                        }

                        // 净收入
                        div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6",
                            h4 { class: "text-sm font-medium text-gray-500 dark:text-gray-400",
                                "净收入"
                            }
                            p { class: "text-2xl font-bold text-blue-600 dark:text-blue-400",
                                "{(props.format_amount)(financial_stats.net_income, None)}"
                            }
                        }

                        // 交易笔数
                        div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6",
                            h4 { class: "text-sm font-medium text-gray-500 dark:text-gray-400",
                                "交易笔数"
                            }
                            p { class: "text-2xl font-bold text-gray-900 dark:text-gray-100",
                                "{financial_stats.transaction_count}"
                            }
                        }
                    }

                    // 收支趋势图表
                    div { class: "space-y-4",

                        // 图表控制选项
                        div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-4",
                            div { class: "space-y-4",
                                h4 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100",
                                    "收支趋势 (过去12个月)"
                                }

                                // 大屏：横向布局
                                div { class: "hidden md:flex items-center justify-between",
                                    div { class: "flex items-center space-x-4",
                                        label { class: "flex items-center",
                                            input {
                                                r#type: "radio",
                                                checked: matches!(*trend_type.read(), TrendGranularity::Month),
                                                onchange: move |_| handle_trend_type_change(TrendGranularity::Month),
                                                class: "mr-2 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                            }
                                            span { class: "text-sm text-gray-700 dark:text-gray-300",
                                                "月度"
                                            }
                                        }
                                        label { class: "flex items-center",
                                            input {
                                                r#type: "radio",
                                                checked: matches!(*trend_type.read(), TrendGranularity::Week),
                                                onchange: move |_| handle_trend_type_change(TrendGranularity::Week),
                                                class: "mr-2 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                            }
                                            span { class: "text-sm text-gray-700 dark:text-gray-300",
                                                "周度"
                                            }
                                        }
                                        label { class: "flex items-center",
                                            input {
                                                r#type: "radio",
                                                checked: matches!(*trend_type.read(), TrendGranularity::Day),
                                                onchange: move |_| handle_trend_type_change(TrendGranularity::Day),
                                                class: "mr-2 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                            }
                                            span { class: "text-sm text-gray-700 dark:text-gray-300",
                                                "日度"
                                            }
                                        }
                                    }

                                    // 收入/支出切换
                                    div { class: "flex items-center space-x-4",
                                        label { class: "flex items-center",
                                            input {
                                                r#type: "checkbox",
                                                checked: *show_income.read(),
                                                onchange: move |e| handle_show_income_change(e.value() == "true"),
                                                class: "mr-2 rounded border-gray-300 text-green-600 focus:ring-green-500"
                                            }
                                            span { class: "text-sm text-gray-700 dark:text-gray-300",
                                                "显示收入"
                                            }
                                        }
                                        label { class: "flex items-center",
                                            input {
                                                r#type: "checkbox",
                                                checked: *show_expense.read(),
                                                onchange: move |e| handle_show_expense_change(e.value() == "true"),
                                                class: "mr-2 rounded border-gray-300 text-red-600 focus:ring-red-500"
                                            }
                                            span { class: "text-sm text-gray-700 dark:text-gray-300",
                                                "显示支出"
                                            }
                                        }
                                    }
                                }

                                // 小屏：纵向布局
                                div { class: "md:hidden space-y-4",

                                    // 时间粒度选择
                                    div {
                                        div { class: "text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                                            "时间粒度"
                                        }
                                        div { class: "grid grid-cols-3 gap-2",
                                            label {
                                                class: "flex items-center justify-center p-2 border border-gray-300 dark:border-gray-600 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors",
                                                onclick: move |_| handle_trend_type_change(TrendGranularity::Month),

                                                span {
                                                    class: if matches!(*trend_type.read(), TrendGranularity::Month) {
                                                        "text-sm font-medium text-blue-600 dark:text-blue-400"
                                                    } else {
                                                        "text-sm font-medium text-gray-700 dark:text-gray-300"
                                                    },
                                                    "月度"
                                                }
                                            }
                                            label {
                                                class: "flex items-center justify-center p-2 border border-gray-300 dark:border-gray-600 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors",
                                                onclick: move |_| handle_trend_type_change(TrendGranularity::Week),

                                                span {
                                                    class: if matches!(*trend_type.read(), TrendGranularity::Week) {
                                                        "text-sm font-medium text-blue-600 dark:text-blue-400"
                                                    } else {
                                                        "text-sm font-medium text-gray-700 dark:text-gray-300"
                                                    },
                                                    "周度"
                                                }
                                            }
                                            label {
                                                class: "flex items-center justify-center p-2 border border-gray-300 dark:border-gray-600 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors",
                                                onclick: move |_| handle_trend_type_change(TrendGranularity::Day),

                                                span {
                                                    class: if matches!(*trend_type.read(), TrendGranularity::Day) {
                                                        "text-sm font-medium text-blue-600 dark:text-blue-400"
                                                    } else {
                                                        "text-sm font-medium text-gray-700 dark:text-gray-300"
                                                    },
                                                    "日度"
                                                }
                                            }
                                        }
                                    }

                                    // 收入/支出切换
                                    div {
                                        div { class: "text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                                            "显示内容"
                                        }
                                        div { class: "grid grid-cols-2 gap-2",
                                            label {
                                                class: "flex items-center justify-center p-2 border border-gray-300 dark:border-gray-600 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors",
                                                onclick: move |_| handle_show_income_change(!*show_income.read()),

                                                span {
                                                    class: if *show_income.read() {
                                                        "text-sm font-medium text-green-600 dark:text-green-400"
                                                    } else {
                                                        "text-sm font-medium text-gray-700 dark:text-gray-300"
                                                    },
                                                    "💰 收入"
                                                }
                                            }
                                            label {
                                                class: "flex items-center justify-center p-2 border border-gray-300 dark:border-gray-600 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors",
                                                onclick: move |_| handle_show_expense_change(!*show_expense.read()),

                                                span {
                                                    class: if *show_expense.read() {
                                                        "text-sm font-medium text-red-600 dark:text-red-400"
                                                    } else {
                                                        "text-sm font-medium text-gray-700 dark:text-gray-300"
                                                    },
                                                    "💸 支出"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 趋势图表
                        div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 transition-all duration-200 ease-in-out",
                            div { class: "p-4 md:p-6",
                                h4 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                                    "收支趋势"
                                }

                                // 固定高度的内容区域
                                div { class: "h-48 md:h-80 relative",
                                    if *trend_loading.read() {
                                        div { class: "absolute inset-0 flex items-center justify-center bg-gray-50 dark:bg-gray-800 rounded-lg",
                                            div { class: "text-center",
                                                div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-2" }
                                                p { class: "text-gray-500 dark:text-gray-400",
                                                    "加载中..."
                                                }
                                            }
                                        }
                                    } else if let Some(error) = trend_error.read().as_ref() {
                                        div { class: "absolute inset-0 flex items-center justify-center bg-gray-50 dark:bg-gray-800 rounded-lg",
                                            div { class: "text-center",
                                                div { class: "text-red-500 text-lg mb-2", "⚠️" }
                                                p { class: "text-red-600 dark:text-red-400 mb-3",
                                                    "{error}"
                                                }
                                                button {
                                                    class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors",
                                                    onclick: move |_| fetch_trend_data(),
                                                    "重试"
                                                }
                                            }
                                        }
                                    } else {
                                        div { class: "h-full transition-opacity duration-200 ease-in-out",
                                            FinancialTrendChart {
                                                data: trend_data.read().clone(),
                                                show_income: *show_income.read(),
                                                show_expense: *show_expense.read(),
                                                granularity: *trend_type.read(),
                                                format_amount: props.format_amount,
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 统计期间
                    div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6",
                        h4 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                            "统计期间"
                        }
                        p { class: "text-gray-600 dark:text-gray-400",
                            "{financial_stats.period_start} 至 {financial_stats.period_end}"
                        }
                    }
                }
            }
        }
    }
}

// 获取财务趋势数据的异步函数
async fn get_financial_trend_data(
    granularity: TrendGranularity,
) -> Result<Vec<TrendData>, Box<dyn std::error::Error>> {
    // 计算时间范围
    let end_date = chrono::Utc::now().naive_utc().date();
    let start_date = match granularity {
        TrendGranularity::Day => end_date - chrono::Duration::days(29), // 30天
        TrendGranularity::Week => end_date - chrono::Duration::weeks(11), // 12周
        TrendGranularity::Month => end_date - chrono::Duration::days(365), // 12个月
    };

    // TODO: 实际的数据库查询逻辑
    // 这里先返回模拟数据
    let mut trend_data = Vec::new();

    let mut current_date = start_date;
    while current_date <= end_date {
        let label = match granularity {
            TrendGranularity::Day => current_date.format("%m-%d").to_string(),
            TrendGranularity::Week => format!("W{}", current_date.format("%W")),
            TrendGranularity::Month => current_date.format("%Y-%m").to_string(),
        };

        trend_data.push(TrendData {
            label,
            income: (rand::random::<f64>() * 5000.0).round(),
            expense: (rand::random::<f64>() * 3000.0).round(),
        });

        current_date = match granularity {
            TrendGranularity::Day => current_date + chrono::Duration::days(1),
            TrendGranularity::Week => current_date + chrono::Duration::weeks(1),
            TrendGranularity::Month => {
                if current_date.month() == 12 {
                    current_date
                        .with_year(current_date.year() + 1)
                        .unwrap()
                        .with_month(1)
                        .unwrap()
                } else {
                    current_date.with_month(current_date.month() + 1).unwrap()
                }
            }
        };
    }

    Ok(trend_data)
}
