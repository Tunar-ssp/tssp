<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import DriveFolderTree from './DriveFolderTree.svelte';

  export interface FolderNode {
    name: string;
    path: string;
    fileCount: number;
    children: FolderNode[];
  }

  interface Props {
    nodes: FolderNode[];
    currentFolder: string;
    expanded: Set<string>;
    depth?: number;
    onSelect: (path: string) => void;
    onToggle: (path: string) => void;
    onMoveToFolder?: (fileIds: string[], path: string) => void;
    onRename?: (oldPath: string, newName: string) => void;
    onDelete?: (path: string) => void;
  }

  let {
    nodes,
    currentFolder,
    expanded,
    depth = 0,
    onSelect,
    onToggle,
    onMoveToFolder,
    onRename,
    onDelete,
  }: Props = $props();

  let dragOverPath = $state<string | null>(null);
  let renamingPath = $state<string | null>(null);
  let renameValue = $state('');
  let contextMenu = $state<{ path: string; x: number; y: number } | null>(null);

  function handleDragOver(e: DragEvent, path: string) {
    if (!e.dataTransfer) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    dragOverPath = path;
  }

  function handleDrop(e: DragEvent, path: string) {
    e.preventDefault();
    dragOverPath = null;
    if (!e.dataTransfer) return;
    try {
      const { fileIds } = JSON.parse(e.dataTransfer.getData('application/json')) as { fileIds: string[] };
      if (fileIds?.length) onMoveToFolder?.(fileIds, path);
    } catch { /* ignore */ }
  }

  function showContextMenu(e: MouseEvent, path: string) {
    e.preventDefault();
    e.stopPropagation();
    contextMenu = { path, x: e.clientX, y: e.clientY };
  }

  function startRename(path: string) {
    const parts = path.split('/');
    renameValue = parts[parts.length - 1];
    renamingPath = path;
    contextMenu = null;
  }

  function finishRename(path: string) {
    const next = renameValue.trim();
    if (next && next !== path.split('/').pop()) {
      onRename?.(path, next);
    }
    renamingPath = null;
    renameValue = '';
  }

  function handleRenameKeydown(e: KeyboardEvent, path: string) {
    if (e.key === 'Enter') { e.preventDefault(); finishRename(path); }
    else if (e.key === 'Escape') { e.preventDefault(); renamingPath = null; }
  }

  function autofocusSelect(node: HTMLInputElement) {
    queueMicrotask(() => { node.focus(); node.select(); });
  }

  function closeCtx() { contextMenu = null; }
</script>

<svelte:window onclick={closeCtx} />

<ul class="folder-tree" class:root={depth === 0}>
  {#each nodes as node (node.path)}
    {@const isOpen = expanded.has(node.path)}
    {@const hasChildren = node.children.length > 0}
    <li>
      <div
        class="tree-row"
        class:active={currentFolder === node.path}
        class:drag-over={dragOverPath === node.path}
        style="padding-left: {8 + depth * 14}px"
        role="treeitem"
        aria-selected={currentFolder === node.path}
        aria-expanded={hasChildren ? isOpen : undefined}
        tabindex="0"
        onclick={() => onSelect(node.path)}
        onkeydown={(e) => {
          if (e.key === 'Enter') onSelect(node.path);
          else if (e.key === 'ArrowRight' && hasChildren && !isOpen) onToggle(node.path);
          else if (e.key === 'ArrowLeft' && hasChildren && isOpen) onToggle(node.path);
        }}
        oncontextmenu={(e) => showContextMenu(e, node.path)}
        ondragover={(e) => handleDragOver(e, node.path)}
        ondragleave={() => (dragOverPath = dragOverPath === node.path ? null : dragOverPath)}
        ondrop={(e) => handleDrop(e, node.path)}
      >
        <button
          type="button"
          class="twisty"
          class:invisible={!hasChildren}
          onclick={(e) => { e.stopPropagation(); if (hasChildren) onToggle(node.path); }}
          tabindex="-1"
          aria-label={isOpen ? 'Collapse' : 'Expand'}
        >
          <Icons.ChevronRight size={13} class={isOpen ? 'twisty-icon open' : 'twisty-icon'} />
        </button>
        {#if isOpen && hasChildren}
          <Icons.FolderOpen size={15} class="folder-glyph" />
        {:else}
          <Icons.Folder size={15} class="folder-glyph" />
        {/if}
        {#if renamingPath === node.path}
          <input
            type="text"
            class="rename-input"
            bind:value={renameValue}
            onblur={() => finishRename(node.path)}
            onkeydown={(e) => handleRenameKeydown(e, node.path)}
            onclick={(e) => e.stopPropagation()}
            use:autofocusSelect
          />
        {:else}
          <span class="tree-name">{node.name}</span>
        {/if}
        {#if node.fileCount > 0}
          <small class="tree-count">{node.fileCount}</small>
        {/if}
      </div>

      {#if isOpen && hasChildren}
        <DriveFolderTree
          nodes={node.children}
          {currentFolder}
          {expanded}
          depth={depth + 1}
          {onSelect}
          {onToggle}
          {onMoveToFolder}
          {onRename}
          {onDelete}
        />
      {/if}
    </li>
  {/each}
</ul>

{#if contextMenu}
  <div
    class="ctx-menu"
    style="top: {contextMenu.y}px; left: {contextMenu.x}px"
    role="menu"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === 'Escape' && closeCtx()}
  >
    <button type="button" onclick={() => { startRename(contextMenu!.path); }}>
      <Icons.Pencil size={13} />
      Rename
    </button>
    <button type="button" class="danger" onclick={() => { onDelete?.(contextMenu!.path); closeCtx(); }}>
      <Icons.Trash2 size={13} />
      Delete folder
    </button>
  </div>
{/if}

<style>
  .folder-tree {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .tree-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 8px;
    cursor: pointer;
    color: var(--text-2);
    font-size: 13px;
    border-radius: 6px;
    margin: 1px 6px;
    transition: background 0.12s, color 0.12s;
    user-select: none;
  }

  .tree-row:hover { background: var(--surface-2); color: var(--text); }
  .tree-row.active { background: var(--blue-soft); color: var(--blue); }
  .tree-row.drag-over {
    background: rgba(59,130,246,0.18);
    box-shadow: inset 0 0 0 1px var(--blue);
    color: var(--text);
  }

  .twisty {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    flex-shrink: 0;
  }
  .twisty.invisible { visibility: hidden; }

  .twisty :global(.twisty-icon) { transition: transform 0.12s ease; }
  .twisty :global(.twisty-icon.open) { transform: rotate(90deg); }

  .tree-row :global(.folder-glyph) { flex-shrink: 0; color: var(--muted); }
  .tree-row.active :global(.folder-glyph) { color: var(--blue); }

  .tree-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tree-count { font-size: 11px; color: var(--muted); font-variant-numeric: tabular-nums; }

  .rename-input {
    flex: 1;
    min-width: 0;
    padding: 1px 4px;
    font-size: 13px;
    border: 1px solid var(--blue);
    border-radius: 4px;
    background: var(--bg);
    color: var(--text);
    outline: none;
  }

  .ctx-menu {
    position: fixed;
    z-index: 2000;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 4px;
    box-shadow: 0 8px 24px rgba(0,0,0,0.4);
    min-width: 160px;
  }

  .ctx-menu button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 10px;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: 13px;
    border-radius: 5px;
    transition: background 0.1s;
    text-align: left;
  }

  .ctx-menu button:hover { background: var(--surface); color: var(--text); }
  .ctx-menu button.danger { color: var(--danger); }
  .ctx-menu button.danger:hover { background: rgba(239,68,68,0.1); }
</style>
