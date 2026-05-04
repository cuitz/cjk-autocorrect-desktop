use serde::Serialize;
use tauri::Emitter;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::dto::FormatResultDto;
use crate::errors::AppError;
use crate::services::formatter::FormatterService;

/// Emitted when the global shortcut formats clipboard content.
#[derive(Debug, Clone, Serialize)]
pub struct ClipboardFormatEvent {
    pub original_text: String,
    pub formatted_text: String,
    pub changed: bool,
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

/// Read clipboard → format → write back. Emits "clipboard-formatted" event to the frontend.
#[tauri::command]
pub async fn format_clipboard(app: tauri::AppHandle) -> Result<(), AppError> {
    let clipboard = app.clipboard();
    let text = clipboard
        .read_text()
        .map_err(|e| AppError::ClipboardError(format!("Failed to read clipboard: {}", e)))?;

    if text.trim().is_empty() {
        return Err(AppError::ClipboardError("Clipboard is empty".to_string()));
    }

    let service = FormatterService::new();
    let request = crate::dto::FormatTextDto { text };
    let result: FormatResultDto = service.format(request)?;

    clipboard
        .write_text(&result.formatted_text)
        .map_err(|e| AppError::ClipboardError(format!("Failed to write clipboard: {}", e)))?;

    let _ = app.emit("clipboard-formatted", result.changed);

    Ok(())
}
