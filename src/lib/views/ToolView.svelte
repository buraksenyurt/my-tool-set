<script lang="ts">
  import type { InstallProgressDto } from "../bindings/InstallProgressDto";
  import type { ToolDto } from "../bindings/ToolDto";
  import { catalog, toolState } from "../catalog.svelte";
  import { formatAgo, formatBytes, HOST_LABELS, SUPPORT_LABELS } from "../format";
  import { openExternal } from "../api";
  import Lightbox from "../Lightbox.svelte";
  import StatePill from "../StatePill.svelte";
  import Readme from "./Readme.svelte";

  let { tool }: { tool: ToolDto } = $props();

  let view = $state<"overview" | "readme">("overview");
  let zoomed = $state<string | null>(null);

  const current = $derived(toolState(tool));
  const progress = $derived(catalog.progress[tool.id]);
  const actionError = $derived(catalog.actionErrors[tool.id]);
  const checkError = $derived(catalog.checkErrors[tool.id]);

  /** Version waiting for a second click before it is removed. */
  let confirmRemove = $state<string | null>(null);

  function progressText(p: InstallProgressDto | "starting"): string {
    if (p === "starting") return "Başlıyor…";
    switch (p.step) {
      case "downloading":
        return `İndiriliyor · ${formatBytes(p.received)} / ${formatBytes(p.total)}`;
      case "verifying":
        return "SHA256 doğrulanıyor…";
      case "unpacking":
        return "Açılıyor…";
    }
  }

  function progressRatio(p: InstallProgressDto | "starting"): number | null {
    if (p === "starting" || p.step !== "downloading" || p.total === 0) return null;
    return p.received / p.total;
  }
</script>

<article class="tool">
  <header>
    <h1>{tool.name}</h1>
    <StatePill state={current} />
    {#if tool.webUrl}
      <button class="link repo" onclick={() => openExternal(tool.webUrl!)}>GitHub ↗</button>
    {/if}
  </header>
  <p class="summary">{tool.summary}</p>

  <ul class="chips" aria-label="Yığın">
    {#each tool.stack as item (item)}
      <li>{item}</li>
    {/each}
  </ul>

  <div class="views" role="tablist" aria-label="Araç görünümü">
    <button role="tab" aria-selected={view === "overview"} onclick={() => (view = "overview")}>Genel bakış</button>
    <button role="tab" aria-selected={view === "readme"} onclick={() => (view = "readme")}>README</button>
  </div>

  {#if view === "readme"}
    <Readme toolId={tool.id} revision={tool.docRevision} />
  {:else}

  <dl class="facts">
    <dt>Tür</dt>
    <dd>{tool.kind === "gui" ? "Masaüstü uygulaması" : "Komut satırı aracı"} · {HOST_LABELS[tool.host]}</dd>
    <dt>Platform</dt>
    <dd class:muted={tool.support !== "yes"}>{SUPPORT_LABELS[tool.support]}</dd>
    {#if tool.origin !== "embedded"}
      <dt>Manifest</dt>
      <dd>{tool.origin === "user" ? "Kullanıcı kataloğundan" : "Repodaki catalog/ klasöründen (geliştirme)"}</dd>
    {/if}
  </dl>

  {#if tool.screenshots.length > 0}
    <ul class="gallery" aria-label="Ekran görüntüleri">
      {#each tool.screenshots as src, i (src + i)}
        <li>
          <button aria-label="Ekran görüntüsü {i + 1}, büyütmek için tıkla" onclick={() => (zoomed = src)}>
            <img {src} alt="Ekran görüntüsü {i + 1}" loading="lazy" />
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  {#if tool.delivery.type === "release"}
    {@const d = tool.delivery}
    <section>
      <h2>Sürüm</h2>
      <p class="line">
        <code>github.com/{d.repo}</code>
      </p>

      {#if d.latest}
        <p class="line">
          Son sürüm <strong>v{d.latest.version}</strong>
          {#if d.latest.assetSize !== null}· {formatBytes(d.latest.assetSize)}{/if}
          <span class="muted">· {formatAgo(d.latest.fetchedAt)} denetlendi</span>
        </p>
        {#if !d.latest.hasBuild}
          <p class="note">Bu sürümde bu platform için bir derleme yok.</p>
        {/if}
      {:else if catalog.checking}
        <p class="muted">Son sürüm denetleniyor…</p>
      {:else}
        <p class="note">Henüz bilinen bir sürüm yok.</p>
      {/if}
      {#if checkError}
        <p class="error">Son denetim başarısız: {checkError}</p>
      {/if}

      {#if progress}
        {@const ratio = progressRatio(progress)}
        <div class="progress" role="status">
          <div class="bar" class:indeterminate={ratio === null}>
            <span style:width={ratio === null ? undefined : `${Math.round(ratio * 100)}%`}></span>
          </div>
          <span class="muted">{progressText(progress)}</span>
        </div>
      {:else if d.latest?.hasBuild && (d.installed.length === 0 || d.updateAvailable)}
        <button class="primary" onclick={() => catalog.install(tool.id)}>
          {d.installed.length === 0 ? "Kur" : "Güncelle"} · v{d.latest.version}
        </button>
      {/if}
      {#if actionError}
        <p class="error" role="alert">{actionError}</p>
      {/if}

      {#if d.installed.length > 0}
        <h3>Kurulu sürümler</h3>
        <ul class="installed">
          {#each d.installed as item, i (item.version)}
            <li>
              <div>
                <strong>v{item.version}</strong>
                {#if i === 0}<span class="muted">· kullanılan</span>{/if}
                <span class="muted">· {formatAgo(item.installedAt)} kuruldu</span>
                <code class="path">{item.folder}</code>
              </div>
              {#if confirmRemove === item.version}
                <span class="confirm">
                  <button class="danger" onclick={() => {
                    confirmRemove = null;
                    catalog.uninstall(tool.id, item.version);
                  }}>Kaldır</button>
                  <button class="ghost" onclick={() => (confirmRemove = null)}>Vazgeç</button>
                </span>
              {:else}
                <button class="ghost" disabled={!!progress} onclick={() => (confirmRemove = item.version)}
                  >Kaldır…</button
                >
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {:else}
    {@const d = tool.delivery}
    <section>
      <h2>Yerel kaynak</h2>
      {#if d.status.kind === "ready"}
        <p class="line">Çalıştırılacak dosya hazır:</p>
        <code class="path">{d.status.exe}</code>
      {:else if d.status.kind === "fromSource"}
        <p class="line">Kaynaktan çalışır:</p>
        <code class="path">{d.status.folder}</code>
        {#if d.devCommand}<pre>{d.devCommand}</pre>{/if}
      {:else if d.status.kind === "notBuilt"}
        <p class="line">Klasör bulundu ama araç henüz derlenmemiş:</p>
        <code class="path">{d.status.folder}</code>
        {#if d.buildCommand}
          <p class="line">Klasörde şu komutla derlenir:</p>
          <pre>{d.buildCommand}</pre>
        {/if}
      {:else if d.status.kind === "unknownRoot"}
        <p class="line">
          <code>settings.toml</code> içinde <code>[roots]</code> altında
          <code>{d.status.root}</code> tanımlı değil. Dosyaya şöyle bir satır ekleyip kataloğu yeniden yükle:
        </p>
        <pre>[roots]
{d.status.root} = "C:/Users/burak/Development/{d.status.root}"</pre>
        <p class="line muted">Dosya: <code class="path">{catalog.data?.settingsFile}</code></p>
        <button class="ghost" onclick={() => catalog.reload()}>Kataloğu yeniden yükle</button>
      {:else if d.status.kind === "missingFolder"}
        <p class="line">Araç klasörü bulunamadı:</p>
        <code class="path">{d.status.folder}</code>
        <button class="ghost" onclick={() => catalog.reload()}>Yeniden dene</button>
      {:else}
        <p class="note">Manifestte bu işletim sistemi için çalıştırılacak dosya tanımlı değil.</p>
      {/if}

      {#if tool.params.length > 0}
        <h3>Parametreler</h3>
        <ul class="params">
          {#each tool.params as param (param.name)}
            <li>
              {param.label}
              <span class="muted">
                · <code>{param.name}</code>{#if param.default != null}&nbsp;= {param.default}{/if}
              </span>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}
  {/if}
</article>

{#if zoomed}
  <Lightbox src={zoomed} alt={tool.name} onclose={() => (zoomed = null)} />
{/if}

<style>
  .tool {
    max-width: 880px;
    padding: 36px 48px 48px;
  }

  .views {
    display: flex;
    gap: 4px;
    margin: 0 0 20px;
    border-bottom: 1px solid var(--border);
  }

  .views button {
    height: 32px;
    padding: 0 12px;
    border: 0;
    border-bottom: 2px solid transparent;
    border-radius: 0;
    margin-bottom: -1px;
    background: none;
    color: var(--text-secondary);
  }

  .views button[aria-selected="true"] {
    border-bottom-color: var(--accent);
    color: var(--text);
    font-weight: 600;
  }

  button.link {
    height: auto;
    padding: 0;
    border: 0;
    background: none;
    color: var(--accent);
  }

  .repo {
    margin-left: auto;
  }

  .gallery {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 12px;
    margin: 24px 0 0;
    padding: 0;
    list-style: none;
  }

  .gallery button {
    display: block;
    width: 100%;
    height: auto;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    overflow: hidden;
    cursor: zoom-in;
  }

  .gallery img {
    display: block;
    width: 100%;
    aspect-ratio: 16 / 10;
    object-fit: cover;
  }

  header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  h1 {
    margin: 0;
    font-size: 24px;
    font-weight: 600;
  }

  h2 {
    margin: 32px 0 10px;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }

  h3 {
    margin: 24px 0 8px;
    font-size: 13px;
    font-weight: 600;
  }

  .summary {
    margin: 8px 0 16px;
    color: var(--text-secondary);
    font-size: 15px;
    line-height: 1.45;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin: 0 0 18px;
    padding: 0;
    list-style: none;
  }

  .chips li {
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--surface);
    color: var(--text-secondary);
    font-size: 12px;
  }

  .facts {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 6px 20px;
    margin: 0;
  }

  .facts dt {
    color: var(--muted);
  }

  .facts dd {
    margin: 0;
  }

  section {
    border-top: 1px solid var(--border);
    margin-top: 28px;
  }

  .line {
    margin: 6px 0;
    line-height: 1.5;
  }

  .note {
    margin: 8px 0;
    padding: 8px 12px;
    border-radius: var(--radius);
    background: var(--hover);
    color: var(--text-secondary);
  }

  .muted {
    color: var(--muted);
  }

  .error {
    margin: 8px 0;
    color: var(--danger);
    overflow-wrap: anywhere;
  }

  code.path {
    display: block;
    margin: 4px 0;
    color: var(--text-secondary);
    overflow-wrap: anywhere;
  }

  pre {
    margin: 8px 0;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    font-family: "Cascadia Code", "JetBrains Mono", ui-monospace, monospace;
    font-size: 12.5px;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  button {
    height: 30px;
    padding: 0 14px;
    border-radius: var(--radius);
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  button.primary {
    margin-top: 12px;
    border: 1px solid var(--accent);
    background: var(--accent);
    color: #fff;
    font-weight: 600;
  }

  button.primary:hover {
    filter: brightness(1.08);
  }

  button.ghost {
    border: 1px solid var(--border);
    background: var(--surface);
  }

  button.ghost:hover:not(:disabled) {
    border-color: var(--text-secondary);
  }

  button.danger {
    border: 1px solid var(--danger);
    background: var(--danger);
    color: #fff;
  }

  .progress {
    display: grid;
    gap: 6px;
    max-width: 420px;
    margin-top: 12px;
  }

  .bar {
    position: relative;
    height: 6px;
    border-radius: 3px;
    background: var(--hover);
    overflow: hidden;
  }

  .bar span {
    position: absolute;
    inset: 0 auto 0 0;
    background: var(--accent);
    transition: width 0.15s linear;
  }

  .bar.indeterminate span {
    width: 30%;
    animation: slide 1.1s ease-in-out infinite;
  }

  @keyframes slide {
    from {
      left: -30%;
    }
    to {
      left: 100%;
    }
  }

  .installed,
  .params {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .installed li {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    padding: 10px 0;
    border-bottom: 1px solid var(--border);
  }

  .installed li > div {
    min-width: 0;
  }

  .confirm {
    display: flex;
    gap: 6px;
  }

  .params li {
    padding: 4px 0;
  }
</style>
