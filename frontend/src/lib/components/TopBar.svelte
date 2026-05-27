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
    <span class="status-pill">
      <span class="status-dot"></span>
      Local
    </span>

    <button type="button" class="action-btn upload-btn" onclick={onUpload}>
      <Icons.Upload size={18} />
      <span>Upload</span>
    </button>

    <button type="button" class="profile-chip" onclick={onSettings} aria-label="Open settings">
      <div class="avatar-orb">{userName.slice(0, 1).toUpperCase()}</div>
      <div class="profile-copy">
        <strong>{userName}</strong>
        <span>{role === 'admin' ? 'Admin' : currentView === 'home' ? title : `${title} • ${dockMode}`}</span>
      </div>
    </button>

    <button type="button" class="icon-btn" onclick={onLogout} aria-label="Sign out">
      <Icons.LogOut size={18} />
    </button>
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
    padding: 12px 20px;
    height: 64px;
    background:
      linear-gradient(180deg, rgba(10, 11, 16, 0.96), rgba(10, 11, 16, 0.92)),
      radial-gradient(circle at 18% 0%, rgba(91, 227, 154, 0.08), transparent 38%),
      radial-gradient(circle at 84% 0%, rgba(255, 95, 162, 0.06), transparent 36%);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
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
    font-size: clamp(18px, 1.7vw, 28px);
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
  .status-pill,
  .action-btn,
  .icon-btn,
  .profile-chip {
    border-radius: 18px;
    height: 40px;
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

  .status-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 0 14px;
    background: rgba(91, 227, 154, 0.1);
    border: 1px solid rgba(91, 227, 154, 0.24);
    color: var(--green);
    font-size: 13px;
    font-weight: 600;
  }

  .status-dot {
    width: 14px;
    height: 14px;
    border-radius: 999px;
    background: currentColor;
    box-shadow: 0 0 16px rgba(91, 227, 154, 0.55);
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
    background: linear-gradient(135deg, rgba(110, 168, 255, 0.18), rgba(255, 255, 255, 0.04));
    font-size: 13px;
    font-weight: 600;
  }

  .icon-btn {
    width: 40px;
    padding: 0;
  }

  .profile-chip {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 4px 12px 4px 8px;
    box-shadow: var(--shadow-card);
  }

  .avatar-orb {
    width: 32px;
    height: 32px;
    border-radius: 999px;
    display: grid;
    place-items: center;
    background: linear-gradient(135deg, rgba(91, 227, 154, 0.75), rgba(255, 95, 162, 0.72));
    color: #08110d;
    font-weight: 700;
    font-size: 13px;
    flex-shrink: 0;
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
    .status-pill,
    .action-btn,
    .icon-btn,
    .profile-chip {
      height: 36px;
    }

    .status-pill,
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
