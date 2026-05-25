<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { files } from '$lib/stores/drive';
  import { notes } from '$lib/stores/notes';
  import { workspaces } from '$lib/stores/workspace';
  import { api } from '$lib/api';
  import { currentView } from '$lib/stores/ui';
  import { onMount } from 'svelte';

  let status: any = null;
  let loading = true;

  onMount(async () => {
    try {
      status = await api.getStatus();
    } finally {
      loading = false;
    }
  });

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function switchView(view: string) {
    $currentView = view as any;
  }
</script>

<div class="home-view">
  <div class="hero">
    <div class="hero-content">
      <h1>Welcome to TSSP</h1>
      <p>Your personal cloud OS</p>
    </div>
  </div>

  {#if loading}
    <div class="loading">
      <div class="spinner" />
      Loading...
    </div>
  {:else}
    <div class="apps-grid">
      <!-- Drive App -->
      <button class="app-card" on:click={() => switchView('drive')}>
        <div class="app-header">
          <div class="app-icon drive">
            <Icons.HardDrive size={32} />
          </div>
          <h2>Cloud Drive</h2>
        </div>
        <p class="app-description">Store, browse, and share files</p>
        <div class="app-stats">
          <div class="stat">
            <span class="stat-value">{$files.length}</span>
            <span class="stat-label">Files</span>
          </div>
          {#if status}
            <div class="stat">
              <span class="stat-value">{formatBytes(status.storage_bytes_used)}</span>
              <span class="stat-label">Used</span>
            </div>
          {/if}
        </div>
        <div class="app-action">
          Open Drive <Icons.ChevronRight size={16} />
        </div>
      </button>

      <!-- Notes App -->
      <button class="app-card" on:click={() => switchView('notes')}>
        <div class="app-header">
          <div class="app-icon notes">
            <Icons.BookOpen size={32} />
          </div>
          <h2>Notes</h2>
        </div>
        <p class="app-description">Write and organize thoughts</p>
        <div class="app-stats">
          <div class="stat">
            <span class="stat-value">{$notes.length}</span>
            <span class="stat-label">Notes</span>
          </div>
        </div>
        <div class="app-action">
          Open Notes <Icons.ChevronRight size={16} />
        </div>
      </button>

      <!-- Workspace App -->
      <button class="app-card" on:click={() => switchView('workspace')}>
        <div class="app-header">
          <div class="app-icon workspace">
            <Icons.Code2 size={32} />
          </div>
          <h2>Workspace</h2>
        </div>
        <p class="app-description">Code editor with multiple files</p>
        <div class="app-stats">
          <div class="stat">
            <span class="stat-value">{$workspaces.length}</span>
            <span class="stat-label">Workspaces</span>
          </div>
        </div>
        <div class="app-action">
          Open Workspace <Icons.ChevronRight size={16} />
        </div>
      </button>

      <!-- Operations App -->
      <button class="app-card" on:click={() => switchView('operations')}>
        <div class="app-header">
          <div class="app-icon operations">
            <Icons.Settings size={32} />
          </div>
          <h2>Operations</h2>
        </div>
        <p class="app-description">Admin, diagnostics, and system</p>
        <div class="app-stats">
          {#if status}
            <div class="stat">
              <span class="stat-value">{status.file_count}</span>
              <span class="stat-label">Total Files</span>
            </div>
          {/if}
        </div>
        <div class="app-action">
          Open Console <Icons.ChevronRight size={16} />
        </div>
      </button>
    </div>

    {#if status}
      <div class="quick-stats">
        <div class="stats-header">System Status</div>
        <div class="stats-grid">
          <div class="quick-stat">
            <span class="label">Files</span>
            <span class="value">{status.file_count}</span>
          </div>
          <div class="quick-stat">
            <span class="label">Storage Used</span>
            <span class="value">{formatBytes(status.storage_bytes_used)}</span>
          </div>
          <div class="quick-stat">
            <span class="label">Total Capacity</span>
            <span class="value">{formatBytes(status.storage_total_bytes)}</span>
          </div>
          <div class="quick-stat">
            <span class="label">Usage</span>
            <span class="value">
              {Math.round((status.storage_bytes_used / status.storage_total_bytes) * 100)}%
            </span>
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .home-view {
    flex: 1;
    overflow: auto;
    background: linear-gradient(135deg, rgba(91, 227, 154, 0.05) 0%, rgba(255, 95, 162, 0.05) 100%);
  }

  .hero {
    padding: 60px 24px;
    text-align: center;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .hero-content h1 {
    margin: 0;
    font-size: var(--fs-40);
    font-weight: 700;
    color: var(--text);
  }

  .hero-content p {
    margin: 8px 0 0;
    font-size: var(--fs-16);
    color: var(--text-2);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 80px 24px;
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

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .apps-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 20px;
    padding: 40px 24px;
    max-width: 1200px;
    margin: 0 auto;
  }

  .app-card {
    padding: 24px;
    border: 1px solid var(--border);
    border-radius: var(--r-5);
    background: var(--surface);
    cursor: pointer;
    transition: all 0.25s;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .app-card:hover {
    border-color: var(--blue);
    background: var(--surface-2);
    transform: translateY(-4px);
    box-shadow: 0 8px 16px rgba(110, 168, 255, 0.1);
  }

  .app-header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .app-icon {
    width: 56px;
    height: 56px;
    border-radius: var(--r-3);
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
  }

  .app-icon.drive {
    background: linear-gradient(135deg, #6ea8ff, #4682d9);
  }

  .app-icon.notes {
    background: linear-gradient(135deg, #fbbf24, #f59e0b);
  }

  .app-icon.workspace {
    background: linear-gradient(135deg, #34d399, #10b981);
  }

  .app-icon.operations {
    background: linear-gradient(135deg, #a394ff, #8b5cf6);
  }

  .app-header h2 {
    margin: 0;
    font-size: var(--fs-18);
    color: var(--text);
  }

  .app-description {
    margin: 0;
    font-size: var(--fs-13);
    color: var(--text-2);
  }

  .app-stats {
    display: flex;
    gap: 16px;
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat-value {
    font-size: var(--fs-16);
    font-weight: 600;
    color: var(--text);
  }

  .stat-label {
    font-size: 11px;
    color: var(--muted);
  }

  .app-action {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--fs-12);
    font-weight: 500;
    color: var(--blue);
    margin-top: auto;
  }

  .quick-stats {
    max-width: 1200px;
    margin: 0 auto 40px;
    padding: 24px;
    border: 1px solid var(--border);
    border-radius: var(--r-5);
    background: var(--surface);
  }

  .stats-header {
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
    margin-bottom: 16px;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 16px;
  }

  .quick-stat {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px;
    border-radius: var(--r-3);
    background: var(--surface-2);
  }

  .quick-stat .label {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    font-weight: 500;
  }

  .quick-stat .value {
    font-size: var(--fs-16);
    font-weight: 600;
    color: var(--text);
  }
</style>
