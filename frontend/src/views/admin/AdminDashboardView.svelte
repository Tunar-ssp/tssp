<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { api, type AdminUser, type AdminSession, type AdminActivityItem } from '$lib/api';
  import { isAdmin } from '$lib/stores/auth';
  import { error as showError } from '$lib/stores/notifications';
  import Btn from '$lib/components/Btn.svelte';
  import Card from '$lib/components/Card.svelte';
  import Bar from '$lib/components/Bar.svelte';
  import StatusDot from '$lib/components/StatusDot.svelte';

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

  async function loadStats() {
    if (!$isAdmin) return;
    isLoading = true;
    try {
      const overview = await api.getAdminOverview();
      stats = {
        fileCount: overview.repository.file_count || 0,
        noteCount: overview.repository.note_count || 0,
        workspaceCount: 0,
        usedStorage: overview.repository.storage_bytes_used || 0,
        uptime: overview.system.uptime_seconds || 0,
        corruptFileCount: 0,
      };
    } catch (e) {
      showError(e instanceof Error ? e.message : 'Failed to load stats');
    } finally {
      isLoading = false;
    }
  }

  async function loadUsers() {
    if (!$isAdmin) return;
    isLoading = true;
    try {
      const data = await api.listAdminUsers();
      users = data.users || [];
    } catch (e) {
      showError(e instanceof Error ? e.message : 'Failed to load users');
    } finally {
      isLoading = false;
    }
  }

  async function loadSessions() {
    if (!$isAdmin) return;
    isLoading = true;
    try {
      const data = await api.listAdminSessions(100);
      sessions = data.sessions || [];
    } catch (e) {
      showError(e instanceof Error ? e.message : 'Failed to load sessions');
    } finally {
      isLoading = false;
    }
  }

  async function loadActivity() {
    if (!$isAdmin) return;
    isLoading = true;
    try {
      const data = await api.listAdminActivity(100);
      activities = data.items || [];
    } catch (e) {
      showError(e instanceof Error ? e.message : 'Failed to load activity');
    } finally {
      isLoading = false;
    }
  }

  function formatTimestamp(secs: number): string {
    const date = new Date(secs * 1000);
    return date.toLocaleString();
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
      if (selectedTab === 'dashboard') {
        loadStats();
      } else if (selectedTab === 'users') {
        loadUsers();
      } else if (selectedTab === 'sessions') {
        loadSessions();
      } else if (selectedTab === 'logs') {
        loadActivity();
      }
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
        onclick={() => (selectedTab = 'dashboard')}
      >
        <Icons.BarChart3 size={16} />
        Dashboard
      </button>
      <button
        class="tab-btn"
        class:active={selectedTab === 'users'}
        onclick={() => (selectedTab = 'users')}
      >
        <Icons.Users size={16} />
        Users
      </button>
      <button
        class="tab-btn"
        class:active={selectedTab === 'sessions'}
        onclick={() => (selectedTab = 'sessions')}
      >
        <Icons.Smartphone size={16} />
        Sessions
      </button>
      <button
        class="tab-btn"
        class:active={selectedTab === 'logs'}
        onclick={() => (selectedTab = 'logs')}
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
              <div class="stat-value">{stats.fileCount}</div>
              <div class="stat-meta">{stats.noteCount} notes · {stats.workspaceCount} workspaces</div>
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
                  value={stats.usedStorage > 0 ? 100 : 0}
                />
              </div>
              <div class="stat-meta">Indexed object storage</div>
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
              <div class="stat-value">{stats.corruptFileCount}</div>
              <div class="stat-meta">Corrupt indexed blobs</div>
            </div>
          </Card>
        </div>

        <div class="actions-section">
          <Card>
            <div class="actions-header">
              <h3>Maintenance</h3>
            </div>
            <div class="actions-list">
              <button type="button" class="action-item" disabled>
                <Icons.Database size={16} />
                <div class="action-text">
                  <span class="action-name">Database Backup</span>
                  <span class="action-desc">Create a system backup</span>
                </div>
                <Icons.ChevronRight size={16} />
              </button>
              <button type="button" class="action-item" disabled>
                <Icons.CheckCircle2 size={16} />
                <div class="action-text">
                  <span class="action-name">Integrity Check</span>
                  <span class="action-desc">Verify system integrity</span>
                </div>
                <Icons.ChevronRight size={16} />
              </button>
              <button type="button" class="action-item" disabled>
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
        <Card>
          <div class="list-container">
            <div class="list-header">
              <h3>System Users</h3>
            </div>
            {#if isLoading}
              <div class="loading">Loading users...</div>
            {:else if users.length === 0}
              <div class="empty">
                <Icons.Users size={32} />
                <p>No users found</p>
              </div>
            {:else}
              <table class="data-table">
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>ID</th>
                    <th>Role</th>
                    <th>Created</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody>
                  {#each users as user}
                    <tr>
                      <td>{user.name}</td>
                      <td><code>{user.id}</code></td>
                      <td><span class={`role role-${user.role}`}>{user.role}</span></td>
                      <td>{formatTimestamp(user.created_at)}</td>
                      <td>
                        {#if user.disabled}
                          <span class="status status-disabled">Disabled</span>
                        {:else}
                          <span class="status status-active">Active</span>
                        {/if}
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </div>
        </Card>
      {:else if selectedTab === 'sessions'}
        <Card>
          <div class="list-container">
            <div class="list-header">
              <h3>Active Sessions</h3>
            </div>
            {#if isLoading}
              <div class="loading">Loading sessions...</div>
            {:else if sessions.length === 0}
              <div class="empty">
                <Icons.Smartphone size={32} />
                <p>No active sessions</p>
              </div>
            {:else}
              <table class="data-table">
                <thead>
                  <tr>
                    <th>User</th>
                    <th>Kind</th>
                    <th>Token</th>
                    <th>Created</th>
                    <th>Expires</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody>
                  {#each sessions as session}
                    <tr>
                      <td>{session.user_name || 'Unknown'}</td>
                      <td><span class="kind">{session.kind}</span></td>
                      <td><code>{session.token_preview}</code></td>
                      <td>{formatTimestamp(session.created_at)}</td>
                      <td>{formatTimestamp(session.expires_at)}</td>
                      <td>
                        {#if session.current}
                          <span class="status status-current">Current</span>
                        {:else}
                          <span class="status status-active">Active</span>
                        {/if}
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </div>
        </Card>
      {:else if selectedTab === 'logs'}
        <Card>
          <div class="list-container">
            <div class="list-header">
              <h3>Activity Log</h3>
            </div>
            {#if isLoading}
              <div class="loading">Loading activity...</div>
            {:else if activities.length === 0}
              <div class="empty">
                <Icons.FileText size={32} />
                <p>No activity recorded</p>
              </div>
            {:else}
              <table class="data-table">
                <thead>
                  <tr>
                    <th>Action</th>
                    <th>Resource</th>
                    <th>Title</th>
                    <th>Time</th>
                  </tr>
                </thead>
                <tbody>
                  {#each activities as item}
                    <tr>
                      <td><span class="kind">{item.kind}</span></td>
                      <td><code>{item.id}</code></td>
                      <td>{item.title}</td>
                      <td>{formatTimestamp(item.occurred_at)}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </div>
        </Card>
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


  .list-container {
    display: flex;
    flex-direction: column;
    gap: var(--s-4);
  }

  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--border);
    padding-bottom: var(--s-4);
  }

  .list-header h3 {
    margin: 0;
    font-size: var(--fs-16);
    font-weight: 600;
    color: var(--text);
  }

  .loading {
    padding: var(--s-6);
    text-align: center;
    color: var(--muted);
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    padding: var(--s-8);
    color: var(--muted);
  }

  .empty p {
    margin: 0;
  }

  .data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--fs-13);
  }

  .data-table thead {
    background-color: var(--surface);
    border-bottom: 1px solid var(--border);
  }

  .data-table th {
    padding: var(--s-3) var(--s-4);
    text-align: left;
    font-weight: 600;
    color: var(--text-2);
  }

  .data-table td {
    padding: var(--s-3) var(--s-4);
    border-bottom: 1px solid var(--border);
    color: var(--text);
  }

  .data-table tbody tr:hover {
    background-color: var(--surface);
  }

  .role {
    display: inline-block;
    padding: var(--s-1) var(--s-2);
    border-radius: 4px;
    font-size: var(--fs-12);
    font-weight: 500;
  }

  .role-admin {
    background-color: var(--red-surface);
    color: var(--red);
  }

  .role-user {
    background-color: var(--blue-surface);
    color: var(--blue);
  }

  .status {
    display: inline-block;
    padding: var(--s-1) var(--s-2);
    border-radius: 4px;
    font-size: var(--fs-12);
    font-weight: 500;
  }

  .status-active {
    background-color: var(--green-surface);
    color: var(--green);
  }

  .status-current {
    background-color: var(--yellow-surface);
    color: var(--yellow);
  }

  .status-disabled {
    background-color: var(--gray-surface);
    color: var(--gray);
  }

  .kind {
    display: inline-block;
    padding: var(--s-1) var(--s-2);
    border-radius: 4px;
    background-color: var(--surface-2);
    font-size: var(--fs-12);
    font-weight: 500;
    color: var(--text-2);
  }

  code {
    font-family: 'Monaco', 'Menlo', monospace;
    font-size: var(--fs-12);
    background-color: var(--surface);
    padding: 2px 4px;
    border-radius: 2px;
  }
</style>
