<script lang="ts">
  import { openExternal } from "../api";
  import type { ReadmeDto } from "../bindings/ReadmeDto";
  import Lightbox from "../Lightbox.svelte";
  import { loadReadme } from "../readme";

  let { toolId, revision }: { toolId: string; revision: string | null } = $props();

  let readme = $state<ReadmeDto | null>(null);
  let failure = $state<string | null>(null);
  let loading = $state(true);
  let zoomed = $state<{ src: string; alt: string } | null>(null);
  let container: HTMLElement | undefined = $state();

  function load(force = false) {
    loading = true;
    failure = null;
    const id = toolId;
    const rev = revision;
    loadReadme(id, rev, force)
      .then((r) => {
        if (id === toolId && rev === revision) readme = r;
      })
      .catch((e: Error) => (failure = e.message))
      .finally(() => (loading = false));
  }

  $effect(() => {
    // Re-run when the tool or the shown revision changes.
    void toolId;
    void revision;
    load();
  });

  // Links must never navigate the shell's own window.
  function onClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    const img = target.closest("img");
    const link = target.closest("a");
    if (link) {
      e.preventDefault();
      const href = link.getAttribute("href") ?? "";
      if (href.startsWith("#")) {
        const raw = decodeURIComponent(href.slice(1));
        const id = raw.startsWith("user-content-") ? raw : `user-content-${raw}`;
        container?.querySelector(`[id="${CSS.escape(id)}"]`)?.scrollIntoView({ behavior: "smooth" });
      } else {
        openExternal(href);
      }
      return;
    }
    if (img?.getAttribute("src")) {
      zoomed = { src: img.getAttribute("src")!, alt: img.getAttribute("alt") ?? "" };
    }
  }
</script>

<div class="readme">
  {#if loading && !readme}
    <p class="muted">README yükleniyor…</p>
  {:else if failure}
    <p class="error" role="alert">README alınamadı: {failure}</p>
    <button class="ghost" onclick={() => load(true)}>Tekrar dene</button>
  {:else if readme}
    <div class="meta">
      {#if readme.revision}
        <span class="muted">
          {readme.revision === "local"
            ? "Yerel klasörden"
            : readme.revision === "HEAD"
              ? "Varsayılan daldan (henüz sürüm yok)"
              : `${readme.revision} sürümünden`}
        </span>
      {/if}
      {#if readme.stale}
        <span class="stale">Çevrimdışı · önbellekteki kopya</span>
      {/if}
      {#if readme.webUrl}
        <button class="link" onclick={() => openExternal(readme!.webUrl!)}>GitHub'da aç ↗</button>
      {/if}
    </div>

    {#if readme.html}
      <!-- Sanitized in Rust (ammonia): no scripts, handlers, styles or foreign schemes. -->
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="markdown" bind:this={container} onclick={onClick}>
        {@html readme.html}
      </div>
    {:else}
      <p class="note">{readme.error ?? "Bu araç için README yok."}</p>
      {#if readme.error}
        <button class="ghost" onclick={() => load(true)}>Tekrar dene</button>
      {/if}
    {/if}
  {/if}
</div>

{#if zoomed}
  <Lightbox src={zoomed.src} alt={zoomed.alt} onclose={() => (zoomed = null)} />
{/if}

<style>
  .readme {
    padding-top: 8px;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
    font-size: 13px;
  }

  .muted {
    color: var(--muted);
  }

  .stale {
    padding: 1px 8px;
    border-radius: 999px;
    background: var(--warn-soft);
    color: var(--warn);
    font-size: 12px;
  }

  .error {
    color: var(--danger);
    overflow-wrap: anywhere;
  }

  .note {
    padding: 8px 12px;
    border-radius: var(--radius);
    background: var(--hover);
    color: var(--text-secondary);
  }

  button.link {
    padding: 0;
    border: 0;
    background: none;
    color: var(--accent);
    cursor: pointer;
  }

  button.ghost {
    height: 30px;
    padding: 0 14px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    cursor: pointer;
  }

  /* --- Markdown, in the spirit of GitHub's, on the shell's tokens --- */
  .markdown {
    line-height: 1.6;
    overflow-wrap: break-word;
  }

  .markdown :global(h1),
  .markdown :global(h2) {
    padding-bottom: 0.3em;
    border-bottom: 1px solid var(--border);
  }

  .markdown :global(h1) {
    margin: 0.6em 0 0.6em;
    font-size: 1.8em;
    font-weight: 600;
  }

  .markdown :global(h2) {
    margin: 1.6em 0 0.7em;
    font-size: 1.4em;
    font-weight: 600;
  }

  .markdown :global(h3) {
    margin: 1.4em 0 0.6em;
    font-size: 1.15em;
    font-weight: 600;
  }

  .markdown :global(h4),
  .markdown :global(h5),
  .markdown :global(h6) {
    margin: 1.2em 0 0.5em;
    font-size: 1em;
    font-weight: 600;
  }

  .markdown :global(p),
  .markdown :global(ul),
  .markdown :global(ol),
  .markdown :global(table),
  .markdown :global(pre),
  .markdown :global(blockquote) {
    margin: 0 0 1em;
  }

  .markdown :global(ul),
  .markdown :global(ol) {
    padding-left: 1.8em;
  }

  .markdown :global(li + li) {
    margin-top: 0.25em;
  }

  .markdown :global(a) {
    color: var(--accent);
    text-decoration: none;
    cursor: pointer;
  }

  .markdown :global(a:hover) {
    text-decoration: underline;
  }

  .markdown :global(img) {
    max-width: 100%;
    height: auto;
    border-radius: 4px;
    cursor: zoom-in;
  }

  .markdown :global(code) {
    padding: 0.15em 0.35em;
    border-radius: 4px;
    background: var(--hover);
  }

  .markdown :global(pre) {
    padding: 12px 14px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    overflow-x: auto;
    line-height: 1.45;
  }

  .markdown :global(pre code) {
    padding: 0;
    background: none;
    font-size: 12.5px;
  }

  .markdown :global(blockquote) {
    padding: 0 1em;
    border-left: 3px solid var(--border);
    color: var(--text-secondary);
  }

  .markdown :global(table) {
    display: block;
    max-width: 100%;
    overflow-x: auto;
    border-collapse: collapse;
  }

  .markdown :global(th),
  .markdown :global(td) {
    padding: 6px 12px;
    border: 1px solid var(--border);
    vertical-align: top;
    text-align: left;
  }

  .markdown :global(th) {
    background: var(--hover);
    font-weight: 600;
  }

  .markdown :global(hr) {
    height: 1px;
    margin: 1.5em 0;
    border: 0;
    background: var(--border);
  }

  .markdown :global(input[type="checkbox"]) {
    margin-right: 0.4em;
  }
</style>
