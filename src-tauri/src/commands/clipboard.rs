use serde::Serialize;
use tauri::{Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::dto::FormatTextDto;
use crate::errors::AppError;
use crate::history_store::store::persist_if_enabled;
use crate::services::formatter::FormatterService;

/// Emitted to the frontend after the clipboard has been formatted (either via
/// the global shortcut or the `format_clipboard` Tauri command). Both code
/// paths share this exact payload shape.
#[derive(Debug, Clone, Serialize)]
pub struct ClipboardFormatEvent {
    pub original_text: String,
    pub formatted_text: String,
    pub changed: bool,
}

/// Read clipboard, format with current rules, write the result back, persist
/// to history (mirroring the `format_text` command), and emit
/// `clipboard-formatted` to the frontend.
///
/// Empty / whitespace-only clipboards are intentionally a silent no-op so the
/// global shortcut and the explicit command share the same UX.
pub fn perform_clipboard_format(app: &tauri::AppHandle) -> Result<(), AppError> {
    let clipboard = app.clipboard();
    let text = clipboard
        .read_text()
        .map_err(|e| AppError::ClipboardError(format!("Failed to read clipboard: {}", e)))?;

    if text.trim().is_empty() {
        return Ok(());
    }

    let service = FormatterService::new();
    let request = FormatTextDto { text };
    let result = service.format(request)?;

    clipboard
        .write_text(&result.formatted_text)
        .map_err(|e| AppError::ClipboardError(format!("Failed to write clipboard: {}", e)))?;

    persist_if_enabled(&result.original_text, &result.formatted_text, result.changed);

    let _ = app.emit(
        "clipboard-formatted",
        ClipboardFormatEvent {
            original_text: result.original_text,
            formatted_text: result.formatted_text,
            changed: result.changed,
        },
    );

    Ok(())
}

/// Handler invoked by the global shortcut. Format the clipboard (errors are
/// swallowed here since there is no UI to surface them to from a shortcut
/// context) and bring the main window forward.
pub fn on_global_shortcut_pressed(app: &tauri::AppHandle) {
    let _ = perform_clipboard_format(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
pub async fn read_clipboard(app: tauri::AppHandle) -> Result<String, AppError> {
    let clipboard = app.clipboard();
    clipboard
        .read_text()
        .map_err(|e| AppError::ClipboardError(format!("Failed to read clipboard: {}", e)))
}

#[tauri::command]
pub async fn write_clipboard(app: tauri::AppHandle, text: String) -> Result<(), AppError> {
    let clipboard = app.clipboard();
    clipboard
        .write_text(&text)
        .map_err(|e| AppError::ClipboardError(format!("Failed to write clipboard: {}", e)))
}

/// Tauri command counterpart to the global shortcut: same behaviour, exposed
/// to the frontend so a button can trigger the same flow.
#[tauri::command]
pub async fn format_clipboard(app: tauri::AppHandle) -> Result<(), AppError> {
    perform_clipboard_format(&app)
}
