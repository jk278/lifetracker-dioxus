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
    
    // 系统主题变化监听（仅在System模式下）
    use_effect(move || {
        let mut theme_state = theme_state.clone();
        spawn(async move {
            loop {
                // 每5秒检查一次系统主题变化
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                
                let mut state = theme_state.write();
                if state.mode == ThemeMode::System {
                    let old_css = state.css_class;
                    state.refresh_system_theme();
                    
                    // 如果主题发生变化，记录日志
                    if state.css_class != old_css {
                        log::info!("系统主题变化检测: {} -> {}", 
                            if old_css == "dark" { "dark" } else { "light" },
                            if state.css_class == "dark" { "dark" } else { "light" }
                        );
                    }
                }
            }
        });
    });
    
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
            log::info!("主题已切换到: {:?}", new_theme);
        } else {
            log::error!("主题切换失败");
        }
    }
}

/// 设置特定主题模式的钩子
pub fn use_theme_setter() -> impl FnMut(ThemeMode) {
    let mut theme_state = use_theme_state();
    
    move |new_theme: ThemeMode| {
        if let Err(e) = life_tracker::set_theme_mode(new_theme) {
            log::error!("设置主题模式失败: {}", e);
            return;
        }
        
        theme_state.write().update(new_theme);
        log::info!("主题已设置为: {:?}", new_theme);
    }
}

/// 主题状态钩子 - 获取当前主题状态和切换函数
pub fn use_theme() -> (Signal<ThemeState>, impl FnMut()) {
    let theme_state = use_theme_state();
    let toggle = use_theme_toggle();
    (theme_state, toggle)
}