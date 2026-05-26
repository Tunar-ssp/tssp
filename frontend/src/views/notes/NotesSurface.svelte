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

  function getWordCount(text: string): number {
    return text.trim().split(/\s+/).filter(Boolean).length;
  }
</script>

<div class:editor-mode={!!$activeNote} class:home-mode={!$activeNote} class="notes-view">
  <NotesSidebar
    notes={$sortedNotes}
    {collectionFilter}
    {activeTag}
    {allTags}
    onCollectionChange={(f) => (collectionFilter = f)}
    onTagChange={(t) => (activeTag = t)}
    onCreateNote={handleCreateNote}
  />

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
      <NotesSearch
        {searchQuery}
        {activeTag}
        onSearchChange={(query) => (searchQuery = query)}
        onTagClear={() => (activeTag = null)}
      />

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
        onPin={() => handlePinNote($activeNote)}
        onDuplicate={() => handleDuplicateNote($activeNote.id)}
        onDelete={() => handleDeleteNote($activeNote.id)}
        onSlashMenu={openSlashMenu}
      />

      <div class="stage-body">
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

          <button class="slash-menu-button" onclick={openSlashMenu} title="Insert slash command">
            <Icons.Sparkles size={16} />
          </button>
        </article>

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

  .notes-rail,
  .note-stage {
    min-height: 0;
  }


  .notes-rail {
    border-right: 1px solid var(--border);
    background: rgba(13, 15, 21, 0.96);
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .rail-list {
    overflow: auto;
    padding: 8px 10px 18px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .note-stage {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
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
    position: relative;
  }

  .slash-menu-button {
    position: absolute;
    bottom: 20px;
    right: 24px;
    width: 48px;
    height: 48px;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(18, 22, 31, 0.96);
    color: var(--text-2);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .slash-menu-button:hover {
    color: var(--text);
    background: rgba(25, 29, 40, 0.96);
  }


  @media (max-width: 1400px) {
    .notes-view.editor-mode {
      grid-template-columns: 280px 320px minmax(0, 1fr);
    }

    .stage-body {
      grid-template-columns: minmax(0, 1fr);
    }
  }

  @media (max-width: 960px) {
    .notes-view.home-mode,
    .notes-view.editor-mode {
      grid-template-columns: 1fr;
    }

    .notes-rail {
      border-right: 0;
      border-bottom: 1px solid var(--border);
      max-height: 360px;
    }
  }
</style>
