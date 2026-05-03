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
  diff_highlight: boolean;
  formatter: {
    mode: string;
    autocorrect_path?: string | null;
  };
  version: number;
}

export interface HistoryItem {
  id: string;
  original_text: string;
  formatted_text: string;
  mode: string;
  changed: boolean;
  created_at: string;
}

export async function formatText(
  text: string,
  mode?: string
): Promise<FormatResult> {
  return invoke("format_text", { request: { text, mode: mode || null } });
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
