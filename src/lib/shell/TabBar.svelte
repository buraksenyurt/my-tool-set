<script lang="ts">
  import { tabs } from "../tabs.svelte";

  function onAuxClick(e: MouseEvent, id: string) {
    // Middle click closes, like in browsers.
    if (e.button === 1) tabs.close(id);
  }
</script>

<div class="tabbar" role="tablist" aria-label="Açık sekmeler">
  {#each tabs.list as tab (tab.id)}
    <div class="tab" class:active={tab.id === tabs.activeId}>
      <button
        role="tab"
        id="tab-{tab.id}"
        aria-selected={tab.id === tabs.activeId}
        aria-controls="panel-{tab.id}"
        onclick={() => (tabs.activeId = tab.id)}
        onauxclick={(e) => onAuxClick(e, tab.id)}>{tab.title}</button
      >
      {#if tab.closable}
        <button class="close" aria-label="{tab.title} sekmesini kapat" onclick={() => tabs.close(tab.id)}
          >×</button
        >
      {/if}
    </div>
  {/each}
</div>

<style>
  .tabbar {
    display: flex;
    align-items: stretch;
    overflow-x: auto;
    background: var(--sidebar);
    border-bottom: 1px solid var(--border);
    scrollbar-width: none;
  }

  .tab {
    display: flex;
    align-items: center;
    border-right: 1px solid var(--border);
    color: var(--text-secondary);
  }

  .tab.active {
    background: var(--bg);
    color: var(--text);
    box-shadow: inset 0 2px 0 var(--accent);
    margin-bottom: -1px;
  }

  .tab button {
    border: 0;
    background: none;
    cursor: pointer;
  }

  .tab button[role="tab"] {
    height: 100%;
    padding: 0 14px;
    white-space: nowrap;
  }

  .tab:has(.close) button[role="tab"] {
    padding-right: 4px;
  }

  .close {
    width: 20px;
    height: 20px;
    margin-right: 8px;
    border-radius: 4px;
    color: var(--muted);
    font-size: 15px;
    line-height: 1;
  }

  .close:hover {
    background: var(--hover);
    color: var(--text);
  }
</style>
