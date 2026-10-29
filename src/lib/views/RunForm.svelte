<script lang="ts">
  // The run form for terminal tools: one field per run.params entry, plus
  // free-form extra arguments. Last values are remembered per tool.
  import { untrack } from "svelte";
  import { pickPath } from "../api";
  import type { ToolDto } from "../bindings/ToolDto";
  import { terminals } from "../terminals.svelte";

  let { tool }: { tool: ToolDto } = $props();

  // The tool tab is re-created per tool ({#key}), so the form is set up once
  // from the tool it was opened for.
  const initial = untrack(() => tool);
  const KEY = `my-tool-set.run-form.${initial.id}`;

  function load(): { values: Record<string, string>; extra: string } {
    try {
      const raw = localStorage.getItem(KEY);
      if (raw) return JSON.parse(raw);
    } catch {
      /* ignore */
    }
    return { values: {}, extra: "" };
  }

  const saved = load();
  let values = $state<Record<string, string>>(
    Object.fromEntries(
      initial.params.map((p) => [p.name, saved.values[p.name] ?? (p.default != null ? String(p.default) : "")]),
    ),
  );
  let extra = $state(saved.extra ?? "");
  let error = $state<string | null>(null);

  // Number inputs bind numbers; Rust takes every value as text.
  const text = (v: unknown): string => (v == null ? "" : String(v));
  const missing = $derived(tool.params.filter((p) => p.default == null && !text(values[p.name]).trim()));

  async function browse(name: string, kind: "folder" | "file") {
    const picked = await pickPath(kind, values[name]);
    if (picked) values[name] = picked;
  }

  async function run(e: SubmitEvent) {
    e.preventDefault();
    error = null;
    if (missing.length > 0) {
      error = `"${missing[0].label}" gerekli`;
      return;
    }
    const clean = Object.fromEntries(Object.entries(values).map(([k, v]) => [k, text(v)]));
    try {
      localStorage.setItem(KEY, JSON.stringify({ values: clean, extra }));
    } catch {
      /* ignore */
    }
    await terminals.open(tool.id, tool.name, clean, text(extra));
  }
</script>

<form class="form" onsubmit={run}>
  {#each tool.params as param (param.name)}
    <label>
      <span class="label">
        {param.label}
        {#if param.flag}<code>{param.flag}</code>{/if}
        {#if param.default == null}<span class="req" aria-label="gerekli">*</span>{/if}
      </span>
      <span class="field">
        <input
          type={param.kind === "number" ? "number" : "text"}
          bind:value={values[param.name]}
          placeholder={param.default != null ? String(param.default) : ""}
          spellcheck="false"
        />
        {#if param.kind === "folder" || param.kind === "file"}
          <button type="button" class="ghost" onclick={() => browse(param.name, param.kind as "folder" | "file")}>
            Seç…
          </button>
        {/if}
      </span>
    </label>
  {/each}
  <label>
    <span class="label">Ek argümanlar <span class="muted">(isteğe bağlı)</span></span>
    <span class="field">
      <input type="text" bind:value={extra} placeholder={'örn. --verbose "boşluklu değer"'} spellcheck="false" />
    </span>
  </label>
  <div class="actions">
    <button type="submit" class="primary">⌨ Terminalde çalıştır</button>
    {#if terminals.running(tool.id) > 0}
      <span class="muted">{terminals.running(tool.id)} terminal açık</span>
    {/if}
  </div>
  {#if error}<p class="error" role="alert">{error}</p>{/if}
</form>

<style>
  .form {
    display: grid;
    gap: 12px;
    max-width: 620px;
    margin-top: 22px;
  }

  label {
    display: grid;
    gap: 5px;
  }

  .label {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .label code {
    margin-left: 4px;
    font-size: 11.5px;
  }

  .req {
    color: var(--danger);
  }

  .field {
    display: flex;
    gap: 8px;
  }

  input {
    flex: 1;
    height: 32px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    color: var(--text);
    font: inherit;
  }

  input:focus {
    outline: none;
    border-color: var(--accent);
  }

  button {
    height: 32px;
    padding: 0 14px;
    border-radius: var(--radius);
    cursor: pointer;
  }

  .ghost {
    border: 1px solid var(--border);
    background: var(--surface);
  }

  .primary {
    border: 1px solid var(--accent);
    background: var(--accent);
    color: #fff;
    font-weight: 600;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .muted {
    color: var(--muted);
    font-size: 13px;
  }

  .error {
    margin: 0;
    color: var(--danger);
  }
</style>
