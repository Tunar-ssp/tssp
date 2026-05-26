<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface BreadcrumbItem {
    label: string;
    icon?: any;
    onClick?: () => void;
    isLast?: boolean;
  }

  interface $$Props {
    items?: BreadcrumbItem[];
    maxItems?: number;
  }

  let {
    items = [],
    maxItems = 5,
  }: $$Props = $props();

  let visibleItems = $derived.by(() => {
    if (items.length <= maxItems) {
      return items;
    }

    const result = [items[0]];
    if (items.length > maxItems) {
      result.push({
        label: `... (${items.length - maxItems} more)`,
        icon: Icons.MoreHorizontal,
      });
    }
    result.push(...items.slice(-(maxItems - 2)));
    return result;
  });
</script>

<nav class="breadcrumb-nav">
  <ol class="breadcrumb-list">
    {#each visibleItems as item, index (index)}
      <li class="breadcrumb-item">
        {#if item.icon}
          <svelte:component this={item.icon} size={14} />
        {/if}
        <button
          type="button"
          class="breadcrumb-button"
          class:active={item.isLast}
          onclick={item.onClick}
          disabled={!item.onClick}
        >
          {item.label}
        </button>
        {#if index < visibleItems.length - 1}
          <span class="breadcrumb-separator">
            <Icons.ChevronRight size={14} />
          </span>
        {/if}
      </li>
    {/each}
  </ol>
</nav>

<style>
  .breadcrumb-nav {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: thin;
  }

  .breadcrumb-list {
    display: flex;
    align-items: center;
    gap: 0;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .breadcrumb-item {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: fit-content;
    color: var(--text-2);
    font-size: 13px;
  }

  .breadcrumb-button {
    padding: 4px 8px;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: 13px;
    border-radius: 4px;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .breadcrumb-button:hover:not(:disabled) {
    background: var(--surface-2);
    color: var(--text);
  }

  .breadcrumb-button.active {
    color: var(--text);
    font-weight: 500;
    cursor: default;
  }

  .breadcrumb-button:disabled {
    cursor: default;
  }

  .breadcrumb-separator {
    display: flex;
    align-items: center;
    color: var(--muted);
  }
</style>
