use tauri::{Emitter, Manager};

use crate::config::app_config::AppConfig;
use crate::dto::{AppConfigDto, AppConfigResponseDto};
use crate::errors::AppError;
use crate::state::AppRuntimeState;
use crate::update_tray_menu_language;
use std::sync::atomic::Ordering;

#[tauri::command]
pub async fn load_config() -> Result<AppConfigResponseDto, AppError> {
    let config = AppConfig::load()?;
    Ok(AppConfigResponseDto::from(config))
}

#[tauri::command]
pub async fn save_config(app: tauri::AppHandle, config: AppConfigDto) -> Result<(), AppError> {
    let previous_config = AppConfig::load().unwrap_or_default();
    let app_config: AppConfig = config.into();

    // Apply runtime changes
    // 1. Auto start
    {
        use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
        if app_config.auto_start {
            let _ = app.autolaunch().enable();
        } else {
            let _ = app.autolaunch().disable();
        }
    }

    // 2. Re-register global shortcut
    {
        use crate::commands::clipboard::ClipboardFormatEvent;
        use crate::services::formatter::FormatterService;
        use tauri_plugin_clipboard_manager::ClipboardExt;
        use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

        let gs = app.global_shortcut();
        let shortcut_str = crate::shortcut::normalize_shortcut(&app_config.shortcut);
        let previous_shortcut = crate::shortcut::normalize_shortcut(&previous_config.shortcut);
        if shortcut_str != previous_shortcut || !gs.is_registered(shortcut_str.as_str()) {
            let previous_was_registered = gs.is_registered(previous_shortcut.as_str());
            if previous_shortcut != shortcut_str && previous_was_registered {
                let _ = gs.unregister(previous_shortcut.as_str());
            }

            let register_result =
                gs.on_shortcut(shortcut_str.as_str(), move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Ok(text) = app.clipboard().read_text() {
                            if !text.trim().is_empty() {
                                let original = text.clone();
                                let service = FormatterService::new();
                                let request = crate::dto::FormatTextDto { text };
                                if let Ok(result) = service.format(request) {
                                    let _ = app.clipboard().write_text(&result.formatted_text);
                                    let _ = app.emit(
                                        "clipboard-formatted",
                                        ClipboardFormatEvent {
                                            original_text: original,
                                            formatted_text: result.formatted_text,
                                            changed: result.changed,
                                        },
                                    );
                                }
                            }
                        }

                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                });

            if let Err(e) = register_result {
                if previous_shortcut != shortcut_str && previous_was_registered {
                    let _ =
                        gs.on_shortcut(previous_shortcut.as_str(), move |app, _shortcut, event| {
                            if event.state == ShortcutState::Pressed {
                                if let Ok(text) = app.clipboard().read_text() {
                                    if !text.trim().is_empty() {
                                        let original = text.clone();
                                        let service = FormatterService::new();
                                        let request = crate::dto::FormatTextDto { text };
                                        if let Ok(result) = service.format(request) {
                                            let _ =
                                                app.clipboard().write_text(&result.formatted_text);
                                            let _ = app.emit(
                                                "clipboard-formatted",
                                                ClipboardFormatEvent {
                                                    original_text: original,
                                                    formatted_text: result.formatted_text,
                                                    changed: result.changed,
                                                },
                                            );
                                        }
                                    }
                                }

                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        });
                }

                return Err(AppError::ShortcutRegistrationError(e.to_string()));
            }
        }
    }

    app.state::<AppRuntimeState>()
        .close_to_tray
        .store(app_config.close_to_tray, Ordering::Relaxed);

    // 3. Update tray menu language if it changed
    if app_config.language != previous_config.language {
        update_tray_menu_language(&app, &app_config.language);
    }

    app_config.save()?;

    Ok(())
}
