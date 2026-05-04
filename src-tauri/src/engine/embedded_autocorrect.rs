use std::{
    sync::{Mutex, OnceLock},
    time::Instant,
};

use crate::engine::types::{FormatRequest, FormatResponse, FormatterEngine};
use crate::errors::AppError;
use autocorrect::config::SeverityMode;

static AUTOCORRECT_CONFIG_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Embedded AutoCorrect engine backed by the upstream Rust crate.
pub struct EmbeddedAutocorrectEngine;

impl EmbeddedAutocorrectEngine {
    pub fn new() -> Self {
        Self
    }
}

impl FormatterEngine for EmbeddedAutocorrectEngine {
    fn format(&self, req: &FormatRequest) -> Result<FormatResponse, AppError> {
        let start = Instant::now();
        let _guard = AUTOCORRECT_CONFIG_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .map_err(|e| AppError::FormatFailed(format!("Formatter lock poisoned: {}", e)))?;
        apply_autocorrect_config(&req.formatter)?;
        let formatted = autocorrect::format(&req.text);
        let elapsed = start.elapsed();
        let changed = req.text != formatted;

        Ok(FormatResponse {
            original_text: req.text.clone(),
            formatted_text: formatted,
            changed,
            diagnostics: vec![],
            elapsed_ms: elapsed.as_millis() as u64,
        })
    }

    fn name(&self) -> &str {
        "autocorrect-embedded"
    }

    fn is_available(&self) -> bool {
        true
    }
}

fn apply_autocorrect_config(
    formatter: &crate::config::app_config::FormatterConfig,
) -> Result<(), AppError> {
    let rules = formatter
        .rules
        .autocorrect_rules()
        .into_iter()
        .map(|(name, enabled)| {
            let mode = if enabled {
                SeverityMode::Error
            } else {
                SeverityMode::Off
            };
            format!("{name}: {}", mode as u8)
        })
        .collect::<Vec<_>>()
        .join("\n  ");
    let config = format!("rules:\n  {rules}\n");

    autocorrect::config::load(&config)
        .map_err(|e| AppError::ConfigError(format!("Failed to apply autocorrect rules: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_standard() {
        let engine = EmbeddedAutocorrectEngine::new();
        let req = FormatRequest {
            text: "你好world,这是测试!".to_string(),
            formatter: Default::default(),
        };
        let resp = engine.format(&req).unwrap();

        assert!(resp.changed);
        assert_eq!(resp.formatted_text, "你好 world，这是测试！");
        assert_eq!(engine.name(), "autocorrect-embedded");
    }

    #[test]
    fn test_format_no_change() {
        let engine = EmbeddedAutocorrectEngine::new();
        let req = FormatRequest {
            text: "你好世界，这是测试。".to_string(),
            formatter: Default::default(),
        };
        let resp = engine.format(&req).unwrap();

        assert!(!resp.changed);
    }

    #[test]
    fn test_format_empty() {
        let engine = EmbeddedAutocorrectEngine::new();
        let req = FormatRequest {
            text: "".to_string(),
            formatter: Default::default(),
        };
        let resp = engine.format(&req).unwrap();

        assert_eq!(resp.formatted_text, "");
        assert!(!resp.changed);
    }

    #[test]
    fn test_disable_space_word_rule() {
        let engine = EmbeddedAutocorrectEngine::new();
        let mut formatter = crate::config::app_config::FormatterConfig::default();
        formatter.rules.space_word = false;
        formatter.rules.fullwidth = false;
        let req = FormatRequest {
            text: "你好world.".to_string(),
            formatter,
        };
        let resp = engine.format(&req).unwrap();

        assert_eq!(resp.formatted_text, "你好world.");
    }
}
