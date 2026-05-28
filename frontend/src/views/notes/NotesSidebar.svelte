<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    notes: any[];
    collectionFilter: 'all' | 'pinned' | 'recent';
    activeTag: string | null;
    allTags: string[];
    onCollectionChange: (filter: 'all' | 'pinned' | 'recent') => void;
    onTagChange: (tag: string | null) => void;
    onCreateNote: () => void;
    onClose?: () => void;
  }

  let {
    notes,
    collectionFilter,
    activeTag,
    allTags,
    onCollectionChange,
    onTagChange,
    onCreateNote,
    onClose,
  }: $$Props = $props();

  const pinnedCount = $derived(notes.filter((n) => n.pinned_at).length);
  const maxTags = $derived(Math.max(...notes.map((n) => n.tags?.length || 0), 0));
</script>

<aside class="notes-sidebar">
  <div class="sb-head">
    <div class="sb-title">
      <Icons.BookText size={16} />
      <span>Notes</span>
    </div>
    {#if onClose}
      <button type="button" class="sb-close" onclick={onClose} title="Hide sidebar (Ctrl+B)">
        <Icons.PanelLeftClose size={16} />
      </button>
    {/if}
  </div>
  <button type="button" class="new-note-btn" onclick={onCreateNote} title="New note (Ctrl+N)">
    <Icons.Plus size={16} />
    <span>New note</span>
  </button>

  <div class="sidebar-section">
    <div class="sidebar-label">Collections</div>
    <button
      type="button"
      class="sidebar-item"
      class:active={collectionFilter === 'all'}
      onclick={() => onCollectionChange('all')}
    >
      <Icons.BookText size={16} />
      <span>All notes</span>
      <small>{notes.length}</small>
    </button>
    <button
      type="button"
      class="sidebar-item"
      class:active={collectionFilter === 'pinned'}
      onclick={() => onCollectionChange('pinned')}
    >
      <Icons.Pin size={16} />
      <span>Pinned</span>
      <small>{pinnedCount}</small>
    </button>
    <button
      type="button"
      class="sidebar-item"
      class:active={collectionFilter === 'recent'}
      onclick={() => onCollectionChange('recent')}
    >
      <Icons.History size={16} />
      <span>Recent</span>
      <small>{Math.min(notes.length, 14)}</small>
    </button>
  </div>

  {#if allTags.length > 0}
    <div class="sidebar-section">
      <div class="sidebar-label">Tags</div>
      <div class="tag-cloud">
        {#each allTags as tag}
          <button
            type="button"
            class="tag-filter"
            class:active={activeTag === tag}
            onclick={() => onTagChange(activeTag === tag ? null : tag)}
          >
            {tag}
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <div class="sidebar-section stats">
    <div class="sidebar-label">Library</div>
    <div class="stats-grid">
      <div>
        <strong>{notes.length}</strong>
        <span>notes</span>
      </div>
      <div>
        <strong>{pinnedCount}</strong>
        <span>pinned</span>
      </div>
      <div>
        <strong>{allTags.length}</strong>
        <span>tags</span>
      </div>
      <div>
        <strong>{maxTags}</strong>
        <span>max tags</span>
      </div>
    </div>
  </div>
</aside>

<style>
  .notes-sidebar {
    padding: 14px 14px 18px;
    border-right: 1px solid var(--border);
    background: rgba(17, 19, 25, 0.9);
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow-y: auto;
  }

  .sb-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .sb-title {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text);
    font-size: 14px;
    font-weight: 600;
  }
  .sb-close {
    width: 28px;
    height: 28px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-2);
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .sb-close:hover {
    background: rgba(255, 255, 255, 0.06);
    color: var(--text);
    border-color: var(--border);
  }

  .new-note-btn {
    height: 40px;
    padding: 0 14px;
    border-radius: 10px;
    border: 1px solid rgba(110, 168, 255, 0.28);
    background: linear-gradient(180deg, rgba(110, 168, 255, 0.96), rgba(95, 149, 233, 0.96));
    color: #06101f;
    font-size: 13px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    cursor: pointer;
    transition: filter 0.18s;
  }
  .new-note-btn:hover { filter: brightness(1.07); }

  .sidebar-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .sidebar-label {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-family: var(--ff-mono);
    font-size: 12px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--dim);
  }

  .sidebar-item {
    min-height: 36px;
    padding: 0 12px;
    border: 1px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: var(--text-2);
    display: flex;
    align-items: center;
    gap: 10px;
    text-align: left;
    cursor: pointer;
    font-size: 13px;
  }

  .sidebar-item small {
    margin-left: auto;
    font-size: 13px;
    color: var(--muted);
    font-family: var(--ff-mono);
  }

  .sidebar-item:hover,
  .sidebar-item.active {
    color: var(--text);
    background: rgba(34, 38, 50, 0.9);
    border-color: var(--border);
  }

  .tag-cloud {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .tag-filter {
    min-height: 32px;
    padding: 0 12px;
    border-radius: 999px;
    border: 1px solid rgba(110, 168, 255, 0.14);
    background: rgba(18, 22, 31, 0.9);
    color: var(--text-2);
    cursor: pointer;
  }

  .tag-filter.active,
  .tag-filter:hover {
    color: var(--text);
    border-color: rgba(110, 168, 255, 0.28);
    background: rgba(28, 36, 52, 0.96);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }

  .stats-grid div {
    padding: 8px 10px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: rgba(20, 23, 31, 0.92);
  }

  .stats-grid strong {
    display: block;
    font-size: 15px;
    color: var(--text);
  }

  .stats-grid span {
    color: var(--muted);
    font-size: 12px;
  }
</style>
