use crate::config::app_config::FormatterConfig;
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

#[derive(Debug)]
pub struct FormatRequest {
    pub text: String,
    pub formatter: FormatterConfig,
}

#[derive(Debug)]
pub struct FormatResponse {
    pub original_text: String,
    pub formatted_text: String,
    pub changed: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub line: usize,
    pub col: usize,
    pub message: String,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Warning,
    Error,
    Information,
}

pub trait FormatterEngine: Send + Sync {
    fn format(&self, req: &FormatRequest) -> Result<FormatResponse, AppError>;
    #[allow(dead_code)]
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
}
