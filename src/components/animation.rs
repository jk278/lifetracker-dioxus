//! # 动画组件库
//!
//! 提供页面过渡、交互动画等功能

use dioxus::prelude::*;
use std::time::Duration;

// ========== 动画基础 ==========

/// 动画方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationDirection {
    Forward,
    Backward,
    None,
}

/// 动画类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationType {
    Slide,
    Fade,
    Scale,
    SlideUp,
    SlideDown,
}

/// 动画状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationState {
    Idle,
    Entering,
    Entered,
    Exiting,
    Exited,
}

// ========== 页面过渡组件 ==========

/// 页面过渡属性
#[derive(Props, Clone, PartialEq)]
pub struct PageTransitionProps {
    pub children: Element,
    pub route_key: String,
    #[props(default = AnimationType::Slide)]
    pub animation_type: AnimationType,
    #[props(default = AnimationDirection::Forward)]
    pub direction: AnimationDirection,
    #[props(default = 300)]
    pub duration: u32,
    #[props(default = false)]
    pub exit_only: bool,
}

/// 页面过渡组件
#[component]
pub fn PageTransition(props: PageTransitionProps) -> Element {
    let mut animation_state = use_signal(|| AnimationState::Idle);
    let mut current_key = use_signal(|| props.route_key.clone());

    // 检测路由变化
    use_effect(move || {
        if current_key.read().clone() != props.route_key {
            // 开始退出动画
            animation_state.set(AnimationState::Exiting);

            // 延迟后切换到新内容
            let route_key_clone = props.route_key.clone();
            spawn(async move {
                gloo::timers::future::sleep(Duration::from_millis(props.duration as u64)).await;
                current_key.set(route_key_clone);

                if !props.exit_only {
                    animation_state.set(AnimationState::Entering);
                    gloo::timers::future::sleep(Duration::from_millis(50)).await;
                    animation_state.set(AnimationState::Entered);
                } else {
                    animation_state.set(AnimationState::Entered);
                }
            });
        }
    });

    // 无动画模式
    if props.direction == AnimationDirection::None {
        return rsx! { div { class: "w-full h-full", {props.children} } };
    }

    // 获取动画CSS类
    let animation_class =
        get_animation_class(props.animation_type, props.direction, animation_state());
    let duration_class = format!("transition-all duration-{}", props.duration);

    rsx! {
        div {
            class: "w-full h-full {animation_class} {duration_class}",
            {props.children}
        }
    }
}

// ========== 视图容器组件 ==========

/// 视图容器属性
#[derive(Props, Clone, PartialEq)]
pub struct ViewContainerProps {
    pub children: Element,
    pub view_key: String,
    #[props(default = "main".to_string())]
    pub view_type: String,
    #[props(default = AnimationType::Slide)]
    pub animation_type: AnimationType,
    #[props(default = AnimationDirection::Forward)]
    pub direction: AnimationDirection,
}

/// 视图容器组件
#[component]
pub fn ViewContainer(props: ViewContainerProps) -> Element {
    let animation_state = use_signal(|| AnimationState::Entered);

    let animation_class =
        get_animation_class(props.animation_type, props.direction, animation_state());
    let duration_class = match props.view_type.as_str() {
        "main" => "transition-all duration-300",
        "system-detail" => "transition-all duration-250",
        _ => "transition-all duration-200",
    };

    rsx! {
        div {
            class: "w-full h-full {animation_class} {duration_class}",
            {props.children}
        }
    }
}

// ========== 交互式按钮组件 ==========

/// 交互式按钮属性
#[derive(Props, Clone, PartialEq)]
pub struct InteractiveButtonProps {
    pub children: Element,
    #[props(default = "primary".to_string())]
    pub variant: String,
    #[props(default = "md".to_string())]
    pub size: String,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = String::new())]
    pub class: String,
    #[props(default = String::new())]
    pub title: String,
    #[props(default = None)]
    pub onclick: Option<EventHandler<MouseEvent>>,
}

/// 交互式按钮组件
#[component]
pub fn InteractiveButton(props: InteractiveButtonProps) -> Element {
    let mut is_pressed = use_signal(|| false);
    let mut is_hovered = use_signal(|| false);

    let variant_class = match props.variant.as_str() {
        "primary" => {
            if props.disabled {
                "bg-gray-300 text-gray-500 cursor-not-allowed"
            } else {
                "bg-blue-500 text-white hover:bg-blue-600 active:bg-blue-700"
            }
        }
        "secondary" => {
            if props.disabled {
                "bg-gray-200 dark:bg-gray-700 text-gray-400 cursor-not-allowed"
            } else {
                "bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600 active:bg-gray-400 dark:active:bg-gray-500"
            }
        }
        "ghost" => {
            if props.disabled {
                "text-gray-400 cursor-not-allowed"
            } else {
                "hover:bg-gray-100 dark:hover:bg-gray-700 active:bg-gray-200 dark:active:bg-gray-600"
            }
        }
        "danger" => {
            if props.disabled {
                "bg-gray-300 text-gray-500 cursor-not-allowed"
            } else {
                "bg-red-500 text-white hover:bg-red-600 active:bg-red-700"
            }
        }
        _ => "bg-gray-200 text-gray-800 hover:bg-gray-300",
    };

    let size_class = match props.size.as_str() {
        "sm" => "px-3 py-1.5 text-sm",
        "md" => "px-4 py-2 text-sm",
        "lg" => "px-6 py-3 text-base",
        _ => "px-4 py-2 text-sm",
    };

    let animation_class = "transition-all duration-200 transform hover:scale-105 active:scale-95";
    let press_class = if is_pressed() { "scale-95" } else { "" };
    let hover_class = if is_hovered() { "scale-105" } else { "" };

    let final_class = format!(
        "font-medium rounded-lg focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 {} {} {} {} {} {}",
        variant_class, size_class, animation_class, press_class, hover_class, props.class
    );

    rsx! {
        button {
            class: "{final_class}",
            disabled: props.disabled,
            title: if !props.title.is_empty() { "{props.title}" } else { "" },
            onmousedown: move |_| is_pressed.set(true),
            onmouseup: move |_| is_pressed.set(false),
            onmouseleave: move |_| { is_pressed.set(false); is_hovered.set(false) },
            onmouseenter: move |_| is_hovered.set(true),
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
            },
            {props.children}
        }
    }
}

// ========== 标签页过渡组件 ==========

/// 标签页过渡属性
#[derive(Props, Clone, PartialEq)]
pub struct TabTransitionProps {
    pub children: Element,
    pub tab_key: String,
    #[props(default = AnimationType::Fade)]
    pub animation_type: AnimationType,
    #[props(default = 200)]
    pub duration: u32,
}

/// 标签页过渡组件
#[component]
pub fn TabTransition(props: TabTransitionProps) -> Element {
    let mut animation_state = use_signal(|| AnimationState::Entered);
    let mut current_key = use_signal(|| props.tab_key.clone());

    // 检测标签变化
    use_effect(move || {
        if current_key.read().clone() != props.tab_key {
            animation_state.set(AnimationState::Exiting);

            let tab_key_clone = props.tab_key.clone();
            spawn(async move {
                gloo::timers::future::sleep(Duration::from_millis(props.duration as u64)).await;
                current_key.set(tab_key_clone);
                animation_state.set(AnimationState::Entering);
                gloo::timers::future::sleep(Duration::from_millis(50)).await;
                animation_state.set(AnimationState::Entered);
            });
        }
    });

    let animation_class = get_animation_class(
        props.animation_type,
        AnimationDirection::Forward,
        animation_state(),
    );
    let duration_class = format!("transition-all duration-{}", props.duration);

    rsx! {
        div {
            class: "w-full h-full {animation_class} {duration_class}",
            {props.children}
        }
    }
}

// ========== 动画列表组件 ==========

/// 动画列表属性
#[derive(Props, Clone, PartialEq)]
pub struct AnimatedListProps {
    pub children: Element,
    #[props(default = 100)]
    pub stagger_delay: u32,
    #[props(default = AnimationType::SlideUp)]
    pub animation_type: AnimationType,
}

/// 动画列表组件
#[component]
pub fn AnimatedList(props: AnimatedListProps) -> Element {
    let mut is_visible = use_signal(|| false);

    // 组件挂载时触发动画
    use_effect(move || {
        spawn(async move {
            gloo::timers::future::sleep(Duration::from_millis(100)).await;
            is_visible.set(true);
        });
    });

    let animation_class = if is_visible() {
        match props.animation_type {
            AnimationType::SlideUp => "translate-y-0 opacity-100",
            AnimationType::SlideDown => "translate-y-0 opacity-100",
            AnimationType::Fade => "opacity-100",
            AnimationType::Scale => "scale-100 opacity-100",
            _ => "opacity-100",
        }
    } else {
        match props.animation_type {
            AnimationType::SlideUp => "translate-y-4 opacity-0",
            AnimationType::SlideDown => "-translate-y-4 opacity-0",
            AnimationType::Fade => "opacity-0",
            AnimationType::Scale => "scale-95 opacity-0",
            _ => "opacity-0",
        }
    };

    rsx! {
        div {
            class: "transition-all duration-500 ease-out {animation_class}",
            {props.children}
        }
    }
}

// ========== 底部抽屉组件 ==========

/// 底部抽屉属性
#[derive(Props, Clone, PartialEq)]
pub struct BottomSheetProps {
    pub children: Element,
    #[props(default = true)]
    pub show: bool,
    #[props(default = None)]
    pub onclose: Option<EventHandler<MouseEvent>>,
    #[props(default = String::new())]
    pub class: String,
}

/// 底部抽屉组件
#[component]
pub fn BottomSheet(props: BottomSheetProps) -> Element {
    if !props.show {
        return rsx! { div {} };
    }

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-end justify-center",
            // 背景遮罩
            div {
                class: "absolute inset-0 bg-black bg-opacity-50 transition-opacity",
                onclick: move |e| {
                    if let Some(handler) = &props.onclose {
                        handler.call(e);
                    }
                }
            }
            // 抽屉内容
            div {
                class: "relative bg-white dark:bg-gray-800 rounded-t-lg shadow-xl w-full max-w-md mx-4 mb-0 transform transition-transform duration-300 ease-out translate-y-0 {props.class}",
                onclick: move |e| e.stop_propagation(),
                {props.children}
            }
        }
    }
}

// ========== 手势包装器组件 ==========

/// 手势包装器属性
#[derive(Props, Clone, PartialEq)]
pub struct GestureWrapperProps {
    pub children: Element,
    #[props(default = None)]
    pub onswipe: Option<EventHandler<String>>,
    #[props(default = None)]
    pub ontap: Option<EventHandler<MouseEvent>>,
    #[props(default = String::new())]
    pub class: String,
}

/// 手势包装器组件
#[component]
pub fn GestureWrapper(props: GestureWrapperProps) -> Element {
    let mut touch_start = use_signal(|| (0.0, 0.0));
    let mut is_touching = use_signal(|| false);

    rsx! {
        div {
            class: "touch-manipulation {props.class}",
            ontouchstart: move |e| {
                if let Some(_touch) = e.touches().first() {
                    // Note: TouchPoint API may differ in different Dioxus versions
                    // Using placeholder coordinates for now
                    touch_start.set((0.0, 0.0)); // TODO: 实现正确的触摸坐标获取
                    is_touching.set(true);
                }
            },
            ontouchend: move |e| {
                if is_touching() {
                    if let Some(_touch) = e.touches().first() {
                        let (start_x, start_y) = touch_start();
                        // TODO: 实现正确的触摸坐标获取
                        let end_x = 0.0;
                        let end_y = 0.0;

                        let dx: f64 = end_x - start_x;
                        let dy: f64 = end_y - start_y;

                        let distance = (dx * dx + dy * dy).sqrt();

                        if distance > 50.0 {
                            let direction = if dx.abs() > dy.abs() {
                                if dx > 0.0 { "right" } else { "left" }
                            } else {
                                if dy > 0.0 { "down" } else { "up" }
                            };

                            if let Some(handler) = &props.onswipe {
                                handler.call(direction.to_string());
                            }
                        }
                    }
                    is_touching.set(false);
                }
            },
            onclick: move |e| {
                if let Some(handler) = &props.ontap {
                    handler.call(e);
                }
            },
            {props.children}
        }
    }
}

// ========== 辅助函数 ==========

/// 获取动画CSS类
fn get_animation_class(
    animation_type: AnimationType,
    direction: AnimationDirection,
    state: AnimationState,
) -> String {
    match (animation_type, direction, state) {
        (AnimationType::Slide, AnimationDirection::Forward, AnimationState::Entering) => {
            "translate-x-0 opacity-100".to_string()
        }
        (AnimationType::Slide, AnimationDirection::Forward, AnimationState::Exiting) => {
            "translate-x-full opacity-0".to_string()
        }
        (AnimationType::Slide, AnimationDirection::Backward, AnimationState::Entering) => {
            "translate-x-0 opacity-100".to_string()
        }
        (AnimationType::Slide, AnimationDirection::Backward, AnimationState::Exiting) => {
            "-translate-x-full opacity-0".to_string()
        }

        (AnimationType::Fade, _, AnimationState::Entering) => "opacity-100".to_string(),
        (AnimationType::Fade, _, AnimationState::Exiting) => "opacity-0".to_string(),

        (AnimationType::Scale, _, AnimationState::Entering) => "scale-100 opacity-100".to_string(),
        (AnimationType::Scale, _, AnimationState::Exiting) => "scale-95 opacity-0".to_string(),

        (AnimationType::SlideUp, _, AnimationState::Entering) => {
            "translate-y-0 opacity-100".to_string()
        }
        (AnimationType::SlideUp, _, AnimationState::Exiting) => {
            "translate-y-4 opacity-0".to_string()
        }

        (AnimationType::SlideDown, _, AnimationState::Entering) => {
            "translate-y-0 opacity-100".to_string()
        }
        (AnimationType::SlideDown, _, AnimationState::Exiting) => {
            "-translate-y-4 opacity-0".to_string()
        }

        (_, _, AnimationState::Entered) => "opacity-100".to_string(),
        (_, _, AnimationState::Idle) => "opacity-100".to_string(),
        (_, _, AnimationState::Exited) => "opacity-0".to_string(),
        (AnimationType::Slide, AnimationDirection::None, AnimationState::Entering) => {
            "opacity-100".to_string()
        }
        (AnimationType::Slide, AnimationDirection::None, AnimationState::Exiting) => {
            "opacity-0".to_string()
        }
    }
}
