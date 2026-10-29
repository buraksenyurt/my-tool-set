<script lang="ts">
  import { inTauri } from "../api";
  import type { AppInfoDto } from "../bindings/AppInfoDto";
  import { platformKey, platformLabel } from "../format";

  let { info, error }: { info: AppInfoDto | null; error: string | null } = $props();
</script>

<div class="about">
  <h1>Hakkında</h1>

  {#if error}
    <p class="error" role="alert">Uygulama bilgisi alınamadı: {error}</p>
  {:else if !info}
    <p class="muted">Yükleniyor…</p>
  {:else}
    <dl>
      <dt>Sürüm</dt>
      <dd>{info.name} v{info.version}</dd>

      <dt>Platform</dt>
      <dd>
        {platformLabel(info.platform)}
        {#if info.platform}
          <code>{platformKey(info.platform)}</code>
        {:else if inTauri()}
          <span class="muted">— araçlar bu platform için yayınlanmıyor</span>
        {/if}
      </dd>

      <dt>Veri klasörü</dt>
      <dd><code>{info.dataDir}</code></dd>

      <dt>Kaynak</dt>
      <dd><code>github.com/buraksenyurt/my-tool-set</code></dd>
    </dl>
  {/if}
</div>

<style>
  .about {
    max-width: 720px;
    padding: 40px 48px;
  }

  h1 {
    margin: 0 0 24px;
    font-size: 22px;
    font-weight: 600;
  }

  dl {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 12px 24px;
    margin: 0;
  }

  dt {
    color: var(--muted);
  }

  dd {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 8px;
    margin: 0;
    overflow-wrap: anywhere;
  }

  .muted {
    color: var(--muted);
  }

  .error {
    color: var(--danger);
  }
</style>
