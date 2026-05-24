<script lang="ts">
  import type { WorkspaceDocumentSummary, WorkspaceRecord } from "../../lib/api";
  import { buildWorkspaceTree } from "../../lib/utils/workspace";

  export let workspace: WorkspaceRecord | null = null;
  export let documents: WorkspaceDocumentSummary[] = [];
  export let activeDocumentId: string | null = null;
  export let onSelectDocument: (documentId: string) => void;
  export let onSelectWorkspace: (workspace: WorkspaceRecord) => void;
</script>

<article class="panel-card ide-panel">
  <header class="panel-head">
    <strong>Explorer</strong>
    <span>workspace tree</span>
  </header>

  {#if workspace}
    <div class="detail-stack">
      <div class="detail-row"><span>Workspace</span><strong>{workspace.name}</strong></div>
      <div class="detail-row"><span>Language</span><strong>{workspace.language}</strong></div>
      <div class="detail-row"><span>Documents</span><strong>{documents.length}</strong></div>
    </div>
    <div class="tree-list">
      {#each buildWorkspaceTree(documents) as entry}
        {#if entry.type === "folder"}
          <div class="tree-folder">
            <strong>{entry.name}</strong>
            {#if entry.children}
              <div class="tree-list">
                {#each entry.children as child}
                  <button
                    type="button"
                    class:active={activeDocumentId === child.documentId}
                    class="tree-item"
                    on:click={() => child.documentId && onSelectDocument(child.documentId)}
                  >
                    {child.name}
                    <span class="nav-link-subtitle">{child.path}</span>
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {:else}
          <button
            type="button"
            class:active={activeDocumentId === entry.documentId}
            class="tree-item"
            on:click={() => entry.documentId && onSelectDocument(entry.documentId)}
          >
            {entry.name}
            <span class="nav-link-subtitle">{entry.path}</span>
          </button>
        {/if}
      {/each}
    </div>
  {:else}
    <div class="empty-copy">Select a workspace to inspect its document tree.</div>
  {/if}
</article>
