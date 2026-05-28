<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { untrack } from 'svelte';
  import type { FolderEntry } from '$lib/api';
  import { formatBytes } from '$lib/utils';
  import DriveFolderTree, { type FolderNode } from './components/panels/DriveFolderTree.svelte';

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
    onNewFolder?: () => void;
    onMoveToFolder?: (fileIds: string[], path: string) => void;
    onClose?: () => void;
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
    onNewFolder,
    onMoveToFolder,
    onClose,
  }: Props = $props();

  // Build a nested tree out of the flat folder paths the backend returns.
  let folderTree = $derived.by(() => {
    const roots: FolderNode[] = [];
    const index = new Map<string, FolderNode>();

    const ensure = (path: string): FolderNode => {
      const existing = index.get(path);
      if (existing) return existing;
      const parts = path.split('/');
      const name = parts[parts.length - 1];
      const node: FolderNode = { name, path, fileCount: 0, children: [] };
      index.set(path, node);
      if (parts.length === 1) {
        roots.push(node);
      } else {
        ensure(parts.slice(0, -1).join('/')).children.push(node);
      }
      return node;
    };

    for (const entry of folders) {
      if (!entry.path) continue;
      ensure(entry.path).fileCount = entry.file_count || 0;
    }

    const sortTree = (list: FolderNode[]) => {
      list.sort((a, b) => a.name.localeCompare(b.name));
      list.forEach((n) => sortTree(n.children));
    };
    sortTree(roots);
    return roots;
  });

  // Auto-expand ancestors of the active folder so it is always visible.
  // Only depend on currentFolder — untrack the expanded read/write so the
  // effect doesn't retrigger itself (effect_update_depth_exceeded).
  let expanded = $state<Set<string>>(new Set());
  $effect(() => {
    const folder = currentFolder;
    if (!folder) return;
    untrack(() => {
      const parts = folder.split('/');
      let changed = false;
      for (let i = 1; i < parts.length; i++) {
        const ancestor = parts.slice(0, i).join('/');
        if (!expanded.has(ancestor)) {
          expanded.add(ancestor);
          changed = true;
        }
      }
      if (changed) expanded = new Set(expanded);
    });
  });

  function toggle(path: string) {
    if (expanded.has(path)) expanded.delete(path);
    else expanded.add(path);
    expanded = new Set(expanded);
  }

  const TOTAL_STORAGE_BYTES = 1024 * 1024 * 1024 * 100; // 100GB
  let storagePercentage = $derived(Math.min(100, (usedBytes / TOTAL_STORAGE_BYTES) * 100));
</script>

<aside class="drive-sidebar">
  {#if onClose}
    <div class="sidebar-top">
      <button type="button" class="sb-close" onclick={onClose} title="Hide sidebar (Ctrl+B)">
        <Icons.PanelLeftClose size={16} />
      </button>
    </div>
  {/if}
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
    <div class="group-label">
      <span>Folders</span>
      {#if onNewFolder}
        <button type="button" class="group-action" onclick={onNewFolder} title="New folder">
          <Icons.FolderPlus size={14} />
        </button>
      {/if}
    </div>
    <button
      type="button"
      class="sidebar-item root-folder"
      class:active={currentFolder === ''}
      onclick={() => onFolderChange?.('')}
    >
      <Icons.HardDrive size={14} />
      <span>All files</span>
    </button>

    {#if folderTree.length > 0}
      <DriveFolderTree
        nodes={folderTree}
        {currentFolder}
        {expanded}
        onSelect={(path) => onFolderChange?.(path)}
        onToggle={toggle}
        {onMoveToFolder}
      />
    {:else}
      <p class="folders-empty">No folders yet</p>
    {/if}
  </div>

  <div class="sidebar-storage">
    <div class="group-label">Storage</div>
    <div class="storage-bar">
      <div class="storage-progress" style="width: {storagePercentage}%"></div>
    </div>
    <div class="storage-info">
      <strong>{formatBytes(usedBytes)}</strong>
      <span>{storagePercentage.toFixed(1)}% used</span>
    </div>
    <span class="storage-objects">{totalObjects} objects</span>
  </div>
</aside>

<style>
  .drive-sidebar {
    width: 240px;
    background: var(--surface);
    border-right: 1px solid var(--border);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    padding: 0 0 16px;
    flex-shrink: 0;
  }

  .sidebar-top {
    padding: 8px 12px;
    display: flex;
    justify-content: flex-end;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }
  .sb-close {
    width: 28px;
    height: 28px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-2);
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .sb-close:hover {
    background: rgba(255, 255, 255, 0.06);
    color: var(--text);
    border-color: var(--border);
  }

  .sidebar-group {
    padding: 12px 0;
  }

  .sidebar-group:not(:first-child) {
    border-top: 1px solid var(--border);
  }

  .group-label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
    margin-bottom: 8px;
  }

  .group-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    border-radius: 5px;
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
  }

  .group-action:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .folders-empty {
    margin: 4px 16px;
    font-size: 12px;
    color: var(--dim);
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

  .storage-bar {
    width: 100%;
    height: 6px;
    background: var(--surface-2);
    border-radius: 3px;
    overflow: hidden;
    margin: 4px 0;
  }

  .storage-progress {
    height: 100%;
    background: linear-gradient(90deg, var(--blue), var(--blue-hover));
    transition: width 300ms ease-out;
  }

  .storage-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }

  .storage-info strong {
    font-size: 13px;
    color: var(--text);
  }

  .storage-info span {
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
  }

  .storage-objects {
    font-size: 11px;
    color: var(--muted);
  }

  @media (max-width: 1200px) {
    .drive-sidebar {
      display: none;
    }
  }
</style>
