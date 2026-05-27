<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { FolderEntry } from '$lib/api';
  import { formatBytes } from '$lib/utils';

  interface Filter {
    id: string;
    label: string;
    icon: any;
  }

  interface Props {
    filters?: Filter[];
    activeLens?: string;
    folders?: FolderEntry[];
    currentFolder?: string;
    publicCount?: number;
    trashCount?: number;
    usedBytes?: number;
    totalObjects?: number;
    onLensChange?: (lens: any) => void;
    onFolderChange?: (path: string) => void;
  }

  let {
    filters = [],
    activeLens = 'all',
    folders = [],
    currentFolder = '',
    publicCount = 0,
    trashCount = 0,
    usedBytes = 0,
    totalObjects = 0,
    onLensChange,
    onFolderChange,
  }: Props = $props();

  function folderCount(path: string, allFolders: FolderEntry[]): number {
    const folder = allFolders.find((f) => f.path === path);
    return folder?.file_count || 0;
  }
</script>

<aside class="drive-sidebar">
  <div class="sidebar-group filters">
    <div class="group-label">Filters</div>
    {#each filters as filter (filter.id)}
      {@const Icon = filter.icon}
      <button
        type="button"
        class="sidebar-item"
        class:active={activeLens === filter.id}
        onclick={() => onLensChange?.(filter.id)}
      >
        <Icon size={14} />
        <span>{filter.label}</span>
        {#if filter.id === 'public'}
          <small>{publicCount}</small>
        {:else if filter.id === 'trash'}
          <small>{trashCount}</small>
        {/if}
      </button>
    {/each}
  </div>

  <div class="sidebar-group folders">
    <div class="group-label">Folders</div>
    <button
      type="button"
      class="sidebar-item"
      class:active={currentFolder === ''}
      onclick={() => onFolderChange?.('')}
    >
      <Icons.FolderOpen size={14} />
      <span>Bucket root</span>
    </button>

    {#each folders as folder (folder.path)}
      <button
        type="button"
        class="sidebar-item"
        class:active={currentFolder === folder.path}
        onclick={() => onFolderChange?.(folder.path)}
      >
        <Icons.Folder size={14} />
        <span>{folder.path}</span>
        <small>{folderCount(folder.path, folders)}</small>
      </button>
    {/each}
  </div>

  <div class="sidebar-storage">
    <div class="group-label">Storage</div>
    <strong>{formatBytes(usedBytes)}</strong>
    <span>{totalObjects} tracked objects</span>
  </div>
</aside>

<style>
  .drive-sidebar {
    width: 200px;
    background: var(--surface);
    border-right: 1px solid var(--border);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    padding: 16px 0;
    flex-shrink: 0;
  }

  .sidebar-group {
    padding: 12px 0;
  }

  .sidebar-group:not(:first-child) {
    border-top: 1px solid var(--border);
  }

  .group-label {
    padding: 0 16px;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
    margin-bottom: 8px;
  }

  .sidebar-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: transparent;
    border: none;
    border-right: 2px solid transparent;
    cursor: pointer;
    color: var(--text-2);
    font-size: 13px;
    transition: all 0.2s;
    text-align: left;
    position: relative;
  }

  .sidebar-item:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .sidebar-item.active {
    color: var(--blue);
    border-right-color: var(--blue);
    background: var(--blue-soft);
  }

  .sidebar-item small {
    margin-left: auto;
    font-size: 11px;
    background: var(--surface-3);
    padding: 2px 6px;
    border-radius: 3px;
    color: var(--muted);
  }

  .sidebar-storage {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 16px;
    border-top: 1px solid var(--border);
    margin-top: auto;
  }

  .sidebar-storage .group-label {
    padding: 0;
    margin: 0;
  }

  .sidebar-storage strong {
    font-size: 16px;
    color: var(--text);
  }

  .sidebar-storage span {
    font-size: 12px;
    color: var(--muted);
  }

  @media (max-width: 1200px) {
    .drive-sidebar {
      display: none;
    }
  }
</style>
