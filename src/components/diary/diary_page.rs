//! # 日记主页面组件
//!
//! 日记模块的主入口，包含标签页导航

use super::{NotesEditor, NotesLibrary, NotesOverview, NotesStats};
use dioxus::prelude::*;

/// 日记页面的子标签
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DiaryTab {
    Overview,
    Editor,
    Library,
    Stats,
}

impl DiaryTab {
    pub fn label(&self) -> &'static str {
        match self {
            DiaryTab::Overview => "概览",
            DiaryTab::Editor => "编辑器",
            DiaryTab::Library => "笔记库",
            DiaryTab::Stats => "统计",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            DiaryTab::Overview => "📚",
            DiaryTab::Editor => "✍️",
            DiaryTab::Library => "📚",
            DiaryTab::Stats => "📊",
        }
    }
}

/// 日记主页面组件
#[component]
pub fn DiaryPage() -> Element {
    // 活动标签状态
    let mut active_tab = use_signal(|| DiaryTab::Overview);

    // 标签定义
    let tabs = [
        DiaryTab::Overview,
        DiaryTab::Editor,
        DiaryTab::Library,
        DiaryTab::Stats,
    ];

    // 渲染活动标签内容
    let render_active_tab = move || match active_tab.read().clone() {
        DiaryTab::Overview => rsx! { NotesOverview {} },
        DiaryTab::Editor => rsx! { NotesEditor {} },
        DiaryTab::Library => rsx! { NotesLibrary {} },
        DiaryTab::Stats => rsx! { NotesStats {} },
    };

    rsx! {
        div { class: "flex flex-col h-full",
            // 标签页导航
            div { class: "flex-shrink-0 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 overflow-x-auto sticky top-0 z-10 pt-2 md:pt-4",
                div { class: "flex px-0 md:px-6",
                    for tab in tabs {
                        div {
                            key: "{tab:?}",
                            class: "relative",
                            button {
                                class: if *active_tab.read() == tab {
                                    "px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap text-blue-600"
                                } else {
                                    "px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                                },
                                onclick: move |_| active_tab.set(tab),
                                span { class: "mr-2", "{tab.icon()}" }
                                "{tab.label()}"
                            }
                            // 选中指示器
                            div { class: if *active_tab.read() == tab {
                                "absolute bottom-0 left-1/2 transform -translate-x-1/2 h-0.5 bg-blue-600 transition-all duration-300 ease-out w-8 opacity-100"
                            } else {
                                "absolute bottom-0 left-1/2 transform -translate-x-1/2 h-0.5 bg-blue-600 transition-all duration-300 ease-out w-0 opacity-0"
                            } }
                        }
                    }
                }
            }

            // 内容区域
            div { class: "flex-1 overflow-y-auto py-4 px-4 md:px-6",
                {render_active_tab()}
            }
        }
    }
}
