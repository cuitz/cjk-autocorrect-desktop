use std::time::Instant;

use crate::engine::types::{FormatMode, FormatRequest, FormatResponse, FormatterEngine};
use crate::errors::AppError;

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
        let formatted = match req.mode {
            FormatMode::Standard => autocorrect::format(&req.text),
            // The upstream Rust API does not expose a separate strict formatter.
            // CLI strict mode only changes lint exit behavior, so formatting stays shared.
            FormatMode::Strict => autocorrect::format(&req.text),
        };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::FormatMode;

    #[test]
    fn test_format_standard() {
        let engine = EmbeddedAutocorrectEngine::new();
        let req = FormatRequest {
            text: "你好world,这是测试!".to_string(),
            mode: FormatMode::Standard,
        };
        let resp = engine.format(&req).unwrap();

        assert!(resp.changed);
        assert_eq!(resp.formatted_text, "你好 world，这是测试！");
        assert_eq!(engine.name(), "autocorrect-embedded");
    }

    #[test]
    fn test_format_strict_uses_embedded_formatter() {
        let engine = EmbeddedAutocorrectEngine::new();
        let req = FormatRequest {
            text: "你好world,这是测试!".to_string(),
            mode: FormatMode::Strict,
        };
        let resp = engine.format(&req).unwrap();

        assert!(resp.changed);
        assert_eq!(resp.formatted_text, "你好 world，这是测试！");
    }

    #[test]
    fn test_format_no_change() {
        let engine = EmbeddedAutocorrectEngine::new();
        let req = FormatRequest {
            text: "你好世界，这是测试。".to_string(),
            mode: FormatMode::Standard,
        };
        let resp = engine.format(&req).unwrap();

        assert!(!resp.changed);
    }

    #[test]
    fn test_format_empty() {
        let engine = EmbeddedAutocorrectEngine::new();
        let req = FormatRequest {
            text: "".to_string(),
            mode: FormatMode::Standard,
        };
        let resp = engine.format(&req).unwrap();

        assert_eq!(resp.formatted_text, "");
        assert!(!resp.changed);
    }
}
