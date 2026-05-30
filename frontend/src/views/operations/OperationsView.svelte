<script lang="ts">
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import { api, type AdminActivityItem, type AdminSession, type AdminUser, type FileRecord } from '$lib/api';
  import SafeConsole from '$lib/components/SafeConsole.svelte';
  import OperationsSidebar from './layout/OperationsSidebar.svelte';
  import OperationsHeader from './layout/OperationsHeader.svelte';
  import OperationsOverview from './components/sections/OperationsOverview.svelte';
  import OperationsUsersTable from './components/tables/OperationsUsersTable.svelte';
  import OperationsSessionsTable from './components/tables/OperationsSessionsTable.svelte';
  import OperationsDevicesTable from './components/tables/OperationsDevicesTable.svelte';
  import OperationsPublicTable from './components/tables/OperationsPublicTable.svelte';
  import OperationsActivityLog from './components/sections/OperationsActivityLog.svelte';
  import OperationsMaintenance from './components/sections/OperationsMaintenance.svelte';
  import { error, success } from '$lib/stores/notifications';
  import { confirmDialog } from '$lib/stores/dialog';

  type AdminSection = 'overview' | 'users' | 'sessions' | 'devices' | 'public' | 'activity' | 'maintenance';
  type NavItem = { id: AdminSection; label: string; icon: any };

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
  let commands = $state<Array<{ id: string; name: string; description?: string; category?: string }>>([]);

  const navGroups: Array<{ label: string; items: NavItem[] }> = [
    {
      label: 'System',
      items: [
        { id: 'overview', label: 'Overview', icon: Icons.Activity },
        { id: 'maintenance', label: 'Maintenance', icon: Icons.Wrench },
        { id: 'activity', label: 'Activity log', icon: Icons.History },
      ],
    },
    {
      label: 'Access',
      items: [
        { id: 'users', label: 'Users', icon: Icons.Users },
        { id: 'sessions', label: 'Sessions', icon: Icons.BadgeCheck },
        { id: 'devices', label: 'Devices', icon: Icons.Smartphone },
      ],
    },
    {
      label: 'Sharing',
      items: [{ id: 'public', label: 'Public links', icon: Icons.Globe }],
    },
  ];
  const navItems: NavItem[] = navGroups.flatMap((group) => group.items);

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
      if (response.output && typeof response.output === 'object') {
        return JSON.stringify(response.output, null, 2);
      }
      if (typeof response.output === 'string') {
        return response.output;
      }
      return response.success ? 'Command completed successfully' : 'Command execution returned no output';
    } catch (cause) {
      return cause instanceof Error ? cause.message : 'Command failed';
    }
  }

  async function revokeDevice(token: string) {
    const ok = await confirmDialog({
      title: 'Remove trusted device',
      message: 'The device will be revoked and must re-authenticate.',
      confirmLabel: 'Remove',
      tone: 'danger',
    });
    if (!ok) return;
    try {
      await api.removeAdminDevice(token);
      devices = devices.filter((item) => item.token !== token);
      success('Device Removed', 'Trusted device was revoked');
    } catch (cause) {
      error('Remove Failed', cause instanceof Error ? cause.message : 'Could not revoke device');
    }
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

  let primaryCommands = $derived(
    commands.filter((command) => ['cleanup_temp', 'cleanup_sessions', 'integrity_check'].includes(command.name))
  );
</script>

<div class="admin-shell">
  <OperationsSidebar
    {activeSection}
    {navGroups}
    {navCount}
    onSectionChange={(section) => (activeSection = section)}
  />

  <section class="admin-main">
    <OperationsHeader
      {activeSection}
      {navItems}
      {status}
      {sessions}
      {loading}
      onRefresh={loadAdmin}
      onShowConsole={() => (showConsole = true)}
    />

    {#if loading}
      <div class="loading-panel">
        <div class="spinner"></div>
        <strong>Loading admin state</strong>
        <p>Fetching overview, people, sessions, public links, and activity.</p>
      </div>
    {:else if activeSection === 'overview'}
      <OperationsOverview
        {overview}
        {status}
        {users}
        {sessions}
        {devices}
        {publicFiles}
        {activityItems}
        {primaryCommands}
        onSectionChange={(section) => (activeSection = section as AdminSection)}
        onRunCommand={runCommand}
        executing={executing}
      />
    {:else if activeSection === 'users'}
      <OperationsUsersTable {users} />
    {:else if activeSection === 'sessions'}
      <OperationsSessionsTable {sessions} />
    {:else if activeSection === 'devices'}
      <OperationsDevicesTable {devices} onRevoke={revokeDevice} />
    {:else if activeSection === 'public'}
      <OperationsPublicTable files={publicFiles} />
    {:else if activeSection === 'activity'}
      <OperationsActivityLog items={activityItems} />
    {:else if activeSection === 'maintenance'}
      <OperationsMaintenance
        commands={primaryCommands}
        {commandOutput}
        {executing}
        onRunCommand={runCommand}
      />
    {/if}
  </section>
</div>

<SafeConsole
  isOpen={showConsole}
  onClose={() => (showConsole = false)}
  onExecuteCommand={executeSafeConsoleCommand}
  commands={commands.map((cmd) => ({ ...cmd, category: cmd.category || 'general' }))}
/>

<style>
  .admin-shell {
    flex: 1;
    display: grid;
    grid-template-columns: 230px minmax(0, 1fr);
    min-height: 0;
  }

  .admin-main {
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: auto;
    padding: 24px 24px 140px;
    gap: 18px;
  }

  .loading-panel {
    border: 1px solid var(--border);
    background: linear-gradient(180deg, rgba(20, 22, 29, 0.95), rgba(14, 15, 21, 0.93));
    box-shadow: var(--shadow-card);
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

  @media (max-width: 760px) {
    .admin-main {
      padding: 18px 16px 120px;
    }
  }
</style>
