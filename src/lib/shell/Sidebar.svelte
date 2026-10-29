<script lang="ts">
  import { catalog, toolState } from "../catalog.svelte";
  import type { ToolDto } from "../bindings/ToolDto";
  import StatePill from "../StatePill.svelte";
  import { openAbout, openIssues, openTool, tabs } from "../tabs.svelte";
  import { theme } from "../theme.svelte";

  let query = $state("");

  const themeLabel = $derived(
    { system: "Sistem teması", light: "Açık tema", dark: "Koyu tema" }[theme.current],
  );

  function matches(tool: ToolDto, q: string): boolean {
    if (!q) return true;
    const hay = [tool.name, tool.summary, tool.category, ...tool.tags, ...tool.stack]
      .join(" ")
      .toLocaleLowerCase("tr-TR");
    return q
      .toLocaleLowerCase("tr-TR")
      .split(/\s+/)
      .every((word) => hay.includes(word));
  }

  const groups = $derived.by(() => {
    const map = new Map<string, ToolDto[]>();
    for (const tool of catalog.tools) {
      if (!matches(tool, query.trim())) continue;
      const list = map.get(tool.category) ?? [];
      list.push(tool);
      map.set(tool.category, list);
    }
    return [...map.entries()];
  });

  const activeToolId = $derived(tabs.active.kind === "tool" ? tabs.active.toolId : undefined);

  /** Shown next to "Kurulu" only; for "Güncelleme var" it would read as the new version. */
  function installedVersion(tool: ToolDto): string | undefined {
    if (tool.delivery.type !== "release" || tool.delivery.updateAvailable) return undefined;
    return tool.delivery.installed[0]?.version;
  }
</script>

<aside>
  <header>
    <span class="brand">my-tool-set</span>
    <span class="actions">
      <button
        class="icon"
        class:spinning={catalog.checking}
        title="Güncellemeleri denetle"
        aria-label="Güncellemeleri denetle"
        disabled={catalog.checking}
        onclick={() => catalog.checkUpdates()}
      >
        <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden="true">
          <path
            d="M13.2 8a5.2 5.2 0 1 1-1.6-3.75M13.4 2.2v3.1h-3.1"
            fill="none"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
      <button
        class="icon"
        title="{themeLabel} — değiştirmek için tıkla"
        aria-label={themeLabel}
        onclick={() => theme.next()}
      >
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
          {#if theme.current === "light"}
            <circle cx="8" cy="8" r="3" fill="currentColor" />
            <g stroke="currentColor" stroke-width="1.4" stroke-linecap="round">
              <path
                d="M8 1.5v1.6M8 12.9v1.6M1.5 8h1.6M12.9 8h1.6M3.4 3.4l1.1 1.1M11.5 11.5l1.1 1.1M3.4 12.6l1.1-1.1M11.5 4.5l1.1-1.1"
              />
            </g>
          {:else if theme.current === "dark"}
            <path d="M13.5 9.6A5.6 5.6 0 0 1 6.4 2.5a5.6 5.6 0 1 0 7.1 7.1Z" fill="currentColor" />
          {:else}
            <circle cx="8" cy="8" r="5.6" fill="none" stroke="currentColor" stroke-width="1.4" />
            <path d="M8 2.4a5.6 5.6 0 0 1 0 11.2Z" fill="currentColor" />
          {/if}
        </svg>
      </button>
    </span>
  </header>

  <div class="search">
    <input
      type="search"
      placeholder="Araç ara"
      aria-label="Araç ara"
      bind:value={query}
      onkeydown={(e) => e.key === "Escape" && (query = "")}
    />
  </div>

  <nav aria-label="Araçlar">
    {#if catalog.loadError}
      <p class="error" role="alert">{catalog.loadError}</p>
    {:else if !catalog.data}
      <p class="muted">Yükleniyor…</p>
    {:else if groups.length === 0}
      <p class="muted">
        {query ? "Eşleşen araç yok." : "Katalog boş."}
      </p>
    {/if}

    {#each groups as [category, list] (category)}
      <!-- lang="en": category ids are English, so "git" must not become "GİT" -->
      <p class="section" lang="en">{category}</p>
      <ul>
        {#each list as tool (tool.id)}
          <li>
            <button
              class="tool"
              class:active={tool.id === activeToolId}
              aria-current={tool.id === activeToolId ? "page" : undefined}
              onclick={() => openTool(tool.id, tool.name)}
            >
              <span class="name">{tool.name}</span>
              {#if catalog.progress[tool.id]}
                <span class="busy" aria-label="Kuruluyor"></span>
              {:else}
                <StatePill state={toolState(tool)} version={installedVersion(tool)} />
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    {/each}
  </nav>

  <footer>
    {#if catalog.issues.length > 0}
      <button class="link warn" onclick={openIssues}>
        ⚠ {catalog.issues.length} katalog sorunu
      </button>
    {/if}
    <button class="link" onclick={openAbout}>Hakkında</button>
  </footer>
</aside>

<style>
  aside {
    grid-row: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    background: var(--sidebar);
    border-right: 1px solid var(--border);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--tabbar-height);
    padding: 0 6px 0 14px;
    border-bottom: 1px solid var(--border);
  }

  .brand {
    font-weight: 600;
    letter-spacing: 0.01em;
  }

  .actions {
    display: flex;
    gap: 2px;
  }

  .search {
    padding: 10px 10px 4px;
  }

  input[type="search"] {
    width: 100%;
    height: 28px;
    padding: 0 9px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: 13px;
  }

  input[type="search"]:focus {
    outline: none;
    border-color: var(--accent);
  }

  nav {
    flex: 1;
    overflow-y: auto;
    padding: 4px 6px 10px;
  }

  .section {
    margin: 12px 8px 4px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  button.tool {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    height: 30px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius);
    background: none;
    text-align: left;
    cursor: pointer;
  }

  button.tool:hover {
    background: var(--hover);
  }

  button.tool.active {
    background: var(--accent-soft);
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .busy {
    width: 12px;
    height: 12px;
    border: 2px solid var(--accent);
    border-right-color: transparent;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .muted,
  .error {
    margin: 12px 8px;
    font-size: 13px;
  }

  .muted {
    color: var(--muted);
  }

  .error {
    color: var(--danger);
  }

  footer {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    padding: 8px 14px;
    border-top: 1px solid var(--border);
  }

  button.icon {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: var(--radius);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
  }

  button.icon:hover:not(:disabled) {
    background: var(--hover);
  }

  button.icon:disabled {
    cursor: default;
  }

  button.icon.spinning svg {
    animation: spin 0.9s linear infinite;
  }

  button.link {
    padding: 0;
    border: 0;
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
  }

  button.link:hover {
    color: var(--accent);
  }

  button.link.warn {
    color: var(--warn);
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .busy,
    button.icon.spinning svg {
      animation-duration: 3s;
    }
  }
</style>
