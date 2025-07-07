//! # 同步操作相关命令
//!
//! 包含同步执行、状态管理等功能

use super::types::{SyncConfigRequest, SyncStatusResponse};
use super::utils::{
    create_conflict_item_from_sync_item, load_sync_config_from_app_state, update_last_sync_time,
    update_sync_status,
};
use crate::storage::StorageManager;
use crate::sync::engine::SyncEngine;
use crate::sync::providers::create_provider;
use crate::sync::validate_sync_config;
use crate::tauri_commands::AppState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

/// 开始同步
#[tauri::command]
pub async fn start_sync(state: State<'_, AppState>) -> std::result::Result<String, String> {
    log::info!("开始手动同步");

    let storage = state.storage.clone();
    let sync_config = load_sync_config_from_app_state(&state).await?;

    if !sync_config.enabled {
        return Err("同步功能未启用".to_string());
    }

    // 创建同步引擎
    let engine = create_sync_engine(&sync_config, storage.clone()).await?;

    // 执行同步
    let sync_result = engine.sync().await.map_err(|e| e.to_string())?;

    // 检查是否有冲突
    if !sync_result.conflicts.is_empty() {
        log::info!(
            "检测到 {} 个冲突，需要手动解决",
            sync_result.conflicts.len()
        );

        // 创建冲突项列表
        let mut conflict_items = Vec::new();
        for conflict in &sync_result.conflicts {
            let conflict_item = create_conflict_item_from_sync_item(conflict);
            log::info!(
                "创建冲突项: id={}, name={}",
                conflict_item.id,
                conflict_item.name
            );
            conflict_items.push(conflict_item);
        }

        // 将冲突存储到全局状态
        {
            let mut pending_conflicts = super::conflicts::PENDING_CONFLICTS.lock().unwrap();
            pending_conflicts.clear();
            pending_conflicts.extend(conflict_items.clone());
            log::info!("已存储 {} 个冲突项到全局状态", pending_conflicts.len());
        }

        // 更新同步状态和时间
        update_sync_status(storage.get_database(), "conflict_pending").await?;
        update_last_sync_time(storage.get_database()).await?;

        return Ok(format!(
            "同步检测到 {} 个冲突需要解决",
            sync_result.conflicts.len()
        ));
    }

    // 无冲突，正常完成
    let result_message = format!(
        "同步完成，下载{}，上传{}",
        sync_result.downloaded_count, sync_result.uploaded_count
    );

    // 更新同步状态和时间
    update_sync_status(storage.get_database(), "success").await?;
    update_last_sync_time(storage.get_database()).await?;

    log::info!("同步成功，已更新最后同步时间");

    Ok(result_message)
}

/// 获取同步状态
#[tauri::command]
pub async fn get_sync_status(
    state: State<'_, AppState>,
) -> std::result::Result<SyncStatusResponse, String> {
    log::info!("获取同步状态");

    let config = {
        let config_guard = state.config.lock().unwrap();
        config_guard.clone()
    };

    // TODO: 实现真实的同步状态获取
    // 这里需要维护一个全局的同步引擎状态

    Ok(SyncStatusResponse {
        status: if config.data.sync.enabled {
            "enabled".to_string()
        } else {
            "disabled".to_string()
        },
        is_syncing: false,
        last_sync_time: config
            .data
            .sync
            .last_sync_time
            .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        next_sync_time: None, // TODO: 计算下次同步时间
        error_message: None,
    })
}

/// 停止同步
#[tauri::command]
pub async fn stop_sync(state: State<'_, AppState>) -> std::result::Result<String, String> {
    log::info!("停止同步");

    // TODO: 实现停止同步逻辑
    // 需要维护全局的同步引擎实例

    Ok("同步已停止".to_string())
}

/// 创建同步引擎
pub async fn create_sync_engine(
    sync_config: &SyncConfigRequest,
    storage: Arc<StorageManager>,
) -> Result<SyncEngine, String> {
    let config = super::utils::create_sync_config_from_request(sync_config)?;

    // 创建同步引擎
    let mut engine = SyncEngine::new(storage, config).map_err(|e| e.to_string())?;

    // 初始化同步引擎（会自动创建提供者并测试连接）
    engine.initialize().await.map_err(|e| e.to_string())?;

    Ok(engine)
}
