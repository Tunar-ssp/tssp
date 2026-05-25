<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    title?: string;
    message?: string;
    action?: string;
    onAction?: () => void;
  }

  let {
    title = 'Something went wrong',
    message = 'An unexpected error occurred. Please try again.',
    action = 'Retry',
    onAction = () => {},
  } = $props<$$Props>();
</script>

<div class="error-state">
  <div class="error-icon">
    <Icons.AlertTriangle size={48} />
  </div>
  <h3 class="error-title">{title}</h3>
  <p class="error-message">{message}</p>
  {#if action}
    <button class="error-action" onclick={onAction}>
      {action}
    </button>
  {/if}
</div>

<style>
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-4);
    padding: var(--s-8);
    color: var(--danger);
    text-align: center;
  }

  .error-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 80px;
    height: 80px;
    border-radius: var(--r-3);
    background: rgba(255, 107, 107, 0.1);
    color: var(--danger);
  }

  .error-title {
    margin: 0;
    font-size: var(--fs-16);
    font-weight: 600;
    color: var(--danger);
  }

  .error-message {
    margin: 0;
    font-size: var(--fs-13);
    color: var(--text-2);
    max-width: 400px;
  }

  .error-action {
    padding: var(--s-3) var(--s-4);
    border: 1px solid var(--danger);
    border-radius: var(--r-2);
    background: transparent;
    color: var(--danger);
    font-weight: 500;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .error-action:hover {
    background: rgba(255, 107, 107, 0.1);
  }
</style>
