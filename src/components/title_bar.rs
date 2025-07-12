//! # 标题栏组件
//!
//! 自定义应用标题栏，包含窗口控制按钮

use dioxus::prelude::*;

/// 标题栏属性
#[derive(Props, Clone, PartialEq)]
pub struct TitleBarProps {
    #[props(default = "LifeTracker".to_string())]
    pub title: String,
    #[props(default = None)]
    pub onminimize: Option<EventHandler<MouseEvent>>,
    #[props(default = None)]
    pub onmaximize: Option<EventHandler<MouseEvent>>,
    #[props(default = None)]
    pub onclose: Option<EventHandler<MouseEvent>>,
}

/// 标题栏组件
#[component]
pub fn TitleBar(props: TitleBarProps) -> Element {
    let mut is_maximized = use_signal(|| false);

    // 检测移动端
    let is_mobile = use_memo(move || {
        // 简单的移动端检测，实际项目中可能需要更复杂的检测逻辑
        cfg!(target_os = "android") || cfg!(target_os = "ios")
    });

    // 最小化窗口
    let handle_minimize = move |e: MouseEvent| {
        if let Some(handler) = &props.onminimize {
            handler.call(e);
        } else {
            // 默认最小化行为
            log::info!("最小化窗口");
        }
    };

    // 最大化/还原窗口
    let handle_maximize = move |e: MouseEvent| {
        if let Some(handler) = &props.onmaximize {
            handler.call(e);
        } else {
            // 默认最大化/还原行为
            is_maximized.set(!is_maximized());
            log::info!(
                "切换窗口大小: {}",
                if is_maximized() {
                    "最大化"
                } else {
                    "还原"
                }
            );
        }
    };

    // 关闭窗口
    let handle_close = move |e: MouseEvent| {
        if let Some(handler) = &props.onclose {
            handler.call(e);
        } else {
            // 默认关闭行为
            log::info!("关闭窗口");
        }
    };

    // 在移动端不显示标题栏
    if is_mobile() {
        return rsx! { div {} };
    }

    rsx! {
        div {
            class: "flex items-center justify-between h-8 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 select-none",
                        // 左侧：应用标题 - 可拖拽区域
            div {
                class: "flex items-center pl-3 flex-1 h-full cursor-move",
                "data-tauri-drag-region": "true",
                span {
                    class: "text-sm font-medium text-gray-700 dark:text-gray-200 truncate",
                    "{props.title}"
                }
            }

            // 右侧：窗口控制按钮 - 非拖拽区域
            div { class: "flex items-center h-full",
                // 最小化按钮
                button {
                    class: "h-full w-12 flex items-center justify-center hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors",
                    title: "最小化",
                    onclick: handle_minimize,
                    span { class: "text-gray-600 dark:text-gray-300", "−" }
                }

                // 最大化/还原按钮
                button {
                    class: "h-full w-12 flex items-center justify-center hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors",
                    title: if is_maximized() { "还原" } else { "最大化" },
                    onclick: handle_maximize,
                    span {
                        class: "text-gray-600 dark:text-gray-300",
                        if is_maximized() { "❐" } else { "▢" }
                    }
                }

                // 关闭按钮
                button {
                    class: "h-full w-12 flex items-center justify-center hover:bg-red-500 hover:text-white transition-colors",
                    title: "关闭",
                    onclick: handle_close,
                    span { class: "text-gray-600 dark:text-gray-300", "✕" }
                }
            }
        }
    }
}
