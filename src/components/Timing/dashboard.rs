//! # 时间追踪仪表板组件
//!
//! 包含计时器、统计卡片、快速操作等功能

use dioxus::prelude::*;
use life_tracker::get_app_state_sync;
use life_tracker::storage::task_models::TaskModel;

/// 计时器状态
#[derive(Clone, Debug, PartialEq)]
pub enum TimerState {
    Stopped,
    Running {
        task_id: String,
        start_time: chrono::DateTime<chrono::Local>,
    },
    Paused {
        task_id: String,
        elapsed_seconds: u64,
    },
}

impl TimerState {
    pub fn is_running(&self) -> bool {
        matches!(self, TimerState::Running { .. })
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, TimerState::Paused { .. })
    }

    pub fn get_task_id(&self) -> Option<&str> {
        match self {
            TimerState::Running { task_id, .. } | TimerState::Paused { task_id, .. } => {
                Some(task_id)
            }
            TimerState::Stopped => None,
        }
    }
}

/// 时间追踪仪表板组件
#[component]
pub fn TimingDashboard() -> Element {
    let timer_state = use_signal(|| TimerState::Stopped);
    let selected_task_id = use_signal(|| None::<String>);

    // 获取任务列表用于显示
    let app_state = get_app_state_sync();
    let tasks = if let Some(database) = app_state.get_database() {
        match database.get_all_tasks() {
            Ok(tasks) => tasks,
            Err(e) => {
                log::error!("获取任务失败: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    rsx! {
        div {
            class: "max-w-6xl mx-auto space-y-8",

            // 页面标题
            div {
                class: "text-center",
                h1 { class: "text-3xl font-bold text-gray-900 dark:text-white mb-2", "时间追踪仪表板" }
                p { class: "text-gray-600 dark:text-gray-400", "管理你的时间，提高工作效率" }
            }

            // 计时器区域
            TimerWidget {
                timer_state: timer_state.read().clone(),
                tasks: tasks.clone(),
                selected_task_id: selected_task_id.read().clone()
            }

            // 快速统计卡片
            div {
                class: "grid grid-cols-1 md:grid-cols-3 gap-6",

                StatCard {
                    icon: "⏱️",
                    title: "今日时长",
                    value: "2小时45分",
                    color: "blue"
                }

                StatCard {
                    icon: "📋",
                    title: "任务数量",
                    value: format!("{}个", tasks.len()),
                    color: "green"
                }

                StatCard {
                    icon: "📈",
                    title: "本周目标",
                    value: "78%",
                    color: "purple"
                }
            }

            // 快速操作区域
            QuickActions {}
        }
    }
}

/// 计时器组件
#[component]
fn TimerWidget(
    timer_state: TimerState,
    tasks: Vec<TaskModel>,
    selected_task_id: Option<String>,
) -> Element {
    let selected_task = selected_task_id
        .as_ref()
        .and_then(|id| tasks.iter().find(|t| t.id.to_string() == *id));

    // 格式化时间显示
    let format_duration = |seconds: u64| -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    };

    // 计算当前经过的时间
    let elapsed_seconds = match &timer_state {
        TimerState::Running { start_time, .. } => {
            let now = chrono::Local::now();
            (now - *start_time).num_seconds().max(0) as u64
        }
        TimerState::Paused {
            elapsed_seconds, ..
        } => *elapsed_seconds,
        TimerState::Stopped => 0,
    };

    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-2xl shadow-lg border border-gray-200 dark:border-gray-700 p-8",

            div {
                class: "text-center space-y-6",

                // 时间显示
                div {
                    class: "space-y-2",
                    div {
                        class: "text-6xl font-mono font-bold text-gray-900 dark:text-white tracking-wider",
                        "{format_duration(elapsed_seconds)}"
                    }
                    div {
                        class: format!("text-lg font-medium {}",
                            match timer_state {
                                TimerState::Running { .. } => "text-green-600 dark:text-green-400",
                                TimerState::Paused { .. } => "text-yellow-600 dark:text-yellow-400",
                                TimerState::Stopped => "text-gray-500 dark:text-gray-400",
                            }
                        ),
                        match timer_state {
                            TimerState::Running { .. } => "● 运行中",
                            TimerState::Paused { .. } => "⏸ 已暂停",
                            TimerState::Stopped => "⏹ 未开始",
                        }
                    }
                }

                // 任务选择器
                TaskSelector {
                    tasks: tasks.clone(),
                    selected_task: selected_task.cloned()
                }

                // 控制按钮
                TimerControls { timer_state: timer_state.clone() }
            }
        }
    }
}

/// 任务选择器组件
#[component]
fn TaskSelector(tasks: Vec<TaskModel>, selected_task: Option<TaskModel>) -> Element {
    let mut show_selector = use_signal(|| false);

    rsx! {
        div {
            class: "space-y-3",

            // 当前选中的任务显示
            button {
                class: "w-full max-w-md mx-auto p-4 bg-gray-50 dark:bg-gray-700 rounded-xl border-2 border-dashed border-gray-300 dark:border-gray-600 hover:border-blue-400 dark:hover:border-blue-500 transition-colors",
                onclick: move |_| show_selector.set(true),

                div {
                    class: "text-sm text-gray-500 dark:text-gray-400 mb-1",
                    "当前任务"
                }
                div {
                    class: "text-lg font-medium text-gray-900 dark:text-white",
                    if let Some(task) = &selected_task {
                        "{task.name}"
                    } else {
                        span { class: "text-gray-400", "点击选择任务" }
                    }
                }
            }

            // 任务选择器下拉菜单
            if show_selector() {
                div {
                    class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                    onclick: move |_| show_selector.set(false),

                    div {
                        class: "bg-white dark:bg-gray-800 rounded-xl p-6 max-w-md w-full mx-4 max-h-96 overflow-y-auto",
                        onclick: move |e| e.stop_propagation(),

                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-white mb-4",
                            "选择任务"
                        }

                        div {
                            class: "space-y-2",
                            if tasks.is_empty() {
                                div {
                                    class: "text-center py-8 text-gray-500 dark:text-gray-400",
                                    "暂无任务，请先创建任务"
                                }
                            } else {
                                for task in tasks.iter() {
                                    button {
                                        key: "{task.id}",
                                        class: "w-full text-left p-3 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors",
                                        onclick: move |_| {
                                            // TODO: 设置选中的任务
                                            show_selector.set(false);
                                        },

                                        div {
                                            class: "font-medium text-gray-900 dark:text-white",
                                            "{task.name}"
                                        }
                                        if let Some(description) = &task.description {
                                            div {
                                                class: "text-sm text-gray-500 dark:text-gray-400 mt-1",
                                                "{description}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 计时器控制按钮组件
#[component]
fn TimerControls(timer_state: TimerState) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-center space-x-4",

            match timer_state {
                TimerState::Stopped => rsx! {
                    button {
                        class: "flex items-center justify-center w-20 h-20 bg-green-600 hover:bg-green-700 text-white rounded-full shadow-lg transition-all transform hover:scale-105 active:scale-95",
                        onclick: move |_| {
                            // TODO: 开始计时器
                            log::info!("开始计时器");
                        },
                        span { class: "text-2xl", "▶️" }
                    }
                },
                TimerState::Running { .. } => rsx! {
                    button {
                        class: "flex items-center justify-center w-20 h-20 bg-yellow-600 hover:bg-yellow-700 text-white rounded-full shadow-lg transition-all transform hover:scale-105 active:scale-95",
                        onclick: move |_| {
                            // TODO: 暂停计时器
                            log::info!("暂停计时器");
                        },
                        span { class: "text-2xl", "⏸️" }
                    }
                    button {
                        class: "flex items-center justify-center w-16 h-16 bg-red-600 hover:bg-red-700 text-white rounded-full shadow-lg transition-all transform hover:scale-105 active:scale-95",
                        onclick: move |_| {
                            // TODO: 停止计时器
                            log::info!("停止计时器");
                        },
                        span { class: "text-xl", "⏹️" }
                    }
                },
                TimerState::Paused { .. } => rsx! {
                    button {
                        class: "flex items-center justify-center w-20 h-20 bg-green-600 hover:bg-green-700 text-white rounded-full shadow-lg transition-all transform hover:scale-105 active:scale-95",
                        onclick: move |_| {
                            // TODO: 继续计时器
                            log::info!("继续计时器");
                        },
                        span { class: "text-2xl", "▶️" }
                    }
                    button {
                        class: "flex items-center justify-center w-16 h-16 bg-red-600 hover:bg-red-700 text-white rounded-full shadow-lg transition-all transform hover:scale-105 active:scale-95",
                        onclick: move |_| {
                            // TODO: 停止计时器
                            log::info!("停止计时器");
                        },
                        span { class: "text-xl", "⏹️" }
                    }
                }
            }
        }
    }
}

/// 统计卡片组件
#[component]
fn StatCard(
    icon: &'static str,
    title: &'static str,
    value: String,
    color: &'static str,
) -> Element {
    let color_classes = match color {
        "blue" => "text-blue-600 dark:text-blue-400",
        "green" => "text-green-600 dark:text-green-400",
        "purple" => "text-purple-600 dark:text-purple-400",
        _ => "text-gray-600 dark:text-gray-400",
    };

    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-6 text-center hover:shadow-xl transition-shadow",

            div { class: "text-4xl mb-3", "{icon}" }
            h3 { class: "text-lg font-semibold text-gray-800 dark:text-white mb-2", "{title}" }
            p { class: "text-2xl font-bold {color_classes}", "{value}" }
        }
    }
}

/// 快速操作区域组件
#[component]
fn QuickActions() -> Element {
    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-6",

            h2 { class: "text-xl font-semibold text-gray-800 dark:text-white mb-4", "快速操作" }

            div {
                class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                button {
                    class: "flex items-center justify-center space-x-3 p-4 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors",
                    onclick: move |_| {
                        // TODO: 快速开始新任务
                        log::info!("快速开始新任务");
                    },
                    span { class: "text-xl", "🚀" }
                    span { class: "font-medium", "快速开始" }
                }

                button {
                    class: "flex items-center justify-center space-x-3 p-4 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors",
                    onclick: move |_| {
                        // TODO: 查看统计
                        log::info!("查看统计");
                    },
                    span { class: "text-xl", "📊" }
                    span { class: "font-medium", "查看统计" }
                }
            }
        }
    }
}
