use std::process::Command;
use std::time::Instant;

use crate::engine::types::{FormatMode, FormatRequest, FormatResponse, FormatterEngine};
use crate::errors::AppError;

/// AutoCorrect CLI engine — calls the local `autocorrect` binary via stdin/stdout.
pub struct AutocorrectCliEngine {
    binary_path: Option<String>,
}

impl AutocorrectCliEngine {
    /// Create engine with auto-detected binary path.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::with_custom_path(None)
    }

    /// Create engine with a custom binary path. Falls back to auto-detection if the
    /// custom path is None or the binary doesn't exist at that path.
    pub fn with_custom_path(custom_path: Option<String>) -> Self {
        let binary_path = match custom_path {
            Some(ref path) if !path.is_empty() => {
                // Use custom path if the file exists; otherwise fall back to auto-detect
                if std::path::Path::new(path).exists() {
                    Some(path.clone())
                } else {
                    tracing::warn!(
                        "Custom autocorrect path does not exist: {}, falling back to auto-detect",
                        path
                    );
                    Self::find_binary()
                }
            }
            _ => Self::find_binary(),
        };
        AutocorrectCliEngine { binary_path }
    }

    /// Locate the `autocorrect` binary on the system PATH.
    fn find_binary() -> Option<String> {
        which_binary("autocorrect")
    }

    #[allow(dead_code)]
    pub fn is_installed(&self) -> bool {
        self.binary_path.is_some()
    }

    pub fn binary_path(&self) -> Option<&str> {
        self.binary_path.as_deref()
    }

    /// Installation instructions per platform.
    pub fn install_hint() -> String {
        if cfg!(target_os = "macos") {
            "未检测到 autocorrect，请先安装：\n\n\
             方式一（推荐）：brew install autocorrect\n\
             方式二：cargo install autocorrect\n\n\
             安装后重启应用即可。"
                .to_string()
        } else if cfg!(target_os = "windows") {
            "未检测到 autocorrect，请先安装：\n\n\
             方式一：scoop install autocorrect\n\
             方式二：cargo install autocorrect\n\n\
             安装后重启应用即可。"
                .to_string()
        } else {
            "未检测到 autocorrect，请先安装：\n\n\
             cargo install autocorrect\n\n\
             安装后重启应用即可。"
                .to_string()
        }
    }
}

impl FormatterEngine for AutocorrectCliEngine {
    fn format(&self, req: &FormatRequest) -> Result<FormatResponse, AppError> {
        let binary = self
            .binary_path
            .as_ref()
            .ok_or_else(|| AppError::EngineUnavailable(Self::install_hint()))?;

        let start = Instant::now();

        let mut cmd = Command::new(binary);
        cmd.arg("--stdin");

        if matches!(req.mode, FormatMode::Strict) {
            cmd.arg("--strict");
        }

        // Inject extended PATH on macOS so autocorrect subprocess can find its dependencies
        if cfg!(target_os = "macos") {
            inject_macos_path(&mut cmd);
        }

        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| AppError::FormatFailed(format!("Failed to spawn autocorrect: {}", e)))?;

        // Write input text to stdin
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(req.text.as_bytes());
            // stdin is dropped here, closing the pipe
        }

        let output = child.wait_with_output().map_err(|e| {
            AppError::FormatFailed(format!("Failed to read autocorrect output: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::FormatFailed(format!(
                "autocorrect exited with code {}: {}",
                output.status.code().unwrap_or(-1),
                stderr.trim()
            )));
        }

        let formatted = String::from_utf8_lossy(&output.stdout).to_string();

        // autocorrect adds a trailing newline to stdout output; trim it
        let formatted = formatted.trim_end_matches('\n').to_string();

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
        "autocorrect-cli"
    }

    fn is_available(&self) -> bool {
        self.binary_path.is_some()
    }
}

/// Find a binary on the system PATH.
///
/// On macOS, GUI apps launched from Finder/Dock do not inherit the shell's PATH,
/// so Homebrew/MacPorts binaries (e.g. `/opt/homebrew/bin`) are invisible.
/// We explicitly inject common third-party paths before running `which`.
fn which_binary(name: &str) -> Option<String> {
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("where");
        c.arg(name);
        c
    } else {
        let mut c = Command::new("which");
        c.arg(name);
        if cfg!(target_os = "macos") {
            inject_macos_path(&mut c);
        }
        c
    };

    match cmd.output() {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if path.is_empty() {
                None
            } else {
                // On some systems `which` may return multiple lines; take the first
                Some(path.lines().next().unwrap_or(&path).to_string())
            }
        }
        _ => None,
    }
}

/// Inject common macOS third-party binary paths into a Command's PATH env.
fn inject_macos_path(cmd: &mut Command) {
    const MACOS_EXTRA_PATHS: &[&str] = &[
        "/opt/homebrew/bin", // Apple Silicon Homebrew
        "/usr/local/bin",    // Intel Homebrew / manual installs
        "/opt/homebrew/sbin",
        "/usr/local/sbin",
        "/opt/local/bin", // MacPorts
        "/opt/local/sbin",
        "/nix/var/nix/profiles/default/bin", // Nix
    ];

    if let Ok(existing_path) = std::env::var("PATH") {
        let extended = format!("{}:{}", MACOS_EXTRA_PATHS.join(":"), existing_path);
        cmd.env("PATH", extended);
    } else {
        cmd.env("PATH", MACOS_EXTRA_PATHS.join(":"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_binary() {
        let path = which_binary("autocorrect");
        if path.is_none() {
            eprintln!("Skipping test: autocorrect not installed");
        }
    }

    #[test]
    fn test_format_standard() {
        let engine = AutocorrectCliEngine::new();
        if !engine.is_available() {
            eprintln!("Skipping test: autocorrect not installed");
            return;
        }

        let req = FormatRequest {
            text: "你好world,这是测试!".to_string(),
            mode: FormatMode::Standard,
        };
        let resp = engine.format(&req).unwrap();
        assert!(resp.changed);
        assert_eq!(resp.formatted_text, "你好 world，这是测试！");
        assert_eq!(engine.name(), "autocorrect-cli");
    }

    #[test]
    fn test_format_no_change() {
        let engine = AutocorrectCliEngine::new();
        if !engine.is_available() {
            eprintln!("Skipping test: autocorrect not installed");
            return;
        }

        let req = FormatRequest {
            text: "你好世界，这是测试。".to_string(),
            mode: FormatMode::Standard,
        };
        let resp = engine.format(&req).unwrap();
        assert!(!resp.changed);
    }

    #[test]
    fn test_format_empty() {
        let engine = AutocorrectCliEngine::new();
        if !engine.is_available() {
            eprintln!("Skipping test: autocorrect not installed");
            return;
        }

        let req = FormatRequest {
            text: "".to_string(),
            mode: FormatMode::Standard,
        };
        let resp = engine.format(&req).unwrap();
        assert!(!resp.changed);
    }

    #[test]
    fn test_install_hint() {
        let hint = AutocorrectCliEngine::install_hint();
        assert!(hint.contains("安装"));
    }
}
