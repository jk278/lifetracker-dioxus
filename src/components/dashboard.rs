//! # 仪表板组件
//!
//! 应用的主页面，显示各功能模块的概览和快速导航

use dioxus::prelude::*;
use dioxus_router::prelude::*;
// use dioxus_free_icons::{icons::bs_icons::*, Icon}; // 临时注释掉避免版本冲突

use super::app::Route;

/// 主仪表板组件
#[component]
pub fn Dashboard() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-100 dark:from-gray-900 dark:via-gray-800 dark:to-gray-700",

            // 顶部欢迎区域 - 现代化设计
            div {
                class: "relative overflow-hidden bg-gradient-to-r from-blue-600 via-purple-600 to-indigo-600 text-white py-16",
                
                // 背景装饰
                div {
                    class: "absolute inset-0 bg-black/10"
                }
                div {
                    class: "absolute top-0 left-0 w-full h-full",
                    div {
                        class: "absolute top-10 left-10 w-20 h-20 bg-white/10 rounded-full blur-xl"
                    }
                    div {
                        class: "absolute top-20 right-20 w-32 h-32 bg-white/5 rounded-full blur-2xl"
                    }
                    div {
                        class: "absolute bottom-10 left-1/3 w-24 h-24 bg-white/10 rounded-full blur-xl"
                    }
                }
                
                div {
                    class: "container mx-auto px-6 relative z-10",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            h1 {
                                class: "text-5xl font-bold mb-4 bg-gradient-to-r from-white to-blue-100 bg-clip-text text-transparent",
                                "欢迎回来！"
                            }
                            p {
                                class: "text-blue-100 text-xl font-medium flex items-center space-x-2",
                                span { class: "w-6 h-6 text-2xl", "☀️" }
                                span { "开始高效的一天！" }
                            }
                        }
                        div {
                            class: "hidden md:block",
                            div {
                                class: "text-6xl opacity-20",
                                span { class: "w-16 h-16", "📊" }
                            }
                        }
                    }
                }
            }

            div {
                class: "container mx-auto px-6 py-12 -mt-8 relative z-10",

                // 今日概览卡片 - 现代化设计
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-12",

                    // 今日时间追踪
                    div {
                        class: "group bg-white/70 dark:bg-gray-800/70 backdrop-blur-sm rounded-2xl shadow-xl hover:shadow-2xl p-6 border border-blue-200/50 dark:border-blue-800/50 transition-all duration-300 hover:-translate-y-2 hover:scale-105",
                        div {
                            class: "flex items-center justify-between",
                            div {
                                div {
                                    class: "flex items-center space-x-2 mb-3",
                                    span { class: "w-5 h-5 text-blue-500", "📊" }
                                    h3 {
                                        class: "text-lg font-semibold text-gray-800 dark:text-white",
                                        "今日追踪"
                                    }
                                }
                                p {
                                    class: "text-3xl font-bold text-blue-600 dark:text-blue-400 mt-2 mb-1",
                                    "2小时 30分"
                                }
                                p {
                                    class: "text-sm text-gray-500 dark:text-gray-400 flex items-center space-x-1",
                                    span { class: "w-4 h-4", "📊" }
                                    span { "3个任务" }
                                }
                            }
                            div {
                                class: "bg-blue-100 dark:bg-blue-900/30 p-4 rounded-xl group-hover:bg-blue-200 dark:group-hover:bg-blue-800/40 transition-colors",
                                span { class: "w-8 h-8 text-blue-600 dark:text-blue-400", "📊" }
                            }
                        }
                    }

                    // 本月收支
                    div {
                        class: "group bg-white/70 dark:bg-gray-800/70 backdrop-blur-sm rounded-2xl shadow-xl hover:shadow-2xl p-6 border border-green-200/50 dark:border-green-800/50 transition-all duration-300 hover:-translate-y-2 hover:scale-105",
                        div {
                            class: "flex items-center justify-between",
                            div {
                                div {
                                    class: "flex items-center space-x-2 mb-3",
                                    span { class: "w-5 h-5 text-green-500", "📊" }
                                    h3 {
                                        class: "text-lg font-semibold text-gray-800 dark:text-white",
                                        "本月收支"
                                    }
                                }
                                p {
                                    class: "text-3xl font-bold text-green-600 dark:text-green-400 mt-2 mb-1",
                                    "+¥1,280"
                                }
                                p {
                                    class: "text-sm text-gray-500 dark:text-gray-400",
                                    "收入 ¥8,500 | 支出 ¥7,220"
                                }
                            }
                            div {
                                class: "bg-green-100 dark:bg-green-900/30 p-4 rounded-xl group-hover:bg-green-200 dark:group-hover:bg-green-800/40 transition-colors",
                                span { class: "w-8 h-8 text-green-600 dark:text-green-400", "📊" }
                            }
                        }
                    }

                    // 笔记条数
                    div {
                        class: "group bg-white/70 dark:bg-gray-800/70 backdrop-blur-sm rounded-2xl shadow-xl hover:shadow-2xl p-6 border border-purple-200/50 dark:border-purple-800/50 transition-all duration-300 hover:-translate-y-2 hover:scale-105",
                        div {
                            class: "flex items-center justify-between",
                            div {
                                div {
                                    class: "flex items-center space-x-2 mb-3",
                                    span { class: "w-5 h-5 text-purple-500", "📊" }
                                    h3 {
                                        class: "text-lg font-semibold text-gray-800 dark:text-white",
                                        "笔记记录"
                                    }
                                }
                                p {
                                    class: "text-3xl font-bold text-purple-600 dark:text-purple-400 mt-2 mb-1",
                                    "24 篇"
                                }
                                p {
                                    class: "text-sm text-gray-500 dark:text-gray-400 flex items-center space-x-1",
                                    span { class: "w-4 h-4", "📊" }
                                    span { "本月新增 8 篇" }
                                }
                            }
                            div {
                                class: "bg-purple-100 dark:bg-purple-900/30 p-4 rounded-xl group-hover:bg-purple-200 dark:group-hover:bg-purple-800/40 transition-colors",
                                span { class: "w-8 h-8 text-purple-600 dark:text-purple-400", "📊" }
                            }
                        }
                    }

                    // 习惯打卡
                    div {
                        class: "group bg-white/70 dark:bg-gray-800/70 backdrop-blur-sm rounded-2xl shadow-xl hover:shadow-2xl p-6 border border-orange-200/50 dark:border-orange-800/50 transition-all duration-300 hover:-translate-y-2 hover:scale-105",
                        div {
                            class: "flex items-center justify-between",
                            div {
                                div {
                                    class: "flex items-center space-x-2 mb-3",
                                    span { class: "w-5 h-5 text-orange-500", "📊" }
                                    h3 {
                                        class: "text-lg font-semibold text-gray-800 dark:text-white",
                                        "习惯打卡"
                                    }
                                }
                                p {
                                    class: "text-3xl font-bold text-orange-600 dark:text-orange-400 mt-2 mb-1",
                                    "85%"
                                }
                                p {
                                    class: "text-sm text-gray-500 dark:text-gray-400 flex items-center space-x-1",
                                    span { class: "w-4 h-4", "📊" }
                                    span { "本周完成率" }
                                }
                            }
                            div {
                                class: "bg-orange-100 dark:bg-orange-900/30 p-4 rounded-xl group-hover:bg-orange-200 dark:group-hover:bg-orange-800/40 transition-colors",
                                span { class: "w-8 h-8 text-orange-600 dark:text-orange-400", "📊" }
                            }
                        }
                    }
                }

                // 主要功能卡片
                div {
                    class: "flex items-center space-x-3 mb-8",
                    span { class: "w-8 h-8 text-gray-700 dark:text-gray-300", "📊" }
                    h2 {
                        class: "text-3xl font-bold bg-gradient-to-r from-gray-700 to-gray-900 dark:from-gray-300 dark:to-gray-100 bg-clip-text text-transparent",
                        "功能导航"
                    }
                }
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 mb-12",

                    // 时间追踪卡片
                    div {
                        class: "group bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl shadow-lg hover:shadow-2xl p-8 border border-gray-200/50 dark:border-gray-700/50 transition-all duration-500 hover:-translate-y-3 hover:scale-105",
                        div {
                            class: "flex items-center space-x-4 mb-6",
                            div {
                                class: "bg-gradient-to-br from-blue-500 to-blue-600 p-4 rounded-2xl shadow-lg group-hover:shadow-xl transition-all duration-300 group-hover:scale-110",
                                span { class: "w-8 h-8 text-white", "📊" }
                            }
                            h3 {
                                class: "text-2xl font-bold text-gray-800 dark:text-white",
                                "时间追踪"
                            }
                        }
                        p {
                            class: "text-gray-600 dark:text-gray-300 mb-6 leading-relaxed",
                            "记录你的工作时间，提高效率。支持任务分类、计时器和统计分析，帮助你更好地管理时间。"
                        }
                        div {
                            class: "flex justify-between items-center",
                            div {
                                class: "flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400",
                                span { class: "w-4 h-4", "📊" }
                                span { "今日: 2小时30分" }
                            }
                            Link {
                                to: Route::TaskManagement {},
                                class: "group/btn bg-gradient-to-r from-blue-500 to-blue-600 text-white px-6 py-3 rounded-xl hover:from-blue-600 hover:to-blue-700 transition-all duration-300 shadow-lg hover:shadow-xl transform hover:scale-105 flex items-center space-x-2",
                                span { class: "w-4 h-4 group-hover/btn:scale-110 transition-transform", "📊" }
                                span { "开始追踪" }
                            }
                        }
                    }

                    // 财务管理卡片
                    div {
                        class: "group bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl shadow-lg hover:shadow-2xl p-8 border border-gray-200/50 dark:border-gray-700/50 transition-all duration-500 hover:-translate-y-3 hover:scale-105",
                        div {
                            class: "flex items-center space-x-4 mb-6",
                            div {
                                class: "bg-gradient-to-br from-green-500 to-emerald-600 p-4 rounded-2xl shadow-lg group-hover:shadow-xl transition-all duration-300 group-hover:scale-110",
                                span { class: "w-8 h-8 text-white", "📊" }
                            }
                            h3 {
                                class: "text-2xl font-bold text-gray-800 dark:text-white",
                                "财务管理"
                            }
                        }
                        p {
                            class: "text-gray-600 dark:text-gray-300 mb-6 leading-relaxed",
                            "管理收入支出，制定预算计划。支持多账户、分类统计和财务报表，让理财更简单。"
                        }
                        div {
                            class: "flex justify-between items-center",
                            div {
                                class: "flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400",
                                span { class: "w-4 h-4", "📊" }
                                span { "本月结余: +¥1,280" }
                            }
                            Link {
                                to: Route::Financial {},
                                class: "group/btn bg-gradient-to-r from-green-500 to-emerald-600 text-white px-6 py-3 rounded-xl hover:from-green-600 hover:to-emerald-700 transition-all duration-300 shadow-lg hover:shadow-xl transform hover:scale-105 flex items-center space-x-2",
                                span { class: "w-4 h-4 group-hover/btn:scale-110 transition-transform", "📊" }
                                span { "查看财务" }
                            }
                        }
                    }

                    // 日记功能卡片
                    div {
                        class: "group bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl shadow-lg hover:shadow-2xl p-8 border border-gray-200/50 dark:border-gray-700/50 transition-all duration-500 hover:-translate-y-3 hover:scale-105",
                        div {
                            class: "flex items-center space-x-4 mb-6",
                            div {
                                class: "bg-gradient-to-br from-purple-500 to-violet-600 p-4 rounded-2xl shadow-lg group-hover:shadow-xl transition-all duration-300 group-hover:scale-110",
                                span { class: "w-8 h-8 text-white", "📊" }
                            }
                            h3 {
                                class: "text-2xl font-bold text-gray-800 dark:text-white",
                                "日记笔记"
                            }
                        }
                        p {
                            class: "text-gray-600 dark:text-gray-300 mb-6 leading-relaxed",
                            "记录生活点滴，整理思考感悟。支持富文本编辑、标签分类和搜索，让记忆永存。"
                        }
                        div {
                            class: "flex justify-between items-center",
                            div {
                                class: "flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400",
                                span { class: "w-4 h-4", "📊" }
                                span { "本月: 8 篇新笔记" }
                            }
                            Link {
                                to: Route::Diary {},
                                class: "group/btn bg-gradient-to-r from-purple-500 to-violet-600 text-white px-6 py-3 rounded-xl hover:from-purple-600 hover:to-violet-700 transition-all duration-300 shadow-lg hover:shadow-xl transform hover:scale-105 flex items-center space-x-2",
                                span { class: "w-4 h-4 group-hover/btn:scale-110 transition-transform", "📊" }
                                span { "写日记" }
                            }
                        }
                    }

                    // 习惯打卡卡片
                    div {
                        class: "group bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl shadow-lg hover:shadow-2xl p-8 border border-gray-200/50 dark:border-gray-700/50 transition-all duration-500 hover:-translate-y-3 hover:scale-105",
                        div {
                            class: "flex items-center space-x-4 mb-6",
                            div {
                                class: "bg-gradient-to-br from-orange-500 to-amber-600 p-4 rounded-2xl shadow-lg group-hover:shadow-xl transition-all duration-300 group-hover:scale-110",
                                span { class: "w-8 h-8 text-white", "📊" }
                            }
                            h3 {
                                class: "text-2xl font-bold text-gray-800 dark:text-white",
                                "习惯打卡"
                            }
                        }
                        p {
                            class: "text-gray-600 dark:text-gray-300 mb-6 leading-relaxed",
                            "培养良好习惯，追踪进度目标。支持提醒设置、连续统计和成就系统，建立更好的自己。"
                        }
                        div {
                            class: "flex justify-between items-center",
                            div {
                                class: "flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400",
                                span { class: "w-4 h-4", "📊" }
                                span { "本周完成率: 85%" }
                            }
                            Link {
                                to: Route::Habits {},
                                class: "group/btn bg-gradient-to-r from-orange-500 to-amber-600 text-white px-6 py-3 rounded-xl hover:from-orange-600 hover:to-amber-700 transition-all duration-300 shadow-lg hover:shadow-xl transform hover:scale-105 flex items-center space-x-2",
                                span { class: "w-4 h-4 group-hover/btn:scale-110 transition-transform", "📊" }
                                span { "查看习惯" }
                            }
                        }
                    }

                    // 数据统计卡片
                    div {
                        class: "group bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl shadow-lg hover:shadow-2xl p-8 border border-gray-200/50 dark:border-gray-700/50 transition-all duration-500 hover:-translate-y-3 hover:scale-105",
                        div {
                            class: "flex items-center space-x-4 mb-6",
                            div {
                                class: "bg-gradient-to-br from-indigo-500 to-blue-600 p-4 rounded-2xl shadow-lg group-hover:shadow-xl transition-all duration-300 group-hover:scale-110",
                                span { class: "w-8 h-8 text-white", "📊" }
                            }
                            h3 {
                                class: "text-2xl font-bold text-gray-800 dark:text-white",
                                "数据统计"
                            }
                        }
                        p {
                            class: "text-gray-600 dark:text-gray-300 mb-6 leading-relaxed",
                            "查看详细数据分析，洞察生活模式。支持多维度图表和趋势分析，数据驱动决策。"
                        }
                        div {
                            class: "flex justify-between items-center",
                            div {
                                class: "flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400",
                                span { class: "w-4 h-4", "📊" }
                                span { "生成综合报告" }
                            }
                            Link {
                                to: Route::Statistics {},
                                class: "group/btn bg-gradient-to-r from-indigo-500 to-blue-600 text-white px-6 py-3 rounded-xl hover:from-indigo-600 hover:to-blue-700 transition-all duration-300 shadow-lg hover:shadow-xl transform hover:scale-105 flex items-center space-x-2",
                                span { class: "w-4 h-4 group-hover/btn:scale-110 transition-transform", "📊" }
                                span { "查看统计" }
                            }
                        }
                    }

                    // 设置管理卡片
                    div {
                        class: "group bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl shadow-lg hover:shadow-2xl p-8 border border-gray-200/50 dark:border-gray-700/50 transition-all duration-500 hover:-translate-y-3 hover:scale-105",
                        div {
                            class: "flex items-center space-x-4 mb-6",
                            div {
                                class: "bg-gradient-to-br from-gray-600 to-gray-700 p-4 rounded-2xl shadow-lg group-hover:shadow-xl transition-all duration-300 group-hover:scale-110",
                                span { class: "w-8 h-8 text-white", "📊" }
                            }
                            h3 {
                                class: "text-2xl font-bold text-gray-800 dark:text-white",
                                "系统设置"
                            }
                        }
                        p {
                            class: "text-gray-600 dark:text-gray-300 mb-6 leading-relaxed",
                            "自定义应用配置，管理数据备份。支持主题切换、同步设置和导入导出，打造专属体验。"
                        }
                        div {
                            class: "flex justify-between items-center",
                            div {
                                class: "flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400",
                                span { class: "w-4 h-4", "📊" }
                                span { "个性化配置" }
                            }
                            Link {
                                to: Route::Settings {},
                                class: "group/btn bg-gradient-to-r from-gray-600 to-gray-700 text-white px-6 py-3 rounded-xl hover:from-gray-700 hover:to-gray-800 transition-all duration-300 shadow-lg hover:shadow-xl transform hover:scale-105 flex items-center space-x-2",
                                span { class: "w-4 h-4 group-hover/btn:scale-110 transition-transform", "📊" }
                                span { "打开设置" }
                            }
                        }
                    }
                }

                // 快速操作区域
                div {
                    class: "flex items-center space-x-3 mb-8",
                    span { class: "w-8 h-8 text-gray-700 dark:text-gray-300", "📊" }
                    h2 {
                        class: "text-3xl font-bold bg-gradient-to-r from-gray-700 to-gray-900 dark:from-gray-300 dark:to-gray-100 bg-clip-text text-transparent",
                        "快速操作"
                    }
                }
                div {
                    class: "bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl shadow-xl p-8 border border-gray-200/50 dark:border-gray-700/50",
                    div {
                        class: "grid grid-cols-2 md:grid-cols-4 gap-6",

                        // 开始计时
                        Link {
                            to: Route::TaskManagement {},
                            class: "group flex flex-col items-center p-6 rounded-2xl bg-gradient-to-br from-blue-50 to-blue-100/50 dark:from-blue-900/20 dark:to-blue-800/10 hover:from-blue-100 hover:to-blue-200/70 dark:hover:from-blue-800/30 dark:hover:to-blue-700/20 transition-all duration-300 hover:scale-110 hover:shadow-lg border border-blue-200/30 dark:border-blue-700/30",
                            div { 
                                class: "bg-blue-500 p-4 rounded-2xl mb-4 group-hover:scale-110 transition-transform duration-300 shadow-lg",
                                span { class: "w-8 h-8 text-white", "📊" }
                            }
                            span { class: "text-sm font-bold text-blue-700 dark:text-blue-300 group-hover:text-blue-800 dark:group-hover:text-blue-200", "开始计时" }
                        }

                        // 记录支出
                        Link {
                            to: Route::Financial {},
                            class: "group flex flex-col items-center p-6 rounded-2xl bg-gradient-to-br from-green-50 to-green-100/50 dark:from-green-900/20 dark:to-green-800/10 hover:from-green-100 hover:to-green-200/70 dark:hover:from-green-800/30 dark:hover:to-green-700/20 transition-all duration-300 hover:scale-110 hover:shadow-lg border border-green-200/30 dark:border-green-700/30",
                            div { 
                                class: "bg-green-500 p-4 rounded-2xl mb-4 group-hover:scale-110 transition-transform duration-300 shadow-lg",
                                span { class: "w-8 h-8 text-white", "📊" }
                            }
                            span { class: "text-sm font-bold text-green-700 dark:text-green-300 group-hover:text-green-800 dark:group-hover:text-green-200", "记录支出" }
                        }

                        // 写日记
                        Link {
                            to: Route::Diary {},
                            class: "group flex flex-col items-center p-6 rounded-2xl bg-gradient-to-br from-purple-50 to-purple-100/50 dark:from-purple-900/20 dark:to-purple-800/10 hover:from-purple-100 hover:to-purple-200/70 dark:hover:from-purple-800/30 dark:hover:to-purple-700/20 transition-all duration-300 hover:scale-110 hover:shadow-lg border border-purple-200/30 dark:border-purple-700/30",
                            div { 
                                class: "bg-purple-500 p-4 rounded-2xl mb-4 group-hover:scale-110 transition-transform duration-300 shadow-lg",
                                span { class: "w-8 h-8 text-white", "📊" }
                            }
                            span { class: "text-sm font-bold text-purple-700 dark:text-purple-300 group-hover:text-purple-800 dark:group-hover:text-purple-200", "写日记" }
                        }

                        // 习惯打卡
                        Link {
                            to: Route::Habits {},
                            class: "group flex flex-col items-center p-6 rounded-2xl bg-gradient-to-br from-orange-50 to-orange-100/50 dark:from-orange-900/20 dark:to-orange-800/10 hover:from-orange-100 hover:to-orange-200/70 dark:hover:from-orange-800/30 dark:hover:to-orange-700/20 transition-all duration-300 hover:scale-110 hover:shadow-lg border border-orange-200/30 dark:border-orange-700/30",
                            div { 
                                class: "bg-orange-500 p-4 rounded-2xl mb-4 group-hover:scale-110 transition-transform duration-300 shadow-lg",
                                span { class: "w-8 h-8 text-white", "📊" }
                            }
                            span { class: "text-sm font-bold text-orange-700 dark:text-orange-300 group-hover:text-orange-800 dark:group-hover:text-orange-200", "习惯打卡" }
                        }
                    }
                }
            }
        }
    }
}
