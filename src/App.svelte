<script lang="ts">
  import { appInfo } from "./lib/api";
  import { catalog } from "./lib/catalog.svelte";
  import type { AppInfoDto } from "./lib/bindings/AppInfoDto";
  import Sidebar from "./lib/shell/Sidebar.svelte";
  import StatusBar from "./lib/shell/StatusBar.svelte";
  import TabBar from "./lib/shell/TabBar.svelte";
  import Workspace from "./lib/shell/Workspace.svelte";
  import { tabs } from "./lib/tabs.svelte";

  let info = $state<AppInfoDto | null>(null);
  let infoError = $state<string | null>(null);

  $effect(() => {
    appInfo()
      .then((i) => (info = i))
      .catch((e: Error) => (infoError = e.message));
    // Show what is cached right away, then ask GitHub for anything newer.
    catalog.load().then(() => catalog.checkUpdates());
  });

  function onKeydown(e: KeyboardEvent) {
    if (!(e.ctrlKey || e.metaKey)) return;
    if (e.key === "Tab") {
      e.preventDefault();
      tabs.cycle(e.shiftKey ? -1 : 1);
    } else if (e.key.toLowerCase() === "w") {
      e.preventDefault();
      tabs.close(tabs.activeId);
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="shell">
  <Sidebar />
  <main>
    <TabBar />
    <Workspace {info} {infoError} />
  </main>
  <StatusBar {info} />
</div>

<style>
  .shell {
    display: grid;
    grid-template-columns: var(--sidebar-width) minmax(0, 1fr);
    grid-template-rows: minmax(0, 1fr) var(--statusbar-height);
    height: 100%;
  }

  main {
    display: grid;
    grid-template-rows: var(--tabbar-height) minmax(0, 1fr);
    min-width: 0;
    min-height: 0;
  }
</style>
