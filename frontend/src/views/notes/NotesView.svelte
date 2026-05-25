<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { sortedNotes, activeNote, loadNotes, setActiveNote, updateActiveNote, createNewNote, deleteNote, isSaving } from '$lib/stores/notes';
  import { success, error } from '$lib/stores/notifications';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import { onMount } from 'svelte';

  let contextMenu = { visible: false, x: 0, y: 0, note: null as any };
  let searchQuery = '';
  let isLoading = true;
  let titleDraft = '';
  let bodyDraft = '';
  let tagInput = '';
  let showTagInput = false;

  onMount(async () => {
    await loadNotes();
    isLoading = false;
  });

  $: {
    if ($activeNote) {
      titleDraft = $activeNote.title;
      bodyDraft = $activeNote.body;
    }
  }

  function handleCreateNote() {
    createNewNote();
  }

  function handleSelectNote(id: string) {
    setActiveNote(id);
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

  async function handleSaveNote() {
    if (!$activeNote) return;
    await updateActiveNote({
      title: titleDraft,
      body: bodyDraft,
    });
    success('Note saved');
  }

  async function handleDeleteNote(note: any) {
    if (confirm(`Delete "${note.title || 'Untitled'}"?`)) {
      await deleteNote(note.id);
    }
  }

  async function handlePin(note: any) {
    // TODO: Implement pin functionality with backend
    success(note.pinned_at ? 'Note unpinned' : 'Note pinned');
  }

  async function handleDuplicate(note: any) {
    // TODO: Implement duplicate functionality
    success('Note duplicated');
  }

  function addTag() {
    if (!tagInput.trim() || !$activeNote) return;
    const newTags = [...$activeNote.tags, tagInput.trim()];
    updateActiveNote({ tags: newTags });
    tagInput = '';
    showTagInput = false;
  }

  function removeTag(tag: string) {
    if (!$activeNote) return;
    const newTags = $activeNote.tags.filter((t: string) => t !== tag);
    updateActiveNote({ tags: newTags });
  }

  $: filteredNotes = searchQuery
    ? $sortedNotes.filter(n =>
        n.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        n.body.toLowerCase().includes(searchQuery.toLowerCase()) ||
        n.tags.some(t => t.toLowerCase().includes(searchQuery.toLowerCase()))
      )
    : $sortedNotes;

  function getContextItems(note: any) {
    return [
      { label: note.pinned_at ? 'Unpin' : 'Pin', action: () => handlePin(note) },
      { label: 'Duplicate', action: () => handleDuplicate(note) },
      { label: 'Delete', action: () => handleDeleteNote(note), danger: true },
    ];
  }
</script>

<div class="notes-view">
  <div class="sidebar">
    <div class="header">
      <h2>Notes</h2>
      <button class="create-btn" on:click={handleCreateNote}>
        <Icons.Plus size={16} />
        New
      </button>
    </div>

    <div class="search-bar">
      <Icons.Search size={16} />
      <input
        type="text"
        placeholder="Search notes..."
        bind:value={searchQuery}
      />
    </div>

    <div class="notes-list">
      {#if isLoading}
        <div class="loading">
          <div class="spinner" />
          Loading notes...
        </div>
      {:else if filteredNotes.length === 0}
        <div class="empty">
          <Icons.BookOpen size={40} />
          <h3>No notes</h3>
          <p>Create a new note to get started</p>
        </div>
      {:else}
        {#each filteredNotes as note (note.id)}
          <div
            class="note-row"
            class:active={$activeNote?.id === note.id}
            on:click={() => handleSelectNote(note.id)}
            on:contextmenu={(e) => showContextMenu(e, note)}
          >
            <div class="note-content">
              <div class="note-title">{note.title || 'Untitled'}</div>
              <div class="note-preview">{note.body.slice(0, 60).replace(/\n/g, ' ')}</div>
              <div class="note-meta">
                {#if note.pinned_at}
                  <Icons.Pin size={12} class="pinned" />
                {/if}
                <span>{note.tags.length > 0 ? note.tags.join(', ') : 'untagged'}</span>
              </div>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <div class="editor">
    {#if !$activeNote}
      <div class="empty-editor">
        <Icons.FileText size={48} />
        <h3>Select a note</h3>
        <p>Click a note to edit or create a new one</p>
      </div>
    {:else}
      <div class="editor-header">
        <input
          type="text"
          class="title-input"
          placeholder="Untitled note"
          bind:value={titleDraft}
          on:change={handleSaveNote}
        />
        <div class="header-actions">
          {#if $isSaving}
            <span class="saving">Saving...</span>
          {/if}
          <button class="action-btn" on:click={handleSaveNote}>
            <Icons.Save size={14} />
            Save
          </button>
          <button
            class="action-btn"
            on:click={(e) => showContextMenu(e, $activeNote)}
          >
            <Icons.MoreVertical size={14} />
          </button>
        </div>
      </div>

      <div class="editor-content">
        <div class="editor-pane">
          <textarea
            class="body-input"
            placeholder="Start typing..."
            bind:value={bodyDraft}
            on:change={handleSaveNote}
          ></textarea>
        </div>

        <div class="sidebar-pane">
          <div class="section">
            <h4>Tags</h4>
            <div class="tags">
              {#each $activeNote.tags as tag}
                <span class="tag">
                  {tag}
                  <button
                    class="tag-remove"
                    on:click={() => removeTag(tag)}
                  >
                    <Icons.X size={12} />
                  </button>
                </span>
              {/each}
              {#if showTagInput}
                <input
                  type="text"
                  class="tag-input"
                  placeholder="Add tag..."
                  bind:value={tagInput}
                  on:keydown={(e) => {
                    if (e.key === 'Enter') addTag();
                    if (e.key === 'Escape') showTagInput = false;
                  }}
                  autofocus
                />
              {:else}
                <button class="tag-add" on:click={() => showTagInput = true}>
                  <Icons.Plus size={12} /> Add tag
                </button>
              {/if}
            </div>
          </div>

          <div class="section">
            <h4>Info</h4>
            <div class="info">
              <div class="info-row">
                <span>Created</span>
                <span class="value">{new Date($activeNote.created_at * 1000).toLocaleDateString()}</span>
              </div>
              <div class="info-row">
                <span>Updated</span>
                <span class="value">{new Date($activeNote.updated_at * 1000).toLocaleDateString()}</span>
              </div>
              <div class="info-row">
                <span>Words</span>
                <span class="value">{bodyDraft.split(/\s+/).filter(w => w).length}</span>
              </div>
            </div>
          </div>

          <div class="section">
            <h4>Actions</h4>
            <button class="action-link" on:click={() => handlePin($activeNote)}>
              <Icons.Pin size={14} />
              {$activeNote.pinned_at ? 'Unpin' : 'Pin'}
            </button>
            <button class="action-link" on:click={() => handleDuplicate($activeNote)}>
              <Icons.Copy size={14} />
              Duplicate
            </button>
            <button
              class="action-link danger"
              on:click={() => handleDeleteNote($activeNote)}
            >
              <Icons.Trash2 size={14} />
              Delete
            </button>
          </div>
        </div>
      </div>
    {/if}
  </div>
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

  .sidebar {
    flex-shrink: 0;
    width: 280px;
    height: 100%;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .header {
    padding: 20px 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--surface);
  }

  .header h2 {
    margin: 0;
    font-size: var(--fs-20);
    color: var(--text);
  }

  .create-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: var(--r-2);
    border: 1px solid var(--border);
    background: var(--blue);
    color: #0a1228;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.15s;
    font-size: var(--fs-12);
  }

  .create-btn:hover {
    opacity: 0.9;
  }

  .search-bar {
    padding: 12px 12px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--surface);
    color: var(--muted);
  }

  .search-bar input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-13);
    outline: none;
  }

  .search-bar input::placeholder {
    color: var(--muted);
  }

  .notes-list {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
  }

  .note-row {
    padding: 12px;
    border-bottom: 1px solid var(--hairline);
    cursor: pointer;
    transition: background 0.15s;
  }

  .note-row:hover {
    background: var(--surface-2);
  }

  .note-row.active {
    background: var(--surface-3);
  }

  .note-content {
    min-width: 0;
  }

  .note-title {
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: var(--fs-13);
  }

  .note-preview {
    font-size: var(--fs-12);
    color: var(--muted);
    margin-top: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .note-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--muted);
    margin-top: 6px;
  }

  .pinned {
    color: var(--orange);
  }

  .editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .empty-editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
  }

  .empty-editor h3 {
    margin: 0;
    color: var(--text-2);
  }

  .empty-editor p {
    margin: 0;
    font-size: var(--fs-12);
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
  }

  .empty h3 {
    margin: 0;
    color: var(--text-2);
  }

  .empty p {
    margin: 0;
    font-size: var(--fs-12);
  }

  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .editor-header {
    padding: 20px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    background: var(--surface);
  }

  .title-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-24);
    font-weight: 600;
    outline: none;
  }

  .title-input::placeholder {
    color: var(--muted);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .saving {
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .action-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    border: none;
    border-radius: var(--r-2);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .action-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .editor-content {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 240px;
    overflow: hidden;
    gap: 1px;
    background: var(--hairline);
  }

  .editor-pane {
    overflow: hidden;
    background: var(--bg);
  }

  .body-input {
    width: 100%;
    height: 100%;
    border: none;
    background: var(--bg);
    color: var(--text);
    padding: 20px;
    font-family: var(--ff-mono);
    font-size: var(--fs-13);
    line-height: 1.6;
    outline: none;
    resize: none;
  }

  .body-input::placeholder {
    color: var(--muted);
  }

  .sidebar-pane {
    background: var(--surface);
    overflow: auto;
    padding: 16px;
    border-left: 1px solid var(--border);
  }

  .section {
    margin-bottom: 24px;
  }

  .section h4 {
    margin: 0 0 12px;
    font-size: var(--fs-11);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: var(--blue-subtle);
    color: var(--blue);
    border-radius: var(--r-2);
    font-size: var(--fs-12);
  }

  .tag-remove {
    border: none;
    background: transparent;
    color: inherit;
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
    opacity: 0.7;
    transition: opacity 0.15s;
  }

  .tag-remove:hover {
    opacity: 1;
  }

  .tag-input {
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--bg);
    color: var(--text);
    font-size: var(--fs-12);
    outline: none;
    width: 100%;
    max-width: 120px;
  }

  .tag-add {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border: 1px dashed var(--border);
    border-radius: var(--r-2);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: var(--fs-12);
    transition: all 0.15s;
  }

  .tag-add:hover {
    border-color: var(--text);
    color: var(--text);
  }

  .info {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    font-size: var(--fs-12);
    color: var(--text-2);
  }

  .info-row .value {
    color: var(--text);
    font-weight: 500;
  }

  .action-link {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: var(--fs-12);
    transition: color 0.15s;
  }

  .action-link:hover {
    color: var(--text);
  }

  .action-link.danger {
    color: var(--danger);
  }

  .action-link.danger:hover {
    color: var(--danger);
    opacity: 0.8;
  }
</style>
