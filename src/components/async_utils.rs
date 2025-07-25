//! # 异步工具模块
//!
//! 基于Dioxus 0.6最佳实践的统一异步处理工具

use dioxus::prelude::*;
use life_tracker::{AppError, Database};
use std::sync::Arc;
use super::app_state_provider::use_database;

/// 异步操作状态
#[derive(Clone, Debug, PartialEq)]
pub enum AsyncState<T> {
    /// 加载中
    Loading,
    /// 加载成功
    Success(T),
    /// 加载失败
    Error(String),
}

impl<T> AsyncState<T> {
    /// 检查是否正在加载
    pub fn is_loading(&self) -> bool {
        matches!(self, AsyncState::Loading)
    }
    
    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        matches!(self, AsyncState::Success(_))
    }
    
    /// 检查是否出错
    pub fn is_error(&self) -> bool {
        matches!(self, AsyncState::Error(_))
    }
    
    /// 获取成功的值
    pub fn get_data(&self) -> Option<&T> {
        match self {
            AsyncState::Success(data) => Some(data),
            _ => None,
        }
    }
    
    /// 获取错误信息
    pub fn get_error(&self) -> Option<&str> {
        match self {
            AsyncState::Error(err) => Some(err),
            _ => None,
        }
    }
}

/// 数据库操作钩子 - 统一的数据库异步操作
/// 
/// 使用use_resource实现，提供自动重试、取消安全等特性
pub fn use_database_query<T, F, Fut>(
    operation: F,
    _deps: impl PartialEq + Clone + 'static,
) -> Resource<Result<T, String>>
where
    T: Clone + 'static,
    F: Fn(Arc<Database>) -> Fut + Clone + 'static,
    Fut: std::future::Future<Output = Result<T, AppError>> + 'static,
{
    // 在钩子层面获取数据库连接
    let database = use_database();
    
    use_resource(move || {
        let operation = operation.clone();
        let database = database.clone();
        async move {
            // 检查数据库连接
            let db = match database {
                Some(db) => db,
                None => return Err("数据库连接不可用".to_string()),
            };
            
            // 执行操作
            operation(db).await.map_err(|e| e.to_string())
        }
    })
}

/// 简化的异步数据加载钩子
/// 
/// 适用于不需要数据库的异步操作
pub fn use_async_data<T, F, Fut>(
    operation: F,
    _deps: impl PartialEq + Clone + 'static,
) -> Resource<T>
where
    T: Clone + 'static,
    F: Fn() -> Fut + 'static,
    Fut: std::future::Future<Output = T> + 'static,
{
    use_resource(move || operation())
}

/// 可取消的异步操作钩子
///
/// 提供取消功能的异步操作，适用于用户可能中断的长时间操作
pub fn use_cancellable_async<T, F, Fut>(
    operation: F,
    _deps: impl PartialEq + Clone + 'static,
) -> (Resource<T>, impl FnMut())
where
    T: Clone + 'static,
    F: Fn() -> Fut + 'static,
    Fut: std::future::Future<Output = T> + 'static,
{
    let resource = use_resource(move || operation());
    let cancel = {
        let mut resource_clone = resource.clone();
        move || {
            resource_clone.cancel();
        }
    };
    
    (resource, cancel)
}

/// 异步表单提交钩子
///
/// 专门用于表单提交等需要防重复提交的场景
pub fn use_async_submit<T, F, Fut>(
    submit_fn: F,
) -> (Signal<AsyncState<T>>, impl Fn())
where
    T: Clone + 'static,
    F: Fn() -> Fut + Clone + 'static,
    Fut: std::future::Future<Output = Result<T, AppError>> + 'static,
{
    let state = use_signal(|| AsyncState::Loading);
    
    let submit = {
        let mut state = state.clone();
        let submit_fn = submit_fn.clone();
        
        move || {
            let mut state = state.clone();
            let submit_fn = submit_fn.clone();
            
            spawn(async move {
                // 防止重复提交
                if state.read().is_loading() {
                    return;
                }
                
                state.set(AsyncState::Loading);
                
                match submit_fn().await {
                    Ok(result) => {
                        state.set(AsyncState::Success(result));
                    }
                    Err(error) => {
                        state.set(AsyncState::Error(error.to_string()));
                    }
                }
            });
        }
    };
    
    (state, submit)
}

/// 分页数据加载钩子
///
/// 专门用于需要分页加载的数据
pub fn use_paginated_data<T, F, Fut>(
    fetch_page: F,
    initial_page: usize,
    page_size: usize,
) -> (Resource<Vec<T>>, Signal<usize>, impl FnMut(), impl FnMut())
where
    T: Clone + 'static,
    F: Fn(usize, usize) -> Fut + Clone + 'static,
    Fut: std::future::Future<Output = Result<Vec<T>, AppError>> + 'static,
{
    let current_page = use_signal(|| initial_page);
    let data = use_resource(move || {
        let fetch_page = fetch_page.clone();
        let page = current_page();
        async move {
            fetch_page(page, page_size).await.unwrap_or_default()
        }
    });
    
    let next_page = {
        let mut current_page = current_page.clone();
        move || {
            current_page.set(current_page() + 1);
        }
    };
    
    let prev_page = {
        let mut current_page = current_page.clone();
        move || {
            if current_page() > 0 {
                current_page.set(current_page() - 1);
            }
        }
    };
    
    (data, current_page, next_page, prev_page)
}

/// 实时数据订阅钩子
///
/// 用于需要定期刷新的数据
pub fn use_live_data<T, F, Fut>(
    fetch_data: F,
    refresh_interval_ms: u64,
) -> Resource<T>
where
    T: Clone + 'static,
    F: Fn() -> Fut + Clone + 'static,
    Fut: std::future::Future<Output = T> + 'static,
{
    let refresh_signal = use_signal(|| 0);
    
    // 定期刷新
    use_effect(move || {
        let mut refresh_signal = refresh_signal.clone();
        spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(refresh_interval_ms)).await;
                refresh_signal.set(refresh_signal() + 1);
            }
        });
    });
    
    use_resource(move || {
        let _refresh_trigger = refresh_signal(); // 依赖刷新信号
        fetch_data()
    })
}

/// 错误边界友好的异步组件包装器
#[component]
pub fn AsyncBoundary<T: Clone + PartialEq + 'static>(
    data: Resource<Result<T, String>>,
    loading: Element,
    error_render: fn(String) -> Element,
    success_render: fn(T) -> Element,
) -> Element {
    match &*data.read_unchecked() {
        Some(Ok(value)) => rsx! { {success_render(value.clone())} },
        Some(Err(err)) => rsx! { {error_render(err.clone())} },
        None => rsx! { {loading} },
    }
}

/// 通用加载组件
#[component]
pub fn LoadingSpinner(message: Option<String>) -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center p-8",
            div {
                class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mb-4"
            }
            if let Some(msg) = message {
                p {
                    class: "text-gray-600 dark:text-gray-400 text-sm",
                    "{msg}"
                }
            }
        }
    }
}

/// 通用错误显示组件
#[component]
pub fn ErrorDisplay(message: String, retry: Option<fn()>) -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center p-8 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-800",
            div {
                class: "text-red-500 mb-4 text-2xl",
                "⚠️"
            }
            p {
                class: "text-red-700 dark:text-red-300 text-center mb-4",
                "{message}"
            }
            if let Some(retry_fn) = retry {
                button {
                    class: "px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 transition-colors",
                    onclick: move |_| retry_fn(),
                    "重试"
                }
            }
        }
    }
}