use crate::dto::{FormatResultDto, FormatTextDto};
use crate::engine::autocorrect_cli::AutocorrectCliEngine;
use crate::engine::types::FormatterEngine;
use crate::errors::AppError;

pub struct FormatterService {
    autocorrect: AutocorrectCliEngine,
}

impl FormatterService {
    /// Create service with auto-detected binary path.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::with_custom_path(None)
    }

    /// Create service with a custom autocorrect binary path.
    pub fn with_custom_path(custom_path: Option<String>) -> Self {
        FormatterService {
            autocorrect: AutocorrectCliEngine::with_custom_path(custom_path),
        }
    }

    /// Returns true if autocorrect CLI is available.
    pub fn is_autocorrect_available(&self) -> bool {
        self.autocorrect.is_available()
    }

    /// Returns the autocorrect binary path if found.
    pub fn autocorrect_path(&self) -> Option<&str> {
        self.autocorrect.binary_path()
    }

    /// Returns installation hint if autocorrect is not installed.
    pub fn install_hint(&self) -> Option<String> {
        if self.autocorrect.is_available() {
            None
        } else {
            Some(AutocorrectCliEngine::install_hint())
        }
    }

    pub fn format(&self, request: FormatTextDto) -> Result<FormatResultDto, AppError> {
        if !self.autocorrect.is_available() {
            return Err(AppError::EngineUnavailable(
                AutocorrectCliEngine::install_hint(),
            ));
        }

        let format_request = request.into();
        let response = self.autocorrect.format(&format_request)?;
        Ok(FormatResultDto::from(response))
    }
}
