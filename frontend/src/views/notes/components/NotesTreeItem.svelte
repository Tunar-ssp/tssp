<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { get } from 'svelte/store';
  import type { NoteTreeNode } from '$lib/notes/tree';
  import { draggingNoteId } from '$lib/stores/notesDnd';
  import NotesTreeItem from './NotesTreeItem.svelte';

  interface Props {
    node: NoteTreeNode;
    activeId: string | null;
    isExpanded: (id: string) => boolean;
    onToggle: (id: string) => void;
    onSelect: (id: string) => void;
    onCreateChild: (id: string) => void;
    onContext: (event: MouseEvent, node: NoteTreeNode) => void;
    onMoveInto: (dragId: string, targetId: string | null) => void;
    canDrop: (dragId: string, targetId: string) => boolean;
  }

  let {
    node,
    activeId,
    isExpanded,
    onToggle,
    onSelect,
    onCreateChild,
    onContext,
    onMoveInto,
    canDrop,
  }: Props = $props();

  let isOver = $state(false);
  let expanded = $derived(isExpanded(node.note.id));
  let hasChildren = $derived(node.children.length > 0);
  let isActive = $derived(activeId === node.note.id);

  function handleDragStart(e: DragEvent) {
    draggingNoteId.set(node.note.id);
    e.dataTransfer?.setData('text/plain', node.note.id);
    if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
  }

  function handleDragOver(e: DragEvent) {
    const dragId = get(draggingNoteId);
    if (!dragId || !canDrop(dragId, node.note.id)) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
    isOver = true;
  }

  function handleDrop(e: DragEvent) {
    isOver = false;
    const dragId = get(draggingNoteId);
    draggingNoteId.set(null);
    if (!dragId || !canDrop(dragId, node.note.id)) return;
    e.preventDefault();
    e.stopPropagation();
    onMoveInto(dragId, node.note.id);
    // Reveal the destination so the moved page is visible.
    if (!isExpanded(node.note.id)) onToggle(node.note.id);
  }
</script>

<div
  class="tree-row"
  class:active={isActive}
  class:drop-over={isOver}
  style="padding-left: {node.depth * 14 + 6}px"
  role="treeitem"
  aria-selected={isActive}
  aria-expanded={hasChildren ? expanded : undefined}
  tabindex="-1"
  draggable="true"
  ondragstart={handleDragStart}
  ondragend={() => { draggingNoteId.set(null); isOver = false; }}
  ondragover={handleDragOver}
  ondragleave={() => (isOver = false)}
  ondrop={handleDrop}
  onclick={() => onSelect(node.note.id)}
  oncontextmenu={(e) => onContext(e, node)}
  onkeydown={(e) => { if (e.key === 'Enter') onSelect(node.note.id); }}
>
  <button
    type="button"
    class="twisty"
    class:invisible={!hasChildren}
    onclick={(e) => { e.stopPropagation(); onToggle(node.note.id); }}
    tabindex="-1"
    aria-label={expanded ? 'Collapse' : 'Expand'}
  >
    <Icons.ChevronRight size={14} class={expanded ? 'rot' : ''} />
  </button>

  <span class="page-icon">
    {#if node.note.icon}
      {node.note.icon}
    {:else}
      <Icons.FileText size={15} />
    {/if}
  </span>

  <span class="page-title">{node.note.title || 'Untitled'}</span>

  <span class="row-actions">
    <button
      type="button"
      class="row-action"
      title="More"
      onclick={(e) => { e.stopPropagation(); onContext(e, node); }}
      tabindex="-1"
    >
      <Icons.MoreHorizontal size={14} />
    </button>
    <button
      type="button"
      class="row-action"
      title="Add a page inside"
      onclick={(e) => { e.stopPropagation(); onCreateChild(node.note.id); }}
      tabindex="-1"
    >
      <Icons.Plus size={14} />
    </button>
  </span>
</div>

{#if expanded && hasChildren}
  {#each node.children as child (child.note.id)}
    <NotesTreeItem
      node={child}
      {activeId}
      {isExpanded}
      {onToggle}
      {onSelect}
      {onCreateChild}
      {onContext}
      {onMoveInto}
      {canDrop}
    />
  {/each}
{/if}

<style>
  .tree-row {
    display: flex;
    align-items: center;
    gap: 2px;
    height: 28px;
    padding-right: 6px;
    border-radius: 6px;
    cursor: pointer;
    color: var(--text-2, #b8c0cc);
    user-select: none;
    position: relative;
  }
  .tree-row:hover {
    background: var(--surface-2, rgba(255, 255, 255, 0.05));
    color: var(--text);
  }
  .tree-row.active {
    background: var(--surface-2, rgba(110, 168, 254, 0.16));
    color: var(--text);
  }
  .tree-row.drop-over {
    background: rgba(110, 168, 254, 0.22);
    box-shadow: inset 0 0 0 1px var(--accent, #6ea8fe);
  }

  .twisty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border: none;
    background: transparent;
    color: var(--muted);
    border-radius: 4px;
    cursor: pointer;
    flex-shrink: 0;
  }
  .twisty:hover {
    background: rgba(255, 255, 255, 0.08);
  }
  .twisty.invisible {
    visibility: hidden;
  }
  :global(.twisty .rot) {
    transform: rotate(90deg);
    transition: transform 0.12s ease;
  }

  .page-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    font-size: 14px;
    line-height: 1;
    flex-shrink: 0;
    color: var(--muted);
  }

  .page-title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13.5px;
  }

  .row-actions {
    display: none;
    align-items: center;
    gap: 1px;
    flex-shrink: 0;
  }
  .tree-row:hover .row-actions {
    display: flex;
  }
  .row-action {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border: none;
    background: transparent;
    color: var(--muted);
    border-radius: 4px;
    cursor: pointer;
  }
  .row-action:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
  }
</style>
