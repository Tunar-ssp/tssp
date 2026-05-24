<script lang="ts">
  import { onMount } from "svelte";
  import {
    getEditorExecutionState,
    getAdminActivity,
    getAdminOverview,
    getAdminSystem,
    listAdminDevices,
    listAdminFiles,
    listAdminFolders,
    listAdminSessions,
    listAdminUsers,
    listConsoleCommands,
    type AdminActivityItem,
    type AdminDevice,
    type AdminOverview,
    type AdminSession,
    type AdminUser,
    type AdminSystem,
    type ConsoleCommand,
  } from "../../lib/api";
  import { formatBytes } from "../../lib/utils/format";
  import { formatRelativeDate } from "../../lib/utils/format";

  let overview: AdminOverview | null = null;
  let system: AdminSystem | null = null;
  let activity: AdminActivityItem[] = [];
  let users: AdminUser[] = [];
  let sessions: AdminSession[] = [];
  let devices: AdminDevice[] = [];
  let commands: ConsoleCommand[] = [];
  let folderCount = 0;
  let fileInventoryCount = 0;
  let executionMessage = "";
  let loading = true;
  let error = "";

  onMount(async () => {
    try {
      const [nextOverview, nextSystem, nextActivity, nextUsers, nextSessions, nextDevices, nextCommands, nextExecution, nextFiles, nextFolders] = await Promise.all([
        getAdminOverview(),
        getAdminSystem(),
        getAdminActivity(),
        listAdminUsers(),
        listAdminSessions(),
        listAdminDevices(),
        listConsoleCommands(),
        getEditorExecutionState(),
        listAdminFiles({ limit: 10 }),
        listAdminFolders(),
      ]);
      overview = nextOverview;
      system = nextSystem;
      activity = nextActivity.items || [];
      users = nextUsers.users || [];
      sessions = nextSessions.sessions || [];
      devices = nextDevices.devices || [];
      commands = nextCommands.commands || [];
      executionMessage = nextExecution.message;
      fileInventoryCount = nextFiles.files?.length || 0;
      folderCount = nextFolders.folders?.length || 0;
    } catch (nextError) {
      error = nextError instanceof Error ? nextError.message : "Failed to load admin data";
    } finally {
      loading = false;
    }
  });

  $: memoryUsed = Math.max(
    0,
    Number(system?.total_memory_bytes || 0) - Number(system?.available_memory_bytes || 0),
  );
  $: diskUsed = Math.max(
    0,
    Number(system?.data_dir_total_bytes || 0) - Number(system?.data_dir_free_bytes || 0),
  );
</script>

<section class="view-grid">
  <div class="hero-card compact">
    <div>
      <div class="eyebrow">Operations</div>
      <h1>Organized control center, not one long admin wall.</h1>
      <p>
        Users, sessions, storage, diagnostics, and the safe console become separate
        operational surfaces with independent loading and clear destructive-action gates.
      </p>
    </div>
  </div>

  <div class="metric-row">
    <article class="metric-card"><span>System health</span><strong>{error ? "Needs auth" : "Healthy"}</strong></article>
    <article class="metric-card"><span>Storage used</span><strong>{overview ? formatBytes(overview.storage_bytes_used) : "—"}</strong></article>
    <article class="metric-card"><span>Files</span><strong>{overview?.file_count ?? "—"}</strong></article>
    <article class="metric-card"><span>Notes</span><strong>{overview?.note_count ?? "—"}</strong></article>
  </div>

  <div class="split-view">
    <article class="panel-card">
      <header class="panel-head">
        <strong>Access control</strong>
        <span>users, devices, sessions</span>
      </header>
      {#if loading}
        <div class="empty-copy">Loading admin overview…</div>
      {:else if error}
        <div class="empty-copy">Admin API unavailable in this session: {error}</div>
      {:else}
        <div class="stack-list">
          <div class="stack-card">
            <strong>server version</strong>
            <span>v{overview?.version || "unknown"} · pinned {overview?.pinned_count ?? 0} · tags {overview?.tag_count ?? 0}</span>
          </div>
          <div class="stack-card">
            <strong>system host</strong>
            <span>{system?.hostname || "host"} · {system?.os || "os"} · {system?.arch || "arch"}</span>
          </div>
          <div class="stack-card">
            <strong>users</strong>
            <span>{users.length} account(s) · {users.filter((user) => user.disabled).length} disabled</span>
          </div>
          <div class="stack-card">
            <strong>sessions</strong>
            <span>{sessions.length} active session(s) · {devices.length} trusted device(s)</span>
          </div>
        </div>
      {/if}
    </article>

    <article class="panel-card">
      <header class="panel-head">
        <strong>Recent activity</strong>
        <span>files, notes, workspaces</span>
      </header>
      {#if loading}
        <div class="empty-copy">Loading activity…</div>
      {:else if error}
        <div class="empty-copy">Activity feed needs an admin session and live backend data.</div>
      {:else if activity.length === 0}
        <div class="empty-copy">No recent activity yet.</div>
      {:else}
        <div class="stack-list">
          {#each activity as item}
            <div class="stack-card">
              <strong>{item.title}</strong>
              <span>{item.kind} · {item.detail} · {formatRelativeDate(item.occurred_at * 1000)}</span>
            </div>
          {/each}
        </div>
      {/if}
    </article>
  </div>

  <div class="split-view">
    <article class="panel-card">
      <header class="panel-head">
        <strong>Safe console</strong>
        <span>allowlisted commands only</span>
      </header>
      {#if loading}
        <div class="empty-copy">Loading telemetry…</div>
      {:else if error}
        <div class="empty-copy">The operations shell is ready; a signed-in admin session is needed to populate it.</div>
      {:else}
        <div class="code-preview">
          <span>&gt; storage_stats</span>
          <span>{"{"}"storage_bytes_used": {overview?.storage_bytes_used ?? 0}, "corrupt_file_count": {overview?.corrupt_file_count ?? 0}{"}"}</span>
          <span>&gt; host_status</span>
          <span>{"{"}"memory_used": "{formatBytes(memoryUsed)}", "disk_used": "{formatBytes(diskUsed)}"{"}"}</span>
          <span>&gt; execution_check</span>
          <span>{executionMessage}</span>
        </div>
      {/if}
    </article>
  </div>

  <div class="split-view">
    <article class="panel-card">
      <header class="panel-head">
        <strong>Console commands</strong>
        <span>safe allowlist</span>
      </header>
      <div class="stack-list">
        {#each commands.slice(0, 6) as command}
          <div class="stack-card">
            <strong>{command.name}</strong>
            <span>{command.category} · {command.description}</span>
          </div>
        {/each}
      </div>
    </article>

    <article class="panel-card">
      <header class="panel-head">
        <strong>Storage inventory</strong>
        <span>files and folders</span>
      </header>
        <div class="stack-list">
        <div class="stack-card">
          <strong>Folders</strong>
          <span>{folderCount} folder group(s) indexed</span>
        </div>
        <div class="stack-card">
          <strong>Files</strong>
          <span>{fileInventoryCount || (overview?.file_count ?? 0)} indexed objects</span>
        </div>
        <div class="stack-card">
          <strong>Folders API</strong>
          <span>Use the dedicated folder endpoint for tree and cleanup operations.</span>
        </div>
      </div>
    </article>
  </div>
</section>
