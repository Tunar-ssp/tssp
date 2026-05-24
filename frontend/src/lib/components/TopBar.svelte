<script lang="ts">
  import { user } from '$lib/stores/auth';
  import { commandPaletteOpen } from '$lib/stores/ui';
  import Button from './Button.svelte';
  import Pill from './Pill.svelte';
  import { Search, Bell, Upload } from 'lucide-svelte';

  export let context = '';
</script>

<header class="topbar">
  <div class="topbar-start">
    <div class="logo">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
        <circle cx="12" cy="12" r="10" opacity="0.5" />
        <circle cx="12" cy="12" r="5" />
      </svg>
      <span class="brand">tssp</span>
      {#if context}
        <span class="sep">/</span>
        <span class="context">{context}</span>
      {/if}
    </div>
  </div>

  <div class="topbar-center">
    <div class="search-bar">
      <Search size={14} />
      <input type="text" placeholder="Search files, notes, workspaces…" />
      <kbd>⌘K</kbd>
    </div>
  </div>

  <div class="topbar-end">
    <Button kind="solid" size="sm" icon={Upload}>Upload</Button>
    <button class="icon-btn">
      <Bell size={14} />
      <span class="notify-dot"></span>
    </button>
    {#if $user}
      <div class="user-pill">
        <div class="avatar"></div>
        <span>{$user.name}</span>
      </div>
    {/if}
  </div>
</header>

<style>
  .topbar {
    height: 52px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 16px;
    border-bottom: 1px solid var(--hairline);
    background: linear-gradient(180deg, rgba(20,22,29,.85) 0%, rgba(14,15,18,.7) 100%);
    backdrop-filter: blur(20px);
    position: relative;
    z-index: 5;
  }

  .topbar-start {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 220px;
  }

  .topbar-center {
    flex: 1;
    display: flex;
    justify-content: center;
  }

  .topbar-end {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 220px;
    justify-content: flex-end;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
  }

  .brand {
    font-size: 22px;
    font-weight: 700;
    color: var(--text);
  }

  .sep {
    color: var(--faint);
    font-size: 12px;
  }

  .context {
    color: var(--text-2);
    font-weight: 500;
  }

  .search-bar {
    width: 460px;
    height: 32px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 10px 0 12px;
    border-radius: 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--muted);
    font-size: 13px;
  }

  .search-bar input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text);
    outline: none;
    font-family: inherit;
  }

  .search-bar input::placeholder {
    color: var(--muted);
  }

  .search-bar kbd {
    font-size: 11px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 5px;
    padding: 1px 6px;
    color: var(--text-2);
  }

  .icon-btn {
    width: 30px;
    height: 30px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    cursor: pointer;
    transition: all 0.15s;
  }

  .icon-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .notify-dot {
    position: absolute;
    top: 5px;
    right: 5px;
    width: 6px;
    height: 6px;
    border-radius: 3px;
    background: var(--pink);
  }

  .user-pill {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px 4px 4px;
    border-radius: 999px;
    background: var(--surface);
    border: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-2);
  }

  .avatar {
    width: 22px;
    height: 22px;
    border-radius: 999px;
    background: linear-gradient(135deg, var(--green), var(--pink));
  }
</style>
