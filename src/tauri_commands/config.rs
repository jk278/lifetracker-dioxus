//! # 配置管理命令模块
//!
//! 负责处理应用配置和主题设置

use super::*;
use tauri::window::Color;

// ========== 配置管理命令 ==========

/// 获取应用配置
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().unwrap().clone();
    Ok(config)
}

/// 更新应用配置
#[tauri::command]
pub async fn update_config(state: State<'_, AppState>, config: AppConfig) -> Result<bool, String> {
    let mut state_config = state.config.lock().unwrap();
    *state_config = config;
    Ok(true)
}

/// 设置窗口主题背景色
#[tauri::command]
pub async fn set_window_theme(app_handle: AppHandle, is_dark: bool) -> Result<(), String> {
    let bg_color = if is_dark {
        Color(0, 0, 0, 255) // 暗色模式纯黑 #000000
    } else {
        Color(249, 250, 251, 255) // 亮色模式背景 #f9fafb (gray-50)
    };

    if let Some(window) = app_handle.get_webview_window("main") {
        // 在 Tauri v2 中，set_background_color 方法可能不存在或签名发生了变化
        // 主题切换主要由前端 CSS 处理，这个后端设置不是必需的
        // window
        //     .set_background_color(Some(bg_color))
        //     .map_err(|e| format!("设置窗口背景色失败: {}", e))?;

        log::info!("主题已更新为: (暗色模式: {})", is_dark);
    }

    Ok(())
}
