<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import ProgressBar from '$lib/components/ProgressBar.svelte';
  import SafeConsole from '$lib/components/SafeConsole.svelte';
  import { success, error } from '$lib/stores/notifications';
  import { onMount } from 'svelte';

  let activeTab = 'dashboard';
  let isLoading = false;
  let systemStatus: any = null;
  let commands: any[] = [];
  let commandOutput = '';
  let selectedCommand = '';
  let isExecuting = false;
  let showConsole = $state(false);

  const tabs = [
    { id: 'dashboard', label: 'Dashboard', icon: Icons.BarChart3 },
    { id: 'maintenance', label: 'Maintenance', icon: Icons.Wrench },
    { id: 'settings', label: 'Settings', icon: Icons.Settings },
  ];

  const maintenanceActions = [
    {
      id: 'cleanup-temp',
      label: 'Clean Temporary Files',
      description: 'Remove unused temporary files',
      icon: Icons.Trash2,
      action: async () => await runCommand('cleanup_temp'),
    },
    {
      id: 'cleanup-sessions',
      label: 'Clean Expired Sessions',
      description: 'Remove expired session tokens',
      icon: Icons.LogOut,
      action: async () => await runCommand('cleanup_sessions'),
    },
    {
      id: 'integrity-check',
      label: 'Run Integrity Check',
      description: 'Verify file integrity and database consistency',
      icon: Icons.CheckCircle2,
      action: async () => await runCommand('integrity_check'),
    },
  ];

  onMount(async () => {
    await loadSystemStatus();
    await loadAvailableCommands();
  });

  async function loadSystemStatus() {
    try {
      isLoading = true;
      const response = await fetch('/api/v1/admin/status');
      if (!response.ok) throw new Error('Failed to load system status');
      systemStatus = await response.json();
    } catch (err) {
      error('Failed to load system status');
      console.error(err);
    } finally {
      isLoading = false;
    }
  }

  async function loadAvailableCommands() {
    try {
      const response = await fetch('/api/v1/admin/console/commands');
      if (!response.ok) throw new Error('Failed to load commands');
      const data = await response.json();
      commands = data.commands || [];
    } catch (err) {
      error('Failed to load available commands');
      console.error(err);
    }
  }

  async function runCommand(commandName: string) {
    try {
      isExecuting = true;
      commandOutput = `Running ${commandName}...`;
      const response = await fetch('/api/v1/admin/console/run', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command: commandName }),
      });
      if (!response.ok) throw new Error('Command failed');
      const data = await response.json();
      commandOutput = data.output || 'Command executed successfully';
      success('Command executed');
    } catch (err) {
      commandOutput = `Error: ${err instanceof Error ? err.message : 'Unknown error'}`;
      error('Command failed');
    } finally {
      isExecuting = false;
    }
  }

  async function refreshSystemStatus() {
    await loadSystemStatus();
    success('System status refreshed');
  }

  let storagePercent = $derived(
    systemStatus
      ? Math.round((systemStatus.disk_used / systemStatus.disk_total) * 100)
      : 0
  );
  let memoryPercent = $derived(
    systemStatus
      ? Math.round((systemStatus.memory_used / systemStatus.memory_total) * 100)
      : 0
  );
</script>

<div class="ops-view">
  <div class="header">
    <div>
      <h2>Operations Console</h2>
      <p class="subtitle">System monitoring, maintenance, and administration</p>
    </div>
    <div class="header-buttons">
      <button class="refresh-btn" on:click={refreshSystemStatus} disabled={isLoading}>
        <div class="refresh-icon" class:spinning={isLoading}>
          <Icons.RotateCw size={16} />
        </div>
        Refresh
      </button>
      <button class="console-btn" on:click={() => (showConsole = !showConsole)}>
        <Icons.Terminal size={16} />
        Console
      </button>
    </div>
  </div>

  <div class="tabs-bar">
    {#each tabs as tab (tab.id)}
      <button
        class="tab-btn"
        class:active={activeTab === tab.id}
        on:click={() => (activeTab = tab.id)}
      >
        <svelte:component this={tab.icon} size={16} />
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="content">
    {#if activeTab === 'dashboard'}
      <div class="dashboard">
        {#if isLoading}
          <div class="loading">
            <div class="spinner" />
            Loading system status...
          </div>
        {:else if systemStatus}
          <div class="metrics-grid">
            <!-- Storage -->
            <div class="metric-card">
              <div class="metric-header">
                <Icons.HardDrive size={20} />
                <h3>Storage</h3>
              </div>
              <div class="metric-value">
                {(systemStatus.disk_used / 1024 / 1024 / 1024).toFixed(1)} GB / {(systemStatus.disk_total / 1024 / 1024 / 1024).toFixed(1)} GB
              </div>
              <ProgressBar value={storagePercent} tone={storagePercent > 80 ? 'danger' : 'blue'} />
              <div class="metric-percent">{storagePercent}% used</div>
            </div>

            <!-- Memory -->
            <div class="metric-card">
              <div class="metric-header">
                <Icons.Zap size={20} />
                <h3>Memory</h3>
              </div>
              <div class="metric-value">
                {(systemStatus.memory_used / 1024 / 1024).toFixed(0)} MB / {(systemStatus.memory_total / 1024 / 1024).toFixed(0)} MB
              </div>
              <ProgressBar value={memoryPercent} tone={memoryPercent > 80 ? 'danger' : 'green'} />
              <div class="metric-percent">{memoryPercent}% used</div>
            </div>

            <!-- CPU -->
            <div class="metric-card">
              <div class="metric-header">
                <Icons.Cpu size={20} />
                <h3>CPU</h3>
              </div>
              <div class="metric-value">
                {systemStatus.cpu_percent?.toFixed(1) || 'N/A'}%
              </div>
              <ProgressBar value={systemStatus.cpu_percent || 0} tone="orange" />
              <div class="metric-percent">Load average: {systemStatus.load_average || 'N/A'}</div>
            </div>

            <!-- Uptime -->
            <div class="metric-card">
              <div class="metric-header">
                <Icons.Clock size={20} />
                <h3>Uptime</h3>
              </div>
              <div class="metric-value">
                {systemStatus.uptime_hours || 'N/A'} hours
              </div>
              <div class="metric-desc">
                Last restart: {systemStatus.last_restart || 'Unknown'}
              </div>
            </div>

            <!-- Files -->
            <div class="metric-card">
              <div class="metric-header">
                <Icons.FileText size={20} />
                <h3>Files</h3>
              </div>
              <div class="metric-value">
                {systemStatus.total_files || 0}
              </div>
              <div class="metric-desc">
                Total {(systemStatus.total_size / 1024 / 1024).toFixed(1)} MB
              </div>
            </div>

            <!-- Database -->
            <div class="metric-card">
              <div class="metric-header">
                <Icons.Database size={20} />
                <h3>Database</h3>
              </div>
              <div class="metric-value">
                {systemStatus.db_size ? (systemStatus.db_size / 1024 / 1024).toFixed(1) + ' MB' : 'N/A'}
              </div>
              <div class="metric-desc">
                Status: {systemStatus.db_status || 'Unknown'}
              </div>
            </div>
          </div>
        {/if}
      </div>
    {:else if activeTab === 'maintenance'}
      <div class="maintenance">
        <div class="section">
          <h3>Maintenance Tasks</h3>
          <div class="actions-grid">
            {#each maintenanceActions as action (action.id)}
              <button
                class="action-card"
                on:click={action.action}
                disabled={isExecuting}
              >
                <div class="action-icon">
                  <svelte:component this={action.icon} size={24} />
                </div>
                <div class="action-info">
                  <div class="action-title">{action.label}</div>
                  <div class="action-desc">{action.description}</div>
                </div>
                <Icons.ChevronRight size={16} />
              </button>
            {/each}
          </div>
        </div>

        {#if commandOutput}
          <div class="command-output">
            <h3>Output</h3>
            <pre>{commandOutput}</pre>
          </div>
        {/if}
      </div>
    {:else if activeTab === 'settings'}
      <div class="settings">
        <div class="section">
          <h3>System Settings</h3>
          <div class="settings-group">
            <div class="setting-item">
              <div class="setting-label">
                <h4>Max File Size</h4>
                <p>Maximum allowed file upload size</p>
              </div>
              <input type="text" class="setting-input" value="100 MB" disabled />
            </div>

            <div class="setting-item">
              <div class="setting-label">
                <h4>Session Timeout</h4>
                <p>Auto-logout after inactivity</p>
              </div>
              <input type="text" class="setting-input" value="24 hours" disabled />
            </div>

            <div class="setting-item">
              <div class="setting-label">
                <h4>Backup Retention</h4>
                <p>How long to keep database backups</p>
              </div>
              <input type="text" class="setting-input" value="30 days" disabled />
            </div>

            <div class="setting-item">
              <div class="setting-label">
                <h4>API Rate Limit</h4>
                <p>Requests per minute per user</p>
              </div>
              <input type="text" class="setting-input" value="1000" disabled />
            </div>
          </div>
        </div>

        <div class="section">
          <h3>Advanced Options</h3>
          <div class="settings-group">
            <div class="setting-toggle">
              <div class="setting-label">
                <h4>Enable Debug Mode</h4>
                <p>Show detailed logs and error traces</p>
              </div>
              <input type="checkbox" class="toggle-switch" disabled />
            </div>

            <div class="setting-toggle">
              <div class="setting-label">
                <h4>Enable Compression</h4>
                <p>Compress files for storage</p>
              </div>
              <input type="checkbox" class="toggle-switch" checked disabled />
            </div>

            <div class="setting-toggle">
              <div class="setting-label">
                <h4>Enable Deduplication</h4>
                <p>Detect and store duplicate content once</p>
              </div>
              <input type="checkbox" class="toggle-switch" checked disabled />
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<SafeConsole
  isOpen={showConsole}
  onClose={() => (showConsole = false)}
  onExecuteCommand={async (cmd) => {
    return await fetch('/api/v1/admin/console/run', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ command: cmd }),
    })
      .then(r => r.json())
      .then(d => d.output || 'Command executed')
      .catch(e => `Error: ${e.message}`);
  }}
  {commands}
/>

<style>
  .ops-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg);
  }

  .header {
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .header h2 {
    margin: 0;
    font-size: var(--fs-24);
    color: var(--text);
  }

  .subtitle {
    margin: 4px 0 0;
    font-size: var(--fs-13);
    color: var(--text-2);
  }

  .refresh-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface-2);
    color: var(--text);
    cursor: pointer;
    font-size: var(--fs-12);
    font-weight: 500;
    transition: all 0.15s;
  }

  .refresh-btn:hover:not(:disabled) {
    background: var(--surface-3);
  }

  .refresh-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .header-buttons {
    display: flex;
    gap: 8px;
  }

  .console-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--blue);
    color: #0a1228;
    cursor: pointer;
    font-size: var(--fs-12);
    font-weight: 500;
    transition: all 0.15s;
  }

  .console-btn:hover {
    opacity: 0.9;
  }

  .refresh-icon {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .refresh-icon.spinning {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .tabs-bar {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    padding: 0 24px;
    overflow-x: auto;
  }

  .tab-btn {
    padding: 12px 16px;
    border: none;
    background: transparent;
    color: var(--text-2);
    font-size: var(--fs-13);
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
    white-space: nowrap;
    border-bottom: 2px solid transparent;
    transition: all 0.15s;
  }

  .tab-btn:hover {
    color: var(--text);
  }

  .tab-btn.active {
    color: var(--blue);
    border-bottom-color: var(--blue);
  }

  .content {
    flex: 1;
    overflow: auto;
  }

  .dashboard {
    padding: 24px;
  }

  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 16px;
  }

  .metric-card {
    padding: 20px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-3);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .metric-header {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--text-2);
  }

  .metric-header h3 {
    margin: 0;
    font-size: var(--fs-13);
    font-weight: 600;
  }

  .metric-value {
    font-size: var(--fs-20);
    font-weight: 600;
    color: var(--text);
  }

  .metric-percent {
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .metric-desc {
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .maintenance {
    padding: 24px;
  }

  .section {
    margin-bottom: 32px;
  }

  .section h3 {
    margin: 0 0 16px;
    font-size: var(--fs-18);
    font-weight: 600;
    color: var(--text);
  }

  .actions-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: 12px;
  }

  .action-card {
    padding: 16px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 16px;
    transition: all 0.15s;
  }

  .action-card:hover:not(:disabled) {
    background: var(--surface-2);
    border-color: var(--text-2);
  }

  .action-card:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .action-icon {
    flex-shrink: 0;
    color: var(--blue);
  }

  .action-info {
    flex: 1;
    text-align: left;
  }

  .action-title {
    font-weight: 600;
    color: var(--text);
    font-size: var(--fs-13);
    margin-bottom: 4px;
  }

  .action-desc {
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .command-output {
    margin-top: 24px;
    padding: 16px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
  }

  .command-output h3 {
    margin: 0 0 12px;
    font-size: var(--fs-13);
    color: var(--text);
  }

  .command-output pre {
    margin: 0;
    padding: 12px;
    background: var(--surface);
    border-radius: var(--r-2);
    color: var(--text);
    font-size: var(--fs-11);
    overflow-x: auto;
    line-height: 1.4;
  }

  .settings {
    padding: 24px;
  }

  .settings-group {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    gap: 20px;
  }

  .setting-label h4 {
    margin: 0 0 4px;
    font-size: var(--fs-13);
    color: var(--text);
  }

  .setting-label p {
    margin: 0;
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .setting-input {
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--bg);
    color: var(--text);
    font-size: var(--fs-12);
    min-width: 150px;
  }

  .setting-toggle {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
  }

  .toggle-switch {
    width: 40px;
    height: 24px;
    cursor: pointer;
  }
</style>
