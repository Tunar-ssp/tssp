<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { api } from '$lib/api';
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
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function formatPercent(used: number, total: number): number {
    return total > 0 ? Math.round((used / total) * 100) : 0;
  }
</script>

<div class="system-status">
  {#if loading}
    <div class="loading">
      <div class="spinner" />
      Loading system status...
    </div>
  {:else if status}
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-header">
          <Icons.File size={20} />
          <span class="stat-title">Files</span>
        </div>
        <div class="stat-value">{status.file_count}</div>
        <div class="stat-subtitle">objects stored</div>
      </div>

      <div class="stat-card">
        <div class="stat-header">
          <Icons.HardDrive size={20} />
          <span class="stat-title">Storage</span>
        </div>
        <div class="stat-value">
          {formatBytes(status.storage_bytes_used)}
        </div>
        <div class="stat-subtitle">
          of {formatBytes(status.storage_total_bytes)}
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-header">
          <Icons.Activity size={20} />
          <span class="stat-title">Usage</span>
        </div>
        <div class="stat-value">
          {formatPercent(status.storage_bytes_used, status.storage_total_bytes)}%
        </div>
        <div class="progress-bar">
          <div
            class="progress-fill"
            style="width: {formatPercent(status.storage_bytes_used, status.storage_total_bytes)}%"
          />
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-header">
          <Icons.Server size={20} />
          <span class="stat-title">Status</span>
        </div>
        <div class="status-badge ok">
          <div class="status-dot" />
          Healthy
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .system-status {
    flex: 1;
    padding: 20px;
    overflow: auto;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
    height: 200px;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
  }

  .stat-card {
    padding: 16px;
    border-radius: var(--r-3);
    border: 1px solid var(--border);
    background: var(--surface);
  }

  .stat-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
    color: var(--text-2);
  }

  .stat-title {
    font-size: var(--fs-12);
    font-weight: 500;
  }

  .stat-value {
    font-size: var(--fs-24);
    font-weight: 600;
    color: var(--text);
    margin-bottom: 4px;
  }

  .stat-subtitle {
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .progress-bar {
    height: 6px;
    background: var(--surface-3);
    border-radius: 3px;
    overflow: hidden;
    margin-top: 8px;
  }

  .progress-fill {
    height: 100%;
    background: var(--blue);
    border-radius: 3px;
    transition: width 0.3s;
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: var(--fs-12);
    font-weight: 500;
  }

  .status-badge.ok {
    background: rgba(91, 227, 154, 0.1);
    color: var(--green);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: currentColor;
  }
</style>
