use crate::dto::{FormatResultDto, FormatTextDto};
use crate::errors::AppError;
use crate::history_store::store::{create_history_item, HistoryStore};
use crate::services::formatter::FormatterService;

#[tauri::command]
pub async fn format_text(mut request: FormatTextDto) -> Result<FormatResultDto, AppError> {
    let config = crate::config::app_config::AppConfig::load().ok();
    let service = FormatterService::new();

    let default_mode = config
        .as_ref()
        .map(|c| {
            if matches!(&c.formatter.mode, crate::engine::FormatMode::Strict) {
                "strict"
            } else {
                "standard"
            }
        })
        .unwrap_or("standard")
        .to_string();
    let mode_str = request.mode.clone().unwrap_or(default_mode);
    request.mode = Some(mode_str.clone());

    let result = service.format(request)?;

    // Save to history if enabled and text changed
    if config.as_ref().is_some_and(|c| c.history_enabled) && result.changed {
        let store = HistoryStore::new()?;
        let item = create_history_item(
            &result.original_text,
            &result.formatted_text,
            &mode_str,
            result.changed,
        );
        let _ = store.append(&item);
    }

    Ok(result)
}
