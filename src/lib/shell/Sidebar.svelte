<script lang="ts">
  import { openAbout } from "../tabs.svelte";
  import { theme } from "../theme.svelte";

  const themeLabel = $derived(
    { system: "Sistem teması", light: "Açık tema", dark: "Koyu tema" }[theme.current],
  );
</script>

<aside>
  <header>
    <span class="brand">my-tool-set</span>
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
            <path d="M8 1.5v1.6M8 12.9v1.6M1.5 8h1.6M12.9 8h1.6M3.4 3.4l1.1 1.1M11.5 11.5l1.1 1.1M3.4 12.6l1.1-1.1M11.5 4.5l1.1-1.1" />
          </g>
        {:else if theme.current === "dark"}
          <path d="M13.5 9.6A5.6 5.6 0 0 1 6.4 2.5a5.6 5.6 0 1 0 7.1 7.1Z" fill="currentColor" />
        {:else}
          <circle cx="8" cy="8" r="5.6" fill="none" stroke="currentColor" stroke-width="1.4" />
          <path d="M8 2.4a5.6 5.6 0 0 1 0 11.2Z" fill="currentColor" />
        {/if}
      </svg>
    </button>
  </header>

  <nav aria-label="Araçlar">
    <p class="section">Araçlar</p>
    <div class="empty">
      <p>Katalog henüz bağlanmadı.</p>
      <p class="hint">Araçlar bir sonraki fazda <code>catalog/</code> klasöründen ve GitHub Releases'tan yüklenecek.</p>
    </div>
  </nav>

  <footer>
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
    padding: 0 8px 0 14px;
    border-bottom: 1px solid var(--border);
  }

  .brand {
    font-weight: 600;
    letter-spacing: 0.01em;
  }

  nav {
    flex: 1;
    overflow-y: auto;
    padding: 10px 8px;
  }

  .section {
    margin: 4px 6px 8px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .empty {
    margin: 0 6px;
    padding: 12px;
    border: 1px dashed var(--border);
    border-radius: var(--radius);
    color: var(--text-secondary);
    font-size: 13px;
  }

  .empty p {
    margin: 0;
  }

  .empty .hint {
    margin-top: 6px;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.45;
  }

  footer {
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
    cursor: pointer;
  }

  button.icon:hover {
    background: var(--hover);
  }

  button.icon {
    color: var(--text-secondary);
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
</style>
