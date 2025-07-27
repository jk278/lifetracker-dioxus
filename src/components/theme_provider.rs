//! # 主题提供者组件
//!
//! 使用Dioxus 0.6的响应式状态管理实现主题管理，替代低效的轮询机制

use dioxus::prelude::*;
use life_tracker::{get_theme_mode, ThemeMode};
use life_tracker::config::theme::ThemeConfig;

/// 计算主题CSS类名
fn calculate_theme_class(theme: ThemeMode) -> &'static str {
    match theme {
        ThemeMode::Dark => "dark",
        ThemeMode::Light => "",
        ThemeMode::System => {
            // 对于System模式，调用系统检测逻辑
            if theme.is_dark() { "dark" } else { "" }
        }
    }
}

/// 全局主题状态管理
#[derive(Clone, Default)]
pub struct ThemeState {
    pub mode: ThemeMode,
    pub css_class: &'static str,
    pub system_theme: Option<String>, // 缓存检测到的系统主题
}

impl ThemeState {
    pub fn new() -> Self {
        let mode = get_theme_mode();
        let system_theme = if mode == ThemeMode::System {
            Some(ThemeConfig::detect_system_theme())
        } else {
            None
        };
        
        Self {
            mode,
            css_class: Self::calculate_effective_class(mode, &system_theme),
            system_theme,
        }
    }
    
    pub fn update(&mut self, new_mode: ThemeMode) {
        self.mode = new_mode;
        
        // 如果切换到System模式，重新检测系统主题
        if new_mode == ThemeMode::System {
            self.system_theme = Some(ThemeConfig::detect_system_theme());
        } else {
            self.system_theme = None;
        }
        
        self.css_class = Self::calculate_effective_class(new_mode, &self.system_theme);
    }
    
    /// 计算有效的CSS类名（考虑系统主题检测结果）
    fn calculate_effective_class(mode: ThemeMode, system_theme: &Option<String>) -> &'static str {
        match mode {
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "",
            ThemeMode::System => {
                match system_theme.as_deref() {
                    Some("dark") => "dark",
                    _ => "",
                }
            }
        }
    }
    
    /// 刷新系统主题检测（用于实时更新）
    pub fn refresh_system_theme(&mut self) {
        if self.mode == ThemeMode::System {
            let new_system_theme = ThemeConfig::detect_system_theme();
            if self.system_theme.as_deref() != Some(&new_system_theme) {
                self.system_theme = Some(new_system_theme);
                self.css_class = Self::calculate_effective_class(self.mode, &self.system_theme);
            }
        }
    }
}

/// 主题提供者组件 - 包装应用根组件
#[component]
pub fn ThemeProvider(children: Element) -> Element {
    // 初始化主题状态
    let theme_state = use_signal(|| ThemeState::new());
    
    // 提供主题状态到上下文
    use_context_provider(|| theme_state);
    
    // 应用主题到document根元素（Tailwind dark mode需要）
    use_effect(move || {
        let css_class = theme_state.read().css_class;
        apply_theme_to_document(css_class);
    });
    
    // 系统主题变化监听 - 移除轮询机制，只在主题切换时检查
    // 注意：Dioxus目前没有跨平台的系统主题变化事件监听，建议用户手动刷新或重启应用
    
    // 响应式主题类名
    let css_class = theme_state.read().css_class;
    
    rsx! {
        div { 
            // 应用主题类到根容器，确保Tailwind dark:前缀生效
            class: format!("theme-root min-h-screen {}", css_class),
            {children}
        }
    }
}

/// 应用主题到document根元素
/// Tailwind CSS dark mode 需要在html根元素上有dark类名
fn apply_theme_to_document(css_class: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        
        // 使用web-sys直接操作DOM
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(html) = document.document_element() {
                    // 移除现有主题类
                    let _ = html.class_list().remove_1("dark");
                    let _ = html.class_list().remove_1("light");
                    
                    // 添加新主题类（如果不为空）
                    if !css_class.is_empty() {
                        let _ = html.class_list().add_1(css_class);
                        
                        // 设置 color-scheme 属性以支持系统组件样式
                        if css_class == "dark" {
                            let _ = html.set_attribute("style", "color-scheme: dark");
                        } else {
                            let _ = html.set_attribute("style", "color-scheme: light");
                        }
                        
                        log::info!("✅ Applied theme class '{}' to document.documentElement", css_class);
                        
                        // 触发自定义事件，通知页面主题已更改
                        if let Ok(event) = document.create_event("CustomEvent") {
                            let _ = event.init_custom_event_with_can_bubble_and_cancelable(
                                "themeChanged", 
                                true, 
                                false
                            );
                            let _ = html.dispatch_event(&event);
                        }
                    } else {
                        let _ = html.remove_attribute("style");
                        log::info!("✅ Theme class cleared");
                    }
                }
                
                // 同时在 body 上设置，以防某些样式需要
                if let Some(body) = document.body() {
                    let _ = body.class_list().remove_1("dark");
                    let _ = body.class_list().remove_1("light");
                    
                    if !css_class.is_empty() {
                        let _ = body.class_list().add_1(css_class);
                    }
                }
            }
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // 桌面应用版本 - 这里可以通过webview的API来操作
        log::info!("Desktop mode: applying theme {}", css_class);
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
            log::info!("Theme switched to: {:?}", new_theme);
        } else {
            log::error!("Theme switch failed");
        }
    }
}

/// 设置特定主题模式的钩子
pub fn use_theme_setter() -> impl FnMut(ThemeMode) {
    let mut theme_state = use_theme_state();
    
    move |new_theme: ThemeMode| {
        if let Err(e) = life_tracker::set_theme_mode(new_theme) {
            log::error!("Failed to set theme mode: {}", e);
            return;
        }
        
        theme_state.write().update(new_theme);
        let css_class = theme_state.read().css_class;
        apply_theme_to_document(css_class);
        log::info!("Theme set to: {:?}", new_theme);
    }
}

/// 主题状态钩子 - 获取当前主题状态和切换函数
pub fn use_theme() -> (Signal<ThemeState>, impl FnMut()) {
    let theme_state = use_theme_state();
    let toggle = use_theme_toggle();
    (theme_state, toggle)
}