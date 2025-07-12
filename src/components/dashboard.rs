//! # 仪表板组件
//!
//! 应用的主页面，显示各功能模块的概览和快速导航

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use super::app::Route;

/// 主仪表板组件
#[component]
pub fn Dashboard() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-gray-50 dark:bg-gray-900",

            // 顶部欢迎区域
            div {
                class: "bg-gradient-to-r from-blue-600 to-purple-600 text-white py-12",
                div {
                    class: "container mx-auto px-4",
                    h1 {
                        class: "text-4xl font-bold mb-2",
                        "欢迎回来！"
                    }
                    p {
                        class: "text-blue-100 text-lg",
                        "开始高效的一天！"
                    }
                }
            }

            div {
                class: "container mx-auto px-4 py-8",

                // 今日概览卡片
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8",

                    // 今日时间追踪
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 border-l-4 border-blue-500",
                        div {
                            class: "flex items-center justify-between",
                            div {
                                h3 {
                                    class: "text-lg font-semibold text-gray-800 dark:text-white",
                                    "今日追踪"
                                }
                                p {
                                    class: "text-3xl font-bold text-blue-600 dark:text-blue-400 mt-2",
                                    "2小时 30分"
                                }
                                p {
                                    class: "text-sm text-gray-600 dark:text-gray-300",
                                    "3个任务"
                                }
                            }
                            div {
                                class: "text-4xl",
                                "⏱️"
                            }
                        }
                    }

                    // 本月收支
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 border-l-4 border-green-500",
                        div {
                            class: "flex items-center justify-between",
                            div {
                                h3 {
                                    class: "text-lg font-semibold text-gray-800 dark:text-white",
                                    "本月收支"
                                }
                                p {
                                    class: "text-3xl font-bold text-green-600 dark:text-green-400 mt-2",
                                    "+¥1,280"
                                }
                                p {
                                    class: "text-sm text-gray-600 dark:text-gray-300",
                                    "收入 ¥8,500 | 支出 ¥7,220"
                                }
                            }
                            div {
                                class: "text-4xl",
                                "💰"
                            }
                        }
                    }

                    // 笔记条数
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 border-l-4 border-purple-500",
                        div {
                            class: "flex items-center justify-between",
                            div {
                                h3 {
                                    class: "text-lg font-semibold text-gray-800 dark:text-white",
                                    "笔记记录"
                                }
                                p {
                                    class: "text-3xl font-bold text-purple-600 dark:text-purple-400 mt-2",
                                    "24 篇"
                                }
                                p {
                                    class: "text-sm text-gray-600 dark:text-gray-300",
                                    "本月新增 8 篇"
                                }
                            }
                            div {
                                class: "text-4xl",
                                "📝"
                            }
                        }
                    }

                    // 习惯打卡
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 border-l-4 border-orange-500",
                        div {
                            class: "flex items-center justify-between",
                            div {
                                h3 {
                                    class: "text-lg font-semibold text-gray-800 dark:text-white",
                                    "习惯打卡"
                                }
                                p {
                                    class: "text-3xl font-bold text-orange-600 dark:text-orange-400 mt-2",
                                    "85%"
                                }
                                p {
                                    class: "text-sm text-gray-600 dark:text-gray-300",
                                    "本周完成率"
                                }
                            }
                            div {
                                class: "text-4xl",
                                "✅"
                            }
                        }
                    }
                }

                // 主要功能卡片
                h2 {
                    class: "text-2xl font-bold text-gray-800 dark:text-white mb-6",
                    "功能导航"
                }
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8",

                    // 时间追踪卡片
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 hover:shadow-xl transition-shadow",
                        h3 {
                            class: "text-xl font-semibold text-gray-800 dark:text-white mb-4 flex items-center",
                            span { class: "text-2xl mr-3", "🕒" }
                            "时间追踪"
                        }
                        p {
                            class: "text-gray-600 dark:text-gray-300 mb-4",
                            "记录你的工作时间，提高效率。支持任务分类、计时器和统计分析。"
                        }
                        div {
                            class: "flex justify-between items-center",
                            div {
                                class: "text-sm text-gray-500 dark:text-gray-400",
                                "今日: 2小时30分"
                            }
                            Link {
                                to: Route::TaskManagement {},
                                class: "bg-blue-500 text-white px-4 py-2 rounded-lg hover:bg-blue-600 transition-colors",
                                "开始追踪"
                            }
                        }
                    }

                    // 财务管理卡片
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 hover:shadow-xl transition-shadow",
                        h3 {
                            class: "text-xl font-semibold text-gray-800 dark:text-white mb-4 flex items-center",
                            span { class: "text-2xl mr-3", "💰" }
                            "财务管理"
                        }
                        p {
                            class: "text-gray-600 dark:text-gray-300 mb-4",
                            "管理收入支出，制定预算计划。支持多账户、分类统计和财务报表。"
                        }
                        div {
                            class: "flex justify-between items-center",
                            div {
                                class: "text-sm text-gray-500 dark:text-gray-400",
                                "本月结余: +¥1,280"
                            }
                            Link {
                                to: Route::Financial {},
                                class: "bg-green-500 text-white px-4 py-2 rounded-lg hover:bg-green-600 transition-colors",
                                "查看财务"
                            }
                        }
                    }

                    // 日记功能卡片
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 hover:shadow-xl transition-shadow",
                        h3 {
                            class: "text-xl font-semibold text-gray-800 dark:text-white mb-4 flex items-center",
                            span { class: "text-2xl mr-3", "📝" }
                            "日记笔记"
                        }
                        p {
                            class: "text-gray-600 dark:text-gray-300 mb-4",
                            "记录生活点滴，整理思考感悟。支持富文本编辑、标签分类和搜索。"
                        }
                        div {
                            class: "flex justify-between items-center",
                            div {
                                class: "text-sm text-gray-500 dark:text-gray-400",
                                "本月: 8 篇新笔记"
                            }
                            Link {
                                to: Route::Diary {},
                                class: "bg-purple-500 text-white px-4 py-2 rounded-lg hover:bg-purple-600 transition-colors",
                                "写日记"
                            }
                        }
                    }

                    // 习惯打卡卡片
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 hover:shadow-xl transition-shadow",
                        h3 {
                            class: "text-xl font-semibold text-gray-800 dark:text-white mb-4 flex items-center",
                            span { class: "text-2xl mr-3", "✅" }
                            "习惯打卡"
                        }
                        p {
                            class: "text-gray-600 dark:text-gray-300 mb-4",
                            "培养良好习惯，追踪进度目标。支持提醒设置、连续统计和成就系统。"
                        }
                        div {
                            class: "flex justify-between items-center",
                            div {
                                class: "text-sm text-gray-500 dark:text-gray-400",
                                "本周完成率: 85%"
                            }
                            Link {
                                to: Route::Habits {},
                                class: "bg-orange-500 text-white px-4 py-2 rounded-lg hover:bg-orange-600 transition-colors",
                                "查看习惯"
                            }
                        }
                    }

                    // 数据统计卡片
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 hover:shadow-xl transition-shadow",
                        h3 {
                            class: "text-xl font-semibold text-gray-800 dark:text-white mb-4 flex items-center",
                            span { class: "text-2xl mr-3", "📊" }
                            "数据统计"
                        }
                        p {
                            class: "text-gray-600 dark:text-gray-300 mb-4",
                            "查看详细数据分析，洞察生活模式。支持多维度图表和趋势分析。"
                        }
                        div {
                            class: "flex justify-between items-center",
                            div {
                                class: "text-sm text-gray-500 dark:text-gray-400",
                                "生成综合报告"
                            }
                            Link {
                                to: Route::Statistics {},
                                class: "bg-indigo-500 text-white px-4 py-2 rounded-lg hover:bg-indigo-600 transition-colors",
                                "查看统计"
                            }
                        }
                    }

                    // 设置管理卡片
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 hover:shadow-xl transition-shadow",
                        h3 {
                            class: "text-xl font-semibold text-gray-800 dark:text-white mb-4 flex items-center",
                            span { class: "text-2xl mr-3", "⚙️" }
                            "系统设置"
                        }
                        p {
                            class: "text-gray-600 dark:text-gray-300 mb-4",
                            "自定义应用配置，管理数据备份。支持主题切换、同步设置和导入导出。"
                        }
                        div {
                            class: "flex justify-between items-center",
                            div {
                                class: "text-sm text-gray-500 dark:text-gray-400",
                                "个性化配置"
                            }
                            Link {
                                to: Route::Settings {},
                                class: "bg-gray-500 text-white px-4 py-2 rounded-lg hover:bg-gray-600 transition-colors",
                                "打开设置"
                            }
                        }
                    }
                }

                // 快速操作区域
                h2 {
                    class: "text-2xl font-bold text-gray-800 dark:text-white mb-6",
                    "快速操作"
                }
                div {
                    class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6",
                    div {
                        class: "grid grid-cols-2 md:grid-cols-4 gap-4",

                        // 开始计时
                        Link {
                            to: Route::TaskManagement {},
                            class: "flex flex-col items-center p-4 rounded-lg bg-blue-50 dark:bg-blue-900/20 hover:bg-blue-100 dark:hover:bg-blue-900/30 transition-colors",
                            div { class: "text-3xl mb-2", "▶️" }
                            span { class: "text-sm font-medium text-blue-700 dark:text-blue-300", "开始计时" }
                        }

                        // 记录支出
                        Link {
                            to: Route::Financial {},
                            class: "flex flex-col items-center p-4 rounded-lg bg-green-50 dark:bg-green-900/20 hover:bg-green-100 dark:hover:bg-green-900/30 transition-colors",
                            div { class: "text-3xl mb-2", "💸" }
                            span { class: "text-sm font-medium text-green-700 dark:text-green-300", "记录支出" }
                        }

                        // 写日记
                        Link {
                            to: Route::Diary {},
                            class: "flex flex-col items-center p-4 rounded-lg bg-purple-50 dark:bg-purple-900/20 hover:bg-purple-100 dark:hover:bg-purple-900/30 transition-colors",
                            div { class: "text-3xl mb-2", "✍️" }
                            span { class: "text-sm font-medium text-purple-700 dark:text-purple-300", "写日记" }
                        }

                        // 习惯打卡
                        Link {
                            to: Route::Habits {},
                            class: "flex flex-col items-center p-4 rounded-lg bg-orange-50 dark:bg-orange-900/20 hover:bg-orange-100 dark:hover:bg-orange-900/30 transition-colors",
                            div { class: "text-3xl mb-2", "📅" }
                            span { class: "text-sm font-medium text-orange-700 dark:text-orange-300", "习惯打卡" }
                        }
                    }
                }
            }
        }
    }
}
