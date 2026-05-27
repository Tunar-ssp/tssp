<script lang="ts">
  import * as Icons from 'lucide-svelte';

  type AdminSection = 'overview' | 'users' | 'sessions' | 'devices' | 'public' | 'activity' | 'maintenance';

  interface NavItem {
    id: AdminSection;
    label: string;
    icon: any;
  }

  interface NavGroup {
    label: string;
    items: NavItem[];
  }

  interface Props {
    activeSection?: AdminSection;
    navGroups?: NavGroup[];
    navCount?: (section: AdminSection) => number | null;
    onSectionChange?: (section: AdminSection) => void;
  }

  let {
    activeSection = 'overview',
    navGroups = [],
    navCount = () => null,
    onSectionChange,
  }: Props = $props();
</script>

<aside class="admin-sidebar">
  <div class="sidebar-title">
    <strong>Admin control</strong>
    <span>Real backend data only</span>
  </div>

  <nav class="admin-nav">
    {#each navGroups as group (group.label)}
      <div class="nav-group">
        <div class="nav-group-label">{group.label}</div>
        {#each group.items as item (item.id)}
          {@const Icon = item.icon}
          <button
            type="button"
            class="nav-item"
            class:active={activeSection === item.id}
            onclick={() => onSectionChange?.(item.id)}
          >
            <Icon size={14} />
            <span>{item.label}</span>
            {#if navCount(item.id) !== null}
              <small>{navCount(item.id)}</small>
            {/if}
          </button>
        {/each}
      </div>
    {/each}
  </nav>
</aside>

<style>
  .admin-sidebar {
    border-right: 1px solid var(--hairline);
    background: rgba(9, 10, 14, 0.78);
    padding: 16px 10px;
    display: flex;
    flex-direction: column;
    gap: 18px;
    overflow: auto;
  }

  .sidebar-title strong {
    display: block;
    color: var(--text);
    font-size: 15px;
  }

  .sidebar-title span {
    font-size: 12px;
    color: var(--muted);
  }

  .admin-nav {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .nav-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .nav-group-label {
    padding: 0 10px;
    font-size: 10px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--dim);
    font-family: var(--ff-mono);
  }

  .nav-item {
    min-height: 36px;
    padding: 0 10px;
    border: 1px solid transparent;
    border-radius: 10px;
    background: transparent;
    color: var(--text-2);
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-family: inherit;
  }

  .nav-item span {
    flex: 1;
    text-align: left;
  }

  .nav-item small {
    color: var(--dim);
    font-size: 11px;
    font-family: var(--ff-mono);
  }

  .nav-item:hover,
  .nav-item.active {
    border-color: var(--border);
    background: var(--surface);
    color: var(--text);
  }

  @media (max-width: 1180px) {
    .admin-sidebar {
      display: none;
    }
  }
</style>
