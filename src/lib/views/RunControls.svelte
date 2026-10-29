<script lang="ts">
  import type { ToolDto } from "../bindings/ToolDto";
  import { clock } from "../clock.svelte";
  import { formatAgo } from "../format";
  import { describeEnd, formatDuration, runs } from "../runs.svelte";
  import RunForm from "./RunForm.svelte";

  let { tool, onShowLogs }: { tool: ToolDto; onShowLogs: () => void } = $props();

  $effect(() => clock.use());

  const running = $derived(runs.active(tool.id, "tool"));
  const building = $derived(runs.active(tool.id, "build"));
  const last = $derived(runs.forTool(tool.id)[0]);
  const error = $derived(runs.errors[tool.id]);

  const canStart = $derived.by(() => {
    if (tool.host !== "external") return false;
    const d = tool.delivery;
    if (d.type === "release") return d.installed.length > 0;
    return d.status.kind === "ready" || d.status.kind === "fromSource";
  });

  const canBuild = $derived(
    tool.delivery.type === "local" &&
      !!tool.delivery.buildCommand &&
      ["ready", "notBuilt", "fromSource"].includes(tool.delivery.status.kind),
  );
  const notBuilt = $derived(tool.delivery.type === "local" && tool.delivery.status.kind === "notBuilt");

  /** Terminal tools that have something to run (installed / built / from source). */
  const canRunInTerminal = $derived.by(() => {
    if (tool.host !== "terminal") return false;
    const d = tool.delivery;
    if (d.type === "release") return d.installed.length > 0;
    return d.status.kind === "ready" || d.status.kind === "fromSource";
  });
</script>

<section class="run">
  {#if running || canStart || canBuild || (tool.host === "terminal" && !canRunInTerminal) || (tool.host !== "terminal" && tool.delivery.type === "release")}
  <div class="row">
    {#if running}
      <button class="stop" onclick={() => runs.stop(running)}>■ Durdur</button>
      <span class="live">
        <span class="dot" aria-hidden="true"></span>
        Çalışıyor · {formatDuration(running.startedAt, clock.now)}{#if running.pid}&nbsp;· PID {running.pid}{/if}
      </span>
    {:else if canStart}
      <button class="primary" onclick={() => runs.start(tool.id)}>▶ Başlat</button>
    {:else if tool.host === "terminal" && !canRunInTerminal}
      <span class="hint">
        {notBuilt ? "Terminalde çalıştırmak için önce derle." : "Terminalde çalıştırmak için önce yerel klasörü ayarla."}
      </span>
    {:else if tool.delivery.type === "release"}
      <span class="hint">Başlatmak için önce kur.</span>
    {/if}

    {#if canBuild}
      {#if building}
        <button class="ghost" onclick={() => runs.stop(building)}>Derlemeyi durdur</button>
        <span class="live">
          <span class="spinner" aria-hidden="true"></span>
          Derleniyor · {formatDuration(building.startedAt, clock.now)}
        </span>
      {:else}
        <button class={notBuilt ? "primary" : "ghost"} onclick={() => runs.build(tool.id)}>
          {notBuilt ? "Derle" : "Yeniden derle"}
        </button>
      {/if}
    {/if}
  </div>
  {/if}

  {#if canRunInTerminal}
    <RunForm {tool} />
  {/if}

  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}

  {#if last && last.state.status !== "running"}
    <p class="last">
      Son {last.kind === "build" ? "derleme" : "çalışma"}:
      <span class:bad={last.state.status === "failed" || (last.state.status === "exited" && !last.state.stopped && last.state.code !== 0)}>
        {describeEnd(last)}
      </span>
      {#if last.endedAt}<span class="muted">· {formatAgo(last.endedAt / 1000, clock.now / 1000)}</span>{/if}
      <button class="link" onclick={onShowLogs}>Loglar →</button>
    </p>
  {:else if running || building}
    <p class="last"><button class="link" onclick={onShowLogs}>Canlı logu izle →</button></p>
  {/if}
</section>

<style>
  .run {
    margin: 22px 0 0;
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
  }

  button {
    height: 32px;
    padding: 0 16px;
    border-radius: var(--radius);
    cursor: pointer;
    font-weight: 600;
  }

  .primary {
    border: 1px solid var(--accent);
    background: var(--accent);
    color: #fff;
  }

  .primary:hover {
    filter: brightness(1.08);
  }

  .stop {
    border: 1px solid var(--danger);
    background: var(--surface);
    color: var(--danger);
  }

  .stop:hover {
    background: var(--danger);
    color: #fff;
  }

  .ghost {
    border: 1px solid var(--border);
    background: var(--surface);
    font-weight: 400;
  }

  .ghost:hover {
    border-color: var(--text-secondary);
  }

  .live {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--ok);
    box-shadow: 0 0 0 3px var(--ok-soft);
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid var(--accent);
    border-right-color: transparent;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .hint {
    color: var(--muted);
  }

  .last {
    margin: 10px 0 0;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .bad {
    color: var(--danger);
  }

  .muted {
    color: var(--muted);
  }

  .error {
    margin: 10px 0 0;
    color: var(--danger);
  }

  button.link {
    height: auto;
    padding: 0;
    margin-left: 6px;
    border: 0;
    background: none;
    color: var(--accent);
    font-weight: 400;
  }
</style>
