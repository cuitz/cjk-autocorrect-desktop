import { useState, useEffect } from "react";
import { FormatPage } from "./components/FormatPage";
import { SettingsPage } from "./components/SettingsPage";
import { HistoryPage } from "./components/HistoryPage";
import { useConfigStore } from "./stores/config";
import { listen } from "@tauri-apps/api/event";
import { resolveLocale } from "./i18n";

type Route = "format" | "settings" | "history";

function App() {
  const [route, setRoute] = useState<Route>("format");
  const { config, load } = useConfigStore();

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
    </div>
  );
}

export default App;
