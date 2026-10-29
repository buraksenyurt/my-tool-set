// The catalog as the UI sees it: tools, their install state, and the
// operations in flight. Every change goes through Rust and the list is then
// reloaded, so Rust stays the single source of truth.

import { checkUpdates, installTool, listTools, reloadCatalog, uninstallTool } from "./api";
import type { CatalogDto } from "./bindings/CatalogDto";
import type { InstallProgressDto } from "./bindings/InstallProgressDto";
import type { ToolDto } from "./bindings/ToolDto";

export type ToolState =
  | "installed"
  | "update"
  | "notInstalled"
  | "noRelease"
  | "noBuild"
  | "localReady"
  | "localNotBuilt"
  | "localSetup";

/** One word for the sidebar and the tool header. */
export function toolState(tool: ToolDto): ToolState {
  const d = tool.delivery;
  if (d.type === "release") {
    if (d.updateAvailable) return "update";
    if (d.installed.length > 0) return "installed";
    if (!d.latest) return "noRelease";
    if (!d.latest.hasBuild) return "noBuild";
    return "notInstalled";
  }
  switch (d.status.kind) {
    case "ready":
    case "fromSource":
      return "localReady";
    case "notBuilt":
      return "localNotBuilt";
    default:
      return "localSetup";
  }
}

export const STATE_LABELS: Record<ToolState, string> = {
  installed: "Kurulu",
  update: "Güncelleme var",
  notInstalled: "Kurulu değil",
  noRelease: "Sürüm yok",
  noBuild: "Bu platformda yok",
  localReady: "Yerel",
  localNotBuilt: "Derlenmemiş",
  localSetup: "Ayar gerekli",
};

class CatalogState {
  data = $state<CatalogDto | null>(null);
  loadError = $state<string | null>(null);

  checking = $state(false);
  /** When the last update check finished (ms since epoch). */
  checkedAt = $state<number | null>(null);
  /** Per-tool error from the last update check. */
  checkErrors = $state<Record<string, string>>({});

  /** Install progress per tool id; present while an install runs. */
  progress = $state<Record<string, InstallProgressDto | "starting">>({});
  /** Last install/uninstall error per tool id. */
  actionErrors = $state<Record<string, string>>({});

  tools = $derived(this.data?.tools ?? []);
  issues = $derived(this.data?.issues ?? []);

  find(id: string): ToolDto | undefined {
    return this.tools.find((t) => t.id === id);
  }

  async load() {
    try {
      this.data = await listTools();
      this.loadError = null;
    } catch (e) {
      this.loadError = (e as Error).message;
    }
  }

  async reload() {
    try {
      this.data = await reloadCatalog();
      this.loadError = null;
    } catch (e) {
      this.loadError = (e as Error).message;
    }
  }

  async checkUpdates() {
    if (this.checking) return;
    this.checking = true;
    try {
      const results = await checkUpdates();
      const errors: Record<string, string> = {};
      for (const r of results) if (r.error) errors[r.id] = r.error;
      this.checkErrors = errors;
      this.checkedAt = Date.now();
      await this.load();
    } catch (e) {
      this.loadError = (e as Error).message;
    } finally {
      this.checking = false;
    }
  }

  async install(id: string) {
    if (this.progress[id]) return;
    this.progress[id] = "starting";
    delete this.actionErrors[id];
    try {
      await installTool(id, (p) => (this.progress[id] = p));
    } catch (e) {
      this.actionErrors[id] = (e as Error).message;
    } finally {
      delete this.progress[id];
      await this.load();
    }
  }

  async uninstall(id: string, version: string) {
    delete this.actionErrors[id];
    try {
      await uninstallTool(id, version);
    } catch (e) {
      this.actionErrors[id] = (e as Error).message;
    } finally {
      await this.load();
    }
  }
}

export const catalog = new CatalogState();
