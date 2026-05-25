<script lang="ts">
  interface DockItem {
    id: string;
    label: string;
    icon: any;
    action: () => void;
    badge?: number;
  }

  interface $$Props {
    items?: DockItem[];
    class?: string;
  }

  let {
    items = [],
    class: className,
  } = $props<$$Props>();

  let hoveredId = $state<string | null>(null);
</script>

<nav class="dock {className || ''}">
  <div class="dock-container">
    {#each items as item (item.id)}
      <button
        class="dock-item"
        class:hovered={hoveredId === item.id}
        on:mouseenter={() => (hoveredId = item.id)}
        on:mouseleave={() => (hoveredId = null)}
        on:click={item.action}
        title={item.label}
      >
        <div class="dock-icon">
          <svelte:component this={item.icon} size={24} />
        </div>
        {#if item.badge}
          <div class="dock-badge">{item.badge}</div>
        {/if}
      </button>
    {/each}
  </div>
</nav>

<style>
  .dock {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 99;
    pointer-events: none;
  }

  .dock-container {
    display: flex;
    align-items: flex-end;
    justify-content: center;
    gap: 8px;
    padding: 12px;
    background: var(--dock-glass);
    border: 1px solid var(--dock-edge);
    border-radius: var(--r-full);
    backdrop-filter: blur(20px);
    box-shadow: var(--shadow-dock);
    pointer-events: all;
  }

  .dock-item {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 50px;
    height: 50px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    border-radius: var(--r-3);
    cursor: pointer;
    position: relative;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .dock-item:hover {
    color: var(--text);
    background: rgba(255, 255, 255, 0.06);
    transform: scale(1.1);
  }

  .dock-item.hovered {
    color: var(--blue);
    background: rgba(110, 168, 255, 0.1);
  }

  .dock-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
  }

  .dock-badge {
    position: absolute;
    top: -2px;
    right: -2px;
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    height: 20px;
    padding: 0 6px;
    background: var(--danger);
    color: white;
    border-radius: var(--r-full);
    font-size: var(--fs-11);
    font-weight: 600;
    border: 2px solid var(--surface);
  }
</style>
