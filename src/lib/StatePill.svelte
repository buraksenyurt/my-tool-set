<script lang="ts">
  import { STATE_LABELS, type ToolState } from "./catalog.svelte";

  let { state, version }: { state: ToolState; version?: string } = $props();

  const tone = $derived(
    (
      {
        installed: "ok",
        localReady: "ok",
        update: "accent",
        notInstalled: "neutral",
        noRelease: "muted",
        noBuild: "muted",
        localNotBuilt: "warn",
        localSetup: "warn",
      } as const
    )[state],
  );
</script>

<span class="pill {tone}">{STATE_LABELS[state]}{#if version}&nbsp;{version}{/if}</span>

<style>
  .pill {
    display: inline-block;
    padding: 1px 7px;
    border-radius: 999px;
    font-size: 11px;
    line-height: 16px;
    white-space: nowrap;
    border: 1px solid transparent;
  }

  .ok {
    background: var(--ok-soft);
    color: var(--ok);
  }

  .accent {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .warn {
    background: var(--warn-soft);
    color: var(--warn);
  }

  .neutral {
    border-color: var(--border);
    color: var(--text-secondary);
  }

  .muted {
    color: var(--muted);
    border-color: var(--border);
    border-style: dashed;
  }
</style>
