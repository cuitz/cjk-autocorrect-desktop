use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    pub original_text: String,
    pub formatted_text: String,
    pub mode: String,
    pub changed: bool,
    pub created_at: String,
}

pub struct HistoryStore {
    path: PathBuf,
}

impl HistoryStore {
    pub fn new() -> Result<Self, AppError> {
        let dir = crate::config::app_config::AppConfig::config_dir()?;
        let path = dir.join("history.jsonl");
        Ok(HistoryStore { path })
    }

    /// Append an item and truncate the file to `limit` entries (newest kept).
    pub fn append(&self, item: &HistoryItem, limit: usize) -> Result<(), AppError> {
        if !self.path.exists() {
            if let Some(parent) = self.path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    AppError::ConfigError(format!("Failed to create history directory: {}", e))
                })?;
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| AppError::ConfigError(format!("Failed to open history file: {}", e)))?;

        let line = serde_json::to_string(item).map_err(|e| {
            AppError::ConfigError(format!("Failed to serialize history item: {}", e))
        })?;

        writeln!(file, "{}", line)
            .map_err(|e| AppError::ConfigError(format!("Failed to write history item: {}", e)))?;

        // Flush and drop before truncating
        drop(file);

        self.truncate(limit)
    }

    /// Keep only the newest `limit` entries and rewrite the file.
    fn truncate(&self, limit: usize) -> Result<(), AppError> {
        let items = self.list_raw()?;
        if items.len() <= limit {
            return Ok(());
        }

        // Keep the newest entries
        let keep = &items[items.len() - limit..];

        let tmp_path = self.path.with_extension("jsonl.tmp");
        {
            let mut file = File::create(&tmp_path).map_err(|e| {
                AppError::ConfigError(format!("Failed to create temp history file: {}", e))
            })?;
            for item in keep {
                let line = serde_json::to_string(item).map_err(|e| {
                    AppError::ConfigError(format!("Failed to serialize history item: {}", e))
                })?;
                writeln!(file, "{}", line).map_err(|e| {
                    AppError::ConfigError(format!("Failed to write temp history file: {}", e))
                })?;
            }
        }

        fs::rename(&tmp_path, &self.path).map_err(|e| {
            AppError::ConfigError(format!("Failed to replace history file: {}", e))
        })?;

        Ok(())
    }

    /// Read all entries from the file (oldest-first order).
    fn list_raw(&self) -> Result<Vec<HistoryItem>, AppError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)
            .map_err(|e| AppError::ConfigError(format!("Failed to open history file: {}", e)))?;

        let reader = BufReader::new(file);
        let items: Vec<HistoryItem> = reader
            .lines()
            .map_while(Result::ok)
            .filter_map(|line| serde_json::from_str::<HistoryItem>(&line).ok())
            .collect();

        Ok(items)
    }

    /// Return at most `limit` entries, newest first.
    pub fn list(&self, limit: Option<usize>) -> Result<Vec<HistoryItem>, AppError> {
        let mut items = self.list_raw()?;
        items.reverse(); // newest first
        if let Some(limit) = limit {
            items.truncate(limit);
        }
        Ok(items)
    }

    pub fn clear(&self) -> Result<(), AppError> {
        if self.path.exists() {
            fs::remove_file(&self.path)
                .map_err(|e| AppError::ConfigError(format!("Failed to clear history: {}", e)))?;
        }
        Ok(())
    }
}

/// Monotonic counter used to guarantee unique history IDs even when multiple
/// items are created within the same millisecond.
static HISTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn create_history_item(
    original: &str,
    formatted: &str,
    mode: &str,
    changed: bool,
) -> HistoryItem {
    let count = HISTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
    let id = format!(
        "{}-{}",
        chrono::Local::now().timestamp_millis(),
        count
    );

    let created_at = chrono::Local::now().to_rfc3339();

    HistoryItem {
        id,
        original_text: original.to_string(),
        formatted_text: formatted.to_string(),
        mode: mode.to_string(),
        changed,
        created_at,
    }
}
