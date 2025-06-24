//! # GUI模块
//!
//! 基于egui的图形用户界面实现

mod app;
mod theme;
mod views;

pub use app::TimeTrackerApp;
pub use theme::Theme;

use crate::core::AppCore;
use crate::errors::{AppError, Result};
use crate::storage::{DatabaseConfig, StorageManager};
use eframe::egui;
use std::sync::{Arc, Mutex};

/// GUI应用程序状态
#[derive(Debug, Clone)]
pub struct AppState {
    /// 应用核心
    pub core: Arc<Mutex<AppCore>>,
    /// 存储管理器
    pub storage: Arc<Mutex<StorageManager>>,
    /// 当前主题
    pub theme: Theme,
    /// 是否显示调试信息
    pub show_debug: bool,
    /// 窗口配置
    pub window_config: WindowConfig,
}

/// 窗口配置
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// 窗口标题
    pub title: String,
    /// 初始窗口大小
    pub initial_size: egui::Vec2,
    /// 最小窗口大小
    pub min_size: egui::Vec2,
    /// 是否可调整大小
    pub resizable: bool,
    /// 是否始终在顶部
    pub always_on_top: bool,
    /// 是否透明
    pub transparent: bool,
    /// 窗口图标路径
    pub icon_path: Option<String>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "TimeTracker - 时间追踪器".to_string(),
            initial_size: egui::Vec2::new(1000.0, 700.0),
            min_size: egui::Vec2::new(600.0, 400.0),
            resizable: true,
            always_on_top: false,
            transparent: false,
            icon_path: None,
        }
    }
}

/// GUI应用程序错误
#[derive(Debug, thiserror::Error)]
pub enum GuiError {
    #[error("GUI初始化失败: {0}")]
    InitializationFailed(String),

    #[error("渲染错误: {0}")]
    RenderError(String),

    #[error("事件处理错误: {0}")]
    EventError(String),

    #[error("主题加载错误: {0}")]
    ThemeError(String),

    #[error("存储错误: {0}")]
    StorageError(#[from] AppError),
}

/// 启动GUI应用程序
pub fn run_gui(storage_path: Option<String>) -> Result<()> {
    log::info!("启动TimeTracker GUI应用程序");

    // 创建存储管理器
    let config = if let Some(path) = storage_path.as_deref() {
        DatabaseConfig {
            database_path: path.to_string(),
            ..DatabaseConfig::default()
        }
    } else {
        DatabaseConfig::default()
    };
    let storage = Arc::new(Mutex::new(StorageManager::new(config)?));

    // 初始化数据库
    {
        let mut storage_guard = storage.lock().unwrap();
        storage_guard.initialize()?;
    }

    // 创建应用核心
    let core = Arc::new(Mutex::new(AppCore::new()));

    // 尝试加载保存的主题配置，如果失败则使用默认主题
    let theme = Theme::try_load_theme_from_config();

    // 创建应用状态
    let app_state = AppState {
        core,
        storage,
        theme,
        show_debug: false,
        window_config: WindowConfig::default(),
    };

    // 配置eframe选项
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(app_state.window_config.initial_size)
            .with_min_inner_size(app_state.window_config.min_size)
            .with_resizable(app_state.window_config.resizable)
            .with_always_on_top()
            .with_transparent(app_state.window_config.transparent)
            .with_title(&app_state.window_config.title),

        centered: true,
        follow_system_theme: true,
        default_theme: eframe::Theme::Dark,
        run_and_return: false,
        event_loop_builder: None,

        #[cfg(feature = "wgpu")]
        renderer: eframe::Renderer::Wgpu,

        #[cfg(not(feature = "wgpu"))]
        renderer: eframe::Renderer::Glow,

        ..Default::default()
    };

    // 创建并运行应用程序
    let app = TimeTrackerApp::new(app_state);

    eframe::run_native(
        &app.state.window_config.title.clone(),
        options,
        Box::new(|_cc| {
            // 这里可以进行额外的初始化
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| AppError::GuiError(format!("GUI运行失败: {}", e)))?;

    log::info!("TimeTracker GUI应用程序已退出");
    Ok(())
}

/// 创建系统托盘（如果支持）
#[cfg(feature = "tray")]
pub fn create_system_tray() -> Result<()> {
    use tray_item::{IconSource, TrayItem};

    let mut tray = TrayItem::new("TimeTracker", IconSource::Resource("icon"))?;

    tray.add_label("TimeTracker")?;
    tray.add_separator()?;

    tray.add_menu_item("显示主窗口", || {
        // TODO: 显示主窗口
        println!("显示主窗口");
    })?;

    tray.add_menu_item("开始计时", || {
        // TODO: 开始计时
        println!("开始计时");
    })?;

    tray.add_menu_item("暂停计时", || {
        // TODO: 暂停计时
        println!("暂停计时");
    })?;

    tray.add_separator()?;

    tray.add_menu_item("设置", || {
        // TODO: 打开设置
        println!("打开设置");
    })?;

    tray.add_menu_item("关于", || {
        // TODO: 显示关于对话框
        println!("关于TimeTracker");
    })?;

    tray.add_separator()?;

    tray.add_menu_item("退出", || {
        // TODO: 退出应用程序
        std::process::exit(0);
    })?;

    Ok(())
}

/// GUI工具函数
pub mod gui_utils {
    use super::*;

    /// 显示错误对话框
    pub fn show_error_dialog(ctx: &egui::Context, title: &str, message: &str) {
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(message);
                ui.separator();
                if ui.button("确定").clicked() {
                    // 关闭对话框的逻辑
                }
            });
    }

    /// 显示确认对话框
    pub fn show_confirm_dialog<F>(ctx: &egui::Context, title: &str, message: &str, on_confirm: F)
    where
        F: FnOnce(),
    {
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(message);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("确定").clicked() {
                        on_confirm();
                    }
                    if ui.button("取消").clicked() {
                        // 关闭对话框的逻辑
                    }
                });
            });
    }

    /// 格式化持续时间为显示文本
    pub fn format_duration(seconds: i64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, secs)
        } else {
            format!("{:02}:{:02}", minutes, secs)
        }
    }

    /// 创建带图标的按钮
    pub fn icon_button(ui: &mut egui::Ui, icon: &str, text: &str) -> egui::Response {
        ui.button(format!("{} {}", icon, text))
    }

    /// 创建状态指示器
    pub fn status_indicator(ui: &mut egui::Ui, is_active: bool, text: &str) {
        ui.horizontal(|ui| {
            let color = if is_active {
                egui::Color32::GREEN
            } else {
                egui::Color32::GRAY
            };

            ui.colored_label(color, "●");
            ui.label(text);
        });
    }

    /// 创建进度条
    pub fn progress_bar(ui: &mut egui::Ui, progress: f32, text: Option<&str>) {
        let progress_bar = egui::ProgressBar::new(progress).show_percentage();

        if let Some(text) = text {
            ui.add(progress_bar.text(text));
        } else {
            ui.add(progress_bar);
        }
    }

    /// 创建分组框
    pub fn group_box<R: Default>(
        ui: &mut egui::Ui,
        title: &str,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let response = egui::CollapsingHeader::new(title)
            .default_open(true)
            .show(ui, add_contents);
        egui::InnerResponse::new(
            response.body_returned.unwrap_or_default(),
            response.header_response,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_config_default() {
        let config = WindowConfig::default();
        assert_eq!(config.title, "TimeTracker - 时间追踪器");
        assert_eq!(config.initial_size, egui::Vec2::new(1000.0, 700.0));
        assert_eq!(config.min_size, egui::Vec2::new(600.0, 400.0));
        assert!(config.resizable);
        assert!(!config.always_on_top);
        assert!(!config.transparent);
        assert!(config.icon_path.is_none());
    }

    #[test]
    fn test_gui_utils_format_duration() {
        use gui_utils::format_duration;

        assert_eq!(format_duration(0), "00:00");
        assert_eq!(format_duration(30), "00:30");
        assert_eq!(format_duration(90), "01:30");
        assert_eq!(format_duration(3661), "01:01:01");
    }
}
