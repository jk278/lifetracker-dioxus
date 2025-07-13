//! # 主应用组件
//!
//! 定义应用的路由和基础布局，是整个 Dioxus 应用的入口点。
//! 该文件负责设置全局应用状态、路由导航以及不同页面组件的渲染。

// 导入 Dioxus 框架的核心库，提供了构建 UI 组件所需的所有宏和函数，如 `rsx!` 宏和 `#[component]` 宏等。
use dioxus::prelude::*;
// 导入 Dioxus 路由库，用于在 Dioxus 应用中实现客户端路由功能，允许根据 URL 路径渲染不同的组件。
use dioxus_router::prelude::*;
// 从当前工作空间的 `life_tracker` crate 中导入 `get_app_state_sync`、`initialize_app_sync` 函数和 `AppState` 结构体。
// `initialize_app_sync` 用于同步初始化应用的核心数据和状态。
// `get_app_state_sync` 用于获取应用初始化后的全局状态。
// `AppState` 结构体定义了整个应用的状态数据模型。
use life_tracker::{get_app_state_sync, initialize_app_sync, AppState};

/// 应用路由定义
///
/// `#[derive(Clone, Routable, Debug, PartialEq)]` 宏是 Dioxus 路由的关键。
/// - `Clone`: 允许路由枚举的实例被复制。
/// - `Routable`: 这是 Dioxus 路由系统提供的 proc-macro，它会自动为 `Route` 枚举生成实现路由所需的所有代码，
///   例如将 URL 路径映射到对应的枚举变体，并生成用于导航的函数。
/// - `Debug`: 允许使用 `{:?}` 格式化输出路由枚举的实例，方便调试。
/// - `PartialEq`: 允许比较路由枚举的两个实例是否相等。
///
/// 每个 `#[route("/path")]` 属性定义了一个 URL 路径和与之对应的 Dioxus 组件。
/// 当应用的 URL 匹配到某个路径时，Dioxus 路由器会自动渲染该路径对应的组件。
#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    /// 根路径 "/" 对应的仪表盘组件。
    #[route("/")]
    Dashboard {},
    /// "/tasks" 路径对应的任务管理组件。
    #[route("/tasks")]
    TaskManagement {},
    /// "/categories" 路径对应的分类管理组件。
    #[route("/categories")]
    CategoryManagement {},
    /// "/statistics" 路径对应的统计报告组件。
    #[route("/statistics")]
    Statistics {},
    /// "/financial" 路径对应的财务管理组件。
    #[route("/financial")]
    Financial {},
    /// "/diary" 路径对应的日记组件。
    #[route("/diary")]
    Diary {},
    /// "/habits" 路径对应的习惯打卡组件。
    #[route("/habits")]
    Habits {},
    /// "/settings" 路径对应的设置组件。
    #[route("/settings")]
    Settings {},
}

/// 主应用组件
///
/// `#[component]` 宏将一个 Rust 函数标记为一个 Dioxus 组件。
/// Dioxus 组件是构建用户界面的基本单元，它们接收属性（props）并返回一个 `Element`。
/// `Element` 是 Dioxus 对 UI 元素的抽象，最终会被渲染成实际的 DOM 结构。
#[component]
pub fn App() -> Element {
    // 使用 `use_signal` 进行同步初始化，`use_signal` 是 Dioxus 提供的一个 Hook，用于创建可变的响应式状态。
    // 这里的 `app_state` 是一个 Signal，它包裹了 `AppState` 类型的值。
    // 当 `app_state` 的值发生变化时，所有依赖于它的组件都会重新渲染。
    // 闭包 `|| { ... }` 会在组件首次渲染时执行一次，用于初始化 `app_state` 的值。
    let app_state = use_signal(|| {
        log::info!("开始同步初始化应用...");
        // 调用 `initialize_app_sync()` 进行应用初始化。这是一个同步函数，意味着它会阻塞当前线程直到初始化完成。
        match initialize_app_sync() {
            // 如果初始化成功，则记录成功日志，并调用 `get_app_state_sync()` 获取应用状态。
            Ok(_) => {
                log::info!("应用初始化成功");
                get_app_state_sync()
            }
            // 如果初始化失败，则记录错误日志，并返回一个默认的 `AppState`，其中 `initialized` 字段为 `false`。
            Err(e) => {
                log::error!("应用初始化失败: {}", e);
                AppState::default()
            }
        }
    });

    // 根据应用初始化状态渲染不同的内容。
    // `app_state.read().initialized` 会读取 `app_state` Signal 内部的值，并访问其 `initialized` 字段。
    // 如果 `initialized` 为 `false`，表示应用初始化失败，将显示一个错误页面。
    if !app_state.read().initialized {
        return rsx! {
            // `rsx!` 宏是 Dioxus 的 JSX-like 语法扩展，用于方便地定义 UI 结构。
            // 它允许你以类似 HTML/XML 的方式编写组件的渲染逻辑。
            div {
                class: "min-h-screen bg-gray-100 dark:bg-gray-900 flex items-center justify-center",
                div {
                    class: "text-center",
                    div {
                        class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"
                    }
                    h2 {
                        class: "text-xl font-semibold text-gray-700 dark:text-gray-300",
                        "应用初始化失败"
                    }
                    p {
                        class: "text-gray-500 dark:text-gray-400",
                        "请检查日志获取详细信息"
                    }
                }
            }
        };
    }

    // 应用已成功初始化，显示主界面。
    // `Router::<Route> {}` 组件是 Dioxus 路由的入口。
    // 它会根据当前的 URL 路径，查找 `Route` 枚举中匹配的路由条目，并渲染对应的组件。
    rsx! {
        Router::<Route> {}
    }
}

// 路由组件定义
// 以下每个函数都对应 `Route` 枚举中的一个变体，并作为该路由路径下渲染的实际组件。

/// 仪表盘组件。
/// 当 URL 匹配到 "/" 时，此组件会被渲染。
#[component]
fn Dashboard() -> Element {
    rsx! {
        // `super::dashboard::Dashboard {}` 表示从当前模块的父模块 (`src/components/`)
        // 导入 `dashboard` 模块中的 `Dashboard` 组件并渲染。
        super::dashboard::Dashboard {}
    }
}

/// 任务管理组件。
/// 当 URL 匹配到 "/tasks" 时，此组件会被渲染。
#[component]
fn TaskManagement() -> Element {
    rsx! {
        // 渲染 `super::timing::TimingPage` 组件。
        // `TimingPage` 可能是时间追踪模块的主页面，其中包含了任务管理相关的逻辑。
        super::timing::TimingPage {}
    }
}

/// 分类管理组件。
/// 当 URL 匹配到 "/categories" 时，此组件会被渲染。
#[component]
fn CategoryManagement() -> Element {
    rsx! {
        div {
            class: "p-8",
            h1 { class: "text-2xl font-bold", "分类管理" }
            p { "分类管理组件正在开发中..." }
        }
    }
}

/// 统计报告组件。
/// 当 URL 匹配到 "/statistics" 时，此组件会被渲染。
#[component]
fn Statistics() -> Element {
    rsx! {
        div {
            class: "p-8",
            h1 { class: "text-2xl font-bold", "统计报告" }
            p { "统计报告组件正在开发中..." }
        }
    }
}

/// 财务管理组件。
/// 当 URL 匹配到 "/financial" 时，此组件会被渲染。
#[component]
fn Financial() -> Element {
    rsx! {
        // 渲染 `super::accounting::AccountingPage` 组件。
        super::accounting::AccountingPage {}
    }
}

/// 日记组件。
/// 当 URL 匹配到 "/diary" 时，此组件会被渲染。
#[component]
fn Diary() -> Element {
    rsx! {
        // 渲染 `super::diary::DiaryPage` 组件。
        super::diary::DiaryPage {}
    }
}

/// 习惯打卡组件。
/// 当 URL 匹配到 "/habits" 时，此组件会被渲染。
#[component]
fn Habits() -> Element {
    rsx! {
        div {
            class: "p-8",
            h1 { class: "text-2xl font-bold", "习惯打卡" }
            p { "习惯打卡组件正在开发中..." }
        }
    }
}

/// 设置组件。
/// 当 URL 匹配到 "/settings" 时，此组件会被渲染。
#[component]
fn Settings() -> Element {
    // `use_navigator` 是 Dioxus 路由提供的一个 Hook，用于获取 `Navigator` 实例。
    // `Navigator` 提供了编程导航的功能，例如 `go_back()`（返回上一页）、`push()`（导航到新路径）等。
    let navigator = use_navigator();

    rsx! {
        // 渲染 `super::settings::SettingsPage` 组件。
        // `show_back_button: true` 是传递给 `SettingsPage` 组件的属性（props），
        // 告诉 `SettingsPage` 组件显示一个返回按钮。
        // `on_back` 也是一个属性，它是一个回调函数。当 `SettingsPage` 内部的返回按钮被点击时，
        // 会触发这个回调，然后调用 `navigator.go_back()` 实现返回上一页的功能。
        super::settings::SettingsPage {
            show_back_button: true,
            on_back: move |_| {
                navigator.go_back();
            },
        }
    }
}
