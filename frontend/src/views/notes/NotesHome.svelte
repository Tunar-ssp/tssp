<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { Note } from '$lib/api';
  import { formatRelative } from '$lib/utils';

  interface $$Props {
    notes: Note[];
    isLoading: boolean;
    layout: 'grid' | 'list';
    searchQuery: string;
    activeTag: string | null;
    onSelectNote?: (id: string) => void;
    onContextMenu?: (e: MouseEvent, note: Note) => void;
    onCreateNote?: () => void;
  }

  let {
    notes,
    isLoading,
    layout,
    searchQuery,
    activeTag,
    onSelectNote,
    onContextMenu,
    onCreateNote,
  }: $$Props = $props();

  let pinnedNotes = $derived(notes.filter((note) => !!note.pinned_at));
  let recentNotes = $derived(notes.slice(0, 8));

  function noteSummary(text: string): string {
    return text
      .replace(/^#{1,6}\s+/gm, '')
      .replace(/^[-*]\s+\[[ x]\]\s+/gim, '')
      .replace(/^[-*]\s+/gm, '')
      .replace(/^>\s+/gm, '')
      .replace(/`{1,3}/g, '')
      .replace(/\s+/g, ' ')
      .trim();
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

<section class="notes-home">
  <header class="home-header">
    <div>
      <h1>{searchQuery.trim() ? 'Search notes' : 'All notes'}</h1>
      <p>
        {notes.length} total · {notes.filter((note) => note.pinned_at).length} pinned
        {#if activeTag}
          · tag: {activeTag}
        {/if}
      </p>
    </div>
  </header>

  {#if isLoading}
    <div class="state-panel">
      <div class="spinner"></div>
      <strong>Loading notes</strong>
      <p>Syncing local pages from the TSSP backend.</p>
    </div>
  {:else if notes.length === 0}
    <div class="state-panel">
      <Icons.StickyNote size={32} />
      <strong>No notes yet</strong>
      <p>Create the first page, or clear your search/tag filters.</p>
      <button type="button" class="inline-action" onclick={onCreateNote}>Create note</button>
    </div>
  {:else}
    {#if pinnedNotes.length > 0}
      <section class="home-section">
        <div class="section-head">
          <span class="section-label"><Icons.Pin size={14} /> Pinned</span>
        </div>
        <div class:card-grid={layout === 'grid'} class:list-grid={layout === 'list'}>
          {#each pinnedNotes.slice(0, layout === 'grid' ? 4 : 8) as note (String(note.id))}
            <button
              type="button"
              class="note-card"
              onclick={() => onSelectNote?.(note.id)}
              oncontextmenu={(event) => onContextMenu?.(event, note)}
              style="--accent: {noteAccent(note)}"
            >
              <div class="card-accent"></div>
              <div class="card-head">
                <strong>{note.title || 'Untitled note'}</strong>
                <Icons.Pin size={14} />
              </div>
              <p>{noteSummary(note.body).slice(0, 180) || 'Open this note to start writing.'}</p>
              <div class="card-foot">
                <div class="card-tags">
                  {#each note.tags.slice(0, 3) as tag}
                    <span>{tag}</span>
                  {/each}
                </div>
                <small>{formatRelative(note.updated_at)}</small>
              </div>
            </button>
          {/each}
        </div>
      </section>
    {/if}

    <section class="home-section">
      <div class="section-head">
        <span class="section-label">{searchQuery.trim() ? 'Matches' : 'Recent notes'}</span>
      </div>
      <div class:card-grid={layout === 'grid'} class:list-grid={layout === 'list'}>
        {#each recentNotes as note (String(note.id))}
          <button
            type="button"
            class="note-card"
            onclick={() => onSelectNote?.(note.id)}
            oncontextmenu={(event) => onContextMenu?.(event, note)}
            style="--accent: {noteAccent(note)}"
          >
            <div class="card-accent"></div>
            <div class="card-head">
              <strong>{note.title || 'Untitled note'}</strong>
              {#if note.pinned_at}
                <Icons.Pin size={14} />
              {/if}
            </div>
            <p>{noteSummary(note.body).slice(0, 180) || 'Open this note to start writing.'}</p>
            <div class="card-foot">
              <div class="card-tags">
                {#each note.tags.slice(0, 3) as tag}
                  <span>{tag}</span>
                {/each}
              </div>
              <small>{formatRelative(note.updated_at)}</small>
            </div>
          </button>
        {/each}
      </div>
    </section>
  {/if}
</section>

<style>
  .notes-home {
    padding: 28px 34px 40px;
    overflow: auto;
  }

  .home-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
    margin-bottom: 28px;
  }

  .home-header h1 {
    margin: 0;
    font-size: clamp(42px, 4vw, 68px);
    line-height: 0.96;
    letter-spacing: -0.04em;
  }

  .home-header p {
    margin: 12px 0 0;
    color: var(--muted);
    font-size: 18px;
  }

  .state-panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    color: var(--muted);
    gap: 12px;
  }

  .state-panel strong {
    color: var(--text);
    font-size: 18px;
  }

  .state-panel p {
    margin: 0;
    color: var(--muted);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .new-note-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 40px;
    padding: 0 18px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--blue);
    color: white;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: filter 0.18s;
    flex-shrink: 0;
  }
  .new-note-btn:hover { filter: brightness(1.1); }

  .inline-action {
    margin-top: 12px;
    padding: 8px 16px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text);
    cursor: pointer;
    font-size: 14px;
    transition: all 0.2s;
  }

  .inline-action:hover {
    background: var(--surface-3);
  }

  .home-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
    margin-bottom: 28px;
  }

  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .section-label {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-family: var(--ff-mono);
    font-size: 12px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--dim);
  }

  .card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 18px;
  }

  .list-grid {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .note-card {
    position: relative;
    min-height: 220px;
    padding: 26px 26px 20px;
    border-radius: 26px;
    border: 1px solid rgba(39, 43, 57, 0.96);
    background: rgba(18, 21, 29, 0.96);
    color: var(--text);
    text-align: left;
    cursor: pointer;
    overflow: hidden;
    transition: all 0.2s;
  }

  .note-card:hover {
    border-color: var(--border);
    background: rgba(20, 24, 32, 0.96);
  }

  .note-card .card-accent {
    position: absolute;
    left: 0;
    top: 20px;
    bottom: 20px;
    width: 6px;
    border-radius: 999px;
    background: var(--accent);
  }

  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .card-head strong {
    font-size: 22px;
    letter-spacing: -0.02em;
  }

  .note-card p {
    margin: 18px 0 0;
    color: var(--muted);
    font-size: 17px;
    line-height: 1.6;
  }

  .card-foot {
    margin-top: 28px;
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 12px;
  }

  .card-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .card-tags span {
    height: 28px;
    padding: 0 10px;
    border-radius: 10px;
    background: rgba(27, 31, 43, 0.96);
    color: var(--text-2);
    font-family: var(--ff-mono);
    display: inline-flex;
    align-items: center;
    font-size: 12px;
  }

  .card-foot small {
    color: var(--muted);
    font-size: 12px;
  }
</style>
