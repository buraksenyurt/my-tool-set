// Runs of tools and builds as the UI sees them, kept in step with Rust through
// the `run-changed` and `run-log` events.

import { buildTool, listRuns, onRunEvents, runLog, startTool, stopRun } from "./api";
import type { LogLineDto } from "./bindings/LogLineDto";
import type { RunDto } from "./bindings/RunDto";
import { catalog } from "./catalog.svelte";

const MAX_LINES = 5000;

class RunsState {
  /** Newest first. */
  list = $state<RunDto[]>([]);
  /** Lines per run id, filled from events and, when a panel opens, from Rust. */
  logs = $state<Record<number, LogLineDto[]>>({});
  /** Last start/stop error per tool id. */
  errors = $state<Record<string, string>>({});

  running = $derived(this.list.filter((r) => r.state.status === "running"));

  private started = false;

  async init() {
    if (this.started) return;
    this.started = true;
    await onRunEvents({
      changed: (run) => this.upsert(run),
      log: ({ runId, lines }) => {
        // Runs this window did not see start are fetched whole by ensureLog.
        if (!(runId in this.logs)) return;
        const next = this.logs[runId].concat(lines);
        this.logs[runId] = next.length > MAX_LINES ? next.slice(-MAX_LINES) : next;
      },
    });
    this.list = await listRuns();
  }

  forTool(toolId: string): RunDto[] {
    return this.list.filter((r) => r.toolId === toolId);
  }

  active(toolId: string, kind: "tool" | "build"): RunDto | undefined {
    return this.list.find((r) => r.toolId === toolId && r.kind === kind && r.state.status === "running");
  }

  /**
   * Runs started in this window get every line through events. For runs that
   * were already there (a reloaded window) the full log is fetched once.
   */
  async ensureLog(runId: number) {
    if (runId in this.logs) return;
    try {
      this.logs[runId] = await runLog(runId);
    } catch {
      this.logs[runId] = [];
    }
  }

  async start(toolId: string) {
    delete this.errors[toolId];
    try {
      this.upsert(await startTool(toolId));
    } catch (e) {
      this.errors[toolId] = (e as Error).message;
    }
  }

  async build(toolId: string) {
    delete this.errors[toolId];
    try {
      this.upsert(await buildTool(toolId));
    } catch (e) {
      this.errors[toolId] = (e as Error).message;
    }
  }

  async stop(run: RunDto) {
    try {
      await stopRun(run.id);
    } catch (e) {
      this.errors[run.toolId] = (e as Error).message;
    }
  }

  private upsert(run: RunDto) {
    const index = this.list.findIndex((r) => r.id === run.id);
    const wasRunning = index >= 0 && this.list[index].state.status === "running";
    if (index >= 0) this.list[index] = run;
    else this.list.unshift(run);
    if (!this.logs[run.id] && run.state.status === "running") this.logs[run.id] = [];
    // A finished build may have produced the executable: refresh local status.
    if (wasRunning && run.kind === "build" && run.state.status !== "running") {
      catalog.reload();
    }
  }
}

export const runs = new RunsState();

/** "2 dk 13 sn" style duration between two epoch-ms values. */
export function formatDuration(from: number, to: number): string {
  const s = Math.max(0, Math.round((to - from) / 1000));
  if (s < 60) return `${s} sn`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m} dk ${s % 60} sn`;
  return `${Math.floor(m / 60)} sa ${m % 60} dk`;
}

export function describeEnd(run: RunDto): string {
  switch (run.state.status) {
    case "running":
      return "Çalışıyor";
    case "failed":
      return run.state.message;
    case "exited":
      if (run.state.stopped) return "Durduruldu";
      if (run.state.code === 0) return "Başarıyla bitti (çıkış kodu 0)";
      if (run.state.code === null) return "Sinyal ile sonlandı";
      return `Hata ile bitti (çıkış kodu ${run.state.code})`;
  }
}
