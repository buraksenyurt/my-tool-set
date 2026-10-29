<script lang="ts">
  import type { AppInfoDto } from "../bindings/AppInfoDto";
  import { platformLabel } from "../format";

  let { info }: { info: AppInfoDto | null } = $props();
</script>

<footer>
  <span>Çalışan araç yok</span>
  <span class="right">
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
