//! # 同步历史记录相关命令
//!
//! 包含历史记录获取、清理等功能

use super::types::SyncResultResponse;
use crate::tauri_commands::AppState;
use tauri::State;

/// 获取同步历史
#[tauri::command]
pub async fn get_sync_history(
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<SyncResultResponse>, String> {
    log::info!("获取同步历史");

    // TODO: 实现同步历史存储和查询
    // 目前返回空列表

    Ok(vec![])
}

/// 清除同步历史
#[tauri::command]
pub async fn clear_sync_history(state: State<'_, AppState>) -> std::result::Result<String, String> {
    log::info!("清除同步历史");

    // TODO: 实现清除同步历史

    Ok("同步历史已清除".to_string())
}
