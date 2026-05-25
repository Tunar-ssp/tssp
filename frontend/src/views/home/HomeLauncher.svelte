<script lang="ts">
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import { api } from '$lib/api';
  import { currentView } from '$lib/stores/ui';
  import type { FileRecord, Note } from '$lib/api';

  type Status = Awaited<ReturnType<typeof api.getStatus>>;

  let status: Status | null = null;
  let loading = true;
  let recentFiles: FileRecord[] = [];
  let recentNotes: Note[] = [];
  let error = '';

  const actions = [
    { id: 'upload', label: 'Upload', icon: Icons.Upload, view: 'drive' },
    { id: 'note', label: 'New note', icon: Icons.FileText, view: 'notes' },
    { id: 'workspace', label: 'New workspace', icon: Icons.Code2, view: 'workspace' },
    { id: 'share', label: 'Share link', icon: Icons.Link2, view: 'drive' },
    { id: 'qr', label: 'QR', icon: Icons.QrCode, view: 'drive' },
  ];

  onMount(async () => {
    try {
      const [statusData, filesData, notesData] = await Promise.all([
        api.getStatus(),
        api.listFiles(8),
        api.listNotes(6),
      ]);
      status = statusData;
      recentFiles = filesData.files || [];
      recentNotes = notesData.notes || [];
    } catch (e) {
      error = e instanceof Error ? e.message : 'Could not load dashboard';
    } finally {
      loading = false;
    }
  });

  function getGreeting() {
    const hour = new Date().getHours();
    if (hour < 12) return 'Good morning';
    if (hour < 18) return 'Good afternoon';
    return 'Good evening';
  }

  function formatDate() {
    const date = new Date();
    return date.toLocaleDateString('en-US', { weekday: 'long', month: 'short', day: 'numeric' }).toUpperCase();
  }

  function formatBytes(bytes = 0): string {
    if (bytes <= 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    return `${(bytes / 1024 ** index).toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
  }

  function formatUptime(seconds = 0): string {
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    if (hours < 48) return `${hours}h ${minutes % 60}m`;
    return `${Math.floor(hours / 24)}d ${hours % 24}h`;
  }

  function openApp(view: string) {
    currentView.set(view);
  }
</script>

<div class="dashboard">
  <header class="dashboard-header">
    <div class="header-greeting">
      <div class="greeting-date">{formatDate()}</div>
      <h1>{getGreeting()}, <span class="highlight">User</span></h1>
      {#if status}
        <p class="system-status">
          Your Orange Pi is healthy. {formatBytes(status.storage_bytes_available)} free across 3 volumes.
          Last sync 12 seconds ago.
        </p>
      {/if}
    </div>

    <div class="header-actions">
      <input type="text" placeholder="Search files, notes, workspaces... or type '/' for commands" class="search-input" />
      {#each actions as action}
        <button
          type="button"
          class="action-btn"
          onclick={() => openApp(action.view)}
          title={action.label}
        >
          <svelte:component this={action.icon} size={16} />
          <span>{action.label}</span>
        </button>
      {/each}
    </div>
  </header>

  <div class="dashboard-content">
    <aside class="dashboard-sidebar">
      {#if status}
        <div class="status-card">
          <div class="status-header">
            <span class="status-badge online">● {status.version || 'unknown'}</span>
            <span class="status-host">tssp v2.0.1</span>
            <span class="status-uptime">up {formatUptime(status.uptime_seconds)}</span>
          </div>

          <div class="metrics-grid">
            <div class="metric-gauge">
              <div class="gauge-circle" style="--value: {status.cpu_percent || 28}">
                <span class="gauge-label">{status.cpu_percent || 28}%</span>
              </div>
              <span class="gauge-name">CPU</span>
            </div>
            <div class="metric-gauge">
              <div class="gauge-circle" style="--value: {Math.round((status.storage_bytes_used / (status.storage_bytes_used + status.storage_bytes_available)) * 100) || 52}">
                <span class="gauge-label">{Math.round((status.storage_bytes_used / (status.storage_bytes_used + status.storage_bytes_available)) * 100) || 52}%</span>
              </div>
              <span class="gauge-name">RAM</span>
            </div>
            <div class="metric-gauge">
              <div class="gauge-circle" style="--value: {Math.round((status.storage_bytes_used / (status.storage_bytes_used + status.storage_bytes_available)) * 100) || 38}">
                <span class="gauge-label">{Math.round((status.storage_bytes_used / (status.storage_bytes_used + status.storage_bytes_available)) * 100) || 38}%</span>
              </div>
              <span class="gauge-name">Disk</span>
            </div>
          </div>

          <div class="status-info">
            <Icons.Wifi size={14} />
            <span>Reachable on LAN · tssp.local</span>
          </div>

          <div class="status-panels">
            <div class="status-panel">
              <span class="panel-label">Storage</span>
              <strong>{status.file_count || 308} / 500 GB</strong>
              <div class="storage-bar">
                <div class="bar-segment docs" style="width: 40%"></div>
                <div class="bar-segment images" style="width: 35%"></div>
                <div class="bar-segment video" style="width: 20%"></div>
                <div class="bar-segment other" style="width: 5%"></div>
              </div>
            </div>
            <div class="status-panel">
              <span class="panel-label">Alerts</span>
              <strong>2 need review</strong>
              <div class="alert-items">
                <div class="alert-item warning">Backup overdue 2d</div>
                <div class="alert-item info">3 public links expire</div>
              </div>
            </div>
          </div>
        </div>
      {/if}
    </aside>

    <main class="dashboard-main">
      <section class="dashboard-section">
        <div class="section-header">
          <h2>Recent files</h2>
          <span class="section-count">{recentFiles.length} this week</span>
          {#if recentFiles.length > 0}
            <a href="#" onclick={() => openApp('drive')} class="section-link">Open Drive →</a>
          {/if}
        </div>
        <div class="files-grid">
          {#each recentFiles.slice(0, 4) as file}
            <div class="file-card" onclick={() => openApp('drive')}>
              <div class="file-icon" style="background: linear-gradient(135deg, {['#ff6b6b', '#4ecdc4', '#45b7d1', '#96ceb4'][Math.random() * 4 | 0]} 0%, {['#ff8e72', '#6dd5ed', '#6ec8e6', '#b8e6c2'][Math.random() * 4 | 0]} 100%)">
                {file.name.slice(0, 1).toUpperCase()}
              </div>
              <span class="file-name">{file.name}</span>
              <span class="file-size">{formatBytes(file.size_bytes)}</span>
            </div>
          {/each}
        </div>
      </section>

      <div class="dashboard-grid">
        <section class="dashboard-section">
          <div class="section-header">
            <h2>Recent notes</h2>
            <span class="section-count">6 pinned</span>
            {#if recentNotes.length > 0}
              <a href="#" onclick={() => openApp('notes')} class="section-link">Open Notes →</a>
            {/if}
          </div>
          <div class="notes-list">
            {#each recentNotes.slice(0, 3) as note}
              <div class="note-item" onclick={() => openApp('notes')}>
                <div class="note-accent"></div>
                <div class="note-content">
                  <strong>{note.title}</strong>
                  <small>{note.body?.slice(0, 60)}...</small>
                </div>
                <span class="note-meta">Now</span>
              </div>
            {/each}
          </div>
        </section>

        <section class="dashboard-section">
          <div class="section-header">
            <h2>Workspaces</h2>
            <span class="section-count">4 open</span>
          </div>
          <div class="workspaces-list">
            {#each ['tssp-web', 'house-budget', 'scratch'] as workspace}
              <div class="workspace-item" onclick={() => openApp('workspace')}>
                <div class="workspace-icon">{workspace[0].toUpperCase()}</div>
                <div>
                  <strong>{workspace}</strong>
                  <small>misc • 12 files</small>
                </div>
              </div>
            {/each}
          </div>
        </section>
      </div>

      <section class="dashboard-section activity-section">
        <div class="section-header">
          <h2>Activity</h2>
          <span class="section-link">live</span>
        </div>
        <div class="activity-list">
          <div class="activity-item">
            <Icons.Upload size={14} />
            <span>Uploaded 14 files to /photos/may</span>
            <span class="time">just now</span>
          </div>
          <div class="activity-item">
            <Icons.Share2 size={14} />
            <span>Shared lease-renewal.pdf</span>
            <span class="time">1h</span>
          </div>
          <div class="activity-item">
            <Icons.User size={14} />
            <span>Mira signed in from iPhone</span>
            <span class="time">1h</span>
          </div>
          <div class="activity-item">
            <Icons.CheckCircle size={14} />
            <span>Integrity check passed</span>
            <span class="time">6h</span>
          </div>
        </div>
      </section>
    </main>
  </div>
</div>

<style>
  .dashboard {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg);
  }

  .dashboard-header {
    padding: var(--s-6);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .header-greeting {
    margin-bottom: var(--s-4);
  }

  .greeting-date {
    font-size: var(--fs-11);
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 1px;
    margin-bottom: var(--s-1);
  }

  .header-greeting h1 {
    margin: 0;
    font-size: var(--fs-32);
    font-weight: 700;
    color: var(--text);
    line-height: 1.2;
  }

  .highlight {
    color: var(--green);
  }

  .system-status {
    margin: var(--s-2) 0 0;
    font-size: var(--fs-13);
    color: var(--text-2);
  }

  .header-actions {
    display: flex;
    gap: var(--s-2);
    align-items: center;
  }

  .search-input {
    flex: 1;
    max-width: 600px;
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--bg);
    color: var(--text);
    font-size: var(--fs-13);
    outline: none;
  }

  .search-input::placeholder {
    color: var(--muted);
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: var(--s-1);
    padding: var(--s-2) var(--s-3);
    border: none;
    border-radius: var(--r-2);
    background: var(--blue);
    color: white;
    font-size: var(--fs-12);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .action-btn:hover {
    background: var(--blue-dark, var(--blue));
    transform: translateY(-1px);
  }

  .dashboard-content {
    flex: 1;
    display: flex;
    gap: var(--s-6);
    padding: var(--s-6);
    overflow: auto;
  }

  .dashboard-sidebar {
    flex-shrink: 0;
    width: 360px;
  }

  .status-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-3);
    padding: var(--s-4);
  }

  .status-header {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
    font-size: var(--fs-12);
    color: var(--text-2);
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .status-badge.online {
    color: var(--green);
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }

  .metric-gauge {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-2);
  }

  .gauge-circle {
    width: 60px;
    height: 60px;
    border-radius: 50%;
    background: conic-gradient(var(--blue) 0deg calc(var(--value) * 3.6deg), var(--surface-2) calc(var(--value) * 3.6deg));
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .gauge-label {
    font-size: var(--fs-11);
    font-weight: 600;
    color: var(--text);
  }

  .gauge-name {
    font-size: var(--fs-11);
    color: var(--muted);
  }

  .status-info {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    margin-bottom: var(--s-3);
    background: var(--bg);
    border-radius: var(--r-2);
    font-size: var(--fs-12);
    color: var(--text-2);
  }

  .status-panels {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .status-panel {
    padding: var(--s-3);
    background: var(--bg);
    border-radius: var(--r-2);
  }

  .panel-label {
    display: block;
    font-size: var(--fs-11);
    color: var(--muted);
    margin-bottom: var(--s-1);
  }

  .status-panel strong {
    display: block;
    font-size: var(--fs-14);
    color: var(--text);
    margin-bottom: var(--s-2);
  }

  .storage-bar {
    display: flex;
    height: 4px;
    border-radius: 2px;
    overflow: hidden;
    gap: 1px;
    background: var(--surface-2);
  }

  .bar-segment {
    flex: 1;
    background: var(--blue);
  }

  .bar-segment.images {
    background: var(--green);
  }

  .bar-segment.video {
    background: var(--orange);
  }

  .bar-segment.other {
    background: var(--muted);
  }

  .alert-items {
    display: flex;
    flex-direction: column;
    gap: var(--s-1);
  }

  .alert-item {
    font-size: var(--fs-12);
    padding: var(--s-1) var(--s-2);
    border-radius: var(--r-1);
    background: var(--surface-2);
  }

  .alert-item.warning {
    background: rgba(251, 191, 36, 0.1);
    color: var(--orange);
  }

  .alert-item.info {
    background: rgba(110, 168, 255, 0.1);
    color: var(--blue);
  }

  .dashboard-main {
    flex: 1;
    min-width: 0;
  }

  .dashboard-section {
    margin-bottom: var(--s-6);
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }

  .section-header h2 {
    margin: 0;
    font-size: var(--fs-16);
    font-weight: 600;
    color: var(--text);
  }

  .section-count {
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .section-link {
    margin-left: auto;
    font-size: var(--fs-12);
    color: var(--blue);
    text-decoration: none;
    cursor: pointer;
  }

  .section-link:hover {
    color: var(--blue-dark, var(--blue));
  }

  .files-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: var(--s-4);
  }

  .file-card {
    padding: var(--s-4);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-3);
    cursor: pointer;
    transition: all 0.2s;
    text-align: center;
  }

  .file-card:hover {
    background: var(--surface-2);
    border-color: var(--blue);
  }

  .file-icon {
    width: 48px;
    height: 48px;
    border-radius: var(--r-2);
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-weight: 600;
    font-size: var(--fs-20);
    margin: 0 auto var(--s-2);
  }

  .file-name {
    display: block;
    font-size: var(--fs-12);
    font-weight: 500;
    color: var(--text);
    margin-bottom: var(--s-1);
  }

  .file-size {
    display: block;
    font-size: var(--fs-11);
    color: var(--muted);
  }

  .dashboard-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--s-6);
  }

  .notes-list {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .note-item {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-3);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    cursor: pointer;
    transition: all 0.2s;
  }

  .note-item:hover {
    background: var(--surface-2);
    border-color: var(--blue);
  }

  .note-accent {
    width: 3px;
    height: 40px;
    background: var(--blue);
    border-radius: 2px;
  }

  .note-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .note-content strong {
    font-size: var(--fs-13);
    color: var(--text);
  }

  .note-content small {
    font-size: var(--fs-11);
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-meta {
    font-size: var(--fs-11);
    color: var(--muted);
    white-space: nowrap;
  }

  .workspaces-list {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .workspace-item {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-3);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    cursor: pointer;
    transition: all 0.2s;
  }

  .workspace-item:hover {
    background: var(--surface-2);
  }

  .workspace-icon {
    width: 40px;
    height: 40px;
    border-radius: var(--r-2);
    background: var(--orange);
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-weight: 600;
    font-size: var(--fs-14);
    flex-shrink: 0;
  }

  .workspace-item strong {
    display: block;
    font-size: var(--fs-13);
    color: var(--text);
  }

  .workspace-item small {
    display: block;
    font-size: var(--fs-11);
    color: var(--muted);
  }

  .activity-section {
    margin-bottom: var(--s-2);
  }

  .activity-list {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .activity-item {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-3);
    background: var(--surface);
    border-left: 2px solid var(--blue);
    border-radius: var(--r-2);
    font-size: var(--fs-12);
    color: var(--text-2);
  }

  .activity-item svg {
    flex-shrink: 0;
    color: var(--blue);
  }

  .activity-item span:last-child {
    margin-left: auto;
    color: var(--muted);
    font-size: var(--fs-11);
  }

  @media (max-width: 768px) {
    .dashboard-content {
      flex-direction: column;
    }

    .dashboard-sidebar {
      width: 100%;
    }

    .files-grid {
      grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    }

    .dashboard-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
