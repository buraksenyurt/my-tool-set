<script lang="ts">
  import type { AppInfoDto } from "../bindings/AppInfoDto";
  import { catalog } from "../catalog.svelte";
  import { platformLabel } from "../format";
  import { runs } from "../runs.svelte";
  import { terminals } from "../terminals.svelte";

  let { info }: { info: AppInfoDto | null } = $props();

  const installing = $derived(
    Object.keys(catalog.progress)
      .map((id) => catalog.find(id)?.name ?? id)
      .join(", "),
  );
  const name = (id: string) => catalog.find(id)?.name ?? id;
  const runningTools = $derived([
    ...new Set([
      ...runs.running.filter((r) => r.kind === "tool").map((r) => name(r.toolId)),
      ...Object.values(terminals.sessions)
        .filter((s) => s.status === "running")
        .map((s) => s.toolName),
    ]),
  ]);
  const buildingTools = $derived(runs.running.filter((r) => r.kind === "build").map((r) => name(r.toolId)));
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
    {:else if buildingTools.length > 0}
      {buildingTools.join(", ")} derleniyor…
    {:else if runningTools.length > 0}
      <span class="dot"></span>{runningTools.join(", ")} çalışıyor
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

  .dot {
    display: inline-block;
    width: 7px;
    height: 7px;
    margin-right: 6px;
    border-radius: 50%;
    background: var(--ok);
  }

  .right {
    display: flex;
    gap: 14px;
  }
</style>
