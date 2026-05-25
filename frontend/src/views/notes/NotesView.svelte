<script lang="ts">
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
  import RichEditor from '$lib/components/RichEditor.svelte';
  import Outline from '$lib/components/Outline.svelte';
  import SlashMenu from '$lib/components/SlashMenu.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import Btn from '$lib/components/Btn.svelte';
  import { onDestroy, onMount } from 'svelte';

  let contextMenu = $state({ visible: false, x: 0, y: 0, note: null as any });
  let searchQuery = $state('');
  let isLoading = $state(true);
  let titleDraft = $state('');
  let bodyDraft = $state('');
  let tagDraft = $state('');
  let showSlashMenu = $state(false);
  let slashMenuPos = $state({ x: 0, y: 0 });
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  onMount(async () => {
    await loadNotes();
    isLoading = false;
  });

  $effect(() => {
    if ($activeNote) {
      titleDraft = $activeNote.title;
      bodyDraft = $activeNote.body;
      tagDraft = '';
    }
  });

  onDestroy(() => {
    if (saveTimer) clearTimeout(saveTimer);
  });

  async function handleCreateNote() {
    try {
      await createNewNote();
      success('Note Created', 'A new note is ready to edit');
    } catch (err) {
      error('Create Failed', err instanceof Error ? err.message : 'Could not create note');
    }
  }

  function handleSelectNote(id: string) {
    setActiveNote(id);
  }

  function scheduleSave() {
    if (saveTimer) clearTimeout(saveTimer);
    // Increased from 900ms to 5000ms to reduce SD card thrashing on Orange Pi
    // Combined with blur-on-save, this provides good UX while protecting hardware
    saveTimer = setTimeout(() => {
      void handleSaveNote(false);
    }, 5000);
  }

  async function handleFieldBlur() {
    // Immediately save when user leaves the editor (don't wait for debounce)
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


  let filteredNotes = $derived(searchQuery
    ? $sortedNotes.filter(n =>
        n.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        n.body.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : $sortedNotes
  );

  function getContextItems(note: any) {
    return [
      { label: note.pinned_at ? 'Unpin' : 'Pin', action: () => handlePinNote(note) },
      { label: 'Duplicate', action: () => handleDuplicateNote(note.id) },
      { label: 'Delete', action: () => handleDeleteNote(note.id), danger: true },
    ];
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

  function formatDate(timestamp: number) {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  function getWordCount(text: string) {
    return text.trim().split(/\s+/).filter(w => w.length > 0).length;
  }
</script>

<div class="notes-view">
  <div class="notes-sidebar">
    <div class="sidebar-header">
      <h2>Notes</h2>
      <Btn kind="primary" size="sm" onClick={handleCreateNote}>
        <Icons.Plus size={14} />
      </Btn>
    </div>

    <div class="search-box">
      <Icons.Search size={16} />
      <input
        type="text"
        placeholder="Search notes..."
        bind:value={searchQuery}
        class="search-input"
      />
    </div>

    <div class="notes-list">
      {#if isLoading}
        <div class="loading">Loading notes...</div>
      {:else if filteredNotes.length === 0}
        <div class="empty">No notes yet. Create one to get started!</div>
      {:else}
        {#each filteredNotes as note (note.id)}
          <button
            class="note-item"
            class:active={$activeNote?.id === note.id}
            type="button"
            onclick={() => handleSelectNote(note.id)}
            oncontextmenu={(e) => showContextMenu(e, note)}
          >
            <div class="note-color" class:pinned={!!note.pinned_at}></div>
            <div class="note-preview">
              <div class="note-title">{note.title || 'Untitled'}</div>
              <div class="note-excerpt">{note.body.substring(0, 40)}...</div>
              {#if note.tags?.length}
                <div class="note-tags">{note.tags.slice(0, 3).join(' · ')}</div>
              {/if}
            </div>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <div class="editor-container">
    {#if !$activeNote}
      <div class="no-note-selected">
        <Icons.FileText size={48} />
        <h3>No note selected</h3>
        <p>Create a new note or select one from the sidebar</p>
      </div>
    {:else}
      <div class="editor-header">
        <input
          type="text"
          bind:value={titleDraft}
          oninput={scheduleSave}
          onchange={() => handleSaveNote()}
          on:blur={handleFieldBlur}
          class="editor-title"
          placeholder="Note title..."
        />
        <div class="editor-actions">
          <button type="button" class="action-chip" onclick={() => handlePinNote($activeNote)}>
            <Icons.Pin size={14} />
            {$activeNote.pinned_at ? 'Pinned' : 'Pin'}
          </button>
          <button type="button" class="action-chip" onclick={() => handleDuplicateNote($activeNote.id)}>
            <Icons.Copy size={14} />
            Duplicate
          </button>
          {#if $isSaving}
            <span class="saving">Saving...</span>
          {:else}
            <span class="saved">Saved</span>
          {/if}
        </div>
      </div>

      <div class="note-meta-strip">
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

      <div class="editor-main">
        <div class="editor-column">
          <RichEditor
            content={bodyDraft}
            onChange={(html) => {
              bodyDraft = html;
              scheduleSave();
            }}
          />
        </div>

        <Outline content={bodyDraft} onSelectItem={() => {}} />
      </div>

      <div class="editor-footer">
        <span>{getWordCount(bodyDraft)} words</span>
        <span>•</span>
        <span>Updated {formatDate($activeNote.updated_at || $activeNote.created_at)}</span>
      </div>
    {/if}
  </div>

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
    display: flex;
    overflow: hidden;
    background: var(--bg);
  }

  .notes-sidebar {
    flex-shrink: 0;
    width: 280px;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border-right: 1px solid var(--border);
    overflow: hidden;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-4);
    border-bottom: 1px solid var(--border);
  }

  .sidebar-header h2 {
    margin: 0;
    font-size: var(--fs-18);
    font-weight: 600;
    color: var(--text);
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-3);
    margin: var(--s-2);
    background: var(--surface-2);
    border-radius: var(--r-2);
    color: var(--muted);
  }

  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-12);
    outline: none;
  }

  .search-input::placeholder {
    color: var(--muted);
  }

  .notes-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .loading,
  .empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--muted);
    font-size: var(--fs-12);
    padding: var(--s-4);
    text-align: center;
  }

  .note-item {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-3);
    border: none;
    background: transparent;
    color: var(--text);
    cursor: pointer;
    text-align: left;
    transition: all var(--duration-quick) var(--ease-smooth);
    border-left: 3px solid transparent;
    font-family: var(--ff-sans);
  }

  .note-item:hover {
    background: var(--surface-2);
  }

  .note-item.active {
    background: var(--surface-2);
    border-left-color: var(--blue);
  }

  .note-color {
    width: 10px;
    height: 42px;
    border-radius: 999px;
    background: linear-gradient(180deg, var(--blue), var(--violet));
    flex-shrink: 0;
  }

  .note-color.pinned {
    background: linear-gradient(180deg, var(--yellow), var(--orange));
    box-shadow: 0 0 18px rgba(251, 191, 36, 0.16);
  }

  .note-preview {
    flex: 1;
    min-width: 0;
  }

  .note-title {
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-excerpt {
    font-size: var(--fs-11);
    color: var(--muted);
    margin-top: 2px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-tags {
    margin-top: 4px;
    font-size: 10px;
    color: var(--blue);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .editor-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .no-note-selected {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-4);
    color: var(--muted);
  }

  .no-note-selected h3 {
    margin: 0;
    color: var(--text-2);
  }

  .editor-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-6);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .editor-title {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-24);
    font-weight: 700;
    outline: none;
    font-family: var(--ff-sans);
  }

  .editor-title::placeholder {
    color: var(--muted);
  }

  .editor-actions {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    margin-left: var(--s-4);
  }

  .action-chip {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    border-radius: var(--r-2);
    cursor: pointer;
    font-size: var(--fs-12);
    font-weight: 500;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .action-chip:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .saving,
  .saved {
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .saved {
    color: var(--green);
  }

  .note-meta-strip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-4);
    padding: var(--s-4) var(--s-6);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .tag-list {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    min-width: 0;
    flex-wrap: wrap;
  }

  .tag-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 1px solid rgba(110, 168, 255, 0.22);
    background: rgba(110, 168, 255, 0.08);
    color: var(--blue);
    border-radius: 999px;
    padding: 5px 9px;
    font-size: var(--fs-12);
    cursor: pointer;
  }

  .tag-chip:hover {
    border-color: rgba(110, 168, 255, 0.48);
    background: rgba(110, 168, 255, 0.14);
  }

  .tag-empty {
    color: var(--muted);
    font-size: var(--fs-12);
  }

  .tag-form {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    flex-shrink: 0;
  }

  .tag-form input {
    width: 132px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--bg);
    color: var(--text);
    padding: var(--s-2) var(--s-3);
    font-size: var(--fs-12);
    outline: none;
  }

  .tag-form input:focus {
    border-color: var(--blue);
  }

  .tag-form button {
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface-2);
    color: var(--text);
    padding: var(--s-2) var(--s-3);
    cursor: pointer;
    font-size: var(--fs-12);
  }

  .tag-form button:hover {
    background: var(--surface-3);
  }

  .editor-main {
    flex: 1;
    display: flex;
    overflow: hidden;
    gap: 0;
  }

  .editor-column {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .editor-footer {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-3) var(--s-6);
    border-top: 1px solid var(--border);
    background: var(--surface);
    font-size: var(--fs-12);
    color: var(--muted);
  }
</style>
