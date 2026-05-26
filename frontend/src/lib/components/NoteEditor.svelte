<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { Note } from '$lib/api';
  import { isSaving, updateActiveNote, deleteNote } from '$lib/stores/notes';

  interface $$Props {
    note?: Note | null;
  }

  let {
    note = null,
  }: $$Props = $props();

  let title = '';
  let body = '';
  let tags = '';

  let saveTimeout: number;

  $effect.pre(() => {
    if (note) {
      title = note.title;
      body = note.body;
      tags = note.tags.join(', ');
    }
  });

  function handleTitleChange(e: Event) {
    const target = e.target as HTMLInputElement;
    title = target.value;
    scheduleAutoSave();
  }

  function handleBodyChange(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    body = target.value;
    scheduleAutoSave();
  }

  function handleTagsChange(e: Event) {
    const target = e.target as HTMLInputElement;
    tags = target.value;
    scheduleAutoSave();
  }

  function scheduleAutoSave() {
    clearTimeout(saveTimeout);
    saveTimeout = window.setTimeout(() => {
      if (note) {
        const newTags = tags
          .split(',')
          .map(t => t.trim())
          .filter(t => t.length > 0);

        updateActiveNote({
          title,
          body,
          tags: newTags,
        }).catch(console.error);
      }
    }, 1000);
  }

  function handleDelete() {
    if (note && confirm(`Delete "${note.title}"?`)) {
      deleteNote(note.id).catch(console.error);
    }
  }
</script>

{#if note}
  <div class="editor-container">
    <div class="editor-header">
      <div class="header-left">
        <input
          type="text"
          class="title-input"
          value={title}
          onchange={handleTitleChange}
          placeholder="Note title"
        />
      </div>

      <div class="header-right">
        {#if $isSaving}
          <div class="saving-indicator">
            <div class="spinner"></div>
            <span>Saving...</span>
          </div>
        {:else}
          <span class="saved-indicator">Saved</span>
        {/if}

        <button class="icon-btn" onclick={handleDelete}>
          <Icons.Trash2 size={16} />
        </button>
      </div>
    </div>

    <div class="editor-body">
      <textarea
        class="body-input"
        value={body}
        oninput={handleBodyChange}
        placeholder="Start typing..."
      />
    </div>

    <div class="editor-footer">
      <input
        type="text"
        class="tags-input"
        value={tags}
        onchange={handleTagsChange}
        placeholder="Add tags separated by commas..."
      />
    </div>
  </div>
{:else}
  <div class="empty-state">
    <Icons.FileText size={40} />
    <p>No note selected</p>
    <p class="secondary">Choose a note from the list or create a new one</p>
  </div>
{/if}

<style>
  .editor-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg);
  }

  .editor-header {
    padding: 16px 24px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    background: var(--surface);
  }

  .header-left {
    flex: 1;
    min-width: 0;
  }

  .title-input {
    width: 100%;
    border: none;
    background: transparent;
    font-size: var(--fs-20);
    font-weight: 600;
    color: var(--text);
    outline: none;
    padding: 0;
  }

  .title-input::placeholder {
    color: var(--muted);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
  }

  .saving-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--fs-12);
    color: var(--text-2);
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 1px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .saved-indicator {
    font-size: var(--fs-12);
    color: var(--green);
  }

  .icon-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    border-radius: var(--r-2);
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .icon-btn:hover {
    background: var(--surface-2);
    color: var(--danger);
    border-color: var(--danger);
  }

  .editor-body {
    flex: 1;
    overflow: hidden;
    display: flex;
  }

  .body-input {
    flex: 1;
    padding: 20px 24px;
    border: none;
    background: transparent;
    color: var(--text);
    font-family: var(--ff-mono);
    font-size: var(--fs-14);
    line-height: 1.6;
    outline: none;
    resize: none;
  }

  .body-input::placeholder {
    color: var(--muted);
  }

  .editor-footer {
    padding: 12px 24px;
    border-top: 1px solid var(--border);
    background: var(--surface);
  }

  .tags-input {
    width: 100%;
    padding: 8px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface-2);
    color: var(--text-2);
    font-size: var(--fs-12);
    outline: none;
  }

  .tags-input:focus {
    border-color: var(--blue);
    color: var(--text);
  }

  .tags-input::placeholder {
    color: var(--muted);
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
  }

  .empty-state p {
    margin: 0;
  }

  .empty-state .secondary {
    font-size: var(--fs-12);
    color: var(--dim);
  }
</style>
