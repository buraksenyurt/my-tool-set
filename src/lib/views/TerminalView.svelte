<script lang="ts">
  import { resizeTerminal } from "../api";
  import { terminals, type TerminalSession } from "../terminals.svelte";
  import { theme } from "../theme.svelte";

  let { session, visible }: { session: TerminalSession; visible: boolean } = $props();

  let host: HTMLDivElement | undefined = $state();

  const LIGHT = {
    background: "#fcfcfb",
    foreground: "#1d1d1b",
    cursor: "#2a78d6",
    selectionBackground: "#cfe0f6",
    black: "#1d1d1b",
    red: "#c43a3a",
    green: "#1b7a4e",
    yellow: "#9a5b00",
    blue: "#2a78d6",
    magenta: "#a1408f",
    cyan: "#137c8a",
    white: "#c8c7c0",
    brightBlack: "#6b6a65",
  };
  const DARK = {
    background: "#141413",
    foreground: "#e8e7e1",
    cursor: "#3987e5",
    selectionBackground: "#24415f",
    black: "#262624",
    red: "#e66767",
    green: "#4cc38a",
    yellow: "#e5a43a",
    blue: "#5a9cf0",
    magenta: "#d184c3",
    cyan: "#4fb8c4",
    white: "#e8e7e1",
    brightBlack: "#898781",
  };

  function isDark(): boolean {
    if (theme.current !== "system") return theme.current === "dark";
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  }

  // Follow the shell's theme, including the OS switching while "system".
  $effect(() => {
    void theme.current;
    const apply = () => (session.term.options.theme = isDark() ? DARK : LIGHT);
    apply();
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  });

  function fit() {
    if (!host || !visible || host.clientWidth === 0) return;
    const before = `${session.term.cols}x${session.term.rows}`;
    session.fit.fit();
    const after = `${session.term.cols}x${session.term.rows}`;
    if (before !== after && session.sessionId != null && session.status === "running") {
      resizeTerminal(session.sessionId, session.term.cols, session.term.rows).catch(() => {});
    }
  }

  // Attach xterm once; it stays attached while the tab is hidden.
  $effect(() => {
    if (!host || session.opened) return;
    session.term.open(host);
    session.opened = true;
  });

  $effect(() => {
    if (!host) return;
    const observer = new ResizeObserver(() => fit());
    observer.observe(host);
    return () => observer.disconnect();
  });

  // Becoming visible: size to the panel and take the keyboard.
  $effect(() => {
    if (!visible) return;
    requestAnimationFrame(() => {
      fit();
      session.term.focus();
    });
  });

  // A session that just started was sized before the panel existed.
  $effect(() => {
    if (session.status === "running") requestAnimationFrame(fit);
  });
</script>

<section class="terminal" class:hidden={!visible} aria-label="{session.toolName} terminali">
  <header>
    <strong>{session.toolName}</strong>
    <code class="cmd" title={session.cwd}>{session.commandLine || "başlatılıyor…"}</code>
    <span class="status {session.status}">
      {#if session.status === "running"}
        <span class="dot"></span>Çalışıyor
      {:else if session.status === "starting"}
        Başlatılıyor…
      {:else if session.status === "exited"}
        Bitti · çıkış kodu {session.code ?? "?"}
      {:else}
        Başlatılamadı
      {/if}
    </span>
    {#if session.status === "running"}
      <button onclick={() => terminals.stop(session.tabId)}>Durdur</button>
    {:else if session.status === "exited" || session.status === "failed"}
      <button class="primary" onclick={() => terminals.rerun(session.tabId)}>Yeniden çalıştır</button>
    {/if}
  </header>
  <div class="screen" bind:this={host}></div>
</section>

<style>
  .terminal {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr);
    height: 100%;
    min-height: 0;
  }

  .terminal.hidden {
    display: none;
  }

  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 14px;
    border-bottom: 1px solid var(--border);
    background: var(--sidebar);
    font-size: 13px;
  }

  .cmd {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-secondary);
  }

  .status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--muted);
    white-space: nowrap;
  }

  .status.running {
    color: var(--ok);
  }

  .status.failed {
    color: var(--danger);
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--ok);
  }

  button {
    height: 26px;
    padding: 0 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    cursor: pointer;
    font-size: 12.5px;
  }

  button.primary {
    border-color: var(--accent);
    background: var(--accent);
    color: #fff;
  }

  .screen {
    min-height: 0;
    padding: 6px 0 0 10px;
    background: var(--surface);
  }

  .screen :global(.xterm) {
    height: 100%;
  }
</style>
