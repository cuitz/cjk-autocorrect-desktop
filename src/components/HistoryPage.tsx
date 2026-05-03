import { useEffect, useMemo, useRef, useState } from "react";
import { useHistoryStore } from "../stores/history";
import { useFormatStore } from "../stores/format";
import { writeClipboard, type HistoryItem } from "../lib/commands";
import { type Locale, type TranslationKey, useI18n } from "../i18n";

type ModeFilter = "all" | "standard" | "strict";

export function HistoryPage({ onBack }: { onBack: () => void }) {
  const { items, isLoading, error, load, clear, clearError } = useHistoryStore();
  const { restoreFromHistory } = useFormatStore();
  const [showConfirm, setShowConfirm] = useState(false);
  const [query, setQuery] = useState("");
  const [modeFilter, setModeFilter] = useState<ModeFilter>("all");
  const [selectedItem, setSelectedItem] = useState<HistoryItem | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const toastTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const { locale, t } = useI18n();

  useEffect(() => {
    load(100);
  }, [load]);

  useEffect(() => {
    return () => {
      if (toastTimerRef.current) clearTimeout(toastTimerRef.current);
    };
  }, []);

  const filteredItems = useMemo(() => {
    const needle = query.trim().toLowerCase();

    return items.filter((item) => {
      const matchesQuery =
        !needle ||
        item.original_text.toLowerCase().includes(needle) ||
        item.formatted_text.toLowerCase().includes(needle);
      const matchesMode = modeFilter === "all" || item.mode === modeFilter;

      return matchesQuery && matchesMode;
    });
  }, [items, query, modeFilter]);

  const showToast = (message: string) => {
    setToast(message);
    if (toastTimerRef.current) clearTimeout(toastTimerRef.current);
    toastTimerRef.current = setTimeout(() => setToast(null), 1600);
  };

  const handleClear = () => {
    setShowConfirm(true);
  };

  const confirmClear = async () => {
    setShowConfirm(false);
    await clear();
    setSelectedItem(null);
  };

  const handleCopy = async (text: string, message = t("history.copied")) => {
    try {
      await writeClipboard(text);
      showToast(message);
    } catch {
      showToast(t("history.copyFailed"));
    }
  };

  const handleRestore = (item: HistoryItem) => {
    restoreFromHistory(item);
    onBack();
  };

  const hasFilter = query.trim().length > 0 || modeFilter !== "all";

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
          <h1 className="toolbar-title">{t("history.historyTitle")}</h1>
          <span className="toolbar-subtitle">
            {filteredItems.length}/{items.length} {t("history.entries")}
          </span>
        </div>
        <div className="toolbar-actions">
          <button
            onClick={() => load(100)}
            className="tool-button"
            aria-label={t("common.refresh")}
          >
            {t("common.refresh")}
          </button>
          <button
            onClick={handleClear}
            disabled={items.length === 0}
            className="tool-button tool-button-danger"
            aria-label={t("common.clear")}
          >
            {t("common.clear")}
          </button>
        </div>
      </div>

      {/* Search and filters */}
      <div className="px-3 py-2 bg-surface border-b border-border-subtle shrink-0">
        <div className="flex items-center gap-2">
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t("history.searchPlaceholder")}
            className="field-control flex-1 min-w-0"
          />
          <SegmentedControl
            options={[
              { value: "all", label: t("history.all") },
              { value: "standard", label: t("common.standard") },
              { value: "strict", label: t("common.strict") },
            ]}
            value={modeFilter}
            onChange={(value) => setModeFilter(value as ModeFilter)}
          />
          {hasFilter && (
            <button
              onClick={() => {
                setQuery("");
                setModeFilter("all");
              }}
              className="tool-button"
            >
              {t("common.reset")}
            </button>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="content-area content-area-scroll">
        {isLoading ? (
          <div className="space-y-2 py-1">
            {[...Array(4)].map((_, i) => (
              <div key={i} className="skeleton h-20 w-full" />
            ))}
          </div>
        ) : items.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-32 gap-2">
            <span className="text-[13px] text-text-tertiary">{t("history.empty")}</span>
            <span className="text-[11px] text-text-tertiary opacity-60">
              {t("history.emptyHint")}
            </span>
          </div>
        ) : filteredItems.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-32 gap-2">
            <span className="text-[13px] text-text-tertiary">{t("history.noMatches")}</span>
            <button
              onClick={() => {
                setQuery("");
                setModeFilter("all");
              }}
              className="tool-button"
            >
              {t("history.showAll")}
            </button>
          </div>
        ) : (
          <div className="space-y-1.5">
            {filteredItems.map((item) => (
              <HistoryRow
                key={item.id}
                item={item}
                locale={locale}
                t={t}
                onCopy={() => handleCopy(item.formatted_text, t("history.copiedResult"))}
                onRestore={() => handleRestore(item)}
                onOpen={() => setSelectedItem(item)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Confirm dialog */}
      {showConfirm && (
        <div
          className="modal-backdrop"
          onClick={() => setShowConfirm(false)}
        >
          <div
            className="modal-sheet"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-body">
              <h3 className="text-[14px] font-semibold text-text-primary mb-2">
                {t("history.clearConfirmTitle")}
              </h3>
              <p className="text-[13px] text-text-secondary mb-4">
                {t("history.clearConfirmBody")}
              </p>
              <div className="flex justify-end gap-2">
                <button
                  onClick={() => setShowConfirm(false)}
                  className="tool-button"
                >
                  {t("common.cancel")}
                </button>
                <button
                  onClick={confirmClear}
                  className="tool-button tool-button-danger-solid"
                >
                  {t("history.clearConfirmAction")}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {selectedItem && (
        <HistoryDetailDialog
          item={selectedItem}
          locale={locale}
          t={t}
          onClose={() => setSelectedItem(null)}
          onCopyOriginal={() => handleCopy(selectedItem.original_text, t("history.copiedOriginal"))}
          onCopyFormatted={() => handleCopy(selectedItem.formatted_text, t("history.copiedResult"))}
          onRestore={() => handleRestore(selectedItem)}
        />
      )}

      {/* Toasts */}
      {error && (
        <Toast tone="danger" message={error} onClose={clearError} />
      )}
      {toast && !error && (
        <Toast tone="success" message={toast} onClose={() => setToast(null)} />
      )}
    </div>
  );
}

function HistoryRow({
  item,
  locale,
  t,
  onCopy,
  onRestore,
  onOpen,
}: {
  item: HistoryItem;
  locale: Locale;
  t: (key: TranslationKey, vars?: Record<string, string | number>) => string;
  onCopy: () => void;
  onRestore: () => void;
  onOpen: () => void;
}) {
  return (
    <div className="list-card group p-3">
      <button onClick={onOpen} className="w-full text-left block">
        <div className="flex items-center justify-between mb-1.5 gap-2">
          <div className="flex items-center gap-1.5 min-w-0">
            <span className="text-[11px] text-text-tertiary shrink-0">
              {item.mode === "strict" ? t("common.strict") : t("common.standard")}
            </span>
          </div>
          <span className="text-[11px] text-text-tertiary shrink-0">
            {formatTime(item.created_at, locale, t)}
          </span>
        </div>
        <div className="text-[13px] text-text-primary line-clamp-2 leading-relaxed selectable">
          {item.original_text}
        </div>
        {item.changed && (
          <div className="mt-1 text-[13px] text-accent line-clamp-2 leading-relaxed selectable">
            → {item.formatted_text}
          </div>
        )}
      </button>
      <div className="mt-2 pt-2 border-t border-border-subtle flex items-center justify-between gap-2">
        <span className="meta-text">
          {item.formatted_text.length} {t("format.chars")}
        </span>
        <div className="flex items-center gap-1">
          <button
            onClick={onOpen}
            className="tool-button"
          >
            {t("common.detail")}
          </button>
          <button
            onClick={onCopy}
            className="tool-button"
          >
            {t("common.copy")}
          </button>
          <button
            onClick={onRestore}
            className="tool-button"
          >
            {t("common.restore")}
          </button>
        </div>
      </div>
    </div>
  );
}

function HistoryDetailDialog({
  item,
  locale,
  t,
  onClose,
  onCopyOriginal,
  onCopyFormatted,
  onRestore,
}: {
  item: HistoryItem;
  locale: Locale;
  t: (key: TranslationKey, vars?: Record<string, string | number>) => string;
  onClose: () => void;
  onCopyOriginal: () => void;
  onCopyFormatted: () => void;
  onRestore: () => void;
}) {
  return (
    <div
      className="modal-backdrop"
      onClick={onClose}
    >
      <div
        className="modal-sheet modal-sheet-wide"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="app-toolbar h-11">
          <div className="flex items-center gap-2 min-w-0">
            <h3 className="text-[14px] font-semibold text-text-primary shrink-0">
              {t("history.title")}
            </h3>
            <span className="text-[11px] text-text-tertiary">
              {item.mode === "strict" ? t("common.strictMode") : t("common.standardMode")}
            </span>
          </div>
          <button
            onClick={onClose}
            className="tool-button"
          >
            ✕
          </button>
        </div>

        <div className="flex-1 overflow-auto min-h-0 p-4 space-y-3">
          <DetailBlock
            title={t("history.original")}
            text={item.original_text}
            actionLabel={t("history.copyOriginal")}
            onAction={onCopyOriginal}
          />
          <DetailBlock
            title={t("history.result")}
            text={item.formatted_text}
            actionLabel={t("history.copyResult")}
            onAction={onCopyFormatted}
          />
        </div>

        <div className="px-4 py-3 border-t border-border flex items-center justify-between gap-2 shrink-0">
          <span className="text-[11px] text-text-tertiary">
            {formatAbsoluteTime(item.created_at, locale)}
          </span>
          <div className="flex items-center gap-2">
            <button
              onClick={onClose}
              className="tool-button"
            >
              {t("common.close")}
            </button>
            <button
              onClick={onRestore}
              className="tool-button tool-button-primary"
            >
              {t("history.restoreToMain")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function DetailBlock({
  title,
  text,
  actionLabel,
  onAction,
}: {
  title: string;
  text: string;
  actionLabel: string;
  onAction: () => void;
}) {
  return (
    <section>
      <div className="flex items-center justify-between mb-1.5">
        <h4 className="text-[11px] font-semibold text-text-tertiary uppercase tracking-[0.1em]">
          {title}
        </h4>
        <button
          onClick={onAction}
          className="tool-button"
        >
          {actionLabel}
        </button>
      </div>
      <pre className="selectable whitespace-pre-wrap break-words m-0 p-3 rounded-lg border border-border bg-surface-inset text-[13px] leading-relaxed text-text-primary font-sans max-h-48 overflow-auto">
        {text}
      </pre>
    </section>
  );
}

function SegmentedControl({
  options,
  value,
  onChange,
}: {
  options: { value: string; label: string }[];
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <div className="segmented-control">
      {options.map((option) => (
        <button
          key={option.value}
          onClick={() => onChange(option.value)}
          className={`segmented-option ${value === option.value ? "segmented-option-active" : ""}`}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

function Toast({
  tone,
  message,
  onClose,
}: {
  tone: "success" | "danger";
  message: string;
  onClose: () => void;
}) {
  return (
    <div className={`toast ${tone === "danger" ? "toast-danger" : "toast-success"}`} role="alert">
      <span>{message}</span>
      <button
        onClick={onClose}
        className="ml-1 text-white/60 hover:text-white transition-colors"
        aria-label="Close"
      >
        &#x2715;
      </button>
    </div>
  );
}

function formatTime(
  iso: string,
  locale: Locale,
  t: (key: TranslationKey, vars?: Record<string, string | number>) => string
): string {
  try {
    const date = new Date(iso);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return t("history.justNow");
    if (diff < 3600000) return t("history.minutesAgo", { count: Math.floor(diff / 60000) });
    if (diff < 86400000) return t("history.hoursAgo", { count: Math.floor(diff / 3600000) });

    return date.toLocaleDateString(locale, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return iso;
  }
}

function formatAbsoluteTime(iso: string, locale: Locale): string {
  try {
    return new Date(iso).toLocaleString(locale, {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  } catch {
    return iso;
  }
}
