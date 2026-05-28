<script lang="ts">
  interface DockItem {
    id: string;
    label: string;
    shortcut?: string;
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
<div class="dock-zone {className || ''}" class:autohide={mode === 'autohide'} class:compact={mode === 'compact'}>
  {#if mode === 'autohide'}
    <div class="dock-hotzone" aria-hidden="true"></div>
    <div class="dock-handle" aria-hidden="true"></div>
  {/if}
  <nav class="dock" aria-label="Application dock">
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
</div>
{/if}

<style>
  .dock-zone {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 140;
    display: flex;
    justify-content: center;
    align-items: flex-end;
    padding-bottom: 22px;
    pointer-events: none;
  }

  /* Invisible strip along the very bottom edge that reveals an autohidden dock */
  .dock-hotzone {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 22px;
    pointer-events: all;
  }

  /* Subtle peek indicator so the hidden dock stays discoverable */
  .dock-handle {
    position: absolute;
    bottom: 6px;
    left: 50%;
    width: 140px;
    height: 5px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.16);
    transform: translateX(-50%);
    transition: opacity var(--duration-normal) var(--ease-smooth);
    pointer-events: none;
  }

  .dock {
    position: relative;
    pointer-events: none;
    transition: transform var(--duration-normal) var(--ease-smooth), opacity var(--duration-normal) var(--ease-smooth);
  }

  .dock-glow {
    display: none;
  }

  .dock-container {
    display: flex;
    align-items: flex-end;
    justify-content: center;
    gap: 12px;
    padding: 10px 12px;
    background: var(--dock-glass);
    border: 1px solid var(--dock-edge);
    border-radius: 18px;
    backdrop-filter: blur(20px);
    box-shadow: var(--shadow-dock);
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
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .dock-shortcut {
    font-family: var(--ff-mono);
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(255,255,255,0.08);
    color: var(--text-2);
    border: 1px solid rgba(255,255,255,0.06);
    text-transform: none;
    letter-spacing: 0;
  }

  .dock-item {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 58px;
    height: 58px;
    padding: 0;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    border-radius: 15px;
    cursor: pointer;
    position: relative;
    transition: transform var(--duration-quick) var(--ease-smooth),
      background var(--duration-quick) var(--ease-smooth),
      border-color var(--duration-quick) var(--ease-smooth),
      color var(--duration-quick) var(--ease-smooth);
  }

  .dock-item:hover {
    color: var(--text);
    background: var(--surface-3);
    border-color: var(--border-2);
    transform: translateY(-6px);
  }

  .dock-item.hovered,
  .dock-item.active {
    color: color-mix(in srgb, var(--item-accent, var(--accent)) 88%, white);
    background: color-mix(in srgb, var(--item-accent, var(--accent)) 16%, var(--surface-2));
    border-color: color-mix(in srgb, var(--item-accent, var(--accent)) 40%, transparent);
  }

  .dock-item-wrap:hover .dock-label,
  .dock-item-wrap:has(.dock-item.active) .dock-label {
    opacity: 1;
    transform: translateY(0);
  }

  .dock-item.active::after {
    content: "";
    position: absolute;
    bottom: -6px;
    width: 5px;
    height: 5px;
    border-radius: 999px;
    background: var(--item-accent, var(--accent));
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

  .dock-zone.compact .dock-container {
    gap: 10px;
    padding: 10px 12px;
  }

  .dock-zone.compact .dock-item {
    width: 58px;
    height: 58px;
    border-radius: 16px;
  }

  .dock-zone.compact .dock-label {
    display: none;
  }

  /* Autohide: dock slides out of view, a slim handle hints it is there */
  .dock-zone.autohide .dock {
    opacity: 0;
    transform: translateY(calc(100% + 30px));
  }

  .dock-zone.autohide:hover .dock {
    opacity: 1;
    transform: translateY(0);
    pointer-events: all;
  }

  .dock-zone.autohide:hover .dock-container {
    pointer-events: all;
  }

  .dock-zone.autohide:hover .dock-handle {
    opacity: 0;
  }

  @media (max-width: 640px) {
    .dock-zone {
      padding-bottom: 14px;
    }

    .dock {
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
