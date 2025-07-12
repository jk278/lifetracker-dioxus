//! # 日记统计组件
//!
//! 显示笔记统计信息，包括概览、标签分布、心情分布等

use dioxus::prelude::*;

/// 标签统计
#[derive(Debug, Clone, PartialEq)]
pub struct TagStat {
    pub tag: String,
    pub count: u32,
    pub percentage: u32,
}

/// 心情统计
#[derive(Debug, Clone, PartialEq)]
pub struct MoodStat {
    pub mood: String,
    pub count: u32,
    pub percentage: u32,
}

/// 心情信息
#[derive(Debug, Clone, PartialEq)]
pub struct MoodInfo {
    pub emoji: &'static str,
    pub label: String,
    pub color: &'static str,
}

/// 趋势数据
#[derive(Debug, Clone, PartialEq)]
pub struct TrendData {
    pub date: String,
    pub count: u32,
}

/// 统计数据
#[derive(Debug, Clone, PartialEq)]
pub struct StatsData {
    pub total_notes: u32,
    pub favorite_notes: u32,
    pub archived_notes: u32,
    pub notes_this_week: u32,
    pub notes_this_month: u32,
    pub most_used_tags: Vec<TagStat>,
    pub mood_distribution: Vec<MoodStat>,
    pub daily_notes_trend: Vec<TrendData>,
}

/// 获取心情信息
fn get_mood_info(mood: &str) -> MoodInfo {
    match mood {
        "happy" => MoodInfo {
            emoji: "😊",
            label: "开心".to_string(),
            color: "bg-yellow-500",
        },
        "sad" => MoodInfo {
            emoji: "😢",
            label: "难过".to_string(),
            color: "bg-blue-500",
        },
        "neutral" => MoodInfo {
            emoji: "😐",
            label: "平静".to_string(),
            color: "bg-gray-500",
        },
        "excited" => MoodInfo {
            emoji: "🤩",
            label: "兴奋".to_string(),
            color: "bg-orange-500",
        },
        "stressed" => MoodInfo {
            emoji: "😰",
            label: "压力".to_string(),
            color: "bg-red-500",
        },
        "relaxed" => MoodInfo {
            emoji: "😌",
            label: "放松".to_string(),
            color: "bg-green-500",
        },
        "anxious" => MoodInfo {
            emoji: "😟",
            label: "焦虑".to_string(),
            color: "bg-purple-500",
        },
        "confident" => MoodInfo {
            emoji: "😎",
            label: "自信".to_string(),
            color: "bg-indigo-500",
        },
        _ => MoodInfo {
            emoji: "😐",
            label: mood.to_string(),
            color: "bg-gray-500",
        },
    }
}

/// 日记统计组件
#[component]
pub fn NotesStats() -> Element {
    // 模拟统计数据
    let stats = use_signal(|| StatsData {
        total_notes: 24,
        favorite_notes: 8,
        archived_notes: 3,
        notes_this_week: 5,
        notes_this_month: 15,
        most_used_tags: vec![
            TagStat {
                tag: "工作".to_string(),
                count: 12,
                percentage: 50,
            },
            TagStat {
                tag: "学习".to_string(),
                count: 8,
                percentage: 33,
            },
            TagStat {
                tag: "生活".to_string(),
                count: 6,
                percentage: 25,
            },
            TagStat {
                tag: "想法".to_string(),
                count: 4,
                percentage: 17,
            },
            TagStat {
                tag: "总结".to_string(),
                count: 3,
                percentage: 13,
            },
        ],
        mood_distribution: vec![
            MoodStat {
                mood: "happy".to_string(),
                count: 10,
                percentage: 42,
            },
            MoodStat {
                mood: "excited".to_string(),
                count: 6,
                percentage: 25,
            },
            MoodStat {
                mood: "neutral".to_string(),
                count: 4,
                percentage: 17,
            },
            MoodStat {
                mood: "relaxed".to_string(),
                count: 3,
                percentage: 13,
            },
            MoodStat {
                mood: "confident".to_string(),
                count: 1,
                percentage: 4,
            },
        ],
        daily_notes_trend: vec![
            TrendData {
                date: "01-10".to_string(),
                count: 2,
            },
            TrendData {
                date: "01-11".to_string(),
                count: 1,
            },
            TrendData {
                date: "01-12".to_string(),
                count: 3,
            },
            TrendData {
                date: "01-13".to_string(),
                count: 0,
            },
            TrendData {
                date: "01-14".to_string(),
                count: 2,
            },
            TrendData {
                date: "01-15".to_string(),
                count: 4,
            },
            TrendData {
                date: "01-16".to_string(),
                count: 1,
            },
        ],
    });

    rsx! {
        div { class: "space-y-6",
            // 顶部工具栏
            div { class: "flex items-center justify-between",
                div { class: "flex items-center space-x-3",
                    span { class: "text-2xl", "📊" }
                    h1 { class: "text-xl font-bold text-gray-900 dark:text-white",
                        "笔记统计"
                    }
                }
            }

            // 概览卡片
            div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                // 总笔记
                div { class: "bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700",
                    div { class: "flex items-center justify-between",
                        div {
                            p { class: "text-sm text-gray-600 dark:text-gray-400", "总笔记" }
                            p { class: "text-2xl font-bold text-gray-900 dark:text-white",
                                "{stats.read().total_notes}"
                            }
                        }
                        div { class: "w-8 h-8 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center",
                            span { class: "text-lg text-blue-600 dark:text-blue-400", "📊" }
                        }
                    }
                }

                // 收藏
                div { class: "bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700",
                    div { class: "flex items-center justify-between",
                        div {
                            p { class: "text-sm text-gray-600 dark:text-gray-400", "收藏" }
                            p { class: "text-2xl font-bold text-gray-900 dark:text-white",
                                "{stats.read().favorite_notes}"
                            }
                        }
                        div { class: "w-8 h-8 bg-red-100 dark:bg-red-900/30 rounded-lg flex items-center justify-center",
                            span { class: "text-lg text-red-600 dark:text-red-400", "❤️" }
                        }
                    }
                }

                // 本周
                div { class: "bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700",
                    div { class: "flex items-center justify-between",
                        div {
                            p { class: "text-sm text-gray-600 dark:text-gray-400", "本周" }
                            p { class: "text-2xl font-bold text-gray-900 dark:text-white",
                                "{stats.read().notes_this_week}"
                            }
                        }
                        div { class: "w-8 h-8 bg-green-100 dark:bg-green-900/30 rounded-lg flex items-center justify-center",
                            span { class: "text-lg text-green-600 dark:text-green-400", "📈" }
                        }
                    }
                }

                // 归档
                div { class: "bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700",
                    div { class: "flex items-center justify-between",
                        div {
                            p { class: "text-sm text-gray-600 dark:text-gray-400", "归档" }
                            p { class: "text-2xl font-bold text-gray-900 dark:text-white",
                                "{stats.read().archived_notes}"
                            }
                        }
                        div { class: "w-8 h-8 bg-gray-100 dark:bg-gray-700 rounded-lg flex items-center justify-center",
                            span { class: "text-lg text-gray-600 dark:text-gray-400", "📦" }
                        }
                    }
                }
            }

            // 标签统计和心情分布
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                // 常用标签
                div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700",
                    div { class: "p-4 border-b border-gray-200 dark:border-gray-700",
                        div { class: "flex items-center space-x-2",
                            span { class: "text-lg text-blue-600", "🏷️" }
                            h3 { class: "font-semibold text-gray-900 dark:text-white",
                                "常用标签"
                            }
                        }
                    }
                    div { class: "p-4 space-y-3",
                        for (index, tag) in stats.read().most_used_tags.iter().enumerate() {
                            div {
                                key: "{tag.tag}",
                                class: "flex items-center justify-between",
                                div { class: "flex items-center space-x-3",
                                    span { class: "w-6 h-6 bg-blue-100 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 rounded-full flex items-center justify-center text-xs font-medium",
                                        "{index + 1}"
                                    }
                                    span { class: "text-gray-900 dark:text-white font-medium",
                                        "{tag.tag}"
                                    }
                                }
                                div { class: "flex items-center space-x-2",
                                    span { class: "text-sm text-gray-600 dark:text-gray-400",
                                        "{tag.count}次"
                                    }
                                    div { class: "w-16 bg-gray-200 dark:bg-gray-700 rounded-full h-2",
                                        div {
                                            class: "bg-blue-600 h-2 rounded-full transition-all duration-300",
                                            style: "width: {tag.percentage}%"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 心情分布
                div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700",
                    div { class: "p-4 border-b border-gray-200 dark:border-gray-700",
                        div { class: "flex items-center space-x-2",
                            span { class: "text-lg text-blue-600", "😊" }
                            h3 { class: "font-semibold text-gray-900 dark:text-white",
                                "心情分布"
                            }
                        }
                    }
                    div { class: "p-4 space-y-3",
                        for mood in stats.read().mood_distribution.iter() {
                            div {
                                key: "{mood.mood}",
                                class: "flex items-center justify-between",
                                div { class: "flex items-center space-x-3",
                                    span { class: "text-lg", "{get_mood_info(&mood.mood).emoji}" }
                                    span { class: "text-gray-900 dark:text-white font-medium",
                                        "{get_mood_info(&mood.mood).label}"
                                    }
                                }
                                div { class: "flex items-center space-x-2",
                                    span { class: "text-sm text-gray-600 dark:text-gray-400",
                                        "{mood.count}次"
                                    }
                                    div { class: "w-16 bg-gray-200 dark:bg-gray-700 rounded-full h-2",
                                        div {
                                            class: "{get_mood_info(&mood.mood).color} h-2 rounded-full transition-all duration-300",
                                            style: "width: {mood.percentage}%"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 每日趋势
            div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700",
                div { class: "p-4 border-b border-gray-200 dark:border-gray-700",
                    div { class: "flex items-center space-x-2",
                        span { class: "text-lg text-blue-600", "📈" }
                        h3 { class: "font-semibold text-gray-900 dark:text-white",
                            "每日趋势 (最近7天)"
                        }
                    }
                }
                div { class: "p-4",
                    div { class: "flex items-end justify-between space-x-2 h-32",
                        for trend in stats.read().daily_notes_trend.iter() {
                            div {
                                key: "{trend.date}",
                                class: "flex flex-col items-center space-y-2 flex-1",
                                div { class: "flex-1 flex items-end w-full",
                                    div {
                                        class: "bg-blue-500 dark:bg-blue-400 w-full rounded-t-sm transition-all duration-300 hover:bg-blue-600 dark:hover:bg-blue-300",
                                        style: "height: {if trend.count > 0 { (trend.count as f32 / 4.0 * 100.0) as i32 } else { 4 }}%",
                                        title: "{trend.count} 篇笔记"
                                    }
                                }
                                div { class: "text-xs text-gray-500 dark:text-gray-400",
                                    "{trend.date}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
