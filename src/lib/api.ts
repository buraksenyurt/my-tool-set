// The only place that talks to Rust. Components call these functions and never
// `invoke` directly, so the IPC surface stays visible in one file.
//
// Outside Tauri (plain `npm run dev` in a browser) the functions return demo
// data, so UI work is possible without starting the desktop shell.

import { Channel, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { AppError } from "./bindings/AppError";
import type { AppInfoDto } from "./bindings/AppInfoDto";
import type { CatalogDto } from "./bindings/CatalogDto";
import type { ErrorKind } from "./bindings/ErrorKind";
import type { InstallProgressDto } from "./bindings/InstallProgressDto";
import type { InstalledDto } from "./bindings/InstalledDto";
import type { LogLineDto } from "./bindings/LogLineDto";
import type { ReadmeDto } from "./bindings/ReadmeDto";
import type { RunDto } from "./bindings/RunDto";
import type { RunLogEvent } from "./bindings/RunLogEvent";
import type { TerminalDto } from "./bindings/TerminalDto";
import type { TerminalExitEvent } from "./bindings/TerminalExitEvent";
import type { UpdateCheckDto } from "./bindings/UpdateCheckDto";
import {
  demoCatalog,
  demoCheckUpdates,
  demoInstall,
  demoReadme,
  demoRuns,
  demoTerminals,
  demoUninstall,
} from "./demo";

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

/** The tool's README as sanitized HTML (pictures served by the shell). */
export async function getReadme(id: string): Promise<ReadmeDto> {
  if (!inTauri()) return demoReadme(id);
  return call<ReadmeDto>("get_readme", { id });
}

/** Opens an http(s) or mailto link in the system browser. */
export async function openExternal(url: string): Promise<void> {
  if (!/^(https?:|mailto:)/i.test(url)) return;
  if (!inTauri()) {
    window.open(url, "_blank", "noopener");
    return;
  }
  await openUrl(url);
}

// --- Runs (tools and builds) ---------------------------------------------

/** Opens a tool in its own window. */
export async function startTool(id: string): Promise<RunDto> {
  if (!inTauri()) return demoRuns.start(id, "tool");
  return call<RunDto>("start_tool", { id });
}

/** Runs a local tool's build command. */
export async function buildTool(id: string): Promise<RunDto> {
  if (!inTauri()) return demoRuns.start(id, "build");
  return call<RunDto>("build_tool", { id });
}

export async function stopRun(runId: number): Promise<void> {
  if (!inTauri()) return demoRuns.stop(runId);
  return call<void>("stop_run", { runId });
}

export async function listRuns(): Promise<RunDto[]> {
  if (!inTauri()) return demoRuns.list();
  return call<RunDto[]>("list_runs");
}

export async function runLog(runId: number): Promise<LogLineDto[]> {
  if (!inTauri()) return demoRuns.log(runId);
  return call<LogLineDto[]>("run_log", { runId });
}

/** Subscribes to run events; returns the unsubscribe function. */
export async function onRunEvents(handlers: {
  changed: (run: RunDto) => void;
  log: (event: RunLogEvent) => void;
}): Promise<() => void> {
  if (!inTauri()) return demoRuns.subscribe(handlers);
  const offChanged = await listen<RunDto>("run-changed", (e) => handlers.changed(e.payload));
  const offLog = await listen<RunLogEvent>("run-log", (e) => handlers.log(e.payload));
  return () => {
    offChanged();
    offLog();
  };
}

// --- Embedded terminal ----------------------------------------------------

/** Starts a terminal tool; output bytes arrive on `onData`. */
export async function openTerminal(
  id: string,
  values: Record<string, string>,
  extra: string,
  cols: number,
  rows: number,
  onData: (bytes: Uint8Array) => void,
): Promise<TerminalDto> {
  if (!inTauri()) return demoTerminals.open(id, cols, rows, onData);
  const channel = new Channel<ArrayBuffer>();
  channel.onmessage = (buffer) => onData(new Uint8Array(buffer));
  return call<TerminalDto>("open_terminal", { id, values, extra, cols, rows, onData: channel });
}

export async function writeTerminal(sessionId: number, data: string): Promise<void> {
  if (!inTauri()) return demoTerminals.write(sessionId, data);
  return call<void>("write_terminal", { sessionId, data });
}

export async function resizeTerminal(sessionId: number, cols: number, rows: number): Promise<void> {
  if (!inTauri()) return;
  return call<void>("resize_terminal", { sessionId, cols, rows });
}

export async function closeTerminal(sessionId: number): Promise<void> {
  if (!inTauri()) return demoTerminals.close(sessionId);
  return call<void>("close_terminal", { sessionId });
}

export async function onTerminalExit(handler: (e: TerminalExitEvent) => void): Promise<() => void> {
  if (!inTauri()) return demoTerminals.onExit(handler);
  return listen<TerminalExitEvent>("terminal-exit", (e) => handler(e.payload));
}

/** Native folder (or file) picker; `null` when cancelled. */
export async function pickPath(kind: "folder" | "file", current?: string): Promise<string | null> {
  if (!inTauri()) return kind === "folder" ? "/home/burak/projeler/easy-git" : "/home/burak/dosya.txt";
  const picked = await openDialog({ directory: kind === "folder", multiple: false, defaultPath: current || undefined });
  return typeof picked === "string" ? picked : null;
}
