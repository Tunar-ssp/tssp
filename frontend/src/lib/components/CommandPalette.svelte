<script lang="ts">
  import { tick } from "svelte";
  import { commandPaletteOpen, commandQuery, closeCommandPalette } from "../stores/ui";
  import { navigate, type RouteId } from "../router";

  let queryInput: HTMLInputElement | null = null;

  const actions: Array<{ title: string; detail: string; route: RouteId }> = [
    { title: "Open Cloud Drive", detail: "Files, folders, uploads, previews", route: "drive" },
    { title: "Open Notes", detail: "Pages, quick capture, writing", route: "notes" },
    { title: "Open Workspace", detail: "Project explorer and code view", route: "workspace" },
    { title: "Open Sharing Center", detail: "Public links and QR access", route: "sharing" },
    { title: "Open Operations", detail: "Users, storage, diagnostics", route: "operations" },
  ];

  function activate(route: RouteId) {
    navigate(route);
    closeCommandPalette();
  }

  $: filtered = actions.filter((item) => {
    const query = $commandQuery.trim().toLowerCase();
    if (!query) return true;
    return `${item.title} ${item.detail}`.toLowerCase().includes(query);
  });

  $: if ($commandPaletteOpen) {
    tick().then(() => queryInput?.focus());
  }
</script>

{#if $commandPaletteOpen}
  <div class="command-overlay" role="presentation" on:click={closeCommandPalette}>
    <section
      class="command-surface"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      on:click|stopPropagation
      on:keydown|stopPropagation
    >
      <div class="command-header">
        <input
          bind:value={$commandQuery}
          bind:this={queryInput}
          class="command-input"
          type="search"
          placeholder="Search files, notes, workspaces, commands"
        />
        <button type="button" class="btn btn-text" on:click={closeCommandPalette}>Close</button>
      </div>

      <div class="command-list">
        {#each filtered as item}
          <button type="button" class="command-item" on:click={() => activate(item.route)}>
            <strong>{item.title}</strong>
            <span>{item.detail}</span>
          </button>
        {/each}
      </div>
    </section>
  </div>
{/if}
