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
use config::app_config::LanguageMode;
use state::AppRuntimeState;
use std::sync::atomic::Ordering;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

/// Tray menu item IDs — shared between setup and runtime update.
pub(crate) const TRAY_MENU_SHOW: &str = "show";
pub(crate) const TRAY_MENU_SETTINGS: &str = "settings";
pub(crate) const TRAY_MENU_QUIT: &str = "quit";

/// Tray menu text for each supported language.
pub(crate) struct TrayLabels {
    pub(crate) show: &'static str,
    pub(crate) settings: &'static str,
    pub(crate) quit: &'static str,
}

pub(crate) fn tray_labels_for_language(lang: &LanguageMode) -> TrayLabels {
    match lang {
        LanguageMode::En => TrayLabels {
            show: "Show Main Window",
            settings: "Settings",
            quit: "Quit",
        },
        LanguageMode::Ja => TrayLabels {
            show: "メインウィンドウを表示",
            settings: "設定",
            quit: "終了",
        },
        LanguageMode::Ko => TrayLabels {
            show: "메인 창 보기",
            settings: "설정",
            quit: "종료",
        },
        // System and ZhCn (default) both use Chinese
        _ => TrayLabels {
            show: "打开主窗口",
            settings: "设置",
            quit: "退出",
        },
    }
}

/// Build a tray menu for the given language using the provided app handle.
fn build_tray_menu(app: &tauri::AppHandle, lang: &LanguageMode) -> tauri::Result<Menu<tauri::Wry>> {
    let labels = tray_labels_for_language(lang);
    let show_item = MenuItem::with_id(app, TRAY_MENU_SHOW, labels.show, true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, TRAY_MENU_SETTINGS, labels.settings, true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, TRAY_MENU_QUIT, labels.quit, true, None::<&str>)?;
    Menu::with_items(app, &[&show_item, &settings_item, &quit_item])
}

/// Update the tray menu to reflect the given language.
pub(crate) fn update_tray_menu_language(app: &tauri::AppHandle, lang: &LanguageMode) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(menu) = build_tray_menu(app, lang) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

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
            let app_config = config::app_config::AppConfig::load().unwrap_or_default();
            let app_handle = app.handle().clone();
            let menu = build_tray_menu(&app_handle, &app_config.language)?;

            let _tray = TrayIconBuilder::with_id("main-tray")
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

            // --- Global shortcut --- format clipboard & show window
            let shortcut_str = shortcut::normalize_shortcut(&app_config.shortcut);
            app.manage(AppRuntimeState::new(app_config.close_to_tray));

            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
                use tauri_plugin_clipboard_manager::ClipboardExt;
                use crate::services::formatter::FormatterService;
                use crate::commands::clipboard::ClipboardFormatEvent;

                let gs = app.global_shortcut();
                gs.on_shortcut(shortcut_str.as_str(), move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        // Format clipboard content
                        if let Ok(text) = app.clipboard().read_text() {
                            if !text.trim().is_empty() {
                                let original = text.clone();
                                let service = FormatterService::new();
                                let request = crate::dto::FormatTextDto {
                                    text,
                                    mode: Some("standard".to_string()),
                                };
                                if let Ok(result) = service.format(request) {
                                    let _ = app.clipboard().write_text(&result.formatted_text);
                                    let _ = app.emit("clipboard-formatted", ClipboardFormatEvent {
                                        original_text: original,
                                        formatted_text: result.formatted_text,
                                        changed: result.changed,
                                    });
                                }
                            }
                        }

                        // Show and focus main window
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
            clipboard::format_clipboard,
            history_cmd::get_history,
            history_cmd::clear_history,
            engine_cmd::check_engine,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
