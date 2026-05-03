import { create } from "zustand";
import { checkEngine, type EngineStatus } from "../lib/commands";

interface EngineState {
  status: EngineStatus | null;
  loading: boolean;

  check: () => Promise<void>;
}

export const useEngineStore = create<EngineState>((set) => ({
  status: null,
  loading: false,

  check: async () => {
    set({ loading: true });
    try {
      const status = await checkEngine();
      set({ status, loading: false });
    } catch {
      set({ loading: false });
    }
  },
}));
