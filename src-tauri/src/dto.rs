use serde::{Deserialize, Serialize};

use crate::config::app_config::{AppConfig, FormatterConfig, LanguageMode, ThemeMode};
use crate::engine::types::{FormatMode, FormatRequest, FormatResponse};
use crate::history_store::store::HistoryItem;

// --- Format DTOs ---

#[derive(Debug, Deserialize)]
pub struct FormatTextDto {
    pub text: String,
    pub mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FormatResultDto {
    pub original_text: String,
    pub formatted_text: String,
    pub changed: bool,
    pub diagnostics: Vec<DiagnosticDto>,
    pub elapsed_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticDto {
    pub line: usize,
    pub col: usize,
    pub message: String,
    pub severity: String,
}

impl From<FormatTextDto> for FormatRequest {
    fn from(dto: FormatTextDto) -> Self {
        let mode = match dto.mode.as_deref() {
            Some("strict") => FormatMode::Strict,
            _ => FormatMode::Standard,
        };
        FormatRequest {
            text: dto.text,
            mode,
        }
    }
}

impl From<FormatResponse> for FormatResultDto {
    fn from(resp: FormatResponse) -> Self {
        FormatResultDto {
            original_text: resp.original_text,
            formatted_text: resp.formatted_text,
            changed: resp.changed,
            diagnostics: resp
                .diagnostics
                .into_iter()
                .map(|d| DiagnosticDto {
                    line: d.line,
                    col: d.col,
                    message: d.message,
                    severity: format!("{:?}", d.severity),
                })
                .collect(),
            elapsed_ms: resp.elapsed_ms,
        }
    }
}

// --- Config DTOs ---

#[derive(Debug, Deserialize)]
pub struct AppConfigDto {
    pub shortcut: String,
    pub auto_start: bool,
    pub close_to_tray: bool,
    pub theme: String,
    #[serde(default = "default_language")]
    pub language: String,
    pub history_enabled: bool,
    #[serde(default)]
    pub history_limit: u32,
    #[serde(default)]
    pub diff_highlight: bool,
    pub formatter: FormatterConfigDto,
}

#[derive(Debug, Serialize)]
pub struct AppConfigResponseDto {
    pub shortcut: String,
    pub auto_start: bool,
    pub close_to_tray: bool,
    pub theme: String,
    pub language: String,
    pub history_enabled: bool,
    pub history_limit: u32,
    pub diff_highlight: bool,
    pub formatter: FormatterConfigDto,
    pub version: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FormatterConfigDto {
    pub mode: String,
    #[serde(default)]
    pub autocorrect_path: Option<String>,
}

impl From<AppConfig> for AppConfigResponseDto {
    fn from(config: AppConfig) -> Self {
        AppConfigResponseDto {
            shortcut: config.shortcut,
            auto_start: config.auto_start,
            close_to_tray: config.close_to_tray,
            theme: format!("{:?}", config.theme).to_lowercase(),
            language: language_to_string(&config.language),
            history_enabled: config.history_enabled,
            history_limit: config.history_limit,
            diff_highlight: config.diff_highlight,
            formatter: FormatterConfigDto {
                mode: format!("{:?}", config.formatter.mode).to_lowercase(),
                autocorrect_path: config.formatter.autocorrect_path,
            },
            version: config.version,
        }
    }
}

impl From<AppConfigDto> for AppConfig {
    fn from(dto: AppConfigDto) -> Self {
        let theme = match dto.theme.as_str() {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::System,
        };
        let language = match dto.language.as_str() {
            "zh-CN" | "zh-cn" | "zh" => LanguageMode::ZhCn,
            "en" | "en-US" | "en-us" => LanguageMode::En,
            "ja" | "ja-JP" | "ja-jp" => LanguageMode::Ja,
            "ko" | "ko-KR" | "ko-kr" => LanguageMode::Ko,
            _ => LanguageMode::System,
        };
        let format_mode = FormatMode::Standard;
        let mut app_config = AppConfig {
            shortcut: dto.shortcut,
            auto_start: dto.auto_start,
            close_to_tray: dto.close_to_tray,
            theme,
            language,
            history_enabled: dto.history_enabled,
            history_limit: dto.history_limit,
            diff_highlight: dto.diff_highlight,
            formatter: FormatterConfig {
                mode: format_mode,
                autocorrect_path: dto.formatter.autocorrect_path,
            },
            version: 1,
        };
        app_config.clamp_history_limit();
        app_config
    }
}

fn default_language() -> String {
    "system".to_string()
}

fn language_to_string(language: &LanguageMode) -> String {
    match language {
        LanguageMode::System => "system",
        LanguageMode::ZhCn => "zh-CN",
        LanguageMode::En => "en",
        LanguageMode::Ja => "ja",
        LanguageMode::Ko => "ko",
    }
    .to_string()
}

// --- History DTOs ---

#[derive(Debug, Deserialize)]
pub struct HistoryListRequest {
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct HistoryItemDto {
    pub id: String,
    pub original_text: String,
    pub formatted_text: String,
    pub mode: String,
    pub changed: bool,
    pub created_at: String,
}

impl From<HistoryItem> for HistoryItemDto {
    fn from(item: HistoryItem) -> Self {
        HistoryItemDto {
            id: item.id,
            original_text: item.original_text,
            formatted_text: item.formatted_text,
            mode: item.mode,
            changed: item.changed,
            created_at: item.created_at,
        }
    }
}
