//! # LifeTracker Dioxus 应用入口
//!
//! 基于 Dioxus 的跨平台生活追踪应用

use dioxus::prelude::*;

mod components;
use components::App;

fn main() {
    // 初始化日志系统
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("LifeTracker Dioxus 应用启动");

    // 启动 Dioxus 桌面应用
    dioxus::launch(App);
}
