<script lang="ts">
  import type { AppInfoDto } from "../bindings/AppInfoDto";
  import { catalog } from "../catalog.svelte";
  import { tabs } from "../tabs.svelte";
  import AboutView from "../views/AboutView.svelte";
  import HomeView from "../views/HomeView.svelte";
  import IssuesView from "../views/IssuesView.svelte";
  import ToolView from "../views/ToolView.svelte";

  let { info, infoError }: { info: AppInfoDto | null; infoError: string | null } = $props();

  const tool = $derived(tabs.active.toolId ? catalog.find(tabs.active.toolId) : undefined);
</script>

<div class="workspace" role="tabpanel" id="panel-{tabs.active.id}" aria-labelledby="tab-{tabs.active.id}">
  {#if tabs.active.kind === "home"}
    <HomeView />
  {:else if tabs.active.kind === "about"}
    <AboutView {info} error={infoError} />
  {:else if tabs.active.kind === "issues"}
    <IssuesView />
  {:else if tabs.active.kind === "tool"}
    {#if tool}
      {#key tool.id}
        <ToolView {tool} />
      {/key}
    {:else if catalog.data}
      <p class="gone">Bu araç artık katalogda yok.</p>
    {/if}
  {/if}
</div>

<style>
  .workspace {
    overflow: auto;
    min-height: 0;
  }

  .gone {
    padding: 36px 48px;
    color: var(--muted);
  }
</style>
