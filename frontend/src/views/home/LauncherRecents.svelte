<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { FileRecord, Note, Workspace } from '$lib/api';

  interface $$Props {
    recentFiles: FileRecord[];
    recentNotes: Note[];
    recentWorkspaces: Workspace[];
    onFileSelect?: (id: string) => void;
    onNoteSelect?: (id: string) => void;
    onWorkspaceSelect?: (id: string) => void;
  }

  let {
    recentFiles = [],
    recentNotes = [],
    recentWorkspaces = [],
    onFileSelect,
    onNoteSelect,
    onWorkspaceSelect,
  }: $$Props = $props();

  function formatRelative(timestamp: number): string {
    const delta = Math.max(0, Math.floor(Date.now() / 1000) - timestamp);
    if (delta < 60) return 'just now';
    if (delta < 3_600) return `${Math.floor(delta / 60)}m ago`;
    if (delta < 86_400) return `${Math.floor(delta / 3_600)}h ago`;
    if (delta < 604_800) return `${Math.floor(delta / 86_400)}d ago`;
    return `${Math.floor(delta / 604_800)}w ago`;
  }

  function getFileIcon(mimeType: string): string {
    if (mimeType.startsWith('image/')) return '🖼️';
    if (mimeType.startsWith('video/')) return '🎬';
    if (mimeType.includes('pdf')) return '📄';
    if (mimeType.startsWith('text/') || mimeType.includes('json')) return '📝';
    return '📎';
  }
</script>

<section class="launcher-recents">
  {#if recentFiles.length > 0}
    <div class="recent-section">
      <h3>Recent Files</h3>
      <div class="recents-grid">
        {#each recentFiles.slice(0, 4) as file}
          <button
            type="button"
            class="recent-item file-item"
            onclick={() => onFileSelect?.(file.id)}
          >
            <span class="item-icon">{getFileIcon(file.mime_type)}</span>
            <div class="item-content">
              <span class="item-name">{file.name}</span>
              <span class="item-time">{formatRelative(file.uploaded_at || 0)}</span>
            </div>
          </button>
        {/each}
      </div>
    </div>
  {/if}

  {#if recentNotes.length > 0}
    <div class="recent-section">
      <h3>Recent Notes</h3>
      <div class="recents-grid">
        {#each recentNotes.slice(0, 4) as note}
          <button
            type="button"
            class="recent-item note-item"
            onclick={() => onNoteSelect?.(note.id)}
          >
            <span class="item-icon">📋</span>
            <div class="item-content">
              <span class="item-name">{note.title || 'Untitled'}</span>
              <span class="item-time">{formatRelative(note.updated_at)}</span>
            </div>
          </button>
        {/each}
      </div>
    </div>
  {/if}

  {#if recentWorkspaces.length > 0}
    <div class="recent-section">
      <h3>Recent Workspaces</h3>
      <div class="recents-grid">
        {#each recentWorkspaces.slice(0, 4) as workspace}
          <button
            type="button"
            class="recent-item workspace-item"
            onclick={() => onWorkspaceSelect?.(workspace.id)}
          >
            <span class="item-icon">💻</span>
            <div class="item-content">
              <span class="item-name">{workspace.name}</span>
              <span class="item-lang">{workspace.language}</span>
            </div>
          </button>
        {/each}
      </div>
    </div>
  {/if}
</section>

<style>
  .launcher-recents {
    display: flex;
    flex-direction: column;
    gap: 40px;
  }

  .recent-section h3 {
    margin: 0 0 16px;
    font-size: 18px;
    font-weight: 600;
    color: var(--text);
  }

  .recents-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 12px;
  }

  .recent-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: transparent;
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
  }

  .recent-item:hover {
    border-color: rgba(110, 168, 255, 0.4);
    background: rgba(110, 168, 255, 0.06);
  }

  .item-icon {
    font-size: 24px;
    flex-shrink: 0;
  }

  .item-content {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .item-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-time,
  .item-lang {
    font-size: 12px;
    color: var(--muted);
  }
</style>
