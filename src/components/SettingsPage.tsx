import { useEffect, useState, useRef } from "react";
import { useConfigStore } from "../stores/config";
import { useEngineStore } from "../stores/engine";
import type { AppConfig } from "../lib/commands";
import { normalizeCommandError } from "../lib/errors";
import { type TranslationKey, useI18n } from "../i18n";

type FormatterRuleKey = keyof AppConfig["formatter"]["rules"];

const DEFAULT_FORMATTER_RULES: AppConfig["formatter"]["rules"] = {
  space_word: true,
  space_punctuation: true,
  space_bracket: true,
  space_dash: true,
  space_backticks: true,
  space_dollar: false,
  fullwidth: true,
  halfwidth_word: true,
  halfwidth_punctuation: true,
  no_space_fullwidth: true,
  no_space_fullwidth_quote: true,
  spellcheck: false,
};

const FORMATTER_RULES: Array<{
  key: FormatterRuleKey;
  ruleName: string;
  labelKey: TranslationKey;
  descriptionKey: TranslationKey;
  beforeExample: string;
  afterExample: string;
}> = [
  {
    key: "space_word",
    ruleName: "space-word",
    labelKey: "settings.ruleSpaceWord",
    descriptionKey: "settings.ruleSpaceWordDesc",
    beforeExample: "今天Version2发布",
    afterExample: "今天 Version2 发布",
  },
  {
    key: "space_punctuation",
    ruleName: "space-punctuation",
    labelKey: "settings.ruleSpacePunctuation",
    descriptionKey: "settings.ruleSpacePunctuationDesc",
    beforeExample: "中文+中文",
    afterExample: "中文 + 中文",
  },
  {
    key: "space_bracket",
    ruleName: "space-bracket",
    labelKey: "settings.ruleSpaceBracket",
    descriptionKey: "settings.ruleSpaceBracketDesc",
    beforeExample: "请参考API(v2)文档",
    afterExample: "请参考 API(v2) 文档",
  },
  {
    key: "space_dash",
    ruleName: "space-dash",
    labelKey: "settings.ruleSpaceDash",
    descriptionKey: "settings.ruleSpaceDashDesc",
    beforeExample: "这是中英混排-示例",
    afterExample: "这是中英混排 - 示例",
  },
  {
    key: "space_backticks",
    ruleName: "space-backticks",
    labelKey: "settings.ruleSpaceBackticks",
    descriptionKey: "settings.ruleSpaceBackticksDesc",
    beforeExample: "代码`code`代码",
    afterExample: "代码 `code` 代码",
  },
  {
    key: "space_dollar",
    ruleName: "space-dollar",
    labelKey: "settings.ruleSpaceDollar",
    descriptionKey: "settings.ruleSpaceDollarDesc",
    beforeExample: "公式$E=mc^2$很经典",
    afterExample: "公式 $E=mc^2$ 很经典",
  },
  {
    key: "fullwidth",
    ruleName: "fullwidth",
    labelKey: "settings.ruleFullwidth",
    descriptionKey: "settings.ruleFullwidthDesc",
    beforeExample: "你好, world!",
    afterExample: "你好，world！",
  },
  {
    key: "halfwidth_word",
    ruleName: "halfwidth-word",
    labelKey: "settings.ruleHalfwidthWord",
    descriptionKey: "settings.ruleHalfwidthWordDesc",
    beforeExample: "型号Ａ１２３升级到Ｂ２",
    afterExample: "型号A123升级到B2",
  },
  {
    key: "halfwidth_punctuation",
    ruleName: "halfwidth-punctuation",
    labelKey: "settings.ruleHalfwidthPunctuation",
    descriptionKey: "settings.ruleHalfwidthPunctuationDesc",
    beforeExample: "Use function（arg1，arg2）；",
    afterExample: "Use function(arg1, arg2);",
  },
  {
    key: "no_space_fullwidth",
    ruleName: "no-space-fullwidth",
    labelKey: "settings.ruleNoSpaceFullwidth",
    descriptionKey: "settings.ruleNoSpaceFullwidthDesc",
    beforeExample: "你好 ，世界 ！",
    afterExample: "你好，世界！",
  },
  {
    key: "no_space_fullwidth_quote",
    ruleName: "no-space-fullwidth-quote",
    labelKey: "settings.ruleNoSpaceFullwidthQuote",
    descriptionKey: "settings.ruleNoSpaceFullwidthQuoteDesc",
    beforeExample: "他说 「 你好 」",
    afterExample: "他说「你好」",
  },
  {
    key: "spellcheck",
    ruleName: "spellcheck",
    labelKey: "settings.ruleSpellcheck",
    descriptionKey: "settings.ruleSpellcheckDesc",
    beforeExample: "我们用Javascript开发",
    afterExample: "我们用 JavaScript 开发",
  },
];

export function SettingsPage({ onBack }: { onBack: () => void }) {
  const { config, isLoading, isSaving, error, load, save, clearError } = useConfigStore();
  const { check: checkEngine } = useEngineStore();
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
      setForm(config);
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
      const error = normalizeCommandError(err);
      setShowSaved(false);
      if (isShortcutRegistrationError(error)) {
        setShortcutError(t("settings.shortcutConflict", { shortcut: formatShortcutForDisplay(form.shortcut) }));
        clearError();
      }
    }
  };

  const updateForm = (partial: Partial<AppConfig>) => {
    setForm((prev) => (prev ? { ...prev, ...partial } : prev));
  };

  const updateFormatterRule = (key: FormatterRuleKey, value: boolean) => {
    setForm((prev) =>
      prev
        ? {
            ...prev,
            formatter: {
              ...prev.formatter,
              rules: {
                ...prev.formatter.rules,
                [key]: value,
              },
            },
          }
        : prev
    );
  };

  const resetFormatterRules = () => {
    setForm((prev) =>
      prev
        ? {
            ...prev,
            formatter: {
              ...prev.formatter,
              rules: DEFAULT_FORMATTER_RULES,
            },
          }
        : prev
    );
  };

  const hasChanges = config && form && JSON.stringify(config) !== JSON.stringify(form);

  return (
    <div className="app-shell page-enter">
      {/* Header */}
      <div className="app-toolbar">
        <div className="toolbar-brand">
          <button
            onClick={onBack}
            className="tool-button"
            aria-label={t("common.back")}
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
          aria-label={isSaving ? t("common.saving") : t("common.save")}
          aria-busy={isSaving}
        >
          {isSaving ? (
            <>
              <span className="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
              {t("common.saving")}
            </>
          ) : t("common.save")}
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
              <p className="shortcut-error">{shortcutError}</p>
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
          <Field label={t("settings.historyLimit")}>
            <input
              type="number"
              min={100}
              max={1000}
              step={50}
              value={form.history_limit}
              onChange={(e) => {
                const v = Math.max(100, Math.min(1000, Number(e.target.value) || 100));
                updateForm({ history_limit: v });
              }}
              disabled={!form.history_enabled}
              className="field-control"
            />
          </Field>
        </Section>

        {/* Formatter */}
        <Section title={t("settings.formatter")}>
          <ToggleField
            label={t("settings.diffHighlight")}
            checked={form.diff_highlight}
            onChange={(v) => updateForm({ diff_highlight: v })}
          />
          <Field label={t("settings.formatRules")}>
            <div className="space-y-1.5">
              {FORMATTER_RULES.map((rule) => (
                <RuleToggle
                  key={rule.key}
                  label={t(rule.labelKey)}
                  description={t(rule.descriptionKey)}
                  ruleName={rule.ruleName}
                  beforeLabel={t("settings.exampleBefore")}
                  afterLabel={t("settings.exampleAfter")}
                  beforeExample={rule.beforeExample}
                  afterExample={rule.afterExample}
                  checked={form.formatter.rules[rule.key]}
                  onChange={(checked) => updateFormatterRule(rule.key, checked)}
                />
              ))}
            </div>
            <div className="mt-2 flex justify-end">
              <button
                onClick={resetFormatterRules}
                className="tool-button"
                type="button"
              >
                {t("settings.resetRules")}
              </button>
            </div>
          </Field>
          <Field label={t("settings.formatEngine")}>
            <div className="engine-badge">
              <span className="status-dot status-dot-success" />
              <span>{t("settings.embeddedEngine")}</span>
            </div>
          </Field>
        </Section>
      </div>

      {/* Error toast */}
      {error && (
        <div className="toast toast-danger" role="alert">
          <span>{error}</span>
          <button onClick={clearError} className="ml-1 text-white/60 hover:text-white transition-colors" aria-label={t("common.close")}>
            &#x2715;
          </button>
        </div>
      )}

      {/* Save success toast */}
      {showSaved && !error && (
        <div className="toast toast-success" role="status">
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

function RuleToggle({
  label,
  description,
  ruleName,
  beforeLabel,
  afterLabel,
  beforeExample,
  afterExample,
  checked,
  onChange,
}: {
  label: string;
  description: string;
  ruleName: string;
  beforeLabel: string;
  afterLabel: string;
  beforeExample: string;
  afterExample: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  const labelId = `rule-toggle-${ruleName}`;

  return (
    <div className="formatter-rule-card">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <span className="block text-[13px] font-medium text-text-primary" id={labelId}>
            {label}
          </span>
          <span className="mt-0.5 block text-[12px] leading-relaxed text-text-secondary">
            {description}
          </span>
        </div>
        <button
          onClick={() => onChange(!checked)}
          role="switch"
          aria-checked={checked}
          aria-labelledby={labelId}
          className={`relative mt-0.5 w-9 h-[20px] rounded-full transition-colors duration-200 shrink-0 ${
            checked ? "bg-accent" : "bg-border"
          }`}
          type="button"
        >
          <span
            className={`absolute top-[2px] left-[2px] w-4 h-4 rounded-full bg-white shadow-sm transition-transform duration-200 ${
              checked ? "translate-x-4" : ""
            }`}
          />
        </button>
      </div>
      <div className="mt-2 space-y-1.5 text-[11px]">
        <span className="formatter-rule-code">{ruleName}</span>
        <div className="formatter-rule-example-row">
          <span className="formatter-rule-pill formatter-rule-pill-before">{beforeLabel}</span>
          <code className="formatter-rule-example">{beforeExample}</code>
        </div>
        <div className="formatter-rule-example-row">
          <span className="formatter-rule-pill formatter-rule-pill-after">{afterLabel}</span>
          <code className="formatter-rule-example">{afterExample}</code>
        </div>
      </div>
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
  // Use refs to hold latest callbacks without re-triggering useEffect.
  // Updated in a passive effect so we don't write during render.
  const onChangeRef = useRef(onChange);
  const onStopRecordRef = useRef(onStopRecord);
  useEffect(() => {
    onChangeRef.current = onChange;
    onStopRecordRef.current = onStopRecord;
  });

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
            className="shortcut-recorder field-control flex-1 min-w-0"
            aria-live="polite"
          >
            <span className="shortcut-recorder-dot" />
            <span className="shortcut-recorder-copy">
              <span className="shortcut-recorder-title">{t("settings.pressShortcut")}</span>
              <span className="shortcut-recorder-hint">{t("settings.escToCancel")}</span>
            </span>
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

function isShortcutRegistrationError(error: { kind: string | null; message: string }): boolean {
  const message = error.message.toLowerCase();
  return (
    error.kind === "ShortcutRegistrationError" ||
    message.includes("shortcut registration") ||
    message.includes("hotkey") ||
    message.includes("already") ||
    message.includes("registered")
  );
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
      <span className="text-[13px] text-text-primary" id={`toggle-label-${label}`}>{label}</span>
      <button
        onClick={() => onChange(!checked)}
        role="switch"
        aria-checked={checked}
        aria-labelledby={`toggle-label-${label}`}
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
