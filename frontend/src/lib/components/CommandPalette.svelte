<script lang="ts">
  import { tick } from "svelte";
  import { commandPaletteOpen, commandQuery, closeCommandPalette } from "../stores/ui";
  import { navigateApp, navigateDriveLens, type AppId } from "../router";
  import { runSearch, type SearchResult } from "../api";

  let queryInput: HTMLInputElement | null = null;
  let searchResults: SearchResult[] = [];
  let searching = false;

  const actions: Array<{ title: string; detail: string; run: () => void }> = [
    { title: "Cloud Drive", detail: "Browse files and folders", run: () => navigateApp("drive") },
    { title: "Knowledge", detail: "Notes and capture", run: () => navigateApp("knowledge") },
    { title: "Workspace", detail: "IDE and projects", run: () => navigateApp("workspace") },
    { title: "Operations", detail: "Admin console", run: () => navigateApp("operations") },
    { title: "Upload files", detail: "Open drive", run: () => navigateApp("drive") },
  ];

  async function runQuery(q: string) {
    if (q.trim().length < 2) {
      searchResults = [];
      return;
    }
    searching = true;
    try {
      const res = await runSearch(q.trim(), { limit: 12 });
      searchResults = res.results || [];
    } catch {
      searchResults = [];
    } finally {
      searching = false;
    }
  }

  function activate(run: () => void) {
    run();
    closeCommandPalette();
  }

  $: filteredActions = actions.filter((item) => {
    const q = $commandQuery.trim().toLowerCase();
    if (!q) return true;
    return `${item.title} ${item.detail}`.toLowerCase().includes(q);
  });

  $: if ($commandPaletteOpen) {
    tick().then(() => queryInput?.focus());
  }

  $: if ($commandPaletteOpen) void runQuery($commandQuery);
</script>

{#if $commandPaletteOpen}
  <div class="command-overlay" role="presentation" on:click={closeCommandPalette}>
    <section class="command-surface" role="dialog" aria-modal="true" on:click|stopPropagation>
      <div class="command-header">
        <input
          bind:this={queryInput}
          bind:value={$commandQuery}
          class="command-input"
          type="search"
          placeholder="Search files, notes, workspaces, commands…"
        />
        <button type="button" class="btn btn-ghost" on:click={closeCommandPalette}>Esc</button>
      </div>
      <div class="command-list">
        {#if searching}
          <div class="command-item muted">Searching…</div>
        {/if}
        {#each searchResults as hit}
          <button
            type="button"
            class="command-item"
            on:click={() =>
              activate(() => {
                if (hit.type === "file") navigateApp("drive");
                else if (hit.type === "note") navigateApp("knowledge");
                else navigateApp("workspace");
              })}
          >
            <strong>{hit.title || hit.name || hit.id}</strong>
            <span>{hit.type} · {hit.snippet || hit.folder_path || ""}</span>
          </button>
        {/each}
        {#each filteredActions as item}
          <button type="button" class="command-item" on:click={() => activate(item.run)}>
            <strong>{item.title}</strong>
            <span>{item.detail}</span>
          </button>
        {/each}
      </div>
    </section>
  </div>
{/if}

<style>
  .command-overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0, 0, 0, 0.55);
    display: grid;
    place-items: start center;
    padding-top: 12vh;
  }
  .command-surface {
    width: min(640px, calc(100vw - 32px));
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.45);
  }
  .command-header {
    display: flex;
    gap: 8px;
    padding: 12px;
    border-bottom: 1px solid var(--border);
  }
  .command-input {
    flex: 1;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 10px 12px;
    background: var(--bg-card);
  }
  .command-list {
    max-height: 360px;
    overflow: auto;
    padding: 8px;
    display: grid;
    gap: 4px;
  }
  .command-item {
    border: 1px solid transparent;
    background: transparent;
    text-align: left;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    display: grid;
    gap: 2px;
  }
  .command-item:hover {
    background: var(--bg-hover);
    border-color: var(--border);
  }
  .command-item strong {
    font-size: 13px;
  }
  .command-item span {
    font-size: 12px;
    color: var(--text-muted);
  }
  .muted {
    color: var(--text-dim);
    padding: 10px 12px;
  }
</style>
