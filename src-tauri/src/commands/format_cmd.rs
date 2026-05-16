use crate::dto::{FormatResultDto, FormatTextDto};
use crate::errors::AppError;
use crate::history_store::store::persist_if_enabled;
use crate::services::formatter::FormatterService;

#[tauri::command]
pub async fn format_text(request: FormatTextDto) -> Result<FormatResultDto, AppError> {
    let service = FormatterService::new();
    let result = service.format(request)?;
    persist_if_enabled(&result.original_text, &result.formatted_text, result.changed);
    Ok(result)
}
