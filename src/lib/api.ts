// The only place that talks to Rust. Components call these functions and never
// `invoke` directly, so the IPC surface stays visible in one file.
//
// Outside Tauri (plain `npm run dev` in a browser) the functions return demo
// data, so UI work is possible without starting the desktop shell.

import { invoke } from "@tauri-apps/api/core";
import type { AppError } from "./bindings/AppError";
import type { AppInfoDto } from "./bindings/AppInfoDto";
import type { ErrorKind } from "./bindings/ErrorKind";

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
};

export async function appInfo(): Promise<AppInfoDto> {
  if (!inTauri()) return DEMO_INFO;
  return call<AppInfoDto>("app_info");
}
