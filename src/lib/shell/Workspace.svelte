<script lang="ts">
  import type { AppInfoDto } from "../bindings/AppInfoDto";
  import { tabs } from "../tabs.svelte";
  import AboutView from "../views/AboutView.svelte";
  import HomeView from "../views/HomeView.svelte";

  let { info, infoError }: { info: AppInfoDto | null; infoError: string | null } = $props();
</script>

<div
  class="workspace"
  role="tabpanel"
  id="panel-{tabs.active.id}"
  aria-labelledby="tab-{tabs.active.id}"
>
  {#if tabs.active.kind === "home"}
    <HomeView />
  {:else if tabs.active.kind === "about"}
    <AboutView {info} error={infoError} />
  {/if}
</div>

<style>
  .workspace {
    overflow: auto;
    min-height: 0;
  }
</style>
