<script lang="ts">
  import { clock } from "../clock.svelte";
  import { describeEnd, formatDuration, runs } from "../runs.svelte";

  let { toolId }: { toolId: string } = $props();

  $effect(() => clock.use());

  const toolRuns = $derived(runs.forTool(toolId));
  let selectedId = $state<number | null>(null);
  // Follow the newest run unless the user picked an older one.
  const run = $derived(toolRuns.find((r) => r.id === selectedId) ?? toolRuns[0]);
  const lines = $derived(run ? (runs.logs[run.id] ?? []) : []);

  let box: HTMLElement | undefined = $state();
  let follow = $state(true);
  let copied = $state(false);

  $effect(() => {
    if (run) runs.ensureLog(run.id);
  });

  // Stay at the bottom while new lines arrive, unless the user scrolled up.
  $effect(() => {
    void lines.length;
    if (follow && box) box.scrollTop = box.scrollHeight;
  });

  function onScroll() {
    if (!box) return;
    follow = box.scrollHeight - box.scrollTop - box.clientHeight < 24;
  }

  async function copy() {
    const text = lines.map((l) => l.text).join("\n");
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {
      /* clipboard blocked */
    }
  }

  function startedLabel(startedAt: number): string {
    return new Date(startedAt).toLocaleTimeString("tr-TR", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
  }
</script>

{#if !run}
  <p class="empty">Bu oturumda henüz bir çalışma yok. Başlattığın ya da derlediğin anda çıktısı burada akar.</p>
{:else}
  <div class="head">
    {#if toolRuns.length > 1}
      <select
        aria-label="Çalışma seç"
        value={run.id}
        onchange={(e) => {
          const id = Number((e.target as HTMLSelectElement).value);
          selectedId = id === toolRuns[0].id ? null : id;
          follow = true;
        }}
      >
        {#each toolRuns as r (r.id)}
          <option value={r.id}>
            {r.kind === "build" ? "Derleme" : "Çalışma"} · {startedLabel(r.startedAt)}{r.state.status === "running" ? " · çalışıyor" : ""}
          </option>
        {/each}
      </select>
    {/if}
    <span class="status" class:live={run.state.status === "running"}>
      {describeEnd(run)} · {formatDuration(run.startedAt, run.endedAt ?? clock.now)}
    </span>
    <span class="spacer"></span>
    <button onclick={copy} disabled={lines.length === 0}>{copied ? "Kopyalandı" : "Kopyala"}</button>
  </div>
  <p class="cmd"><code>{run.command}</code> <span class="muted">· {run.cwd}</span></p>

  <div class="log" bind:this={box} onscroll={onScroll} role="log" aria-live="polite">
    {#each lines as line, i (i)}
      <div class="line" class:err={line.stream === "stderr"}>{line.text || " "}</div>
    {:else}
      <div class="muted">{run.state.status === "running" ? "Çıktı bekleniyor…" : "Bu çalışma hiç çıktı üretmedi."}</div>
    {/each}
  </div>
  {#if !follow && run.state.status === "running"}
    <button class="jump" onclick={() => { follow = true; if (box) box.scrollTop = box.scrollHeight; }}>↓ En alta git</button>
  {/if}
{/if}

<style>
  .empty {
    padding: 8px 12px;
    border-radius: var(--radius);
    background: var(--hover);
    color: var(--text-secondary);
  }

  .head {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 6px;
  }

  .spacer {
    flex: 1;
  }

  select,
  button {
    height: 28px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .status {
    color: var(--text-secondary);
    font-size: 13px;
    font-variant-numeric: tabular-nums;
  }

  .status.live {
    color: var(--ok);
  }

  .cmd {
    margin: 0 0 8px;
    font-size: 12px;
    overflow-wrap: anywhere;
  }

  .muted {
    color: var(--muted);
  }

  .log {
    height: min(60vh, 520px);
    padding: 10px 12px;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    font-family: "Cascadia Code", "JetBrains Mono", ui-monospace, monospace;
    font-size: 12.5px;
    line-height: 1.5;
  }

  .line {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .line.err {
    color: var(--danger);
  }

  .jump {
    position: relative;
    top: -44px;
    float: right;
    margin-right: 14px;
  }
</style>
