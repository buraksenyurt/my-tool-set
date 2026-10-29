// Demo data for `npm run dev` in a plain browser. Mirrors the real catalog/
// so the UI can be designed without the desktop shell. Never used in Tauri.

import type { CatalogDto } from "./bindings/CatalogDto";
import type { InstallProgressDto } from "./bindings/InstallProgressDto";
import type { InstalledDto } from "./bindings/InstalledDto";
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
      type: "local",
      status: {
        kind: "ready",
        folder: "C:\\Users\\burak\\Development\\coding-vibes\\GitDash",
        exe: "C:\\Users\\burak\\Development\\coding-vibes\\GitDash\\zig-out\\bin\\GitDash.exe",
      },
      buildCommand: "zig build-exe src/main.zig -O ReleaseFast -femit-bin=zig-out/bin/GitDash",
      devCommand: null,
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
