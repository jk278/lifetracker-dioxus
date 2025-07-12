//! # 设置模块
//!
//! 包含应用配置、主题设置等功能

use dioxus::prelude::*;
use life_tracker::config::{get_default_config_path, AppConfig, ConfigManager};

#[derive(Props, Clone, PartialEq)]
pub struct SettingsPageProps {
    pub show_back_button: bool,
    pub on_back: Option<EventHandler<()>>,
}

#[component]
pub fn SettingsPage(props: SettingsPageProps) -> Element {
    // 配置状态
    let config = use_signal(|| AppConfig::default());
    let loading = use_signal(|| false);
    let saving = use_signal(|| false);
    let error_message = use_signal(|| None::<String>);

    // 主题状态
    let theme_mode = use_signal(|| "system".to_string());
    let theme_color = use_signal(|| "blue".to_string());

    // 加载配置
    let load_config = {
        let mut config = config.clone();
        let mut loading = loading.clone();
        let mut error_message = error_message.clone();
        let mut theme_mode = theme_mode.clone();

        move || {
            spawn(async move {
                loading.set(true);
                error_message.set(None);

                match load_app_config().await {
                    Ok(app_config) => {
                        config.set(app_config.clone());

                        // 设置主题状态
                        if app_config.ui.dark_mode {
                            theme_mode.set("dark".to_string());
                        } else if app_config.ui.theme == "system" {
                            theme_mode.set("system".to_string());
                        } else {
                            theme_mode.set("light".to_string());
                        }

                        log::info!("Configuration loaded successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to load configuration: {}", e);
                        error_message.set(Some("获取配置失败".to_string()));
                    }
                }

                loading.set(false);
            });
        }
    };

    // 保存配置
    let save_config = {
        let config = config.read().clone();
        let mut saving = saving.clone();
        let mut error_message = error_message.clone();

        move || {
            spawn(async move {
                saving.set(true);
                error_message.set(None);

                match save_app_config(&config).await {
                    Ok(_) => {
                        log::info!("Configuration saved successfully");
                        // 可以添加成功提示
                    }
                    Err(e) => {
                        log::error!("Failed to save configuration: {}", e);
                        error_message.set(Some("保存配置失败".to_string()));
                    }
                }

                saving.set(false);
            });
        }
    };

    // 初始化加载配置
    use_effect(move || {
        load_config();
    });

    // 主题切换处理
    let mut handle_theme_change = {
        let mut config = config.clone();
        let mut theme_mode = theme_mode.clone();

        move |new_theme: String| {
            theme_mode.set(new_theme.clone());
            let mut current_config = config.read().clone();

            match new_theme.as_str() {
                "dark" => {
                    current_config.ui.dark_mode = true;
                    current_config.ui.theme = "dark".to_string();
                }
                "light" => {
                    current_config.ui.dark_mode = false;
                    current_config.ui.theme = "light".to_string();
                }
                "system" => {
                    current_config.ui.theme = "system".to_string();
                    // 系统主题的处理逻辑
                }
                _ => {}
            }

            config.set(current_config);
        }
    };

    // 主题颜色变化处理
    let mut handle_theme_color_change = {
        let mut config = config.clone();
        let mut theme_color = theme_color.clone();

        move |new_color: String| {
            theme_color.set(new_color.clone());
            let mut current_config = config.read().clone();
            current_config.ui.theme = new_color;
            config.set(current_config);
        }
    };

    // 通用配置更新处理
    let mut handle_config_update = {
        let mut config = config.clone();

        move |key: String, value: SettingValue| {
            let mut current_config = config.read().clone();

            // 根据key更新对应的配置值
            match key.as_str() {
                "general.auto_start_timer" => {
                    if let Some(val) = value.as_bool() {
                        current_config.general.auto_start_timer = val;
                    }
                }
                "general.minimize_to_tray" => {
                    if let Some(val) = value.as_bool() {
                        current_config.general.minimize_to_tray = val;
                    }
                }
                "general.work_reminder_interval" => {
                    if let Some(val) = value.as_i64() {
                        current_config.general.work_reminder_interval = Some(val as u32);
                    }
                }
                "general.break_reminder_interval" => {
                    if let Some(val) = value.as_i64() {
                        current_config.general.break_reminder_interval = Some(val as u32);
                    }
                }
                "notifications.enabled" => {
                    if let Some(val) = value.as_bool() {
                        current_config.notifications.enabled = val;
                    }
                }
                "notifications.sound_notifications" => {
                    if let Some(val) = value.as_bool() {
                        current_config.notifications.sound_notifications = val;
                    }
                }
                "data.auto_backup" => {
                    if let Some(val) = value.as_bool() {
                        current_config.data.auto_backup = val;
                    }
                }
                "data.data_retention_days" => {
                    if let Some(val) = value.as_i64() {
                        current_config.data.data_retention_days = Some(val as u32);
                    }
                }
                _ => {}
            }

            config.set(current_config);
        }
    };

    if *loading.read() {
        return rsx! {
            div { class: "flex justify-center items-center h-64",
                div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" }
            }
        };
    }

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
                            "设置"
                        }
                    }
                    button {
                        class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors",
                        disabled: *saving.read(),
                        onclick: move |_| save_config(),
                        "💾 "
                        if *saving.read() { "保存中..." } else { "保存设置" }
                    }
                }
            }

            // 错误消息
            if let Some(error) = error_message.read().as_ref() {
                div { class: "p-4 bg-red-50 dark:bg-red-900/20 border-l-4 border-red-500 mx-4 mt-4",
                    p { class: "text-red-700 dark:text-red-400", "{error}" }
                }
            }

            // 可滚动内容区域
            div { class: "flex-1 overflow-y-auto py-4 px-4 md:px-6 space-y-6",

                // 界面设置
                div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg p-6",
                    div { class: "flex items-center mb-4",
                        span { class: "text-2xl mr-2", "🎨" }
                        h3 { class: "text-lg font-semibold text-gray-900 dark:text-white",
                            "界面设置"
                        }
                    }

                    div { class: "space-y-4",
                        // 明暗模式
                        div {
                            label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3",
                                "明暗模式"
                            }
                            div { class: "grid grid-cols-3 gap-3",
                                for (value, label, icon) in [
                                    ("system", "跟随系统", "🖥️"),
                                    ("light", "浅色", "☀️"),
                                    ("dark", "深色", "🌙"),
                                ] {
                                    button {
                                        class: if *theme_mode.read() == value {
                                            "flex flex-col items-center p-3 rounded-lg border-2 border-blue-600 bg-blue-50 dark:bg-blue-900/20 text-gray-900 dark:text-white transition-all"
                                        } else {
                                            "flex flex-col items-center p-3 rounded-lg border-2 border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 hover:border-gray-300 dark:hover:border-gray-500 text-gray-900 dark:text-gray-100 transition-all"
                                        },
                                        onclick: move |_| handle_theme_change(value.to_string()),
                                        span { class: "text-xl mb-1", "{icon}" }
                                        span { class: "text-sm font-medium", "{label}" }
                                    }
                                }
                            }
                        }

                        // 主题色彩
                        div {
                            label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3",
                                "主题色彩"
                            }
                            div { class: "grid grid-cols-3 gap-3",
                                for (color, name, hex) in [
                                    ("blue", "蓝色", "#3b82f6"),
                                    ("green", "绿色", "#10b981"),
                                    ("purple", "紫色", "#8b5cf6"),
                                    ("pink", "粉色", "#ec4899"),
                                    ("orange", "橙色", "#f59e0b"),
                                    ("red", "红色", "#ef4444"),
                                ] {
                                    button {
                                        class: if *theme_color.read() == color {
                                            "flex flex-col items-center p-3 rounded-lg border-2 border-gray-400 dark:border-gray-300 bg-gray-50 dark:bg-gray-700 transition-all"
                                        } else {
                                            "flex flex-col items-center p-3 rounded-lg border-2 border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 hover:border-gray-300 dark:hover:border-gray-500 transition-all"
                                        },
                                        onclick: move |_| handle_theme_color_change(color.to_string()),
                                        div {
                                            class: "w-6 h-6 rounded-full mb-2 border-2 border-white dark:border-gray-800 shadow-sm",
                                            style: "background-color: {hex}"
                                        }
                                        span { class: "text-sm text-gray-700 dark:text-gray-300", "{name}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // 常规设置
                div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg p-6",
                    div { class: "flex items-center mb-4",
                        span { class: "text-2xl mr-2", "⚙️" }
                        h3 { class: "text-lg font-semibold text-gray-900 dark:text-white",
                            "常规设置"
                        }
                    }

                    div { class: "space-y-4",
                        // 自动开始计时
                        div { class: "flex items-center justify-between",
                            div {
                                label { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                    "自动开始计时"
                                }
                                p { class: "text-sm text-gray-500 dark:text-gray-400",
                                    "启动应用时自动开始计时"
                                }
                            }
                            input {
                                r#type: "checkbox",
                                checked: config.read().general.auto_start_timer,
                                onchange: move |e| {
                                    handle_config_update("general.auto_start_timer".to_string(), SettingValue::Boolean(e.value() == "true"));
                                },
                                class: "h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                            }
                        }

                        // 最小化到托盘
                        div { class: "flex items-center justify-between",
                            div {
                                label { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                    "最小化到托盘"
                                }
                                p { class: "text-sm text-gray-500 dark:text-gray-400",
                                    "关闭窗口时最小化到系统托盘"
                                }
                            }
                            input {
                                r#type: "checkbox",
                                checked: config.read().general.minimize_to_tray,
                                onchange: move |e| {
                                    handle_config_update("general.minimize_to_tray".to_string(), SettingValue::Boolean(e.value() == "true"));
                                },
                                class: "h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                            }
                        }

                        // 工作提醒间隔
                        div {
                            label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                                "工作提醒间隔（分钟）"
                            }
                            input {
                                r#type: "number",
                                value: config.read().general.work_reminder_interval.unwrap_or(25).to_string(),
                                min: "1",
                                max: "480",
                                onchange: move |e| {
                                    if let Ok(val) = e.value().parse::<i64>() {
                                        handle_config_update("general.work_reminder_interval".to_string(), SettingValue::Integer(val));
                                    }
                                },
                                class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            }
                        }

                        // 休息提醒间隔
                        div {
                            label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                                "休息提醒间隔（分钟）"
                            }
                            input {
                                r#type: "number",
                                value: config.read().general.break_reminder_interval.unwrap_or(5).to_string(),
                                min: "1",
                                max: "60",
                                onchange: move |e| {
                                    if let Ok(val) = e.value().parse::<i64>() {
                                        handle_config_update("general.break_reminder_interval".to_string(), SettingValue::Integer(val));
                                    }
                                },
                                class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            }
                        }
                    }
                }

                // 通知设置
                div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg p-6",
                    div { class: "flex items-center mb-4",
                        span { class: "text-2xl mr-2", "🔔" }
                        h3 { class: "text-lg font-semibold text-gray-900 dark:text-white",
                            "通知设置"
                        }
                    }

                    div { class: "space-y-4",
                        // 启用通知
                        div { class: "flex items-center justify-between",
                            div {
                                label { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                    "启用通知"
                                }
                                p { class: "text-sm text-gray-500 dark:text-gray-400",
                                    "启用桌面通知功能"
                                }
                            }
                            input {
                                r#type: "checkbox",
                                checked: config.read().notifications.enabled,
                                onchange: move |e| {
                                    handle_config_update("notifications.enabled".to_string(), SettingValue::Boolean(e.value() == "true"));
                                },
                                class: "h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                            }
                        }

                        // 声音通知
                        div { class: "flex items-center justify-between",
                            div {
                                label { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                    "声音通知"
                                }
                                p { class: "text-sm text-gray-500 dark:text-gray-400",
                                    "通知时播放声音"
                                }
                            }
                            input {
                                r#type: "checkbox",
                                checked: config.read().notifications.sound_notifications,
                                onchange: move |e| {
                                    handle_config_update("notifications.sound_notifications".to_string(), SettingValue::Boolean(e.value() == "true"));
                                },
                                class: "h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                            }
                        }
                    }
                }

                // 数据设置
                div { class: "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg p-6",
                    div { class: "flex items-center mb-4",
                        span { class: "text-2xl mr-2", "💾" }
                        h3 { class: "text-lg font-semibold text-gray-900 dark:text-white",
                            "数据设置"
                        }
                    }

                    div { class: "space-y-4",
                        // 自动备份
                        div { class: "flex items-center justify-between",
                            div {
                                label { class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                                    "自动备份"
                                }
                                p { class: "text-sm text-gray-500 dark:text-gray-400",
                                    "定期自动备份数据"
                                }
                            }
                            input {
                                r#type: "checkbox",
                                checked: config.read().data.auto_backup,
                                onchange: move |e| {
                                    handle_config_update("data.auto_backup".to_string(), SettingValue::Boolean(e.value() == "true"));
                                },
                                class: "h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                            }
                        }

                        // 数据保留天数
                        div {
                            label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                                "数据保留天数"
                            }
                            input {
                                r#type: "number",
                                value: config.read().data.data_retention_days.unwrap_or(365).to_string(),
                                min: "1",
                                max: "9999",
                                onchange: move |e| {
                                    if let Ok(val) = e.value().parse::<i64>() {
                                        handle_config_update("data.data_retention_days".to_string(), SettingValue::Integer(val));
                                    }
                                },
                                class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            }
                        }
                    }
                }
            }
        }
    }
}

// 设置值枚举（简化版）
#[derive(Debug, Clone, PartialEq)]
enum SettingValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

impl SettingValue {
    fn as_bool(&self) -> Option<bool> {
        match self {
            SettingValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            SettingValue::Integer(i) => Some(*i),
            _ => None,
        }
    }
}

// 异步函数：加载配置
async fn load_app_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    // 使用现有的配置管理器
    let config_path =
        get_default_config_path().map_err(|e| format!("Failed to get config path: {}", e))?;
    let config_manager = ConfigManager::new(config_path)
        .map_err(|e| format!("Failed to create config manager: {}", e))?;
    Ok(config_manager.config().clone())
}

// 异步函数：保存配置
async fn save_app_config(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let config_path =
        get_default_config_path().map_err(|e| format!("Failed to get config path: {}", e))?;
    let mut config_manager = ConfigManager::new(config_path)
        .map_err(|e| format!("Failed to create config manager: {}", e))?;

    // 更新配置
    *config_manager.config_mut() = config.clone();

    // 保存配置
    config_manager
        .save()
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}
