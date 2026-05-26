<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface BreadcrumbItem {
    label: string;
    href?: string;
    onClick?: () => void;
  }

  interface $$Props {
    items?: BreadcrumbItem[];
    class?: string;
  }

  let {
    items = [],
    class: className,
  }: $$Props = $props();
</script>

<nav class="breadcrumb {className || ''}" aria-label="Breadcrumb">
  <ol class="breadcrumb-list">
    {#each items as item, idx (idx)}
      <li class="breadcrumb-item">
        {#if item.href}
          <a href={item.href}>{item.label}</a>
        {:else if item.onClick}
          <button onclick={item.onClick}>{item.label}</button>
        {:else}
          <span class="current">{item.label}</span>
        {/if}

        {#if idx < items.length - 1}
          <Icons.ChevronRight size={14} class="separator" />
        {/if}
      </li>
    {/each}
  </ol>
</nav>

<style>
  .breadcrumb {
    display: flex;
    align-items: center;
  }

  .breadcrumb-list {
    display: flex;
    align-items: center;
    list-style: none;
    margin: 0;
    padding: 0;
    gap: 0;
  }

  .breadcrumb-item {
    display: flex;
    align-items: center;
    font-size: var(--fs-13);
    color: var(--text-2);
  }

  .breadcrumb-item a,
  .breadcrumb-item button {
    background: none;
    border: none;
    color: var(--blue);
    cursor: pointer;
    text-decoration: none;
    padding: var(--s-1) var(--s-2);
    border-radius: var(--r-1);
    transition: all var(--duration-quick) var(--ease-smooth);
    font-family: var(--ff-sans);
    font-size: inherit;
  }

  .breadcrumb-item a:hover,
  .breadcrumb-item button:hover {
    background: var(--surface-2);
  }

  .breadcrumb-item .current {
    padding: var(--s-1) var(--s-2);
    color: var(--text);
  }

  .separator {
    margin: 0 var(--s-1);
    color: var(--muted);
    flex-shrink: 0;
  }
</style>
