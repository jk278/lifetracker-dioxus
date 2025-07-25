//! # 主题提供者组件
//!
//! 使用Dioxus 0.6的响应式状态管理实现主题管理，替代低效的轮询机制

use dioxus::prelude::*;
use life_tracker::{get_theme_mode, ThemeMode};

/// 计算主题CSS类名
fn calculate_theme_class(theme: ThemeMode) -> &'static str {
    match theme {
        ThemeMode::Dark => "dark",
        ThemeMode::Light | ThemeMode::System => {
            // 对于System模式，需要检测实际效果
            if theme.is_dark() { "dark" } else { "" }
        }
    }
}

/// 全局主题状态管理
#[derive(Clone, Default)]
pub struct ThemeState {
    pub mode: ThemeMode,
    pub css_class: &'static str,
}

impl ThemeState {
    pub fn new() -> Self {
        let mode = get_theme_mode();
        Self {
            mode,
            css_class: calculate_theme_class(mode),
        }
    }
    
    pub fn update(&mut self, new_mode: ThemeMode) {
        self.mode = new_mode;
        self.css_class = calculate_theme_class(new_mode);
    }
}

/// 主题提供者组件 - 包装应用根组件
#[component]
pub fn ThemeProvider(children: Element) -> Element {
    // 初始化主题状态
    let theme_state = use_signal(|| ThemeState::new());
    
    // 提供主题状态到上下文
    use_context_provider(|| theme_state);
    
    // 响应式主题类名
    let css_class = theme_state.read().css_class;
    
    rsx! {
        div { 
            class: format!("theme-root {}", css_class),
            {children}
        }
    }
}

/// 获取主题状态钩子
pub fn use_theme_state() -> Signal<ThemeState> {
    use_context::<Signal<ThemeState>>()
}

/// 获取主题CSS类名钩子 - 兼容性API
pub fn get_theme_class_signal() -> Memo<&'static str> {
    let theme_state = use_theme_state();
    use_memo(move || theme_state.read().css_class)
}

/// 主题切换钩子 - 提供主题切换功能
pub fn use_theme_toggle() -> impl FnMut() {
    let mut theme_state = use_theme_state();
    
    move || {
        if let Ok(new_theme) = life_tracker::toggle_theme() {
            theme_state.write().update(new_theme);
        }
    }
}

/// 主题状态钩子 - 获取当前主题状态和切换函数
pub fn use_theme() -> (Signal<ThemeState>, impl FnMut()) {
    let theme_state = use_theme_state();
    let toggle = use_theme_toggle();
    (theme_state, toggle)
}