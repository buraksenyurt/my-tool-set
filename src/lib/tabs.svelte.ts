// Open tabs in the workspace. The home tab is always there and cannot be
// closed; every other tab is opened at most once per id.

export type TabKind = "home" | "about" | "tool" | "issues" | "terminal";

export interface Tab {
  id: string;
  kind: TabKind;
  title: string;
  closable: boolean;
  /** For `tool` and `terminal` tabs. */
  toolId?: string;
}

type CloseHook = (tab: Tab) => void;

const HOME: Tab = { id: "home", kind: "home", title: "Ana sayfa", closable: false };

class TabsState {
  list = $state<Tab[]>([HOME]);
  activeId = $state<string>(HOME.id);

  active = $derived(this.list.find((t) => t.id === this.activeId) ?? HOME);

  private closeHooks: CloseHook[] = [];

  /** Lets other stores clean up when one of their tabs closes. */
  onClose(hook: CloseHook) {
    this.closeHooks.push(hook);
  }

  /** Opens a tab, or focuses it if it is already open. */
  open(tab: Omit<Tab, "closable"> & { closable?: boolean }) {
    if (!this.list.some((t) => t.id === tab.id)) {
      this.list.push({ closable: true, ...tab });
    }
    this.activeId = tab.id;
  }

  close(id: string) {
    const index = this.list.findIndex((t) => t.id === id);
    if (index < 0 || !this.list[index].closable) return;
    const [closed] = this.list.splice(index, 1);
    for (const hook of this.closeHooks) hook(closed);
    if (this.activeId === id) {
      // Focus the neighbour on the left, like browsers and editors do.
      this.activeId = this.list[Math.max(0, index - 1)].id;
    }
  }

  /** Moves focus by `step` tabs, wrapping around. */
  cycle(step: number) {
    const index = this.list.findIndex((t) => t.id === this.activeId);
    const next = (index + step + this.list.length) % this.list.length;
    this.activeId = this.list[next].id;
  }
}

export const tabs = new TabsState();

export const openAbout = () => tabs.open({ id: "about", kind: "about", title: "Hakkında" });

export const openIssues = () =>
  tabs.open({ id: "issues", kind: "issues", title: "Katalog sorunları" });

export const openTool = (id: string, name: string) =>
  tabs.open({ id: `tool:${id}`, kind: "tool", title: name, toolId: id });
