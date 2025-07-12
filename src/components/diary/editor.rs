//! # 日记编辑器组件
//!
//! 提供富文本编辑功能，包括标题、内容、心情和标签

use dioxus::prelude::*;

/// 心情选项
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mood {
    Happy,
    Sad,
    Neutral,
    Excited,
    Stressed,
    Relaxed,
    Anxious,
    Confident,
}

impl Mood {
    fn label(&self) -> &'static str {
        match self {
            Mood::Happy => "😊 开心",
            Mood::Sad => "😢 难过",
            Mood::Neutral => "😐 平静",
            Mood::Excited => "🤩 兴奋",
            Mood::Stressed => "😰 压力",
            Mood::Relaxed => "😌 放松",
            Mood::Anxious => "😟 焦虑",
            Mood::Confident => "😎 自信",
        }
    }

    fn color(&self) -> &'static str {
        match self {
            Mood::Happy => "text-yellow-500",
            Mood::Sad => "text-blue-500",
            Mood::Neutral => "text-gray-500",
            Mood::Excited => "text-orange-500",
            Mood::Stressed => "text-red-500",
            Mood::Relaxed => "text-green-500",
            Mood::Anxious => "text-purple-500",
            Mood::Confident => "text-indigo-500",
        }
    }

    fn all() -> [Mood; 8] {
        [
            Mood::Happy,
            Mood::Sad,
            Mood::Neutral,
            Mood::Excited,
            Mood::Stressed,
            Mood::Relaxed,
            Mood::Anxious,
            Mood::Confident,
        ]
    }
}

/// 日记编辑器组件
#[component]
pub fn NotesEditor() -> Element {
    // 状态管理
    let mut title = use_signal(|| String::new());
    let mut content = use_signal(|| String::new());
    let mut selected_mood = use_signal(|| None::<Mood>);
    let mut tags = use_signal(|| Vec::<String>::new());
    let mut is_favorite = use_signal(|| false);
    let mut tag_input = use_signal(|| String::new());

    // 处理保存
    let handle_save = move |_| {
        log::info!(
            "保存笔记: 标题={}, 内容长度={}, 心情={:?}, 标签={:?}, 收藏={}",
            title.read().clone(),
            content.read().len(),
            selected_mood.read().clone(),
            tags.read().clone(),
            is_favorite.read()
        );
        // TODO: 实现保存逻辑
    };

    // 处理心情选择
    let mut handle_mood_select = move |mood: Mood| {
        if selected_mood.read().as_ref() == Some(&mood) {
            selected_mood.set(None);
        } else {
            selected_mood.set(Some(mood));
        }
    };

    // 处理标签添加
    let mut handle_add_tag = move |tag: String| {
        if !tag.is_empty() && !tags.read().contains(&tag) {
            tags.write().push(tag);
        }
    };

    // 处理标签删除
    let mut handle_remove_tag = move |tag_to_remove: String| {
        tags.write().retain(|tag| tag != &tag_to_remove);
    };

    rsx! {
        div { class: "space-y-6",
            // 顶部工具栏
            div { class: "flex items-center justify-between",
                div { class: "flex items-center space-x-3",
                    span { class: "text-2xl", "✍️" }
                    h1 { class: "text-xl font-bold text-gray-900 dark:text-white",
                        "笔记编辑器"
                    }
                }
                div { class: "flex items-center space-x-2",
                    button {
                        class: if is_favorite() {
                            "p-2 text-red-500 bg-red-50 dark:bg-red-900/20 rounded-lg transition-colors"
                        } else {
                            "p-2 text-gray-600 dark:text-gray-300 hover:text-red-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                        },
                        onclick: move |_| is_favorite.set(!is_favorite()),
                        span { class: "text-lg", if is_favorite() { "❤️" } else { "🤍" } }
                    }
                    button {
                        class: "flex items-center space-x-2 px-3 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors",
                        onclick: handle_save,
                        span { class: "text-lg", "💾" }
                        span { class: "text-sm font-medium", "保存" }
                    }
                }
            }

            // 编辑区域
            div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700",
                div { class: "p-6 space-y-4",
                    // 标题输入
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "标题"
                        }
                        input {
                            r#type: "text",
                            value: "{title}",
                            oninput: move |e| title.set(e.value().clone()),
                            placeholder: "请输入笔记标题...",
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400"
                        }
                    }

                    // 心情选择
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "心情"
                        }
                        div { class: "flex flex-wrap gap-2",
                            for mood in Mood::all() {
                                button {
                                    key: "{mood:?}",
                                    class: if selected_mood.read().as_ref() == Some(&mood) {
                                        "px-3 py-1 rounded-full text-sm bg-blue-600 text-white transition-colors"
                                    } else {
                                        "px-3 py-1 rounded-full text-sm bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                                    },
                                    onclick: move |_| handle_mood_select(mood),
                                    "{mood.label()}"
                                }
                            }
                        }
                    }

                    // 标签输入
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "标签"
                        }
                        // 显示现有标签
                        if !tags.read().is_empty() {
                            div { class: "flex flex-wrap gap-2 mb-2",
                                for tag in tags.read().iter() {
                                    span {
                                        key: "{tag}",
                                        class: "inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 dark:bg-blue-900/20 text-blue-800 dark:text-blue-400",
                                        "{tag}"
                                        button {
                                            class: "ml-1 hover:text-blue-600 dark:hover:text-blue-300",
                                            onclick: {
                                                let tag_to_remove = tag.clone();
                                                move |_| handle_remove_tag(tag_to_remove.clone())
                                            },
                                            span { class: "text-xs", "✕" }
                                        }
                                    }
                                }
                            }
                        }
                        input {
                            r#type: "text",
                            value: "{tag_input}",
                            oninput: move |e| tag_input.set(e.value().clone()),
                            placeholder: "输入标签并按回车添加...",
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400",
                            onkeydown: move |e| {
                                if e.key() == Key::Enter {
                                    let tag = tag_input.read().trim().to_string();
                                    if !tag.is_empty() {
                                        handle_add_tag(tag);
                                        tag_input.set(String::new());
                                    }
                                }
                            }
                        }
                    }

                    // 内容编辑
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                            "内容"
                        }
                        textarea {
                            value: "{content}",
                            oninput: move |e| content.set(e.value().clone()),
                            placeholder: "开始写下您的想法...",
                            rows: 12,
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 resize-none"
                        }
                    }
                }
            }
        }
    }
}
