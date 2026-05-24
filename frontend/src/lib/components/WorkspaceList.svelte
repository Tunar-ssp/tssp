<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { Workspace } from '$lib/api';
  import { activeWorkspaceId, createNewWorkspace } from '$lib/stores/workspace';

  export let workspaces: Workspace[] = [];
  export let onSelectWorkspace: (id: string) => void = () => {};

  function getLanguageIcon(lang: string): any {
    const icons: Record<string, any> = {
      js: Icons.Zap,
      ts: Icons.Zap,
      py: Icons.Code2,
      rs: Icons.Code2,
      go: Icons.Code2,
      java: Icons.Coffee,
      c: Icons.Code2,
      cpp: Icons.Code2,
      html: Icons.Code,
      css: Icons.Palette,
      json: Icons.Code2,
      yaml: Icons.Code2,
      md: Icons.FileText,
    };
    return icons[lang] || Icons.File;
  }

  async function handleNewWorkspace() {
    await createNewWorkspace();
  }
</script>

<div class="workspace-list-container">
  <div class="list-header">
    <h3>Workspaces</h3>
    <button class="new-btn" on:click={handleNewWorkspace}>
      <Icons.Plus size={16} />
    </button>
  </div>

  {#if workspaces.length === 0}
    <div class="empty-list">
      <Icons.Code2 size={32} />
      <p>No workspaces</p>
      <p class="secondary">Create a new workspace to start coding</p>
    </div>
  {:else}
    <div class="workspace-list">
      {#each workspaces as ws (ws.id)}
        <button
          class="ws-item"
          class:active={$activeWorkspaceId === ws.id}
          on:click={() => {
            $activeWorkspaceId = ws.id;
            onSelectWorkspace(ws.id);
          }}
        >
          <div class="item-icon">
            <svelte:component this={getLanguageIcon(ws.language)} size={16} />
          </div>
          <div class="item-info">
            <div class="item-name">{ws.name || 'untitled'}</div>
            <div class="item-lang">{ws.language}</div>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .workspace-list-container {
    width: 240px;
    height: 100%;
    border-right: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .list-header {
    padding: 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--border);
  }

  .list-header h3 {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
  }

  .new-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border-radius: var(--r-2);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .new-btn:hover {
    background: var(--blue);
    border-color: var(--blue);
    color: #0a1228;
  }

  .empty-list {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--muted);
    padding: 20px;
    text-align: center;
  }

  .empty-list p {
    margin: 0;
  }

  .empty-list .secondary {
    font-size: var(--fs-12);
    color: var(--dim);
  }

  .workspace-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .ws-item {
    padding: 10px 12px;
    border: none;
    border-bottom: 1px solid var(--hairline);
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: all 0.15s;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .ws-item:hover {
    background: var(--surface-2);
  }

  .ws-item.active {
    background: var(--surface-hi);
    border-left: 2px solid var(--blue);
    padding-left: 10px;
  }

  .item-icon {
    flex-shrink: 0;
    color: var(--text-2);
  }

  .item-info {
    flex: 1;
    min-width: 0;
  }

  .item-name {
    font-size: var(--fs-12);
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-lang {
    font-size: 10px;
    color: var(--muted);
  }
</style>
