use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::engine::FormatMode;
use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LanguageMode {
    #[default]
    System,
    ZhCn,
    En,
    Ja,
    Ko,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterConfig {
    pub mode: FormatMode,
    /// Deprecated. Kept only so older config files continue to deserialize.
    #[serde(default)]
    pub autocorrect_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub shortcut: String,
    pub auto_start: bool,
    pub close_to_tray: bool,
    pub theme: ThemeMode,
    #[serde(default)]
    pub language: LanguageMode,
    pub history_enabled: bool,
    pub formatter: FormatterConfig,
    pub version: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            shortcut: "CommandOrControl+Shift+F".to_string(),
            auto_start: false,
            close_to_tray: true,
            theme: ThemeMode::System,
            language: LanguageMode::System,
            history_enabled: true,
            formatter: FormatterConfig {
                mode: FormatMode::Standard,
                autocorrect_path: None,
            },
            version: 1,
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> Result<PathBuf, AppError> {
        let dir = dirs::data_dir()
            .ok_or_else(|| {
                AppError::ConfigError("Cannot determine app data directory".to_string())
            })?
            .join("cjk-autocorrect-desktop");

        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| {
                AppError::ConfigError(format!("Failed to create config directory: {}", e))
            })?;
        }

        Ok(dir)
    }

    pub fn config_path() -> Result<PathBuf, AppError> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    pub fn load() -> Result<Self, AppError> {
        let path = Self::config_path()?;

        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| AppError::ConfigError(format!("Failed to read config: {}", e)))?;

        match serde_json::from_str(&content) {
            Ok(config) => Ok(config),
            Err(e) => {
                eprintln!("Config file corrupted, falling back to default: {}", e);
                let default = Self::default();
                let _ = default.save();
                Ok(default)
            }
        }
    }

    pub fn save(&self) -> Result<(), AppError> {
        let path = Self::config_path()?;

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&path, content)
            .map_err(|e| AppError::ConfigError(format!("Failed to write config: {}", e)))?;

        Ok(())
    }
}
