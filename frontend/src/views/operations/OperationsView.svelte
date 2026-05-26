<script lang="ts">
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import { api, type AdminActivityItem, type AdminSession, type AdminUser, type FileRecord } from '$lib/api';
  import SafeConsole from '$lib/components/SafeConsole.svelte';
  import { error, success } from '$lib/stores/notifications';
  import { formatBytes, formatRelative } from '$lib/utils';

  type AdminSection = 'overview' | 'users' | 'sessions' | 'devices' | 'public' | 'activity' | 'maintenance';

  let activeSection = $state<AdminSection>('overview');
  let loading = $state(true);
  let executing = $state(false);
  let showConsole = $state(false);
  let commandOutput = $state('');

  let overview = $state<Awaited<ReturnType<typeof api.getAdminOverview>> | null>(null);
  let status = $state<Awaited<ReturnType<typeof api.getAdminStatus>> | null>(null);
  let users = $state<AdminUser[]>([]);
  let sessions = $state<AdminSession[]>([]);
  let devices = $state<Array<{ id: string; token: string; trusted_at?: number }>>([]);
  let publicFiles = $state<FileRecord[]>([]);
  let activityItems = $state<AdminActivityItem[]>([]);
  let commands = $state<Array<{ id: string; name: string; description?: string }>>([]);

  const navGroups = [
    {
      label: 'System',
      items: [
        { id: 'overview' as const, label: 'Overview', icon: Icons.Activity },
        { id: 'maintenance' as const, label: 'Maintenance', icon: Icons.Wrench },
        { id: 'activity' as const, label: 'Activity log', icon: Icons.History },
      ],
    },
    {
      label: 'Access',
      items: [
        { id: 'users' as const, label: 'Users', icon: Icons.Users },
        { id: 'sessions' as const, label: 'Sessions', icon: Icons.BadgeCheck },
        { id: 'devices' as const, label: 'Devices', icon: Icons.Smartphone },
      ],
    },
    {
      label: 'Sharing',
      items: [{ id: 'public' as const, label: 'Public links', icon: Icons.Globe }],
    },
  ];
  const navItems = navGroups.flatMap((group) => group.items);

  onMount(async () => {
    await loadAdmin();
  });

  async function loadAdmin() {
    loading = true;
    try {
      const [
        overviewData,
        statusData,
        userData,
        sessionData,
        deviceData,
        publicData,
        activityData,
        commandData,
      ] = await Promise.all([
        api.getAdminOverview(),
        api.getAdminStatus(),
        api.listAdminUsers(),
        api.listAdminSessions(20),
        api.listAdminDevices(),
        api.listPublicFiles(),
        api.listAdminActivity(24),
        api.listAdminConsoleCommands(),
      ]);

      overview = overviewData;
      status = statusData;
      users = userData.users || [];
      sessions = sessionData.sessions || [];
      devices = deviceData.devices || [];
      publicFiles = publicData.files || [];
      activityItems = activityData.items || [];
      commands = commandData.commands || [];
    } catch (cause) {
      error('Admin Failed', cause instanceof Error ? cause.message : 'Could not load admin data');
    } finally {
      loading = false;
    }
  }

  async function runCommand(command: string) {
    executing = true;
    commandOutput = `Running ${command}...`;

    try {
      const response = await api.runAdminConsoleCommand(command);
      commandOutput = response.output || 'Command completed';
      success('Command Executed', command);
      await loadAdmin();
    } catch (cause) {
      commandOutput = cause instanceof Error ? cause.message : 'Command failed';
      error('Command Failed', commandOutput);
    } finally {
      executing = false;
    }
  }

  async function executeSafeConsoleCommand(command: string) {
    try {
      const response = await api.runAdminConsoleCommand(command);

      // Handle structured JSON output from backend
      if (response.output && typeof response.output === 'object') {
        return JSON.stringify(response.output, null, 2);
      }

      // Fallback to string output
      if (typeof response.output === 'string') {
        return response.output;
      }

      // Generic success message
      return response.success ? 'Command completed successfully' : 'Command execution returned no output';
    } catch (cause) {
      return cause instanceof Error ? cause.message : 'Command failed';
    }
  }

  async function revokeDevice(token: string) {
    if (!confirm('Remove this trusted device?')) return;
    try {
      await api.removeAdminDevice(token);
      devices = devices.filter((item) => item.token !== token);
      success('Device Removed', 'Trusted device was revoked');
    } catch (cause) {
      error('Remove Failed', cause instanceof Error ? cause.message : 'Could not revoke device');
    }
  }

  function sectionTitle(section: AdminSection) {
    return navItems.find((item) => item.id === section)?.label || 'Admin';
  }

  function navCount(section: AdminSection) {
    switch (section) {
      case 'users':
        return users.length;
      case 'sessions':
        return sessions.length;
      case 'devices':
        return devices.length;
      case 'public':
        return publicFiles.length;
      case 'activity':
        return activityItems.length;
      default:
        return null;
    }
  }

  function healthTone() {
    const statusValue = status?.status?.toLowerCase() || '';
    if (statusValue.includes('ok') || statusValue.includes('healthy') || statusValue.includes('ready')) {
      return 'good';
    }
    if (statusValue.includes('warn') || statusValue.includes('degraded')) {
      return 'warn';
    }
    return 'neutral';
  }

  function healthLabel() {
    return status?.status || 'Status unavailable';
  }

  let primaryCommands = $derived(
    commands.filter((command) => ['cleanup_temp', 'cleanup_sessions', 'integrity_check'].includes(command.name))
  );
</script>

<div class="admin-shell">
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
              onclick={() => (activeSection = item.id)}
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

  <section class="admin-main">
    <header class="admin-header">
      <div>
        <div class="eyebrow">Operations</div>
        <h1>{sectionTitle(activeSection)}</h1>
        <p>Organized system control for users, sessions, sharing, maintenance, and diagnostics.</p>
        <div class="header-meta">
          <span class={`health-pill ${healthTone()}`}>
            <span class="health-dot"></span>
            {healthLabel()}
          </span>
          <span class="meta-pill">{formatBytes(status?.disk_used || 0)} disk used</span>
          <span class="meta-pill">{sessions.length} live sessions</span>
        </div>
      </div>

      <div class="header-actions">
        <button type="button" class="ghost-btn" onclick={loadAdmin} disabled={loading}>
          <Icons.RefreshCcw size={14} />
          Refresh
        </button>
        <button type="button" class="accent-btn" onclick={() => (showConsole = true)}>
          <Icons.Terminal size={14} />
          Safe console
        </button>
      </div>
    </header>

    {#if loading}
      <div class="loading-panel">
        <div class="spinner"></div>
        <strong>Loading admin state</strong>
        <p>Fetching overview, people, sessions, public links, and activity.</p>
      </div>
    {:else if activeSection === 'overview'}
      <div class="admin-content">
        <div class="overview-hero">
          <article class="panel hero-card hero-primary">
            <div class="panel-head hero-head">
              <div>
                <h2>System posture</h2>
                <p class="panel-copy">Real machine health, repository volume, and persistence status.</p>
              </div>
              <span class={`health-pill ${healthTone()}`}>
                <span class="health-dot"></span>
                {healthLabel()}
              </span>
            </div>

            <div class="metrics-grid">
              <article class="metric-card">
                <span>Repository files</span>
                <strong>{overview?.repository.file_count || 0}</strong>
                <p>{formatBytes(overview?.repository.storage_bytes_used || 0)} in use</p>
              </article>
              <article class="metric-card">
                <span>Notes</span>
                <strong>{overview?.repository.note_count || 0}</strong>
                <p>Knowledge base entries</p>
              </article>
              <article class="metric-card">
                <span>CPU</span>
                <strong>{overview?.system.cpu_percent ?? 0}%</strong>
                <p>Live system load</p>
              </article>
              <article class="metric-card">
                <span>Memory</span>
                <strong>{overview?.system.memory_percent ?? 0}%</strong>
                <p>RAM in use</p>
              </article>
            </div>
          </article>

          <article class="panel hero-card hero-secondary">
            <div class="panel-head">
              <h2>Operational lanes</h2>
              <span class="panel-meta">Jump directly into active areas</span>
            </div>
            <div class="snapshot-list">
              <button type="button" class="snapshot-card" onclick={() => (activeSection = 'users')}>
                <div>
                  <strong>Users</strong>
                  <p>Access inventory and roles</p>
                </div>
                <span>{users.length}</span>
              </button>
              <button type="button" class="snapshot-card" onclick={() => (activeSection = 'sessions')}>
                <div>
                  <strong>Sessions</strong>
                  <p>Browser and CLI presence</p>
                </div>
                <span>{sessions.length}</span>
              </button>
              <button type="button" class="snapshot-card" onclick={() => (activeSection = 'devices')}>
                <div>
                  <strong>Trusted devices</strong>
                  <p>Remembered device access</p>
                </div>
                <span>{devices.length}</span>
              </button>
              <button type="button" class="snapshot-card" onclick={() => (activeSection = 'public')}>
                <div>
                  <strong>Public links</strong>
                  <p>Externally visible objects</p>
                </div>
                <span>{publicFiles.length}</span>
              </button>
            </div>
          </article>
        </div>

        <div class="split-grid">
          <article class="panel">
            <div class="panel-head">
              <h2>System posture</h2>
            </div>
            <div class="stat-list">
              <div class="stat-row"><span>Status</span><strong>{status?.status || 'unknown'}</strong></div>
              <div class="stat-row"><span>Disk</span><strong>{formatBytes(status?.disk_used || 0)} / {formatBytes(status?.disk_total || 0)}</strong></div>
              <div class="stat-row"><span>Memory</span><strong>{formatBytes(status?.memory_used || 0)} / {formatBytes(status?.memory_total || 0)}</strong></div>
              <div class="stat-row"><span>Database</span><strong>{formatBytes(status?.db_size || 0)} · {status?.db_status || 'unknown'}</strong></div>
              <div class="stat-row"><span>Uptime</span><strong>{status?.uptime_hours || 0}h</strong></div>
            </div>
          </article>

          <article class="panel">
            <div class="panel-head">
              <h2>Recent activity</h2>
            </div>
            <div class="activity-list">
              {#each activityItems.slice(0, 6) as item (item.id)}
                <div class="activity-row">
                  <div class="activity-dot"></div>
                  <div>
                    <strong>{item.title}</strong>
                    <p>{item.detail}</p>
                  </div>
                  <span>{formatRelative(item.occurred_at)}</span>
                </div>
              {/each}
            </div>
          </article>

          <article class="panel">
            <div class="panel-head">
              <h2>Maintenance queue</h2>
              <span class="panel-meta">{primaryCommands.length} safe actions</span>
            </div>
            <div class="command-grid">
              {#each primaryCommands as command (command.id)}
                <button type="button" class="command-card" onclick={() => runCommand(command.name)} disabled={executing}>
                  <div>
                    <strong>{command.name}</strong>
                    <p>{command.description || 'Safe backend maintenance command'}</p>
                  </div>
                  <Icons.ChevronRight size={16} />
                </button>
              {/each}
            </div>
          </article>
        </div>
      </div>
    {:else if activeSection === 'users'}
      <div class="admin-content">
        <article class="panel table-panel">
          <div class="panel-head">
            <h2>Users</h2>
            <span class="panel-meta">{users.length} accounts</span>
          </div>
          <div class="table">
            <div class="table-head users">
              <span>Name</span>
              <span>Role</span>
              <span>Created</span>
              <span>Status</span>
            </div>
            {#each users as userRow (userRow.id)}
              <div class="table-row users">
                <strong>{userRow.name}</strong>
                <span>{userRow.role}</span>
                <span>{formatRelative(userRow.created_at)}</span>
                <span>{userRow.disabled ? 'disabled' : 'active'}</span>
              </div>
            {/each}
          </div>
        </article>
      </div>
    {:else if activeSection === 'sessions'}
      <div class="admin-content">
        <article class="panel table-panel">
          <div class="panel-head">
            <h2>Sessions</h2>
            <span class="panel-meta">{sessions.length} active</span>
          </div>
          <div class="table">
            <div class="table-head sessions">
              <span>User</span>
              <span>Kind</span>
              <span>Created</span>
              <span>Expires</span>
              <span>Token</span>
            </div>
            {#each sessions as session (session.token)}
              <div class="table-row sessions">
                <strong>{session.user_name || session.role || 'session'}</strong>
                <span>{session.kind}</span>
                <span>{formatRelative(session.created_at)}</span>
                <span>{formatRelative(session.expires_at)}</span>
                <span class="mono">{session.token_preview}</span>
              </div>
            {/each}
          </div>
        </article>
      </div>
    {:else if activeSection === 'devices'}
      <div class="admin-content">
        <article class="panel table-panel">
          <div class="panel-head">
            <h2>Trusted devices</h2>
            <span class="panel-meta">{devices.length} enrolled</span>
          </div>
          <div class="table">
            <div class="table-head devices">
              <span>Device</span>
              <span>Token</span>
              <span>Trusted</span>
              <span></span>
            </div>
            {#each devices as device (device.token)}
              <div class="table-row devices">
                <strong>{device.id}</strong>
                <span class="mono">{device.token}</span>
                <span>{formatRelative(device.trusted_at)}</span>
                <button type="button" class="inline-action" onclick={() => revokeDevice(device.token)}>Revoke</button>
              </div>
            {/each}
          </div>
        </article>
      </div>
    {:else if activeSection === 'public'}
      <div class="admin-content">
        <article class="panel table-panel">
          <div class="panel-head">
            <h2>Public links</h2>
            <span class="panel-meta">{publicFiles.length} visible objects</span>
          </div>
          <div class="table">
            <div class="table-head public">
              <span>Name</span>
              <span>Size</span>
              <span>Folder</span>
              <span>Updated</span>
            </div>
            {#each publicFiles as file (file.id)}
              <div class="table-row public">
                <strong>{file.name}</strong>
                <span>{formatBytes(file.size_bytes)}</span>
                <span>{file.folder_path || 'Bucket root'}</span>
                <span>{formatRelative(file.updated_at || file.uploaded_at)}</span>
              </div>
            {/each}
          </div>
        </article>
      </div>
    {:else if activeSection === 'activity'}
      <div class="admin-content">
        <article class="panel table-panel">
          <div class="panel-head">
            <h2>Audit events</h2>
            <span class="panel-meta">{activityItems.length} recent items</span>
          </div>
          <div class="activity-log">
            {#each activityItems as item (item.id)}
              <div class="activity-card">
                <div>
                  <strong>{item.title}</strong>
                  <p>{item.detail}</p>
                </div>
                <span>{formatRelative(item.occurred_at)}</span>
              </div>
            {/each}
          </div>
        </article>
      </div>
    {:else if activeSection === 'maintenance'}
      <div class="admin-content">
        <div class="split-grid">
          <article class="panel">
            <div class="panel-head">
              <h2>Maintenance actions</h2>
            </div>
            <div class="command-grid">
              {#each primaryCommands as command (command.id)}
                <button type="button" class="command-card" onclick={() => runCommand(command.name)} disabled={executing}>
                  <div>
                    <strong>{command.name}</strong>
                    <p>{command.description || 'Safe backend maintenance command'}</p>
                  </div>
                  <Icons.ChevronRight size={16} />
                </button>
              {/each}
            </div>
          </article>

          <article class="panel">
            <div class="panel-head">
              <h2>Command output</h2>
            </div>
            <pre class="command-output">{commandOutput || 'Run a maintenance command to inspect output here.'}</pre>
          </article>
        </div>
      </div>
    {/if}
  </section>
</div>

<SafeConsole
  isOpen={showConsole}
  onClose={() => (showConsole = false)}
  onExecuteCommand={executeSafeConsoleCommand}
  {commands}
/>

<style>
  .admin-shell {
    flex: 1;
    display: grid;
    grid-template-columns: 230px minmax(0, 1fr);
    min-height: 0;
  }

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

  .nav-item,
  .ghost-btn,
  .accent-btn,
  .inline-action {
    font-family: inherit;
    cursor: pointer;
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

  .admin-main {
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: auto;
    padding: 24px 24px 140px;
    gap: 18px;
  }

  .admin-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .eyebrow {
    font-size: 11px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--dim);
    font-family: var(--ff-mono);
  }

  .admin-header h1 {
    margin: 8px 0 0;
    font-size: 34px;
    line-height: 1;
    letter-spacing: -0.04em;
    font-family: var(--ff-display);
    color: var(--text);
  }

  .admin-header p {
    margin: 10px 0 0;
    color: var(--muted);
    font-size: 14px;
  }

  .header-meta {
    margin-top: 14px;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .health-pill,
  .meta-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 32px;
    padding: 0 12px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.03);
    color: var(--text-2);
    font-size: 12px;
    font-weight: 500;
  }

  .health-pill.good {
    border-color: rgba(91, 227, 154, 0.22);
    background: rgba(91, 227, 154, 0.1);
    color: var(--green);
  }

  .health-pill.warn {
    border-color: rgba(251, 191, 36, 0.24);
    background: rgba(251, 191, 36, 0.1);
    color: var(--warning);
  }

  .health-dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: currentColor;
    box-shadow: 0 0 10px currentColor;
  }

  .header-actions {
    display: flex;
    gap: 10px;
  }

  .ghost-btn,
  .accent-btn {
    height: 36px;
    padding: 0 14px;
    border-radius: 12px;
    border: 1px solid var(--border);
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .ghost-btn {
    background: var(--surface);
    color: var(--text-2);
  }

  .accent-btn {
    background: linear-gradient(135deg, rgba(163, 148, 255, 0.28), rgba(110, 168, 255, 0.22));
    color: var(--text);
    border-color: rgba(163, 148, 255, 0.18);
  }

  .loading-panel,
  .panel,
  .metric-card {
    border: 1px solid var(--border);
    background: linear-gradient(180deg, rgba(20, 22, 29, 0.95), rgba(14, 15, 21, 0.93));
    box-shadow: var(--shadow-card);
  }

  .loading-panel {
    min-height: 320px;
    border-radius: 18px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    color: var(--muted);
  }

  .loading-panel strong {
    color: var(--text);
  }

  .admin-content {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .overview-hero {
    display: grid;
    grid-template-columns: minmax(0, 1.6fr) minmax(280px, 0.9fr);
    gap: 14px;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 12px;
  }

  .metric-card,
  .panel {
    border-radius: 18px;
    padding: 16px;
  }

  .hero-card {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .hero-head {
    justify-content: space-between;
    align-items: flex-start;
  }

  .panel-copy {
    margin: 8px 0 0;
    color: var(--muted);
    font-size: 13px;
  }

  .metric-card span {
    font-size: 11px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--muted);
    font-family: var(--ff-mono);
  }

  .metric-card strong {
    margin-top: 10px;
    font-size: 28px;
    color: var(--text);
  }

  .metric-card p,
  .panel-meta,
  .stat-row span,
  .activity-row p,
  .activity-card p,
  .command-card p {
    color: var(--muted);
    font-size: 12px;
  }

  .metric-card p {
    margin: 6px 0 0;
  }

  .split-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 14px;
  }

  .snapshot-list {
    display: grid;
    gap: 10px;
  }

  .snapshot-card {
    width: 100%;
    padding: 14px;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.02);
    color: inherit;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 14px;
    align-items: center;
    text-align: left;
    cursor: pointer;
    transition: transform var(--duration-quick) var(--ease-smooth), border-color var(--duration-quick) var(--ease-smooth), background var(--duration-quick) var(--ease-smooth);
  }

  .snapshot-card:hover {
    transform: translateY(-1px);
    border-color: var(--border-2);
    background: rgba(255, 255, 255, 0.04);
  }

  .snapshot-card p {
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 12px;
  }

  .snapshot-card span {
    min-width: 34px;
    height: 34px;
    padding: 0 10px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: rgba(163, 148, 255, 0.12);
    color: var(--violet);
    font-family: var(--ff-mono);
    font-size: 12px;
    font-weight: 600;
  }

  .panel-head {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 14px;
  }

  .panel-head h2 {
    margin: 0;
    color: var(--text);
    font-size: 16px;
  }

  .stat-list,
  .activity-list,
  .activity-log {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .stat-row,
  .activity-row,
  .activity-card {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 0;
    border-bottom: 1px solid var(--hairline);
  }

  .stat-row:last-child,
  .activity-row:last-child,
  .activity-card:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .stat-row strong,
  .activity-row strong,
  .activity-card strong,
  .table-row strong,
  .command-card strong {
    color: var(--text);
  }

  .activity-dot {
    width: 10px;
    height: 10px;
    border-radius: 999px;
    background: var(--violet);
    margin-top: 5px;
    box-shadow: 0 0 12px rgba(163, 148, 255, 0.45);
  }

  .table-panel {
    padding: 0;
    overflow: hidden;
  }

  .table {
    display: flex;
    flex-direction: column;
  }

  .table-head,
  .table-row {
    display: grid;
    gap: 12px;
    align-items: center;
    padding: 12px 16px;
  }

  .table-head {
    border-top: 1px solid var(--hairline);
    border-bottom: 1px solid var(--hairline);
    font-size: 10px;
    color: var(--dim);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    font-family: var(--ff-mono);
  }

  .table-row {
    border-bottom: 1px solid var(--hairline);
    color: var(--text-2);
    font-size: 13px;
  }

  .table-row:last-child {
    border-bottom: none;
  }

  .table-head.users,
  .table-row.users {
    grid-template-columns: 1.4fr 0.7fr 0.8fr 0.7fr;
  }

  .table-head.sessions,
  .table-row.sessions {
    grid-template-columns: 1fr 0.8fr 0.8fr 0.8fr 0.8fr;
  }

  .table-head.devices,
  .table-row.devices {
    grid-template-columns: 1fr 1.2fr 0.8fr 120px;
  }

  .table-head.public,
  .table-row.public {
    grid-template-columns: 1.6fr 0.7fr 0.9fr 0.8fr;
  }

  .mono {
    font-family: var(--ff-mono);
  }

  .inline-action {
    height: 30px;
    padding: 0 10px;
    border: 1px solid rgba(255, 107, 107, 0.18);
    border-radius: 10px;
    background: rgba(255, 107, 107, 0.08);
    color: var(--danger);
  }

  .command-grid {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .command-card {
    width: 100%;
    padding: 14px;
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--surface);
    color: inherit;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    text-align: left;
  }

  .command-output {
    min-height: 220px;
    margin: 0;
    padding: 14px;
    border-radius: 14px;
    background: rgba(7, 8, 12, 0.92);
    border: 1px solid var(--hairline);
    color: var(--text-2);
    white-space: pre-wrap;
    font-family: var(--ff-mono);
    font-size: 12px;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border-radius: 999px;
    border: 2px solid rgba(255, 255, 255, 0.12);
    border-top-color: var(--blue);
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 1180px) {
    .admin-shell {
      grid-template-columns: 1fr;
    }

    .admin-sidebar {
      display: none;
    }

    .overview-hero,
    .metrics-grid,
    .split-grid {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 760px) {
    .admin-main {
      padding: 18px 16px 120px;
    }

    .admin-header {
      flex-direction: column;
    }

    .header-actions {
      width: 100%;
      justify-content: stretch;
    }

    .header-actions button {
      flex: 1;
      justify-content: center;
    }

    .metrics-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .hero-head {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>
