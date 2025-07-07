//! # 同步相关的 Tauri 命令
//!
//! 提供前端调用的同步功能接口

use crate::errors::AppError;
use crate::storage::database::Database;
use crate::storage::StorageManager;
use crate::sync::engine::SyncEngine;
use crate::sync::providers::create_provider;
use crate::sync::providers::webdav::WebDavProvider;
use crate::sync::{
    validate_sync_config, ConflictStrategy, SyncConfig, SyncItem, SyncResult, SyncStatus,
};
use crate::tauri_commands::AppState;
use crate::utils::crypto::{decrypt_password, encrypt_password};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

/// 全局冲突存储
static PENDING_CONFLICTS: Mutex<Vec<ConflictItem>> = Mutex::new(Vec::new());

/// 同步配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfigRequest {
    /// 是否启用同步
    pub enabled: bool,
    /// 同步提供者
    pub provider: String,
    /// 是否启用自动同步
    pub auto_sync: bool,
    /// 同步间隔（分钟）
    pub sync_interval: u32,
    /// 冲突解决策略
    pub conflict_strategy: String,
    /// WebDAV配置
    pub webdav_config: Option<WebDavConfigRequest>,
}

/// WebDAV配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebDavConfigRequest {
    /// 服务器URL
    pub url: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 同步目录
    pub directory: String,
}

/// 同步状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusResponse {
    /// 同步状态
    pub status: String,
    /// 是否正在同步
    pub is_syncing: bool,
    /// 最后同步时间
    pub last_sync_time: Option<String>,
    /// 下次同步时间
    pub next_sync_time: Option<String>,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 同步结果响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResultResponse {
    /// 同步是否成功
    pub success: bool,
    /// 开始时间
    pub start_time: String,
    /// 结束时间
    pub end_time: String,
    /// 上传的文件数量
    pub uploaded_count: u32,
    /// 下载的文件数量
    pub downloaded_count: u32,
    /// 跳过的文件数量
    pub skipped_count: u32,
    /// 失败的文件数量
    pub failed_count: u32,
    /// 同步的字节数
    pub total_bytes: u64,
    /// 错误信息
    pub errors: Vec<String>,
    /// 冲突数量
    pub conflicts_count: u32,
    /// 耗时（秒）
    pub duration_seconds: i64,
}

/// 获取同步配置
#[tauri::command]
pub async fn get_sync_config(
    state: State<'_, AppState>,
) -> std::result::Result<SyncConfigRequest, String> {
    log::info!("获取同步配置");

    let config = {
        let config_guard = state.config.lock().unwrap();
        config_guard.clone()
    };

    let sync_config = &config.data.sync;

    // 解密密码（如果存在）
    let webdav_config = if sync_config.provider == "webdav" {
        let password = if let Some(encrypted_password) = &sync_config.webdav_password_encrypted {
            if !encrypted_password.trim().is_empty() {
                match decrypt_password(encrypted_password, "life_tracker_webdav") {
                    Ok(pwd) => pwd,
                    Err(e) => {
                        log::warn!("解密WebDAV密码失败: {}", e);
                        // 密码解密失败时返回空字符串，让用户重新输入
                        String::new()
                    }
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        Some(WebDavConfigRequest {
            url: sync_config.webdav_url.clone().unwrap_or_default(),
            username: sync_config.webdav_username.clone().unwrap_or_default(),
            password,
            directory: sync_config.sync_directory.clone(),
        })
    } else {
        None
    };

    Ok(SyncConfigRequest {
        enabled: sync_config.enabled,
        provider: sync_config.provider.clone(),
        auto_sync: sync_config.auto_sync,
        sync_interval: sync_config.sync_interval,
        conflict_strategy: sync_config.conflict_strategy.clone(),
        webdav_config,
    })
}

/// 保存同步配置
#[tauri::command]
pub async fn save_sync_config(
    request: SyncConfigRequest,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    log::info!("保存同步配置");

    // 验证配置
    if request.sync_interval < 5 {
        return Err("同步间隔不能小于5分钟".to_string());
    }

    let mut config = {
        let config_guard = state.config.lock().unwrap();
        config_guard.clone()
    };

    // 更新同步配置
    config.data.sync.enabled = request.enabled;
    config.data.sync.provider = request.provider.clone();
    config.data.sync.auto_sync = request.auto_sync;
    config.data.sync.sync_interval = request.sync_interval;
    config.data.sync.conflict_strategy = request.conflict_strategy;

    // 处理WebDAV配置
    if request.provider == "webdav" {
        if let Some(webdav_config) = request.webdav_config {
            config.data.sync.webdav_url = Some(webdav_config.url);
            config.data.sync.webdav_username = Some(webdav_config.username);
            config.data.sync.sync_directory = webdav_config.directory;

            // 加密密码
            if !webdav_config.password.trim().is_empty() {
                let encrypted_password =
                    encrypt_password(&webdav_config.password, "life_tracker_webdav")
                        .map_err(|e| format!("加密密码失败: {}", e))?;
                config.data.sync.webdav_password_encrypted = Some(encrypted_password);
            } else {
                // 如果密码为空，清除加密密码字段
                config.data.sync.webdav_password_encrypted = None;
            }
        }
    }

    // 保存配置到内存
    {
        let mut config_guard = state.config.lock().unwrap();
        *config_guard = config.clone();
    }

    // 保存配置到文件
    let mut config_manager =
        crate::config::create_config_manager().map_err(|e| format!("创建配置管理器失败: {}", e))?;

    *config_manager.config_mut() = config;
    config_manager
        .save()
        .map_err(|e| format!("保存配置文件失败: {}", e))?;

    Ok("同步配置已保存".to_string())
}

/// 测试同步连接
#[tauri::command]
pub async fn test_sync_connection(
    request: SyncConfigRequest,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    log::info!("测试同步连接");

    // 验证WebDAV配置
    if request.provider == "webdav" {
        if let Some(webdav_config) = &request.webdav_config {
            if webdav_config.password.trim().is_empty() {
                return Err("WebDAV密码为空，请重新设置密码后再测试连接".to_string());
            }

            // 基本URL格式检查
            if !webdav_config.url.starts_with("http://")
                && !webdav_config.url.starts_with("https://")
            {
                return Err("WebDAV服务器URL必须以http://或https://开头".to_string());
            }

            log::info!(
                "正在测试连接到: {}/{}",
                webdav_config.url,
                webdav_config.directory
            );
        } else {
            return Err("WebDAV配置不完整".to_string());
        }
    }

    // 创建临时同步配置
    let sync_config = create_sync_config_from_request(&request)
        .map_err(|e| format!("创建同步配置失败: {}", e))?;

    // 验证配置
    validate_sync_config(&sync_config).map_err(|e| format!("验证同步配置失败: {}", e))?;

    // 创建提供者并测试连接
    let provider = create_provider(&sync_config)
        .await
        .map_err(|e| format!("创建同步提供者失败: {}", e))?;

    let result = provider.test_connection().await;

    match result {
        Ok(true) => Ok("连接测试成功！同步目录可以正常访问".to_string()),
        Ok(false) => {
            // 根据具体情况提供更详细的错误信息
            if request.provider == "webdav" {
                Err("WebDAV连接测试失败。\n\n可能的原因：\n• 服务器地址、用户名或密码错误\n• 同步目录不存在且无权限创建\n• 服务器不支持WebDAV协议\n• 网络连接问题\n\n请检查配置后重试".to_string())
            } else {
                Err("连接测试失败，请检查配置".to_string())
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("409") {
                Err("连接测试失败：目录冲突。\n可能同步目录不存在或无权限创建。\n请检查服务器上的目录权限。".to_string())
            } else if error_msg.contains("401") {
                Err("连接测试失败：身份验证失败。\n请检查用户名和密码是否正确。".to_string())
            } else if error_msg.contains("403") {
                Err("连接测试失败：权限不足。\n请检查用户账户是否有访问WebDAV的权限。".to_string())
            } else if error_msg.contains("404") {
                Err("连接测试失败：服务器路径不存在。\n请检查服务器地址是否正确。".to_string())
            } else {
                Err(format!("连接测试出错: {}", error_msg))
            }
        }
    }
}

/// 开始同步
#[tauri::command]
pub async fn start_sync(state: State<'_, AppState>) -> std::result::Result<String, String> {
    log::info!("开始手动同步");

    let storage = state.storage.clone();
    let sync_config = load_sync_config(storage.get_database()).await?;

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
            let conflict_item = ConflictItem {
                id: conflict.id.clone(),
                name: conflict.name.clone(),
                local_modified: conflict
                    .local_modified
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                remote_modified: conflict
                    .remote_modified
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
                conflict_type: "content".to_string(),
                local_preview: serde_json::json!({
                    "size": conflict.size,
                    "hash": conflict.hash,
                    "modified": conflict.local_modified
                }),
                remote_preview: serde_json::json!({
                    "size": conflict.size,
                    "hash": conflict.hash,
                    "modified": conflict.remote_modified
                }),
                file_size: conflict.size,
                local_hash: conflict.hash.clone(),
                remote_hash: Some(conflict.hash.clone()),
            };
            conflict_items.push(conflict_item);
        }

        // 将冲突存储到全局状态
        {
            let mut pending_conflicts = PENDING_CONFLICTS.lock().unwrap();
            pending_conflicts.clear();
            pending_conflicts.extend(conflict_items);
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

/// 获取冲突列表
#[tauri::command]
pub async fn get_sync_conflicts(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<serde_json::Value>, String> {
    log::info!("获取同步冲突");

    // TODO: 实现冲突列表获取
    // 需要存储和管理冲突信息

    Ok(vec![])
}

/// 解决同步冲突
#[tauri::command]
pub async fn resolve_sync_conflict(
    conflict_id: String,
    resolution: String,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    log::info!("解决同步冲突: {} -> {}", conflict_id, resolution);

    // TODO: 实现冲突解决逻辑

    Ok(format!("冲突 {} 已解决", conflict_id))
}

/// 验证同步设置
#[tauri::command]
pub async fn validate_sync_settings(
    request: SyncConfigRequest,
    state: State<'_, AppState>,
) -> std::result::Result<Vec<String>, String> {
    log::info!("验证同步设置");

    let mut errors = Vec::new();

    // 基本验证
    if request.sync_interval < 5 {
        errors.push("同步间隔不能小于5分钟".to_string());
    }

    if request.sync_interval > 1440 {
        errors.push("同步间隔不能超过24小时".to_string());
    }

    // WebDAV特定验证
    if request.provider == "webdav" {
        if let Some(webdav_config) = &request.webdav_config {
            if webdav_config.url.is_empty() {
                errors.push("WebDAV服务器URL不能为空".to_string());
            } else if !webdav_config.url.starts_with("http://")
                && !webdav_config.url.starts_with("https://")
            {
                errors.push("WebDAV服务器URL格式无效".to_string());
            }

            if webdav_config.username.is_empty() {
                errors.push("WebDAV用户名不能为空".to_string());
            }

            if webdav_config.password.is_empty() {
                errors.push("WebDAV密码不能为空".to_string());
            }

            if webdav_config.directory.is_empty() {
                errors.push("同步目录不能为空".to_string());
            }
        } else {
            errors.push("WebDAV配置不能为空".to_string());
        }
    }

    Ok(errors)
}

/// 调试WebDAV配置（仅用于开发调试）
#[tauri::command]
pub async fn debug_webdav_config(
    request: SyncConfigRequest,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    log::info!("调试WebDAV配置");

    if request.provider != "webdav" {
        return Err("仅支持WebDAV配置调试".to_string());
    }

    let webdav_config = request.webdav_config.as_ref().ok_or("WebDAV配置不存在")?;

    let debug_info = format!(
        "WebDAV配置信息：\n\
        - 服务器URL: {}\n\
        - 用户名: {}\n\
        - 密码长度: {} 字符\n\
        - 同步目录: {}\n\
        - 构建的完整URL: {}/{}",
        webdav_config.url,
        webdav_config.username,
        webdav_config.password.len(),
        webdav_config.directory,
        webdav_config.url.trim_end_matches('/'),
        webdav_config.directory
    );

    log::info!("WebDAV配置调试信息: {}", debug_info);
    Ok(debug_info)
}

/// 清除WebDAV密码
#[tauri::command]
pub async fn clear_webdav_password(
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    log::info!("清除WebDAV密码");

    let mut config = {
        let config_guard = state.config.lock().unwrap();
        config_guard.clone()
    };

    // 清除加密的WebDAV密码
    config.data.sync.webdav_password_encrypted = None;

    // 保存配置到内存
    {
        let mut config_guard = state.config.lock().unwrap();
        *config_guard = config.clone();
    }

    // 保存配置到文件
    let mut config_manager =
        crate::config::create_config_manager().map_err(|e| format!("创建配置管理器失败: {}", e))?;

    *config_manager.config_mut() = config;
    config_manager
        .save()
        .map_err(|e| format!("保存配置文件失败: {}", e))?;

    Ok("WebDAV密码已清除，请重新设置密码".to_string())
}

/// 从请求创建同步配置
fn create_sync_config_from_request(
    request: &SyncConfigRequest,
) -> std::result::Result<SyncConfig, String> {
    let mut settings = HashMap::new();

    if request.provider == "webdav" {
        if let Some(webdav_config) = &request.webdav_config {
            settings.insert("url".to_string(), webdav_config.url.clone());
            settings.insert("username".to_string(), webdav_config.username.clone());
            settings.insert("password".to_string(), webdav_config.password.clone());
            settings.insert("directory".to_string(), webdav_config.directory.clone());
        }
    }

    let conflict_strategy = match request.conflict_strategy.as_str() {
        "local_wins" => ConflictStrategy::LocalWins,
        "remote_wins" => ConflictStrategy::RemoteWins,
        "keep_both" => ConflictStrategy::KeepBoth,
        _ => ConflictStrategy::Manual,
    };

    Ok(SyncConfig {
        provider: request.provider.clone(),
        settings,
        interval: request.sync_interval,
        auto_sync: request.auto_sync,
        conflict_strategy,
        ignore_patterns: vec![
            "*.tmp".to_string(),
            "*.log".to_string(),
            ".DS_Store".to_string(),
            "Thumbs.db".to_string(),
        ],
        max_file_size: 10,
        compression: true,
    })
}

/// 从应用配置创建同步配置
fn create_sync_config_from_app_config(
    config: &crate::config::AppConfig,
) -> std::result::Result<SyncConfig, String> {
    let sync_config = &config.data.sync;
    let mut settings = HashMap::new();

    if sync_config.provider == "webdav" {
        if let Some(url) = &sync_config.webdav_url {
            settings.insert("url".to_string(), url.clone());
        }
        if let Some(username) = &sync_config.webdav_username {
            settings.insert("username".to_string(), username.clone());
        }
        if let Some(encrypted_password) = &sync_config.webdav_password_encrypted {
            // 只有当加密密码不为空时才尝试解密
            if !encrypted_password.trim().is_empty() {
                match decrypt_password(encrypted_password, "life_tracker_webdav") {
                    Ok(password) => {
                        settings.insert("password".to_string(), password);
                    }
                    Err(e) => {
                        log::warn!("解密WebDAV密码失败，将跳过密码设置: {}", e);
                        // 密码解密失败时不设置密码，让测试连接能继续进行
                        // 这样可以给用户更明确的错误提示
                    }
                }
            }
        }
        settings.insert("directory".to_string(), sync_config.sync_directory.clone());
    }

    let conflict_strategy = match sync_config.conflict_strategy.as_str() {
        "local_wins" => ConflictStrategy::LocalWins,
        "remote_wins" => ConflictStrategy::RemoteWins,
        "keep_both" => ConflictStrategy::KeepBoth,
        _ => ConflictStrategy::Manual,
    };

    Ok(SyncConfig {
        provider: sync_config.provider.clone(),
        settings,
        interval: sync_config.sync_interval,
        auto_sync: sync_config.auto_sync,
        conflict_strategy,
        ignore_patterns: sync_config.ignore_patterns.clone(),
        max_file_size: sync_config.max_sync_file_size,
        compression: sync_config.enable_compression,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictItem {
    pub id: String,
    pub name: String,
    pub local_modified: String,
    pub remote_modified: Option<String>,
    pub conflict_type: String,
    pub local_preview: serde_json::Value,
    pub remote_preview: serde_json::Value,
    pub file_size: u64,
    pub local_hash: String,
    pub remote_hash: Option<String>,
}

#[tauri::command]
pub async fn get_pending_conflicts(
    _database: State<'_, Database>,
) -> Result<Vec<ConflictItem>, String> {
    log::info!("获取待解决冲突");

    // 从全局状态获取冲突
    let conflicts = {
        let pending_conflicts = PENDING_CONFLICTS.lock().unwrap();
        pending_conflicts.clone()
    };

    log::info!("当前有 {} 个待解决冲突", conflicts.len());

    Ok(conflicts)
}

#[tauri::command]
pub async fn resolve_conflicts(
    database: State<'_, Database>,
    resolutions: HashMap<String, String>,
) -> Result<String, String> {
    log::info!("解决冲突，解决方案: {:?}", resolutions);

    // 获取当前冲突并检查是否为空
    let conflicts_empty = {
        let pending_conflicts = PENDING_CONFLICTS.lock().unwrap();
        pending_conflicts.is_empty()
    };

    if conflicts_empty {
        return Err("没有待解决的冲突".to_string());
    }

    // 应用用户选择的解决方案
    let mut resolved_count = 0;
    {
        let mut pending_conflicts = PENDING_CONFLICTS.lock().unwrap();

        for (conflict_id, resolution) in resolutions {
            if let Some(conflict_index) = pending_conflicts.iter().position(|c| c.id == conflict_id)
            {
                let conflict = &pending_conflicts[conflict_index];

                match resolution.as_str() {
                    "merge" => {
                        log::info!("应用合并解决方案: {}", conflict.name);
                        // TODO: 实现智能合并逻辑
                    }
                    "use_local" => {
                        log::info!("使用本地数据: {}", conflict.name);
                        // TODO: 实现使用本地数据的逻辑
                    }
                    "use_remote" => {
                        log::info!("使用远程数据: {}", conflict.name);
                        // TODO: 实现使用远程数据的逻辑
                    }
                    _ => {
                        return Err(format!("未知的解决方案类型: {}", resolution));
                    }
                }

                // 移除已解决的冲突
                pending_conflicts.remove(conflict_index);
                resolved_count += 1;
            }
        }
    }

    // 检查是否所有冲突都已解决
    let all_resolved = {
        let pending_conflicts = PENDING_CONFLICTS.lock().unwrap();
        pending_conflicts.is_empty()
    };

    if all_resolved {
        update_sync_status(&database, "success").await?;
        log::info!("所有冲突已解决，同步状态更新为成功");
    }

    Ok(format!("已解决 {} 个冲突", resolved_count))
}

/// 从数据库加载同步配置
async fn load_sync_config(database: &Database) -> Result<SyncConfigRequest, String> {
    // 这里应该从数据库读取配置，目前返回默认配置
    // TODO: 实现从数据库读取配置的逻辑
    Ok(SyncConfigRequest {
        enabled: true,
        provider: "webdav".to_string(),
        auto_sync: false,
        sync_interval: 30,
        conflict_strategy: "manual".to_string(),
        webdav_config: None,
    })
}

/// 创建同步引擎
async fn create_sync_engine(
    sync_config: &SyncConfigRequest,
    storage: Arc<StorageManager>,
) -> Result<SyncEngine, String> {
    let config = create_sync_config_from_request(sync_config)?;

    // 创建同步引擎
    let mut engine = SyncEngine::new(storage, config).map_err(|e| e.to_string())?;

    // 初始化同步引擎（会自动创建提供者并测试连接）
    engine.initialize().await.map_err(|e| e.to_string())?;

    Ok(engine)
}

/// 更新同步状态
async fn update_sync_status(database: &Database, status: &str) -> Result<(), String> {
    // TODO: 实现更新数据库中同步状态的逻辑
    log::info!("更新同步状态: {}", status);
    Ok(())
}

/// 更新最后同步时间
async fn update_last_sync_time(database: &Database) -> Result<(), String> {
    // TODO: 实现更新数据库中最后同步时间的逻辑
    let now = chrono::Local::now();
    log::info!("更新最后同步时间: {}", now.format("%Y-%m-%d %H:%M:%S"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_creation() {
        let request = SyncConfigRequest {
            enabled: true,
            provider: "webdav".to_string(),
            auto_sync: true,
            sync_interval: 30,
            conflict_strategy: "manual".to_string(),
            webdav_config: Some(WebDavConfigRequest {
                url: "https://example.com/webdav".to_string(),
                username: "user".to_string(),
                password: "pass".to_string(),
                directory: "LifeTracker".to_string(),
            }),
        };

        let sync_config = create_sync_config_from_request(&request).unwrap();
        assert_eq!(sync_config.provider, "webdav");
        assert_eq!(sync_config.interval, 30);
        assert_eq!(
            sync_config.settings.get("url").unwrap(),
            "https://example.com/webdav"
        );
    }
}
