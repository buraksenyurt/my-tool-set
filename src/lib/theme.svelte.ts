// Light/dark preference. Remembered per machine with localStorage: losing it is
// harmless, so every access is wrapped and falls back to "system".

import { getCurrentWindow } from "@tauri-apps/api/window";
import { inTauri } from "./api";

/** "system" follows the OS setting; the others override it. */
export type Theme = "system" | "light" | "dark";

const KEY = "my-tool-set.theme";
const ORDER: Theme[] = ["system", "light", "dark"];

function load(): Theme {
  try {
    const raw = localStorage.getItem(KEY);
    if (raw === "light" || raw === "dark" || raw === "system") return raw;
  } catch {
    /* private mode or blocked storage */
  }
  return "system";
}

class ThemeState {
  current = $state<Theme>(load());

  /** Cycles system → light → dark → system. */
  next() {
    this.current = ORDER[(ORDER.indexOf(this.current) + 1) % ORDER.length];
    try {
      localStorage.setItem(KEY, this.current);
    } catch {
      /* ignore */
    }
    this.apply();
  }

  apply() {
    const root = document.documentElement;
    if (this.current === "system") delete root.dataset.theme;
    else root.dataset.theme = this.current;

    // Keep the native title bar in step with the page.
    if (inTauri()) {
      getCurrentWindow()
        .setTheme(this.current === "system" ? null : this.current)
        .catch(() => {});
    }
  }
}

export const theme = new ThemeState();
