<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import DeviceManager from './DeviceManager.svelte';
  import {
    currentView,
    moveDockApp,
    preferences,
    setAccent,
    setDefaultDriveView,
    setDensity,
    setDockMode,
    setLandingApp,
    setTheme,
    type AccentMode,
    type AppView,
    type DensityMode,
    type DockMode,
  } from '$lib/stores/ui';

  interface $$Props {
    isOpen?: boolean;
    onClose?: () => void;
    class?: string;
  }

  let {
    isOpen = false,
    onClose,
    class: className,
  } = $props<$$Props>();

  const dockModes: DockMode[] = ['always', 'autohide', 'compact'];
  const densityModes: DensityMode[] = ['comfortable', 'compact'];
  const accentModes: AccentMode[] = ['green', 'blue', 'violet'];
  const landingApps: AppView[] = ['home', 'drive', 'notes', 'workspace', 'admin'];

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && onClose) {
      onClose();
    }
  }
</script>

{#if isOpen}
  <div
    class="tray-backdrop"
    role="presentation"
    tabindex="-1"
    onclick={handleBackdropClick}
    onkeydown={(e) => {
      if (e.key === 'Escape' && onClose) onClose();
    }}
  >
    <div class="tray {className || ''}">
      <div class="tray-header">
        <div>
          <span class="eyebrow">Settings</span>
          <h3 class="tray-title">Shell preferences</h3>
        </div>
        {#if onClose}
          <button type="button" class="tray-close" onclick={onClose} aria-label="Close">
            <Icons.X size={18} />
          </button>
        {/if}
      </div>

      <div class="tray-content">
        <section class="setting-section">
          <header>
            <span class="eyebrow">Appearance</span>
            <h4>Theme and density</h4>
          </header>

          <label class="setting-item">
            <div class="setting-info">
              <div class="setting-label">Theme</div>
              <div class="setting-desc">Global chrome and surface colors</div>
            </div>
            <select class="setting-select" value={$preferences.theme} onchange={(event) => setTheme((event.currentTarget as HTMLSelectElement).value as 'dark' | 'light')}>
              <option value="dark">Dark</option>
              <option value="light">Light</option>
            </select>
          </label>

          <div class="setting-item stacked">
            <div class="setting-info">
              <div class="setting-label">Accent</div>
              <div class="setting-desc">Used for active states and status glow</div>
            </div>
            <div class="choice-row">
              {#each accentModes as accent}
                <button type="button" class="choice-chip" class:active={$preferences.accent === accent} onclick={() => setAccent(accent)}>
                  {accent}
                </button>
              {/each}
            </div>
          </div>

          <div class="setting-item stacked">
            <div class="setting-info">
              <div class="setting-label">Density</div>
              <div class="setting-desc">Controls spacing across rails and cards</div>
            </div>
            <div class="choice-row">
              {#each densityModes as density}
                <button type="button" class="choice-chip" class:active={$preferences.density === density} onclick={() => setDensity(density)}>
                  {density}
                </button>
              {/each}
            </div>
          </div>
        </section>

        <section class="setting-section">
          <header>
            <span class="eyebrow">Dock</span>
            <h4>Launcher behavior</h4>
          </header>

          <label class="setting-item">
            <div class="setting-info">
              <div class="setting-label">Visibility</div>
              <div class="setting-desc">Always visible, compact, or auto-hide</div>
            </div>
            <select class="setting-select" value={$preferences.dockMode} onchange={(event) => setDockMode((event.currentTarget as HTMLSelectElement).value as DockMode)}>
              {#each dockModes as mode}
                <option value={mode}>{mode}</option>
              {/each}
            </select>
          </label>

          <div class="setting-item stacked">
            <div class="setting-info">
              <div class="setting-label">App order</div>
              <div class="setting-desc">Reorder dock apps without changing launcher</div>
            </div>

            <div class="order-list">
              {#each $preferences.dockOrder as appId, index (appId)}
                <div class="order-row">
                  <span class="order-name">{index + 1}. {appId}</span>
                  <div class="order-actions">
                    <button type="button" class="mini-btn" onclick={() => moveDockApp(appId, -1)} disabled={index === 0}>
                      <Icons.ChevronUp size={14} />
                    </button>
                    <button type="button" class="mini-btn" onclick={() => moveDockApp(appId, 1)} disabled={index === $preferences.dockOrder.length - 1}>
                      <Icons.ChevronDown size={14} />
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        </section>

        <section class="setting-section">
          <header>
            <span class="eyebrow">Defaults</span>
            <h4>Startup behavior</h4>
          </header>

          <label class="setting-item">
            <div class="setting-info">
              <div class="setting-label">Landing app</div>
              <div class="setting-desc">Where the shell opens after auth</div>
            </div>
            <select class="setting-select" value={$preferences.landingApp} onchange={(event) => setLandingApp((event.currentTarget as HTMLSelectElement).value as AppView)}>
              {#each landingApps as appId}
                <option value={appId}>{appId}</option>
              {/each}
            </select>
          </label>

          <label class="setting-item">
            <div class="setting-info">
              <div class="setting-label">Default Drive view</div>
              <div class="setting-desc">Applied when Drive opens without prior state</div>
            </div>
            <select class="setting-select" value={$preferences.defaultDriveView} onchange={(event) => setDefaultDriveView((event.currentTarget as HTMLSelectElement).value as 'grid' | 'list')}>
              <option value="grid">Grid</option>
              <option value="list">List</option>
            </select>
          </label>

          <div class="setting-item">
            <div class="setting-info">
              <div class="setting-label">Current app</div>
              <div class="setting-desc">Live shell state</div>
            </div>
            <span class="state-pill">{$currentView}</span>
          </div>
        </section>

        <div class="device-section">
          <DeviceManager />
        </div>
      </div>

      <div class="tray-footer">
        <span class="tray-link muted" aria-label="Preferences are stored in the current browser for now">
          <Icons.Settings size={14} />
          Local preferences until server-backed settings land
        </span>
      </div>
    </div>
  </div>
{/if}

<style>
  .tray-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.42);
    z-index: 998;
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

  .tray {
    position: fixed;
    top: 0;
    right: 0;
    width: min(460px, 100%);
    height: 100vh;
    background:
      linear-gradient(180deg, rgba(20, 22, 29, 0.98), rgba(10, 11, 16, 0.98)),
      radial-gradient(circle at 0% 0%, rgba(91, 227, 154, 0.08), transparent 42%);
    border-left: 1px solid rgba(255, 255, 255, 0.08);
    display: flex;
    flex-direction: column;
    animation: slideInRight var(--duration-normal) var(--ease-smooth);
  }

  @keyframes slideInRight {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }

  .tray-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    padding: 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;
  }

  .eyebrow {
    display: inline-block;
    font-family: var(--ff-mono);
    font-size: 12px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .tray-title {
    margin: 6px 0 0;
    font-size: 26px;
    font-weight: 700;
    color: var(--text);
  }

  .tray-close {
    width: 40px;
    height: 40px;
    padding: 0;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.03);
    color: var(--text-2);
    border-radius: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .tray-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px 120px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .setting-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 18px;
    border-radius: 24px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  .setting-section header h4 {
    margin: 6px 0 0;
    font-size: 18px;
  }

  .setting-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 16px;
    border-radius: 18px;
    background: rgba(255, 255, 255, 0.03);
  }

  .setting-item.stacked {
    align-items: flex-start;
    flex-direction: column;
  }

  .setting-info {
    flex: 1;
  }

  .setting-label {
    font-size: 15px;
    font-weight: 600;
  }

  .setting-desc {
    margin-top: 4px;
    font-size: 13px;
    color: var(--muted);
  }

  .setting-select,
  .state-pill {
    min-width: 140px;
    padding: 10px 12px;
    border-radius: 14px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.05);
    color: var(--text);
  }

  .state-pill {
    display: inline-flex;
    justify-content: center;
    text-transform: capitalize;
  }

  .choice-row,
  .order-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .choice-chip,
  .mini-btn {
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-2);
  }

  .choice-chip {
    padding: 8px 12px;
    border-radius: 14px;
    text-transform: capitalize;
  }

  .choice-chip.active {
    border-color: rgba(91, 227, 154, 0.22);
    background: rgba(91, 227, 154, 0.12);
    color: var(--text);
  }

  .order-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: 100%;
  }

  .order-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 14px;
    border-radius: 16px;
    background: rgba(255, 255, 255, 0.03);
  }

  .order-name {
    text-transform: capitalize;
  }

  .mini-btn {
    width: 36px;
    height: 36px;
    border-radius: 12px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .mini-btn:disabled {
    opacity: 0.35;
  }

  .device-section {
    padding-top: 6px;
  }

  .tray-footer {
    padding: 20px 24px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(255, 255, 255, 0.02);
    flex-shrink: 0;
  }

  .tray-link {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--muted);
    font-size: 13px;
  }
</style>
