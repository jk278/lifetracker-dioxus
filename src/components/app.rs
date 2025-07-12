//! # 主应用组件
//!
//! 定义应用的路由和基础布局

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use life_tracker::{get_app_state_sync, initialize_app_sync, AppState};

/// 应用路由定义
#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Dashboard {},
    #[route("/tasks")]
    TaskManagement {},
    #[route("/categories")]
    CategoryManagement {},
    #[route("/statistics")]
    Statistics {},
    #[route("/financial")]
    Financial {},
    #[route("/diary")]
    Diary {},
    #[route("/habits")]
    Habits {},
    #[route("/settings")]
    Settings {},
}

/// 主应用组件
#[component]
pub fn App() -> Element {
    // 使用 use_signal 进行同步初始化，避免 runtime 嵌套
    let app_state = use_signal(|| {
        log::info!("开始同步初始化应用...");
        match initialize_app_sync() {
            Ok(_) => {
                log::info!("应用初始化成功");
                get_app_state_sync()
            }
            Err(e) => {
                log::error!("应用初始化失败: {}", e);
                AppState::default()
            }
        }
    });

    // 根据初始化状态渲染不同内容
    if !app_state.read().initialized {
        return rsx! {
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

    // 应用已成功初始化，显示主界面
    rsx! {
        Router::<Route> {}
    }
}

// 路由组件
#[component]
fn Dashboard() -> Element {
    rsx! {
        super::dashboard::Dashboard {}
    }
}

#[component]
fn TaskManagement() -> Element {
    rsx! {
        super::timing::TimingPage {}
    }
}

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

#[component]
fn Financial() -> Element {
    rsx! {
        super::accounting::AccountingPage {}
    }
}

#[component]
fn Diary() -> Element {
    rsx! {
        super::diary::DiaryPage {}
    }
}

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

#[component]
fn Settings() -> Element {
    rsx! {
        div {
            class: "p-8",
            h1 { class: "text-2xl font-bold", "设置" }
            p { "设置组件正在开发中..." }
        }
    }
}
