<script lang="ts">
  import { onMount } from "svelte";
  import {
    getAdminActivity,
    getAdminOverview,
    getAdminSystem,
    listAdminDevices,
    listAdminFiles,
    listAdminFolders,
    listAdminSessions,
    listAdminUsers,
    listConsoleCommands,
    runConsoleCommand,
    type AdminActivityItem,
    type AdminDevice,
    type AdminOverview,
    type AdminSession,
    type AdminUser,
    type AdminSystem,
    type ConsoleCommand,
    type ConsoleOutput,
  } from "../../lib/api";
  import { opsSection, navigateOps, type OpsSection } from "../../lib/router";
  import { formatBytes, formatRelativeDate } from "../../lib/utils/format";

  const tabs: { id: OpsSection; label: string }[] = [
    { id: "overview", label: "Overview" },
    { id: "access", label: "Access" },
    { id: "files", label: "Files" },
    { id: "storage", label: "Storage" },
    { id: "console", label: "Console" },
  ];

  let overview: AdminOverview | null = null;
  let system: AdminSystem | null = null;
  let activity: AdminActivityItem[] = [];
  let users: AdminUser[] = [];
  let sessions: AdminSession[] = [];
  let devices: AdminDevice[] = [];
  let commands: ConsoleCommand[] = [];
  let adminFiles: Awaited<ReturnType<typeof listAdminFiles>>["files"] = [];
  let folders: Awaited<ReturnType<typeof listAdminFolders>>["folders"] = [];
  let loading = true;
  let error = "";
  let selectedCommand = "";
  let consoleOutput: ConsoleOutput | null = null;
  let consoleRunning = false;

  async function load() {
    loading = true;
    error = "";
    try {
      const [ov, sys, act, us, sess, dev, cmds, files, flds] = await Promise.all([
        getAdminOverview(),
        getAdminSystem(),
        getAdminActivity(),
        listAdminUsers(),
        listAdminSessions(),
        listAdminDevices(),
        listConsoleCommands(),
        listAdminFiles({ limit: 50 }),
        listAdminFolders(),
      ]);
      overview = ov;
      system = sys;
      activity = act.items || [];
      users = us.users || [];
      sessions = sess.sessions || [];
      devices = dev.devices || [];
      commands = cmds.commands || [];
      adminFiles = files.files || [];
      folders = flds.folders || [];
      if (!selectedCommand && commands[0]) selectedCommand = commands[0].name;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load operations data";
    } finally {
      loading = false;
    }
  }

  async function runCommand() {
    if (!selectedCommand) return;
    consoleRunning = true;
    try {
      consoleOutput = await runConsoleCommand(selectedCommand);
    } catch (e) {
      error = e instanceof Error ? e.message : "Command failed";
    } finally {
      consoleRunning = false;
    }
  }

  onMount(() => {
    void load();
    const timer = setInterval(() => void load(), 15000);
    return () => clearInterval(timer);
  });

  $: memUsed = Math.max(0, Number(system?.total_memory_bytes || 0) - Number(system?.available_memory_bytes || 0));
  $: diskUsed = Math.max(0, Number(system?.data_dir_total_bytes || 0) - Number(system?.data_dir_free_bytes || 0));
</script>

<section class="ops">
  <nav class="ops-tabs">
    {#each tabs as tab}
      <button type="button" class="ops-tab" class:active={$opsSection === tab.id} on:click={() => navigateOps(tab.id)}>
        {tab.label}
      </button>
    {/each}
  </nav>

  {#if loading && !overview}
    <div class="empty-state"><strong>Loading operations…</strong></div>
  {:else if error && !overview}
    <div class="empty-state"><strong>Operations unavailable</strong>{error}</div>
  {:else}
    {#if $opsSection === "overview"}
      <div class="metrics">
        <article><span>Files</span><strong>{overview?.file_count ?? 0}</strong></article>
        <article><span>Notes</span><strong>{overview?.note_count ?? 0}</strong></article>
        <article><span>Storage</span><strong>{formatBytes(overview?.storage_bytes_used || 0)}</strong></article>
        <article><span>Corrupt</span><strong>{overview?.corrupt_file_count ?? 0}</strong></article>
      </div>
      <div class="grid-2">
        <div class="panel">
          <h3>System</h3>
          <p>{system?.hostname} · {system?.os} · {system?.arch}</p>
          <p>Memory {formatBytes(memUsed)} · Disk {formatBytes(diskUsed)}</p>
          <p>Uptime {Math.floor((system?.uptime_seconds || overview?.uptime_seconds || 0) / 60)} min</p>
        </div>
        <div class="panel">
          <h3>Recent activity</h3>
          {#each activity.slice(0, 8) as item}
            <div class="row"><strong>{item.title}</strong><span>{formatRelativeDate(item.occurred_at * 1000)}</span></div>
          {:else}
            <p class="muted">No recent activity</p>
          {/each}
        </div>
      </div>
    {:else if $opsSection === "access"}
      <div class="grid-2">
        <div class="panel">
          <h3>Users ({users.length})</h3>
          <table class="table">
            <thead><tr><th>Name</th><th>Role</th><th>Status</th></tr></thead>
            <tbody>
              {#each users as user}
                <tr><td>{user.name}</td><td>{user.role}</td><td>{user.disabled ? "disabled" : "active"}</td></tr>
              {/each}
            </tbody>
          </table>
        </div>
        <div class="panel">
          <h3>Sessions ({sessions.length})</h3>
          {#each sessions.slice(0, 12) as s}
            <div class="row"><span class="mono">{s.token_preview}</span><span>{s.user_name || s.kind}</span></div>
          {/each}
          <h3>Devices ({devices.length})</h3>
          {#each devices.slice(0, 8) as d}
            <div class="row"><span>{d.device_name}</span><span class="mono">{d.user_id}</span></div>
          {/each}
        </div>
      </div>
    {:else if $opsSection === "files"}
      <div class="panel">
        <h3>File inventory</h3>
        <table class="table">
          <thead><tr><th>Name</th><th>Folder</th><th>Size</th><th>Visibility</th></tr></thead>
          <tbody>
            {#each adminFiles as file}
              <tr>
                <td>{file.name}</td>
                <td>{file.folder_path || "—"}</td>
                <td>{formatBytes(file.size_bytes)}</td>
                <td>{file.visibility}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else if $opsSection === "storage"}
      <div class="panel">
        <h3>Folder breakdown</h3>
        <table class="table">
          <thead><tr><th>Path</th><th>Files</th></tr></thead>
          <tbody>
            {#each folders as folder}
              <tr><td>{folder.path || "Bucket root"}</td><td>{folder.file_count}</td></tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else}
      <div class="grid-2">
        <div class="panel">
          <h3>Safe console</h3>
          <p class="muted">Allowlisted commands only — no arbitrary shell.</p>
          <select class="select" bind:value={selectedCommand}>
            {#each commands as cmd}
              <option value={cmd.name}>{cmd.name} — {cmd.description}</option>
            {/each}
          </select>
          <button type="button" class="btn btn-primary" disabled={consoleRunning} on:click={runCommand}>
            {consoleRunning ? "Running…" : "Run command"}
          </button>
        </div>
        <div class="panel">
          <h3>Output</h3>
          {#if consoleOutput}
            <pre class="console-out">{JSON.stringify(consoleOutput.output, null, 2)}</pre>
          {:else}
            <p class="muted">Select a command and run it.</p>
          {/if}
        </div>
      </div>
    {/if}
  {/if}
</section>

<style>
  .ops { display: flex; flex-direction: column; height: 100%; min-height: 0; overflow: auto; padding: 16px; gap: 16px; }
  .ops-tabs { display: flex; flex-wrap: wrap; gap: 6px; border-bottom: 1px solid var(--border); padding-bottom: 10px; }
  .ops-tab { border: 1px solid transparent; background: transparent; color: var(--text-muted); padding: 8px 12px; border-radius: var(--radius-sm); font-size: 13px; }
  .ops-tab.active { background: var(--brand-dim); color: var(--text); border-color: rgba(37,99,235,0.35); }
  .metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 10px; }
  .metrics article { border: 1px solid var(--border); border-radius: var(--radius-md); padding: 14px; background: var(--bg-card); }
  .metrics span { display: block; font-size: 11px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.06em; }
  .metrics strong { font-size: 22px; }
  .grid-2 { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
  .panel { border: 1px solid var(--border); border-radius: var(--radius-md); padding: 16px; background: var(--bg-card); }
  .panel h3 { margin: 0 0 12px; font-size: 14px; }
  .row { display: flex; justify-content: space-between; gap: 8px; padding: 6px 0; border-bottom: 1px solid var(--border); font-size: 13px; }
  .table { width: 100%; border-collapse: collapse; font-size: 13px; }
  .table th, .table td { padding: 8px; border-bottom: 1px solid var(--border); text-align: left; }
  .select { width: 100%; margin: 10px 0; padding: 8px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--bg-surface); }
  .console-out { background: #0a0a0c; border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; font-family: var(--font-mono); font-size: 12px; overflow: auto; max-height: 360px; }
  .muted { color: var(--text-muted); font-size: 13px; }
  @media (max-width: 900px) { .grid-2 { grid-template-columns: 1fr; } }
</style>
