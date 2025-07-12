//! # 通用组件模块
//!
//! 包含按钮、表单、加载状态等可复用的UI组件

use dioxus::prelude::*;

// ========== 错误处理 ==========

/// 错误信息结构
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorInfo {
    pub message: String,
    pub details: Option<String>,
    pub error_type: ErrorType,
}

/// 错误类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorType {
    Network,
    Database,
    Validation,
    Permission,
    Unknown,
}

impl ErrorType {
    pub fn icon(&self) -> &'static str {
        match self {
            ErrorType::Network => "🌐",
            ErrorType::Database => "🗄️",
            ErrorType::Validation => "⚠️",
            ErrorType::Permission => "🔒",
            ErrorType::Unknown => "❌",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            ErrorType::Network => "text-blue-500",
            ErrorType::Database => "text-purple-500",
            ErrorType::Validation => "text-yellow-500",
            ErrorType::Permission => "text-orange-500",
            ErrorType::Unknown => "text-red-500",
        }
    }
}

/// 错误显示组件属性
#[derive(Props, Clone, PartialEq)]
pub struct ErrorBoundaryProps {
    pub children: Element,
    #[props(default = None)]
    pub error: Option<ErrorInfo>,
    #[props(default = None)]
    pub fallback: Option<Element>,
    #[props(default = None)]
    pub onreset: Option<EventHandler<MouseEvent>>,
    #[props(default = None)]
    pub onreload: Option<EventHandler<MouseEvent>>,
}

/// 错误边界组件
#[component]
pub fn ErrorBoundary(props: ErrorBoundaryProps) -> Element {
    if let Some(error) = &props.error {
        // 如果有自定义fallback，使用它
        if let Some(fallback) = &props.fallback {
            return rsx! { {fallback} };
        }

        // 默认错误UI
        return rsx! {
            div { class: "min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center p-4",
                Card { class: "p-8 max-w-md w-full", shadow: true,
                    div { class: "flex items-center space-x-3 mb-4",
                        span { class: "text-2xl {error.error_type.color()}", "{error.error_type.icon()}" }
                        h2 { class: "text-xl font-semibold text-gray-900 dark:text-white", "出现错误" }
                    }

                    p { class: "text-gray-600 dark:text-gray-300 mb-6", "{error.message}" }

                    if let Some(details) = &error.details {
                        details { class: "mb-6",
                            summary { class: "cursor-pointer text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200",
                                "查看错误详情"
                            }
                            div { class: "mt-2 p-3 bg-gray-100 dark:bg-gray-700 rounded text-xs font-mono text-gray-800 dark:text-gray-200 overflow-auto max-h-32",
                                "{details}"
                            }
                        }
                    }

                    div { class: "flex space-x-3",
                        if let Some(onreset) = props.onreset.as_ref() {
                            {
                                let reset_handler = onreset.clone();
                                rsx! {
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        full_width: true,
                                        icon: "🔄",
                                        onclick: move |e| reset_handler.call(e),
                                        "重试"
                                    }
                                }
                            }
                        }
                        if let Some(onreload) = props.onreload.as_ref() {
                            {
                                let reload_handler = onreload.clone();
                                rsx! {
                                    Button {
                                        variant: ButtonVariant::Secondary,
                                        full_width: true,
                                        onclick: move |e| reload_handler.call(e),
                                        "刷新页面"
                                    }
                                }
                            }
                        }
                    }

                    p { class: "text-xs text-gray-500 dark:text-gray-400 mt-4 text-center",
                        "如果问题持续存在，请尝试重启应用程序"
                    }
                }
            }
        };
    }

    // 没有错误时显示子组件
    rsx! { {props.children} }
}

/// 错误处理Hook - 简化版本
pub fn use_error_handler() -> Signal<Option<ErrorInfo>> {
    use_signal(|| None::<ErrorInfo>)
}

/// 设置错误的辅助函数
pub fn set_error_info(error_signal: &mut Signal<Option<ErrorInfo>>, error: ErrorInfo) {
    log::error!("Error occurred: {:?}", error);
    error_signal.set(Some(error));
}

/// 清除错误的辅助函数
pub fn clear_error_info(error_signal: &mut Signal<Option<ErrorInfo>>) {
    error_signal.set(None);
}

// ========== 按钮组件 ==========

/// 按钮变体
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Success,
    Danger,
    Warning,
    Ghost,
    Outline,
}

/// 按钮尺寸
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

/// 按钮属性
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    pub children: Element,
    #[props(default = ButtonVariant::Primary)]
    pub variant: ButtonVariant,
    #[props(default = ButtonSize::Medium)]
    pub size: ButtonSize,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub loading: bool,
    #[props(default = false)]
    pub full_width: bool,
    #[props(default = String::new())]
    pub class: String,
    #[props(default = String::new())]
    pub title: String,
    #[props(default = None)]
    pub icon: Option<String>,
    #[props(default = None)]
    pub onclick: Option<EventHandler<MouseEvent>>,
}

/// 通用按钮组件
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let base_class = "inline-flex items-center justify-center font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed";

    let variant_class = match props.variant {
        ButtonVariant::Primary => "bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500",
        ButtonVariant::Secondary => "bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600 focus:ring-gray-500",
        ButtonVariant::Success => "bg-green-600 text-white hover:bg-green-700 focus:ring-green-500",
        ButtonVariant::Danger => "bg-red-600 text-white hover:bg-red-700 focus:ring-red-500",
        ButtonVariant::Warning => "bg-yellow-600 text-white hover:bg-yellow-700 focus:ring-yellow-500",
        ButtonVariant::Ghost => "text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 focus:ring-gray-500",
        ButtonVariant::Outline => "border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 focus:ring-gray-500",
    };

    let size_class = match props.size {
        ButtonSize::Small => "px-3 py-1.5 text-sm",
        ButtonSize::Medium => "px-4 py-2 text-sm",
        ButtonSize::Large => "px-6 py-3 text-base",
    };

    let width_class = if props.full_width { "w-full" } else { "" };

    let final_class = format!(
        "{} {} {} {} {}",
        base_class, variant_class, size_class, width_class, props.class
    );

    rsx! {
        button {
            class: "{final_class}",
            disabled: props.disabled || props.loading,
            title: if !props.title.is_empty() { "{props.title}" } else { "" },
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
            },

            // 加载状态
            if props.loading {
                div { class: "animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2" }
            }

            // 图标
            if let Some(icon) = &props.icon {
                span { class: "mr-2", "{icon}" }
            }

            // 内容
            {props.children}
        }
    }
}

// ========== 卡片组件 ==========

/// 卡片属性
#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    pub children: Element,
    #[props(default = false)]
    pub hover: bool,
    #[props(default = false)]
    pub shadow: bool,
    #[props(default = String::new())]
    pub class: String,
    #[props(default = None)]
    pub onclick: Option<EventHandler<MouseEvent>>,
}

/// 通用卡片组件
#[component]
pub fn Card(props: CardProps) -> Element {
    let base_class =
        "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700";
    let hover_class = if props.hover {
        "hover:shadow-md transition-shadow cursor-pointer"
    } else {
        ""
    };
    let shadow_class = if props.shadow { "shadow-lg" } else { "" };

    let final_class = format!(
        "{} {} {} {}",
        base_class, hover_class, shadow_class, props.class
    );

    rsx! {
        div {
            class: "{final_class}",
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
            },
            {props.children}
        }
    }
}

// ========== 输入框组件 ==========

/// 输入框属性
#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    #[props(default = String::new())]
    pub value: String,
    #[props(default = "text".to_string())]
    pub r#type: String,
    #[props(default = String::new())]
    pub placeholder: String,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = String::new())]
    pub class: String,
    #[props(default = String::new())]
    pub label: String,
    #[props(default = String::new())]
    pub error: String,
    #[props(default = String::new())]
    pub help: String,
    #[props(default = None)]
    pub icon: Option<String>,
    #[props(default = None)]
    pub oninput: Option<EventHandler<FormEvent>>,
}

/// 通用输入框组件
#[component]
pub fn Input(props: InputProps) -> Element {
    let base_class = "w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors disabled:opacity-50 disabled:cursor-not-allowed";
    let border_class = if !props.error.is_empty() {
        "border-red-300 dark:border-red-600"
    } else {
        "border-gray-300 dark:border-gray-600"
    };
    let bg_class = "bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400";

    let final_class = format!(
        "{} {} {} {}",
        base_class, border_class, bg_class, props.class
    );

    rsx! {
        div { class: "space-y-1",
            // 标签
            if !props.label.is_empty() {
                label {
                    class: "block text-sm font-medium text-gray-700 dark:text-gray-300",
                    "{props.label}"
                    if props.required {
                        span { class: "text-red-500 ml-1", "*" }
                    }
                }
            }

            // 输入框容器
            div { class: "relative",
                // 图标
                if let Some(icon) = &props.icon {
                    span {
                        class: "absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 dark:text-gray-500",
                        "{icon}"
                    }
                }

                // 输入框
                input {
                    r#type: "{props.r#type}",
                    value: "{props.value}",
                    placeholder: "{props.placeholder}",
                    disabled: props.disabled,
                    required: props.required,
                    class: if props.icon.is_some() { format!("{} pl-10", final_class) } else { final_class },
                    oninput: move |e| {
                        if let Some(handler) = &props.oninput {
                            handler.call(e);
                        }
                    }
                }
            }

            // 错误信息
            if !props.error.is_empty() {
                p { class: "text-sm text-red-600 dark:text-red-400", "{props.error}" }
            }

            // 帮助信息
            if !props.help.is_empty() {
                p { class: "text-sm text-gray-500 dark:text-gray-400", "{props.help}" }
            }
        }
    }
}

// ========== 文本区域组件 ==========

/// 文本区域属性
#[derive(Props, Clone, PartialEq)]
pub struct TextareaProps {
    #[props(default = String::new())]
    pub value: String,
    #[props(default = String::new())]
    pub placeholder: String,
    #[props(default = 3)]
    pub rows: i32,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = String::new())]
    pub class: String,
    #[props(default = String::new())]
    pub label: String,
    #[props(default = String::new())]
    pub error: String,
    #[props(default = String::new())]
    pub help: String,
    #[props(default = None)]
    pub oninput: Option<EventHandler<FormEvent>>,
}

/// 通用文本区域组件
#[component]
pub fn Textarea(props: TextareaProps) -> Element {
    let base_class = "w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors disabled:opacity-50 disabled:cursor-not-allowed resize-y";
    let border_class = if !props.error.is_empty() {
        "border-red-300 dark:border-red-600"
    } else {
        "border-gray-300 dark:border-gray-600"
    };
    let bg_class = "bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400";

    let final_class = format!(
        "{} {} {} {}",
        base_class, border_class, bg_class, props.class
    );

    rsx! {
        div { class: "space-y-1",
            // 标签
            if !props.label.is_empty() {
                label {
                    class: "block text-sm font-medium text-gray-700 dark:text-gray-300",
                    "{props.label}"
                    if props.required {
                        span { class: "text-red-500 ml-1", "*" }
                    }
                }
            }

            // 文本区域
            textarea {
                value: "{props.value}",
                placeholder: "{props.placeholder}",
                rows: props.rows,
                disabled: props.disabled,
                required: props.required,
                class: "{final_class}",
                oninput: move |e| {
                    if let Some(handler) = &props.oninput {
                        handler.call(e);
                    }
                }
            }

            // 错误信息
            if !props.error.is_empty() {
                p { class: "text-sm text-red-600 dark:text-red-400", "{props.error}" }
            }

            // 帮助信息
            if !props.help.is_empty() {
                p { class: "text-sm text-gray-500 dark:text-gray-400", "{props.help}" }
            }
        }
    }
}

// ========== 空状态组件 ==========

/// 空状态属性
#[derive(Props, Clone, PartialEq)]
pub struct EmptyStateProps {
    #[props(default = "📭".to_string())]
    pub icon: String,
    #[props(default = "暂无数据".to_string())]
    pub title: String,
    #[props(default = String::new())]
    pub description: String,
    #[props(default = String::new())]
    pub action_text: String,
    #[props(default = String::new())]
    pub class: String,
    #[props(default = None)]
    pub onaction: Option<EventHandler<MouseEvent>>,
}

/// 通用空状态组件
#[component]
pub fn EmptyState(props: EmptyStateProps) -> Element {
    rsx! {
        div { class: "text-center py-8 {props.class}",
            span { class: "text-4xl block mb-4", "{props.icon}" }
            h3 { class: "text-lg font-medium text-gray-900 dark:text-white mb-2", "{props.title}" }
            if !props.description.is_empty() {
                p { class: "text-gray-600 dark:text-gray-400 mb-4", "{props.description}" }
            }
            if !props.action_text.is_empty() {
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |e| {
                        if let Some(handler) = &props.onaction {
                            handler.call(e);
                        }
                    },
                    icon: "➕",
                    "{props.action_text}"
                }
            }
        }
    }
}

// ========== 加载状态组件 ==========

/// 加载状态属性
#[derive(Props, Clone, PartialEq)]
pub struct LoadingProps {
    #[props(default = "加载中...".to_string())]
    pub text: String,
    #[props(default = ButtonSize::Medium)]
    pub size: ButtonSize,
    #[props(default = String::new())]
    pub class: String,
}

/// 通用加载状态组件
#[component]
pub fn Loading(props: LoadingProps) -> Element {
    let (spinner_size, text_size) = match props.size {
        ButtonSize::Small => ("h-4 w-4", "text-sm"),
        ButtonSize::Medium => ("h-6 w-6", "text-base"),
        ButtonSize::Large => ("h-8 w-8", "text-lg"),
    };

    rsx! {
        div { class: "flex items-center justify-center space-x-2 {props.class}",
            div { class: "animate-spin rounded-full {spinner_size} border-b-2 border-blue-500" }
            span { class: "text-gray-600 dark:text-gray-400 {text_size}", "{props.text}" }
        }
    }
}

// ========== 标签组件 ==========

/// 标签变体
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TagVariant {
    Default,
    Primary,
    Success,
    Warning,
    Danger,
    Info,
}

/// 标签属性
#[derive(Props, Clone, PartialEq)]
pub struct TagProps {
    pub children: Element,
    #[props(default = TagVariant::Default)]
    pub variant: TagVariant,
    #[props(default = ButtonSize::Small)]
    pub size: ButtonSize,
    #[props(default = false)]
    pub removable: bool,
    #[props(default = String::new())]
    pub class: String,
    #[props(default = None)]
    pub onremove: Option<EventHandler<MouseEvent>>,
}

/// 通用标签组件
#[component]
pub fn Tag(props: TagProps) -> Element {
    let base_class = "inline-flex items-center rounded-full font-medium";

    let variant_class = match props.variant {
        TagVariant::Default => "bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200",
        TagVariant::Primary => "bg-blue-100 dark:bg-blue-900/20 text-blue-800 dark:text-blue-400",
        TagVariant::Success => {
            "bg-green-100 dark:bg-green-900/20 text-green-800 dark:text-green-400"
        }
        TagVariant::Warning => {
            "bg-yellow-100 dark:bg-yellow-900/20 text-yellow-800 dark:text-yellow-400"
        }
        TagVariant::Danger => "bg-red-100 dark:bg-red-900/20 text-red-800 dark:text-red-400",
        TagVariant::Info => "bg-cyan-100 dark:bg-cyan-900/20 text-cyan-800 dark:text-cyan-400",
    };

    let size_class = match props.size {
        ButtonSize::Small => "px-2 py-1 text-xs",
        ButtonSize::Medium => "px-3 py-1.5 text-sm",
        ButtonSize::Large => "px-4 py-2 text-base",
    };

    let final_class = format!(
        "{} {} {} {}",
        base_class, variant_class, size_class, props.class
    );

    rsx! {
        span { class: "{final_class}",
            {props.children}
            if props.removable {
                button {
                    class: "ml-1 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-full p-0.5 transition-colors",
                    onclick: move |e| {
                        if let Some(handler) = &props.onremove {
                            handler.call(e);
                        }
                    },
                    span { class: "text-xs", "✕" }
                }
            }
        }
    }
}

// ========== 通知组件 ==========

/// 通知类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationVariant {
    Success,
    Warning,
    Error,
    Info,
}

/// 通知属性
#[derive(Props, Clone, PartialEq)]
pub struct NotificationProps {
    pub children: Element,
    #[props(default = NotificationVariant::Info)]
    pub variant: NotificationVariant,
    #[props(default = true)]
    pub closable: bool,
    #[props(default = String::new())]
    pub class: String,
    #[props(default = None)]
    pub onclose: Option<EventHandler<MouseEvent>>,
}

/// 通用通知组件
#[component]
pub fn Notification(props: NotificationProps) -> Element {
    let base_class = "p-4 rounded-lg border flex items-start space-x-3";

    let (variant_class, icon) = match props.variant {
        NotificationVariant::Success => ("bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800 text-green-800 dark:text-green-400", "✓"),
        NotificationVariant::Warning => ("bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800 text-yellow-800 dark:text-yellow-400", "⚠"),
        NotificationVariant::Error => ("bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800 text-red-800 dark:text-red-400", "✕"),
        NotificationVariant::Info => ("bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800 text-blue-800 dark:text-blue-400", "ℹ"),
    };

    let final_class = format!("{} {} {}", base_class, variant_class, props.class);

    rsx! {
        div { class: "{final_class}",
            span { class: "flex-shrink-0 font-bold", "{icon}" }
            div { class: "flex-1", {props.children} }
            if props.closable {
                button {
                    class: "flex-shrink-0 ml-4 hover:opacity-75 transition-opacity",
                    onclick: move |e| {
                        if let Some(handler) = &props.onclose {
                            handler.call(e);
                        }
                    },
                    span { "✕" }
                }
            }
        }
    }
}

// ========== 模态框组件 ==========

/// 模态框属性
#[derive(Props, Clone, PartialEq)]
pub struct ModalProps {
    pub children: Element,
    #[props(default = true)]
    pub show: bool,
    #[props(default = true)]
    pub closable: bool,
    #[props(default = String::new())]
    pub title: String,
    #[props(default = String::new())]
    pub class: String,
    #[props(default = None)]
    pub onclose: Option<EventHandler<MouseEvent>>,
}

/// 通用模态框组件
#[component]
pub fn Modal(props: ModalProps) -> Element {
    if !props.show {
        return rsx! { div {} };
    }

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center",
            // 背景遮罩
            div {
                class: "absolute inset-0 bg-black bg-opacity-50 transition-opacity",
                onclick: move |e| {
                    if props.closable {
                        if let Some(handler) = &props.onclose {
                            handler.call(e);
                        }
                    }
                }
            }
            // 模态框内容
            div {
                class: "relative bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md mx-4 {props.class}",
                onclick: move |e| e.stop_propagation(),

                // 标题栏
                if !props.title.is_empty() || props.closable {
                    div { class: "flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700",
                        if !props.title.is_empty() {
                            h3 { class: "text-lg font-semibold text-gray-900 dark:text-white", "{props.title}" }
                        }
                        if props.closable {
                            button {
                                class: "text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors",
                                onclick: move |e| {
                                    if let Some(handler) = &props.onclose {
                                        handler.call(e);
                                    }
                                },
                                span { "✕" }
                            }
                        }
                    }
                }

                // 内容
                div { class: "p-4", {props.children} }
            }
        }
    }
}
