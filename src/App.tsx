import { useState, useEffect, useRef } from "react";
import { FormatPage } from "./components/FormatPage";
import { SettingsPage } from "./components/SettingsPage";
import { HistoryPage } from "./components/HistoryPage";
import { useConfigStore } from "./stores/config";
import { useFormatStore } from "./stores/format";
import { listen } from "@tauri-apps/api/event";
import { resolveLocale, useI18n } from "./i18n";

type Route = "format" | "settings" | "history";

function App() {
  const [route, setRoute] = useState<Route>("format");
  const { config, load } = useConfigStore();
  const populateFromClipboard = useFormatStore((s) => s.populateFromClipboard);
  const [clipboardToast, setClipboardToast] = useState<string | null>(null);
  const toastTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const { t } = useI18n();

  // Load config on mount to determine theme
  useEffect(() => {
    load();
  }, [load]);

  // Apply theme whenever config changes
  useEffect(() => {
    if (!config) return;

    const root = document.documentElement;
    const theme = config.theme;

    if (theme === "dark") {
      root.classList.add("dark");
    } else if (theme === "light") {
      root.classList.remove("dark");
    } else {
      // System theme
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      if (mq.matches) {
        root.classList.add("dark");
      } else {
        root.classList.remove("dark");
      }

      // Listen for system theme changes
      const handler = (e: MediaQueryListEvent) => {
        if (e.matches) {
          root.classList.add("dark");
        } else {
          root.classList.remove("dark");
        }
      };
      mq.addEventListener("change", handler);
      return () => mq.removeEventListener("change", handler);
    }
  }, [config?.theme]);

  useEffect(() => {
    document.documentElement.lang = resolveLocale(config?.language);
  }, [config?.language]);

  const tCallbackRef = useRef(t);
  const populateFromClipboardRef = useRef(populateFromClipboard);
  useEffect(() => {
    tCallbackRef.current = t;
    populateFromClipboardRef.current = populateFromClipboard;
  });

  useEffect(() => {
    const unlisten = listen<{
      original_text: string;
      formatted_text: string;
      changed: boolean;
    }>("clipboard-formatted", (event) => {
      const { original_text, formatted_text, changed } = event.payload;
      populateFromClipboardRef.current(original_text, formatted_text, changed);
      const msg = changed
        ? tCallbackRef.current("format.clipboardChanged")
        : tCallbackRef.current("format.clipboardNoChange");
      setClipboardToast(msg);
      if (toastTimerRef.current) clearTimeout(toastTimerRef.current);
      toastTimerRef.current = setTimeout(() => setClipboardToast(null), 2000);
    });

    return () => {
      unlisten.then((fn) => fn());
      if (toastTimerRef.current) clearTimeout(toastTimerRef.current);
    };
  }, []);

  useEffect(() => {
    const unlisten = listen<string>("navigate", (event) => {
      if (event.payload === "/settings") {
        setRoute("settings");
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const renderPage = () => {
    switch (route) {
      case "format":
        return <FormatPage onNavigate={setRoute} />;
      case "settings":
        return <SettingsPage onBack={() => setRoute("format")} />;
      case "history":
        return <HistoryPage onBack={() => setRoute("format")} />;
    }
  };

  return (
    <div className="flex flex-col h-screen">
      {renderPage()}
      {clipboardToast && (
        <div className="toast toast-success" role="status">
          {clipboardToast}
        </div>
      )}
    </div>
  );
}

export default App;
