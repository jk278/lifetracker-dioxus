//! # 配置管理模块
//!
//! 提供应用程序配置的加载、保存和管理功能

pub mod settings;
pub mod theme;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::errors::Result;

/// 应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 常规设置
    pub general: GeneralConfig,
    /// 界面设置
    pub ui: UiConfig,
    /// 通知设置
    pub notifications: NotificationConfig,
    /// 数据设置
    pub data: DataConfig,
    /// 快捷键设置
    pub shortcuts: ShortcutConfig,
    /// 高级设置
    pub advanced: AdvancedConfig,
    /// 配置版本
    pub version: String,
    /// 最后更新时间
    pub last_updated: DateTime<Local>,
}

/// 常规配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// 应用程序语言
    pub language: String,
    /// 启动时自动开始计时
    pub auto_start_timer: bool,
    /// 最小化到系统托盘
    pub minimize_to_tray: bool,
    /// 开机自启动
    pub auto_start: bool,
    /// 默认任务名称
    pub default_task_name: Option<String>,
    /// 默认分类ID
    pub default_category_id: Option<String>,
    /// 工作时间提醒间隔（分钟）
    pub work_reminder_interval: Option<u32>,
    /// 休息时间提醒间隔（分钟）
    pub break_reminder_interval: Option<u32>,
}

/// 界面配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// 主题名称
    pub theme: String,
    /// 深色模式
    pub dark_mode: bool,
    /// 字体大小
    pub font_size: f32,
    /// 字体族
    pub font_family: String,
    /// 窗口大小
    pub window_size: (f32, f32),
    /// 窗口位置
    pub window_position: Option<(f32, f32)>,
    /// 窗口最大化
    pub window_maximized: bool,
    /// 显示侧边栏
    pub show_sidebar: bool,
    /// 显示状态栏
    pub show_status_bar: bool,
    /// 显示工具栏
    pub show_toolbar: bool,
    /// 动画效果
    pub enable_animations: bool,
    /// 透明度
    pub opacity: f32,
}

/// 通知配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// 启用通知
    pub enabled: bool,
    /// 桌面通知
    pub desktop_notifications: bool,
    /// 声音通知
    pub sound_notifications: bool,
    /// 通知声音文件路径
    pub sound_file: Option<PathBuf>,
    /// 任务开始通知
    pub notify_task_start: bool,
    /// 任务结束通知
    pub notify_task_end: bool,
    /// 工作时间提醒
    pub notify_work_time: bool,
    /// 休息时间提醒
    pub notify_break_time: bool,
    /// 通知持续时间（秒）
    pub notification_duration: u32,
}

/// 数据配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    /// 数据库文件路径
    pub database_path: PathBuf,
    /// 自动备份
    pub auto_backup: bool,
    /// 备份间隔（天）
    pub backup_interval: u32,
    /// 备份保留数量
    pub backup_retention: u32,
    /// 备份目录
    pub backup_directory: PathBuf,
    /// 数据导出格式
    pub export_format: String,
    /// 自动清理旧数据
    pub auto_cleanup: bool,
    /// 数据保留天数
    pub data_retention_days: Option<u32>,
    /// 同步配置
    pub sync: SyncConfig,
}

/// 同步配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// 启用同步
    pub enabled: bool,
    /// 同步提供者 ("webdav", "github", "local")
    pub provider: String,
    /// 自动同步
    pub auto_sync: bool,
    /// 同步间隔（分钟）
    pub sync_interval: u32,
    /// WebDAV 服务器URL
    pub webdav_url: Option<String>,
    /// WebDAV 用户名
    pub webdav_username: Option<String>,
    /// WebDAV 密码（加密存储）
    pub webdav_password_encrypted: Option<String>,
    /// 同步目录路径
    pub sync_directory: String,
    /// 最后同步时间
    pub last_sync_time: Option<DateTime<Local>>,
    /// 同步时忽略的文件类型
    pub ignore_patterns: Vec<String>,
    /// 冲突解决策略 ("manual", "local_wins", "remote_wins")
    pub conflict_strategy: String,
    /// 启用数据压缩
    pub enable_compression: bool,
    /// 最大同步文件大小（MB）
    pub max_sync_file_size: u32,
}

/// 快捷键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    /// 开始/暂停计时
    pub start_pause_timer: String,
    /// 停止计时
    pub stop_timer: String,
    /// 新建任务
    pub new_task: String,
    /// 新建分类
    pub new_category: String,
    /// 打开设置
    pub open_settings: String,
    /// 显示/隐藏窗口
    pub toggle_window: String,
    /// 退出应用
    pub quit_app: String,
    /// 刷新数据
    pub refresh_data: String,
    /// 导出数据
    pub export_data: String,
    /// 导入数据
    pub import_data: String,
}

/// 高级配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    /// 调试模式
    pub debug_mode: bool,
    /// 日志级别
    pub log_level: String,
    /// 日志文件路径
    pub log_file: Option<PathBuf>,
    /// 性能监控
    pub performance_monitoring: bool,
    /// 数据库连接池大小
    pub db_pool_size: u32,
    /// 数据库查询超时（秒）
    pub db_query_timeout: u32,
    /// 网络超时（秒）
    pub network_timeout: u32,
    /// 启用实验性功能
    pub enable_experimental_features: bool,
    /// 自定义CSS
    pub custom_css: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ui: UiConfig::default(),
            notifications: NotificationConfig::default(),
            data: DataConfig::default(),
            shortcuts: ShortcutConfig::default(),
            advanced: AdvancedConfig::default(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            last_updated: Local::now(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(),
            auto_start_timer: false,
            minimize_to_tray: true,
            auto_start: false,
            default_task_name: None,
            default_category_id: None,
            work_reminder_interval: Some(25), // 番茄工作法
            break_reminder_interval: Some(5),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            dark_mode: false,
            font_size: 14.0,
            font_family: "Microsoft YaHei".to_string(),
            window_size: (1200.0, 800.0),
            window_position: None,
            window_maximized: false,
            show_sidebar: true,
            show_status_bar: true,
            show_toolbar: true,
            enable_animations: true,
            opacity: 1.0,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            desktop_notifications: true,
            sound_notifications: false,
            sound_file: None,
            notify_task_start: true,
            notify_task_end: true,
            notify_work_time: true,
            notify_break_time: true,
            notification_duration: 5,
        }
    }
}

impl Default for DataConfig {
    fn default() -> Self {
        let app_dir = crate::utils::get_app_data_dir().unwrap_or_else(|_| PathBuf::from("."));

        Self {
            database_path: app_dir.join("timetracker.db"),
            auto_backup: true,
            backup_interval: 7,
            backup_retention: 30,
            backup_directory: app_dir.join("backups"),
            export_format: "json".to_string(),
            auto_cleanup: false,
            data_retention_days: None,
            sync: SyncConfig::default(),
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "webdav".to_string(),
            auto_sync: false,
            sync_interval: 30, // 30分钟
            webdav_url: None,
            webdav_username: None,
            webdav_password_encrypted: None,
            sync_directory: "LifeTracker".to_string(),
            last_sync_time: None,
            ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.log".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
            ],
            conflict_strategy: "manual".to_string(),
            enable_compression: true,
            max_sync_file_size: 10, // 10MB
        }
    }
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            start_pause_timer: "Ctrl+Space".to_string(),
            stop_timer: "Ctrl+Shift+Space".to_string(),
            new_task: "Ctrl+N".to_string(),
            new_category: "Ctrl+Shift+N".to_string(),
            open_settings: "Ctrl+,".to_string(),
            toggle_window: "Ctrl+Shift+T".to_string(),
            quit_app: "Ctrl+Q".to_string(),
            refresh_data: "F5".to_string(),
            export_data: "Ctrl+E".to_string(),
            import_data: "Ctrl+I".to_string(),
        }
    }
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            debug_mode: false,
            log_level: "info".to_string(),
            log_file: None,
            performance_monitoring: false,
            db_pool_size: 10,
            db_query_timeout: 30,
            network_timeout: 10,
            enable_experimental_features: false,
            custom_css: None,
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let config = if config_path.exists() {
            Self::load_config(&config_path)?
        } else {
            AppConfig::default()
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    /// 获取配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 获取可变配置
    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// 保存配置
    pub fn save(&mut self) -> Result<()> {
        self.config.last_updated = Local::now();
        Self::save_config(&self.config_path, &self.config)
    }

    /// 重新加载配置
    pub fn reload(&mut self) -> Result<()> {
        self.config = Self::load_config(&self.config_path)?;
        Ok(())
    }

    /// 重置为默认配置
    pub fn reset_to_default(&mut self) -> Result<()> {
        self.config = AppConfig::default();
        Self::save_config(&self.config_path, &self.config)
    }

    /// 导出配置到指定路径
    pub fn export_config<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        Self::save_config(path.as_ref(), &self.config)
    }

    /// 导入配置
    pub fn import_config<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        self.config = Self::load_config(path.as_ref())?;
        self.save()
    }

    /// 从文件加载配置
    fn load_config(path: &std::path::Path) -> Result<AppConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig =
            toml::from_str(&content).map_err(|e| format!("配置文件解析错误: {}", e))?;
        Ok(config)
    }

    /// 保存配置到文件
    fn save_config(path: &std::path::Path, config: &AppConfig) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content =
            toml::to_string_pretty(config).map_err(|e| format!("配置序列化错误: {}", e))?;

        std::fs::write(path, content)?;
        Ok(())
    }

    /// 验证配置
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // 验证窗口大小
        if self.config.ui.window_size.0 < 400.0 || self.config.ui.window_size.1 < 300.0 {
            errors.push("窗口大小过小".to_string());
        }

        // 验证字体大小
        if self.config.ui.font_size < 8.0 || self.config.ui.font_size > 72.0 {
            errors.push("字体大小超出范围".to_string());
        }

        // 验证透明度
        if self.config.ui.opacity < 0.1 || self.config.ui.opacity > 1.0 {
            errors.push("透明度超出范围".to_string());
        }

        // 验证备份设置
        if self.config.data.auto_backup {
            if self.config.data.backup_interval == 0 {
                errors.push("备份间隔不能为0".to_string());
            }
            if self.config.data.backup_retention == 0 {
                errors.push("备份保留数量不能为0".to_string());
            }
        }

        // 验证提醒间隔
        if let Some(interval) = self.config.general.work_reminder_interval {
            if interval == 0 || interval > 480 {
                // 最大8小时
                errors.push("工作提醒间隔超出范围".to_string());
            }
        }

        if let Some(interval) = self.config.general.break_reminder_interval {
            if interval == 0 || interval > 60 {
                // 最大1小时
                errors.push("休息提醒间隔超出范围".to_string());
            }
        }

        // 验证通知持续时间
        if self.config.notifications.notification_duration == 0
            || self.config.notifications.notification_duration > 60
        {
            errors.push("通知持续时间超出范围".to_string());
        }

        errors
    }

    /// 获取配置文件路径
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

/// 获取默认配置文件路径
pub fn get_default_config_path() -> Result<PathBuf> {
    let app_dir = crate::utils::get_app_data_dir()?;
    Ok(app_dir.join("config.toml"))
}

/// 创建默认配置管理器
pub fn create_config_manager() -> Result<ConfigManager> {
    let config_path = get_default_config_path()?;
    ConfigManager::new(config_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.general.language, "zh-CN");
        assert_eq!(config.ui.theme, "default");
        assert!(config.notifications.enabled);
        assert!(config.data.auto_backup);
    }

    #[test]
    fn test_config_manager() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let mut manager = ConfigManager::new(config_path.clone()).unwrap();

        // 修改配置
        manager.config_mut().general.language = "en-US".to_string();
        manager.config_mut().ui.dark_mode = true;

        // 保存配置
        manager.save().unwrap();

        // 重新加载
        let manager2 = ConfigManager::new(config_path).unwrap();
        assert_eq!(manager2.config().general.language, "en-US");
        assert!(manager2.config().ui.dark_mode);
    }

    #[test]
    fn test_config_validation() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let mut manager = ConfigManager::new(config_path).unwrap();

        // 设置无效值
        manager.config_mut().ui.window_size = (100.0, 100.0); // 太小
        manager.config_mut().ui.font_size = 100.0; // 太大
        manager.config_mut().ui.opacity = 2.0; // 超出范围

        let errors = manager.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("窗口大小")));
        assert!(errors.iter().any(|e| e.contains("字体大小")));
        assert!(errors.iter().any(|e| e.contains("透明度")));
    }

    #[test]
    fn test_export_import_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        let export_path = temp_dir.path().join("exported_config.toml");

        let mut manager = ConfigManager::new(config_path).unwrap();
        manager.config_mut().general.language = "ja-JP".to_string();

        // 导出配置
        manager.export_config(&export_path).unwrap();
        assert!(export_path.exists());

        // 修改配置
        manager.config_mut().general.language = "ko-KR".to_string();

        // 导入配置
        manager.import_config(&export_path).unwrap();
        assert_eq!(manager.config().general.language, "ja-JP");
    }
}
