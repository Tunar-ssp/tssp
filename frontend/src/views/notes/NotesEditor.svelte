<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { Note } from '$lib/api';
  import BlockEditor from '$lib/components/blocks/BlockEditor.svelte';
  import {
    initializeEditor,
    editorMarkdown,
    editorBlocks,
    scheduleAutosave,
  } from '$lib/blocks/editorStore';
  import { markdownToBlocks, blocksToMarkdown } from '$lib/blocks/utils';
  import { onMount } from 'svelte';

  interface $$Props {
    note: Note | null;
    titleDraft: string;
    bodyDraft: string;
    isSaving?: boolean;
    onTitleChange?: (title: string) => void;
    onBodyChange?: (body: string) => void;
    class?: string;
  }

  let {
    note,
    titleDraft = '',
    bodyDraft = '',
    isSaving = false,
    onTitleChange,
    onBodyChange,
    class: className,
  }: $$Props = $props();

  let unsubscribe: (() => void) | null = null;

  $effect(() => {
    if (note) {
      // Cleanup previous subscription if any
      if (unsubscribe) unsubscribe();

      // Parse body into blocks and initialize editor
      const blocks = markdownToBlocks(note.body);
      initializeEditor(blocks, note.id);

      // Set up markdown sync
      unsubscribe = editorMarkdown.subscribe((markdown) => {
        onBodyChange?.(markdown);
        scheduleAutosave();
      });
    }

    return () => {
      if (unsubscribe) {
        unsubscribe();
        unsubscribe = null;
      }
    };
  });

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  function getBlockCount(): number {
    return $editorBlocks.length;
  }

  function getWordCount(): number {
    return $editorMarkdown.trim().split(/\s+/).filter((word) => word.length > 0).length;
  }
</script>

<div class="editor-shell">
  {#if note}
    <div class="canvas-head">
      <input
        type="text"
        value={titleDraft}
        oninput={(e) => onTitleChange?.((e.target as HTMLInputElement).value)}
        onchange={(e) => onTitleChange?.((e.target as HTMLInputElement).value)}
        class="editor-title"
        placeholder="Note title..."
      />
      <div class="canvas-meta">
        <span><Icons.History size={14} /> Edited {formatDate(note.updated_at || note.created_at)}</span>
        <span>{getWordCount()} words</span>
        <span>{getBlockCount()} blocks</span>
      </div>
    </div>

    <div class="editor-body">
      <BlockEditor />
    </div>
  {/if}
</div>

<style>
  .editor-shell {
    flex: 1;
    position: relative;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 0;
  }

  .canvas-head {
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .editor-title {
    font-size: 40px;
    font-weight: 700;
    line-height: 1.2;
    letter-spacing: -0.02em;
    color: var(--text);
    background: transparent;
    border: none;
    outline: none;
    padding: 0;
    margin: 0 0 4px;
    font-family: inherit;
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
    gap: 4px;
  }

  .editor-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
</style>
