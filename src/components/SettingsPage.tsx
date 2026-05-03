import { useEffect, useState, useRef } from "react";
import { useConfigStore } from "../stores/config";
import { useEngineStore } from "../stores/engine";
import type { AppConfig } from "../lib/commands";
import { type TranslationKey, useI18n } from "../i18n";

export function SettingsPage({ onBack }: { onBack: () => void }) {
  const { config, isLoading, isSaving, error, load, save, clearError } = useConfigStore();
  const { status: engineStatus, check: checkEngine } = useEngineStore();
  const [form, setForm] = useState<AppConfig | null>(null);
  const [showSaved, setShowSaved] = useState(false);
  const [isRecording, setIsRecording] = useState(false);
  const [shortcutError, setShortcutError] = useState<string | null>(null);
  const savedTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const { t } = useI18n();

  useEffect(() => {
    load();
    checkEngine();
  }, [load, checkEngine]);

  useEffect(() => {
    if (config) {
      setForm({ ...config });
    }
  }, [config]);

  useEffect(() => {
    return () => {
      if (savedTimerRef.current) clearTimeout(savedTimerRef.current);
    };
  }, []);

  if (isLoading || !form) {
    return (
      <div className="flex items-center justify-center h-full bg-surface">
        <span className="text-[14px] text-text-tertiary">{t("settings.loading")}</span>
      </div>
    );
  }

  const handleSave = async () => {
    if (!form) return;
    setShortcutError(null);
    try {
      await save(form);
      setShowSaved(true);
      if (savedTimerRef.current) clearTimeout(savedTimerRef.current);
      savedTimerRef.current = setTimeout(() => setShowSaved(false), 2000);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (msg.includes("ShortcutRegistration") || msg.includes("shortcut")) {
        setShortcutError(t("settings.saveFailedShortcut", { shortcut: form.shortcut }));
      }
    }
  };

  const updateForm = (partial: Partial<AppConfig>) => {
    setForm((prev) => (prev ? { ...prev, ...partial } : prev));
  };

  const updateFormatter = (partial: Partial<AppConfig["formatter"]>) => {
    setForm((prev) =>
      prev ? { ...prev, formatter: { ...prev.formatter, ...partial } } : prev
    );
  };

  const hasChanges = config && form && JSON.stringify(config) !== JSON.stringify(form);

  return (
    <div className="app-shell">
      {/* Header */}
      <div className="app-toolbar">
        <div className="toolbar-brand">
          <button
            onClick={onBack}
            className="tool-button"
          >
            {t("common.back")}
          </button>
          <span className="toolbar-divider" />
          <h1 className="toolbar-title">{t("common.settings")}</h1>
        </div>
        <button
          onClick={handleSave}
          disabled={isSaving || !hasChanges}
          className="tool-button tool-button-primary"
        >
          {isSaving ? t("common.saving") : t("common.save")}
        </button>
      </div>

      {/* Settings form */}
      <div className="content-area content-area-scroll space-y-4">
        {/* General */}
        <Section title={t("settings.general")}>
          <Field label={t("settings.shortcut")} hint={t("settings.recordHint")}>
            <ShortcutInput
              value={form.shortcut}
              isRecording={isRecording}
              t={t}
              onStartRecord={() => { setIsRecording(true); setShortcutError(null); }}
              onStopRecord={() => setIsRecording(false)}
              onChange={(shortcut) => {
                updateForm({ shortcut });
                setIsRecording(false);
              }}
            />
            {shortcutError && (
              <p className="mt-1 text-[12px] text-danger">{shortcutError}</p>
            )}
          </Field>

          <ToggleField
            label={t("settings.autostart")}
            checked={form.auto_start}
            onChange={(v) => updateForm({ auto_start: v })}
          />

          <ToggleField
            label={t("settings.closeToTray")}
            checked={form.close_to_tray}
            onChange={(v) => updateForm({ close_to_tray: v })}
          />
        </Section>

        {/* Appearance */}
        <Section title={t("settings.appearance")}>
          <Field label={t("settings.theme")}>
            <SegmentedControl
              options={[
                { value: "system", label: t("settings.themeSystem") },
                { value: "light", label: t("settings.themeLight") },
                { value: "dark", label: t("settings.themeDark") },
              ]}
              value={form.theme}
              onChange={(v) => updateForm({ theme: v })}
            />
          </Field>
          <Field label={t("settings.language")}>
            <SegmentedControl
              options={[
                { value: "system", label: t("settings.languageSystem") },
                { value: "zh-CN", label: t("settings.languageZh") },
                { value: "en", label: t("settings.languageEn") },
                { value: "ja", label: t("settings.languageJa") },
                { value: "ko", label: t("settings.languageKo") },
              ]}
              value={form.language ?? "system"}
              onChange={(v) => updateForm({ language: v })}
            />
          </Field>
        </Section>

        {/* History */}
        <Section title={t("settings.history")}>
          <ToggleField
            label={t("settings.historyEnabled")}
            checked={form.history_enabled}
            onChange={(v) => updateForm({ history_enabled: v })}
          />
        </Section>

        {/* Formatter */}
        <Section title={t("settings.formatter")}>
          <Field label={t("settings.formatMode")}>
            <SegmentedControl
              options={[
                { value: "standard", label: t("common.standardMode") },
                { value: "strict", label: t("common.strictMode") },
              ]}
              value={form.formatter.mode}
              onChange={(v) => updateFormatter({ mode: v })}
            />
            <p className="mt-1.5 text-[11px] text-text-tertiary leading-relaxed">
              {form.formatter.mode === "standard"
                ? t("settings.standardHelp")
                : t("settings.strictHelp")}
            </p>
          </Field>
          <Field label={t("settings.autocorrectPath")} hint={t("settings.autocorrectPathHint")}>
            <div className="flex gap-2">
              <input
                type="text"
                value={form.formatter.autocorrect_path || ""}
                onChange={(e) => updateFormatter({ autocorrect_path: e.target.value || null })}
                placeholder={engineStatus?.autocorrect_path || t("settings.autocorrectPathPlaceholder")}
                className="field-control flex-1 min-w-0"
              />
              {form.formatter.autocorrect_path && (
                <button
                  onClick={() => updateFormatter({ autocorrect_path: null })}
                  className="tool-button"
                  title={t("settings.clearCustomPath")}
                >
                  {t("settings.reset")}
                </button>
              )}
            </div>
            <p className="mt-1 text-[11px] text-text-tertiary leading-relaxed">
              {engineStatus?.autocorrect_installed ? (
                <span className="text-success">
                  {t("settings.autocorrectDetected", { path: engineStatus.autocorrect_path || "" })}
                </span>
              ) : (
                <span className="text-danger">
                  {t("settings.autocorrectMissing")}
                </span>
              )}
            </p>
          </Field>
        </Section>
      </div>

      {/* Error toast */}
      {error && (
        <div className="toast toast-danger">
          <span>{error}</span>
          <button onClick={clearError} className="ml-1 text-white/60 hover:text-white transition-colors">
            ✕
          </button>
        </div>
      )}

      {/* Save success toast */}
      {showSaved && !error && (
        <div className="toast toast-success">
          {t("settings.saved")}
        </div>
      )}
    </div>
  );
}

// --- Segmented Control ---
function SegmentedControl({
  options,
  value,
  onChange,
}: {
  options: { value: string; label: string }[];
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <div className="segmented-control w-full">
      {options.map((opt) => (
        <button
          key={opt.value}
          onClick={() => onChange(opt.value)}
          className={`segmented-option flex-1 ${value === opt.value ? "segmented-option-active" : ""}`}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}

// --- Shortcut Input Component ---
function ShortcutInput({
  value,
  isRecording,
  t,
  onStartRecord,
  onStopRecord,
  onChange,
}: {
  value: string;
  isRecording: boolean;
  t: (key: TranslationKey, vars?: Record<string, string | number>) => string;
  onStartRecord: () => void;
  onStopRecord: () => void;
  onChange: (shortcut: string) => void;
}) {
  // Use refs to hold latest callbacks without re-triggering useEffect
  const onChangeRef = useRef(onChange);
  const onStopRecordRef = useRef(onStopRecord);
  onChangeRef.current = onChange;
  onStopRecordRef.current = onStopRecord;

  // Use window-level keydown listener to reliably capture modifier keys
  useEffect(() => {
    if (!isRecording) return;

    // Guard against key-repeat causing multiple callbacks before React re-renders
    let committed = false;

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      if (committed) return;

      const modifierKeys = ["Control", "Alt", "Shift", "Meta", "Command"];
      if (modifierKeys.includes(e.key)) return;

      const parts: string[] = [];
      if (e.metaKey) parts.push("Super");
      if (e.ctrlKey) parts.push("Control");
      if (e.altKey) parts.push("Alt");
      if (e.shiftKey) parts.push("Shift");

      if (e.key === "Escape") {
        committed = true;
        onStopRecordRef.current();
        return;
      }

      const key = shortcutKeyFromEvent(e);
      if (!key) {
        return;
      }

      parts.push(key);
      committed = true;
      onChangeRef.current(parts.join("+"));
      onStopRecordRef.current();
    };

    window.addEventListener("keydown", handleKeyDown, true);
    return () => window.removeEventListener("keydown", handleKeyDown, true);
  }, [isRecording]);

  const displayValue = formatShortcutForDisplay(value);

  return (
    <div className="flex gap-2">
      {isRecording ? (
        <>
          <div
            className="field-control flex-1 min-w-0 text-left border-accent bg-accent-subtle text-accent flex items-center justify-between gap-3"
            aria-live="polite"
          >
            <span className="truncate">{t("settings.pressShortcut")}</span>
            <span className="text-[11px] text-text-tertiary shrink-0">{t("settings.escToCancel")}</span>
          </div>
          <button
            onClick={onStopRecord}
            className="tool-button"
            title={t("settings.cancelRecording")}
          >
            {t("common.cancel")}
          </button>
        </>
      ) : (
        <button
          onClick={onStartRecord}
          className="field-control flex-1 min-w-0 text-left hover:border-text-tertiary"
        >
          {displayValue || t("settings.clickToRecord")}
        </button>
      )}
      {value && !isRecording && (
        <button
          onClick={onStartRecord}
          className="tool-button"
          title={t("settings.rerecord")}
        >
          {t("settings.record")}
        </button>
      )}
    </div>
  );
}

function shortcutKeyFromEvent(e: KeyboardEvent): string | null {
  const code = e.code;

  if (/^Key[A-Z]$/.test(code)) return code;
  if (/^Digit[0-9]$/.test(code)) return code;
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code;

  const codeMap: Record<string, string> = {
    Backquote: "Backquote",
    Backslash: "Backslash",
    BracketLeft: "BracketLeft",
    BracketRight: "BracketRight",
    Comma: "Comma",
    Equal: "Equal",
    Minus: "Minus",
    Period: "Period",
    Quote: "Quote",
    Semicolon: "Semicolon",
    Slash: "Slash",
    Space: "Space",
    Backspace: "Backspace",
    Delete: "Delete",
    Enter: "Enter",
    Tab: "Tab",
    ArrowUp: "ArrowUp",
    ArrowDown: "ArrowDown",
    ArrowLeft: "ArrowLeft",
    ArrowRight: "ArrowRight",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
    Insert: "Insert",
  };

  return codeMap[code] ?? null;
}

function formatShortcutForDisplay(shortcut: string): string {
  const keyLabels: Record<string, string> = {
    Backquote: "`",
    Backslash: "\\",
    BracketLeft: "[",
    BracketRight: "]",
    Comma: ",",
    Equal: "=",
    Minus: "-",
    Period: ".",
    Quote: "'",
    Semicolon: ";",
    Slash: "/",
    Space: "Space",
    Enter: "Enter",
    Return: "Enter",
    ArrowUp: "↑",
    Up: "↑",
    ArrowDown: "↓",
    Down: "↓",
    ArrowLeft: "←",
    Left: "←",
    ArrowRight: "→",
    Right: "→",
  };

  return shortcut
    .split("+")
    .map((part) => {
      const token = part.trim();
      if (token === "CommandOrControl") return isMacLike() ? "⌘" : "Ctrl";
      if (token === "Command" || token === "Cmd" || token === "Super") return "⌘";
      if (token === "Control" || token === "Ctrl") return isMacLike() ? "⌃" : "Ctrl";
      if (token === "Alt" || token === "Option") return "⌥";
      if (token === "Shift") return "⇧";
      if (/^Key[A-Z]$/.test(token)) return token.slice(3);
      if (/^Digit[0-9]$/.test(token)) return token.slice(5);
      return keyLabels[token] ?? token;
    })
    .join(" + ");
}

function isMacLike(): boolean {
  return /Mac|iPhone|iPad|iPod/.test(navigator.platform);
}

// --- Shared Components ---
function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h2 className="text-[11px] font-semibold text-text-tertiary uppercase tracking-[0.1em] mb-2 px-0.5">
        {title}
      </h2>
      <div className="panel p-3 space-y-3">
        {children}
      </div>
    </div>
  );
}

function Field({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <div>
      <div className="flex items-baseline justify-between mb-1">
        <label className="text-[12px] font-medium text-text-secondary">{label}</label>
        {hint && (
          <span className="text-[11px] text-text-tertiary">{hint}</span>
        )}
      </div>
      {children}
    </div>
  );
}

function ToggleField({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-[13px] text-text-primary">{label}</span>
      <button
        onClick={() => onChange(!checked)}
        className={`relative w-9 h-[20px] rounded-full transition-colors duration-200 ${
          checked ? "bg-accent" : "bg-border"
        }`}
      >
        <span
          className={`absolute top-[2px] left-[2px] w-4 h-4 rounded-full bg-white shadow-sm transition-transform duration-200 ${
            checked ? "translate-x-4" : ""
          }`}
        />
      </button>
    </div>
  );
}
