use serde::{Deserialize, Serialize};

use crate::config::app_config::{
    AppConfig, FormatterConfig, FormatterRules, LanguageMode, ThemeMode,
};
use crate::engine::types::{FormatRequest, FormatResponse};
use crate::history_store::store::HistoryItem;

// --- Format DTOs ---

#[derive(Debug, Deserialize)]
pub struct FormatTextDto {
    pub text: String,
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
        FormatRequest {
            text: dto.text,
            formatter: FormatterConfig::default(),
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
    #[serde(default)]
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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FormatterConfigDto {
    #[serde(default)]
    pub rules: FormatterRulesDto,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormatterRulesDto {
    #[serde(default = "default_true")]
    pub space_word: bool,
    #[serde(default = "default_true")]
    pub space_punctuation: bool,
    #[serde(default = "default_true")]
    pub space_bracket: bool,
    #[serde(default = "default_true")]
    pub space_dash: bool,
    #[serde(default = "default_true")]
    pub space_backticks: bool,
    #[serde(default)]
    pub space_dollar: bool,
    #[serde(default = "default_true")]
    pub fullwidth: bool,
    #[serde(default = "default_true")]
    pub halfwidth_word: bool,
    #[serde(default = "default_true")]
    pub halfwidth_punctuation: bool,
    #[serde(default = "default_true")]
    pub no_space_fullwidth: bool,
    #[serde(default = "default_true")]
    pub no_space_fullwidth_quote: bool,
    #[serde(default)]
    pub spellcheck: bool,
}

impl Default for FormatterRulesDto {
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

impl From<AppConfig> for AppConfigResponseDto {
    fn from(config: AppConfig) -> Self {
        AppConfigResponseDto {
            shortcut: config.shortcut,
            auto_start: config.auto_start,
            close_to_tray: config.close_to_tray,
            theme: theme_to_string(&config.theme),
            language: language_to_string(&config.language),
            history_enabled: config.history_enabled,
            history_limit: config.history_limit,
            diff_highlight: config.diff_highlight,
            formatter: FormatterConfigDto::from(config.formatter),
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
        let mut app_config = AppConfig {
            shortcut: dto.shortcut,
            auto_start: dto.auto_start,
            close_to_tray: dto.close_to_tray,
            theme,
            language,
            history_enabled: dto.history_enabled,
            history_limit: dto.history_limit,
            diff_highlight: dto.diff_highlight,
            formatter: FormatterConfig::from(dto.formatter),
            version: 1,
        };
        app_config.clamp_history_limit();
        app_config
    }
}

impl From<FormatterConfig> for FormatterConfigDto {
    fn from(config: FormatterConfig) -> Self {
        Self {
            rules: FormatterRulesDto::from(config.rules),
        }
    }
}

impl From<FormatterConfigDto> for FormatterConfig {
    fn from(dto: FormatterConfigDto) -> Self {
        Self {
            rules: FormatterRules::from(dto.rules),
        }
    }
}

impl From<FormatterRules> for FormatterRulesDto {
    fn from(rules: FormatterRules) -> Self {
        Self {
            space_word: rules.space_word,
            space_punctuation: rules.space_punctuation,
            space_bracket: rules.space_bracket,
            space_dash: rules.space_dash,
            space_backticks: rules.space_backticks,
            space_dollar: rules.space_dollar,
            fullwidth: rules.fullwidth,
            halfwidth_word: rules.halfwidth_word,
            halfwidth_punctuation: rules.halfwidth_punctuation,
            no_space_fullwidth: rules.no_space_fullwidth,
            no_space_fullwidth_quote: rules.no_space_fullwidth_quote,
            spellcheck: rules.spellcheck,
        }
    }
}

impl From<FormatterRulesDto> for FormatterRules {
    fn from(dto: FormatterRulesDto) -> Self {
        Self {
            space_word: dto.space_word,
            space_punctuation: dto.space_punctuation,
            space_bracket: dto.space_bracket,
            space_dash: dto.space_dash,
            space_backticks: dto.space_backticks,
            space_dollar: dto.space_dollar,
            fullwidth: dto.fullwidth,
            halfwidth_word: dto.halfwidth_word,
            halfwidth_punctuation: dto.halfwidth_punctuation,
            no_space_fullwidth: dto.no_space_fullwidth,
            no_space_fullwidth_quote: dto.no_space_fullwidth_quote,
            spellcheck: dto.spellcheck,
        }
    }
}

fn default_language() -> String {
    "system".to_string()
}

fn default_true() -> bool {
    true
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

fn theme_to_string(theme: &ThemeMode) -> String {
    match theme {
        ThemeMode::System => "system",
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
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
    pub changed: bool,
    pub created_at: String,
}

impl From<HistoryItem> for HistoryItemDto {
    fn from(item: HistoryItem) -> Self {
        HistoryItemDto {
            id: item.id,
            original_text: item.original_text,
            formatted_text: item.formatted_text,
            changed: item.changed,
            created_at: item.created_at,
        }
    }
}
