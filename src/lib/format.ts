import type { Platform } from "./bindings/Platform";

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
