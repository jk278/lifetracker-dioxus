//! # 习惯打卡模块
//!
//! 包含习惯追踪、打卡记录等功能

use crate::components::common::Card;
use dioxus::prelude::*;

/// 习惯打卡主页面组件
#[component]
pub fn HabitsPage() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-gradient-to-br from-orange-50 via-amber-50 to-yellow-100 dark:from-gray-900 dark:via-gray-800 dark:to-gray-700",

            div {
                class: "max-w-7xl mx-auto px-6 py-8",

                // 页面标题
                header {
                    class: "mb-8",
                    h1 {
                        class: "text-4xl font-bold text-gray-800 dark:text-white mb-4 flex items-center space-x-3",
                        span { class: "text-5xl", "🎯" }
                        span { "习惯打卡" }
                    }
                    p {
                        class: "text-lg text-gray-600 dark:text-gray-300",
                        "培养良好习惯，追踪进度目标，建立更好的自己"
                    }
                }

                // 今日习惯概览
                div {
                    class: "grid grid-cols-1 md:grid-cols-3 gap-6 mb-8",
                    
                    Card {
                        shadow: true,
                        class: "bg-white dark:bg-gray-800 p-6",
                        div {
                            class: "text-center",
                            div { class: "text-4xl mb-3", "🔥" }
                            h3 { class: "text-lg font-semibold text-gray-900 dark:text-white mb-2", "连续天数" }
                            p { class: "text-3xl font-bold text-orange-600 dark:text-orange-400", "7 天" }
                            p { class: "text-sm text-gray-500 dark:text-gray-400 mt-1", "最长记录: 30 天" }
                        }
                    }

                    Card {
                        shadow: true,
                        class: "bg-white dark:bg-gray-800 p-6",
                        div {
                            class: "text-center",
                            div { class: "text-4xl mb-3", "📊" }
                            h3 { class: "text-lg font-semibold text-gray-900 dark:text-white mb-2", "完成率" }
                            p { class: "text-3xl font-bold text-green-600 dark:text-green-400", "85%" }
                            p { class: "text-sm text-gray-500 dark:text-gray-400 mt-1", "本周平均" }
                        }
                    }

                    Card {
                        shadow: true,
                        class: "bg-white dark:bg-gray-800 p-6",
                        div {
                            class: "text-center",
                            div { class: "text-4xl mb-3", "⭐" }
                            h3 { class: "text-lg font-semibold text-gray-900 dark:text-white mb-2", "总积分" }
                            p { class: "text-3xl font-bold text-purple-600 dark:text-purple-400", "1,250" }
                            p { class: "text-sm text-gray-500 dark:text-gray-400 mt-1", "累计获得" }
                        }
                    }
                }

                // 今日习惯列表
                Card {
                    shadow: true,
                    class: "bg-white dark:bg-gray-800 p-6 mb-8",
                    h2 { class: "text-2xl font-bold text-gray-900 dark:text-white mb-6 flex items-center space-x-2",
                        span { class: "text-2xl", "📅" }
                        span { "今日习惯" }
                    }

                    div {
                        class: "space-y-4",
                        
                        // 示例习惯项
                        for (habit, icon, completed, streak) in [
                            ("早起 (6:30前)", "🌅", true, 7),
                            ("运动锻炼", "💪", true, 5),
                            ("阅读 30 分钟", "📚", false, 0),
                            ("冥想 10 分钟", "🧘", false, 0),
                            ("喝 8 杯水", "💧", true, 12),
                        ] {
                            div {
                                class: format!("flex items-center justify-between p-4 rounded-lg border transition-all duration-200 {}",
                                    if completed {
                                        "bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800"
                                    } else {
                                        "bg-gray-50 dark:bg-gray-700 border-gray-200 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-600"
                                    }
                                ),
                                div {
                                    class: "flex items-center space-x-4",
                                    span { class: "text-2xl", "{icon}" }
                                    div {
                                        h3 { class: "font-medium text-gray-900 dark:text-white", "{habit}" }
                                        if streak > 0 {
                                            p { class: "text-sm text-orange-600 dark:text-orange-400", "🔥 连续 {streak} 天" }
                                        }
                                    }
                                }
                                
                                button {
                                    class: format!("px-4 py-2 rounded-lg font-medium transition-colors {}",
                                        if completed {
                                            "bg-green-600 text-white cursor-default"
                                        } else {
                                            "bg-orange-600 hover:bg-orange-700 text-white"
                                        }
                                    ),
                                    disabled: completed,
                                    if completed { "✅ 已完成" } else { "👆 打卡" }
                                }
                            }
                        }
                    }
                }

                // 功能开发中提示
                Card {
                    shadow: true,
                    class: "bg-gradient-to-r from-orange-100 to-yellow-100 dark:from-orange-900/20 dark:to-yellow-900/20 border border-orange-200 dark:border-orange-800 p-8",
                    div {
                        class: "text-center",
                        div { class: "text-6xl mb-4", "🚧" }
                        h3 { class: "text-xl font-bold text-orange-800 dark:text-orange-200 mb-3", "功能开发中" }
                        p { class: "text-orange-700 dark:text-orange-300 mb-4", "习惯管理功能正在积极开发中，即将支持：" }
                        div {
                            class: "grid grid-cols-1 md:grid-cols-2 gap-4 text-left",
                            div {
                                class: "space-y-2",
                                p { class: "text-orange-700 dark:text-orange-300 flex items-center space-x-2",
                                    span { "✨" }
                                    span { "自定义习惯创建和编辑" }
                                }
                                p { class: "text-orange-700 dark:text-orange-300 flex items-center space-x-2",
                                    span { "⏰" }
                                    span { "提醒时间设置" }
                                }
                                p { class: "text-orange-700 dark:text-orange-300 flex items-center space-x-2",
                                    span { "📊" }
                                    span { "详细的统计分析" }
                                }
                            }
                            div {
                                class: "space-y-2",
                                p { class: "text-orange-700 dark:text-orange-300 flex items-center space-x-2",
                                    span { "🏆" }
                                    span { "成就系统和奖励机制" }
                                }
                                p { class: "text-orange-700 dark:text-orange-300 flex items-center space-x-2",
                                    span { "📱" }
                                    span { "移动端推送通知" }
                                }
                                p { class: "text-orange-700 dark:text-orange-300 flex items-center space-x-2",
                                    span { "📈" }
                                    span { "进度图表和趋势分析" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
