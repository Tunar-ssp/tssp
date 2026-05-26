<script lang="ts">
  /**
   * Right-rail outline panel - auto-generated from headings
   * Shows document structure and allows quick navigation
   */

  import { editorBlocks, editorSelection, setSelection } from '$lib/blocks/editorStore';
  import type { Block } from '$lib/blocks/types';
  import * as Icons from 'lucide-svelte';

  interface OutlineItem {
    id: string;
    title: string;
    level: number;
  }

  let outline: OutlineItem[] = $state([]);

  $effect(() => {
    // Extract headings to build outline
    const items: OutlineItem[] = [];

    function traverse(blocks: Block[]) {
      for (const block of blocks) {
        if (block.type.startsWith('heading_')) {
          const level = parseInt(block.type.split('_')[1]);
          items.push({
            id: block.id,
            title: block.content || `Untitled Heading ${level}`,
            level,
          });
        }
        if (block.children) {
          traverse(block.children);
        }
      }
    }

    traverse($editorBlocks);
    outline = items;
  });

  function jumpToHeading(id: string) {
    setSelection(id, 0);

    // Scroll into view (future: implement smooth scroll)
    const el = document.querySelector(`[data-block-id="${id}"]`);
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }

  function getPaddingLevel(level: number): string {
    return `${(level - 1) * 12}px`;
  }
</script>

<div class="outline-panel">
  <div class="outline-header">
    <Icons.List size={16} />
    <h3>Outline</h3>
  </div>

  {#if outline.length === 0}
    <div class="outline-empty">
      <p>Add headings to see outline</p>
    </div>
  {:else}
    <nav class="outline-list">
      {#each outline as item (item.id)}
        <button
          class="outline-item"
          class:active={item.id === $editorSelection?.blockId}
          style:padding-left={getPaddingLevel(item.level)}
          onclick={() => jumpToHeading(item.id)}
        >
          <span class="level-badge">H{item.level}</span>
          <span class="outline-title">{item.title}</span>
        </button>
      {/each}
    </nav>
  {/if}
</div>

<style>
  .outline-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    border-left: 1px solid var(--border);
    background-color: var(--bg-secondary);
    overflow: hidden;
  }

  .outline-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 16px;
    border-bottom: 1px solid var(--border);
    color: var(--text);
  }

  .outline-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .outline-empty {
    padding: 24px 16px;
    text-align: center;
    color: var(--muted);
    font-size: 13px;
  }

  .outline-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    padding: 8px;
    gap: 4px;
  }

  .outline-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border: none;
    background: none;
    color: var(--text);
    cursor: pointer;
    text-align: left;
    font-size: 13px;
    border-radius: 4px;
    transition: background-color 0.2s;
  }

  .outline-item:hover {
    background-color: rgba(0, 0, 0, 0.05);
  }

  .outline-item.active {
    background-color: rgba(59, 130, 246, 0.1);
    color: var(--text);
  }

  .level-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 3px;
    background-color: var(--bg);
    font-size: 10px;
    font-weight: 600;
    color: var(--muted);
    flex-shrink: 0;
  }

  .outline-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
