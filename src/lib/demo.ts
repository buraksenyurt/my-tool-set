// Demo data for `npm run dev` in a plain browser. Mirrors the real catalog/
// so the UI can be designed without the desktop shell. Never used in Tauri.

import type { CatalogDto } from "./bindings/CatalogDto";
import type { InstallProgressDto } from "./bindings/InstallProgressDto";
import type { InstalledDto } from "./bindings/InstalledDto";
import type { LogLineDto } from "./bindings/LogLineDto";
import type { RunDto } from "./bindings/RunDto";
import type { RunLogEvent } from "./bindings/RunLogEvent";
import type { TerminalDto } from "./bindings/TerminalDto";
import type { TerminalExitEvent } from "./bindings/TerminalExitEvent";
import type { ReadmeDto } from "./bindings/ReadmeDto";
import type { ToolDto } from "./bindings/ToolDto";
import type { UpdateCheckDto } from "./bindings/UpdateCheckDto";

const now = () => Math.floor(Date.now() / 1000);

const tools: BaseTool[] = [
  {
    id: "docker-city",
    name: "DockerCity",
    summary: "docker-compose servislerini oyunlaştırılmış bir şehir haritası olarak gösterir.",
    category: "docker",
    kind: "gui",
    stack: ["C#", ".NET 10", "WinUI 3", "YamlDotNet", "EF Core"],
    tags: ["docker", "compose", "görselleştirme"],
    host: "external",
    origin: "embedded",
    support: "no",
    params: [],
    delivery: {
      type: "release",
      repo: "buraksenyurt/docker-city",
      installed: [],
      latest: { version: "0.10.0", tag: "v0.10.0", fetchedAt: now(), hasBuild: false, assetSize: null },
      updateAvailable: false,
    },
  },
  {
    id: "docky",
    name: "Docky",
    summary: "Hazır geliştirme modellerinden docker-compose dosyası üreten .NET tool.",
    category: "docker",
    kind: "cli",
    stack: ["C#", ".NET 9", "Spectre.Console"],
    tags: ["docker", "compose", "üretici"],
    host: "terminal",
    origin: "embedded",
    support: "yes",
    params: [],
    delivery: {
      type: "local",
      status: { kind: "unknownRoot", root: "coding-vibes" },
      buildCommand: "dotnet build src/DockyApp.csproj",
      devCommand: "dotnet run --project src/DockyApp.csproj --",
    },
  },
  {
    id: "easy-git",
    name: "easy-git",
    summary: "Git dallarını metro haritası gibi gösteren salt okunur dashboard.",
    category: "git",
    kind: "gui",
    stack: ["Rust", "Tauri 2", "Svelte 5", "gitoxide"],
    tags: ["git", "görselleştirme", "eğitim"],
    host: "external",
    origin: "embedded",
    support: "yes",
    params: [],
    delivery: {
      type: "release",
      repo: "buraksenyurt/easy-git",
      installed: [
        {
          version: "0.1.0",
          folder: "C:\\Users\\burak\\AppData\\Local\\dev.buraksenyurt.mytoolset\\tools\\easy-git\\0.1.0",
          entry: "C:\\Users\\burak\\AppData\\Local\\dev.buraksenyurt.mytoolset\\tools\\easy-git\\0.1.0\\easy-git.exe",
          installedAt: now() - 86400,
        },
      ],
      latest: { version: "0.2.0", tag: "v0.2.0", fetchedAt: now(), hasBuild: true, assetSize: 6_291_456 },
      updateAvailable: true,
    },
  },
  {
    id: "git-dash",
    name: "GitDash",
    summary: "Yerel bir git reposunun commit aktivitesini terminalde takvim ısı haritası olarak gösterir.",
    category: "git",
    kind: "cli",
    stack: ["Zig 0.17"],
    tags: ["git", "terminal"],
    host: "terminal",
    origin: "embedded",
    support: "yes",
    params: [
      { name: "repo", kind: "folder", label: "Repo klasörü" },
      { name: "days", kind: "number", label: "Kaç gün geriye", default: 180 },
    ],
    delivery: {
      type: "release",
      repo: "buraksenyurt/git-dash",
      installed: [
        {
          version: "0.1.0",
          folder: "C:\\Users\\burak\\AppData\\Local\\dev.buraksenyurt.mytoolset\\tools\\git-dash\\0.1.0",
          entry: "C:\\Users\\burak\\AppData\\Local\\dev.buraksenyurt.mytoolset\\tools\\git-dash\\0.1.0\\git-dash-0.1.0-windows-x64.exe",
          installedAt: now() - 3600,
        },
      ],
      latest: { version: "0.1.0", tag: "v0.1.0", fetchedAt: now(), hasBuild: true, assetSize: 927_744 },
      updateAvailable: false,
    },
  },
  {
    id: "java-atlas",
    name: "java-atlas",
    summary: "Java modül, paket ve kütüphane bağımlılıklarını ülke haritası gibi çizer.",
    category: "java",
    kind: "gui",
    stack: ["Rust", "Tauri 2", "Svelte 5", "tree-sitter"],
    tags: ["java", "bağımlılık", "görselleştirme", "eğitim"],
    host: "external",
    origin: "embedded",
    support: "yes",
    params: [],
    delivery: {
      type: "release",
      repo: "buraksenyurt/java-atlas",
      installed: [],
      latest: { version: "0.1.0", tag: "v0.1.0", fetchedAt: now(), hasBuild: true, assetSize: 7_340_032 },
      updateAvailable: false,
    },
  },
];

// Placeholder picture for the gallery (an SVG data URL; the CSP allows data:).
const SHOT =
  "data:image/svg+xml," +
  encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 400"><rect width="640" height="400" fill="#dfe8f4"/><g fill="#2a78d6"><rect x="40" y="60" width="560" height="14" rx="7"/><rect x="40" y="110" width="380" height="14" rx="7" opacity=".6"/><circle cx="120" cy="250" r="40"/><circle cx="320" cy="250" r="40" opacity=".7"/><circle cx="520" cy="250" r="40" opacity=".4"/></g></svg>`,
  );

type BaseTool = Omit<ToolDto, "screenshots" | "webUrl" | "docRevision">;

function withDocs(t: BaseTool): ToolDto {
  const release = t.delivery.type === "release";
  return {
    ...t,
    screenshots: t.id === "easy-git" ? [SHOT, SHOT] : [],
    webUrl: t.delivery.type === "release" ? `https://github.com/${t.delivery.repo}` : null,
    docRevision: release ? "v0.1.0" : "local",
  };
}

export function demoCatalog(): CatalogDto {
  return {
    tools: structuredClone(tools).map(withDocs),
    issues: [],
    settingsFile: "(tarayıcı önizlemesi)/settings.toml",
    userCatalogDir: "(tarayıcı önizlemesi)/catalog",
  };
}

export async function demoCheckUpdates(): Promise<UpdateCheckDto[]> {
  await sleep(600);
  return tools
    .filter((t) => t.delivery.type === "release")
    .map((t) => ({
      id: t.id,
      version: t.delivery.type === "release" ? (t.delivery.latest?.version ?? null) : null,
      error: null,
    }));
}

export async function demoReadme(id: string): Promise<ReadmeDto> {
  await sleep(300);
  const tool = tools.find((t) => t.id === id);
  return {
    html: `<h1 id="user-content-${id}">${tool?.name ?? id}</h1>
<p>${tool?.summary ?? ""}</p>
<p><img src="${SHOT}" alt="Ekran görüntüsü"></p>
<h2 id="user-content-kurulum">Kurulum</h2>
<pre><code>npm install
npm run tauri dev</code></pre>
<p>Ayrıntılar için <a href="https://github.com/buraksenyurt/${id}" rel="noopener noreferrer">GitHub</a> ve <a href="#user-content-kurulum">kurulum</a>.</p>
<table><thead><tr><th>Katman</th><th>Hedef</th></tr></thead><tbody><tr><td>Ürün</td><td>Tek bakışta anlaşılır bir harita</td></tr></tbody></table>
<blockquote><p>Demo içerik: tarayıcı önizlemesi.</p></blockquote>`,
    revision: "v0.1.0",
    webUrl: `https://github.com/buraksenyurt/${id}/blob/v0.1.0/README.md`,
    stale: false,
    error: null,
  };
}

export async function demoInstall(
  id: string,
  onProgress: (p: InstallProgressDto) => void,
): Promise<InstalledDto> {
  const tool = tools.find((t) => t.id === id);
  if (!tool || tool.delivery.type !== "release" || !tool.delivery.latest) {
    throw new Error("Demo: kurulacak sürüm yok");
  }
  const total = tool.delivery.latest.assetSize ?? 1;
  for (let i = 0; i <= 10; i++) {
    onProgress({ step: "downloading", received: Math.round((total * i) / 10), total });
    await sleep(120);
  }
  onProgress({ step: "verifying" });
  await sleep(250);
  onProgress({ step: "unpacking" });
  await sleep(250);
  const installed: InstalledDto = {
    version: tool.delivery.latest.version,
    folder: `(demo)/tools/${id}/${tool.delivery.latest.version}`,
    entry: `(demo)/tools/${id}/${tool.delivery.latest.version}/${id}`,
    installedAt: now(),
  };
  tool.delivery.installed.unshift(installed);
  tool.delivery.updateAvailable = false;
  return installed;
}

export async function demoUninstall(id: string, version: string): Promise<void> {
  const tool = tools.find((t) => t.id === id);
  if (tool?.delivery.type === "release") {
    tool.delivery.installed = tool.delivery.installed.filter((i) => i.version !== version);
  }
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

// --- Runs: a fake process that prints a few lines and stays up until stopped
// (tools) or finishes by itself (builds).

type RunHandlers = { changed: (r: RunDto) => void; log: (e: RunLogEvent) => void };

class DemoRuns {
  private runs: RunDto[] = [];
  private logs = new Map<number, LogLineDto[]>();
  private handlers = new Set<RunHandlers>();
  private nextId = 1;

  subscribe(h: RunHandlers): () => void {
    this.handlers.add(h);
    return () => this.handlers.delete(h);
  }

  async list(): Promise<RunDto[]> {
    return structuredClone(this.runs).reverse();
  }

  async log(id: number): Promise<LogLineDto[]> {
    return structuredClone(this.logs.get(id) ?? []);
  }

  async start(toolId: string, kind: "tool" | "build"): Promise<RunDto> {
    if (this.runs.some((r) => r.toolId === toolId && r.kind === kind && r.state.status === "running")) {
      throw new Error(`${toolId} zaten çalışıyor`);
    }
    const run: RunDto = {
      id: this.nextId++,
      toolId,
      kind,
      command: kind === "build" ? "zig build-exe src/main.zig -O ReleaseFast" : `(demo)/tools/${toolId}/${toolId}`,
      cwd: `(demo)/${toolId}`,
      pid: 4000 + this.nextId,
      startedAt: Date.now(),
      endedAt: null,
      state: { status: "running" },
    };
    this.runs.push(run);
    this.logs.set(run.id, []);
    this.emitChanged(run);
    const lines: LogLineDto[] =
      kind === "build"
        ? [
            { stream: "stdout", text: "info: derleme başladı" },
            { stream: "stderr", text: "warning: unused variable `x`" },
            { stream: "stdout", text: "info: tamamlandı (3,2 sn)" },
          ]
        : [
            { stream: "stdout", text: `${toolId} 0.1.0 başlatıldı` },
            { stream: "stderr", text: "[INFO] webview hazır" },
          ];
    lines.forEach((line, i) =>
      setTimeout(() => this.append(run.id, [line]), 300 * (i + 1)),
    );
    if (kind === "build") setTimeout(() => this.finish(run.id, 0, false), 300 * (lines.length + 1));
    return structuredClone(run);
  }

  async stop(id: number): Promise<void> {
    setTimeout(() => this.finish(id, null, true), 400);
  }

  private append(id: number, lines: LogLineDto[]) {
    const run = this.runs.find((r) => r.id === id);
    if (!run || run.state.status !== "running") return;
    this.logs.get(id)?.push(...lines);
    for (const h of this.handlers) h.log({ runId: id, lines: structuredClone(lines) });
  }

  private finish(id: number, code: number | null, stopped: boolean) {
    const run = this.runs.find((r) => r.id === id);
    if (!run || run.state.status !== "running") return;
    run.state = { status: "exited", code, stopped };
    run.endedAt = Date.now();
    this.emitChanged(run);
  }

  private emitChanged(run: RunDto) {
    for (const h of this.handlers) h.changed(structuredClone(run));
  }
}

export const demoRuns = new DemoRuns();

// --- Terminal: a tiny line-echo "program" with colours, for UI work.

class DemoTerminals {
  private sessions = new Map<number, { onData: (b: Uint8Array) => void; line: string }>();
  private exitHandlers = new Set<(e: TerminalExitEvent) => void>();
  private nextId = 1;
  private enc = new TextEncoder();

  async open(toolId: string, cols: number, rows: number, onData: (b: Uint8Array) => void): Promise<TerminalDto> {
    const sessionId = this.nextId++;
    this.sessions.set(sessionId, { onData, line: "" });
    const say = (s: string) => onData(this.enc.encode(s));
    setTimeout(() => {
      say(`\x1b[1;32m${toolId}\x1b[0m demo terminali (${cols}x${rows})\r\n`);
      say("Son 14 gün: \x1b[48;5;22m  \x1b[48;5;28m  \x1b[48;5;34m  \x1b[48;5;40m  \x1b[0m ğüşıöç\r\n");
      say("Bir şey yaz ve Enter'a bas (çıkmak için: exit)\r\n> ");
    }, 150);
    return { sessionId, toolId, commandLine: `${toolId} --demo`, cwd: "(demo)" };
  }

  async write(sessionId: number, data: string): Promise<void> {
    const s = this.sessions.get(sessionId);
    if (!s) throw new Error("oturum yok");
    const say = (t: string) => s.onData(this.enc.encode(t));
    for (const ch of data) {
      if (ch === "\r") {
        say("\r\n");
        if (s.line.trim() === "exit") return this.finish(sessionId, 0);
        say(`yankı: \x1b[36m${s.line}\x1b[0m\r\n> `);
        s.line = "";
      } else if (ch === "\x7f") {
        if (s.line) {
          s.line = s.line.slice(0, -1);
          say("\b \b");
        }
      } else {
        s.line += ch;
        say(ch);
      }
    }
  }

  async close(sessionId: number): Promise<void> {
    this.finish(sessionId, 1);
  }

  onExit(h: (e: TerminalExitEvent) => void): () => void {
    this.exitHandlers.add(h);
    return () => this.exitHandlers.delete(h);
  }

  private finish(sessionId: number, code: number) {
    if (!this.sessions.delete(sessionId)) return;
    for (const h of this.exitHandlers) h({ sessionId, code });
  }
}

export const demoTerminals = new DemoTerminals();
