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
    // 更新内存中的配置
    {
        let mut state_config = state.config.lock().unwrap();
        *state_config = config.clone();
    }

    // 持久化保存到配置文件
    let mut config_manager =
        crate::config::create_config_manager().map_err(|e| format!("创建配置管理器失败: {}", e))?;

    *config_manager.config_mut() = config;
    config_manager
        .save()
        .map_err(|e| format!("保存配置文件失败: {}", e))?;

    log::info!("配置已成功更新并保存到文件");
    Ok(true)
}

/// 设置窗口主题背景色
#[tauri::command]
pub async fn set_window_theme(app_handle: AppHandle, is_dark: bool) -> Result<(), String> {
    // 移动端兼容处理
    #[cfg(target_os = "android")]
    {
        log::info!("移动端主题已更新为: (暗色模式: {})", is_dark);
        return Ok(());
    }

    // 桌面端设置窗口背景色，使用与启动时一致的颜色
    #[cfg(not(target_os = "android"))]
    {
        let bg_color = if is_dark {
            Color(15, 20, 25, 255) // 暗色模式背景 #0f1419
        } else {
            Color(249, 250, 251, 255) // 亮色模式背景 #f9fafb (gray-50)
        };

        if let Some(window) = app_handle.get_webview_window("main") {
            // 在 Tauri v2 中，set_background_color 方法存在且签名正确
            window
                .set_background_color(Some(bg_color))
                .map_err(|e| format!("设置窗口背景色失败: {}", e))?;

            log::info!(
                "桌面端主题已更新为: (暗色模式: {})，背景色: {:?}",
                is_dark,
                bg_color
            );
        }
    }

    Ok(())
}
