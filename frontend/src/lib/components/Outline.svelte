<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface OutlineItem {
    id: string;
    level: number;
    title: string;
    lineNumber: number;
  }

  interface $$Props {
    content?: string;
    onSelectItem?: (lineNumber: number) => void;
    class?: string;
  }

  let {
    content = '',
    onSelectItem,
    class: className,
  }: $$Props = $props();

  let outline = $derived.by(() => {
    const items: OutlineItem[] = [];
    const lines = content.split('\n');
    let id = 0;

    lines.forEach((line, index) => {
      const match = line.match(/^(#+)\s+(.+)$/);
      if (match) {
        const level = match[1].length;
        const title = match[2];
        items.push({
          id: `heading-${id++}`,
          level,
          title,
          lineNumber: index,
        });
      }
    });

    return items;
  });

  function getIndent(level: number) {
    return level - 1;
  }
</script>

<div class="outline {className || ''}">
  <div class="outline-header">
    <h3>Outline</h3>
  </div>

  {#if outline.length === 0}
    <div class="outline-empty">
      <p>No headings found</p>
    </div>
  {:else}
    <div class="outline-list">
      {#each outline as item (item.id)}
        <button
          class="outline-item"
          style="--level: {getIndent(item.level)}"
          onclick={() => onSelectItem?.(item.lineNumber)}
        >
          <Icons.Heading size={14} />
          <span class="item-text">{item.title}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .outline {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
    border-right: 1px solid var(--border);
    overflow: hidden;
  }

  .outline-header {
    padding: var(--s-4);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .outline-header h3 {
    margin: 0;
    font-size: var(--fs-13);
    font-weight: 600;
    color: var(--text);
  }

  .outline-empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--s-6);
    text-align: center;
    color: var(--muted);
    font-size: var(--fs-12);
  }

  .outline-empty p {
    margin: 0;
  }

  .outline-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .outline-item {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    padding-left: calc(var(--s-3) + var(--level) * 16px);
    border: none;
    background: transparent;
    color: var(--text-2);
    font-size: var(--fs-12);
    text-align: left;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-family: var(--ff-sans);
  }

  .outline-item:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .item-text {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
