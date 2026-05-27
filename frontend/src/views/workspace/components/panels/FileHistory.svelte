<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface HistoryEntry {
    id: string;
    timestamp: number;
    action: 'created' | 'modified' | 'renamed' | 'deleted';
    fileName: string;
    author?: string;
    message?: string;
    changeSize?: number;
  }

  interface $$Props {
    entries?: HistoryEntry[];
    onSelectEntry?: (entry: HistoryEntry) => void;
  }

  let {
    entries = [],
    onSelectEntry = () => {},
  }: $$Props = $props();

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (days === 0) {
      const hours = Math.floor(diff / (1000 * 60 * 60));
      if (hours === 0) {
        const minutes = Math.floor(diff / (1000 * 60));
        return `${minutes}m ago`;
      }
      return `${hours}h ago`;
    }
    if (days === 1) return 'yesterday';
    if (days < 7) return `${days}d ago`;

    return date.toLocaleDateString();
  }

  function getActionIcon(action: string) {
    switch (action) {
      case 'created':
        return Icons.Plus;
      case 'modified':
        return Icons.Edit;
      case 'renamed':
        return Icons.RefreshCw;
      case 'deleted':
        return Icons.Trash2;
      default:
        return Icons.Clock;
    }
  }

  function getActionLabel(action: string): string {
    switch (action) {
      case 'created':
        return 'Created';
      case 'modified':
        return 'Modified';
      case 'renamed':
        return 'Renamed';
      case 'deleted':
        return 'Deleted';
      default:
        return 'Updated';
    }
  }
</script>

<div class="file-history">
  <div class="history-header">
    <h3>File History</h3>
  </div>

  {#if entries.length === 0}
    <div class="empty-state">
      <Icons.Clock size={24} />
      <p>No history</p>
    </div>
  {:else}
    <div class="history-list">
      {#each entries as entry (entry.id)}
        <button
          type="button"
          class="history-entry"
          onclick={() => onSelectEntry(entry)}
        >
          <div class="entry-icon">
            <svelte:component this={getActionIcon(entry.action)} size={14} />
          </div>
          <div class="entry-content">
            <div class="entry-title">
              <span class="action">{getActionLabel(entry.action)}</span>
              <span class="file-name">{entry.fileName}</span>
            </div>
            {#if entry.message}
              <p class="entry-message">{entry.message}</p>
            {/if}
            <div class="entry-meta">
              {#if entry.author}
                <span>{entry.author}</span>
              {/if}
              <span>{formatDate(entry.timestamp)}</span>
              {#if entry.changeSize}
                <span>{entry.changeSize} bytes</span>
              {/if}
            </div>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .file-history {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
  }

  .history-header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .history-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-2);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 40px 16px;
    color: var(--muted);
    text-align: center;
  }

  .empty-state p {
    margin: 0;
    font-size: 14px;
  }

  .history-list {
    flex: 1;
    overflow-y: auto;
  }

  .history-entry {
    display: flex;
    gap: 12px;
    width: 100%;
    padding: 12px 16px;
    border: none;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
    border-bottom: 1px solid var(--hairline);
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .history-entry:hover {
    background: var(--surface-2);
  }

  .entry-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 4px;
    background: var(--bg);
    color: var(--text-2);
    flex-shrink: 0;
  }

  .entry-content {
    flex: 1;
    min-width: 0;
  }

  .entry-title {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }

  .action {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-2);
  }

  .file-name {
    font-size: 13px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .entry-message {
    margin: 0 0 6px;
    font-size: 12px;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .entry-meta {
    display: flex;
    gap: 12px;
    font-size: 10px;
    color: var(--dim);
    font-family: var(--ff-mono);
  }

  .entry-meta span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
