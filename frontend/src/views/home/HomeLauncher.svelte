<script lang="ts">
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import { api, type AdminActivityItem, type FileRecord, type Note, type Workspace } from '$lib/api';
  import { isAdmin, user } from '$lib/stores/auth';
  import { navigateTo } from '$lib/stores/ui';
  import LauncherHero from './components/LauncherHero.svelte';
  import LauncherApps from './components/LauncherApps.svelte';
  import LauncherStatus from './components/LauncherStatus.svelte';
  import LauncherRecentFiles from './components/LauncherRecentFiles.svelte';
  import LauncherRecentNotes from './components/LauncherRecentNotes.svelte';
  import LauncherRecentWorkspaces from './components/LauncherRecentWorkspaces.svelte';
  import LauncherActivityPanel from './components/LauncherActivityPanel.svelte';

  type Status = Awaited<ReturnType<typeof api.getStatus>>;
  type AdminSystem = Awaited<ReturnType<typeof api.getAdminSystem>>;

  let loading = $state(true);
  let errorMessage = $state('');
  let status = $state<Status | undefined>();
  let adminSystem = $state<AdminSystem | undefined>();
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
        const [statusData, filesData, notesData, workspacesData, systemData, activityData] = await Promise.all([
          ...baseRequests,
          api.getAdminSystem().catch(() => undefined),
          api.listAdminActivity(8),
        ]);

        status = statusData;
        recentFiles = filesData.files || [];
        recentNotes = notesData.notes || [];
        recentWorkspaces = workspacesData.workspaces || [];
        adminSystem = systemData;
        activityItems = activityData.items || [];
      } else {
        const [statusData, filesData, notesData, workspacesData] = await Promise.all(baseRequests);

        status = statusData;
        recentFiles = filesData.files || [];
        recentNotes = notesData.notes || [];
        recentWorkspaces = workspacesData.workspaces || [];
        adminSystem = undefined;
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
</script>

<div class="launcher">
  <div class="hero-row">
    <LauncherHero
      greeting={getGreeting()}
      dateLabel={formatDateLabel()}
      userName={$user?.name || 'operator'}
      {status}
      onUpload={requestUpload}
      onNewNote={() => navigateTo('notes')}
      onOpenWorkspace={() => navigateTo('workspace')}
      onOpenShare={() => navigateTo('drive')}
    />
    <LauncherStatus {status} system={adminSystem} />
  </div>

  <LauncherApps
    isAdmin={$isAdmin}
    fileCount={status?.file_count ?? 0}
    noteCount={status?.note_count ?? 0}
    workspaceCount={recentWorkspaces.length}
    storageBytes={status?.storage_bytes_used ?? 0}
    pinnedCount={status?.pinned_count ?? 0}
    onOpen={(view) => navigateTo(view)}
  />

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
    <LauncherRecentFiles
      files={recentFiles}
      onOpenFile={openFile}
      onOpenDrive={() => navigateTo('drive')}
    />

    <div class="column">
      <LauncherRecentNotes
        notes={recentNotes}
        onOpenNote={openNote}
        onOpenNotes={() => navigateTo('notes')}
      />

      <LauncherRecentWorkspaces
        workspaces={recentWorkspaces}
        onOpenWorkspace={openWorkspace}
        onOpenWorkspaces={() => navigateTo('workspace')}
      />
    </div>

    <LauncherActivityPanel
      isAdmin={$isAdmin}
      activityItems={activityItems}
      {status}
      workspaceCount={recentWorkspaces.length}
      onOpenAdmin={() => navigateTo('admin')}
    />
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

  .hero-row {
    display: grid;
    grid-template-columns: minmax(0, 1.4fr) minmax(320px, 380px);
    gap: 18px;
    align-items: start;
  }

  .message-panel {
    border: 1px solid rgba(255, 255, 255, 0.07);
    background: linear-gradient(180deg, rgba(20, 22, 29, 0.94), rgba(15, 16, 22, 0.92));
    box-shadow: var(--shadow-card);
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
    grid-template-columns: minmax(0, 1.2fr) minmax(300px, 1fr) minmax(300px, 1fr);
    gap: 18px;
    align-items: start;
  }

  .column {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 1260px) {
    .hero-row,
    .launcher-grid {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 720px) {
    .launcher {
      padding: 18px 16px 124px;
    }
  }
</style>
