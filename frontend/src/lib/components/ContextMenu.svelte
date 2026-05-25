<script lang="ts">
  import * as Icons from 'lucide-svelte';

  export let x: number = 0;
  export let y: number = 0;
  export let visible: boolean = false;
  export let items: Array<{ icon?: any; label: string; action: () => void; danger?: boolean }> = [];

  function handleClickOutside() {
    visible = false;
  }
</script>

{#if visible}
  <button
    type="button"
    class="context-overlay"
    aria-label="Close context menu"
    onclick={handleClickOutside}
  ></button>
  <div class="context-menu" role="menu" style="left: {x}px; top: {y}px">
    {#each items as item (item.label)}
      {@const Icon = item.icon}
      <button
        type="button"
        role="menuitem"
        class="context-item {item.danger ? 'danger' : ''}"
        onclick={() => {
          item.action();
          visible = false;
        }}
      >
        {#if Icon}
          <div class="item-icon">
            <Icon size={16} />
          </div>
        {/if}
        <span>{item.label}</span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .context-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 999;
    padding: 0;
    border: none;
    background: transparent;
  }

  .context-menu {
    position: fixed;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    box-shadow: var(--shadow-modal);
    z-index: 1000;
    min-width: 180px;
    overflow: hidden;
    animation: contextIn 0.15s ease-out;
  }

  @keyframes contextIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .context-item {
    width: 100%;
    padding: 10px 12px;
    border: none;
    background: transparent;
    color: var(--text);
    text-align: left;
    font-size: var(--fs-13);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 10px;
    transition: background 0.15s;
    border-bottom: 1px solid var(--hairline);
  }

  .context-item:last-child {
    border-bottom: none;
  }

  .context-item:hover {
    background: var(--surface-2);
  }

  .context-item.danger {
    color: var(--danger);
  }

  .context-item.danger:hover {
    background: rgba(255, 107, 107, 0.1);
  }

  .item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
  }
</style>
