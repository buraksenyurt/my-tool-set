<script lang="ts">
  import { catalog } from "../catalog.svelte";
</script>

<div class="issues">
  <h1>Katalog sorunları</h1>
  <p class="lead">
    Bu manifestler okunamadığı ya da kurallara uymadığı için listeye alınmadı. Diğer araçlar
    etkilenmez.
  </p>

  {#if catalog.issues.length === 0}
    <p class="muted">Sorun yok.</p>
  {:else}
    <ul>
      {#each catalog.issues as issue, i (i)}
        <li>
          <code>{issue.location}</code>
          <p>{issue.message}</p>
        </li>
      {/each}
    </ul>
  {/if}

  <p class="muted">
    Kendi araçlarını <code>{catalog.data?.userCatalogDir}</code> altına
    <code>&lt;id&gt;/tool.toml</code> olarak ekleyebilirsin; aynı <code>id</code> yerleşik bir aracı
    geçersiz kılar.
  </p>
  <button onclick={() => catalog.reload()}>Kataloğu yeniden yükle</button>
</div>

<style>
  .issues {
    max-width: 760px;
    padding: 36px 48px;
  }

  h1 {
    margin: 0 0 8px;
    font-size: 22px;
    font-weight: 600;
  }

  .lead {
    margin: 0 0 20px;
    color: var(--text-secondary);
  }

  ul {
    margin: 0 0 20px;
    padding: 0;
    list-style: none;
  }

  li {
    padding: 12px 14px;
    margin-bottom: 8px;
    border: 1px solid var(--border);
    border-left: 3px solid var(--warn);
    border-radius: var(--radius);
    background: var(--surface);
  }

  li code {
    overflow-wrap: anywhere;
    color: var(--text-secondary);
  }

  li p {
    margin: 6px 0 0;
    overflow-wrap: anywhere;
  }

  .muted {
    color: var(--muted);
    line-height: 1.5;
  }

  button {
    height: 30px;
    padding: 0 14px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    cursor: pointer;
  }
</style>
