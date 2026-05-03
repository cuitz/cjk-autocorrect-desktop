use crate::dto::{HistoryItemDto, HistoryListRequest};
use crate::errors::AppError;
use crate::history_store::store::HistoryStore;

#[tauri::command]
pub async fn get_history(
    request: Option<HistoryListRequest>,
) -> Result<Vec<HistoryItemDto>, AppError> {
    let store = HistoryStore::new()?;
    let limit = request.and_then(|r| r.limit);
    let items = store.list(limit)?;
    Ok(items.into_iter().map(HistoryItemDto::from).collect())
}

#[tauri::command]
pub async fn clear_history() -> Result<(), AppError> {
    let store = HistoryStore::new()?;
    store.clear()
}
