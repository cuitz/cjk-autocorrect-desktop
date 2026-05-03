use serde::Serialize;

use crate::services::formatter::FormatterService;

#[derive(Debug, Serialize)]
pub struct EngineStatusDto {
    /// Whether the embedded autocorrect engine is available.
    pub autocorrect_installed: bool,
    /// Engine label kept in the legacy path field for frontend compatibility.
    pub autocorrect_path: Option<String>,
    /// Installation hint if autocorrect is not available.
    pub install_hint: Option<String>,
}

#[tauri::command]
pub async fn check_engine() -> EngineStatusDto {
    let service = FormatterService::new();
    EngineStatusDto {
        autocorrect_installed: service.is_autocorrect_available(),
        autocorrect_path: Some(service.engine_name().to_string()),
        install_hint: service.install_hint(),
    }
}
