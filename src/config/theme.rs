//! # 主题配置模块
//!
//! 提供应用程序主题相关的配置管理功能

use serde::{Deserialize, Serialize};

/// 主题配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// 当前主题名称
    pub current_theme: String,
    /// 是否跟随系统主题
    pub follow_system: bool,
    /// 自定义主题设置
    pub custom_themes: Vec<CustomTheme>,
}

/// 自定义主题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTheme {
    /// 主题名称
    pub name: String,
    /// 主题描述
    pub description: Option<String>,
    /// 是否为深色主题
    pub is_dark: bool,
    /// 主题颜色配置
    pub colors: ThemeColors,
}

/// 主题颜色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    /// 主色调
    pub primary: String,
    /// 次要颜色
    pub secondary: String,
    /// 背景色
    pub background: String,
    /// 表面色
    pub surface: String,
    /// 文本色
    pub text: String,
    /// 错误色
    pub error: String,
    /// 警告色
    pub warning: String,
    /// 成功色
    pub success: String,
    /// 信息色
    pub info: String,
}

impl ThemeConfig {
    /// 检测系统主题偏好
    /// 在应用启动时调用，避免webview加载时的主题闪烁
    pub fn detect_system_theme() -> String {
        #[cfg(target_os = "windows")]
        {
            // Windows 注册表检查
            match Self::detect_windows_theme() {
                Some(theme) => return theme,
                None => log::warn!("无法从Windows注册表检测主题，使用默认值"),
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS 系统偏好检查
            match Self::detect_macos_theme() {
                Some(theme) => return theme,
                None => log::warn!("无法从macOS系统偏好检测主题，使用默认值"),
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux 桌面环境检查
            match Self::detect_linux_theme() {
                Some(theme) => return theme,
                None => log::warn!("无法从Linux桌面环境检测主题，使用默认值"),
            }
        }

        // 默认返回亮色主题
        "light".to_string()
    }

    #[cfg(target_os = "windows")]
    fn detect_windows_theme() -> Option<String> {
        use std::process::Command;

        // 使用PowerShell查询Windows主题设置
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Get-ItemProperty -Path 'HKCU:SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize' -Name 'AppsUseLightTheme' | Select-Object -ExpandProperty AppsUseLightTheme"
            ])
            .output()
            .ok()?;

        if output.status.success() {
            let result = String::from_utf8(output.stdout).ok()?;
            let value = result.trim().parse::<u32>().ok()?;

            // Windows: 0 = 暗色模式, 1 = 亮色模式
            Some(if value == 0 { "dark" } else { "light" }.to_string())
        } else {
            None
        }
    }

    #[cfg(target_os = "macos")]
    fn detect_macos_theme() -> Option<String> {
        use std::process::Command;

        // 使用defaults命令查询macOS主题设置
        let output = Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
            .ok()?;

        if output.status.success() {
            let result = String::from_utf8(output.stdout).ok()?;
            // macOS: "Dark" = 暗色模式, 其他或错误 = 亮色模式
            Some(
                if result.trim() == "Dark" {
                    "dark"
                } else {
                    "light"
                }
                .to_string(),
            )
        } else {
            // 命令失败通常意味着设置为亮色模式
            Some("light".to_string())
        }
    }

    #[cfg(target_os = "linux")]
    fn detect_linux_theme() -> Option<String> {
        use std::process::Command;

        // 尝试多种Linux桌面环境的主题检测方法

        // 1. GNOME/GTK
        if let Ok(output) = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
            .output()
        {
            if output.status.success() {
                let theme_name = String::from_utf8(output.stdout).ok()?;
                if theme_name.to_lowercase().contains("dark") {
                    return Some("dark".to_string());
                }
            }
        }

        // 2. KDE Plasma
        if let Ok(output) = Command::new("kreadconfig5")
            .args(["--group", "Colors:Window", "--key", "BackgroundNormal"])
            .output()
        {
            if output.status.success() {
                let bg_color = String::from_utf8(output.stdout).ok()?;
                // 简单的颜色亮度检测（KDE背景色）
                if Self::is_dark_color(&bg_color) {
                    return Some("dark".to_string());
                }
            }
        }

        // 3. 环境变量检查
        if let Ok(theme) = std::env::var("GTK_THEME") {
            if theme.to_lowercase().contains("dark") {
                return Some("dark".to_string());
            }
        }

        None
    }

    #[cfg(target_os = "linux")]
    fn is_dark_color(color_str: &str) -> bool {
        // 简单的颜色亮度检测
        // 这里可以实现更复杂的颜色分析逻辑
        color_str.to_lowercase().contains("dark")
            || color_str.starts_with("#") && color_str.len() >= 7 && {
                // 提取RGB值并计算亮度
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&color_str[1..3], 16),
                    u8::from_str_radix(&color_str[3..5], 16),
                    u8::from_str_radix(&color_str[5..7], 16),
                ) {
                    // 使用感知亮度公式
                    let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
                    luminance < 128.0 // 小于128认为是暗色
                } else {
                    false
                }
            }
    }

    /// 根据检测到的系统主题返回对应的CSS类名
    pub fn get_initial_theme_class() -> String {
        let detected_theme = Self::detect_system_theme();
        log::info!("检测到系统主题: {}", detected_theme);
        detected_theme
    }

    /// 获取主题对应的背景色（用于避免闪烁）
    pub fn get_theme_background_color(theme: &str) -> String {
        match theme {
            "dark" => "#1a1a1a".to_string(), // 暗色背景
            _ => "#ffffff".to_string(),      // 亮色背景
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            current_theme: Self::detect_system_theme(), // 默认使用检测到的系统主题
            follow_system: true,
            custom_themes: vec![],
        }
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            primary: "#007ACC".to_string(),
            secondary: "#6C757D".to_string(),
            background: "#FFFFFF".to_string(),
            surface: "#F8F9FA".to_string(),
            text: "#212529".to_string(),
            error: "#DC3545".to_string(),
            warning: "#FFC107".to_string(),
            success: "#28A745".to_string(),
            info: "#17A2B8".to_string(),
        }
    }
}
