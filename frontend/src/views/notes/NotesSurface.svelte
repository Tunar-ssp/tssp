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
  import SlashMenu from '$lib/components/SlashMenu.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import { consumeSelectionIntent } from '$lib/stores/ui';
  import { estimateBlockCount, renderMarkdownLite } from '$lib/utils/markdown';
  import { registerKeyboardShortcuts, getWordCount } from '$lib/utils';
  import NotesEditor from './NotesEditor.svelte';
  import NotesList from './NotesList.svelte';
  import NotesSearch from './NotesSearch.svelte';
  import NotesHome from './NotesHome.svelte';
  import NotesSidebar from './NotesSidebar.svelte';
  import NoteEditorHeader from './NoteEditorHeader.svelte';
  import NoteTagsPanel from './NoteTagsPanel.svelte';
  import NoteInspector from './NoteInspector.svelte';

  type CollectionFilter = 'all' | 'pinned' | 'recent';
  type InspectorTab = 'preview' | 'outline' | 'meta';
  type HomeLayout = 'grid' | 'list';

  let showSidebar = $state(typeof localStorage !== 'undefined' ? JSON.parse(localStorage.getItem('notes-sidebar-open') ?? 'true') : true);
  let showInspector = $state(typeof localStorage !== 'undefined' ? JSON.parse(localStorage.getItem('notes-inspector-open') ?? 'false') : false);
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
  let isCreating = $state(false);

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

  const handleNotesKeydown = (e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'b') {
      showSidebar = !showSidebar;
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'n' && !e.shiftKey) {
      e.preventDefault();
      void handleCreateNote();
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'i') {
      e.preventDefault();
      showInspector = !showInspector;
    }
  };

  $effect(() => {
    if (typeof window === 'undefined') return;
    const cleanup = registerKeyboardShortcuts(
      [
        { key: 'b', ctrl: true, handler: handleNotesKeydown },
        { key: 'n', ctrl: true, handler: handleNotesKeydown },
        { key: 'i', ctrl: true, handler: handleNotesKeydown },
      ],
      window
    );
    return cleanup;
  });

  onDestroy(() => {
    if (saveTimer) clearTimeout(saveTimer);
    if (typeof window !== 'undefined') {
      window.removeEventListener('insert', handleInsertSnippet as EventListener);
    }
  });

  // Persist sidebar state
  $effect(() => {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('notes-sidebar-open', String(showSidebar));
    }
  });

  // Persist inspector state
  $effect(() => {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('notes-inspector-open', String(showInspector));
    }
  });

  let lastActiveId = $state<string | null>(null);

  $effect(() => {
    if ($activeNote) {
      if ($activeNote.id !== lastActiveId) {
        titleDraft = $activeNote.title;
        bodyDraft = $activeNote.body;
        tagDraft = '';
        lastActiveId = $activeNote.id;
      }
    } else {
      titleDraft = '';
      bodyDraft = '';
      tagDraft = '';
      lastActiveId = null;
    }
  });

  let allTags = $derived(
    Array.from(new Set($sortedNotes.flatMap((note) => note.tags || []))).sort((left, right) => left.localeCompare(right))
  );

  let tagFrequency = $derived.by(() => {
    const freq = new Map<string, number>();
    $sortedNotes.forEach(note => {
      (note.tags || []).forEach(tag => {
        freq.set(tag, (freq.get(tag) || 0) + 1);
      });
    });
    return freq;
  });

  let frequentTags = $derived(
    Array.from(tagFrequency.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5)
      .map(([tag]) => tag)
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
    if (isCreating) return;
    isCreating = true;
    try {
      const note = await createNewNote();
      setActiveNote(note.id);
      success('Note Created');
    } catch (err) {
      error('Create Failed', err instanceof Error ? err.message : 'Could not create note');
    } finally {
      isCreating = false;
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
      const wasPinned = !!note.pinned_at;
      await toggleNotePin(note.id, wasPinned);
      success(!wasPinned ? 'Note Pinned' : 'Note Unpinned', 'Pinned notes stay at the top');
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
</script>

<div class:editor-mode={!!$activeNote} class:home-mode={!$activeNote} class:sidebar-hidden={!showSidebar} class="notes-view">
  {#if showSidebar}
    <NotesSidebar
      notes={$sortedNotes}
      {collectionFilter}
      {activeTag}
      {allTags}
      onCollectionChange={(f) => (collectionFilter = f)}
      onTagChange={(t) => (activeTag = t)}
      onCreateNote={handleCreateNote}
    />
  {/if}

  <button 
    class="sidebar-toggle" 
    class:sidebar-hidden={!showSidebar}
    onclick={() => showSidebar = !showSidebar}
    title={showSidebar ? 'Hide sidebar' : 'Show sidebar'}
  >
    <Icons.PanelLeft size={18} />
  </button>

  {#if !$activeNote}
    <NotesHome
      notes={filteredNotes}
      {isLoading}
      layout={homeLayout}
      {searchQuery}
      {activeTag}
      onSelectNote={handleSelectNote}
      onContextMenu={showContextMenu}
      onCreateNote={handleCreateNote}
    />
  {:else}
    <section class="notes-rail">
      <div class="rail-header">
        <NotesSearch
          {searchQuery}
          {activeTag}
          onSearchChange={(query) => (searchQuery = query)}
          onTagClear={() => (activeTag = null)}
        />
        <button class="nav-home-btn" onclick={() => setActiveNote(null)} title="Back to Launcher">
          <Icons.LayoutGrid size={18} />
        </button>
      </div>

      <div class="rail-list">
        <NotesList
          notes={filteredNotes}
          activeNoteId={$activeNote?.id}
          onSelectNote={handleSelectNote}
          onDeleteNote={handleDeleteNote}
          onPinNote={handlePinNote}
          onContextMenu={showContextMenu}
        />
      </div>
    </section>

    <section class="note-stage">
      <NoteEditorHeader
        note={$activeNote}
        isSaving={$isSaving}
        showInspector={showInspector}
        onPin={() => handlePinNote($activeNote)}
        onDuplicate={() => handleDuplicateNote($activeNote.id)}
        onDelete={() => handleDeleteNote($activeNote.id)}
        onSlashMenu={openSlashMenu}
        onToggleInspector={() => showInspector = !showInspector}
      />

      <div class="stage-body" class:inspector-open={showInspector}>
        <article class="note-canvas">
          <NotesEditor
            note={$activeNote}
            {titleDraft}
            {bodyDraft}
            isSaving={$isSaving}
            onTitleChange={(title) => {
              titleDraft = title;
              scheduleSave();
            }}
            onBodyChange={(body) => {
              bodyDraft = body;
              scheduleSave();
            }}
          />

          <NoteTagsPanel
            tags={$activeNote.tags}
            {tagDraft}
            onTagDraftChange={(v) => (tagDraft = v)}
            onAddTag={addTag}
            onRemoveTag={removeTag}
          />
        </article>

        {#if showInspector}
          <NoteInspector
            tab={inspectorTab}
            {previewHtml}
            content={bodyDraft}
            createdAt={$activeNote.created_at}
            updatedAt={$activeNote.updated_at}
            wordCount={totalWords}
            {blockCount}
            onTabChange={(t) => (inspectorTab = t)}
          />
        {/if}
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
  onClose={() => contextMenu.visible = false}
/>

<style>
  .notes-view {
    flex: 1;
    height: 100vh;
    display: grid;
    background: #090a0f;
    position: relative;
    overflow: hidden;
  }

  .notes-view.home-mode {
    grid-template-columns: 300px minmax(0, 1fr);
  }

  .notes-view.home-mode.sidebar-hidden {
    grid-template-columns: 0px minmax(0, 1fr);
  }

  .notes-view.editor-mode {
    grid-template-columns: 280px 320px minmax(0, 1fr);
  }

  .notes-view.editor-mode.sidebar-hidden {
    grid-template-columns: 0px 320px minmax(0, 1fr);
  }

  .sidebar-toggle {
    position: absolute;
    top: 20px;
    left: 20px;
    z-index: 150;
    width: 36px;
    height: 36px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: rgba(20, 24, 32, 0.96);
    color: var(--text-2);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
  }

  .sidebar-toggle:hover {
    color: var(--text);
    background: var(--surface);
  }

  .sidebar-toggle.sidebar-hidden {
    left: 20px;
  }

  .notes-rail,
  .note-stage {
    min-height: 0;
    display: flex;
    flex-direction: column;
  }


  .notes-rail {
    border-right: 1px solid var(--border);
    background: rgba(10, 12, 18, 0.98);
    min-width: 0;
  }

  .rail-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 16px;
    padding-left: 60px; /* Space for toggle when sidebar is hidden */
    border-bottom: 1px solid var(--border);
  }

  .nav-home-btn {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-2);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    flex-shrink: 0;
  }

  .rail-list {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
  }

  .note-stage {
    min-width: 0;
    background: #0c0d12;
  }


  .stage-body {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr);
  }

  .stage-body.inspector-open {
    grid-template-columns: minmax(0, 1fr) 320px;
  }

  .note-canvas {
    min-width: 0;
    overflow-y: auto;
    padding: 40px min(60px, 6vw);
    display: flex;
    flex-direction: column;
    gap: 24px;
    position: relative;
    scrollbar-width: thin;
  }


  @media (max-width: 1400px) {
    .stage-body {
      grid-template-columns: minmax(0, 1fr);
    }
  }

  @media (max-width: 1100px) {
    .notes-view.editor-mode {
      grid-template-columns: 0px 280px minmax(0, 1fr);
    }
  }

  @media (max-width: 800px) {
    .notes-view.editor-mode {
      grid-template-columns: 1fr;
    }
    .notes-rail {
      display: none;
    }
  }
</style>
