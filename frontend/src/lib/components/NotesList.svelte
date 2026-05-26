<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { Note } from '$lib/api';
  import { activeNoteId, searchQuery, createNewNote } from '$lib/stores/notes';

  interface $$Props {
    notes?: Note[];
    onSelectNote?: (id: string) => void;
  }

  let {
    notes = [],
    onSelectNote = () => {},
  }: $$Props = $props();

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const hours = diff / (1000 * 60 * 60);
    const days = diff / (1000 * 60 * 60 * 24);

    if (hours < 1) return 'Just now';
    if (hours < 24) return `${Math.floor(hours)}h ago`;
    if (days < 7) return `${Math.floor(days)}d ago`;
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  }

  async function handleNewNote() {
    await createNewNote();
  }
</script>

<div class="notes-list-container">
  <div class="list-header">
    <h3>Notes</h3>
    <button class="new-btn" onclick={handleNewNote}>
      <Icons.Plus size={16} />
    </button>
  </div>

  <div class="search-box">
    <Icons.Search size={14} />
    <input
      type="text"
      placeholder="Search notes..."
      bind:value={$searchQuery}
    />
  </div>

  {#if notes.length === 0}
    <div class="empty-list">
      <Icons.BookOpen size={32} />
      <p>No notes yet</p>
      <p class="secondary">Create a new note to get started</p>
    </div>
  {:else}
    <div class="notes-list">
      {#each notes as note (note.id)}
        <button
          class="note-item"
          class:active={$activeNoteId === note.id}
          onclick={() => {
            $activeNoteId = note.id;
            onSelectNote(note.id);
          }}
        >
          <div class="item-header">
            <div class="item-title">{note.title || 'Untitled'}</div>
            <span class="item-date">{formatDate(note.updated_at)}</span>
          </div>

          {#if note.tags.length > 0}
            <div class="item-tags">
              {#each note.tags.slice(0, 2) as tag}
                <span class="tag">{tag}</span>
              {/each}
              {#if note.tags.length > 2}
                <span class="tag-more">+{note.tags.length - 2}</span>
              {/if}
            </div>
          {/if}

          <div class="item-preview">{note.body.slice(0, 60)}</div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .notes-list-container {
    width: 280px;
    height: 100%;
    border-right: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .list-header {
    padding: 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--border);
  }

  .list-header h3 {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
  }

  .new-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border-radius: var(--r-2);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .new-btn:hover {
    background: var(--blue);
    border-color: var(--blue);
    color: #0a1228;
  }

  .search-box {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--muted);
  }

  .search-box input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text-2);
    font-size: var(--fs-12);
    outline: none;
  }

  .search-box input::placeholder {
    color: var(--muted);
  }

  .empty-list {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--muted);
    padding: 20px;
    text-align: center;
  }

  .empty-list p {
    margin: 0;
  }

  .empty-list .secondary {
    font-size: var(--fs-12);
    color: var(--dim);
  }

  .notes-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .note-item {
    padding: 12px;
    border: none;
    border-bottom: 1px solid var(--hairline);
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: all 0.15s;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .note-item:hover {
    background: var(--surface-2);
  }

  .note-item.active {
    background: var(--surface-hi);
    border-left: 2px solid var(--blue);
    padding-left: 10px;
  }

  .item-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
  }

  .item-title {
    font-size: var(--fs-12);
    font-weight: 600;
    color: var(--text);
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-date {
    font-size: 10px;
    color: var(--muted);
    flex-shrink: 0;
  }

  .item-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .tag {
    display: inline-block;
    padding: 1px 6px;
    border-radius: 3px;
    background: var(--blue-soft);
    color: var(--blue);
    font-size: 10px;
    font-weight: 500;
  }

  .tag-more {
    display: inline-block;
    padding: 1px 6px;
    font-size: 10px;
    color: var(--muted);
  }

  .item-preview {
    font-size: 11px;
    color: var(--muted);
    line-height: 1.3;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
