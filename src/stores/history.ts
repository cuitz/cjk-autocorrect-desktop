import { create } from "zustand";
import { getHistory, clearHistory as clearHistoryCmd, type HistoryItem } from "../lib/commands";

interface HistoryState {
  items: HistoryItem[];
  isLoading: boolean;
  error: string | null;

  load: (limit?: number) => Promise<void>;
  clear: () => Promise<void>;
  clearError: () => void;
}

export const useHistoryStore = create<HistoryState>((set) => ({
  items: [],
  isLoading: false,
  error: null,

  load: async (limit?: number) => {
    set({ isLoading: true, error: null });
    try {
      const items = await getHistory(limit);
      set({ items, isLoading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        isLoading: false,
      });
    }
  },

  clear: async () => {
    try {
      await clearHistoryCmd();
      set({ items: [] });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
      });
    }
  },

  clearError: () => set({ error: null }),
}));
