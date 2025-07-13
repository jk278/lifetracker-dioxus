//! # LifeTracker Dioxus 应用入口
//!
//! 基于 Dioxus 的跨平台生活追踪应用。
//!
//! 这个文件是 Rust 二进制项目（可执行文件）的入口点。
//! 当你运行 `cargo run` 或 `dx serve` 时，程序会从 `main` 函数开始执行。

// 一个项目可以有多个 main.rs 文件（通过在 Cargo.toml 中配置 [[bin]] 部分），每个文件对应一个可执行程序。

// `use` 关键字用于将其他模块或 crates（Rust 的包）中的项引入当前作用域，
// 这样你就可以直接使用它们，而不需要写完整的路径。

// `dioxus::prelude::*` 引入了 Dioxus 框架中常用的所有宏、类型和函数。
// `prelude` 模块通常包含一个库最常用的部分，方便用户快速上手。
// use dioxus::prelude::*;  // 移除未使用的导入

// `mod components;` 声明并引入了一个名为 `components` 的模块。
// 这意味着在 `src/components/mod.rs` 或 `src/components.rs` 文件中定义了其他组件。
mod components;
mod simple_app;
mod minimal_app;
// `use components::App;` 从 `components` 模块中引入了 `App` 这个项。
// `App` 通常是 Dioxus 应用的根组件。
use components::App;

// `fn main()` 是 Rust 程序的入口点。
// 当程序启动时，首先会执行 `main` 函数。
fn main() {
    // `env_logger` 是一个用于配置日志输出的 crate。
    // `Builder::from_default_env()` 会尝试从环境变量（如 `RUST_LOG`）中读取日志配置。
    // 例如，设置 `RUST_LOG=info` 会让程序输出所有信息级别及以上的日志。
    let mut builder = env_logger::Builder::from_default_env();
    // `.filter_level(log::LevelFilter::Info)` 设置默认的日志过滤级别为 `Info`。
    // 这意味着只有 `Info`、`Warn`、`Error` 级别的日志会被打印出来。
    builder.filter_level(log::LevelFilter::Info);
    // `.init()` 初始化日志系统，使其开始工作。
    builder.init();

    // `log::info!` 是一个宏，用于打印信息级别的日志消息。
    // 这些消息在开发和调试时非常有用，可以追踪程序的执行流程。
    log::info!("LifeTracker Dioxus 应用启动");

    // `dioxus::launch(App)` 是 Dioxus 框架的核心函数之一。
    // 它负责启动 Dioxus 应用，并将 `App` 组件作为应用的根组件进行渲染。
    // `App` 组件现在使用简单的状态管理避免路由系统复杂性，但保留了所有原有功能。
    dioxus::launch(App);
}
