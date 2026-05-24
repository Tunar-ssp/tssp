<script lang="ts">
  import type { AppId } from "../router";
  import { apps, navigateApp } from "../router";

  export let app: AppId;
  export let mobileOpen = false;
  export let onCloseMobile: () => void;
</script>

<aside class="sidebar" class:open={mobileOpen}>
  <div class="sidebar-inner">
    <div class="sidebar-brand">
      <div class="brand-icon" aria-hidden="true"></div>
      <div>
        <strong>TSSP</strong>
        <span>Local Cloud OS</span>
      </div>
    </div>

    <nav class="sidebar-nav" aria-label="Applications">
      {#each apps as item}
        <button
          type="button"
          class="nav-item"
          class:active={app === item.id}
          on:click={() => {
            navigateApp(item.id);
            onCloseMobile();
          }}
        >
          <span class="nav-item-label">{item.label}</span>
          <span class="nav-item-sub">{item.subtitle}</span>
        </button>
      {/each}
    </nav>
  </div>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    border-right: 1px solid var(--border);
    background: var(--bg-elevated);
    flex-shrink: 0;
  }

  .sidebar-inner {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 16px 12px;
    height: 100%;
  }

  .sidebar-brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 8px 12px;
    border-bottom: 1px solid var(--border);
  }

  .sidebar-brand strong {
    display: block;
    font-size: 14px;
    font-weight: 600;
  }

  .sidebar-brand span {
    display: block;
    font-size: 11px;
    color: var(--text-muted);
  }

  .brand-icon {
    width: 28px;
    height: 28px;
    border-radius: 7px;
    background: var(--brand);
    box-shadow: var(--shadow-panel);
  }

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .nav-item {
    border: 1px solid transparent;
    background: transparent;
    color: var(--text);
    text-align: left;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    display: grid;
    gap: 2px;
  }

  .nav-item:hover {
    background: var(--bg-hover);
  }

  .nav-item.active {
    background: var(--brand-dim);
    border-color: rgba(37, 99, 235, 0.35);
  }

  .nav-item-label {
    font-size: 13px;
    font-weight: 600;
  }

  .nav-item-sub {
    font-size: 11px;
    color: var(--text-muted);
  }

  @media (max-width: 900px) {
    .sidebar {
      position: fixed;
      inset: 0 auto 0 0;
      z-index: 40;
      transform: translateX(-100%);
      transition: transform 0.2s ease;
    }

    .sidebar.open {
      transform: translateX(0);
    }
  }
</style>
