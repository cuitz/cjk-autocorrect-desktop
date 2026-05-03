use serde::Serialize;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Format engine unavailable: {0}")]
    EngineUnavailable(String),

    #[error("Format failed: {0}")]
    FormatFailed(String),

    #[error("Clipboard error: {0}")]
    ClipboardError(String),

    #[error("Shortcut registration error: {0}")]
    ShortcutRegistrationError(String),

    #[error("Update error: {0}")]
    UpdateError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct ErrorResponse {
            kind: String,
            message: String,
        }

        let kind = match self {
            AppError::ConfigError(_) => "ConfigError",
            AppError::EngineUnavailable(_) => "EngineUnavailable",
            AppError::FormatFailed(_) => "FormatFailed",
            AppError::ClipboardError(_) => "ClipboardError",
            AppError::ShortcutRegistrationError(_) => "ShortcutRegistrationError",
            AppError::UpdateError(_) => "UpdateError",
            AppError::UnknownError(_) => "UnknownError",
        };

        ErrorResponse {
            kind: kind.to_string(),
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::UnknownError(err.to_string())
    }
}
