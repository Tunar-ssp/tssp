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
    imageCount?: number;
    videoCount?: number;
    documentCount?: number;
    allFilesCount?: number;
    usedBytes?: number;
    totalObjects?: number;
    onLensChange?: (lens: any) => void;
    onFolderChange?: (path: string) => void;
    onNewFolder?: () => void;
    onMoveToFolder?: (fileIds: string[], path: string) => void;
    onRenameFolder?: (oldPath: string, newName: string) => void;
    onDeleteFolder?: (path: string) => void;
    onClose?: () => void;
  }

  let {
    filters = [],
    activeLens = 'all',
    folders = [],
    currentFolder = '',
    publicCount = 0,
    trashCount = 0,
    imageCount = 0,
    videoCount = 0,
    documentCount = 0,
    allFilesCount = 0,
    usedBytes = 0,
    totalObjects = 0,
    onLensChange,
    onFolderChange,
    onNewFolder,
    onMoveToFolder,
    onRenameFolder,
    onDeleteFolder,
    onClose,
  }: Props = $props();

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

  function getCount(id: string) {
    if (id === 'all') return allFilesCount;
    if (id === 'public') return publicCount;
    if (id === 'trash') return trashCount;
    if (id === 'images') return imageCount;
    if (id === 'videos') return videoCount;
    if (id === 'documents') return documentCount;
    return 0;
  }
</script>

<aside class="drive-sidebar">
  <div class="sidebar-header">
    <span class="sidebar-title">Files</span>
    <button type="button" class="sb-close" onclick={onClose} title="Collapse sidebar">
      <Icons.PanelLeftClose size={15} />
    </button>
  </div>

  <nav class="sidebar-nav">
    {#each filters as filter (filter.id)}
      {@const Icon = filter.icon}
      {@const count = getCount(filter.id)}
      <button
        type="button"
        class="nav-item"
        class:active={activeLens === filter.id && currentFolder === ''}
        onclick={() => { onLensChange?.(filter.id); if (filter.id !== 'trash') onFolderChange?.(''); }}
      >
        <Icon size={16} />
        <span>{filter.label}</span>
        {#if count > 0}
          <span class="nav-count">{count}</span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="section-divider"></div>

  <div class="folders-section">
    <div class="section-header">
      <span>Folders</span>
      <button type="button" class="new-folder-btn" onclick={onNewFolder} title="New folder">
        <Icons.FolderPlus size={13} />
      </button>
    </div>
    <button
      type="button"
      class="nav-item"
      class:active={currentFolder === '' && activeLens === 'all'}
      onclick={() => { onFolderChange?.(''); onLensChange?.('all'); }}
    >
      <Icons.Home size={16} />
      <span>Home</span>
    </button>

    {#if folderTree.length > 0}
      <DriveFolderTree
        nodes={folderTree}
        {currentFolder}
        {expanded}
        onSelect={(path) => { onFolderChange?.(path); onLensChange?.('all'); }}
        onToggle={toggle}
        {onMoveToFolder}
        onRename={onRenameFolder}
        onDelete={onDeleteFolder}
      />
    {/if}
  </div>

  <div class="sidebar-footer">
    <div class="storage-row">
      <Icons.HardDrive size={14} />
      <span class="storage-label">{formatBytes(usedBytes)} used</span>
      <span class="storage-objs">{totalObjects} files</span>
    </div>
  </div>
</aside>

<style>
  .drive-sidebar {
    width: 220px;
    background: var(--surface);
    border-right: 1px solid var(--border);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px 8px;
    border-bottom: 1px solid var(--border);
  }

  .sidebar-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-2);
    letter-spacing: 0.2px;
  }

  .sb-close {
    width: 26px;
    height: 26px;
    border: none;
    background: transparent;
    color: var(--muted);
    border-radius: 5px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.12s, color 0.12s;
  }
  .sb-close:hover { background: var(--surface-2); color: var(--text); }

  .sidebar-nav {
    padding: 6px 6px 0;
  }

  .nav-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 7px 10px;
    background: transparent;
    border: none;
    border-radius: 7px;
    cursor: pointer;
    color: var(--text-2);
    font-size: 13px;
    transition: background 0.1s, color 0.1s;
    text-align: left;
    margin-bottom: 1px;
  }
  .nav-item:hover { background: var(--surface-2); color: var(--text); }
  .nav-item.active { background: var(--blue-soft); color: var(--blue); }
  .nav-item span:first-of-type { flex: 1; }

  .nav-count {
    font-size: 11px;
    background: var(--surface-3);
    color: var(--muted);
    padding: 1px 6px;
    border-radius: 10px;
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .section-divider {
    height: 1px;
    background: var(--border);
    margin: 6px 0;
  }

  .folders-section {
    flex: 1;
    overflow-y: auto;
    padding: 0 6px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 6px 4px 10px;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
  }

  .new-folder-btn {
    width: 22px;
    height: 22px;
    border: none;
    background: transparent;
    color: var(--muted);
    border-radius: 5px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.1s, color 0.1s;
  }
  .new-folder-btn:hover { background: var(--surface-2); color: var(--text); }

  .sidebar-footer {
    padding: 10px 12px;
    border-top: 1px solid var(--border);
    margin-top: auto;
  }

  .storage-row {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--muted);
  }

  .storage-label {
    font-size: 12px;
    flex: 1;
  }

  .storage-objs {
    font-size: 11px;
    color: var(--dim);
  }

  @media (max-width: 900px) {
    .drive-sidebar { display: none; }
  }
</style>
