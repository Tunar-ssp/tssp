<script lang="ts">
  /**
   * Enhanced notes container with editor, outline, and metadata
   * Combines BlockEditor with right-rail outline and inspector
   */

  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import BlockEditor from './BlockEditor.svelte';
  import BlockOutline from './BlockOutline.svelte';
  import type { Note } from '$lib/api';
  import {
    editorIsDirty,
    editorIsSaving,
    editorMarkdown,
  } from '$lib/blocks/editorStore';

  interface Props {
    note: Note | null;
    title: string;
    onTitleChange?: (title: string) => void;
    onSave?: () => void;
    isSaving?: boolean;
  }

  let { note, title, onTitleChange, onSave, isSaving = false }: Props = $props();

  let showOutline = $state(true);

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  function handleSave() {
    onSave?.();
  }
</script>

<div class="notes-container">
  <!-- Header -->
  <div class="notes-header">
    <div class="header-left">
      <input
        type="text"
        value={title}
        oninput={(e) => onTitleChange?.((e.target as HTMLInputElement).value)}
        class="note-title"
        placeholder="Note title..."
      />
      {#if note}
        <div class="note-metadata">
          <span><Icons.Calendar size={14} /> {formatDate(note.updated_at || note.created_at)}</span>
          <span><Icons.Tag size={14} /> {note.tags.length} tags</span>
        </div>
      {/if}
    </div>

    <div class="header-right">
      {#if $editorIsDirty && !$editorIsSaving}
        <button class="save-btn primary" onclick={handleSave}>
          <Icons.Save size={16} />
          Save
        </button>
      {/if}

      {#if $editorIsSaving}
        <div class="saving-indicator">
          <Icons.Loader size={16} class="spinner" />
          Saving...
        </div>
      {/if}

      <button
        class="outline-toggle"
        onclick={() => (showOutline = !showOutline)}
        title={showOutline ? 'Hide outline' : 'Show outline'}
      >
        <Icons.PanelRight size={16} />
      </button>
    </div>
  </div>

  <!-- Main content area -->
  <div class="notes-content">
    <!-- Editor -->
    <div class="editor-section">
      <BlockEditor />
    </div>

    <!-- Right rail outline -->
    {#if showOutline}
      <div class="outline-section">
        <BlockOutline />
      </div>
    {/if}
  </div>
</div>

<style>
  .notes-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background-color: var(--bg);
  }

  .notes-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    border-bottom: 1px solid var(--border);
    gap: 16px;
    flex-shrink: 0;
  }

  .header-left {
    flex: 1;
    min-width: 0;
  }

  .note-title {
    width: 100%;
    font-size: 24px;
    font-weight: 600;
    color: var(--text);
    background: transparent;
    border: none;
    outline: none;
    padding: 0;
    margin: 0 0 8px 0;
  }

  .note-title::placeholder {
    color: var(--muted);
  }

  .note-metadata {
    display: flex;
    gap: 16px;
    font-size: 12px;
    color: var(--muted);
  }

  .note-metadata span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
  }

  .save-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    background-color: rgba(59, 130, 246, 1);
    color: white;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    transition: background-color 0.2s;
  }

  .save-btn:hover {
    background-color: rgba(59, 130, 246, 0.9);
  }

  .saving-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--muted);
  }

  .spinner {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .outline-toggle {
    padding: 6px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-secondary);
    color: var(--text);
    cursor: pointer;
    transition: all 0.2s;
  }

  .outline-toggle:hover {
    background-color: rgba(0, 0, 0, 0.05);
  }

  .notes-content {
    display: flex;
    flex: 1;
    overflow: hidden;
    gap: 0;
  }

  .editor-section {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
  }

  .outline-section {
    width: 280px;
    min-width: 280px;
    max-width: 280px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
