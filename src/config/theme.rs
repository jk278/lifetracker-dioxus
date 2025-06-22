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

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            current_theme: "default".to_string(),
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
