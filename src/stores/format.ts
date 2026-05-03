import { create } from "zustand";
import { formatText, type FormatResult, type HistoryItem } from "../lib/commands";

interface FormatState {
  inputText: string;
  result: FormatResult | null;
  isFormatting: boolean;
  error: string | null;
  mode: string;

  setInputText: (text: string) => void;
  setMode: (mode: string) => void;
  restoreFromHistory: (item: HistoryItem) => void;
  format: () => Promise<void>;
  clear: () => void;
  clearError: () => void;
}

export const useFormatStore = create<FormatState>((set, get) => ({
  inputText: "",
  result: null,
  isFormatting: false,
  error: null,
  mode: "standard",

  setInputText: (text) => set({ inputText: text }),
  setMode: (mode) => set({ mode }),
  restoreFromHistory: (item) =>
    set({
      inputText: item.original_text,
      result: {
        original_text: item.original_text,
        formatted_text: item.formatted_text,
        changed: item.changed,
        diagnostics: [],
        elapsed_ms: 0,
      },
      mode: item.mode === "strict" ? "strict" : "standard",
      isFormatting: false,
      error: null,
    }),

  format: async () => {
    const { inputText, mode } = get();
    if (!inputText.trim()) {
      set({ error: "EMPTY_INPUT" });
      return;
    }

    set({ isFormatting: true, error: null });
    try {
      const result = await formatText(inputText, mode);
      set({ result, isFormatting: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        isFormatting: false,
      });
    }
  },

  clear: () => set({ inputText: "", result: null, error: null }),

  clearError: () => set({ error: null }),
}));
