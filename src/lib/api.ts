// The only place that talks to Rust. Components call these functions and never
// `invoke` directly, so the IPC surface stays visible in one file.
//
// Outside Tauri (plain `npm run dev` in a browser) the functions return demo
// data, so UI work is possible without starting the desktop shell.

import { Channel, invoke } from "@tauri-apps/api/core";
import type { AppError } from "./bindings/AppError";
import type { AppInfoDto } from "./bindings/AppInfoDto";
import type { CatalogDto } from "./bindings/CatalogDto";
import type { ErrorKind } from "./bindings/ErrorKind";
import type { InstallProgressDto } from "./bindings/InstallProgressDto";
import type { InstalledDto } from "./bindings/InstalledDto";
import type { UpdateCheckDto } from "./bindings/UpdateCheckDto";
import { demoCatalog, demoCheckUpdates, demoInstall, demoUninstall } from "./demo";

export const inTauri = (): boolean => "__TAURI_INTERNALS__" in window;

/** An `Error` that remembers which `ErrorKind` Rust reported. */
export class ApiError extends Error {
  constructor(
    public readonly kind: ErrorKind | "unknown",
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

function isAppError(value: unknown): value is AppError {
  return typeof value === "object" && value !== null && "kind" in value && "message" in value;
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (err) {
    if (isAppError(err)) throw new ApiError(err.kind, err.message);
    throw new ApiError("unknown", String(err));
  }
}

const DEMO_INFO: AppInfoDto = {
  name: "my-tool-set",
  version: "0.1.0",
  platform: null,
  dataDir: "(tarayıcı önizlemesi)",
  configDir: "(tarayıcı önizlemesi)",
};

export async function appInfo(): Promise<AppInfoDto> {
  if (!inTauri()) return DEMO_INFO;
  return call<AppInfoDto>("app_info");
}

export async function listTools(): Promise<CatalogDto> {
  if (!inTauri()) return demoCatalog();
  return call<CatalogDto>("list_tools");
}

/** Re-reads settings.toml and the catalog folders. */
export async function reloadCatalog(): Promise<CatalogDto> {
  if (!inTauri()) return demoCatalog();
  return call<CatalogDto>("reload_catalog");
}

/** Fetches the latest release.json of every release tool. */
export async function checkUpdates(): Promise<UpdateCheckDto[]> {
  if (!inTauri()) return demoCheckUpdates();
  return call<UpdateCheckDto[]>("check_updates");
}

export async function installTool(
  id: string,
  onProgress: (p: InstallProgressDto) => void,
): Promise<InstalledDto> {
  if (!inTauri()) return demoInstall(id, onProgress);
  const channel = new Channel<InstallProgressDto>();
  channel.onmessage = onProgress;
  return call<InstalledDto>("install_tool", { id, onProgress: channel });
}

export async function uninstallTool(id: string, version: string): Promise<void> {
  if (!inTauri()) return demoUninstall(id, version);
  return call<void>("uninstall_tool", { id, version });
}
