import { create } from "zustand";
import { loadConfig, saveConfig, type AppConfig } from "../lib/commands";

interface ConfigState {
  config: AppConfig | null;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;

  load: () => Promise<void>;
  save: (config: AppConfig) => Promise<void>;
  clearError: () => void;
}

export const useConfigStore = create<ConfigState>((set) => ({
  config: null,
  isLoading: false,
  isSaving: false,
  error: null,

  load: async () => {
    set({ isLoading: true, error: null });
    try {
      const config = await loadConfig();
      set({ config, isLoading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        isLoading: false,
      });
    }
  },

  save: async (config: AppConfig) => {
    set({ isSaving: true, error: null });
    try {
      await saveConfig(config);
      set({ config, isSaving: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        isSaving: false,
      });
    }
  },

  clearError: () => set({ error: null }),
}));
