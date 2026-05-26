<script lang="ts">
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import { api, type AdminActivityItem, type FileRecord, type Note, type Workspace } from '$lib/api';
  import FileIcon from '$lib/components/FileIcon.svelte';
  import { isAdmin, user } from '$lib/stores/auth';
  import { navigateTo, openCommandPalette } from '$lib/stores/ui';
  import { formatBytes, formatRelative } from '$lib/utils/formatters';

  type Status = Awaited<ReturnType<typeof api.getStatus>>;
  type AdminOverview = Awaited<ReturnType<typeof api.getAdminOverview>>;

  let loading = $state(true);
  let errorMessage = $state('');
  let status = $state<Status | null>(null);
  let adminOverview = $state<AdminOverview | null>(null);
  let recentFiles = $state<FileRecord[]>([]);
  let recentNotes = $state<Note[]>([]);
  let recentWorkspaces = $state<Workspace[]>([]);
  let activityItems = $state<AdminActivityItem[]>([]);

  onMount(async () => {
    await loadLauncher();
  });

  async function loadLauncher() {
    loading = true;
    errorMessage = '';

    try {
      const baseRequests = [
        api.getStatus(),
        api.listFiles(8),
        api.listNotes(6),
        api.listWorkspaces(5),
      ] as const;

      if ($isAdmin) {
        const [statusData, filesData, notesData, workspacesData, overviewData, activityData] = await Promise.all([
          ...baseRequests,
          api.getAdminOverview(),
          api.listAdminActivity(8),
        ]);

        status = statusData;
        recentFiles = filesData.files || [];
        recentNotes = notesData.notes || [];
        recentWorkspaces = workspacesData.workspaces || [];
        adminOverview = overviewData;
        activityItems = activityData.items || [];
      } else {
        const [statusData, filesData, notesData, workspacesData] = await Promise.all(baseRequests);

        status = statusData;
        recentFiles = filesData.files || [];
        recentNotes = notesData.notes || [];
        recentWorkspaces = workspacesData.workspaces || [];
        adminOverview = null;
        activityItems = [];
      }
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'Could not load launcher';
    } finally {
      loading = false;
    }
  }

  function formatDateLabel() {
    return new Date().toLocaleDateString('en-US', {
      weekday: 'long',
      month: 'short',
      day: 'numeric',
    }).toUpperCase();
  }

  function getGreeting() {
    const hour = new Date().getHours();
    if (hour < 12) return 'Good morning';
    if (hour < 18) return 'Good afternoon';
    return 'Good evening';
  }

  function formatUptime(seconds = 0) {
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    if (hours < 48) return `${hours}h ${minutes % 60}m`;
    return `${Math.floor(hours / 24)}d ${hours % 24}h`;
  }

  function fileAccent(file: FileRecord) {
    if (file.mime_type.startsWith('image/')) return 'var(--green)';
    if (file.mime_type.startsWith('video/')) return 'var(--pink)';
    if (file.mime_type.includes('json') || file.mime_type.includes('javascript') || file.mime_type.includes('text/')) return 'var(--orange)';
    return 'var(--blue)';
  }

  function requestUpload() {
    navigateTo('drive');
    if (typeof document !== 'undefined') {
      document.dispatchEvent(new CustomEvent('tssp:request-upload'));
    }
  }

  function openFile(file: FileRecord) {
    navigateTo('drive', { kind: 'file', id: file.id });
  }

  function openNote(note: Note) {
    navigateTo('notes', { kind: 'note', id: note.id });
  }

  function openWorkspace(workspace: Workspace) {
    navigateTo('workspace', { kind: 'workspace', id: workspace.id });
  }

  const quickActions = [
    { id: 'upload', label: 'Upload', icon: Icons.Upload, action: requestUpload },
    { id: 'note', label: 'New note', icon: Icons.FileText, action: () => navigateTo('notes') },
    { id: 'workspace', label: 'Open workspace', icon: Icons.Code2, action: () => navigateTo('workspace') },
    { id: 'share', label: 'Public links', icon: Icons.Share2, action: () => navigateTo('drive') },
    { id: 'command', label: 'Command palette', icon: Icons.Search, action: () => openCommandPalette() },
  ];
</script>

<div class="launcher">
  <section class="hero">
    <div class="hero-copy">
      <div class="eyebrow">{formatDateLabel()}</div>
      <h1>{getGreeting()}, <span>{$user?.name || 'operator'}</span></h1>
      <p>
        {#if status}
          Local cloud is available. <strong>{formatBytes(status.storage_bytes_used)}</strong> in use across
          <strong> {status.file_count}</strong> files, <strong>{status.note_count}</strong> notes, and
          <strong> {recentWorkspaces.length}</strong> workspaces.
        {:else}
          Restore your local cloud, notes, workspaces, and operations from one shell.
        {/if}
      </p>

      <button type="button" class="launcher-search" onclick={() => openCommandPalette()}>
        <Icons.Search size={18} />
        <span>Search files, notes, workspaces, public links</span>
        <kbd>⌘K</kbd>
      </button>

      <div class="quick-actions">
        {#each quickActions as action (action.id)}
          {@const Icon = action.icon}
          <button type="button" class="quick-action" onclick={action.action}>
            <Icon size={14} />
            <span>{action.label}</span>
          </button>
        {/each}
      </div>
    </div>

    <div class="status-stack">
      <article class="status-card">
        <div class="status-head">
          <div class="status-pill">
            <span class="status-dot"></span>
            <span>orange-pi</span>
          </div>
          <span class="status-version">{status?.version || 'tssp'}</span>
          <span class="status-uptime">up {formatUptime(status?.uptime_seconds || 0)}</span>
        </div>

        <div class="status-rings">
          <div class="ring green">
            <strong>{adminOverview?.system.cpu_percent ?? 'ok'}</strong>
            <span>CPU</span>
          </div>
          <div class="ring blue">
            <strong>{adminOverview?.system.memory_percent ?? 'local'}</strong>
            <span>Memory</span>
          </div>
          <div class="ring orange">
            <strong>{adminOverview?.system.disk_percent ?? 'ready'}</strong>
            <span>Disk</span>
          </div>
        </div>

        <div class="status-foot">
          <div class="status-line">
            <Icons.Globe size={13} />
            <span>{status?.public_url || 'LAN only'}</span>
          </div>
          <div class="status-line">
            <Icons.Activity size={13} />
            <span>{status?.recent_upload_count_24h || 0} uploads in 24h</span>
          </div>
        </div>
      </article>

      <div class="mini-grid">
        <article class="mini-card">
          <div class="mini-label">Storage</div>
          <div class="mini-value">{formatBytes(status?.storage_bytes_used || 0)}</div>
          <div class="mini-sub">{status?.file_count || 0} files tracked</div>
        </article>
        <article class="mini-card">
          <div class="mini-label">Pinned</div>
          <div class="mini-value">{status?.pinned_count || 0}</div>
          <div class="mini-sub">{status?.tag_count || 0} tags in use</div>
        </article>
      </div>
    </div>
  </section>

  {#if errorMessage}
    <section class="message-panel error">
      <Icons.CircleAlert size={16} />
      <div>
        <strong>Launcher data could not load</strong>
        <p>{errorMessage}</p>
      </div>
      <button type="button" onclick={loadLauncher}>Retry</button>
    </section>
  {/if}

  {#if loading}
    <section class="message-panel loading">
      <div class="spinner"></div>
      <div>
        <strong>Loading launcher</strong>
        <p>Restoring files, notes, workspaces, and system state.</p>
      </div>
    </section>
  {/if}

  <section class="launcher-grid">
    <article class="panel">
      <div class="panel-head">
        <div>
          <h2>Recent files</h2>
          <p>Jump back into Drive from the latest objects.</p>
        </div>
        <button type="button" class="link-btn" onclick={() => navigateTo('drive')}>Open Drive</button>
      </div>

      {#if recentFiles.length === 0}
        <div class="empty-card">
          <Icons.HardDrive size={24} />
          <strong>No files yet</strong>
          <p>Upload to Drive and the latest objects will appear here.</p>
        </div>
      {:else}
        <div class="file-grid">
          {#each recentFiles.slice(0, 8) as file (file.id)}
            <button type="button" class="file-card" onclick={() => openFile(file)}>
              <div class="file-preview" style={`--accent:${fileAccent(file)}`}>
                <FileIcon mimeType={file.mime_type} name={file.name} size={30} />
                {#if file.visibility === 'public'}
                  <span class="chip">Public</span>
                {/if}
              </div>
              <div class="file-meta">
                <strong>{file.name}</strong>
                <span>{formatBytes(file.size_bytes)} · {formatRelative(file.updated_at || file.uploaded_at)}</span>
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </article>

    <div class="column">
      <article class="panel">
        <div class="panel-head">
          <div>
            <h2>Recent notes</h2>
            <p>Pick up where you left off.</p>
          </div>
          <button type="button" class="link-btn" onclick={() => navigateTo('notes')}>Open Notes</button>
        </div>

        {#if recentNotes.length === 0}
          <div class="empty-card compact">
            <Icons.BookText size={20} />
            <strong>No notes yet</strong>
            <p>Create a note to start your knowledge base.</p>
          </div>
        {:else}
          <div class="note-list">
            {#each recentNotes.slice(0, 4) as note (note.id)}
              <button type="button" class="note-card" onclick={() => openNote(note)}>
                <div class="note-stripe"></div>
                <div class="note-body">
                  <div class="note-title-row">
                    <strong>{note.title || 'Untitled note'}</strong>
                    {#if note.pinned_at}
                      <Icons.Pin size={12} />
                    {/if}
                  </div>
                  <p>{note.body?.replace(/\s+/g, ' ').trim() || 'No content yet.'}</p>
                  <div class="note-footer">
                    <span>{formatRelative(note.updated_at)}</span>
                    {#if note.tags?.length}
                      <span>{note.tags.slice(0, 2).join(' · ')}</span>
                    {/if}
                  </div>
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </article>

      <article class="panel">
        <div class="panel-head">
          <div>
            <h2>Open workspaces</h2>
            <p>Workspace stays honest about editing, not execution.</p>
          </div>
          <button type="button" class="link-btn" onclick={() => navigateTo('workspace')}>Open Workspace</button>
        </div>

        {#if recentWorkspaces.length === 0}
          <div class="empty-card compact">
            <Icons.Code2 size={20} />
            <strong>No workspaces yet</strong>
            <p>Create a workspace to start editing local project files and docs.</p>
          </div>
        {:else}
          <div class="workspace-list">
            {#each recentWorkspaces.slice(0, 4) as workspace (workspace.id)}
              <button type="button" class="workspace-row" onclick={() => openWorkspace(workspace)}>
                <div class="workspace-icon">{workspace.name.slice(0, 1).toUpperCase()}</div>
                <div class="workspace-copy">
                  <strong>{workspace.name}</strong>
                  <span>{workspace.language || 'text'} · {formatRelative(workspace.updated_at)}</span>
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </article>
    </div>

    <article class="panel activity-panel">
      <div class="panel-head">
        <div>
          <h2>{$isAdmin ? 'Admin activity' : 'System posture'}</h2>
          <p>{$isAdmin ? 'Real audit events from the backend.' : 'Your local-first cloud is healthy and ready.'}</p>
        </div>
        {#if $isAdmin}
          <button type="button" class="link-btn" onclick={() => navigateTo('admin')}>Open Admin</button>
        {/if}
      </div>

      {#if $isAdmin && activityItems.length > 0}
        <div class="activity-list">
          {#each activityItems as item (item.id)}
            <div class="activity-row">
              <div class="activity-glyph">
                <Icons.Activity size={13} />
              </div>
              <div class="activity-copy">
                <strong>{item.title}</strong>
                <p>{item.detail}</p>
              </div>
              <span>{formatRelative(item.occurred_at)}</span>
            </div>
          {/each}
        </div>
      {:else}
        <div class="health-stack">
          <div class="health-card">
            <Icons.HardDrive size={18} />
            <div>
              <strong>Drive ready</strong>
              <p>{status?.file_count || 0} files indexed for Drive, media, public, and search.</p>
            </div>
          </div>
          <div class="health-card">
            <Icons.BookText size={18} />
            <div>
              <strong>Notes synced</strong>
              <p>{status?.note_count || 0} notes available with local autosave and tags.</p>
            </div>
          </div>
          <div class="health-card">
            <Icons.Code2 size={18} />
            <div>
              <strong>Workspace shell</strong>
              <p>{recentWorkspaces.length} active workspaces available for editing. Execution stays disabled unless supported.</p>
            </div>
          </div>
        </div>
      {/if}
    </article>
  </section>
</div>

<style>
  .launcher {
    flex: 1;
    overflow: auto;
    padding: 30px 28px 140px;
    display: flex;
    flex-direction: column;
    gap: 22px;
  }

  .hero {
    display: grid;
    grid-template-columns: minmax(0, 1.4fr) minmax(320px, 380px);
    gap: 18px;
    align-items: start;
  }

  .hero-copy {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .eyebrow {
    font-size: 11px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .hero-copy h1 {
    margin: 0;
    font-family: var(--ff-display);
    font-size: clamp(2.2rem, 4vw, 3rem);
    line-height: 0.95;
    letter-spacing: -0.04em;
  }

  .hero-copy h1 span {
    color: var(--green);
  }

  .hero-copy p {
    margin: 0;
    max-width: 680px;
    color: var(--text-2);
    font-size: 15px;
    line-height: 1.6;
  }

  .hero-copy strong {
    color: var(--text);
  }

  .launcher-search {
    height: 54px;
    width: min(100%, 640px);
    padding: 0 16px;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(12, 13, 18, 0.78);
    color: var(--text-2);
    display: flex;
    align-items: center;
    gap: 12px;
    box-shadow: 0 18px 42px rgba(0, 0, 0, 0.34);
    cursor: pointer;
  }

  .launcher-search span {
    flex: 1;
    text-align: left;
    font-size: 15px;
  }

  .launcher-search kbd {
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    padding: 4px 9px;
    font-size: 11px;
    font-family: var(--ff-mono);
  }

  .quick-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .quick-action {
    height: 34px;
    padding: 0 13px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: rgba(12, 13, 18, 0.65);
    color: var(--text-2);
    display: inline-flex;
    align-items: center;
    gap: 7px;
    cursor: pointer;
  }

  .quick-action:hover,
  .launcher-search:hover {
    border-color: var(--border-2);
    background: rgba(18, 20, 27, 0.9);
    color: var(--text);
  }

  .status-stack {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .status-card,
  .mini-card,
  .panel,
  .message-panel {
    border: 1px solid rgba(255, 255, 255, 0.07);
    background: linear-gradient(180deg, rgba(20, 22, 29, 0.94), rgba(15, 16, 22, 0.92));
    box-shadow: var(--shadow-card);
  }

  .status-card {
    border-radius: 20px;
    padding: 18px;
  }

  .status-head,
  .status-foot,
  .status-line {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-head {
    margin-bottom: 18px;
  }

  .status-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 28px;
    padding: 0 12px;
    border-radius: 999px;
    background: rgba(17, 20, 27, 0.9);
    border: 1px solid var(--border);
    font-size: 12px;
    color: var(--text);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: var(--success);
    box-shadow: 0 0 14px rgba(52, 211, 153, 0.7);
  }

  .status-version,
  .status-uptime,
  .status-line {
    font-size: 12px;
    color: var(--muted);
  }

  .status-version {
    margin-left: auto;
  }

  .status-rings {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
    margin-bottom: 16px;
  }

  .ring {
    min-height: 98px;
    border-radius: 18px;
    border: 1px solid var(--border);
    background:
      radial-gradient(circle at top, rgba(255, 255, 255, 0.06), transparent 58%),
      var(--surface);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    gap: 4px;
  }

  .ring strong {
    font-size: 20px;
    color: var(--text);
  }

  .ring span {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.14em;
  }

  .ring.green strong {
    color: var(--green);
  }

  .ring.blue strong {
    color: var(--blue);
  }

  .ring.orange strong {
    color: var(--orange);
  }

  .status-foot {
    justify-content: space-between;
    gap: 14px;
    flex-wrap: wrap;
    padding-top: 14px;
    border-top: 1px dashed var(--hairline);
  }

  .mini-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .mini-card {
    border-radius: 16px;
    padding: 14px;
  }

  .mini-label {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.14em;
  }

  .mini-value {
    margin-top: 8px;
    font-size: 22px;
    font-weight: 600;
    color: var(--text);
  }

  .mini-sub {
    margin-top: 4px;
    font-size: 12px;
    color: var(--muted);
  }

  .message-panel {
    border-radius: 18px;
    padding: 14px 16px;
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .message-panel strong {
    display: block;
    margin-bottom: 4px;
    color: var(--text);
    font-size: 14px;
  }

  .message-panel p {
    margin: 0;
    color: var(--muted);
    font-size: 13px;
  }

  .message-panel button {
    margin-left: auto;
    height: 34px;
    padding: 0 14px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text);
    cursor: pointer;
  }

  .message-panel.error {
    border-color: rgba(255, 107, 107, 0.28);
  }

  .message-panel.loading .spinner {
    width: 18px;
    height: 18px;
    border-width: 2px;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border-radius: 999px;
    border: 2px solid rgba(255, 255, 255, 0.12);
    border-top-color: var(--blue);
    animation: spin 0.8s linear infinite;
  }

  .launcher-grid {
    display: grid;
    grid-template-columns: minmax(0, 1.55fr) minmax(320px, 1fr) minmax(300px, 0.95fr);
    gap: 18px;
    align-items: start;
  }

  .column {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .panel {
    border-radius: 22px;
    padding: 18px;
  }

  .panel-head {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    margin-bottom: 16px;
  }

  .panel-head h2 {
    margin: 0;
    font-size: 17px;
    color: var(--text);
  }

  .panel-head p {
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 13px;
    line-height: 1.5;
  }

  .link-btn {
    margin-left: auto;
    border: none;
    background: none;
    color: var(--blue);
    font-size: 12px;
    cursor: pointer;
  }

  .file-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 12px;
  }

  .file-card,
  .note-card,
  .workspace-row {
    border: 1px solid var(--border);
    background: var(--surface);
    color: inherit;
    cursor: pointer;
  }

  .file-card {
    border-radius: 14px;
    padding: 10px;
    text-align: left;
  }

  .file-card:hover,
  .note-card:hover,
  .workspace-row:hover {
    border-color: var(--border-2);
    background: var(--surface-2);
  }

  .file-preview {
    position: relative;
    aspect-ratio: 4 / 3;
    border-radius: 10px;
    margin-bottom: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    background:
      linear-gradient(135deg, color-mix(in srgb, var(--accent) 24%, transparent), rgba(30, 32, 40, 0.95)),
      var(--surface-2);
    color: var(--text);
  }

  .chip {
    position: absolute;
    top: 8px;
    left: 8px;
    height: 22px;
    padding: 0 8px;
    border-radius: 999px;
    background: rgba(110, 168, 255, 0.14);
    color: var(--blue);
    display: inline-flex;
    align-items: center;
    font-size: 10px;
    font-family: var(--ff-mono);
  }

  .file-meta {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .file-meta strong,
  .workspace-copy strong {
    font-size: 13px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-meta span,
  .workspace-copy span,
  .note-footer {
    font-size: 11px;
    color: var(--muted);
  }

  .note-list,
  .workspace-list,
  .activity-list,
  .health-stack {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .note-card {
    position: relative;
    border-radius: 14px;
    padding: 12px 12px 12px 16px;
    display: flex;
    text-align: left;
  }

  .note-stripe {
    position: absolute;
    left: 0;
    top: 10px;
    bottom: 10px;
    width: 3px;
    border-radius: 3px;
    background: var(--green);
  }

  .note-body {
    width: 100%;
    padding-left: 4px;
  }

  .note-title-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .note-title-row strong {
    flex: 1;
    min-width: 0;
    text-align: left;
    font-size: 13px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-body p {
    margin: 8px 0 10px;
    font-size: 12px;
    color: var(--text-2);
    line-height: 1.55;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .note-footer {
    display: flex;
    justify-content: space-between;
    gap: 10px;
  }

  .workspace-row {
    border-radius: 14px;
    padding: 12px;
    display: flex;
    align-items: center;
    gap: 12px;
    text-align: left;
  }

  .workspace-icon {
    width: 34px;
    height: 34px;
    border-radius: 10px;
    background: linear-gradient(135deg, rgba(255, 138, 61, 0.9), rgba(255, 95, 162, 0.55));
    color: rgba(8, 9, 12, 0.86);
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--ff-mono);
    font-size: 13px;
    font-weight: 700;
  }

  .workspace-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .activity-panel {
    min-height: 100%;
  }

  .activity-row,
  .health-card,
  .empty-card {
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--surface);
  }

  .activity-row {
    display: grid;
    grid-template-columns: 28px minmax(0, 1fr) auto;
    align-items: start;
    gap: 10px;
    padding: 12px;
  }

  .activity-glyph {
    color: var(--blue);
  }

  .activity-copy strong,
  .health-card strong,
  .empty-card strong {
    display: block;
    color: var(--text);
    font-size: 13px;
  }

  .activity-copy p,
  .health-card p,
  .empty-card p {
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.5;
  }

  .activity-row > span {
    color: var(--muted);
    font-size: 11px;
    font-family: var(--ff-mono);
  }

  .health-card,
  .empty-card {
    padding: 14px;
    display: flex;
    gap: 12px;
    align-items: flex-start;
  }

  .empty-card {
    min-height: 140px;
    justify-content: center;
    flex-direction: column;
    align-items: flex-start;
  }

  .empty-card.compact {
    min-height: 110px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 1260px) {
    .hero,
    .launcher-grid {
      grid-template-columns: 1fr;
    }

    .file-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 720px) {
    .launcher {
      padding: 18px 16px 124px;
    }

    .file-grid {
      grid-template-columns: 1fr;
    }

    .quick-actions {
      overflow: auto;
      padding-bottom: 4px;
    }

    .launcher-search {
      width: 100%;
    }
  }
</style>
