use serde::Serialize;

use crate::services::formatter::FormatterService;

#[derive(Debug, Serialize)]
pub struct EngineStatusDto {
    /// Whether autocorrect CLI is installed
    pub autocorrect_installed: bool,
    /// Path to autocorrect binary, if found
    pub autocorrect_path: Option<String>,
    /// Installation hint if autocorrect is not installed
    pub install_hint: Option<String>,
}

#[tauri::command]
pub async fn check_engine() -> EngineStatusDto {
    // Load config to get custom autocorrect path
    let config = crate::config::app_config::AppConfig::load().ok();
    let custom_path = config
        .as_ref()
        .and_then(|c| c.formatter.autocorrect_path.clone());

    let service = FormatterService::with_custom_path(custom_path);
    EngineStatusDto {
        autocorrect_installed: service.is_autocorrect_available(),
        autocorrect_path: service.autocorrect_path().map(|s| s.to_string()),
        install_hint: service.install_hint(),
    }
}
