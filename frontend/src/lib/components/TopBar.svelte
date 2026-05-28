<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { AppView } from '$lib/stores/ui';

  interface $$Props {
    currentView: AppView;
    title: string;
    crumbs?: string[];
    userName?: string;
    role?: string;
    dockMode?: string;
    onHome?: () => void;
    onCommandPalette?: () => void;
    onUpload?: () => void;
    onSettings?: () => void;
    onLogout?: () => void;
    class?: string;
  }

  let {
    currentView,
    title,
    crumbs = [],
    userName = 'local',
    role = 'user',
    dockMode = 'always',
    onHome,
    onCommandPalette,
    onUpload,
    onSettings,
    onLogout,
    class: className,
  }: $$Props = $props();

  let menuOpen = $state(false);
</script>

<header class="topbar {className || ''}">
  <div class="brand-strip">
    <div class="crumbs" aria-label="Current app">
      {#each crumbs as crumb, index (crumb)}
        {#if index > 0}
          <Icons.ChevronRight size={16} />
        {/if}
        <span class:active={index === crumbs.length - 1}>{crumb}</span>
      {/each}
    </div>
  </div>

  <button
    type="button"
    class="search-surface"
    onclick={onCommandPalette}
    aria-label="Open command palette"
  >
    <div class="search-copy">
      <Icons.Search size={20} />
      <span>Search files, notes, workspaces, commands...</span>
    </div>
    <span class="shortcut-pill">⌘K</span>
  </button>

  <div class="topbar-actions">
    {#if currentView === 'drive'}
      <button type="button" class="action-btn upload-btn" onclick={onUpload} title="Upload files to Drive">
        <Icons.Upload size={16} />
        <span>Upload</span>
      </button>
    {/if}

    <div class="account">
      <button
        type="button"
        class="profile-chip"
        class:open={menuOpen}
        onclick={() => (menuOpen = !menuOpen)}
        aria-haspopup="menu"
        aria-expanded={menuOpen}
        title="Account"
      >
        <div class="avatar-orb">
          {userName.slice(0, 1).toUpperCase()}
          <span class="presence-dot" title="Running locally"></span>
        </div>
        <div class="profile-copy">
          <strong>{userName}</strong>
          <span>{role === 'admin' ? 'Administrator' : 'Local account'}</span>
        </div>
        <Icons.ChevronDown size={15} class="chev" />
      </button>

      {#if menuOpen}
        <button type="button" class="menu-scrim" aria-label="Close menu" onclick={() => (menuOpen = false)}></button>
        <div class="account-menu" role="menu">
          <div class="menu-id">
            <div class="avatar-orb sm">{userName.slice(0, 1).toUpperCase()}</div>
            <div>
              <strong>{userName}</strong>
              <span>{role === 'admin' ? 'Administrator' : 'Local account'}</span>
            </div>
          </div>
          <div class="menu-sep"></div>
          <button type="button" class="menu-item" role="menuitem" onclick={() => { menuOpen = false; onSettings?.(); }}>
            <Icons.Settings2 size={16} />
            <span>Settings</span>
          </button>
          <button type="button" class="menu-item" role="menuitem" onclick={() => { menuOpen = false; onCommandPalette?.(); }}>
            <Icons.Command size={16} />
            <span>Command palette</span>
            <kbd>⌘K</kbd>
          </button>
          <div class="menu-sep"></div>
          <button type="button" class="menu-item danger" role="menuitem" onclick={() => { menuOpen = false; onLogout?.(); }}>
            <Icons.LogOut size={16} />
            <span>Sign out</span>
          </button>
        </div>
      {/if}
    </div>
  </div>
</header>

<style>
  .topbar {
    position: sticky;
    top: 0;
    z-index: 120;
    display: grid;
    grid-template-columns: auto minmax(280px, 1fr) auto;
    align-items: center;
    gap: var(--s-4);
    padding: 10px 20px;
    height: 56px;
    background: color-mix(in srgb, var(--bg) 86%, transparent);
    border-bottom: 1px solid var(--hairline);
    backdrop-filter: blur(18px);
  }

  .brand-strip {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
  }

  .action-btn,
  .icon-btn,
  .profile-chip,
  .search-surface {
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.02);
    color: var(--text);
    transition: transform var(--duration-normal) var(--ease-smooth), border-color var(--duration-normal) var(--ease-smooth), background var(--duration-normal) var(--ease-smooth);
  }

  .crumbs {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    color: var(--muted);
    font-size: 15px;
    font-weight: 500;
  }

  .crumbs span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .crumbs span.active {
    color: var(--text);
  }

  .search-surface {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    width: 100%;
    height: 48px;
    padding: 0 16px;
    border-radius: 20px;
    text-align: left;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
  }

  .search-copy {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    color: var(--muted);
    font-size: clamp(14px, 1.2vw, 16px);
  }

  .search-copy span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .shortcut-pill,
  .action-btn,
  .icon-btn,
  .profile-chip {
    border-radius: 14px;
    height: 38px;
  }

  .shortcut-pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 56px;
    padding: 8px 14px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-2);
    font-family: var(--ff-mono);
    font-size: 13px;
  }

  .topbar-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }

  .action-btn,
  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 0 14px;
    box-shadow: var(--shadow-card);
  }

  .upload-btn {
    background: var(--accent);
    border-color: transparent;
    color: #fff;
    font-size: 13px;
    font-weight: 600;
  }

  .upload-btn:hover {
    background: var(--accent-hover);
    border-color: transparent;
  }

  .icon-btn {
    width: 40px;
    padding: 0;
  }

  .account {
    position: relative;
  }

  .profile-chip {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 4px 10px 4px 8px;
    box-shadow: var(--shadow-card);
  }

  .profile-chip :global(.chev) {
    color: var(--muted);
    transition: transform 0.15s;
  }

  .profile-chip.open :global(.chev) {
    transform: rotate(180deg);
  }

  .menu-scrim {
    position: fixed;
    inset: 0;
    z-index: 130;
    border: none;
    background: transparent;
    cursor: default;
  }

  .account-menu {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    z-index: 131;
    width: 248px;
    padding: 6px;
    border-radius: var(--r-5);
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    box-shadow: var(--shadow-pop);
    animation: menu-in 0.12s var(--ease-smooth);
  }

  @keyframes menu-in {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .menu-id {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
  }

  .menu-id strong {
    display: block;
    font-size: 13px;
    color: var(--text);
  }

  .menu-id span {
    font-size: 11px;
    color: var(--muted);
  }

  .avatar-orb.sm {
    width: 34px;
    height: 34px;
  }

  .menu-sep {
    height: 1px;
    margin: 6px 4px;
    background: var(--hairline);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 9px 10px;
    border: none;
    background: transparent;
    color: var(--text-2);
    border-radius: var(--r-3);
    cursor: pointer;
    font-size: 13px;
    text-align: left;
    transition: background 0.12s, color 0.12s;
  }

  .menu-item span {
    flex: 1;
  }

  .menu-item kbd {
    font-family: var(--ff-mono);
    font-size: 10px;
    color: var(--muted);
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 5px;
  }

  .menu-item:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .menu-item.danger:hover {
    background: var(--danger-soft);
    color: var(--danger);
  }

  .avatar-orb {
    position: relative;
    width: 30px;
    height: 30px;
    border-radius: 999px;
    display: grid;
    place-items: center;
    background: linear-gradient(135deg, rgba(91, 227, 154, 0.75), rgba(255, 95, 162, 0.72));
    color: #08110d;
    font-weight: 700;
    font-size: 13px;
    flex-shrink: 0;
  }

  .presence-dot {
    position: absolute;
    right: -1px;
    bottom: -1px;
    width: 9px;
    height: 9px;
    border-radius: 999px;
    background: var(--green);
    border: 2px solid #0a0b10;
    box-shadow: 0 0 8px rgba(91, 227, 154, 0.6);
  }

  .profile-copy {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    min-width: 0;
  }

  .profile-copy strong,
  .profile-copy span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .profile-copy strong {
    font-size: 13px;
  }

  .profile-copy span {
    color: var(--muted);
    font-size: 11px;
  }

  .search-surface:hover,
  .action-btn:hover,
  .icon-btn:hover,
  .profile-chip:hover {
    transform: translateY(-1px);
    border-color: rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.05);
  }

  @media (max-width: 1280px) {
    .topbar {
      grid-template-columns: auto 1fr;
      grid-template-areas:
        "brand actions"
        "search search";
    }

    .brand-strip {
      grid-area: brand;
    }

    .search-surface {
      grid-area: search;
    }

    .topbar-actions {
      grid-area: actions;
    }

    .crumbs {
      display: none;
    }
  }

  @media (max-width: 900px) {
    .topbar {
      gap: 10px;
      padding: 10px 16px;
      height: 56px;
    }

    .brand-mark {
      width: 40px;
      height: 40px;
      border-radius: 10px;
    }

    .brand-wordmark {
      font-size: 22px;
    }

    .search-surface {
      height: 40px;
      padding: 0 12px;
      border-radius: 18px;
    }

    .search-copy {
      font-size: 13px;
      gap: 8px;
    }

    .shortcut-pill,
    .action-btn,
    .icon-btn,
    .profile-chip {
      height: 36px;
    }

    .upload-btn span,
    .profile-copy {
      display: none;
    }

    .action-btn,
    .icon-btn {
      width: 36px;
      padding: 0;
    }

    .profile-chip {
      padding: 2px;
    }

    .avatar-orb {
      width: 28px;
      height: 28px;
      font-size: 12px;
    }
  }
</style>
