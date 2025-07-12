//! # 关于页面
//!
//! 显示应用信息、功能特性、系统信息、许可证等内容

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct AboutPageProps {
    pub show_back_button: bool,
    pub on_back: Option<EventHandler<()>>,
}

// 应用信息结构
#[derive(Debug, Clone, PartialEq)]
struct AppInfo {
    name: String,
    version: String,
    description: String,
    author: String,
    email: String,
    website: String,
    repository: String,
    license: String,
    build_date: String,
    build_target: String,
}

// 功能特性结构
#[derive(Debug, Clone, PartialEq)]
struct Feature {
    icon: String,
    title: String,
    desc: String,
}

// 系统信息结构
#[derive(Debug, Clone, PartialEq)]
struct SystemInfo {
    os: String,
    user_agent: String,
    language: String,
    memory_usage: String,
    screen_resolution: String,
}

// 致谢项结构
#[derive(Debug, Clone, PartialEq)]
struct Acknowledgment {
    name: String,
    desc: String,
    url: String,
}

#[component]
pub fn AboutPage(props: AboutPageProps) -> Element {
    // 显示控制状态
    let mut show_details = use_signal(|| false);
    let mut show_system_info = use_signal(|| false);
    let mut show_license = use_signal(|| false);

    // 应用信息
    let app_info = use_memo(|| AppInfo {
        name: "LifeTracker".to_string(),
        version: "0.1.0".to_string(),
        description: "综合性的生活追踪和管理工具".to_string(),
        author: "LifeTracker Team".to_string(),
        email: "contact@lifetracker.dev".to_string(),
        website: "https://lifetracker.dev".to_string(),
        repository: "https://github.com/lifetracker/lifetracker".to_string(),
        license: "MIT".to_string(),
        build_date: "2024-01-15".to_string(),
        build_target: "Windows x64".to_string(),
    });

    // 功能特性
    let features = use_memo(|| {
        vec![
            Feature {
                icon: "⏱️".to_string(),
                title: "精确的时间跟踪".to_string(),
                desc: "记录每个任务的开始和结束时间".to_string(),
            },
            Feature {
                icon: "📊".to_string(),
                title: "详细的统计分析".to_string(),
                desc: "提供多维度的时间使用分析".to_string(),
            },
            Feature {
                icon: "🏷️".to_string(),
                title: "灵活的分类管理".to_string(),
                desc: "支持自定义分类和标签".to_string(),
            },
            Feature {
                icon: "📈".to_string(),
                title: "趋势分析".to_string(),
                desc: "分析工作模式和效率趋势".to_string(),
            },
            Feature {
                icon: "🔔".to_string(),
                title: "智能提醒".to_string(),
                desc: "休息提醒和目标达成通知".to_string(),
            },
            Feature {
                icon: "💾".to_string(),
                title: "数据备份".to_string(),
                desc: "支持数据导出和备份恢复".to_string(),
            },
            Feature {
                icon: "🎨".to_string(),
                title: "主题定制".to_string(),
                desc: "多种主题和界面定制选项".to_string(),
            },
            Feature {
                icon: "⌨️".to_string(),
                title: "快捷键支持".to_string(),
                desc: "提高操作效率的快捷键".to_string(),
            },
        ]
    });

    // 系统信息
    let system_info = use_memo(|| SystemInfo {
        os: "Windows 10".to_string(), // 实际应该通过系统API获取
        user_agent: "Dioxus/0.6".to_string(),
        language: "zh-CN".to_string(),
        memory_usage: "约 80MB".to_string(),
        screen_resolution: "1920x1080".to_string(), // 实际应该通过API获取
    });

    // 致谢信息
    let acknowledgments = use_memo(|| {
        vec![
            Acknowledgment {
                name: "Rust".to_string(),
                desc: "系统编程语言".to_string(),
                url: "https://rust-lang.org".to_string(),
            },
            Acknowledgment {
                name: "Dioxus".to_string(),
                desc: "跨平台UI框架".to_string(),
                url: "https://dioxuslabs.com".to_string(),
            },
            Acknowledgment {
                name: "Tailwind CSS".to_string(),
                desc: "实用优先的CSS框架".to_string(),
                url: "https://tailwindcss.com".to_string(),
            },
            Acknowledgment {
                name: "SQLite".to_string(),
                desc: "嵌入式数据库".to_string(),
                url: "https://sqlite.org".to_string(),
            },
            Acknowledgment {
                name: "tokio".to_string(),
                desc: "异步运行时".to_string(),
                url: "https://tokio.rs".to_string(),
            },
            Acknowledgment {
                name: "serde".to_string(),
                desc: "序列化框架".to_string(),
                url: "https://serde.rs".to_string(),
            },
        ]
    });

    rsx! {
        div { class: "h-full flex flex-col",

            // 固定顶部导航栏
            div { class: "flex-shrink-0 px-4 md:px-6 py-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800",
                div { class: "flex items-center justify-between",
                    div { class: "flex items-center space-x-3",
                        // 返回按钮
                        if props.show_back_button {
                            button {
                                class: "flex items-center justify-center w-8 h-8 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors",
                                onclick: move |_| {
                                    if let Some(handler) = &props.on_back {
                                        handler.call(());
                                    }
                                },
                                title: "返回",
                                "←"
                            }
                        }
                        h2 { class: "text-2xl font-bold text-gray-900 dark:text-white",
                            "关于"
                        }
                    }
                }
            }

            // 可滚动内容区域
            div { class: "flex-1 overflow-y-auto py-4 px-4 md:px-6",
                div { class: "max-w-4xl mx-auto space-y-8",

                    // 页面介绍
                    div { class: "text-center",
                        p { class: "text-gray-600 dark:text-gray-400",
                            "了解更多关于我们的应用程序"
                        }
                    }

                    // 应用图标和基本信息
                    div { class: "text-center space-y-6",
                        // 应用图标
                        div { class: "flex justify-center",
                            div { class: "w-20 h-20 bg-blue-600 rounded-full flex items-center justify-center",
                                span { class: "text-3xl text-white", "⏱️" }
                            }
                        }

                        // 应用名称和版本
                        div {
                            h1 { class: "text-4xl font-bold text-blue-600 dark:text-blue-400 mb-2",
                                "{app_info.read().name}"
                            }
                            p { class: "text-lg text-gray-500 dark:text-gray-400",
                                "版本 {app_info.read().version}"
                            }
                            p { class: "text-gray-600 dark:text-gray-300 mt-2",
                                "{app_info.read().description}"
                            }
                        }
                    }

                    // 控制选项
                    div { class: "flex justify-center space-x-6",
                        label { class: "flex items-center space-x-2",
                            input {
                                r#type: "checkbox",
                                checked: *show_details.read(),
                                onchange: move |e| show_details.set(e.value() == "true"),
                                class: "rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                            }
                            span { class: "text-sm text-gray-600 dark:text-gray-400",
                                "显示详细信息"
                            }
                        }
                        label { class: "flex items-center space-x-2",
                            input {
                                r#type: "checkbox",
                                checked: *show_system_info.read(),
                                onchange: move |e| show_system_info.set(e.value() == "true"),
                                class: "rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                            }
                            span { class: "text-sm text-gray-600 dark:text-gray-400",
                                "显示系统信息"
                            }
                        }
                        label { class: "flex items-center space-x-2",
                            input {
                                r#type: "checkbox",
                                checked: *show_license.read(),
                                onchange: move |e| show_license.set(e.value() == "true"),
                                class: "rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                            }
                            span { class: "text-sm text-gray-600 dark:text-gray-400",
                                "显示许可证"
                            }
                        }
                    }

                    // 基本信息
                    div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg p-6",
                        h3 { class: "text-xl font-semibold text-gray-900 dark:text-white mb-4",
                            "基本信息"
                        }
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                            div { class: "space-y-2",
                                div { class: "flex justify-between",
                                    span { class: "text-gray-600 dark:text-gray-400", "开发者:" }
                                    span { class: "text-gray-900 dark:text-white", "{app_info.read().author}" }
                                }
                                div { class: "flex justify-between",
                                    span { class: "text-gray-600 dark:text-gray-400", "许可证:" }
                                    span { class: "text-gray-900 dark:text-white", "{app_info.read().license}" }
                                }
                                div { class: "flex justify-between",
                                    span { class: "text-gray-600 dark:text-gray-400", "构建日期:" }
                                    span { class: "text-gray-900 dark:text-white", "{app_info.read().build_date}" }
                                }
                            }
                            div { class: "space-y-2",
                                div { class: "flex justify-between",
                                    span { class: "text-gray-600 dark:text-gray-400", "构建目标:" }
                                    span { class: "text-gray-900 dark:text-white", "{app_info.read().build_target}" }
                                }
                                div { class: "flex justify-between",
                                    span { class: "text-gray-600 dark:text-gray-400", "框架:" }
                                    span { class: "text-gray-900 dark:text-white", "Dioxus + Rust" }
                                }
                                div { class: "flex justify-between",
                                    span { class: "text-gray-600 dark:text-gray-400", "状态:" }
                                    span { class: "text-green-600 dark:text-green-400", "运行中" }
                                }
                            }
                        }
                    }

                    // 相关链接
                    div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg p-6",
                        h3 { class: "text-xl font-semibold text-gray-900 dark:text-white mb-4",
                            "相关链接"
                        }
                        div { class: "flex flex-wrap gap-4",
                            a {
                                href: "{app_info.read().website}",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "flex items-center space-x-2 px-4 py-2 bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 rounded-lg hover:bg-blue-100 dark:hover:bg-blue-900/30 transition-colors",
                                span { class: "text-lg", "🌐" }
                                span { "官方网站" }
                            }
                            a {
                                href: "mailto:{app_info.read().email}",
                                class: "flex items-center space-x-2 px-4 py-2 bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400 rounded-lg hover:bg-green-100 dark:hover:bg-green-900/30 transition-colors",
                                span { class: "text-lg", "📧" }
                                span { "联系我们" }
                            }
                            a {
                                href: "{app_info.read().repository}",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "flex items-center space-x-2 px-4 py-2 bg-gray-50 dark:bg-gray-700 text-gray-600 dark:text-gray-400 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors",
                                span { class: "text-lg", "🔗" }
                                span { "源代码" }
                            }
                        }
                    }

                    // 主要功能
                    div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg p-6",
                        h3 { class: "text-xl font-semibold text-gray-900 dark:text-white mb-4",
                            "主要功能"
                        }
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                            for feature in features.read().iter() {
                                div { class: "flex items-start space-x-3",
                                    span { class: "text-2xl", "{feature.icon}" }
                                    div {
                                        h4 { class: "font-medium text-gray-900 dark:text-white",
                                            "{feature.title}"
                                        }
                                        p { class: "text-sm text-gray-600 dark:text-gray-400",
                                            "{feature.desc}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 系统信息
                    if *show_system_info.read() {
                        div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg p-6",
                            h3 { class: "text-xl font-semibold text-gray-900 dark:text-white mb-4",
                                "系统信息"
                            }
                            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                                div { class: "space-y-2",
                                    div { class: "flex items-center justify-between",
                                        span { class: "text-gray-600 dark:text-gray-400", "操作系统:" }
                                        span { class: "text-gray-900 dark:text-white", "{system_info.read().os}" }
                                    }
                                    div { class: "flex items-center justify-between",
                                        span { class: "text-gray-600 dark:text-gray-400", "语言:" }
                                        span { class: "text-gray-900 dark:text-white", "{system_info.read().language}" }
                                    }
                                    div { class: "flex items-center justify-between",
                                        span { class: "text-gray-600 dark:text-gray-400", "内存使用:" }
                                        span { class: "text-gray-900 dark:text-white", "{system_info.read().memory_usage}" }
                                    }
                                }
                                div { class: "space-y-2",
                                    div { class: "flex items-center justify-between",
                                        span { class: "text-gray-600 dark:text-gray-400", "屏幕分辨率:" }
                                        span { class: "text-gray-900 dark:text-white", "{system_info.read().screen_resolution}" }
                                    }
                                }
                            }
                        }
                    }

                    // 许可证信息
                    if *show_license.read() {
                        div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg p-6",
                            h3 { class: "text-xl font-semibold text-gray-900 dark:text-white mb-4",
                                "许可证信息"
                            }
                            div { class: "bg-gray-50 dark:bg-gray-700 rounded-lg p-4",
                                h4 { class: "font-medium text-gray-900 dark:text-white mb-2",
                                    "MIT License"
                                }
                                p { class: "text-sm text-gray-600 dark:text-gray-300 leading-relaxed",
                                    "Copyright (c) 2024 LifeTracker Team"
                                }
                                br {}
                                p { class: "text-sm text-gray-600 dark:text-gray-300 leading-relaxed",
                                    "Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the \"Software\"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:"
                                }
                                br {}
                                p { class: "text-sm text-gray-600 dark:text-gray-300 leading-relaxed",
                                    "The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software."
                                }
                                br {}
                                p { class: "text-sm text-gray-600 dark:text-gray-300 leading-relaxed",
                                    "THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT."
                                }
                            }
                        }
                    }

                    // 版本历史
                    if *show_details.read() {
                        div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg p-6",
                            h3 { class: "text-xl font-semibold text-gray-900 dark:text-white mb-4",
                                "版本历史"
                            }
                            div { class: "space-y-3",
                                for (version, date, desc) in [
                                    ("v1.0.0", "2024-01-15", "首个正式版本发布"),
                                    ("v0.9.0", "2024-01-01", "添加统计分析功能"),
                                    ("v0.8.0", "2023-12-15", "实现Dioxus界面"),
                                    ("v0.7.0", "2023-12-01", "添加数据库支持"),
                                    ("v0.6.0", "2023-11-15", "实现核心时间跟踪功能"),
                                ] {
                                    div { class: "flex items-center space-x-4",
                                        span { class: "font-medium text-blue-600 dark:text-blue-400 w-16", "{version}" }
                                        span { class: "text-gray-500 dark:text-gray-400 w-20", "{date}" }
                                        span { class: "text-gray-900 dark:text-white", "{desc}" }
                                    }
                                }
                            }
                        }
                    }

                    // 致谢
                    if *show_details.read() {
                        div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg p-6",
                            h3 { class: "text-xl font-semibold text-gray-900 dark:text-white mb-4 flex items-center space-x-2",
                                span { class: "text-red-500", "❤️" }
                                span { "致谢" }
                            }
                            p { class: "text-gray-600 dark:text-gray-400 mb-4",
                                "感谢以下开源项目和贡献者:"
                            }
                            div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
                                for acknowledgment in acknowledgments.read().iter() {
                                    div { class: "flex items-center space-x-3",
                                        span { class: "w-2 h-2 bg-blue-600 rounded-full" }
                                        div {
                                            span { class: "font-medium text-gray-900 dark:text-white",
                                                "{acknowledgment.name}"
                                            }
                                            span { class: "text-gray-600 dark:text-gray-400",
                                                " - {acknowledgment.desc}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 底部版权信息
                    div { class: "text-center py-6",
                        p { class: "text-sm text-gray-500 dark:text-gray-400",
                            "© 2024 LifeTracker Team. All rights reserved."
                        }
                    }
                }
            }
        }
    }
}
