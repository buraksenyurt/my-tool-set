// Terminal sessions shown in tabs. The xterm.js instance lives here, not in a
// component, so switching tabs never loses the screen, and output that
// arrives while a tab is hidden is still drawn.

import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import { tick } from "svelte";
import { closeTerminal, onTerminalExit, openTerminal, writeTerminal } from "./api";
import { tabs } from "./tabs.svelte";

export type TerminalStatus = "starting" | "running" | "exited" | "failed";

export interface TerminalSession {
  tabId: string;
  toolId: string;
  toolName: string;
  values: Record<string, string>;
  extra: string;
  sessionId: number | null;
  commandLine: string;
  cwd: string;
  status: TerminalStatus;
  code: number | null;
  error: string | null;
  term: Terminal;
  fit: FitAddon;
  /** Set once xterm has been attached to the DOM. */
  opened: boolean;
}

class TerminalsState {
  sessions = $state<Record<string, TerminalSession>>({});
  private counter = 0;
  private started = false;

  async init() {
    if (this.started) return;
    this.started = true;
    tabs.onClose((tab) => {
      if (tab.kind === "terminal") this.dispose(tab.id);
    });
    await onTerminalExit(({ sessionId, code }) => {
      const s = Object.values(this.sessions).find((x) => x.sessionId === sessionId);
      if (!s) return;
      s.status = "exited";
      s.code = code;
      s.term.write(`\r\n\x1b[2m[işlem bitti · çıkış kodu ${code ?? "?"}]\x1b[0m\r\n`);
    });
  }

  running(toolId: string): number {
    return Object.values(this.sessions).filter((s) => s.toolId === toolId && s.status === "running").length;
  }

  /** Opens a new tab running the tool with these form values. */
  async open(toolId: string, toolName: string, values: Record<string, string>, extra: string) {
    const tabId = `term:${++this.counter}`;
    const term = new Terminal({
      cursorBlink: true,
      convertEol: false,
      fontFamily: '"Cascadia Code", "JetBrains Mono", ui-monospace, monospace',
      fontSize: 13,
      scrollback: 5000,
      allowProposedApi: false,
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    const session: TerminalSession = {
      tabId,
      toolId,
      toolName,
      values,
      extra,
      sessionId: null,
      commandLine: "",
      cwd: "",
      status: "starting",
      code: null,
      error: null,
      term,
      fit,
      opened: false,
    };
    this.sessions[tabId] = session;
    term.onData((data) => {
      const s = this.sessions[tabId];
      if (s?.sessionId != null && s.status === "running") writeTerminal(s.sessionId, data).catch(() => {});
    });
    const same = Object.values(this.sessions).filter((s) => s.toolId === toolId).length;
    tabs.open({
      id: tabId,
      kind: "terminal",
      title: same > 1 ? `${toolName} ⌨ ${same}` : `${toolName} ⌨`,
      toolId,
    });
    // Let the tab mount and size itself first: the program should start with
    // the real column count (GitDash draws to the terminal width).
    await tick();
    await new Promise((resolve) => requestAnimationFrame(() => resolve(null)));
    if (session.opened) {
      try {
        fit.fit();
      } catch {
        /* not visible yet */
      }
    }
    await this.start(tabId);
  }

  /** Runs the same command again in the same tab. */
  async rerun(tabId: string) {
    const s = this.sessions[tabId];
    if (!s || s.status === "running" || s.status === "starting") return;
    s.term.write("\r\n");
    await this.start(tabId);
  }

  async stop(tabId: string) {
    const s = this.sessions[tabId];
    if (s?.sessionId != null && s.status === "running") await closeTerminal(s.sessionId);
  }

  private async start(tabId: string) {
    const s = this.sessions[tabId];
    s.status = "starting";
    s.error = null;
    s.code = null;
    try {
      const dto = await openTerminal(s.toolId, s.values, s.extra, s.term.cols, s.term.rows, (bytes) =>
        this.sessions[tabId]?.term.write(bytes),
      );
      // The tab may have been closed while starting.
      if (!this.sessions[tabId]) {
        closeTerminal(dto.sessionId);
        return;
      }
      s.sessionId = dto.sessionId;
      s.commandLine = dto.commandLine;
      s.cwd = dto.cwd;
      if (s.status === "starting") s.status = "running";
    } catch (e) {
      s.status = "failed";
      s.error = (e as Error).message;
      s.term.write(`\x1b[31m${s.error}\x1b[0m\r\n`);
    }
  }

  private dispose(tabId: string) {
    const s = this.sessions[tabId];
    if (!s) return;
    if (s.sessionId != null && s.status === "running") closeTerminal(s.sessionId).catch(() => {});
    s.term.dispose();
    delete this.sessions[tabId];
  }
}

export const terminals = new TerminalsState();
