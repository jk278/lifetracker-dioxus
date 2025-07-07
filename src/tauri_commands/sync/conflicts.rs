//! # 同步冲突处理相关命令
//!
//! 包含冲突获取、解决等功能

use super::types::ConflictItem;
use super::utils::update_sync_status;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

/// 全局冲突存储
pub static PENDING_CONFLICTS: Mutex<Vec<ConflictItem>> = Mutex::new(Vec::new());

/// 获取冲突列表
#[tauri::command]
pub async fn get_sync_conflicts(
    state: State<'_, crate::tauri_commands::AppState>,
) -> std::result::Result<Vec<serde_json::Value>, String> {
    log::info!("获取同步冲突");

    // TODO: 实现冲突列表获取
    // 需要存储和管理冲突信息

    Ok(vec![])
}

/// 获取待解决冲突
#[tauri::command]
pub async fn get_pending_conflicts(
    state: State<'_, crate::tauri_commands::AppState>,
) -> Result<Vec<ConflictItem>, String> {
    log::info!("=== 开始获取待解决冲突 ===");

    // 从全局状态获取冲突
    let conflicts = {
        let pending_conflicts = PENDING_CONFLICTS.lock().unwrap();
        log::info!(
            "全局冲突状态锁定成功，当前冲突数: {}",
            pending_conflicts.len()
        );

        // 详细记录每个冲突项
        for (index, conflict) in pending_conflicts.iter().enumerate() {
            log::info!(
                "冲突项 {}: id={}, name={}, type={}, local_hash={}, remote_hash={:?}",
                index + 1,
                conflict.id,
                conflict.name,
                conflict.conflict_type,
                conflict.local_hash,
                conflict.remote_hash
            );

            // 如果是数据完整性冲突，记录详细信息
            if conflict.conflict_type.contains("data_") {
                log::info!("  -> 数据完整性冲突详情:");
                log::info!("     本地数据预览: {}", conflict.local_preview);
                log::info!("     远程数据预览: {}", conflict.remote_preview);
                log::info!("     文件大小: {} 字节", conflict.file_size);
            }
        }

        pending_conflicts.clone()
    };

    log::info!("当前有 {} 个待解决冲突", conflicts.len());

    // 直接返回冲突列表，不再自动创建冲突项
    log::info!("=== 返回 {} 个冲突项 ===", conflicts.len());
    Ok(conflicts)
}

/// 解决冲突
#[tauri::command]
pub async fn resolve_conflicts(
    state: State<'_, crate::tauri_commands::AppState>,
    resolutions: HashMap<String, String>,
) -> Result<String, String> {
    log::info!("解决冲突，解决方案: {:?}", resolutions);

    // 获取当前冲突并检查是否为空
    let conflicts_empty = {
        let pending_conflicts = PENDING_CONFLICTS.lock().unwrap();
        pending_conflicts.is_empty()
    };

    if conflicts_empty {
        log::info!("全局冲突列表为空，但可能存在未保存的冲突");
    }

    // 获取同步配置
    let sync_config = super::utils::load_sync_config_from_app_state(&state).await?;
    if !sync_config.enabled {
        return Err("同步功能未启用".to_string());
    }

    // 应用用户选择的解决方案
    let mut resolved_count = 0;
    let mut resolution_errors = Vec::new();

    // 先处理所有解决方案，再更新冲突列表
    for (conflict_id, resolution) in resolutions {
        log::info!("处理冲突: {} -> {}", conflict_id, resolution);

        // 处理本地数据冲突项
        if conflict_id == "local_data" || conflict_id == "fresh_data_conflict" {
            log::info!("处理本地数据冲突，应用解决方案: {}", resolution);

            match resolution.as_str() {
                "use_remote" => {
                    log::info!("用户选择使用远程数据，强制下载远程数据");

                    // 直接下载远程数据并覆盖本地数据
                    match force_download_remote_data(&state, "data.json").await {
                        Ok(_) => {
                            log::info!("强制下载远程数据成功");
                            resolved_count += 1;
                        }
                        Err(e) => {
                            let error_msg = format!("强制下载远程数据失败: {}", e);
                            log::error!("{}", error_msg);
                            resolution_errors.push(error_msg);
                        }
                    }
                }
                "use_local" => {
                    log::info!("用户选择使用本地数据，强制上传本地数据");

                    // 直接上传本地数据并覆盖远程数据
                    match force_upload_local_data(&state, "data.json").await {
                        Ok(_) => {
                            log::info!("强制上传本地数据成功");
                            resolved_count += 1;
                        }
                        Err(e) => {
                            let error_msg = format!("强制上传本地数据失败: {}", e);
                            log::error!("{}", error_msg);
                            resolution_errors.push(error_msg);
                        }
                    }
                }
                "merge" => {
                    log::info!("用户选择合并数据，执行智能合并");

                    // 执行智能合并
                    match perform_smart_merge(&state, "data.json").await {
                        Ok(_) => {
                            log::info!("智能合并成功");
                            resolved_count += 1;
                        }
                        Err(e) => {
                            let error_msg = format!("智能合并失败: {}", e);
                            log::error!("{}", error_msg);
                            resolution_errors.push(error_msg);
                        }
                    }
                }
                _ => {
                    return Err(format!("未知的解决方案类型: {}", resolution));
                }
            }
            break;
        }

        // 处理正常的冲突项
        let conflict_exists = {
            let pending_conflicts = PENDING_CONFLICTS.lock().unwrap();
            pending_conflicts.iter().any(|c| c.id == conflict_id)
        };

        if conflict_exists {
            match resolution.as_str() {
                "merge" => {
                    log::info!("应用合并解决方案: {}", conflict_id);
                    match perform_smart_merge(&state, &conflict_id).await {
                        Ok(_) => {
                            log::info!("智能合并成功: {}", conflict_id);
                            resolved_count += 1;
                        }
                        Err(e) => {
                            let error_msg = format!("智能合并失败 {}: {}", conflict_id, e);
                            log::error!("{}", error_msg);
                            resolution_errors.push(error_msg);
                        }
                    }
                }
                "use_local" => {
                    log::info!("使用本地数据: {}", conflict_id);
                    match force_upload_local_data(&state, &conflict_id).await {
                        Ok(_) => {
                            log::info!("强制上传本地数据成功: {}", conflict_id);
                            resolved_count += 1;
                        }
                        Err(e) => {
                            let error_msg = format!("强制上传本地数据失败 {}: {}", conflict_id, e);
                            log::error!("{}", error_msg);
                            resolution_errors.push(error_msg);
                        }
                    }
                }
                "use_remote" => {
                    log::info!("使用远程数据: {}", conflict_id);
                    match force_download_remote_data(&state, &conflict_id).await {
                        Ok(_) => {
                            log::info!("强制下载远程数据成功: {}", conflict_id);
                            resolved_count += 1;
                        }
                        Err(e) => {
                            let error_msg = format!("强制下载远程数据失败 {}: {}", conflict_id, e);
                            log::error!("{}", error_msg);
                            resolution_errors.push(error_msg);
                        }
                    }
                }
                _ => {
                    return Err(format!("未知的解决方案类型: {}", resolution));
                }
            }
        }
    }

    // 如果没有错误，清空冲突列表
    if resolution_errors.is_empty() && resolved_count > 0 {
        let mut pending_conflicts = PENDING_CONFLICTS.lock().unwrap();
        pending_conflicts.clear();
    }

    // 如果有错误，返回错误信息
    if !resolution_errors.is_empty() {
        return Err(resolution_errors.join("; "));
    }

    // 检查是否所有冲突都已解决
    let all_resolved = {
        let pending_conflicts = PENDING_CONFLICTS.lock().unwrap();
        pending_conflicts.is_empty()
    };

    if all_resolved {
        // 更新同步状态为成功
        if let Err(e) = update_sync_status(state.storage.get_database(), "success").await {
            log::warn!("更新同步状态失败: {}", e);
        }
        // 更新最后同步时间
        if let Err(e) = super::utils::update_last_sync_time_in_config(&state).await {
            log::warn!("更新最后同步时间失败: {}", e);
        }
        log::info!("所有冲突已解决，同步状态已更新为成功");
    }

    let result_message = if resolved_count > 0 {
        format!("成功解决了 {} 个冲突", resolved_count)
    } else {
        "没有找到匹配的冲突项".to_string()
    };

    log::info!("冲突解决完成: {}", result_message);
    Ok(result_message)
}

/// 强制上传本地数据
async fn force_upload_local_data(
    state: &State<'_, crate::tauri_commands::AppState>,
    file_name: &str,
) -> Result<(), String> {
    log::info!("强制上传本地数据: {}", file_name);

    // 获取同步配置
    let sync_config = super::utils::load_sync_config_from_app_state(state).await?;

    // 创建同步提供者
    let provider = crate::sync::providers::create_provider(
        &super::utils::create_sync_config_from_request(&sync_config)?,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 序列化本地数据
    let storage = state.storage.clone();
    let serializer = crate::sync::engine::DataSerializer::new(storage);
    let local_data = serializer
        .serialize_all_data()
        .await
        .map_err(|e| e.to_string())?;

    // 创建上传项
    let upload_item = crate::sync::SyncItem {
        id: file_name.to_string(),
        name: file_name.to_string(),
        local_path: file_name.to_string(),
        remote_path: format!("LifeTracker/{}", file_name),
        size: local_data.len() as u64,
        local_modified: chrono::Local::now(),
        remote_modified: None,
        hash: format!("{:x}", md5::compute(&local_data)),
        status: crate::sync::SyncStatus::Idle,
        direction: crate::sync::SyncDirection::Upload,
    };

    // 执行上传
    provider
        .upload_file(&upload_item, &local_data)
        .await
        .map_err(|e| e.to_string())?;

    log::info!("强制上传本地数据完成: {}", file_name);
    Ok(())
}

/// 强制下载远程数据
async fn force_download_remote_data(
    state: &State<'_, crate::tauri_commands::AppState>,
    file_name: &str,
) -> Result<(), String> {
    log::info!("强制下载远程数据: {}", file_name);

    // 获取同步配置
    let sync_config = super::utils::load_sync_config_from_app_state(state).await?;

    // 创建同步提供者
    let provider = crate::sync::providers::create_provider(
        &super::utils::create_sync_config_from_request(&sync_config)?,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 获取远程文件列表
    let remote_files = provider
        .list_remote_files("LifeTracker")
        .await
        .map_err(|e| e.to_string())?;

    // 找到目标文件
    let target_file = remote_files
        .iter()
        .find(|f| f.name == file_name)
        .ok_or_else(|| format!("远程文件不存在: {}", file_name))?;

    // 下载文件
    let remote_data = provider
        .download_file(target_file)
        .await
        .map_err(|e| e.to_string())?;

    // 导入数据到本地存储
    let storage = state.storage.clone();
    let serializer = crate::sync::engine::DataSerializer::new(storage);
    serializer
        .import_data(&remote_data)
        .await
        .map_err(|e| e.to_string())?;

    log::info!("强制下载远程数据完成: {}", file_name);
    Ok(())
}

/// 执行智能合并
async fn perform_smart_merge(
    state: &State<'_, crate::tauri_commands::AppState>,
    file_name: &str,
) -> Result<(), String> {
    log::info!("执行智能合并: {}", file_name);

    // 获取同步配置
    let sync_config = super::utils::load_sync_config_from_app_state(state).await?;

    // 创建同步提供者
    let provider = crate::sync::providers::create_provider(
        &super::utils::create_sync_config_from_request(&sync_config)?,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 获取本地数据
    let storage = state.storage.clone();
    let serializer = crate::sync::engine::DataSerializer::new(storage);
    let local_data = serializer
        .serialize_all_data()
        .await
        .map_err(|e| e.to_string())?;
    let local_json: serde_json::Value =
        serde_json::from_slice(&local_data).map_err(|e| e.to_string())?;

    // 获取远程数据
    let remote_files = provider
        .list_remote_files("LifeTracker")
        .await
        .map_err(|e| e.to_string())?;
    let target_file = remote_files
        .iter()
        .find(|f| f.name == file_name)
        .ok_or_else(|| format!("远程文件不存在: {}", file_name))?;

    // 下载远程数据
    let remote_data = provider
        .download_file(target_file)
        .await
        .map_err(|e| e.to_string())?;
    let remote_json: serde_json::Value =
        serde_json::from_slice(&remote_data).map_err(|e| e.to_string())?;

    // 创建同步引擎来执行合并
    let sync_engine_config = super::utils::create_sync_config_from_request(&sync_config)?;
    let mut engine =
        crate::sync::engine::SyncEngine::new(state.storage.clone(), sync_engine_config)
            .map_err(|e| e.to_string())?;
    engine.initialize().await.map_err(|e| e.to_string())?;

    // 执行智能合并
    let merge_config = crate::sync::engine::MergeConfig {
        deduplicate: true,
        priority_strategy: crate::sync::engine::MergePriorityStrategy::TimestampFirst,
    };

    let merged_data = engine
        .smart_merge(&local_json, &remote_json, &merge_config)
        .await
        .map_err(|e| e.to_string())?;

    // 保存合并后的数据到本地
    let merged_data_bytes = serde_json::to_vec(&merged_data).map_err(|e| e.to_string())?;
    serializer
        .import_data(&merged_data_bytes)
        .await
        .map_err(|e| e.to_string())?;

    // 上传合并后的数据到远程
    let upload_item = crate::sync::SyncItem {
        id: file_name.to_string(),
        name: file_name.to_string(),
        local_path: file_name.to_string(),
        remote_path: format!("LifeTracker/{}", file_name),
        size: merged_data_bytes.len() as u64,
        local_modified: chrono::Local::now(),
        remote_modified: None,
        hash: format!("{:x}", md5::compute(&merged_data_bytes)),
        status: crate::sync::SyncStatus::Idle,
        direction: crate::sync::SyncDirection::Upload,
    };

    provider
        .upload_file(&upload_item, &merged_data_bytes)
        .await
        .map_err(|e| e.to_string())?;

    log::info!("智能合并完成: {}", file_name);
    Ok(())
}

/// 解决同步冲突
#[tauri::command]
pub async fn resolve_sync_conflict(
    conflict_id: String,
    resolution: String,
    state: State<'_, crate::tauri_commands::AppState>,
) -> std::result::Result<String, String> {
    log::info!("解决同步冲突: {} -> {}", conflict_id, resolution);

    // TODO: 实现冲突解决逻辑

    Ok(format!("冲突 {} 已解决", conflict_id))
}
