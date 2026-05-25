<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { isAdmin } from '$lib/stores/auth';
  import { error as showError } from '$lib/stores/notifications';
  import Btn from '$lib/components/Btn.svelte';
  import Card from '$lib/components/Card.svelte';
  import Bar from '$lib/components/Bar.svelte';
  import StatusDot from '$lib/components/StatusDot.svelte';

  interface SystemStats {
    totalUsers: number;
    activeUsers: number;
    totalStorage: number;
    usedStorage: number;
    uptime: number;
    databaseSize: number;
  }

  let stats = $state<SystemStats>({
    totalUsers: 0,
    activeUsers: 0,
    totalStorage: 0,
    usedStorage: 0,
    uptime: 0,
    databaseSize: 0,
  });

  let isLoading = $state(false);
  let selectedTab = $state<'dashboard' | 'users' | 'sessions' | 'logs'>('dashboard');

  async function loadStats() {
    if (!$isAdmin) return;
    isLoading = true;
    try {
      const response = await fetch('/api/admin/stats');
      if (!response.ok) throw new Error('Failed to load stats');
      stats = await response.json();
    } catch (e) {
      showError(e instanceof Error ? e.message : 'Failed to load stats');
    } finally {
      isLoading = false;
    }
  }

  function formatBytes(bytes: number) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }

  function formatUptime(seconds: number) {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${days}d ${hours}h ${minutes}m`;
  }

  $effect(() => {
    if ($isAdmin) {
      loadStats();
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

    <div class="dashboard-tabs">
      <button
        class="tab-btn"
        class:active={selectedTab === 'dashboard'}
        on:click={() => (selectedTab = 'dashboard')}
      >
        <Icons.BarChart3 size={16} />
        Dashboard
      </button>
      <button
        class="tab-btn"
        class:active={selectedTab === 'users'}
        on:click={() => (selectedTab = 'users')}
      >
        <Icons.Users size={16} />
        Users
      </button>
      <button
        class="tab-btn"
        class:active={selectedTab === 'sessions'}
        on:click={() => (selectedTab = 'sessions')}
      >
        <Icons.Smartphone size={16} />
        Sessions
      </button>
      <button
        class="tab-btn"
        class:active={selectedTab === 'logs'}
        on:click={() => (selectedTab = 'logs')}
      >
        <Icons.FileText size={16} />
        Activity Logs
      </button>
    </div>

    <div class="dashboard-content">
      {#if selectedTab === 'dashboard'}
        <div class="stats-grid">
          <Card>
            <div class="stat-card">
              <div class="stat-header">
                <h3>Total Users</h3>
                <Icons.Users size={20} />
              </div>
              <div class="stat-value">{stats.totalUsers}</div>
              <div class="stat-meta">{stats.activeUsers} active today</div>
            </div>
          </Card>

          <Card>
            <div class="stat-card">
              <div class="stat-header">
                <h3>Storage</h3>
                <Icons.HardDrive size={20} />
              </div>
              <div class="stat-value">{formatBytes(stats.usedStorage)}</div>
              <div class="stat-progress">
                <Bar
                  value={(stats.usedStorage / stats.totalStorage) * 100}
                  tone="ok"
                />
              </div>
              <div class="stat-meta">{formatBytes(stats.totalStorage)} total</div>
            </div>
          </Card>

          <Card>
            <div class="stat-card">
              <div class="stat-header">
                <h3>Uptime</h3>
                <Icons.Zap size={20} />
              </div>
              <div class="stat-value">{formatUptime(stats.uptime)}</div>
              <div class="stat-meta">
                <StatusDot tone="ok" />
                System healthy
              </div>
            </div>
          </Card>

          <Card>
            <div class="stat-card">
              <div class="stat-header">
                <h3>Database</h3>
                <Icons.Database size={20} />
              </div>
              <div class="stat-value">{formatBytes(stats.databaseSize)}</div>
              <div class="stat-meta">SQLite database size</div>
            </div>
          </Card>
        </div>

        <div class="actions-section">
          <Card>
            <div class="actions-header">
              <h3>Maintenance</h3>
            </div>
            <div class="actions-list">
              <button class="action-item">
                <Icons.Database size={16} />
                <div class="action-text">
                  <span class="action-name">Database Backup</span>
                  <span class="action-desc">Create a system backup</span>
                </div>
                <Icons.ChevronRight size={16} />
              </button>
              <button class="action-item">
                <Icons.CheckCircle2 size={16} />
                <div class="action-text">
                  <span class="action-name">Integrity Check</span>
                  <span class="action-desc">Verify system integrity</span>
                </div>
                <Icons.ChevronRight size={16} />
              </button>
              <button class="action-item">
                <Icons.RotateCw size={16} />
                <div class="action-text">
                  <span class="action-name">System Restart</span>
                  <span class="action-desc">Restart the service</span>
                </div>
                <Icons.ChevronRight size={16} />
              </button>
            </div>
          </Card>
        </div>
      {:else if selectedTab === 'users'}
        <div class="empty-state">
          <Icons.Users size={48} />
          <h3>User Management</h3>
          <p>Coming soon: Manage users and permissions</p>
        </div>
      {:else if selectedTab === 'sessions'}
        <div class="empty-state">
          <Icons.Smartphone size={48} />
          <h3>Session Management</h3>
          <p>Coming soon: View and manage active sessions</p>
        </div>
      {:else if selectedTab === 'logs'}
        <div class="empty-state">
          <Icons.FileText size={48} />
          <h3>Activity Logs</h3>
          <p>Coming soon: View system activity logs</p>
        </div>
      {/if}
    </div>
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

  .dashboard-tabs {
    display: flex;
    gap: 0;
    padding: 0 var(--s-6);
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
  }

  .tab-btn {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-4) 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: var(--fs-13);
    font-weight: 500;
    border-bottom: 2px solid transparent;
    transition: all var(--duration-quick) var(--ease-smooth);
    white-space: nowrap;
    margin-right: var(--s-6);
  }

  .tab-btn:hover {
    color: var(--text);
  }

  .tab-btn.active {
    color: var(--blue);
    border-bottom-color: var(--blue);
  }

  .dashboard-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-6);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: var(--s-4);
    margin-bottom: var(--s-8);
  }

  .stat-card {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .stat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .stat-header h3 {
    margin: 0;
    font-size: var(--fs-13);
    font-weight: 600;
    color: var(--text-2);
  }

  .stat-header svg {
    color: var(--muted);
  }

  .stat-value {
    font-size: var(--fs-28);
    font-weight: 700;
    color: var(--text);
  }

  .stat-progress {
    display: flex;
    gap: var(--s-2);
  }

  .stat-meta {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .actions-section {
    margin-bottom: var(--s-8);
  }

  .actions-header {
    margin-bottom: var(--s-4);
  }

  .actions-header h3 {
    margin: 0;
    font-size: var(--fs-16);
    font-weight: 600;
    color: var(--text);
  }

  .actions-list {
    display: flex;
    flex-direction: column;
  }

  .action-item {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-4);
    border: none;
    background: transparent;
    color: var(--text);
    cursor: pointer;
    text-align: left;
    transition: all var(--duration-quick) var(--ease-smooth);
    border-bottom: 1px solid var(--hairline);
  }

  .action-item:hover {
    background: var(--surface-2);
  }

  .action-item:last-child {
    border-bottom: none;
  }

  .action-item svg:first-of-type {
    color: var(--blue);
    flex-shrink: 0;
  }

  .action-item svg:last-of-type {
    color: var(--muted);
    margin-left: auto;
  }

  .action-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .action-name {
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text);
  }

  .action-desc {
    font-size: var(--fs-11);
    color: var(--muted);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    padding: var(--s-8);
    color: var(--muted);
  }

  .empty-state h3 {
    margin: 0;
    color: var(--text-2);
  }

  .empty-state p {
    margin: 0;
  }
</style>
