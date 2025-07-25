//! # 应用状态提供者
//!
//! 统一的全局状态管理，使用Dioxus Context API替代重复的get_app_state_sync调用

use dioxus::prelude::*;
use life_tracker::{get_app_state_sync, initialize_app_sync, AppState, Database};
use std::sync::Arc;

/// 应用上下文状态
#[derive(Clone)]
pub struct AppContext {
    /// 应用状态信号
    pub app_state: Signal<AppState>,
    /// 初始化状态
    pub initialized: bool,
}

impl AppContext {
    /// 创建新的应用上下文
    pub fn new() -> Self {
        log::info!("初始化应用上下文...");
        
        // 同步初始化应用
        let (app_state, initialized) = match initialize_app_sync() {
            Ok(_) => {
                log::info!("应用初始化成功");
                let state = get_app_state_sync();
                (state.clone(), state.initialized)
            }
            Err(e) => {
                log::error!("应用初始化失败: {}", e);
                (AppState::default(), false)
            }
        };
        
        Self {
            app_state: Signal::new(app_state),
            initialized,
        }
    }
    
    /// 获取数据库连接的便捷方法
    pub fn get_database(&self) -> Option<Arc<Database>> {
        self.app_state.read().get_database()
    }
    
    /// 更新应用状态
    pub fn update_state(&mut self, new_state: AppState) {
        self.app_state.set(new_state.clone());
        self.initialized = new_state.initialized;
    }
}

/// 应用状态提供者组件
#[component]
pub fn AppStateProvider(children: Element) -> Element {
    // 初始化应用上下文
    let app_context = use_signal(|| AppContext::new());
    
    // 提供上下文到子组件
    use_context_provider(|| app_context);
    
    // 检查初始化状态
    let context = app_context.read();
    if !context.initialized {
        return rsx! {
            div {
                class: "min-h-screen bg-gray-100 dark:bg-gray-900 flex items-center justify-center",
                div {
                    class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8 text-center",
                    div {
                        class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"
                    }
                    h2 {
                        class: "text-xl font-semibold text-gray-900 dark:text-white mb-2",
                        "应用初始化中..."
                    }
                    p {
                        class: "text-gray-500 dark:text-gray-400",
                        "请稍候，正在加载应用组件"
                    }
                }
            }
        };
    }
    
    rsx! { {children} }
}

/// 获取应用上下文钩子
pub fn use_app_context() -> Signal<AppContext> {
    use_context::<Signal<AppContext>>()
}

/// 获取数据库连接钩子
pub fn use_database() -> Option<Arc<Database>> {
    let context = use_app_context();
    let ctx_read = context.read();
    ctx_read.get_database()
}

/// 获取应用状态钩子
pub fn use_app_state() -> Signal<AppState> {
    let context = use_app_context();
    let ctx_read = context.read();
    ctx_read.app_state
}

/// 异步操作封装 - 统一错误处理模式
pub async fn with_database<T, F, Fut>(operation: F) -> Result<T, String>
where
    F: Fn(Arc<Database>) -> Fut,
    Fut: std::future::Future<Output = Result<T, life_tracker::AppError>>,
{
    // 这个函数需要在组件外部调用，所以不能使用use_database钩子
    // 保留原有的get_app_state_sync作为fallback
    let app_state = get_app_state_sync();
    if let Some(database) = app_state.get_database() {
        operation(database).await.map_err(|e| e.to_string())
    } else {
        Err("数据库连接不可用".to_string())
    }
}

/// 同步数据库操作封装
pub fn with_database_sync<T, F>(operation: F) -> Result<T, String>
where
    F: Fn(&Database) -> Result<T, life_tracker::AppError>,
{
    let app_state = get_app_state_sync();
    if let Some(database) = app_state.get_database() {
        operation(&database).map_err(|e| e.to_string())
    } else {
        Err("数据库连接不可用".to_string())
    }
}

/// 数据库操作钩子 - 在组件中使用
pub fn use_database_operation() -> impl Fn(&dyn Fn(&Database) -> Result<(), life_tracker::AppError>) -> Result<(), String> {
    let context = use_app_context();
    
    move |operation| {
        let ctx = context.read();
        if let Some(database) = ctx.get_database() {
            operation(&*database).map_err(|e| e.to_string())
        } else {
            Err("数据库连接不可用".to_string())
        }
    }
}