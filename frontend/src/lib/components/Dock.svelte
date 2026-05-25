<script lang="ts">
  interface DockItem {
    id: string;
    label: string;
    icon: any;
    action: () => void;
    badge?: number;
    accent?: string;
  }

  interface $$Props {
    items?: DockItem[];
    activeId?: string;
    class?: string;
  }

  let {
    items = [],
    activeId = '',
    class: className,
  } = $props<$$Props>();

  let hoveredId = $state<string | null>(null);
</script>

<nav class="dock {className || ''}">
  <div class="dock-glow" aria-hidden="true"></div>
  <div class="dock-container">
    {#each items as item (item.id)}
      {@const Icon = item.icon}
      <button
        class="dock-item"
        class:active={activeId === item.id}
        class:hovered={hoveredId === item.id}
        style={item.accent ? `--item-accent:${item.accent}` : ''}
        onmouseenter={() => (hoveredId = item.id)}
        onmouseleave={() => (hoveredId = null)}
        onclick={item.action}
        title={item.label}
        aria-label={`Open ${item.label}`}
        aria-current={activeId === item.id ? 'page' : undefined}
      >
        <div class="dock-icon">
          <Icon size={24} />
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
    bottom: 18px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 90;
    pointer-events: none;
  }

  .dock-glow {
    position: absolute;
    inset: -20px 8px 0;
    z-index: -1;
    border-radius: 42px;
    background:
      radial-gradient(circle at 20% 20%, rgba(110, 168, 255, 0.22), transparent 34%),
      radial-gradient(circle at 78% 12%, rgba(91, 227, 154, 0.18), transparent 30%),
      radial-gradient(circle at 50% 100%, rgba(255, 95, 162, 0.16), transparent 40%);
    filter: blur(18px);
    opacity: 0.85;
  }

  .dock-container {
    display: flex;
    align-items: flex-end;
    justify-content: center;
    gap: 10px;
    padding: 10px 12px;
    background:
      linear-gradient(180deg, rgba(255,255,255,0.14), rgba(255,255,255,0.035)),
      rgba(14, 16, 22, 0.72);
    border: 1px solid rgba(255,255,255,0.13);
    border-radius: 24px;
    backdrop-filter: blur(20px);
    box-shadow:
      0 20px 50px rgba(0,0,0,.58),
      0 1px 0 rgba(255,255,255,.2) inset,
      0 -1px 0 rgba(0,0,0,.45) inset;
    pointer-events: all;
  }

  .dock-item {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 58px;
    height: 58px;
    padding: 0;
    border: 1px solid rgba(255,255,255,0.1);
    background:
      linear-gradient(145deg, color-mix(in srgb, var(--item-accent, var(--blue)) 28%, transparent), rgba(255,255,255,0.04)),
      rgba(255,255,255,0.035);
    color: var(--text);
    border-radius: 16px;
    cursor: pointer;
    position: relative;
    transition: all var(--duration-quick) var(--ease-smooth);
    box-shadow:
      0 10px 20px rgba(0,0,0,.3),
      0 1px 0 rgba(255,255,255,.12) inset;
  }

  .dock-item:hover {
    color: var(--text);
    border-color: color-mix(in srgb, var(--item-accent, var(--blue)) 52%, white 8%);
    transform: translateY(-8px) scale(1.08);
  }

  .dock-item.hovered,
  .dock-item.active {
    background:
      linear-gradient(145deg, color-mix(in srgb, var(--item-accent, var(--blue)) 46%, transparent), rgba(255,255,255,0.08)),
      rgba(255,255,255,0.04);
  }

  .dock-item.active::after {
    content: "";
    position: absolute;
    bottom: -8px;
    width: 7px;
    height: 7px;
    border-radius: 999px;
    background: var(--orange);
    box-shadow: 0 0 14px color-mix(in srgb, var(--orange) 70%, transparent);
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

  @media (max-width: 640px) {
    .dock {
      bottom: 12px;
    }

    .dock-container {
      gap: 8px;
      padding: 8px;
      border-radius: 20px;
    }

    .dock-item {
      width: 50px;
      height: 50px;
      border-radius: 14px;
    }
  }
</style>
