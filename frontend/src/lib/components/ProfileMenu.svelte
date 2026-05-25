<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { user } from '$lib/stores/auth';
  import { currentView } from '$lib/stores/ui';

  interface $$Props {
    isOpen?: boolean;
    onClose?: () => void;
    onLogout?: () => void;
    class?: string;
  }

  let {
    isOpen = false,
    onClose,
    onLogout,
    class: className,
  } = $props<$$Props>();

  async function handleLogout() {
    if (onLogout) {
      await onLogout();
    } else {
      try {
        await fetch('/api/auth/logout', { method: 'POST' });
        user.set(null);
        currentView.set('home');
        if (onClose) onClose();
      } catch (e) {
        console.error('Logout failed', e);
      }
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && onClose) {
      onClose();
    }
  }
</script>

{#if isOpen && $user}
  <div class="menu-backdrop" on:click={handleBackdropClick}>
    <div class="profile-menu {className || ''}">
      <div class="menu-header">
        <div class="user-avatar">
          {$user.email.charAt(0).toUpperCase()}
        </div>
        <div class="user-info">
          <div class="user-email">{$user.email}</div>
          <div class="user-role">{$user.role === 'admin' ? '👑 Admin' : 'User'}</div>
        </div>
      </div>

      <div class="menu-divider"></div>

      <div class="menu-items">
        <a href="/profile" class="menu-item" on:click={onClose}>
          <Icons.User size={16} />
          <span>Profile</span>
        </a>
        <a href="/devices" class="menu-item" on:click={onClose}>
          <Icons.Smartphone size={16} />
          <span>Trusted Devices</span>
        </a>
        {#if $user.role === 'admin'}
          <a href="/admin" class="menu-item admin" on:click={onClose}>
            <Icons.Settings size={16} />
            <span>Admin Panel</span>
          </a>
        {/if}
      </div>

      <div class="menu-divider"></div>

      <button class="menu-item danger" on:click={handleLogout}>
        <Icons.LogOut size={16} />
        <span>Sign Out</span>
      </button>
    </div>
  </div>
{/if}

<style>
  .menu-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 997;
    animation: fadeIn var(--duration-quick) var(--ease-smooth);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .profile-menu {
    position: fixed;
    top: 56px;
    right: 0;
    width: 100%;
    max-width: 300px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-top: none;
    border-radius: 0 0 var(--r-2) 0;
    box-shadow: var(--shadow-card);
    z-index: 998;
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

  .menu-header {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-4);
  }

  .user-avatar {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    background: var(--blue-subtle);
    border-radius: var(--r-2);
    color: var(--blue);
    font-weight: 600;
    font-size: var(--fs-16);
    flex-shrink: 0;
  }

  .user-info {
    flex: 1;
    min-width: 0;
  }

  .user-email {
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .user-role {
    font-size: var(--fs-11);
    color: var(--muted);
    margin-top: 2px;
  }

  .menu-divider {
    height: 1px;
    background: var(--border);
    margin: var(--s-2) 0;
  }

  .menu-items {
    display: flex;
    flex-direction: column;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-3) var(--s-4);
    background: none;
    border: none;
    color: var(--text-2);
    cursor: pointer;
    font-size: var(--fs-13);
    text-decoration: none;
    transition: all var(--duration-quick) var(--ease-smooth);
    font-family: var(--ff-sans);
  }

  .menu-item:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .menu-item.admin {
    color: var(--orange);
  }

  .menu-item.admin:hover {
    background: rgba(255, 138, 61, 0.1);
  }

  .menu-item.danger {
    color: var(--danger);
  }

  .menu-item.danger:hover {
    background: rgba(255, 107, 107, 0.1);
  }
</style>
