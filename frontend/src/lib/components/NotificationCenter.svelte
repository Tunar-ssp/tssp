<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { notifications, removeNotification } from '$lib/stores/notifications';
  import { fly, slide } from 'svelte/transition';
</script>

<div class="notification-stack">
  {#each $notifications as notif (notif.id)}
    <div class="notification {notif.type}" in:fly={{y: -100, duration: 300}} out:slide={{duration: 300}}>
      <div class="notification-icon">
        {#if notif.type === 'success'}
          <Icons.CheckCircle2 size={20} />
        {:else if notif.type === 'error'}
          <Icons.AlertCircle size={20} />
        {:else if notif.type === 'warning'}
          <Icons.AlertTriangle size={20} />
        {:else}
          <Icons.Info size={20} />
        {/if}
      </div>

      <div class="notification-content">
        <div class="notification-title">{notif.title}</div>
        <div class="notification-message">{notif.message}</div>
      </div>

      <button class="notification-close" on:click={() => removeNotification(notif.id)}>
        <Icons.X size={16} />
      </button>
    </div>
  {/each}
</div>

<style>
  .notification-stack {
    position: fixed;
    top: 80px;
    right: 20px;
    z-index: 1000;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 400px;
  }

  .notification {
    padding: 14px 16px;
    border-radius: var(--r-3);
    border: 1px solid;
    background: var(--surface);
    display: flex;
    align-items: flex-start;
    gap: 12px;
    box-shadow: var(--shadow-card);
    animation: slideInRight 0.3s ease-out;
  }

  @keyframes slideInRight {
    from {
      opacity: 0;
      transform: translateX(20px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  .notification.success {
    background: rgba(52, 211, 153, 0.1);
    border-color: rgba(52, 211, 153, 0.25);
    color: var(--green);
  }

  .notification.error {
    background: rgba(255, 107, 107, 0.1);
    border-color: rgba(255, 107, 107, 0.25);
    color: var(--danger);
  }

  .notification.warning {
    background: rgba(251, 191, 36, 0.1);
    border-color: rgba(251, 191, 36, 0.25);
    color: var(--warning);
  }

  .notification.info {
    background: rgba(110, 168, 255, 0.1);
    border-color: rgba(110, 168, 255, 0.25);
    color: var(--blue);
  }

  .notification-icon {
    flex-shrink: 0;
    margin-top: 2px;
  }

  .notification-content {
    flex: 1;
  }

  .notification-title {
    font-size: var(--fs-13);
    font-weight: 600;
    margin-bottom: 2px;
  }

  .notification-message {
    font-size: var(--fs-12);
    opacity: 0.9;
  }

  .notification-close {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    padding: 0;
    border: none;
    background: transparent;
    color: inherit;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: background 0.15s;
  }

  .notification-close:hover {
    background: rgba(255, 255, 255, 0.1);
  }
</style>
