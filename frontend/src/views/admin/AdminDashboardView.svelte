<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { api, type AdminUser, type AdminSession, type AdminActivityItem } from '$lib/api';
  import { isAdmin } from '$lib/stores/auth';
  import { loadDataWithHandler } from '$lib/utils/apiHelpers';
  import Card from '$lib/components/Card.svelte';
  import TabPanel from '$lib/components/TabPanel.svelte';
  import AdminStats from './components/AdminStats.svelte';
  import AdminUsersPanel from './components/AdminUsersPanel.svelte';
  import AdminSessionsPanel from './components/AdminSessionsPanel.svelte';
  import AdminActivityPanel from './components/AdminActivityPanel.svelte';

  interface SystemStats {
    fileCount: number;
    noteCount: number;
    workspaceCount: number;
    usedStorage: number;
    uptime: number;
    corruptFileCount: number;
  }

  let stats = $state<SystemStats>({
    fileCount: 0,
    noteCount: 0,
    workspaceCount: 0,
    usedStorage: 0,
    uptime: 0,
    corruptFileCount: 0,
  });

  let users = $state<AdminUser[]>([]);
  let sessions = $state<AdminSession[]>([]);
  let activities = $state<AdminActivityItem[]>([]);
  let isLoading = $state(false);
  let selectedTab = $state<'dashboard' | 'users' | 'sessions' | 'logs'>('dashboard');

  const tabs = $derived([
    {
      id: 'dashboard',
      label: 'Dashboard',
      icon: Icons.BarChart3,
    },
    {
      id: 'users',
      label: 'Users',
      icon: Icons.Users,
    },
    {
      id: 'sessions',
      label: 'Sessions',
      icon: Icons.Smartphone,
    },
    {
      id: 'logs',
      label: 'Activity Logs',
      icon: Icons.FileText,
    },
  ]);

  async function loadStats() {
    if (!$isAdmin) return;
    const result = await loadDataWithHandler(
      () => api.getAdminOverview(),
      (data) => {
        stats = {
          fileCount: data.repository.file_count || 0,
          noteCount: data.repository.note_count || 0,
          workspaceCount: 0,
          usedStorage: data.repository.storage_bytes_used || 0,
          uptime: data.system.uptime_seconds || 0,
          corruptFileCount: 0,
        };
      },
      { errorMessage: 'Failed to load stats' }
    );
    return result !== null;
  }

  async function loadUsers() {
    if (!$isAdmin) return;
    const result = await loadDataWithHandler(
      () => api.listAdminUsers(),
      (data) => {
        users = data.users || [];
      },
      { errorMessage: 'Failed to load users' }
    );
    return result !== null;
  }

  async function loadSessions() {
    if (!$isAdmin) return;
    const result = await loadDataWithHandler(
      () => api.listAdminSessions(100),
      (data) => {
        sessions = data.sessions || [];
      },
      { errorMessage: 'Failed to load sessions' }
    );
    return result !== null;
  }

  async function loadActivity() {
    if (!$isAdmin) return;
    const result = await loadDataWithHandler(
      () => api.listAdminActivity(100),
      (data) => {
        activities = data.items || [];
      },
      { errorMessage: 'Failed to load activity' }
    );
    return result !== null;
  }

  function handleTabChange(tabId: string) {
    selectedTab = tabId as any;
    switch (tabId) {
      case 'dashboard':
        loadStats();
        break;
      case 'users':
        loadUsers();
        break;
      case 'sessions':
        loadSessions();
        break;
      case 'logs':
        loadActivity();
        break;
    }
  }

  $effect(() => {
    if ($isAdmin) {
      handleTabChange(selectedTab);
    }
  });
</script>

{#if !$isAdmin}
  <div class="not-admin">
    <Icons.Lock size={48} />
    <h3>Admin Access Required</h3>
    <p>Only administrators can view the admin dashboard</p>
  </div>
{:else}
  <div class="admin-dashboard">
    <div class="dashboard-header">
      <h1>Admin Dashboard</h1>
      <p>System overview and management</p>
    </div>

    <TabPanel {tabs} activeTab={selectedTab} onTabChange={handleTabChange}>
      {#if selectedTab === 'dashboard'}
        <AdminStats {...stats} />
      {:else if selectedTab === 'users'}
        <AdminUsersPanel {users} {isLoading} onReload={loadUsers} />
      {:else if selectedTab === 'sessions'}
        <AdminSessionsPanel {sessions} {isLoading} />
      {:else if selectedTab === 'logs'}
        <AdminActivityPanel {activities} {isLoading} />
      {/if}
    </TabPanel>
  </div>
{/if}

<style>
  .not-admin {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    color: var(--muted);
  }

  .not-admin h3 {
    margin: 0;
    color: var(--text-2);
  }

  .not-admin p {
    margin: 0;
  }

  .admin-dashboard {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .dashboard-header {
    padding: var(--s-6);
    border-bottom: 1px solid var(--border);
  }

  .dashboard-header h1 {
    margin: 0;
    font-size: var(--fs-32);
    font-weight: 700;
    color: var(--text);
  }

  .dashboard-header p {
    margin: var(--s-2) 0 0;
    font-size: var(--fs-13);
    color: var(--muted);
  }

  :global(.tab-panel) {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: var(--s-6);
    overflow-y: auto;
  }

  :global(.tab-panel .tab-buttons) {
    border-bottom: 1px solid var(--border);
    margin: calc(-1 * var(--s-6)) calc(-1 * var(--s-6)) var(--s-6) calc(-1 * var(--s-6));
    padding: var(--s-4) var(--s-6);
  }

  :global(.admin-panel) {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
</style>
