import type { HostMode } from "./bindings/HostMode";
import type { Platform } from "./bindings/Platform";
import type { Support } from "./bindings/Support";

const OS_NAMES = { windows: "Windows", linux: "Linux", macos: "macOS" } as const;

/** "Windows x64", or a hint when the platform is unknown. */
export function platformLabel(platform: Platform | null): string {
  if (!platform) return "Platform bilinmiyor";
  return `${OS_NAMES[platform.os]} ${platform.arch}`;
}

/** The key the tools' release.json files use, e.g. "windows-x64". */
export function platformKey(platform: Platform): string {
  return `${platform.os}-${platform.arch}`;
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1).replace(".", ",")} MB`;
}

/** "az önce", "12 dk önce", "3 sa önce", or a date. Input: seconds since epoch. */
export function formatAgo(seconds: number, now = Date.now() / 1000): string {
  const diff = Math.max(0, now - seconds);
  if (diff < 60) return "az önce";
  if (diff < 3600) return `${Math.floor(diff / 60)} dk önce`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} sa önce`;
  return new Date(seconds * 1000).toLocaleDateString("tr-TR");
}

export const HOST_LABELS: Record<HostMode, string> = {
  external: "Ayrı pencerede açılır",
  terminal: "Gömülü terminalde çalışır",
  module: "Kabuk içinde modül",
};

export const SUPPORT_LABELS: Record<Support, string> = {
  yes: "Bu platformda destekleniyor",
  untested: "Bu platformda denenmedi",
  no: "Bu platformda desteklenmiyor",
};
