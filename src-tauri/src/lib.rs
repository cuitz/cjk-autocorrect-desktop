mod commands;
mod config;
mod dto;
mod engine;
mod errors;
mod history_store;
mod services;
mod shortcut;
mod state;

use commands::{app_config, clipboard, engine_cmd, format_cmd, history_cmd};
use state::AppRuntimeState;
use std::sync::atomic::Ordering;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            // --- System tray ---
            let show_item = MenuItem::with_id(app, "show", "打开主窗口", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_item, &settings_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&menu)
                .tooltip("CJK AutoCorrect")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.emit("navigate", "/settings");
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // --- Global shortcut ---
            let app_config = config::app_config::AppConfig::load().unwrap_or_default();
            let shortcut_str = shortcut::normalize_shortcut(&app_config.shortcut);
            app.manage(AppRuntimeState::new(app_config.close_to_tray));

            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
                let gs = app.global_shortcut();
                gs.on_shortcut(shortcut_str.as_str(), move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })?;
            }

            // --- Window close-to-tray behavior ---
            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                window.clone().on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let state = app_handle.state::<AppRuntimeState>();
                        if state.close_to_tray.load(Ordering::Relaxed) {
                            api.prevent_close();
                            let _ = window.hide();
                        }
                    }
                });
            }

            // --- Auto start ---
            if app_config.auto_start {
                use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
                let _ = app.autolaunch().enable();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            format_cmd::format_text,
            app_config::load_config,
            app_config::save_config,
            clipboard::read_clipboard,
            clipboard::write_clipboard,
            history_cmd::get_history,
            history_cmd::clear_history,
            engine_cmd::check_engine,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
