import { useFormatStore } from "../stores/format";
import { useEngineStore } from "../stores/engine";
import { useConfigStore } from "../stores/config";
import { readClipboard, writeClipboard } from "../lib/commands";
import { useEffect, useRef, useMemo, useState } from "react";
import appIcon from "../../src-tauri/icons/128x128.png";
import { useI18n } from "../i18n";
import { diffChars } from "../lib/diff";

interface FormatPageProps {
  onNavigate: (route: "format" | "settings" | "history") => void;
}

export function FormatPage({ onNavigate }: FormatPageProps) {
  const {
    inputText,
    result,
    isFormatting,
    error,
    setInputText,
    format,
    clear,
    clearError,
  } = useFormatStore();

  const { status: engineStatus, check: checkEngine } = useEngineStore();
  const { config } = useConfigStore();
  const { t } = useI18n();
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const toastTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const [actionToast, setActionToast] = useState<string | null>(null);

  const diffSegments = useMemo(() => {
    if (!result || !result.changed || !config?.diff_highlight) return null;
    return diffChars(result.original_text, result.formatted_text);
  }, [result, config?.diff_highlight]);
  const toastError =
    error === "EMPTY_INPUT"
      ? t("format.emptyInput")
      : error;
  const formatShortcutHint = isMacLike()
    ? t("format.copyShortcutMac")
    : t("format.copyShortcutWindows");

  useEffect(() => {
    checkEngine();
  }, [checkEngine]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    return () => {
      if (toastTimerRef.current) clearTimeout(toastTimerRef.current);
    };
  }, []);

  const showActionToast = (message: string) => {
    setActionToast(message);
    if (toastTimerRef.current) clearTimeout(toastTimerRef.current);
    toastTimerRef.current = setTimeout(() => setActionToast(null), 2000);
  };

  const handlePaste = async () => {
    try {
      const text = await readClipboard();
      setInputText(text);
      showActionToast(t("format.pasteSuccess"));
    } catch {
      setInputText(t("format.pasteError"));
    }
  };

  const handleCopy = async () => {
    if (!result?.formatted_text) return;
    try {
      await writeClipboard(result.formatted_text);
      showActionToast(t("format.copySuccess"));
    } catch {
      // Silently fail
    }
  };

  const handleFormat = async () => {
    await format();
    const { error, result } = useFormatStore.getState();
    if (error || !result) return;
    showActionToast(result.changed ? t("format.formatSuccess") : t("format.formatNoChangeToast"));
  };

  const handleClear = () => {
    const hasContent = inputText.length > 0 || !!result;
    clear();
    if (hasContent) {
      showActionToast(t("format.clearSuccess"));
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      void handleFormat();
    }
  };

  return (
    <div className="app-shell page-enter">
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
          <button
            onClick={handlePaste}
            className="tool-button"
            aria-label={t("format.paste")}
          >
            {t("format.paste")}
          </button>
          <button
            onClick={() => void handleFormat()}
            disabled={isFormatting || !inputText.trim()}
            className="tool-button tool-button-primary"
            aria-label={isFormatting ? t("format.formatting") : t("format.format")}
            aria-busy={isFormatting}
          >
            {isFormatting ? (
              <>
                <span className="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                {t("format.formatting")}
              </>
            ) : t("format.format")}
          </button>
          <button
            onClick={handleCopy}
            disabled={!result?.formatted_text}
            className="tool-button"
            aria-label={t("format.copy")}
          >
            {t("format.copy")}
          </button>
          <button
            onClick={handleClear}
            className="tool-button"
            aria-label={t("format.clear")}
          >
            {t("format.clear")}
          </button>

          <div className="toolbar-divider" />

          <button
            onClick={() => onNavigate("history")}
            className="tool-button"
            title={t("common.history")}
            aria-label={t("common.history")}
          >
            {t("format.history")}
          </button>
          <button
            onClick={() => onNavigate("settings")}
            className="tool-button"
            title={t("common.settings")}
            aria-label={t("common.settings")}
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
            <span className="meta-text" aria-live="polite">
              {inputText.length > 0 ? `${inputText.length} ${t("format.chars")}` : ""}
            </span>
          </div>
          <textarea
            ref={inputRef}
            value={inputText}
            onChange={(e) => setInputText(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t("format.inputPlaceholder")}
            className="editor-textarea placeholder-text-tertiary"
            aria-label={t("format.input")}
          />
        </div>

        {/* Output pane */}
        <div className="panel editor-panel flex-1">
          <div className="panel-header">
            <span className="panel-label">
              {t("format.output")}
            </span>
            {result && (
              <span className="meta-text" aria-live="polite">
                {result.formatted_text.length} {t("format.chars")}
              </span>
            )}
          </div>
          <div className="editor-output selectable" role="region" aria-label={t("format.output")}>
            {isFormatting ? (
              <div className="space-y-2 p-2">
                <div className="skeleton h-3 w-3/4" />
                <div className="skeleton h-3 w-1/2" />
                <div className="skeleton h-3 w-5/6" />
              </div>
            ) : result ? (
              <pre className="whitespace-pre-wrap font-sans m-0">
                {diffSegments ? (
                  diffSegments.map((seg, i) => (
                    <span
                      key={i}
                      className={
                        seg.type === "add" ? "diff-add"
                        : seg.type === "change" ? "diff-change"
                        : ""
                      }
                    >
                      {seg.text}
                    </span>
                  ))
                ) : (
                  result.formatted_text
                )}
              </pre>
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
        <div className="toast toast-danger" role="alert">
          <span>{toastError}</span>
          <button
            onClick={clearError}
            className="ml-1 text-white/60 hover:text-white transition-colors"
            aria-label={t("common.close")}
          >
            &#x2715;
          </button>
        </div>
      )}

      {actionToast && !toastError && (
        <div className="toast toast-success" role="status">
          {actionToast}
        </div>
      )}

    </div>
  );
}

function isMacLike(): boolean {
  return /Mac|iPhone|iPad|iPod/.test(navigator.platform);
}
