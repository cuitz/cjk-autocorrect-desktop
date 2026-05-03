import { useFormatStore } from "../stores/format";
import { useEngineStore } from "../stores/engine";
import { useConfigStore } from "../stores/config";
import { readClipboard, writeClipboard } from "../lib/commands";
import { useEffect } from "react";
import appIcon from "../../src-tauri/icons/128x128.png";
import { useI18n } from "../i18n";

interface FormatPageProps {
  onNavigate: (route: "format" | "settings" | "history") => void;
}

export function FormatPage({ onNavigate }: FormatPageProps) {
  const {
    inputText,
    result,
    isFormatting,
    error,
    mode,
    setInputText,
    setMode,
    setError,
    format,
    clear,
    clearError,
  } = useFormatStore();

  const { status: engineStatus, check: checkEngine } = useEngineStore();
  const { config } = useConfigStore();
  const { t } = useI18n();
  const toastError =
    error === "EMPTY_INPUT"
      ? t("format.emptyInput")
      : error === "STRICT_UNAVAILABLE"
        ? t("format.strictUnavailable")
        : error;
  const formatShortcutHint = isMacLike()
    ? t("format.copyShortcutMac")
    : t("format.copyShortcutWindows");

  useEffect(() => {
    checkEngine();
  }, [checkEngine]);

  useEffect(() => {
    if (config?.formatter.mode === "strict") {
      if (mode !== "standard") setMode("standard");
      return;
    }

    if (config?.formatter.mode && config.formatter.mode !== mode) {
      setMode(config.formatter.mode);
    }
  }, [config?.formatter.mode, mode, setMode]);

  const handlePaste = async () => {
    try {
      const text = await readClipboard();
      setInputText(text);
    } catch {
      setInputText(t("format.pasteError"));
    }
  };

  const handleCopy = async () => {
    if (!result?.formatted_text) return;
    try {
      await writeClipboard(result.formatted_text);
    } catch {
      // Silently fail
    }
  };

  const handleFormat = () => {
    format();
  };

  const handleStrictUnavailable = () => {
    setError("STRICT_UNAVAILABLE");
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      format();
    }
  };

  return (
    <div className="app-shell">
      {/* Toolbar */}
      <div className="app-toolbar">
        <div className="toolbar-brand">
          <img
            src={appIcon}
            alt=""
            className="h-5 w-5 rounded-[6px] shadow-sm"
            draggable={false}
          />
          <span className="toolbar-title">
            {t("appName")}
          </span>
        </div>
        <div className="toolbar-actions">
          {/* Mode selector — segmented control */}
          <div className="segmented-control">
            <button
              onClick={() => setMode("standard")}
              className={`segmented-option ${mode === "standard" ? "segmented-option-active" : ""}`}
            >
              {t("common.standard")}
            </button>
            <button
              onClick={handleStrictUnavailable}
              aria-disabled="true"
              title={t("format.strictUnavailable")}
              className="segmented-option segmented-option-disabled"
            >
              {t("common.strict")}
            </button>
          </div>

          <div className="toolbar-divider" />

          <button
            onClick={handlePaste}
            className="tool-button"
          >
            {t("format.paste")}
          </button>
          <button
            onClick={handleFormat}
            disabled={isFormatting || !inputText.trim()}
            className="tool-button tool-button-primary"
          >
            {isFormatting ? t("format.formatting") : t("format.format")}
          </button>
          <button
            onClick={handleCopy}
            disabled={!result?.formatted_text}
            className="tool-button"
          >
            {t("format.copy")}
          </button>
          <button
            onClick={clear}
            className="tool-button"
          >
            {t("format.clear")}
          </button>

          <div className="toolbar-divider" />

          <button
            onClick={() => onNavigate("history")}
            className="tool-button"
            title={t("common.history")}
          >
            {t("format.history")}
          </button>
          <button
            onClick={() => onNavigate("settings")}
            className="tool-button"
            title={t("common.settings")}
          >
            {t("format.settings")}
          </button>
        </div>
      </div>

      {/* Main content — split panes */}
      <div className="content-area flex flex-col gap-3">
        {/* Input pane */}
        <div className="panel editor-panel flex-1">
          <div className="panel-header">
            <span className="panel-label">
              {t("format.input")}
            </span>
            <span className="meta-text">
              {inputText.length > 0 ? `${inputText.length} ${t("format.chars")}` : ""}
            </span>
          </div>
          <textarea
            value={inputText}
            onChange={(e) => setInputText(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t("format.inputPlaceholder")}
            className="editor-textarea placeholder-text-tertiary"
          />
        </div>

        {/* Output pane */}
        <div className="panel editor-panel flex-1">
          <div className="panel-header">
            <span className="panel-label">
              {t("format.output")}
            </span>
            {result && (
              <span className="meta-text">
                {result.formatted_text.length} {t("format.chars")}
              </span>
            )}
          </div>
          <div className="editor-output selectable">
            {result ? (
              <pre className="whitespace-pre-wrap font-sans m-0">{result.formatted_text}</pre>
            ) : (
              <span className="empty-placeholder">
                {t("format.outputPlaceholder")}
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Status bar */}
      <div className="status-bar">
        <div>
          {result ? (
            <span className="status-group">
              <span className={result.changed ? "status-chip status-chip-warning" : "status-chip status-chip-success"}>
                {result.changed ? t("format.changed") : t("format.noChange")}
              </span>
              <span>{result.elapsed_ms}ms</span>
            </span>
          ) : (
            <span>{t("format.ready")}</span>
          )}
        </div>
        <div className="status-group">
          <span className="status-group">
            <span className={engineStatus?.autocorrect_installed ? "status-dot status-dot-success" : "status-dot"} />
            <span>{t("format.embeddedEngine")}</span>
          </span>
          <span className="opacity-50">{formatShortcutHint}</span>
        </div>
      </div>

      {/* Error toast */}
      {toastError && (
        <div className="toast toast-danger">
          <span>{toastError}</span>
          <button
            onClick={clearError}
            className="ml-1 text-white/60 hover:text-white transition-colors"
          >
            ✕
          </button>
        </div>
      )}

    </div>
  );
}

function isMacLike(): boolean {
  return /Mac|iPhone|iPad|iPod/.test(navigator.platform);
}
