import { invoke } from "@tauri-apps/api/core";

export interface FormatResult {
  original_text: string;
  formatted_text: string;
  changed: boolean;
  diagnostics: Array<{
    line: number;
    col: number;
    message: string;
    severity: string;
  }>;
  elapsed_ms: number;
}

export interface AppConfig {
  shortcut: string;
  auto_start: boolean;
  close_to_tray: boolean;
  theme: string;
  language: string;
  history_enabled: boolean;
  history_limit: number;
  diff_highlight: boolean;
  formatter: FormatterConfig;
  version: number;
}

export interface FormatterConfig {
  rules: FormatterRules;
}

export interface FormatterRules {
  space_word: boolean;
  space_punctuation: boolean;
  space_bracket: boolean;
  space_dash: boolean;
  space_backticks: boolean;
  space_dollar: boolean;
  fullwidth: boolean;
  halfwidth_word: boolean;
  halfwidth_punctuation: boolean;
  no_space_fullwidth: boolean;
  no_space_fullwidth_quote: boolean;
  spellcheck: boolean;
}

export interface HistoryItem {
  id: string;
  original_text: string;
  formatted_text: string;
  changed: boolean;
  created_at: string;
}

export async function formatText(text: string): Promise<FormatResult> {
  return invoke("format_text", { request: { text } });
}

export async function loadConfig(): Promise<AppConfig> {
  return invoke("load_config");
}

export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke("save_config", { config });
}

export async function readClipboard(): Promise<string> {
  return invoke("read_clipboard");
}

export async function writeClipboard(text: string): Promise<void> {
  return invoke("write_clipboard", { text });
}

export async function getHistory(limit?: number): Promise<HistoryItem[]> {
  return invoke("get_history", { request: limit ? { limit } : null });
}

export async function clearHistory(): Promise<void> {
  return invoke("clear_history");
}

export interface EngineStatus {
  autocorrect_installed: boolean;
  autocorrect_path: string | null;
  install_hint: string | null;
}

export async function checkEngine(): Promise<EngineStatus> {
  return invoke("check_engine");
}

export async function formatClipboard(): Promise<void> {
  return invoke("format_clipboard");
}
