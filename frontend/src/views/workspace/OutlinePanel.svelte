<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface OutlineItem {
    id: string;
    label: string;
    line: number;
    level: number;
    type: 'heading' | 'function' | 'class' | 'section';
  }

  interface $$Props {
    items?: OutlineItem[];
    activeItemId?: string;
    onNavigate?: (line: number) => void;
  }

  let {
    items = [],
    activeItemId = '',
    onNavigate = () => {},
  }: $$Props = $props();

  function getIcon(type: string) {
    switch (type) {
      case 'heading':
        return Icons.Heading2;
      case 'function':
        return Icons.Function;
      case 'class':
        return Icons.Braces;
      case 'section':
        return Icons.Layers;
      default:
        return Icons.Circle;
    }
  }

  function getTypeColor(type: string) {
    switch (type) {
      case 'heading':
        return 'var(--blue)';
      case 'function':
        return 'var(--green)';
      case 'class':
        return 'var(--violet)';
      case 'section':
        return 'var(--orange)';
      default:
        return 'var(--text-2)';
    }
  }
</script>

<div class="outline-panel">
  <div class="outline-header">
    <h3>Outline</h3>
  </div>

  {#if items.length === 0}
    <div class="empty-state">
      <Icons.Inbox size={24} />
      <p>No outline available</p>
      <small>Open a code file to see structure</small>
    </div>
  {:else}
    <div class="outline-list">
      {#each items as item (item.id)}
        <button
          type="button"
          class="outline-item"
          class:active={activeItemId === item.id}
          style="--indent: {item.level * 16}px; --type-color: {getTypeColor(item.type)}"
          onclick={() => onNavigate(item.line)}
          title={`Line ${item.line}`}
        >
          <span class="outline-icon">
            <svelte:component this={getIcon(item.type)} size={14} />
          </span>
          <span class="outline-label">{item.label}</span>
          <span class="outline-line">{item.line}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .outline-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
  }

  .outline-header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .outline-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-2);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 32px 16px;
    color: var(--muted);
    text-align: center;
  }

  .empty-state p {
    margin: 0;
    font-size: 14px;
  }

  .empty-state small {
    font-size: 12px;
    color: var(--dim);
  }

  .outline-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .outline-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    margin-left: var(--indent);
    border: none;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
    font-size: 12px;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .outline-item:hover {
    background: var(--surface-2);
  }

  .outline-item.active {
    background: color-mix(in srgb, var(--type-color) 12%, transparent);
    border-left: 2px solid var(--type-color);
    padding-left: 10px;
  }

  .outline-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--type-color);
    flex-shrink: 0;
  }

  .outline-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .outline-line {
    font-family: var(--ff-mono);
    font-size: 10px;
    color: var(--muted);
    flex-shrink: 0;
  }
</style>
