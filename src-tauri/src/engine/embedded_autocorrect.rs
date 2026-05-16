use std::{
    sync::{Mutex, OnceLock},
    time::Instant,
};

use crate::engine::types::{FormatRequest, FormatResponse, FormatterEngine};
use crate::errors::AppError;
use autocorrect::config::SeverityMode;

static AUTOCORRECT_CONFIG_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static SPELLCHECK_WORDS_LOADED: OnceLock<()> = OnceLock::new();
/// Caches the last applied rule signature so repeat calls with identical rules
/// skip the YAML serialise + parse round-trip inside `autocorrect::config::load`.
static LAST_APPLIED_RULES: OnceLock<Mutex<Option<u64>>> = OnceLock::new();

const BUILT_IN_SPELLCHECK_WORDS: &[&str] = &[
    "api = API",
    "css = CSS",
    "github = GitHub",
    "html = HTML",
    "ios = iOS",
    "ipad = iPad",
    "iphone = iPhone",
    "javascript = JavaScript",
    "macos = macOS",
    "typescript = TypeScript",
    "url = URL",
    "wifi = Wi-Fi",
];

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
    ensure_spellcheck_words_loaded()?;

    let signature = rules_signature(&formatter.rules);
    let cache = LAST_APPLIED_RULES.get_or_init(|| Mutex::new(None));
    let mut guard = cache
        .lock()
        .map_err(|e| AppError::FormatFailed(format!("Rules cache lock poisoned: {}", e)))?;
    if *guard == Some(signature) {
        return Ok(());
    }

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

    *guard = Some(signature);
    Ok(())
}

/// Compact 64-bit signature of the 12 rule toggles. Stable across runs.
fn rules_signature(rules: &crate::config::app_config::FormatterRules) -> u64 {
    let mut bits: u64 = 0;
    for (idx, (_, enabled)) in rules.autocorrect_rules().iter().enumerate() {
        if *enabled {
            bits |= 1 << idx;
        }
    }
    bits
}

fn ensure_spellcheck_words_loaded() -> Result<(), AppError> {
    if SPELLCHECK_WORDS_LOADED.get().is_some() {
        return Ok(());
    }

    let words = BUILT_IN_SPELLCHECK_WORDS
        .iter()
        .map(|word| format!("    - {word}"))
        .collect::<Vec<_>>()
        .join("\n");
    let config = format!("spellcheck:\n  words:\n{words}\n");

    autocorrect::config::load(&config)
        .map_err(|e| AppError::ConfigError(format!("Failed to load spellcheck words: {}", e)))?;
    let _ = SPELLCHECK_WORDS_LOADED.set(());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::app_config::{FormatterConfig, FormatterRules};

    fn format_with_rules(text: &str, configure_rules: impl FnOnce(&mut FormatterRules)) -> String {
        let engine = EmbeddedAutocorrectEngine::new();
        let mut formatter = FormatterConfig {
            rules: FormatterRules {
                space_word: false,
                space_punctuation: false,
                space_bracket: false,
                space_dash: false,
                space_backticks: false,
                space_dollar: false,
                fullwidth: false,
                halfwidth_word: false,
                halfwidth_punctuation: false,
                no_space_fullwidth: false,
                no_space_fullwidth_quote: false,
                spellcheck: false,
            },
        };
        configure_rules(&mut formatter.rules);
        let req = FormatRequest {
            text: text.to_string(),
            formatter,
        };

        engine.format(&req).unwrap().formatted_text
    }

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

    #[test]
    fn test_rule_examples_match_embedded_engine_behavior() {
        assert_eq!(
            format_with_rules("hello你好", |rules| rules.space_word = true),
            "hello 你好"
        );
        assert_eq!(
            format_with_rules("今天Version2发布", |rules| rules.space_word = true),
            "今天 Version2 发布"
        );
        assert_eq!(
            format_with_rules("中文+中文", |rules| rules.space_punctuation = true),
            "中文 + 中文"
        );
        assert_eq!(
            format_with_rules("请参考API(v2)文档", |rules| rules.space_bracket = true),
            "请参考API(v2) 文档"
        );
        assert_eq!(
            format_with_rules("你好-世界", |rules| rules.space_dash = true),
            "你好 - 世界"
        );
        assert_eq!(
            format_with_rules("代码`code`代码", |rules| rules.space_backticks = true),
            "代码 `code` 代码"
        );
        assert_eq!(
            format_with_rules("令$x+y=1$成立", |rules| rules.space_dollar = true),
            "令 $x+y=1$ 成立"
        );
        assert_eq!(
            format_with_rules("你好,世界!", |rules| rules.fullwidth = true),
            "你好，世界！"
        );
        assert_eq!(
            format_with_rules("ＡＰＩ ２０２４", |rules| rules.halfwidth_word =
                true),
            "API 2024"
        );
        assert_eq!(
            format_with_rules("型号Ａ１２３升级到Ｂ２", |rules| {
                rules.halfwidth_word = true
            }),
            "型号A123升级到B2"
        );
        assert_eq!(
            format_with_rules("Said：Come and，Join us！", |rules| {
                rules.halfwidth_punctuation = true
            }),
            "Said: Come and, Join us!"
        );
        assert_eq!(
            format_with_rules("你好 ，世界 ！", |rules| rules.no_space_fullwidth =
                true),
            "你好，世界！"
        );
        assert_eq!(
            format_with_rules("你好 “Quote” 世界", |rules| {
                rules.no_space_fullwidth_quote = true
            }),
            "你好“Quote”世界"
        );

        assert_eq!(
            format_with_rules("支持 Rust,Python,Go", |rules| rules.space_punctuation =
                true),
            "支持 Rust,Python,Go"
        );
        assert_eq!(
            format_with_rules("我们用Javascript开发", |rules| {
                rules.spellcheck = true;
                rules.space_word = true;
            }),
            "我们用 JavaScript 开发"
        );
    }

    #[test]
    fn test_spellcheck_rule_uses_built_in_word_list() {
        assert_eq!(
            format_with_rules("开放 IOS 接口", |rules| rules.spellcheck = true),
            "开放 iOS 接口"
        );
        assert_eq!(
            format_with_rules("我们用 Javascript 和 github 开发", |rules| {
                rules.spellcheck = true;
                rules.space_word = true;
            }),
            "我们用 JavaScript 和 GitHub 开发"
        );
        assert_eq!(
            format_with_rules("请访问 url 并打开 wifi", |rules| rules.spellcheck =
                true),
            "请访问 URL 并打开 Wi-Fi"
        );
    }

    #[test]
    fn test_spellcheck_rule_can_be_disabled() {
        assert_eq!(
            format_with_rules("开放 IOS 接口", |rules| rules.spellcheck = false),
            "开放 IOS 接口"
        );
    }
}
