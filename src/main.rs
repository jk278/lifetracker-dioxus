//! # LifeTracker 桌面端入口点
//!
//! 桌面端应用启动入口，包含桌面端特有的功能如系统托盘

// Windows 子系统配置：GUI应用不显示控制台窗口
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use life_tracker::run_desktop_app;
use std::process;

/// 桌面端主程序入口点
fn main() {
    // 使用 lib.rs 提供的桌面端运行函数，添加系统托盘功能
    if let Err(e) = run_desktop_app(|builder| {
        // 添加桌面端特有功能：系统托盘
        add_system_tray(builder)
    }) {
        eprintln!("应用启动失败: {}", e);
        process::exit(1);
    }
}

/// 添加系统托盘功能（桌面端特有）
fn add_system_tray(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    use tauri::{
        menu::{Menu, MenuItem, PredefinedMenuItem},
        tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
        Manager,
    };

    builder.setup(|app| {
        log::info!("桌面端应用初始化开始");

        // 基础应用初始化逻辑
        // 显示主窗口
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
        }

        // 初始化应用状态
        let app_state = life_tracker::create_app_state()?;
        app.manage(app_state);

        // 创建系统托盘菜单
        let start_timer_item =
            MenuItem::with_id(app, "start_timer", "开始计时", true, None::<&str>)?;
        let pause_timer_item =
            MenuItem::with_id(app, "pause_timer", "暂停计时", true, None::<&str>)?;
        let stop_timer_item = MenuItem::with_id(app, "stop_timer", "停止计时", true, None::<&str>)?;
        let separator = PredefinedMenuItem::separator(app)?;
        let show_window_item =
            MenuItem::with_id(app, "show_window", "显示窗口", true, None::<&str>)?;
        let restart_item = MenuItem::with_id(app, "restart", "重启应用", true, None::<&str>)?;
        let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

        let tray_menu = Menu::with_items(
            app,
            &[
                &start_timer_item,
                &pause_timer_item,
                &stop_timer_item,
                &separator,
                &show_window_item,
                &restart_item,
                &quit_item,
            ],
        )?;

        // 创建系统托盘
        let _tray: tauri::tray::TrayIcon = TrayIconBuilder::new()
            .icon(app.default_window_icon().unwrap().clone())
            .menu(&tray_menu)
            .show_menu_on_left_click(false)
            .on_menu_event(move |app, event| {
                match event.id.as_ref() {
                    "start_timer" => {
                        log::info!("从托盘开始计时");
                        // 这里可以调用计时器命令
                    }
                    "pause_timer" => {
                        log::info!("从托盘暂停计时");
                        // 这里可以调用计时器命令
                    }
                    "stop_timer" => {
                        log::info!("从托盘停止计时");
                        // 这里可以调用计时器命令
                    }
                    "show_window" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "restart" => {
                        log::info!("重启应用");
                        app.restart();
                    }
                    "quit" => {
                        log::info!("从托盘退出应用");
                        app.exit(0);
                    }
                    _ => {}
                }
            })
            .on_tray_icon_event(|tray, event| match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            })
            .build(app)?;

        log::info!("系统托盘已创建");
        log::info!("桌面端应用初始化完成");
        Ok(())
    })
}
