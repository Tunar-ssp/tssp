<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface CommitInfo {
    id: string;
    message: string;
    author: string;
    timestamp: number;
    files: number;
    additions: number;
    deletions: number;
  }

  interface $$Props {
    commits?: CommitInfo[];
    currentBranch?: string;
    onViewCommit?: (commit: CommitInfo) => void;
  }

  let {
    commits = [],
    currentBranch = 'main',
    onViewCommit = () => {},
  }: $$Props = $props();

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return 'just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  }

  function truncateHash(hash: string): string {
    return hash.substring(0, 7);
  }
</script>

<div class="version-control">
  <div class="vc-header">
    <div class="branch-info">
      <Icons.GitBranch size={14} />
      <span class="branch-name">{currentBranch}</span>
    </div>
  </div>

  {#if commits.length === 0}
    <div class="empty-state">
      <Icons.GitMerge size={24} />
      <p>No commits</p>
      <small>Commit history will appear here</small>
    </div>
  {:else}
    <div class="commits-list">
      {#each commits as commit (commit.id)}
        <button
          type="button"
          class="commit-item"
          onclick={() => onViewCommit(commit)}
        >
          <div class="commit-head">
            <span class="commit-hash">{truncateHash(commit.id)}</span>
            <span class="commit-message">{commit.message}</span>
          </div>
          <div class="commit-meta">
            <span class="author">{commit.author}</span>
            <span class="date">{formatDate(commit.timestamp)}</span>
          </div>
          <div class="commit-stats">
            <span class="files">{commit.files} files</span>
            <span class="additions">+{commit.additions}</span>
            <span class="deletions">-{commit.deletions}</span>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .version-control {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
  }

  .vc-header {
    display: flex;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .branch-info {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text);
    font-size: 13px;
  }

  .branch-name {
    font-weight: 600;
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

  .empty-state small {
    font-size: 11px;
    color: var(--dim);
  }

  .commits-list {
    flex: 1;
    overflow-y: auto;
  }

  .commit-item {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 100%;
    padding: 12px 16px;
    border-bottom: 1px solid var(--hairline);
    background: transparent;
    border: none;
    text-align: left;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .commit-item:hover {
    background: var(--surface-2);
  }

  .commit-head {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .commit-hash {
    font-family: var(--ff-mono);
    font-size: 11px;
    color: var(--text-2);
    background: var(--bg);
    padding: 2px 6px;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .commit-message {
    flex: 1;
    font-size: 13px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .commit-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    color: var(--muted);
  }

  .author {
    flex-shrink: 0;
  }

  .date {
    flex-shrink: 0;
  }

  .commit-stats {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 10px;
    color: var(--muted);
    font-family: var(--ff-mono);
  }

  .files {
    color: var(--text-2);
  }

  .additions {
    color: var(--green);
  }

  .deletions {
    color: var(--danger);
  }
</style>
