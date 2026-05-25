<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import {
    sortedNotes,
    activeNote,
    loadNotes,
    setActiveNote,
    updateActiveNote,
    createNewNote,
    deleteNote,
    duplicateNote,
    toggleNotePin,
    replaceActiveNoteTags,
    isSaving,
  } from '$lib/stores/notes';
  import { success, error } from '$lib/stores/notifications';
  import TiptapEditor from '$lib/components/TiptapEditor.svelte';
  import Outline from '$lib/components/Outline.svelte';
  import SlashMenu from '$lib/components/SlashMenu.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import { consumeSelectionIntent } from '$lib/stores/ui';
  import { estimateBlockCount, renderMarkdownLite } from '$lib/utils/markdown';

  type CollectionFilter = 'all' | 'pinned' | 'recent';
  type InspectorTab = 'preview' | 'outline' | 'meta';
  type HomeLayout = 'grid' | 'list';

  let contextMenu = $state({ visible: false, x: 0, y: 0, note: null as any });
  let searchQuery = $state('');
  let isLoading = $state(true);
  let titleDraft = $state('');
  let bodyDraft = $state('');
  let tagDraft = $state('');
  let collectionFilter = $state<CollectionFilter>('all');
  let inspectorTab = $state<InspectorTab>('preview');
  let homeLayout = $state<HomeLayout>('grid');
  let activeTag = $state<string | null>(null);
  let showSlashMenu = $state(false);
  let slashMenuPos = $state({ x: 0, y: 0 });
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  onMount(async () => {
    await loadNotes();
    const intent = consumeSelectionIntent();
    if (intent?.kind === 'note') {
      setActiveNote(intent.id);
    }

    if (typeof window !== 'undefined') {
      window.addEventListener('insert', handleInsertSnippet as EventListener);
    }

    isLoading = false;
  });

  onDestroy(() => {
    if (saveTimer) clearTimeout(saveTimer);
    if (typeof window !== 'undefined') {
      window.removeEventListener('insert', handleInsertSnippet as EventListener);
    }
  });

  $effect(() => {
    if ($activeNote) {
      titleDraft = $activeNote.title;
      bodyDraft = $activeNote.body;
      tagDraft = '';
    } else {
      titleDraft = '';
      bodyDraft = '';
      tagDraft = '';
    }
  });

  let allTags = $derived(
    Array.from(new Set($sortedNotes.flatMap((note) => note.tags || []))).sort((left, right) => left.localeCompare(right))
  );

  let filteredNotes = $derived.by(() =>
    $sortedNotes.filter((note) => {
      if (collectionFilter === 'pinned' && !note.pinned_at) return false;
      if (collectionFilter === 'recent') {
        const ageSeconds = Math.floor(Date.now() / 1000) - note.updated_at;
        if (ageSeconds > 14 * 86_400) return false;
      }
      if (activeTag && !note.tags.includes(activeTag)) return false;

      if (!searchQuery.trim()) return true;
      const query = searchQuery.toLowerCase();
      return (
        note.title.toLowerCase().includes(query) ||
        note.body.toLowerCase().includes(query) ||
        note.tags.some((tag) => tag.toLowerCase().includes(query))
      );
    })
  );

  let pinnedNotes = $derived(filteredNotes.filter((note) => !!note.pinned_at));
  let recentNotes = $derived(filteredNotes.slice(0, 8));
  let previewHtml = $derived(renderMarkdownLite(bodyDraft));
  let blockCount = $derived(estimateBlockCount(bodyDraft));
  let totalWords = $derived(getWordCount(bodyDraft));

  function handleInsertSnippet(event: Event) {
    const snippet = (event as CustomEvent<{ text?: string }>).detail?.text;
    if (!$activeNote || !snippet) return;
    bodyDraft = bodyDraft.trim().length ? `${bodyDraft.trimEnd()}\n\n${snippet}` : snippet;
    showSlashMenu = false;
    scheduleSave();
    inspectorTab = 'preview';
  }

  async function handleCreateNote() {
    try {
      const note = await createNewNote();
      setActiveNote(note.id);
      success('Note Created', 'A new note is ready to edit');
    } catch (err) {
      error('Create Failed', err instanceof Error ? err.message : 'Could not create note');
    }
  }

  function handleSelectNote(id: string) {
    setActiveNote(id);
  }

  function clearSelection() {
    setActiveNote(null);
  }

  function scheduleSave() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      void handleSaveNote(false);
    }, 5000);
  }

  async function handleFieldBlur() {
    if (saveTimer) {
      clearTimeout(saveTimer);
      await handleSaveNote(false);
    }
  }

  async function handleSaveNote(showToast = true) {
    if (!$activeNote) return;
    try {
      await updateActiveNote({
        title: titleDraft,
        body: bodyDraft,
      });
      if (showToast) success('Note Saved', 'Changes were written to TSSP');
    } catch (err) {
      error('Save Failed', err instanceof Error ? err.message : 'Could not save note');
    }
  }

  async function handleDeleteNote(id: string) {
    if (!confirm('Delete this note?')) return;
    try {
      await deleteNote(id);
      success('Note Deleted', 'The note was removed');
    } catch (err) {
      error('Delete Failed', err instanceof Error ? err.message : 'Could not delete note');
    }
  }

  async function handleDuplicateNote(id: string) {
    try {
      await duplicateNote(id);
      success('Note Duplicated', 'A copy was created and opened');
    } catch (err) {
      error('Duplicate Failed', err instanceof Error ? err.message : 'Could not duplicate note');
    }
  }

  async function handlePinNote(note: any) {
    try {
      await toggleNotePin(note.id, !!note.pinned_at);
      success(note.pinned_at ? 'Note Unpinned' : 'Note Pinned', 'Pinned notes stay at the top');
    } catch (err) {
      error('Pin Failed', err instanceof Error ? err.message : 'Could not update pin state');
    }
  }

  async function addTag() {
    if (!$activeNote || !tagDraft.trim()) return;
    const nextTag = tagDraft.trim();
    const tags = Array.from(new Set([...($activeNote.tags || []), nextTag]));
    try {
      await replaceActiveNoteTags(tags);
      tagDraft = '';
      activeTag = nextTag;
      success('Tag Added', `"${nextTag}" was added`);
    } catch (err) {
      error('Tag Failed', err instanceof Error ? err.message : 'Could not update tags');
    }
  }

  async function removeTag(tag: string) {
    if (!$activeNote) return;
    const tags = ($activeNote.tags || []).filter((item: string) => item !== tag);
    try {
      await replaceActiveNoteTags(tags);
      success('Tag Removed', `"${tag}" was removed`);
    } catch (err) {
      error('Tag Failed', err instanceof Error ? err.message : 'Could not update tags');
    }
  }

  function showContextMenu(event: MouseEvent, note: any) {
    event.preventDefault();
    event.stopPropagation();
    contextMenu = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      note,
    };
  }

  function openSlashMenu(event: MouseEvent) {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    slashMenuPos = { x: rect.left, y: rect.bottom + 10 };
    showSlashMenu = true;
  }

  function getContextItems(note: any) {
    return [
      { label: note.pinned_at ? 'Unpin' : 'Pin', action: () => handlePinNote(note) },
      { label: 'Duplicate', action: () => handleDuplicateNote(note.id) },
      { label: 'Delete', action: () => handleDeleteNote(note.id), danger: true },
    ];
  }

  function formatDate(timestamp: number) {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  function formatRelative(timestamp: number) {
    const delta = Math.max(0, Math.floor(Date.now() / 1000) - timestamp);
    if (delta < 60) return 'just now';
    if (delta < 3_600) return `${Math.floor(delta / 60)}m`;
    if (delta < 86_400) return `${Math.floor(delta / 3_600)}h`;
    if (delta < 604_800) return `${Math.floor(delta / 86_400)}d`;
    return `${Math.floor(delta / 604_800)}w`;
  }

  function getWordCount(text: string) {
    return text.trim().split(/\s+/).filter((word) => word.length > 0).length;
  }

  function noteSummary(text: string) {
    return text
      .replace(/^#{1,6}\s+/gm, '')
      .replace(/^[-*]\s+\[[ x]\]\s+/gim, '')
      .replace(/^[-*]\s+/gm, '')
      .replace(/^>\s+/gm, '')
      .replace(/`{1,3}/g, '')
      .replace(/\s+/g, ' ')
      .trim();
  }

  function noteAccent(note: any) {
    if (note.pinned_at) return 'var(--pink)';
    const seed = (note.tags[0] || note.id || '').toLowerCase();
    if (seed.includes('ops') || seed.includes('infra') || seed.includes('server')) return 'var(--green)';
    if (seed.includes('roadmap') || seed.includes('work')) return 'var(--orange)';
    if (seed.includes('personal') || seed.includes('home')) return 'var(--pink)';
    return 'var(--blue)';
  }
</script>

<div class:editor-mode={!!$activeNote} class:home-mode={!$activeNote} class="notes-view">
  <aside class="notes-sidebar">
    <button type="button" class="new-note-btn" onclick={handleCreateNote}>
      <Icons.Plus size={20} />
      <span>New note</span>
    </button>

    <div class="sidebar-section">
      <div class="sidebar-label">Collections</div>
      <button type="button" class="sidebar-item" class:active={collectionFilter === 'all'} onclick={() => (collectionFilter = 'all')}>
        <Icons.BookText size={16} />
        <span>All notes</span>
        <small>{$sortedNotes.length}</small>
      </button>
      <button type="button" class="sidebar-item" class:active={collectionFilter === 'pinned'} onclick={() => (collectionFilter = 'pinned')}>
        <Icons.Pin size={16} />
        <span>Pinned</span>
        <small>{$sortedNotes.filter((note) => note.pinned_at).length}</small>
      </button>
      <button type="button" class="sidebar-item" class:active={collectionFilter === 'recent'} onclick={() => (collectionFilter = 'recent')}>
        <Icons.History size={16} />
        <span>Recent</span>
        <small>{Math.min($sortedNotes.length, 14)}</small>
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
              onclick={() => (activeTag = activeTag === tag ? null : tag)}
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
          <strong>{$sortedNotes.length}</strong>
          <span>notes</span>
        </div>
        <div>
          <strong>{$sortedNotes.filter((note) => note.pinned_at).length}</strong>
          <span>pinned</span>
        </div>
        <div>
          <strong>{allTags.length}</strong>
          <span>tags</span>
        </div>
        <div>
          <strong>{Math.max(...$sortedNotes.map((note) => note.tags.length), 0)}</strong>
          <span>max tags</span>
        </div>
      </div>
    </div>
  </aside>

  {#if !$activeNote}
    <section class="notes-home">
      <header class="home-header">
        <div>
          <h1>{searchQuery.trim() ? 'Search notes' : 'All notes'}</h1>
          <p>
            {$sortedNotes.length} total · {$sortedNotes.filter((note) => note.pinned_at).length} pinned
            {#if activeTag}
              · tag: {activeTag}
            {/if}
          </p>
        </div>

        <div class="home-actions">
          <label class="search-field">
            <Icons.Search size={16} />
            <input bind:value={searchQuery} placeholder="Search notes..." />
          </label>

          <div class="layout-toggle">
            <button type="button" class:active={homeLayout === 'grid'} onclick={() => (homeLayout = 'grid')}>
              <Icons.LayoutGrid size={15} />
            </button>
            <button type="button" class:active={homeLayout === 'list'} onclick={() => (homeLayout = 'list')}>
              <Icons.List size={15} />
            </button>
          </div>
        </div>
      </header>

      {#if isLoading}
        <div class="state-panel">
          <div class="spinner"></div>
          <strong>Loading notes</strong>
          <p>Syncing local pages from the TSSP backend.</p>
        </div>
      {:else if filteredNotes.length === 0}
        <div class="state-panel">
          <Icons.StickyNote size={32} />
          <strong>No notes yet</strong>
          <p>Create the first page, or clear your search/tag filters.</p>
          <button type="button" class="inline-action" onclick={handleCreateNote}>Create note</button>
        </div>
      {:else}
        {#if pinnedNotes.length > 0}
          <section class="home-section">
            <div class="section-head">
              <span class="section-label"><Icons.Pin size={14} /> Pinned</span>
            </div>
            <div class:card-grid={homeLayout === 'grid'} class:list-grid={homeLayout === 'list'}>
              {#each pinnedNotes.slice(0, homeLayout === 'grid' ? 4 : 8) as note (note.id)}
                <button
                  type="button"
                  class="note-card"
                  onclick={() => handleSelectNote(note.id)}
                  oncontextmenu={(event) => showContextMenu(event, note)}
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
            <button type="button" class="ghost-link" onclick={() => (searchQuery = '')}>Clear</button>
          </div>
          <div class:card-grid={homeLayout === 'grid'} class:list-grid={homeLayout === 'list'}>
            {#each recentNotes as note (note.id)}
              <button
                type="button"
                class="note-card"
                onclick={() => handleSelectNote(note.id)}
                oncontextmenu={(event) => showContextMenu(event, note)}
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
  {:else}
    <section class="notes-rail">
      <div class="rail-search">
        <label class="search-field">
          <Icons.Search size={16} />
          <input bind:value={searchQuery} placeholder="Search notes..." />
        </label>
        {#if activeTag}
          <button type="button" class="active-tag-chip" onclick={() => (activeTag = null)}>
            {activeTag}
            <Icons.X size={12} />
          </button>
        {/if}
      </div>

      <div class="rail-list">
        {#if filteredNotes.length === 0}
          <div class="state-panel compact">
            <Icons.SearchX size={20} />
            <strong>No matching notes</strong>
            <p>Try a different search or clear the active tag filter.</p>
          </div>
        {:else}
          {#each filteredNotes as note (note.id)}
            <button
              type="button"
              class="rail-note"
              class:active={$activeNote?.id === note.id}
              onclick={() => handleSelectNote(note.id)}
              oncontextmenu={(event) => showContextMenu(event, note)}
            >
              <div class="rail-note-accent" style="background: {noteAccent(note)}"></div>
              <div class="rail-note-body">
                <div class="rail-note-head">
                  <strong>{note.title || 'Untitled note'}</strong>
                  {#if note.pinned_at}
                    <Icons.Pin size={12} />
                  {/if}
                </div>
                <p>{noteSummary(note.body).slice(0, 96) || 'No content yet'}</p>
                <div class="rail-note-meta">
                  <span>{formatRelative(note.updated_at)}</span>
                  <span>{getWordCount(note.body)} words</span>
                </div>
              </div>
            </button>
          {/each}
        {/if}
      </div>
    </section>

    <section class="note-stage">
      <header class="stage-header">
        <div class="stage-path">
          <button type="button" class="ghost-link" onclick={clearSelection}>Notes</button>
          <span>/</span>
          {#each $activeNote.tags.slice(0, 2) as tag}
            <span class="path-chip">{tag}</span>
          {/each}
          <strong>{$activeNote.title || 'Untitled note'}</strong>
        </div>

        <div class="stage-actions">
          {#if $isSaving}
            <span class="save-state saving"><span class="status-dot"></span>Saving</span>
          {:else}
            <span class="save-state"><span class="status-dot"></span>Saved</span>
          {/if}
          <button type="button" class="action-btn" onclick={() => handlePinNote($activeNote)}>
            <Icons.Pin size={14} />
            {$activeNote.pinned_at ? 'Pinned' : 'Pin'}
          </button>
          <button type="button" class="action-btn" onclick={(event) => openSlashMenu(event)}>
            <Icons.Sparkles size={14} />
            Slash blocks
          </button>
          <button type="button" class="action-btn" onclick={() => handleDuplicateNote($activeNote.id)}>
            <Icons.Copy size={14} />
            Duplicate
          </button>
          <button type="button" class="icon-btn danger" title="Delete note" onclick={() => handleDeleteNote($activeNote.id)}>
            <Icons.Trash2 size={14} />
          </button>
        </div>
      </header>

      <div class="stage-body">
        <article class="note-canvas">
          <div class="canvas-head">
            <input
              type="text"
              bind:value={titleDraft}
              oninput={scheduleSave}
              onchange={() => handleSaveNote()}
              onblur={handleFieldBlur}
              class="editor-title"
              placeholder="Note title..."
            />
            <div class="canvas-meta">
              <span><Icons.History size={14} /> Edited {formatDate($activeNote.updated_at || $activeNote.created_at)}</span>
              <span>{totalWords} words</span>
              <span>{blockCount} blocks</span>
            </div>
          </div>

          <div class="tag-strip">
            <div class="tag-list" aria-label="Note tags">
              {#if $activeNote.tags?.length}
                {#each $activeNote.tags as tag}
                  <button type="button" class="tag-chip" onclick={() => removeTag(tag)} title="Remove tag">
                    {tag}
                    <Icons.X size={12} />
                  </button>
                {/each}
              {:else}
                <span class="tag-empty">No tags yet</span>
              {/if}
            </div>

            <form
              class="tag-form"
              onsubmit={(event) => {
                event.preventDefault();
                void addTag();
              }}
            >
              <input bind:value={tagDraft} placeholder="Add tag" aria-label="Add note tag" />
              <button type="submit">Add</button>
            </form>
          </div>

          <div class="editor-shell">
            <TiptapEditor
              content={bodyDraft}
              onChange={(nextContent) => {
                bodyDraft = nextContent;
                scheduleSave();
              }}
            />
          </div>
        </article>

        <aside class="note-inspector">
          <div class="inspector-tabs">
            <button type="button" class:active={inspectorTab === 'preview'} onclick={() => (inspectorTab = 'preview')}>Preview</button>
            <button type="button" class:active={inspectorTab === 'outline'} onclick={() => (inspectorTab = 'outline')}>Outline</button>
            <button type="button" class:active={inspectorTab === 'meta'} onclick={() => (inspectorTab = 'meta')}>Meta</button>
          </div>

          {#if inspectorTab === 'preview'}
            <div class="preview-pane">
              {#if bodyDraft.trim()}
                <div class="markdown-preview">
                  {@html previewHtml}
                </div>
              {:else}
                <div class="inspector-empty">
                  <Icons.PanelRightOpen size={18} />
                  <p>Preview updates as you write markdown-style content.</p>
                </div>
              {/if}
            </div>
          {:else if inspectorTab === 'outline'}
            <div class="outline-pane">
              <Outline content={bodyDraft} onSelectItem={() => {}} />
            </div>
          {:else}
            <div class="meta-pane">
              <div class="meta-card">
                <span class="meta-label">Created</span>
                <strong>{formatDate($activeNote.created_at)}</strong>
              </div>
              <div class="meta-card">
                <span class="meta-label">Updated</span>
                <strong>{formatDate($activeNote.updated_at)}</strong>
              </div>
              <div class="meta-card">
                <span class="meta-label">Words</span>
                <strong>{totalWords}</strong>
              </div>
              <div class="meta-card">
                <span class="meta-label">Blocks</span>
                <strong>{blockCount}</strong>
              </div>

              <div class="meta-note">
                <Icons.Info size={16} />
                <p>Note sharing and linked references are not available from the backend yet, so this editor only exposes real storage-backed actions.</p>
              </div>
            </div>
          {/if}
        </aside>
      </div>
    </section>
  {/if}

  <SlashMenu
    isOpen={showSlashMenu}
    x={slashMenuPos.x}
    y={slashMenuPos.y}
    onClose={() => (showSlashMenu = false)}
  />
</div>

<ContextMenu
  bind:visible={contextMenu.visible}
  x={contextMenu.x}
  y={contextMenu.y}
  items={contextMenu.note ? getContextItems(contextMenu.note) : []}
/>

<style>
  .notes-view {
    flex: 1;
    min-height: 0;
    display: grid;
    background: linear-gradient(180deg, rgba(12, 14, 20, 0.98), rgba(9, 10, 14, 1));
  }

  .notes-view.home-mode {
    grid-template-columns: 300px minmax(0, 1fr);
  }

  .notes-view.editor-mode {
    grid-template-columns: 300px 360px minmax(0, 1fr);
  }

  .notes-sidebar,
  .notes-rail,
  .note-stage,
  .notes-home {
    min-height: 0;
  }

  .notes-sidebar {
    padding: 22px 20px;
    border-right: 1px solid var(--border);
    background: rgba(17, 19, 25, 0.9);
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .new-note-btn {
    min-height: 84px;
    border-radius: 24px;
    border: 1px solid rgba(110, 168, 255, 0.28);
    background: linear-gradient(180deg, rgba(110, 168, 255, 0.96), rgba(95, 149, 233, 0.96));
    color: #06101f;
    font-size: 18px;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 14px;
    cursor: pointer;
    box-shadow: 0 18px 40px rgba(22, 41, 73, 0.35);
  }

  .sidebar-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .sidebar-label,
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

  .sidebar-item {
    min-height: 52px;
    padding: 0 16px;
    border: 1px solid transparent;
    border-radius: 18px;
    background: transparent;
    color: var(--text-2);
    display: flex;
    align-items: center;
    gap: 12px;
    text-align: left;
    cursor: pointer;
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
    padding: 14px;
    border-radius: 18px;
    border: 1px solid var(--border);
    background: rgba(20, 23, 31, 0.92);
  }

  .stats-grid strong {
    display: block;
    font-size: 20px;
    color: var(--text);
  }

  .stats-grid span {
    color: var(--muted);
    font-size: 12px;
  }

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

  .home-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .search-field {
    min-width: 320px;
    height: 52px;
    padding: 0 16px;
    border-radius: 18px;
    border: 1px solid var(--border);
    background: rgba(18, 20, 27, 0.9);
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--muted);
  }

  .search-field input {
    width: 100%;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font-size: 15px;
  }

  .layout-toggle {
    display: inline-flex;
    padding: 4px;
    border-radius: 18px;
    background: rgba(20, 23, 31, 0.96);
    border: 1px solid var(--border);
  }

  .layout-toggle button,
  .inspector-tabs button,
  .action-btn,
  .icon-btn {
    border: none;
    cursor: pointer;
  }

  .layout-toggle button {
    width: 42px;
    height: 42px;
    border-radius: 14px;
    background: transparent;
    color: var(--text-2);
  }

  .layout-toggle button.active {
    background: rgba(52, 60, 80, 0.9);
    color: var(--text);
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

  .ghost-link {
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    font-size: 14px;
  }

  .ghost-link:hover {
    color: var(--text);
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

  .card-head,
  .rail-note-head {
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
  }

  .card-foot small {
    color: var(--muted);
    font-family: var(--ff-mono);
  }

  .notes-rail {
    border-right: 1px solid var(--border);
    background: rgba(13, 15, 21, 0.96);
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .rail-search {
    padding: 18px;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .active-tag-chip {
    align-self: flex-start;
    height: 30px;
    padding: 0 10px;
    border-radius: 999px;
    border: 1px solid rgba(110, 168, 255, 0.24);
    background: rgba(20, 28, 42, 0.9);
    color: var(--blue);
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }

  .rail-list {
    overflow: auto;
    padding: 8px 10px 18px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .rail-note {
    padding: 16px;
    border-radius: 20px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text);
    display: grid;
    grid-template-columns: 4px minmax(0, 1fr);
    gap: 14px;
    text-align: left;
    cursor: pointer;
  }

  .rail-note:hover,
  .rail-note.active {
    background: rgba(28, 31, 40, 0.95);
    border-color: var(--border);
  }

  .rail-note-accent {
    border-radius: 999px;
  }

  .rail-note-body p {
    margin: 8px 0 0;
    color: var(--muted);
    line-height: 1.55;
  }

  .rail-note-meta {
    margin-top: 12px;
    display: flex;
    justify-content: space-between;
    gap: 10px;
    color: var(--dim);
    font-family: var(--ff-mono);
    font-size: 12px;
  }

  .note-stage {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .stage-header {
    min-height: 76px;
    padding: 16px 24px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    background: rgba(12, 14, 20, 0.98);
  }

  .stage-path {
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--muted);
    flex-wrap: wrap;
  }

  .stage-path strong {
    color: var(--text);
    font-size: 16px;
  }

  .path-chip {
    height: 34px;
    padding: 0 14px;
    border-radius: 999px;
    background: rgba(41, 30, 18, 0.96);
    color: var(--orange);
    border: 1px solid rgba(255, 138, 61, 0.22);
    display: inline-flex;
    align-items: center;
  }

  .stage-actions {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .save-state {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--green);
    font-size: 14px;
  }

  .save-state.saving {
    color: var(--warning);
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 999px;
    background: currentColor;
    box-shadow: 0 0 18px currentColor;
  }

  .action-btn,
  .icon-btn {
    height: 40px;
    padding: 0 14px;
    border-radius: 14px;
    background: rgba(18, 22, 31, 0.96);
    border: 1px solid var(--border);
    color: var(--text-2);
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .action-btn:hover,
  .icon-btn:hover {
    color: var(--text);
    background: rgba(25, 29, 40, 0.96);
  }

  .icon-btn {
    width: 40px;
    justify-content: center;
    padding: 0;
  }

  .icon-btn.danger {
    color: var(--danger);
  }

  .stage-body {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 320px;
  }

  .note-canvas {
    min-width: 0;
    min-height: 0;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .canvas-head {
    padding: 20px 24px;
    border-radius: 24px;
    border: 1px solid var(--border);
    background: rgba(15, 17, 23, 0.98);
  }

  .editor-title {
    width: 100%;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font-size: clamp(34px, 4vw, 72px);
    line-height: 0.94;
    letter-spacing: -0.05em;
    font-weight: 700;
    margin: 0;
  }

  .editor-title::placeholder {
    color: var(--dim);
  }

  .canvas-meta {
    margin-top: 14px;
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    color: var(--muted);
    font-size: 14px;
  }

  .canvas-meta span {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .tag-strip {
    padding: 16px 18px;
    border-radius: 20px;
    border: 1px solid var(--border);
    background: rgba(15, 17, 23, 0.92);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }

  .tag-chip {
    height: 34px;
    padding: 0 12px;
    border-radius: 999px;
    border: 1px solid rgba(110, 168, 255, 0.16);
    background: rgba(23, 29, 43, 0.9);
    color: var(--blue);
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .tag-empty {
    color: var(--muted);
    font-size: 14px;
  }

  .tag-form {
    display: flex;
    gap: 10px;
  }

  .tag-form input {
    flex: 1;
    height: 40px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: rgba(12, 14, 20, 0.98);
    color: var(--text);
    padding: 0 12px;
  }

  .tag-form button,
  .inline-action {
    height: 40px;
    padding: 0 14px;
    border-radius: 14px;
    border: 1px solid rgba(110, 168, 255, 0.22);
    background: rgba(110, 168, 255, 0.1);
    color: var(--blue);
    cursor: pointer;
  }

  .editor-shell {
    flex: 1;
    min-height: 0;
  }

  .note-inspector {
    min-height: 0;
    border-left: 1px solid var(--border);
    background: rgba(14, 16, 22, 0.98);
    display: flex;
    flex-direction: column;
  }

  .inspector-tabs {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    padding: 12px;
    border-bottom: 1px solid var(--border);
    gap: 8px;
  }

  .inspector-tabs button {
    height: 40px;
    border-radius: 14px;
    background: transparent;
    color: var(--text-2);
  }

  .inspector-tabs button.active {
    background: rgba(34, 42, 58, 0.95);
    color: var(--text);
  }

  .preview-pane,
  .outline-pane,
  .meta-pane {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .preview-pane {
    padding: 18px;
  }

  .outline-pane :global(.outline) {
    height: 100%;
    border-right: 0;
    background: transparent;
  }

  .meta-pane {
    padding: 18px;
    display: grid;
    align-content: start;
    gap: 12px;
  }

  .meta-card,
  .meta-note {
    padding: 16px;
    border-radius: 18px;
    border: 1px solid var(--border);
    background: rgba(18, 21, 29, 0.96);
  }

  .meta-label {
    display: block;
    color: var(--dim);
    font-family: var(--ff-mono);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.16em;
    margin-bottom: 8px;
  }

  .meta-card strong {
    font-size: 20px;
    color: var(--text);
  }

  .meta-note {
    display: flex;
    gap: 10px;
    color: var(--muted);
    line-height: 1.6;
  }

  .markdown-preview {
    color: var(--text);
    line-height: 1.7;
  }

  .markdown-preview :global(h1),
  .markdown-preview :global(h2),
  .markdown-preview :global(h3),
  .markdown-preview :global(h4),
  .markdown-preview :global(h5),
  .markdown-preview :global(h6) {
    margin: 0 0 12px;
    line-height: 1.05;
    letter-spacing: -0.03em;
  }

  .markdown-preview :global(h1) {
    font-size: 42px;
  }

  .markdown-preview :global(h2) {
    font-size: 30px;
  }

  .markdown-preview :global(p),
  .markdown-preview :global(ul),
  .markdown-preview :global(ol),
  .markdown-preview :global(blockquote),
  .markdown-preview :global(pre) {
    margin: 0 0 16px;
  }

  .markdown-preview :global(blockquote) {
    margin-left: 0;
    padding: 12px 14px;
    border-left: 3px solid var(--green);
    background: rgba(17, 45, 32, 0.48);
    border-radius: 12px;
  }

  .markdown-preview :global(pre) {
    overflow: auto;
    padding: 14px;
    border-radius: 16px;
    background: rgba(12, 14, 20, 0.98);
    border: 1px solid var(--border);
  }

  .markdown-preview :global(code) {
    font-family: var(--ff-mono);
  }

  .markdown-preview :global(.task-list) {
    list-style: none;
    padding-left: 0;
  }

  .markdown-preview :global(.task-item) {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .markdown-preview :global(a) {
    color: var(--blue);
  }

  .state-panel,
  .inspector-empty {
    min-height: 220px;
    border-radius: 26px;
    border: 1px dashed var(--border);
    background: rgba(14, 16, 22, 0.82);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
    text-align: center;
    padding: 20px;
  }

  .state-panel.compact {
    min-height: 160px;
  }

  .state-panel strong {
    color: var(--text);
  }

  .spinner {
    width: 22px;
    height: 22px;
    border-radius: 999px;
    border: 2px solid rgba(110, 168, 255, 0.22);
    border-top-color: var(--blue);
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 1400px) {
    .notes-view.editor-mode {
      grid-template-columns: 280px 320px minmax(0, 1fr);
    }

    .stage-body {
      grid-template-columns: minmax(0, 1fr);
    }

    .note-inspector {
      border-left: 0;
      border-top: 1px solid var(--border);
      min-height: 320px;
    }
  }

  @media (max-width: 960px) {
    .notes-view.home-mode,
    .notes-view.editor-mode {
      grid-template-columns: 1fr;
    }

    .notes-sidebar {
      border-right: 0;
      border-bottom: 1px solid var(--border);
    }

    .notes-rail {
      border-right: 0;
      border-bottom: 1px solid var(--border);
      max-height: 360px;
    }

    .home-header,
    .stage-header {
      flex-direction: column;
      align-items: stretch;
    }

    .home-actions {
      width: 100%;
      flex-wrap: wrap;
    }

    .search-field {
      min-width: 0;
      width: 100%;
    }
  }
</style>
