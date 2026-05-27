<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { formatRelative } from '$lib/utils/formatters';
  import type { Note } from '$lib/api';

  interface Props {
    notes?: Note[];
    onOpenNote?: (note: Note) => void;
    onOpenNotes?: () => void;
  }

  let { notes = [], onOpenNote, onOpenNotes }: Props = $props();
</script>

<article class="panel">
  <div class="panel-head">
    <div>
      <h2>Recent notes</h2>
      <p>Pick up where you left off.</p>
    </div>
    <button type="button" class="link-btn" onclick={onOpenNotes}>Open Notes</button>
  </div>

  {#if notes.length === 0}
    <div class="empty-card compact">
      <Icons.BookText size={20} />
      <strong>No notes yet</strong>
      <p>Create a note to start your knowledge base.</p>
    </div>
  {:else}
    <div class="note-list">
      {#each notes.slice(0, 4) as note (note.id)}
        <button type="button" class="note-card" onclick={() => onOpenNote?.(note)}>
          <div class="note-stripe"></div>
          <div class="note-body">
            <div class="note-title-row">
              <strong>{note.title || 'Untitled note'}</strong>
              {#if note.pinned_at}
                <Icons.Pin size={12} />
              {/if}
            </div>
            <p>{note.body?.replace(/\s+/g, ' ').trim() || 'No content yet.'}</p>
            <div class="note-footer">
              <span>{formatRelative(note.updated_at)}</span>
              {#if note.tags?.length}
                <span>{note.tags.slice(0, 2).join(' · ')}</span>
              {/if}
            </div>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</article>

<style>
  .panel {
    border: 1px solid rgba(255, 255, 255, 0.07);
    background: linear-gradient(180deg, rgba(20, 22, 29, 0.94), rgba(15, 16, 22, 0.92));
    box-shadow: var(--shadow-card);
    border-radius: 22px;
    padding: 18px;
  }

  .panel-head {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    margin-bottom: 16px;
  }

  .panel-head h2 {
    margin: 0;
    font-size: 17px;
    color: var(--text);
  }

  .panel-head p {
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 13px;
    line-height: 1.5;
  }

  .link-btn {
    margin-left: auto;
    border: none;
    background: none;
    color: var(--blue);
    font-size: 12px;
    cursor: pointer;
  }

  .note-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .note-card {
    position: relative;
    border: 1px solid var(--border);
    background: var(--surface);
    color: inherit;
    cursor: pointer;
    border-radius: 14px;
    padding: 12px 12px 12px 16px;
    display: flex;
    text-align: left;
    transition: all 150ms;
  }

  .note-card:hover {
    border-color: var(--border-2);
    background: var(--surface-2);
  }

  .note-stripe {
    position: absolute;
    left: 0;
    top: 10px;
    bottom: 10px;
    width: 3px;
    border-radius: 3px;
    background: var(--green);
  }

  .note-body {
    width: 100%;
    padding-left: 4px;
  }

  .note-title-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .note-title-row strong {
    flex: 1;
    min-width: 0;
    text-align: left;
    font-size: 13px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-body p {
    margin: 8px 0 10px;
    font-size: 12px;
    color: var(--text-2);
    line-height: 1.55;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .note-footer {
    display: flex;
    justify-content: space-between;
    gap: 10px;
    font-size: 11px;
    color: var(--muted);
  }

  .empty-card {
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--surface);
    min-height: 110px;
    justify-content: center;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 14px;
  }

  .empty-card.compact {
    min-height: 110px;
  }

  .empty-card strong {
    display: block;
    color: var(--text);
    font-size: 13px;
  }

  .empty-card p {
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.5;
  }
</style>
