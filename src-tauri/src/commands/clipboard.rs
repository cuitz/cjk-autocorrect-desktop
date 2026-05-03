use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::errors::AppError;

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
