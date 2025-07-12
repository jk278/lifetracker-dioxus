//! # 同步提供者模块
//!
//! 提供不同的同步服务实现，包括 WebDAV、GitHub、本地网络等

pub mod webdav;

use crate::errors::{AppError, Result};
use crate::sync::{SyncConfig, SyncProvider};
use std::sync::Arc;

/// 同步提供者的动态类型
pub type SyncProviderBox = Box<dyn SyncProvider>;

/// 创建同步提供者的工厂函数
pub async fn create_provider(config: &SyncConfig) -> Result<SyncProviderBox> {
    match config.provider.as_str() {
        "webdav" => {
            let provider = webdav::WebDavProvider::new(config).await?;
            Ok(Box::new(provider))
        }
        "github" => {
            // TODO: 实现 GitHub 提供者
            Err(AppError::Sync("GitHub 提供者尚未实现".to_string()))
        }
        "local" => {
            // TODO: 实现本地网络提供者
            Err(AppError::Sync("本地网络提供者尚未实现".to_string()))
        }
        _ => Err(AppError::Sync(format!(
            "不支持的同步提供者: {}",
            config.provider
        ))),
    }
}

/// 获取支持的同步提供者列表
pub fn get_supported_providers() -> Vec<&'static str> {
    vec!["webdav"]
}

/// 验证提供者配置
pub fn validate_provider_config(config: &SyncConfig) -> Result<()> {
    match config.provider.as_str() {
        "webdav" => webdav::validate_config(config),
        "github" => Err(AppError::Sync("GitHub 提供者尚未实现".to_string())),
        "local" => Err(AppError::Sync("本地网络提供者尚未实现".to_string())),
        _ => Err(AppError::Sync(format!(
            "不支持的同步提供者: {}",
            config.provider
        ))),
    }
}
