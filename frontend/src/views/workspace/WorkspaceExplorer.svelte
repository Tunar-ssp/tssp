<script lang="ts">
  import type { WorkspaceDocumentSummary, WorkspaceRecord } from "../../lib/api";
  import { buildWorkspaceTree } from "../../lib/utils/workspace";

  export let workspaces: WorkspaceRecord[] = [];
  export let workspace: WorkspaceRecord | null = null;
  export let documents: WorkspaceDocumentSummary[] = [];
  export let activeDocumentId: string | null = null;
  export let onSelectDocument: (documentId: string) => void;
  export let onSelectWorkspace: (workspace: WorkspaceRecord) => void;
  export let onCreateProject: () => void;
  export let onCreateFile: () => void;
</script>

<aside class="explorer">
  <header class="explorer-head">
    <strong>Explorer</strong>
    <button type="button" class="btn btn-sm btn-ghost" on:click={onCreateProject} title="New project">+</button>
  </header>

  <div class="project-list">
    {#each workspaces as ws}
      <button
        type="button"
        class="project-btn"
        class:active={workspace?.id === ws.id}
        on:click={() => onSelectWorkspace(ws)}
      >
        {ws.name}
      </button>
    {/each}
  </div>

  {#if workspace}
    <div class="explorer-meta">
      <span>{workspace.language}</span>
      <button type="button" class="btn btn-sm btn-ghost" on:click={onCreateFile}>New file</button>
    </div>
    <div class="tree">
      {#each buildWorkspaceTree(documents) as entry}
        {#if entry.type === "folder"}
          <div class="folder">
            <span class="folder-name">{entry.name}/</span>
            {#if entry.children}
              {#each entry.children as child}
                <button
                  type="button"
                  class="file-btn"
                  class:active={activeDocumentId === child.documentId}
                  on:click={() => child.documentId && onSelectDocument(child.documentId)}
                >
                  {child.name}
                </button>
              {/each}
            {/if}
          </div>
        {:else}
          <button
            type="button"
            class="file-btn"
            class:active={activeDocumentId === entry.documentId}
            on:click={() => entry.documentId && onSelectDocument(entry.documentId)}
          >
            {entry.name}
          </button>
        {/if}
      {/each}
    </div>
  {:else}
    <p class="muted">Select a project</p>
  {/if}
</aside>

<style>
  .explorer {
    border-right: 1px solid var(--border);
    background: var(--bg-elevated);
    padding: 12px;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .explorer-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted);
  }
  .project-list { display: grid; gap: 4px; }
  .project-btn {
    text-align: left;
    border: 1px solid transparent;
    background: transparent;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 600;
  }
  .project-btn.active { background: var(--brand-dim); border-color: rgba(37,99,235,0.35); }
  .explorer-meta { display: flex; justify-content: space-between; align-items: center; font-size: 12px; color: var(--text-muted); }
  .tree { display: grid; gap: 2px; }
  .folder-name { display: block; font-size: 11px; color: var(--text-dim); padding: 6px 0 2px; }
  .file-btn {
    width: 100%;
    text-align: left;
    border: none;
    background: transparent;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-family: var(--font-mono);
  }
  .file-btn:hover, .file-btn.active { background: var(--bg-hover); }
  .muted { color: var(--text-muted); font-size: 12px; }
</style>
