<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { Snippet } from 'svelte';

  interface $$Props {
    icon?: any;
    title?: string;
    description?: string;
    action?: string;
    onAction?: () => void;
    variant?: 'default' | 'search' | 'error' | 'permission';
    children?: Snippet;
  }

  let {
    icon = Icons.Inbox,
    title = 'Nothing here',
    description = '',
    action = '',
    onAction = () => {},
    variant = 'default',
    children,
  }: $$Props = $props();

  const variantIcons = {
    default: Icons.Inbox,
    search: Icons.Search,
    error: Icons.AlertCircle,
    permission: Icons.Lock,
  };

  let iconComponent = $derived(icon || variantIcons[variant]);
  let variantClass = $derived(`empty-${variant}`);
</script>

<div class="empty-state {variantClass}">
  <div class="empty-icon">
    <svelte:component this={iconComponent} size={48} />
  </div>
  <h3 class="empty-title">{title}</h3>
  {#if description}
    <p class="empty-description">{description}</p>
  {/if}
  {#if action}
    <button class="empty-action" onclick={onAction}>
      {action}
    </button>
  {/if}
  {#if children}
    {@render children()}
  {/if}
</div>

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-4);
    padding: var(--s-8);
    color: var(--muted);
    text-align: center;
  }

  .empty-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 80px;
    height: 80px;
    border-radius: var(--r-3);
    background: var(--surface);
    color: var(--text-2);
  }

  .empty-title {
    margin: 0;
    font-size: var(--fs-16);
    font-weight: 600;
    color: var(--text-2);
  }

  .empty-description {
    margin: 0;
    font-size: var(--fs-13);
    color: var(--muted);
    max-width: 400px;
  }

  .empty-action {
    padding: var(--s-3) var(--s-4);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--blue);
    color: #0a1228;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .empty-action:hover {
    opacity: 0.9;
  }

  .empty-default .empty-icon {
    background: rgba(110, 168, 255, 0.1);
    color: var(--blue);
  }

  .empty-search .empty-icon {
    background: rgba(168, 168, 168, 0.1);
    color: var(--muted);
  }

  .empty-error .empty-icon {
    background: rgba(255, 107, 107, 0.1);
    color: var(--danger);
  }

  .empty-permission .empty-icon {
    background: rgba(251, 191, 36, 0.1);
    color: var(--warning);
  }
</style>
