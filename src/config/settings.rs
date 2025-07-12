//! # 设置管理模块
//!
//! 提供具体的设置项管理和操作功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ConfigManager;
use crate::Result;

/// 设置项类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingType {
    Boolean,
    Integer,
    Float,
    String,
    Path,
    Color,
    Shortcut,
    List,
}

/// 设置项定义
#[derive(Debug, Clone)]
pub struct SettingDefinition {
    pub key: String,
    pub name: String,
    pub description: String,
    pub setting_type: SettingType,
    pub default_value: SettingValue,
    pub min_value: Option<SettingValue>,
    pub max_value: Option<SettingValue>,
    pub allowed_values: Option<Vec<SettingValue>>,
    pub category: String,
    pub requires_restart: bool,
    pub validation_fn: Option<fn(&SettingValue) -> Result<()>>,
}

/// 设置值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SettingValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<String>),
}

impl SettingValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            SettingValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            SettingValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            SettingValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            SettingValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<String>> {
        match self {
            SettingValue::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            SettingValue::Boolean(_) => "boolean",
            SettingValue::Integer(_) => "integer",
            SettingValue::Float(_) => "float",
            SettingValue::String(_) => "string",
            SettingValue::List(_) => "list",
        }
    }
}

/// 设置管理器
pub struct SettingsManager {
    config_manager: ConfigManager,
    definitions: HashMap<String, SettingDefinition>,
    custom_settings: HashMap<String, SettingValue>,
}

impl SettingsManager {
    /// 创建新的设置管理器
    pub fn new(config_manager: ConfigManager) -> Self {
        let mut manager = Self {
            config_manager,
            definitions: HashMap::new(),
            custom_settings: HashMap::new(),
        };

        manager.register_default_settings();
        manager
    }

    /// 注册默认设置
    fn register_default_settings(&mut self) {
        // 常规设置
        self.register_setting(SettingDefinition {
            key: "general.language".to_string(),
            name: "语言".to_string(),
            description: "应用程序界面语言".to_string(),
            setting_type: SettingType::String,
            default_value: SettingValue::String("zh-CN".to_string()),
            allowed_values: Some(vec![
                SettingValue::String("zh-CN".to_string()),
                SettingValue::String("en-US".to_string()),
                SettingValue::String("ja-JP".to_string()),
            ]),
            category: "常规".to_string(),
            requires_restart: true,
            min_value: None,
            max_value: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "general.auto_start_timer".to_string(),
            name: "自动开始计时".to_string(),
            description: "启动应用时自动开始计时".to_string(),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Boolean(false),
            category: "常规".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            allowed_values: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "general.minimize_to_tray".to_string(),
            name: "最小化到托盘".to_string(),
            description: "关闭窗口时最小化到系统托盘".to_string(),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Boolean(true),
            category: "常规".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            allowed_values: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "general.work_reminder_interval".to_string(),
            name: "工作提醒间隔".to_string(),
            description: "工作时间提醒间隔（分钟）".to_string(),
            setting_type: SettingType::Integer,
            default_value: SettingValue::Integer(25),
            min_value: Some(SettingValue::Integer(1)),
            max_value: Some(SettingValue::Integer(480)),
            category: "常规".to_string(),
            requires_restart: false,
            allowed_values: None,
            validation_fn: None,
        });

        // 界面设置
        self.register_setting(SettingDefinition {
            key: "ui.theme".to_string(),
            name: "主题".to_string(),
            description: "应用程序主题".to_string(),
            setting_type: SettingType::String,
            default_value: SettingValue::String("default".to_string()),
            allowed_values: Some(vec![
                SettingValue::String("default".to_string()),
                SettingValue::String("dark".to_string()),
                SettingValue::String("light".to_string()),
                SettingValue::String("blue".to_string()),
                SettingValue::String("green".to_string()),
            ]),
            category: "界面".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "ui.dark_mode".to_string(),
            name: "深色模式".to_string(),
            description: "启用深色模式".to_string(),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Boolean(false),
            category: "界面".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            allowed_values: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "ui.font_size".to_string(),
            name: "字体大小".to_string(),
            description: "界面字体大小".to_string(),
            setting_type: SettingType::Float,
            default_value: SettingValue::Float(14.0),
            min_value: Some(SettingValue::Float(8.0)),
            max_value: Some(SettingValue::Float(72.0)),
            category: "界面".to_string(),
            requires_restart: false,
            allowed_values: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "ui.font_family".to_string(),
            name: "字体族".to_string(),
            description: "界面字体族".to_string(),
            setting_type: SettingType::String,
            default_value: SettingValue::String("Microsoft YaHei".to_string()),
            allowed_values: Some(vec![
                SettingValue::String("Microsoft YaHei".to_string()),
                SettingValue::String("SimHei".to_string()),
                SettingValue::String("Arial".to_string()),
                SettingValue::String("Helvetica".to_string()),
                SettingValue::String("Times New Roman".to_string()),
            ]),
            category: "界面".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "ui.enable_animations".to_string(),
            name: "动画效果".to_string(),
            description: "启用界面动画效果".to_string(),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Boolean(true),
            category: "界面".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            allowed_values: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "ui.opacity".to_string(),
            name: "透明度".to_string(),
            description: "窗口透明度".to_string(),
            setting_type: SettingType::Float,
            default_value: SettingValue::Float(1.0),
            min_value: Some(SettingValue::Float(0.1)),
            max_value: Some(SettingValue::Float(1.0)),
            category: "界面".to_string(),
            requires_restart: false,
            allowed_values: None,
            validation_fn: None,
        });

        // 通知设置
        self.register_setting(SettingDefinition {
            key: "notifications.enabled".to_string(),
            name: "启用通知".to_string(),
            description: "启用系统通知".to_string(),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Boolean(true),
            category: "通知".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            allowed_values: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "notifications.desktop_notifications".to_string(),
            name: "桌面通知".to_string(),
            description: "显示桌面通知".to_string(),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Boolean(true),
            category: "通知".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            allowed_values: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "notifications.sound_notifications".to_string(),
            name: "声音通知".to_string(),
            description: "播放通知声音".to_string(),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Boolean(false),
            category: "通知".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            allowed_values: None,
            validation_fn: None,
        });

        // 数据设置
        self.register_setting(SettingDefinition {
            key: "data.auto_backup".to_string(),
            name: "自动备份".to_string(),
            description: "自动备份数据".to_string(),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Boolean(true),
            category: "数据".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            allowed_values: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "data.backup_interval".to_string(),
            name: "备份间隔".to_string(),
            description: "自动备份间隔（天）".to_string(),
            setting_type: SettingType::Integer,
            default_value: SettingValue::Integer(7),
            min_value: Some(SettingValue::Integer(1)),
            max_value: Some(SettingValue::Integer(365)),
            category: "数据".to_string(),
            requires_restart: false,
            allowed_values: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "data.backup_retention".to_string(),
            name: "备份保留".to_string(),
            description: "备份文件保留数量".to_string(),
            setting_type: SettingType::Integer,
            default_value: SettingValue::Integer(30),
            min_value: Some(SettingValue::Integer(1)),
            max_value: Some(SettingValue::Integer(365)),
            category: "数据".to_string(),
            requires_restart: false,
            allowed_values: None,
            validation_fn: None,
        });

        // 同步设置
        self.register_setting(SettingDefinition {
            key: "sync.enabled".to_string(),
            name: "启用同步".to_string(),
            description: "启用多端数据同步".to_string(),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Boolean(false),
            category: "同步".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            allowed_values: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "sync.provider".to_string(),
            name: "同步提供者".to_string(),
            description: "选择同步服务提供者".to_string(),
            setting_type: SettingType::String,
            default_value: SettingValue::String("webdav".to_string()),
            category: "同步".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            allowed_values: Some(vec![
                SettingValue::String("webdav".to_string()),
                SettingValue::String("github".to_string()),
                SettingValue::String("local".to_string()),
            ]),
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "sync.auto_sync".to_string(),
            name: "自动同步".to_string(),
            description: "自动进行数据同步".to_string(),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Boolean(false),
            category: "同步".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            allowed_values: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "sync.sync_interval".to_string(),
            name: "同步间隔".to_string(),
            description: "自动同步间隔（分钟）".to_string(),
            setting_type: SettingType::Integer,
            default_value: SettingValue::Integer(30),
            min_value: Some(SettingValue::Integer(5)),
            max_value: Some(SettingValue::Integer(1440)),
            category: "同步".to_string(),
            requires_restart: false,
            allowed_values: None,
            validation_fn: None,
        });

        self.register_setting(SettingDefinition {
            key: "sync.conflict_strategy".to_string(),
            name: "冲突解决策略".to_string(),
            description: "数据冲突时的解决方式".to_string(),
            setting_type: SettingType::String,
            default_value: SettingValue::String("manual".to_string()),
            category: "同步".to_string(),
            requires_restart: false,
            min_value: None,
            max_value: None,
            allowed_values: Some(vec![
                SettingValue::String("manual".to_string()),
                SettingValue::String("local_wins".to_string()),
                SettingValue::String("remote_wins".to_string()),
            ]),
            validation_fn: None,
        });
    }

    /// 注册设置项
    pub fn register_setting(&mut self, definition: SettingDefinition) {
        self.definitions.insert(definition.key.clone(), definition);
    }

    /// 获取设置值
    pub fn get_setting(&self, key: &str) -> Option<SettingValue> {
        // 首先检查自定义设置
        if let Some(value) = self.custom_settings.get(key) {
            return Some(value.clone());
        }

        // 然后从配置中获取
        self.get_setting_from_config(key)
    }

    /// 设置值
    pub fn set_setting(&mut self, key: &str, value: SettingValue) -> Result<()> {
        // 验证设置项是否存在
        let definition = self
            .definitions
            .get(key)
            .ok_or_else(|| format!("未知设置项: {}", key))?;

        // 验证值类型
        self.validate_setting_value(definition, &value)?;

        // 应用到配置
        self.apply_setting_to_config(key, &value)?;

        // 保存自定义设置
        self.custom_settings.insert(key.to_string(), value);

        // 保存配置
        self.config_manager.save()?;

        Ok(())
    }

    /// 重置设置为默认值
    pub fn reset_setting(&mut self, key: &str) -> Result<()> {
        let definition = self
            .definitions
            .get(key)
            .ok_or_else(|| format!("未知设置项: {}", key))?;

        let default_value = definition.default_value.clone();
        self.set_setting(key, default_value)
    }

    /// 获取所有设置定义
    pub fn get_all_definitions(&self) -> &HashMap<String, SettingDefinition> {
        &self.definitions
    }

    /// 按分类获取设置定义
    pub fn get_definitions_by_category(&self, category: &str) -> Vec<&SettingDefinition> {
        self.definitions
            .values()
            .filter(|def| def.category == category)
            .collect()
    }

    /// 获取所有分类
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .definitions
            .values()
            .map(|def| def.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }

    /// 搜索设置
    pub fn search_settings(&self, query: &str) -> Vec<&SettingDefinition> {
        let query = query.to_lowercase();
        self.definitions
            .values()
            .filter(|def| {
                def.name.to_lowercase().contains(&query)
                    || def.description.to_lowercase().contains(&query)
                    || def.key.to_lowercase().contains(&query)
            })
            .collect()
    }

    /// 导出设置
    pub fn export_settings(&self) -> HashMap<String, SettingValue> {
        let mut settings = HashMap::new();

        for key in self.definitions.keys() {
            if let Some(value) = self.get_setting(key) {
                settings.insert(key.clone(), value);
            }
        }

        settings
    }

    /// 导入设置
    pub fn import_settings(
        &mut self,
        settings: HashMap<String, SettingValue>,
    ) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        for (key, value) in settings {
            if let Err(e) = self.set_setting(&key, value) {
                errors.push(format!("{}: {}", key, e));
            }
        }

        Ok(errors)
    }

    /// 验证所有设置
    pub fn validate_all_settings(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (key, definition) in &self.definitions {
            if let Some(value) = self.get_setting(key) {
                if let Err(e) = self.validate_setting_value(definition, &value) {
                    errors.push(format!("{}: {}", key, e));
                }
            }
        }

        errors
    }

    /// 从配置获取设置值
    fn get_setting_from_config(&self, key: &str) -> Option<SettingValue> {
        let config = self.config_manager.config();

        match key {
            "general.language" => Some(SettingValue::String(config.general.language.clone())),
            "general.auto_start_timer" => {
                Some(SettingValue::Boolean(config.general.auto_start_timer))
            }
            "general.minimize_to_tray" => {
                Some(SettingValue::Boolean(config.general.minimize_to_tray))
            }
            "general.work_reminder_interval" => config
                .general
                .work_reminder_interval
                .map(|i| SettingValue::Integer(i as i64)),
            "ui.theme" => Some(SettingValue::String(config.ui.theme.clone())),
            "ui.dark_mode" => Some(SettingValue::Boolean(config.ui.dark_mode)),
            "ui.font_size" => Some(SettingValue::Float(config.ui.font_size as f64)),
            "ui.font_family" => Some(SettingValue::String(config.ui.font_family.clone())),
            "ui.enable_animations" => Some(SettingValue::Boolean(config.ui.enable_animations)),
            "ui.opacity" => Some(SettingValue::Float(config.ui.opacity as f64)),
            "notifications.enabled" => Some(SettingValue::Boolean(config.notifications.enabled)),
            "notifications.desktop_notifications" => Some(SettingValue::Boolean(
                config.notifications.desktop_notifications,
            )),
            "notifications.sound_notifications" => Some(SettingValue::Boolean(
                config.notifications.sound_notifications,
            )),
            "data.auto_backup" => Some(SettingValue::Boolean(config.data.auto_backup)),
            "data.backup_interval" => {
                Some(SettingValue::Integer(config.data.backup_interval as i64))
            }
            "data.backup_retention" => {
                Some(SettingValue::Integer(config.data.backup_retention as i64))
            }
            "sync.enabled" => Some(SettingValue::Boolean(config.data.sync.enabled)),
            "sync.provider" => Some(SettingValue::String(config.data.sync.provider.clone())),
            "sync.auto_sync" => Some(SettingValue::Boolean(config.data.sync.auto_sync)),
            "sync.sync_interval" => {
                Some(SettingValue::Integer(config.data.sync.sync_interval as i64))
            }
            "sync.conflict_strategy" => Some(SettingValue::String(
                config.data.sync.conflict_strategy.clone(),
            )),
            _ => None,
        }
    }

    /// 应用设置到配置
    fn apply_setting_to_config(&mut self, key: &str, value: &SettingValue) -> Result<()> {
        let config = self.config_manager.config_mut();

        match key {
            "general.language" => {
                if let Some(s) = value.as_string() {
                    config.general.language = s.clone();
                }
            }
            "general.auto_start_timer" => {
                if let Some(b) = value.as_bool() {
                    config.general.auto_start_timer = b;
                }
            }
            "general.minimize_to_tray" => {
                if let Some(b) = value.as_bool() {
                    config.general.minimize_to_tray = b;
                }
            }
            "general.work_reminder_interval" => {
                if let Some(i) = value.as_i64() {
                    config.general.work_reminder_interval = Some(i as u32);
                }
            }
            "ui.theme" => {
                if let Some(s) = value.as_string() {
                    config.ui.theme = s.clone();
                }
            }
            "ui.dark_mode" => {
                if let Some(b) = value.as_bool() {
                    config.ui.dark_mode = b;
                }
            }
            "ui.font_size" => {
                if let Some(f) = value.as_f64() {
                    config.ui.font_size = f as f32;
                }
            }
            "ui.font_family" => {
                if let Some(s) = value.as_string() {
                    config.ui.font_family = s.clone();
                }
            }
            "ui.enable_animations" => {
                if let Some(b) = value.as_bool() {
                    config.ui.enable_animations = b;
                }
            }
            "ui.opacity" => {
                if let Some(f) = value.as_f64() {
                    config.ui.opacity = f as f32;
                }
            }
            "notifications.enabled" => {
                if let Some(b) = value.as_bool() {
                    config.notifications.enabled = b;
                }
            }
            "notifications.desktop_notifications" => {
                if let Some(b) = value.as_bool() {
                    config.notifications.desktop_notifications = b;
                }
            }
            "notifications.sound_notifications" => {
                if let Some(b) = value.as_bool() {
                    config.notifications.sound_notifications = b;
                }
            }
            "data.auto_backup" => {
                if let Some(b) = value.as_bool() {
                    config.data.auto_backup = b;
                }
            }
            "data.backup_interval" => {
                if let Some(i) = value.as_i64() {
                    config.data.backup_interval = i as u32;
                }
            }
            "data.backup_retention" => {
                if let Some(i) = value.as_i64() {
                    config.data.backup_retention = i as u32;
                }
            }
            "sync.enabled" => {
                if let Some(b) = value.as_bool() {
                    config.data.sync.enabled = b;
                }
            }
            "sync.provider" => {
                if let Some(s) = value.as_string() {
                    config.data.sync.provider = s.clone();
                }
            }
            "sync.auto_sync" => {
                if let Some(b) = value.as_bool() {
                    config.data.sync.auto_sync = b;
                }
            }
            "sync.sync_interval" => {
                if let Some(i) = value.as_i64() {
                    config.data.sync.sync_interval = i as u32;
                }
            }
            "sync.conflict_strategy" => {
                if let Some(s) = value.as_string() {
                    config.data.sync.conflict_strategy = s.clone();
                }
            }
            _ => return Err(format!("不支持的设置项: {}", key).into()),
        }

        Ok(())
    }

    /// 验证设置值
    fn validate_setting_value(
        &self,
        definition: &SettingDefinition,
        value: &SettingValue,
    ) -> Result<()> {
        // 检查类型匹配
        let expected_type = match definition.setting_type {
            SettingType::Boolean => "boolean",
            SettingType::Integer => "integer",
            SettingType::Float => "float",
            SettingType::String
            | SettingType::Path
            | SettingType::Color
            | SettingType::Shortcut => "string",
            SettingType::List => "list",
        };

        if value.type_name() != expected_type {
            return Err(format!(
                "类型不匹配: 期望 {}, 实际 {}",
                expected_type,
                value.type_name()
            )
            .into());
        }

        // 检查范围
        if let (Some(min), Some(max)) = (&definition.min_value, &definition.max_value) {
            match (value, min, max) {
                (
                    SettingValue::Integer(v),
                    SettingValue::Integer(min_v),
                    SettingValue::Integer(max_v),
                ) => {
                    if v < min_v || v > max_v {
                        return Err(
                            format!("值超出范围: {} 不在 [{}, {}] 内", v, min_v, max_v).into()
                        );
                    }
                }
                (
                    SettingValue::Float(v),
                    SettingValue::Float(min_v),
                    SettingValue::Float(max_v),
                ) => {
                    if v < min_v || v > max_v {
                        return Err(
                            format!("值超出范围: {} 不在 [{}, {}] 内", v, min_v, max_v).into()
                        );
                    }
                }
                _ => {}
            }
        }

        // 检查允许的值
        if let Some(allowed) = &definition.allowed_values {
            if !allowed.contains(value) {
                return Err(format!("值不在允许列表中: {:?}", value).into());
            }
        }

        // 自定义验证
        if let Some(validation_fn) = definition.validation_fn {
            validation_fn(value)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_settings_manager() -> SettingsManager {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        let config_manager = ConfigManager::new(config_path).unwrap();
        SettingsManager::new(config_manager)
    }

    #[test]
    fn test_setting_value_types() {
        let bool_val = SettingValue::Boolean(true);
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(bool_val.type_name(), "boolean");

        let int_val = SettingValue::Integer(42);
        assert_eq!(int_val.as_i64(), Some(42));
        assert_eq!(int_val.type_name(), "integer");

        let float_val = SettingValue::Float(3.14);
        assert_eq!(float_val.as_f64(), Some(3.14));
        assert_eq!(float_val.type_name(), "float");

        let string_val = SettingValue::String("test".to_string());
        assert_eq!(string_val.as_string(), Some(&"test".to_string()));
        assert_eq!(string_val.type_name(), "string");
    }

    #[test]
    fn test_settings_manager() {
        let mut manager = create_test_settings_manager();

        // 测试获取默认值
        let language = manager.get_setting("general.language").unwrap();
        assert_eq!(language.as_string(), Some(&"zh-CN".to_string()));

        // 测试设置值
        manager
            .set_setting(
                "general.language",
                SettingValue::String("en-US".to_string()),
            )
            .unwrap();
        let language = manager.get_setting("general.language").unwrap();
        assert_eq!(language.as_string(), Some(&"en-US".to_string()));

        // 测试无效值
        let result = manager.set_setting(
            "general.language",
            SettingValue::String("invalid".to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_setting_validation() {
        let mut manager = create_test_settings_manager();

        // 测试类型验证
        let result = manager.set_setting("ui.dark_mode", SettingValue::String("true".to_string()));
        assert!(result.is_err());

        // 测试范围验证
        let result = manager.set_setting("ui.font_size", SettingValue::Float(100.0));
        assert!(result.is_err());

        // 测试有效值
        let result = manager.set_setting("ui.font_size", SettingValue::Float(16.0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_categories_and_search() {
        let manager = create_test_settings_manager();

        let categories = manager.get_categories();
        assert!(categories.contains(&"常规".to_string()));
        assert!(categories.contains(&"界面".to_string()));

        let general_settings = manager.get_definitions_by_category("常规");
        assert!(!general_settings.is_empty());

        let search_results = manager.search_settings("字体");
        assert!(!search_results.is_empty());
    }

    #[test]
    fn test_export_import() {
        let mut manager = create_test_settings_manager();

        // 修改一些设置
        manager
            .set_setting("ui.dark_mode", SettingValue::Boolean(true))
            .unwrap();
        manager
            .set_setting("ui.font_size", SettingValue::Float(16.0))
            .unwrap();

        // 导出设置
        let exported = manager.export_settings();
        assert!(!exported.is_empty());

        // 重置设置
        manager.reset_setting("ui.dark_mode").unwrap();
        assert_eq!(
            manager.get_setting("ui.dark_mode").unwrap().as_bool(),
            Some(false)
        );

        // 导入设置
        let errors = manager.import_settings(exported).unwrap();
        assert!(errors.is_empty());
        assert_eq!(
            manager.get_setting("ui.dark_mode").unwrap().as_bool(),
            Some(true)
        );
    }
}
