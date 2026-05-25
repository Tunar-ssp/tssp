<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { sortedNotes, activeNote, loadNotes, setActiveNote, updateActiveNote, createNewNote, deleteNote, isSaving } from '$lib/stores/notes';
  import { success, error } from '$lib/stores/notifications';
  import RichEditor from '$lib/components/RichEditor.svelte';
  import Outline from '$lib/components/Outline.svelte';
  import SlashMenu from '$lib/components/SlashMenu.svelte';
  import ColorPicker from '$lib/components/ColorPicker.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import Btn from '$lib/components/Btn.svelte';
  import { onMount } from 'svelte';

  let contextMenu = { visible: false, x: 0, y: 0, note: null as any };
  let searchQuery = $state('');
  let isLoading = $state(true);
  let titleDraft = $state('');
  let bodyDraft = $state('');
  let noteColor = $state('#6ea8ff');
  let showColorPicker = $state(false);
  let showSlashMenu = $state(false);
  let slashMenuPos = { x: 0, y: 0 };

  onMount(async () => {
    await loadNotes();
    isLoading = false;
  });

  $effect(() => {
    if ($activeNote) {
      titleDraft = $activeNote.title;
      bodyDraft = $activeNote.body;
      noteColor = $activeNote.color || '#6ea8ff';
    }
  });

  function handleCreateNote() {
    createNewNote();
    success('New note created');
  }

  function handleSelectNote(id: string) {
    setActiveNote(id);
  }

  async function handleSaveNote() {
    if (!$activeNote) return;
    await updateActiveNote({
      title: titleDraft,
      body: bodyDraft,
      color: noteColor,
    });
    success('Note saved');
  }

  async function handleDeleteNote(id: string) {
    if (!confirm('Delete this note?')) return;
    await deleteNote(id);
    success('Note deleted');
  }

  function showContextMenu(event: MouseEvent, note: any) {
    event.preventDefault();
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
      { label: 'Duplicate', action: () => null },
      { label: 'Archive', action: () => null },
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

  function getWordCount(text: string) {
    return text.trim().split(/\s+/).filter(w => w.length > 0).length;
  }
</script>

<div class="notes-view">
  <div class="notes-sidebar">
    <div class="sidebar-header">
      <h2>Notes</h2>
      <Btn kind="primary" size="sm" on:click={handleCreateNote}>
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
            on:click={() => handleSelectNote(note.id)}
            on:contextmenu={(e) => showContextMenu(e, note)}
          >
            <div class="note-color" style="background: {note.color || '#6ea8ff'}"></div>
            <div class="note-preview">
              <div class="note-title">{note.title || 'Untitled'}</div>
              <div class="note-excerpt">{note.body.substring(0, 40)}...</div>
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
          on:change={handleSaveNote}
          class="editor-title"
          placeholder="Note title..."
        />
        <div class="editor-actions">
          <button class="color-btn" on:click={() => (showColorPicker = !showColorPicker)}>
            <div class="color-preview" style="background: {noteColor}"></div>
            Color
          </button>
          {#if $isSaving}
            <span class="saving">Saving...</span>
          {:else}
            <span class="saved">Saved</span>
          {/if}
        </div>
      </div>

      {#if showColorPicker}
        <div class="color-picker-container">
          <ColorPicker
            color={noteColor}
            onChange={(c) => {
              noteColor = c;
              handleSaveNote();
            }}
          />
        </div>
      {/if}

      <div class="editor-main">
        <div class="editor-column">
          <RichEditor
            content={bodyDraft}
            onChange={(html) => {
              bodyDraft = html;
              handleSaveNote();
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
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
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

  .color-btn {
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

  .color-btn:hover {
    background: var(--surface-3);
  }

  .color-preview {
    width: 16px;
    height: 16px;
    border-radius: 50%;
  }

  .saving,
  .saved {
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .saved {
    color: var(--green);
  }

  .color-picker-container {
    padding: var(--s-4) var(--s-6);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
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

  .editor-textarea {
    flex: 1;
    border: none;
    background: var(--bg);
    color: var(--text);
    padding: var(--s-6);
    font-family: var(--ff-sans);
    font-size: var(--fs-14);
    line-height: var(--lh-relaxed);
    outline: none;
    resize: none;
  }

  .editor-textarea::placeholder {
    color: var(--muted);
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
