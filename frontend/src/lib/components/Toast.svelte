<script lang="ts">
  import { onMount } from 'svelte';

  interface $$Props {
    tone?: 'success' | 'warning' | 'danger' | 'info';
    title?: string;
    body?: string;
    action?: () => void;
    actionLabel?: string;
    onClose?: () => void;
    autoDismiss?: number;
    class?: string;
  }

  let {
    tone = 'info',
    title,
    body,
    action,
    actionLabel = 'Action',
    onClose,
    autoDismiss = 4000,
    class: className,
    ...rest
  }: $$Props = $props();

  let timeout: ReturnType<typeof setTimeout>;

  onMount(() => {
    if (autoDismiss > 0 && onClose) {
      timeout = setTimeout(onClose, autoDismiss);
    }

    return () => clearTimeout(timeout);
  });

  const toneClasses = {
    success: 'toast-success',
    warning: 'toast-warning',
    danger: 'toast-danger',
    info: 'toast-info',
  };
</script>

<div class="toast {toneClasses[tone]} {className || ''}" {...rest}>
  <div class="toast-content">
    {#if title}
      <div class="toast-title">{title}</div>
    {/if}
    {#if body}
      <div class="toast-body">{body}</div>
    {/if}
  </div>

  {#if action}
    <button class="toast-action" onclick={action}>
      {actionLabel}
    </button>
  {/if}

  {#if onClose}
    <button class="toast-close" onclick={onClose} aria-label="Close">
      ×
    </button>
  {/if}
</div>

<style>
  .toast {
    display: flex;
    align-items: flex-start;
    gap: var(--s-3);
    padding: var(--s-3) var(--s-4);
    border-radius: var(--r-2);
    border-left: 3px solid;
    background: var(--surface);
    box-shadow: var(--shadow-card);
    font-size: var(--fs-13);
    animation: slideIn var(--duration-normal) var(--ease-smooth);
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateX(-20px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  .toast-success {
    border-left-color: var(--success);
  }

  .toast-warning {
    border-left-color: var(--warning);
  }

  .toast-danger {
    border-left-color: var(--danger);
  }

  .toast-info {
    border-left-color: var(--blue);
  }

  .toast-content {
    flex: 1;
    min-width: 0;
  }

  .toast-title {
    font-weight: 500;
    color: var(--text);
    margin-bottom: 2px;
  }

  .toast-body {
    color: var(--text-2);
    font-size: var(--fs-12);
  }

  .toast-action {
    flex-shrink: 0;
    padding: 4px 12px;
    border: none;
    border-radius: var(--r-1);
    background: var(--surface-2);
    color: var(--text-2);
    font-size: var(--fs-12);
    font-weight: 500;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .toast-action:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .toast-close {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    font-size: var(--fs-16);
    font-weight: 300;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color var(--duration-quick) var(--ease-smooth);
  }

  .toast-close:hover {
    color: var(--text);
  }
</style>
