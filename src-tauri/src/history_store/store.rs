use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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

    pub fn append(&self, item: &HistoryItem) -> Result<(), AppError> {
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

        Ok(())
    }

    pub fn list(&self, limit: Option<usize>) -> Result<Vec<HistoryItem>, AppError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)
            .map_err(|e| AppError::ConfigError(format!("Failed to open history file: {}", e)))?;

        let reader = BufReader::new(file);
        let mut items: Vec<HistoryItem> = reader
            .lines()
            .map_while(Result::ok)
            .filter_map(|line| serde_json::from_str::<HistoryItem>(&line).ok())
            .collect();

        // Most recent first
        items.reverse();

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

pub fn create_history_item(
    original: &str,
    formatted: &str,
    mode: &str,
    changed: bool,
) -> HistoryItem {
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string());

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
