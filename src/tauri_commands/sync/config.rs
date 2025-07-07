//! # 同步配置相关命令
//!
//! 包含同步配置的获取、保存、测试等功能

use super::types::{SyncConfigRequest, WebDavConfigRequest};
use super::utils::create_sync_config_from_request;
use crate::sync::providers::create_provider;
use crate::sync::validate_sync_config;
use crate::tauri_commands::AppState;
use crate::utils::crypto::{decrypt_password, encrypt_password};
use std::collections::HashMap;
use tauri::State;

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
