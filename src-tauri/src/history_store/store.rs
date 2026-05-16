use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    pub original_text: String,
    pub formatted_text: String,
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

    /// Append an item, then truncate the file to `limit` entries (newest kept)
    /// only when the on-disk line count actually exceeds `limit`.
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

        // Cheap pre-check: only walk the file when the line count likely exceeds the limit.
        if self.line_count_exceeds(limit)? {
            self.truncate(limit)?;
        }
        Ok(())
    }

    /// Stream-count newlines and return true once the count strictly exceeds
    /// `limit`. This avoids parsing the entire file in the common case where
    /// the history is well under the cap.
    fn line_count_exceeds(&self, limit: usize) -> Result<bool, AppError> {
        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(e) => {
                return Err(AppError::ConfigError(format!(
                    "Failed to open history file: {}",
                    e
                )))
            }
        };
        let mut reader = BufReader::new(file);
        let mut buf = [0u8; 8192];
        let mut count: usize = 0;
        loop {
            let n = reader.read(&mut buf).map_err(|e| {
                AppError::ConfigError(format!("Failed to read history file: {}", e))
            })?;
            if n == 0 {
                return Ok(false);
            }
            for &b in &buf[..n] {
                if b == b'\n' {
                    count += 1;
                    if count > limit {
                        return Ok(true);
                    }
                }
            }
        }
    }

    /// Keep only the newest `limit` entries and rewrite the file.
    fn truncate(&self, limit: usize) -> Result<(), AppError> {
        let items = self.list_raw()?;
        if items.len() <= limit {
            return Ok(());
        }

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

        fs::rename(&tmp_path, &self.path)
            .map_err(|e| AppError::ConfigError(format!("Failed to replace history file: {}", e)))?;

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

    /// Check whether the last entry has the same original/formatted pair as the
    /// candidate. Reads only the trailing portion of the file (up to ~8 KiB) to
    /// avoid parsing the entire history on every format call.
    pub fn is_duplicate_of_last(&self, original: &str, formatted: &str) -> Result<bool, AppError> {
        if !self.path.exists() {
            return Ok(false);
        }
        let last_line = match self.read_last_line()? {
            Some(line) => line,
            None => return Ok(false),
        };
        let Ok(last) = serde_json::from_str::<HistoryItem>(&last_line) else {
            // Corrupt trailing line: be conservative and don't treat it as a duplicate.
            return Ok(false);
        };
        Ok(last.original_text == original && last.formatted_text == formatted)
    }

    /// Read the last non-empty line from the file by seeking back from EOF.
    fn read_last_line(&self) -> Result<Option<String>, AppError> {
        let mut file = File::open(&self.path)
            .map_err(|e| AppError::ConfigError(format!("Failed to open history file: {}", e)))?;
        let len = file
            .metadata()
            .map_err(|e| AppError::ConfigError(format!("Failed to stat history file: {}", e)))?
            .len();
        if len == 0 {
            return Ok(None);
        }

        // Read the trailing window in 4 KiB chunks until we find a newline that
        // separates a non-empty final line, or until we've read the whole file.
        const CHUNK: u64 = 4096;
        let mut window: Vec<u8> = Vec::new();
        let mut read_so_far: u64 = 0;
        loop {
            let to_read = CHUNK.min(len - read_so_far);
            if to_read == 0 {
                break;
            }
            let new_pos = len - read_so_far - to_read;
            file.seek(SeekFrom::Start(new_pos)).map_err(|e| {
                AppError::ConfigError(format!("Failed to seek history file: {}", e))
            })?;
            let mut buf = vec![0u8; to_read as usize];
            file.read_exact(&mut buf).map_err(|e| {
                AppError::ConfigError(format!("Failed to read history file: {}", e))
            })?;
            buf.extend_from_slice(&window);
            window = buf;
            read_so_far += to_read;

            // Trim trailing newlines/whitespace, then look for the next newline.
            let trimmed_end = window
                .iter()
                .rposition(|&b| b != b'\n' && b != b'\r')
                .map(|i| i + 1)
                .unwrap_or(0);
            if trimmed_end == 0 {
                if read_so_far >= len {
                    return Ok(None);
                }
                continue;
            }
            if let Some(nl) = window[..trimmed_end].iter().rposition(|&b| b == b'\n') {
                let line_bytes = &window[nl + 1..trimmed_end];
                let line = String::from_utf8_lossy(line_bytes).into_owned();
                return Ok(Some(line));
            }
            if read_so_far >= len {
                let line_bytes = &window[..trimmed_end];
                let line = String::from_utf8_lossy(line_bytes).into_owned();
                return Ok(Some(line));
            }
        }
        Ok(None)
    }
}

/// Monotonic counter used to guarantee unique history IDs even when multiple
/// items are created within the same millisecond.
static HISTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn create_history_item(original: &str, formatted: &str, changed: bool) -> HistoryItem {
    let count = HISTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
    let id = format!("{}-{}", chrono::Local::now().timestamp_millis(), count);

    let created_at = chrono::Local::now().to_rfc3339();

    HistoryItem {
        id,
        original_text: original.to_string(),
        formatted_text: formatted.to_string(),
        changed,
        created_at,
    }
}

/// Best-effort history persistence shared by all formatting code paths.
/// Honours the user's history settings, deduplicates against the most recent
/// entry, and never propagates errors so a history failure can't fail the
/// caller's primary operation.
pub fn persist_if_enabled(original: &str, formatted: &str, changed: bool) {
    if !changed {
        return;
    }
    let Ok(config) = crate::config::app_config::AppConfig::load() else {
        return;
    };
    if !config.history_enabled {
        return;
    }
    let Ok(store) = HistoryStore::new() else {
        return;
    };
    let duplicate = store
        .is_duplicate_of_last(original, formatted)
        .unwrap_or(false);
    if duplicate {
        return;
    }
    let item = create_history_item(original, formatted, changed);
    let _ = store.append(&item, config.history_limit as usize);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_lines(path: &PathBuf, lines: &[&str]) {
        let mut f = File::create(path).unwrap();
        for line in lines {
            writeln!(f, "{}", line).unwrap();
        }
    }

    fn make_item(original: &str, formatted: &str) -> HistoryItem {
        create_history_item(original, formatted, original != formatted)
    }

    #[test]
    fn read_last_line_picks_final_entry() {
        let dir = std::env::temp_dir().join(format!(
            "hist-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("history.jsonl");

        let item_a = make_item("hello", "hello world");
        let item_b = make_item("foo", "foo bar");
        let line_a = serde_json::to_string(&item_a).unwrap();
        let line_b = serde_json::to_string(&item_b).unwrap();
        write_lines(&path, &[&line_a, &line_b]);

        let store = HistoryStore { path: path.clone() };
        let last = store.read_last_line().unwrap().unwrap();
        let parsed: HistoryItem = serde_json::from_str(&last).unwrap();
        assert_eq!(parsed.original_text, "foo");

        // Duplicate detection should match the last entry only.
        assert!(store.is_duplicate_of_last("foo", "foo bar").unwrap());
        assert!(!store.is_duplicate_of_last("hello", "hello world").unwrap());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn read_last_line_handles_trailing_newlines() {
        let dir = std::env::temp_dir().join(format!(
            "hist-test-trail-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("history.jsonl");

        let item = make_item("a", "b");
        let line = serde_json::to_string(&item).unwrap();
        let mut f = File::create(&path).unwrap();
        write!(f, "{}\n\n\n", line).unwrap();
        drop(f);

        let store = HistoryStore { path: path.clone() };
        let last = store.read_last_line().unwrap().unwrap();
        let parsed: HistoryItem = serde_json::from_str(&last).unwrap();
        assert_eq!(parsed.original_text, "a");

        fs::remove_dir_all(&dir).ok();
    }
}
