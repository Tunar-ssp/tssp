<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    totalFiles?: number;
    totalLines?: number;
    totalWords?: number;
    totalChars?: number;
    diskUsage?: number;
    lastSaved?: number;
    isDirty?: boolean;
  }

  let {
    totalFiles = 0,
    totalLines = 0,
    totalWords = 0,
    totalChars = 0,
    diskUsage = 0,
    lastSaved = 0,
    isDirty = false,
  }: $$Props = $props();

  function formatBytes(bytes: number): string {
    if (!bytes) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    const value = bytes / 1024 ** index;
    return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
  }

  function formatTime(timestamp: number): string {
    if (!timestamp) return 'never';
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return 'just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  }

  let stats = $derived([
    { icon: Icons.FileText, label: 'Files', value: totalFiles.toString() },
    { icon: Icons.Layers, label: 'Lines', value: totalLines.toLocaleString() },
    { icon: Icons.Type, label: 'Words', value: totalWords.toLocaleString() },
    { icon: Icons.Hash, label: 'Characters', value: totalChars.toLocaleString() },
  ]);
</script>

<div class="workspace-stats">
  <div class="stats-header">
    <h3>Workspace Stats</h3>
    {#if isDirty}
      <span class="unsaved-indicator" title="Unsaved changes">●</span>
    {/if}
  </div>

  <div class="stats-grid">
    {#each stats as stat (stat.label)}
      <div class="stat-card">
        <div class="stat-icon">
          <svelte:component this={stat.icon} size={16} />
        </div>
        <div class="stat-info">
          <span class="stat-label">{stat.label}</span>
          <span class="stat-value">{stat.value}</span>
        </div>
      </div>
    {/each}
  </div>

  <div class="stats-section">
    <div class="section-title">Storage</div>
    <div class="storage-info">
      <Icons.HardDrive size={14} />
      <span>{formatBytes(diskUsage)}</span>
    </div>
  </div>

  <div class="stats-section">
    <div class="section-title">Last Saved</div>
    <div class="saved-info">
      <Icons.Save size={14} />
      <span>{formatTime(lastSaved)}</span>
    </div>
  </div>
</div>

<style>
  .workspace-stats {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
    background: var(--surface);
    border-radius: 12px;
    border: 1px solid var(--border);
  }

  .stats-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }

  .stats-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text);
  }

  .unsaved-indicator {
    color: var(--orange);
    font-size: 10px;
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .stats-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .stat-card {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px;
    background: var(--bg);
    border-radius: 8px;
    border: 1px solid var(--hairline);
  }

  .stat-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-2);
    flex-shrink: 0;
  }

  .stat-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat-label {
    font-size: 10px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .stat-value {
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
  }

  .stats-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 8px;
    border-top: 1px solid var(--hairline);
  }

  .section-title {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .storage-info,
  .saved-info {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text);
    font-size: 13px;
  }
</style>
