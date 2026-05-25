<script lang="ts">
  import { onDestroy } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import type { Note } from '$lib/api';
  import TiptapEditor from '$lib/components/TiptapEditor.svelte';
  import SlashMenu from '$lib/components/SlashMenu.svelte';

  interface $$Props {
    note: Note | null;
    titleDraft: string;
    bodyDraft: string;
    isSaving?: boolean;
    onTitleChange?: (title: string) => void;
    onBodyChange?: (body: string) => void;
    onSlashMenuInsert?: (text: string) => void;
    class?: string;
  }

  let {
    note,
    titleDraft = '',
    bodyDraft = '',
    isSaving = false,
    onTitleChange,
    onBodyChange,
    onSlashMenuInsert,
    class: className,
  }: $$Props = $props();

  let showSlashMenu = $state(false);
  let slashMenuPos = $state({ x: 0, y: 0 });
  let titleInput: HTMLInputElement | null = $state(null);

  function handleTitleChange(value: string) {
    titleDraft = value;
    onTitleChange?.(value);
  }

  function handleBodyChange(value: string) {
    bodyDraft = value;
    onBodyChange?.(value);
  }

  function openSlashMenu(event: MouseEvent) {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    slashMenuPos = { x: rect.left, y: rect.bottom + 10 };
    showSlashMenu = true;
  }

  function handleSlashInsert(event: CustomEvent<{ text: string }>) {
    const snippet = event.detail.text;
    if (snippet) {
      bodyDraft = bodyDraft.trim().length ? `${bodyDraft.trimEnd()}\n\n${snippet}` : snippet;
      handleBodyChange(bodyDraft);
      onSlashMenuInsert?.(snippet);
      showSlashMenu = false;
    }
  }

  onDestroy(() => {
    showSlashMenu = false;
  });
</script>

<div class="notes-editor {className || ''}">
  {#if note}
    <div class="editor-header">
      <input
        bind:this={titleInput}
        type="text"
        class="note-title-input"
        placeholder="Note title..."
        value={titleDraft}
        onchange={(e) => handleTitleChange(e.currentTarget.value)}
        oninput={(e) => handleTitleChange(e.currentTarget.value)}
      />
      {#if isSaving}
        <span class="saving-indicator">
          <Icons.Loader2 size={14} class="spin" />
          Saving...
        </span>
      {/if}
    </div>

    <div class="editor-body">
      <TiptapEditor
        content={bodyDraft}
        onchange={handleBodyChange}
        placeholder="Start typing... Use /slash for commands"
      />
      <button class="slash-menu-button" onclick={openSlashMenu} title="Insert slash command">
        <Icons.Sliders size={16} />
      </button>
    </div>

    {#if showSlashMenu}
      <SlashMenu
        position={slashMenuPos}
        oninsert={handleSlashInsert}
        onclose={() => (showSlashMenu = false)}
      />
    {/if}
  {:else}
    <div class="editor-empty">
      <Icons.FileText size={48} />
      <p>Select a note to edit</p>
    </div>
  {/if}
</div>

<style>
  .notes-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
  }

  .editor-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: var(--s-4);
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
  }

  .note-title-input {
    flex: 1;
    font-size: 20px;
    font-weight: 600;
    color: var(--text);
    background: transparent;
    border: none;
    outline: none;
    padding: 0;
  }

  .note-title-input::placeholder {
    color: var(--muted);
  }

  .saving-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-2);
  }

  .saving-indicator :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .editor-body {
    flex: 1;
    position: relative;
    overflow: hidden;
  }

  .slash-menu-button {
    position: absolute;
    bottom: 16px;
    right: 16px;
    width: 40px;
    height: 40px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    border-radius: var(--r-2);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .slash-menu-button:hover {
    background: var(--surface);
    color: var(--text);
    border-color: var(--text-2);
  }

  .editor-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--muted);
    gap: 12px;
  }

  .editor-empty p {
    margin: 0;
    font-size: 14px;
  }
</style>
