<script lang="ts">
  import type { AppInfoDto } from "../bindings/AppInfoDto";
  import { catalog } from "../catalog.svelte";
  import { tabs } from "../tabs.svelte";
  import AboutView from "../views/AboutView.svelte";
  import HomeView from "../views/HomeView.svelte";
  import IssuesView from "../views/IssuesView.svelte";
  import { terminals } from "../terminals.svelte";
  import TerminalView from "../views/TerminalView.svelte";
  import ToolView from "../views/ToolView.svelte";

  let { info, infoError }: { info: AppInfoDto | null; infoError: string | null } = $props();

  const tool = $derived(
    tabs.active.kind === "tool" && tabs.active.toolId ? catalog.find(tabs.active.toolId) : undefined,
  );
  const terminalTabs = $derived(tabs.list.filter((t) => t.kind === "terminal"));
</script>

<!-- Terminals stay mounted (hidden) so their screen and running output survive tab switches. -->
{#each terminalTabs as tab (tab.id)}
  {@const session = terminals.sessions[tab.id]}
  {#if session}
    <div
      class="workspace terminal-panel"
      class:hidden={tabs.active.id !== tab.id}
      role="tabpanel"
      id="panel-{tab.id}"
      aria-labelledby="tab-{tab.id}"
    >
      <TerminalView {session} visible={tabs.active.id === tab.id} />
    </div>
  {/if}
{/each}

{#if tabs.active.kind !== "terminal"}
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
{/if}

<style>
  .workspace {
    grid-row: 2;
    grid-column: 1;
    overflow: auto;
    min-height: 0;
  }

  .terminal-panel {
    overflow: hidden;
  }

  .hidden {
    display: none;
  }

  .gone {
    padding: 36px 48px;
    color: var(--muted);
  }
</style>
