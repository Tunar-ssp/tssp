<script lang="ts">
  import { onMount } from 'svelte';
  import type { ModalProps } from './primitives.svelte';

  interface $$Props extends ModalProps {
    class?: string;
  }

  let {
    title,
    size = 'md',
    onClose,
    isOpen = false,
    class: className,
    children,
    ...rest
  }: $$Props = $props();

  const sizeClasses = {
    sm: 'modal-sm',
    md: 'modal-md',
    lg: 'modal-lg',
  };

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && onClose) {
      onClose();
    }
  }

  onMount(() => {
    const handleKeydown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen && onClose) {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener('keydown', handleKeydown);
    }

    return () => {
      document.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

{#if isOpen}
  <div class="modal-backdrop" on:click={handleBackdropClick}>
    <div class="modal {sizeClasses[size]} {className || ''}" {...rest}>
      <div class="modal-header">
        <h2 class="modal-title">{title}</h2>
        {#if onClose}
          <button class="modal-close" on:click={onClose} aria-label="Close">
            ×
          </button>
        {/if}
      </div>
      <div class="modal-body">
        <slot />
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1001;
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

  .modal {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-3);
    box-shadow: var(--shadow-modal);
    max-height: 90vh;
    overflow-y: auto;
    animation: modalSlideIn var(--duration-normal) var(--ease-smooth);
  }

  @keyframes modalSlideIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .modal-sm {
    width: 90%;
    max-width: 400px;
  }

  .modal-md {
    width: 90%;
    max-width: 600px;
  }

  .modal-lg {
    width: 90%;
    max-width: 900px;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-6);
    border-bottom: 1px solid var(--border);
  }

  .modal-title {
    margin: 0;
    font-size: var(--fs-18);
    font-weight: 600;
    color: var(--text);
  }

  .modal-close {
    width: 32px;
    height: 32px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    font-size: var(--fs-20);
    font-weight: 300;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-2);
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .modal-close:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .modal-body {
    padding: var(--s-6);
  }
</style>
