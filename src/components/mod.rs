//! # 组件模块
//!
//! 包含所有 UI 组件的定义和组织。这个文件作为 `src/components` 目录的根模块，
//! 负责声明和导出所有子模块中定义的组件，使得其他模块可以方便地导入和使用这些 UI 元素。
//! 这种模块化设计有助于保持代码的组织性、可维护性和可重用性。

// mod.rs 是一个 模块（module）的根文件。当你在一个目录中组织子模块时，mod.rs 文件充当该目录的模块声明入口。
// 它声明了该目录下的所有子模块，并导出这些模块的公共接口，使得其他模块可以方便地导入和使用这些 UI 元素。
// 在 Rust 2018 Edition 之后，mod.rs 的作用逐渐被简化。
// 现在，你可以在父模块文件中直接声明子模块，例如在 src/main.rs 中直接 mod my_module;，然后 my_module 的内容会放在 src/my_module.rs 或 src/my_module/mod.rs。
// 不过，mod.rs 仍然是一种常见的组织方式，尤其是在需要将一个目录作为一个模块来处理时。

// 模块（module）是 Rust 中组织代码的一种方式，类似于其他编程语言中的命名空间或包。
// 模块允许你将相关的代码组织在一起，并提供命名空间来避免命名冲突。
// 模块可以包含函数、结构体、枚举、常量、类型别名、模块等。
// 模块可以嵌套，形成层级结构。
// 模块可以被其他模块引用，形成依赖关系。

// 重新导出（pub use）其子模块中的公共项，以便外部更容易访问。
// 声明 `about` 模块。该模块可能包含应用的“关于”页面或相关信息。
pub mod about;
// 声明 `accounting` 模块，并使用 `#[path = "Accounting/mod.rs"]` 指定其文件路径。
// 这表明 `accounting` 模块的定义位于 `Accounting` 目录下，而不是默认的 `accounting.rs` 或 `accounting/`。
// 该模块负责财务管理相关的 UI 组件和逻辑。
#[path = "Accounting/mod.rs"]
pub mod accounting; // 现在是模块化的 accounting/ 目录
                    // 声明 `animation` 模块，包含各种 UI 动画组件。
pub mod animation;
// 声明 `app` 模块，通常包含主应用组件 (`App`)，是整个应用的根组件和路由入口。
pub mod app;
// 声明 `app_state_provider` 模块，提供统一的全局状态管理。
pub mod app_state_provider;
// 声明 `async_utils` 模块，提供统一的异步处理工具。
pub mod async_utils;
// 声明 `common` 模块，包含通用的、可复用的 UI 组件，如按钮、输入框、卡片等。
pub mod common;
// 声明 `dashboard` 模块，包含仪表盘页面或其子组件。
pub mod dashboard;
// 声明 `data_management` 模块，包含数据管理相关的 UI 组件，如数据导入导出、清除等。
pub mod data_management;
// 声明 `diary` 模块，并使用 `#[path = "diary/mod.rs"]` 指定其文件路径。
// 该模块负责日记/笔记功能相关的 UI 组件和逻辑。
pub mod diary; // 现在是模块化的 diary/ 目录
               // 声明 `habits` 模块，包含习惯打卡功能相关的 UI 组件和逻辑。
pub mod habits;
// 声明 `settings` 模块，包含应用设置相关的 UI 组件。
pub mod settings;
// 声明 `system_page` 模块，可能包含系统信息或管理页面。
pub mod system_page;
// 声明 `timing` 模块，并使用 `#[path = "Timing/mod.rs"]` 指定其文件路径。
// 该模块负责时间追踪功能相关的 UI 组件和逻辑。
#[path = "Timing/mod.rs"]
pub mod timing; // 现在是模块化的 timing/ 目录
                // 声明 `title_bar` 模块，包含应用标题栏组件。
pub mod title_bar;
// 声明 `theme_provider` 模块，提供响应式主题管理系统。
pub mod theme_provider;

// 重新导出主要组件
// `pub use` 语句的目的是简化其他模块导入组件的方式。
// 例如，如果其他文件想使用 `AboutPage`，他们可以直接 `use crate::components::AboutPage;`，
// 而不需要写 `use crate::components::about::AboutPage;`。

/// 重新导出 `about` 模块中的 `AboutPage` 组件。
pub use about::AboutPage;
/// 重新导出 `app` 模块中的 `App` 组件，作为应用的根组件。
pub use app::App;
// pub use dashboard::Dashboard; // 暂时注释未使用的导入
/// 重新导出 `data_management` 模块中的 `DataManagementPage` 组件。
pub use data_management::DataManagementPage;
/// 重新导出 `settings` 模块中的 `SettingsPage` 组件。
pub use settings::SettingsPage;
// pub use system_page::SystemPage; // 暂时注释未使用的导入
// pub use title_bar::TitleBar; // 暂时注释未使用的导入

// 重新导出 timing 模块的主要组件 - 暂时注释未使用的
// pub use timing::{
//     CategoryManagement,    // 分类管理组件
//     StatisticsPlaceholder, // 统计功能占位符组件
//     TaskManagementContent, // 任务管理内容显示组件
//     TimingDashboard,       // 时间追踪仪表盘组件
//     TimingPage,            // 时间追踪模块主页面组件
// };

// 重新导出 accounting 模块的主要组件 - 暂时注释未使用的
// pub use accounting::AccountingPage;

// 重新导出 diary 模块的主要组件 - 暂时注释未使用的
// pub use diary::DiaryPage;

// 重新导出通用组件 - 暂时注释大部分未使用的
// pub use common::{
//     clear_error_info,        // 清除错误信息的函数
//     set_error_info,          // 设置错误信息的函数
//     use_error_handler,       // 用于错误处理的 Hook
//     Button,                  // 通用按钮组件
//     ButtonSize,              // 按钮大小的枚举类型
//     ButtonVariant,           // 按钮变体（样式）的枚举类型
//     Card,                    // 卡片组件，用于内容分组和展示
//     EmptyState,              // 空状态组件，在没有数据时显示
//     ErrorBoundary,           // 错误边界组件，用于捕获子组件的渲染错误
//     ErrorInfo,               // 错误信息的结构体
//     ErrorType,               // 错误类型的枚举
//     Input,                   // 输入框组件
//     Loading,                 // 加载状态指示组件
//     Modal,                   // 模态框组件
//     Notification,            // 通知组件
//     NotificationVariant,     // 通知变体（样式）的枚举类型
//     Tag,                     // 标签组件
//     TagVariant,              // 标签变体（样式）的枚举类型
//     Textarea,                // 多行文本输入框组件
// };

// 重新导出动画组件 - 暂时注释未使用的
// pub use animation::{
//     AnimatedList,        // 列表动画组件
//     AnimationDirection,  // 动画方向的枚举类型
//     AnimationType,       // 动画类型的枚举类型
//     BottomSheet,         // 底部抽屉组件
//     GestureWrapper,      // 手势封装组件
//     InteractiveButton,   // 交互式按钮组件
//     PageTransition,      // 页面过渡动画组件
//     TabTransition,       // 标签页过渡动画组件
//     ViewContainer,       // 视图容器组件
// };
