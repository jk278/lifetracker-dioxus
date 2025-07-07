//! # 同步模块
//!
//! 同步功能相关的 Tauri 命令

pub mod config;
pub mod conflicts;
pub mod history;
pub mod operations;
pub mod types;
pub mod utils;

// 导出所有类型
pub use types::*;

// 导出所有命令
pub use config::*;
pub use conflicts::*;
pub use history::*;
pub use operations::*;

// 导出辅助函数
pub use utils::*;

// 单元测试
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
