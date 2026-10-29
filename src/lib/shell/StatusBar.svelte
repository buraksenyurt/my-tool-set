<script lang="ts">
  import type { AppInfoDto } from "../bindings/AppInfoDto";
  import { catalog } from "../catalog.svelte";
  import { platformLabel } from "../format";

  let { info }: { info: AppInfoDto | null } = $props();

  const installing = $derived(
    Object.keys(catalog.progress)
      .map((id) => catalog.find(id)?.name ?? id)
      .join(", "),
  );
  const updates = $derived(
    catalog.tools.filter((t) => t.delivery.type === "release" && t.delivery.updateAvailable).length,
  );
</script>

<footer>
  <span>
    {#if installing}
      {installing} kuruluyor…
    {:else if catalog.checking}
      Güncellemeler denetleniyor…
    {:else if updates > 0}
      {updates} araç için güncelleme var
    {:else}
      Çalışan araç yok
    {/if}
  </span>
  <span class="right">
    {#if catalog.data}<span>{catalog.tools.length} araç</span>{/if}
    {#if info}
      <span>{platformLabel(info.platform)}</span>
      <span>v{info.version}</span>
    {/if}
  </span>
</footer>

<style>
  footer {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    border-top: 1px solid var(--border);
    background: var(--sidebar);
    color: var(--muted);
    font-size: 12px;
  }

  .right {
    display: flex;
    gap: 14px;
  }
</style>
