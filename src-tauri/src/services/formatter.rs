use crate::dto::{FormatResultDto, FormatTextDto};
use crate::engine::embedded_autocorrect::EmbeddedAutocorrectEngine;
use crate::engine::types::FormatterEngine;
use crate::errors::AppError;

pub struct FormatterService {
    autocorrect: EmbeddedAutocorrectEngine,
}

impl FormatterService {
    pub fn new() -> Self {
        FormatterService {
            autocorrect: EmbeddedAutocorrectEngine::new(),
        }
    }

    /// Returns true when the embedded autocorrect engine is available.
    pub fn is_autocorrect_available(&self) -> bool {
        self.autocorrect.is_available()
    }

    pub fn engine_name(&self) -> &str {
        self.autocorrect.name()
    }

    pub fn install_hint(&self) -> Option<String> {
        None
    }

    pub fn format(&self, request: FormatTextDto) -> Result<FormatResultDto, AppError> {
        let format_request = request.into();
        let response = self.autocorrect.format(&format_request)?;
        Ok(FormatResultDto::from(response))
    }
}
