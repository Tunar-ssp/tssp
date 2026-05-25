<script lang="ts">
  import type { SheetProps } from './primitives.svelte';

  interface $$Props extends SheetProps {
    class?: string;
  }

  let {
    side = 'right',
    isOpen = false,
    onClose,
    title,
    class: className,
    children,
    ...rest
  }: $$Props = $props();

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && onClose) {
      onClose();
    }
  }
</script>

{#if isOpen}
  <div class="sheet-backdrop" on:click={handleBackdropClick}>
    <div
      class="sheet sheet-{side} {className || ''}"
      {...rest}
    >
      {#if title}
        <div class="sheet-header">
          <h2 class="sheet-title">{title}</h2>
          {#if onClose}
            <button class="sheet-close" on:click={onClose} aria-label="Close">
              ×
            </button>
          {/if}
        </div>
      {/if}
      <div class="sheet-body">
        <slot />
      </div>
    </div>
  </div>
{/if}

<style>
  .sheet-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.4);
    z-index: 999;
    animation: fadeIn var(--duration-normal) var(--ease-smooth);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .sheet {
    position: fixed;
    height: 100vh;
    background: var(--surface);
    border: 1px solid var(--border);
    box-shadow: var(--shadow-card);
    z-index: 999;
    display: flex;
    flex-direction: column;
  }

  .sheet-right {
    right: 0;
    top: 0;
    width: min(100%, 400px);
    animation: slideInRight var(--duration-normal) var(--ease-smooth);
  }

  .sheet-bottom {
    bottom: 0;
    left: 0;
    right: 0;
    height: 50vh;
    max-height: 80vh;
    animation: slideInUp var(--duration-normal) var(--ease-smooth);
  }

  @keyframes slideInRight {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }

  @keyframes slideInUp {
    from {
      transform: translateY(100%);
    }
    to {
      transform: translateY(0);
    }
  }

  .sheet-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-5);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .sheet-title {
    margin: 0;
    font-size: var(--fs-16);
    font-weight: 600;
    color: var(--text);
  }

  .sheet-close {
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    font-size: var(--fs-18);
    font-weight: 300;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-2);
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .sheet-close:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .sheet-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-5);
  }
</style>
