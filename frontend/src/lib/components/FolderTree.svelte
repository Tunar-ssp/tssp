<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { currentFolder } from '$lib/stores/drive';

  export let folders: string[] = [];
  export let onSelectFolder: (path: string) => void = () => {};

  function selectFolder(path: string) {
    $currentFolder = path;
    onSelectFolder(path);
  }
</script>

<div class="folder-tree">
  <div class="tree-section">
    <button
      class="folder-item root"
      class:active={$currentFolder === ''}
      on:click={() => selectFolder('')}
    >
      <Icons.HardDrive size={16} />
      <span>My Drive</span>
    </button>

    {#if folders.length > 0}
      <div class="divider" />
      <div class="folder-list">
        {#each folders as folder (folder)}
          <button
            class="folder-item"
            class:active={$currentFolder === folder}
            on:click={() => selectFolder(folder)}
          >
            <Icons.Folder size={16} />
            <span class="folder-name">{folder || 'Root'}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <div class="tree-section bottom">
    <button class="folder-item">
      <Icons.Trash2 size={16} />
      <span>Trash</span>
    </button>
  </div>
</div>

<style>
  .folder-tree {
    width: 220px;
    height: 100%;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border);
    background: var(--surface);
    overflow: hidden;
  }

  .tree-section {
    padding: 12px 8px;
    overflow-y: auto;
    flex: 1;
  }

  .tree-section.bottom {
    flex: 0;
    border-top: 1px solid var(--border);
  }

  .divider {
    height: 1px;
    background: var(--hairline);
    margin: 8px 0;
  }

  .folder-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .folder-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 10px;
    border-radius: var(--r-2);
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-2);
    font-size: var(--fs-13);
    cursor: pointer;
    transition: all 0.15s;
  }

  .folder-item:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .folder-item.active {
    background: var(--surface-2);
    border-color: var(--border);
    color: var(--text);
  }

  .folder-item.root {
    font-weight: 500;
    color: var(--text);
  }

  .folder-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }
</style>
