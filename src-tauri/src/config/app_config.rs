use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum LanguageMode {
    #[default]
    System,
    ZhCn,
    En,
    Ja,
    Ko,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormatterConfig {
    #[serde(default = "default_engine_rules")]
    pub rules: FormatterRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterRules {
    /// Auto add spacing between CJK text and Latin letters or numbers.
    #[serde(default = "default_true")]
    pub space_word: bool,
    /// Add space around punctuation-like symbols near CJK text.
    #[serde(default = "default_true")]
    pub space_punctuation: bool,
    /// Add space around brackets when near CJK text.
    #[serde(default = "default_true")]
    pub space_bracket: bool,
    /// Add space around dash separators near CJK text.
    #[serde(default = "default_true")]
    pub space_dash: bool,
    /// Add space around backticks near CJK text.
    #[serde(default = "default_true")]
    pub space_backticks: bool,
    /// Add space around dollar markers near CJK text.
    #[serde(default)]
    pub space_dollar: bool,
    /// Convert punctuation near CJK text to fullwidth punctuation.
    #[serde(default = "default_true")]
    pub fullwidth: bool,
    /// Convert fullwidth alphanumeric characters to halfwidth.
    #[serde(default = "default_true")]
    pub halfwidth_word: bool,
    /// Convert fullwidth punctuation to halfwidth in English contexts.
    #[serde(default = "default_true")]
    pub halfwidth_punctuation: bool,
    /// Remove spaces around fullwidth punctuation.
    #[serde(default = "default_true")]
    pub no_space_fullwidth: bool,
    /// Remove spaces around fullwidth quotes.
    #[serde(default = "default_true")]
    pub no_space_fullwidth_quote: bool,
    /// Apply autocorrect's spellcheck dictionary.
    #[serde(default)]
    pub spellcheck: bool,
}

impl Default for FormatterRules {
    fn default() -> Self {
        Self {
            space_word: true,
            space_punctuation: true,
            space_bracket: true,
            space_dash: true,
            space_backticks: true,
            space_dollar: false,
            fullwidth: true,
            halfwidth_word: true,
            halfwidth_punctuation: true,
            no_space_fullwidth: true,
            no_space_fullwidth_quote: true,
            spellcheck: false,
        }
    }
}

impl FormatterRules {
    pub fn autocorrect_rules(&self) -> [(&'static str, bool); 12] {
        [
            ("space-word", self.space_word),
            ("space-punctuation", self.space_punctuation),
            ("space-bracket", self.space_bracket),
            ("space-dash", self.space_dash),
            ("space-backticks", self.space_backticks),
            ("space-dollar", self.space_dollar),
            ("fullwidth", self.fullwidth),
            ("halfwidth-word", self.halfwidth_word),
            ("halfwidth-punctuation", self.halfwidth_punctuation),
            ("no-space-fullwidth", self.no_space_fullwidth),
            ("no-space-fullwidth-quote", self.no_space_fullwidth_quote),
            ("spellcheck", self.spellcheck),
        ]
    }
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
    #[serde(default = "default_history_limit")]
    pub history_limit: u32,
    #[serde(default)]
    pub diff_highlight: bool,
    #[serde(default)]
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
            history_limit: 500,
            diff_highlight: false,
            formatter: FormatterConfig::default(),
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

const HISTORY_LIMIT_MIN: u32 = 100;
const HISTORY_LIMIT_MAX: u32 = 1000;

fn default_history_limit() -> u32 {
    500
}

fn default_true() -> bool {
    true
}

fn default_engine_rules() -> FormatterRules {
    FormatterRules::default()
}

impl AppConfig {
    /// Clamp history_limit to valid range [100, 1000].
    pub fn clamp_history_limit(&mut self) {
        self.history_limit = self
            .history_limit
            .clamp(HISTORY_LIMIT_MIN, HISTORY_LIMIT_MAX);
    }
}
