<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    filePath?: string;
    fileName?: string;
    fileSize?: number;
    language?: string;
    lineCount?: number;
    wordCount?: number;
    charCount?: number;
    isDirty?: boolean;
    isLoading?: boolean;
  }

  let {
    filePath = '',
    fileName = 'untitled',
    fileSize = 0,
    language = 'text',
    lineCount = 0,
    wordCount = 0,
    charCount = 0,
    isDirty = false,
    isLoading = false,
  }: $$Props = $props();

  function formatBytes(bytes: number): string {
    if (!bytes) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    const value = bytes / 1024 ** index;
    return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
  }

  let stats = $derived.by(() => [
    { icon: Icons.FileText, label: 'Language', value: language },
    { icon: Icons.Layers, label: 'Lines', value: lineCount.toString() },
    { icon: Icons.Type, label: 'Words', value: wordCount.toString() },
    { icon: Icons.Hash, label: 'Characters', value: charCount.toString() },
    { icon: Icons.HardDrive, label: 'Size', value: formatBytes(fileSize) },
  ]);
</script>

<div class="file-info-panel">
  <div class="file-header">
    <div class="file-name-section">
      <Icons.File size={20} />
      <div class="name-info">
        <h4>{fileName}</h4>
        <p class="path">{filePath || 'unsaved'}</p>
      </div>
      {#if isDirty}
        <span class="dirty-indicator" title="Unsaved changes">●</span>
      {/if}
    </div>
  </div>

  <div class="stats-grid">
    {#each stats as stat (stat.label)}
      <div class="stat-item">
        <div class="stat-icon">
          <svelte:component this={stat.icon} size={16} />
        </div>
        <div class="stat-content">
          <span class="stat-label">{stat.label}</span>
          <span class="stat-value">{stat.value}</span>
        </div>
      </div>
    {/each}
  </div>

  {#if isLoading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading file information...</p>
    </div>
  {/if}
</div>

<style>
  .file-info-panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
    background: var(--surface);
    border-radius: 12px;
    border: 1px solid var(--border);
  }

  .file-header {
    border-bottom: 1px solid var(--border);
    padding-bottom: 12px;
  }

  .file-name-section {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--text);
  }

  .name-info {
    flex: 1;
    min-width: 0;
  }

  .name-info h4 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .path {
    margin: 4px 0 0;
    font-size: 12px;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dirty-indicator {
    color: var(--orange);
    font-size: 14px;
    flex-shrink: 0;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .stat-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
    background: var(--bg);
    border-radius: 8px;
    border: 1px solid var(--hairline);
  }

  .stat-icon {
    color: var(--text-2);
    flex-shrink: 0;
  }

  .stat-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat-label {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .stat-value {
    font-size: 13px;
    color: var(--text);
    font-weight: 500;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 20px;
    color: var(--muted);
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--surface-2);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
