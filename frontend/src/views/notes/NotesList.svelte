<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { Note } from '$lib/api';

  interface $$Props {
    notes: Note[];
    activeNoteId?: string | null;
    onSelectNote?: (id: string) => void;
    onDeleteNote?: (id: string) => void;
    onPinNote?: (note: Note) => void;
    onContextMenu?: (e: MouseEvent, note: Note) => void;
    class?: string;
  }

  let {
    notes = [],
    activeNoteId,
    onSelectNote,
    onDeleteNote,
    onPinNote,
    onContextMenu,
    class: className,
  }: $$Props = $props();

  function formatRelative(timestamp: number): string {
    const now = Math.floor(Date.now() / 1000);
    const diff = now - timestamp;
    const minutes = Math.floor(diff / 60);
    const hours = Math.floor(diff / 3600);
    const days = Math.floor(diff / 86400);

    if (minutes < 1) return 'now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;

    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
    });
  }

  function noteSummary(text: string): string {
    return text
      .replace(/^#{1,6}\s+/gm, '')
      .replace(/^[-*]\s+\[[ x]\]\s+/gim, '')
      .replace(/^[-*]\s+/gm, '')
      .replace(/^>\s+/gm, '')
      .replace(/`{1,3}/g, '')
      .replace(/\s+/g, ' ')
      .trim()
      .slice(0, 120);
  }

  function noteAccent(note: Note): string {
    if (note.pinned_at) return '#ff006e';
    const seed = (note.tags?.[0] || note.id || '').toLowerCase();
    if (seed.includes('ops') || seed.includes('infra')) return '#10b981';
    if (seed.includes('roadmap') || seed.includes('work')) return '#f59e0b';
    if (seed.includes('personal') || seed.includes('home')) return '#ff006e';
    if (seed.includes('log')) return '#6366f1';
    return '#0ea5e9';
  }
</script>

<div class="notes-grid {className || ''}">
  {#if notes.length === 0}
    <div class="empty-state">
      <Icons.FileText size={48} />
      <h3>No notes yet</h3>
      <p>Create your first note to get started</p>
    </div>
  {:else}
    {#each notes as note (String(note.id))}
      <button
        class="note-card"
        class:active={activeNoteId === note.id}
        style="--accent: {noteAccent(note)}"
        onclick={() => onSelectNote?.(note.id)}
        oncontextmenu={(e) => {
          e.preventDefault();
          onContextMenu?.(e, note);
        }}
        title={note.title}
      >
        <div class="card-accent"></div>
        <div class="card-content">
          <div class="card-header">
            <h3 class="card-title">{note.title || 'Untitled'}</h3>
            {#if note.pinned_at}
              <Icons.Pin size={14} strokeWidth={2.5} />
            {/if}
          </div>
          <p class="card-summary">{noteSummary(note.body)}</p>
          <div class="card-footer">
            <span class="card-time">{formatRelative(note.updated_at || note.created_at)}</span>
            {#if note.tags?.length}
              <div class="card-tags">
                {#each note.tags.slice(0, 2) as tag}
                  <span class="tag-badge">{tag}</span>
                {/each}
                {#if note.tags.length > 2}
                  <span class="tag-more">+{note.tags.length - 2}</span>
                {/if}
              </div>
            {/if}
          </div>
        </div>
      </button>
    {/each}
  {/if}
</div>

<style>
  .notes-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 16px;
    padding: 24px;
    height: 100%;
    overflow-y: auto;
    background: var(--bg);
  }

  .empty-state {
    grid-column: 1 / -1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 300px;
    color: var(--muted);
    gap: 12px;
  }

  .empty-state h3 {
    margin: 8px 0 0 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-2);
  }

  .empty-state p {
    margin: 0;
    font-size: 13px;
  }

  .note-card {
    display: flex;
    flex-direction: column;
    height: 220px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    overflow: hidden;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
    text-align: left;
    position: relative;
  }

  .note-card:hover {
    border-color: var(--text-2);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    transform: translateY(-2px);
  }

  .note-card.active {
    border-color: var(--accent);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  }

  .card-accent {
    position: absolute;
    top: 0;
    left: 0;
    width: 4px;
    height: 100%;
    background: var(--accent);
  }

  .card-content {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    padding-left: 16px;
    flex: 1;
    overflow: hidden;
  }

  .card-header {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    justify-content: space-between;
  }

  .card-title {
    margin: 0;
    font-weight: 600;
    color: var(--text);
    font-size: 15px;
    line-height: 1.3;
    word-break: break-word;
    flex: 1;
  }

  .card-summary {
    margin: 0;
    font-size: 13px;
    color: var(--text-2);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    flex: 1;
  }

  .card-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-top: auto;
  }

  .card-time {
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
  }

  .card-tags {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .tag-badge {
    display: inline-block;
    padding: 2px 6px;
    background: var(--surface-2);
    color: var(--text-2);
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    white-space: nowrap;
  }

  .tag-more {
    font-size: 11px;
    color: var(--muted);
  }
</style>
