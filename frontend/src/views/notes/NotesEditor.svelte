<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { Note } from '$lib/api';
  import NotionEditor from './components/NotionEditor.svelte';
  import EmojiPicker from './components/EmojiPicker.svelte';

  interface Props {
    note: Note;
    titleDraft: string;
    bodyDraft: string;
    isSaving?: boolean;
    onTitleChange?: (title: string) => void;
    onBodyChange?: (body: string) => void;
    onIconChange?: (icon: string | null) => void;
  }

  let {
    note,
    titleDraft = '',
    bodyDraft = '',
    isSaving = false,
    onTitleChange,
    onBodyChange,
    onIconChange,
  }: Props = $props();

  let showEmoji = $state(false);

  let wordCount = $derived(
    bodyDraft.trim() ? bodyDraft.trim().split(/\s+/).filter((w) => w.length > 0).length : 0,
  );

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  function handleTitleKeydown(e: KeyboardEvent) {
    // Pressing Enter in the title moves focus into the body editor.
    if (e.key === 'Enter') {
      e.preventDefault();
      const content = document.querySelector('.notion-editor .tiptap-content');
      (content as HTMLElement | null)?.focus();
    }
  }
</script>

<div class="editor-shell">
  <div class="canvas-head">
    <div class="icon-row">
      {#if note.icon}
        <button type="button" class="page-emoji" onclick={() => (showEmoji = !showEmoji)} title="Change icon">
          {note.icon}
        </button>
      {:else}
        <button type="button" class="add-icon" onclick={() => (showEmoji = !showEmoji)}>
          <Icons.Smile size={15} /> Add icon
        </button>
      {/if}
      {#if showEmoji}
        <EmojiPicker
          onPick={(emoji) => onIconChange?.(emoji)}
          onRemove={note.icon ? () => onIconChange?.(null) : undefined}
          onClose={() => (showEmoji = false)}
        />
      {/if}
    </div>
    <input
      type="text"
      value={titleDraft}
      oninput={(e) => onTitleChange?.((e.target as HTMLInputElement).value)}
      onkeydown={handleTitleKeydown}
      class="editor-title"
      placeholder="Untitled"
    />
    <div class="canvas-meta">
      <span><Icons.History size={13} /> Edited {formatDate(note.updated_at || note.created_at)}</span>
      <span>{wordCount} {wordCount === 1 ? 'word' : 'words'}</span>
      {#if isSaving}
        <span class="saving"><Icons.Loader2 size={13} class="spin" /> Saving…</span>
      {/if}
    </div>
  </div>

  <div class="editor-body">
    <NotionEditor
      noteId={note.id}
      markdown={note.body}
      onChange={(body) => onBodyChange?.(body)}
    />
  </div>
</div>

<style>
  .editor-shell {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-height: 0;
  }

  .canvas-head {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .icon-row {
    position: relative;
    min-height: 22px;
  }
  .page-emoji {
    border: none;
    background: transparent;
    font-size: 52px;
    line-height: 1;
    cursor: pointer;
    padding: 0;
    border-radius: 8px;
  }
  .page-emoji:hover {
    background: var(--surface-2, rgba(255, 255, 255, 0.06));
  }
  .add-icon {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: none;
    background: transparent;
    color: var(--muted);
    font-size: 13px;
    cursor: pointer;
    padding: 4px 6px;
    border-radius: 6px;
    opacity: 0;
    transition: opacity 0.15s;
  }
  .editor-shell:hover .add-icon {
    opacity: 1;
  }
  .add-icon:hover {
    background: var(--surface-2, rgba(255, 255, 255, 0.06));
    color: var(--text);
  }

  .editor-title {
    font-size: 38px;
    font-weight: 700;
    line-height: 1.2;
    letter-spacing: -0.025em;
    color: var(--text);
    background: transparent;
    border: none;
    outline: none;
    padding: 0;
    margin: 0;
    font-family: inherit;
    width: 100%;
  }

  .editor-title::placeholder {
    color: var(--muted);
  }

  .canvas-meta {
    display: flex;
    align-items: center;
    gap: 16px;
    font-size: 12px;
    color: var(--muted);
  }

  .canvas-meta span {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }

  .canvas-meta .saving {
    color: var(--accent, #6ea8fe);
  }

  :global(.canvas-meta .spin) {
    animation: spin 0.9s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .editor-body {
    flex: 1;
    min-height: 0;
  }
</style>
