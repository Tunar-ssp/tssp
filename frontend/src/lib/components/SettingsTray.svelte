<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import DeviceManager from './DeviceManager.svelte';

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

  let theme = $state<'dark' | 'light'>(
    typeof localStorage !== 'undefined'
      ? (localStorage.getItem('theme') as 'dark' | 'light') || 'dark'
      : 'dark'
  );

  function toggleTheme() {
    theme = theme === 'dark' ? 'light' : 'dark';
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('theme', theme);
    }
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', theme);
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && onClose) {
      onClose();
    }
  }
</script>

{#if isOpen}
  <div class="tray-backdrop" on:click={handleBackdropClick}>
    <div class="tray {className || ''}">
      <div class="tray-header">
        <h3 class="tray-title">Settings</h3>
        {#if onClose}
          <button class="tray-close" on:click={onClose} aria-label="Close">
            <Icons.X size={18} />
          </button>
        {/if}
      </div>

      <div class="tray-content">
        <div class="setting-group">
          <label class="setting-item">
            <div class="setting-info">
              <div class="setting-label">Theme</div>
              <div class="setting-desc">Dark • Light</div>
            </div>
            <button
              class="theme-toggle"
              on:click={toggleTheme}
              title="Toggle theme"
            >
              {#if theme === 'dark'}
                <Icons.Moon size={16} />
              {:else}
                <Icons.Sun size={16} />
              {/if}
            </button>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-item">
            <div class="setting-info">
              <div class="setting-label">View Mode</div>
              <div class="setting-desc">Compact • Default</div>
            </div>
            <select class="setting-select">
              <option>Default</option>
              <option>Compact</option>
            </select>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-item">
            <div class="setting-info">
              <div class="setting-label">Notifications</div>
              <div class="setting-desc">Enabled</div>
            </div>
            <input type="checkbox" class="setting-checkbox" checked />
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-item">
            <div class="setting-info">
              <div class="setting-label">Sound</div>
              <div class="setting-desc">Disabled</div>
            </div>
            <input type="checkbox" class="setting-checkbox" />
          </label>
        </div>

        <div class="device-section">
          <DeviceManager />
        </div>
      </div>

      <div class="tray-footer">
        <a href="/settings" class="tray-link">
          <Icons.Settings size={14} />
          Advanced Settings
        </a>
      </div>
    </div>
  </div>
{/if}

<style>
  .tray-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.4);
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
    top: 56px;
    right: 0;
    width: 100%;
    max-width: 360px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-top: none;
    max-height: calc(100vh - 56px);
    overflow-y: auto;
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
    align-items: center;
    justify-content: space-between;
    padding: var(--s-4);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .tray-title {
    margin: 0;
    font-size: var(--fs-16);
    font-weight: 600;
    color: var(--text);
  }

  .tray-close {
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-2);
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .tray-close:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .tray-content {
    flex: 1;
    padding: var(--s-4);
    display: flex;
    flex-direction: column;
    gap: var(--s-4);
  }

  .setting-group {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .setting-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-4);
    padding: var(--s-3);
    background: var(--surface-2);
    border-radius: var(--r-2);
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .setting-item:hover {
    background: var(--surface-3);
  }

  .setting-info {
    flex: 1;
  }

  .setting-label {
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text);
  }

  .setting-desc {
    font-size: var(--fs-11);
    color: var(--muted);
    margin-top: 2px;
  }

  .theme-toggle {
    width: 32px;
    height: 32px;
    padding: 0;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    border-radius: var(--r-2);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .theme-toggle:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .setting-select,
  .setting-checkbox {
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    border-radius: var(--r-2);
    font-family: var(--ff-sans);
    font-size: var(--fs-12);
  }

  .tray-footer {
    padding: var(--s-4);
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }

  .tray-link {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-3);
    color: var(--blue);
    text-decoration: none;
    font-size: var(--fs-13);
    border-radius: var(--r-2);
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .tray-link:hover {
    background: var(--surface-2);
  }

  .device-section {
    margin-top: var(--s-2);
    padding-top: var(--s-4);
    border-top: 1px solid var(--border);
  }
</style>
