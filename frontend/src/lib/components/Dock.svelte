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
    mode?: 'always' | 'autohide' | 'compact' | 'hidden';
    class?: string;
  }

  let {
    items = [],
    activeId = '',
    mode = 'always',
    class: className,
  }: $$Props = $props();

  let hoveredId = $state<string | null>(null);
</script>

{#if mode !== 'hidden'}
<nav class="dock {className || ''}" class:autohide={mode === 'autohide'} class:compact={mode === 'compact'}>
  <div class="dock-glow" aria-hidden="true"></div>
  <div class="dock-container">
    {#each items as item (item.id)}
      {@const Icon = item.icon}
      <div class="dock-item-wrap">
        <span class="dock-label">{item.label}</span>
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
            <Icon size={mode === 'compact' ? 22 : 26} />
          </div>
          {#if item.badge}
            <div class="dock-badge">{item.badge}</div>
          {/if}
        </button>
      </div>
    {/each}
  </div>
</nav>
{/if}

<style>
  .dock {
    position: fixed;
    bottom: 22px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 140;
    pointer-events: none;
    transition: transform var(--duration-normal) var(--ease-smooth), opacity var(--duration-normal) var(--ease-smooth);
  }

  .dock-glow {
    position: absolute;
    inset: -24px 14px 0;
    z-index: -1;
    border-radius: 40px;
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
    gap: 14px;
    padding: 14px 18px 12px;
    background:
      linear-gradient(180deg, rgba(255,255,255,0.14), rgba(255,255,255,0.035)),
      rgba(14, 16, 22, 0.72);
    border: 1px solid rgba(255,255,255,0.12);
    border-radius: 30px;
    backdrop-filter: blur(20px);
    box-shadow:
      0 20px 50px rgba(0,0,0,.58),
      0 1px 0 rgba(255,255,255,.2) inset,
      0 -1px 0 rgba(0,0,0,.45) inset;
    pointer-events: all;
  }

  .dock-item-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .dock-label {
    opacity: 0;
    transform: translateY(6px);
    color: var(--text-2);
    font-size: 12px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    transition: opacity var(--duration-normal) var(--ease-smooth), transform var(--duration-normal) var(--ease-smooth);
  }

  .dock-item {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 72px;
    height: 72px;
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

  .dock-item-wrap:hover .dock-label,
  .dock-item-wrap:has(.dock-item.active) .dock-label {
    opacity: 1;
    transform: translateY(0);
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

  .dock.compact .dock-container {
    gap: 10px;
    padding: 10px 12px;
  }

  .dock.compact .dock-item {
    width: 58px;
    height: 58px;
    border-radius: 16px;
  }

  .dock.compact .dock-label {
    display: none;
  }

  .dock.autohide {
    opacity: 0;
    pointer-events: none;
    transform: translateX(-50%) translateY(80px);
  }

  .dock.autohide:hover {
    opacity: 1;
    pointer-events: all;
    transform: translateX(-50%) translateY(0);
  }

  @media (max-width: 640px) {
    .dock {
      bottom: 14px;
      width: calc(100vw - 20px);
    }

    .dock-container {
      gap: 8px;
      width: 100%;
      padding: 10px 12px;
      border-radius: 24px;
      justify-content: space-between;
    }

    .dock-item {
      width: 54px;
      height: 54px;
      border-radius: 14px;
    }

    .dock-label {
      display: none;
    }
  }
</style>
