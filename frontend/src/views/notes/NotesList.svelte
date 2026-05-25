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

  function formatDate(timestamp: number) {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
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
      .slice(0, 80);
  }

  function noteAccent(note: Note): string {
    if (note.pinned_at) return 'var(--pink)';
    const seed = (note.tags?.[0] || note.id || '').toLowerCase();
    if (seed.includes('ops') || seed.includes('infra')) return 'var(--green)';
    if (seed.includes('roadmap') || seed.includes('work')) return 'var(--orange)';
    if (seed.includes('personal') || seed.includes('home')) return 'var(--pink)';
    return 'var(--blue)';
  }
</script>

<div class="notes-list {className || ''}">
  {#if notes.length === 0}
    <div class="empty-state">
      <Icons.FileText size={32} />
      <p>No notes yet</p>
    </div>
  {:else}
    {#each notes as note (note.id)}
      <button
        class="note-item"
        class:active={activeNoteId === note.id}
        style="--accent: {noteAccent(note)}"
        onclick={() => onSelectNote?.(note.id)}
        oncontextmenu={(e) => {
          e.preventDefault();
          onContextMenu?.(e, note);
        }}
        title={note.title}
      >
        <div class="note-header">
          <div class="note-title">{note.title || 'Untitled'}</div>
          {#if note.pinned_at}
            <Icons.Pin size={12} />
          {/if}
        </div>
        <div class="note-summary">{noteSummary(note.body)}</div>
        <div class="note-meta">
          <span class="note-date">{formatDate(note.updated_at || note.created_at)}</span>
          {#if note.tags?.length}
            <span class="note-tags">{note.tags.length} tags</span>
          {/if}
        </div>
      </button>
    {/each}
  {/if}
</div>

<style>
  .notes-list {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
    background: var(--surface);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--muted);
    gap: 8px;
  }

  .empty-state p {
    margin: 0;
    font-size: 14px;
  }

  .note-item {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: var(--s-3);
    border: none;
    background: transparent;
    border-bottom: 1px solid var(--hairline);
    cursor: pointer;
    transition: background var(--duration-quick) var(--ease-smooth);
    text-align: left;
    min-height: 0;
  }

  .note-item:hover {
    background: var(--surface-2);
  }

  .note-item.active {
    background: var(--surface-2);
    border-left: 4px solid var(--accent);
  }

  .note-header {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: space-between;
  }

  .note-title {
    flex: 1;
    font-weight: 600;
    color: var(--text);
    font-size: 14px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-summary {
    font-size: 12px;
    color: var(--text-2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    color: var(--muted);
  }

  .note-date {
    flex: 1;
  }

  .note-tags {
    background: var(--surface);
    padding: 0 6px;
    border-radius: 2px;
  }
</style>
