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
  }

  let {
    nodes,
    currentFolder,
    expanded,
    depth = 0,
    onSelect,
    onToggle,
    onMoveToFolder,
  }: Props = $props();

  let dragOverPath = $state<string | null>(null);

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
    } catch {
      // ignore invalid drop payloads
    }
  }
</script>

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
        ondragover={(e) => handleDragOver(e, node.path)}
        ondragleave={() => (dragOverPath = dragOverPath === node.path ? null : dragOverPath)}
        ondrop={(e) => handleDrop(e, node.path)}
      >
        <button
          type="button"
          class="twisty"
          class:invisible={!hasChildren}
          onclick={(e) => {
            e.stopPropagation();
            if (hasChildren) onToggle(node.path);
          }}
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
        <span class="tree-name">{node.name}</span>
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
        />
      {/if}
    </li>
  {/each}
</ul>

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

  .tree-row:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .tree-row.active {
    background: var(--blue-soft);
    color: var(--blue);
  }

  .tree-row.drag-over {
    background: rgba(59, 130, 246, 0.18);
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

  .twisty.invisible {
    visibility: hidden;
  }

  .twisty :global(.twisty-icon) {
    transition: transform 0.12s ease;
  }

  .twisty :global(.twisty-icon.open) {
    transform: rotate(90deg);
  }

  .tree-row :global(.folder-glyph) {
    flex-shrink: 0;
    color: var(--muted);
  }

  .tree-row.active :global(.folder-glyph) {
    color: var(--blue);
  }

  .tree-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tree-count {
    font-size: 11px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
</style>
