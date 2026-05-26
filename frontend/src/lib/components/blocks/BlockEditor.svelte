<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import type { Block, SlashCommand } from '$lib/blocks/types';
  import {
    editorBlocks,
    editorSelection,
    editorIsDirty,
    editorIsSaving,
    updateBlock,
    insertBlock,
    deleteBlock,
    setSelection,
    scheduleAutosave,
    editorMarkdown,
    undo,
    redo,
    canUndo,
    canRedo,
  } from '$lib/blocks/editorStore';
  import { createBlock, generateBlockId, detectBlockType, extractMarkdownPrefix, findBlockById, markdownToBlocks } from '$lib/blocks/utils';
  import { findCommand } from '$lib/blocks/slashCommands';
  import BlockParagraph from './BlockParagraph.svelte';
  import BlockHeading from './BlockHeading.svelte';
  import BlockListItem from './BlockListItem.svelte';
  import BlockQuote from './BlockQuote.svelte';
  import BlockCallout from './BlockCallout.svelte';
  import BlockCode from './BlockCode.svelte';
  import SlashCommandMenu from './SlashCommandMenu.svelte';

  interface Props {
    class?: string;
  }

  let { class: className }: Props = $props();

  let editorElement: HTMLDivElement;
  let showSlashMenu = $state(false);
  let slashQuery = $state('');
  let slashPosition = $state({ top: 0, left: 0 });
  let currentBlockId = $state<string | null>(null);

  onMount(() => {
    editorElement?.focus();
  });

  /**
   * Handle input from a block and update it
   */
  function handleBlockUpdate(blockId: string, content: string) {
    updateBlock(blockId, { content });
    scheduleAutosave();

    // Auto-detect block type from markdown
    const detectedType = detectBlockType(content);
    const block = findBlockById($editorBlocks, blockId);

    if (block && block.type !== detectedType && content.includes(' ')) {
      const prefix = extractMarkdownPrefix(content);
      const cleanContent = content.slice(prefix.length);

      updateBlock(blockId, {
        type: detectedType,
        content: cleanContent,
      });
    }
  }

  /**
   * Handle keyboard events in blocks
   */
  function handleBlockKeyDown(blockId: string, e: KeyboardEvent) {
    const block = findBlockById($editorBlocks, blockId);
    if (!block) return;

    currentBlockId = blockId;

    // Slash command trigger
    if (e.key === '/' && !showSlashMenu) {
      e.preventDefault();
      showSlashMenu = true;
      slashQuery = '';
      updateSlashMenuPosition(blockId);
      return;
    }

    // Skip slash menu functionality for non-paragraph blocks
    if (showSlashMenu) {
      if (e.key === 'Escape') {
        showSlashMenu = false;
      }
      return;
    }

    // Backspace on empty block - delete it
    if (e.key === 'Backspace' && block.content === '') {
      e.preventDefault();
      if ($editorBlocks.length > 1) {
        deleteBlock(blockId);
      }
      return;
    }

    // Enter - create new block
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      const newBlock = createBlock({ type: block.type });
      insertBlock(newBlock, blockId);
      return;
    }

    // Shift+Enter - soft break (add newline in content)
    if (e.key === 'Enter' && e.shiftKey) {
      e.preventDefault();
      // This will be handled by contenteditable naturally
      return;
    }

    // Tab - indent (future: implement nesting)
    if (e.key === 'Tab') {
      e.preventDefault();
      // TODO: implement block nesting
      return;
    }

    // Cmd/Ctrl+B - bold shortcut
    if ((e.metaKey || e.ctrlKey) && e.key === 'b') {
      e.preventDefault();
      document.execCommand('bold');
      return;
    }

    // Cmd/Ctrl+I - italic shortcut
    if ((e.metaKey || e.ctrlKey) && e.key === 'i') {
      e.preventDefault();
      document.execCommand('italic');
      return;
    }

    // Cmd/Ctrl+Z - undo
    if ((e.metaKey || e.ctrlKey) && e.key === 'z' && !e.shiftKey) {
      e.preventDefault();
      undo();
      return;
    }

    // Cmd/Ctrl+Shift+Z or Cmd/Ctrl+Y - redo
    if (((e.metaKey || e.ctrlKey) && e.key === 'z' && e.shiftKey) ||
        ((e.metaKey || e.ctrlKey) && e.key === 'y')) {
      e.preventDefault();
      redo();
      return;
    }

    // Cmd/Ctrl+Backspace at start of line - delete block
    if (e.key === 'Backspace' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      if ($editorBlocks.length > 1) {
        deleteBlock(blockId);
      }
      return;
    }
  }

  /**
   * Handle slash menu input
   */
  function handleSlashInput(e: InputEvent) {
    const target = e.target as HTMLDivElement;
    const text = target.textContent || '';

    // Extract what's after the slash
    const slashIndex = text.lastIndexOf('/');
    if (slashIndex === -1) {
      showSlashMenu = false;
      return;
    }

    slashQuery = text.slice(slashIndex + 1);

    if (slashQuery.includes(' ') || slashQuery.length > 50) {
      showSlashMenu = false;
    }
  }

  /**
   * Position the slash menu near the cursor
   */
  function updateSlashMenuPosition(blockId: string) {
    // Get the block element and position menu below it
    const blockElements = editorElement?.querySelectorAll('[data-block-id]');
    for (const el of blockElements || []) {
      if (el.getAttribute('data-block-id') === blockId) {
        const rect = el.getBoundingClientRect();
        slashPosition = {
          top: rect.top + rect.height + 8,
          left: rect.left,
        };
        break;
      }
    }
  }

  /**
   * Handle slash command selection
   */
  function handleSlashCommandSelect(command: SlashCommand) {
    showSlashMenu = false;

    if (!currentBlockId) return;

    const block = findBlockById($editorBlocks, currentBlockId);
    if (!block) return;

    // Remove the "/" from the current block
    const cleanContent = block.content.replace(/\/$/, '');
    updateBlock(currentBlockId, { content: cleanContent });

    // Create new block from command
    const newBlock = createBlock(command.action(currentBlockId), generateBlockId());
    insertBlock(newBlock, currentBlockId);

    // Delete original block if empty
    if (cleanContent === '') {
      deleteBlock(currentBlockId);
    }
  }

  onDestroy(() => {
    // Cleanup if needed
  });
</script>

<div bind:this={editorElement} class="block-editor {className}">
  {#if $editorBlocks.length === 0}
    <div class="empty-state">
      <Icons.Feather size={32} />
      <p>Start typing or press <kbd>/</kbd> for commands</p>
    </div>
  {:else}
    <div class="blocks-container" role="document">
      {#each $editorBlocks as block (block.id)}
        <div class="block-wrapper" data-block-id={block.id}>
          {#if block.type === 'paragraph'}
            <BlockParagraph
              {block}
              isSelected={block.id === $editorSelection?.blockId}
              onUpdate={(content) => handleBlockUpdate(block.id, content)}
              onKeyDown={(e) => handleBlockKeyDown(block.id, e)}
            />
          {:else if block.type === 'heading_1'}
            <BlockHeading
              {block}
              level={1}
              isSelected={block.id === $editorSelection?.blockId}
              onUpdate={(content) => handleBlockUpdate(block.id, content)}
              onKeyDown={(e) => handleBlockKeyDown(block.id, e)}
            />
          {:else if block.type === 'heading_2'}
            <BlockHeading
              {block}
              level={2}
              isSelected={block.id === $editorSelection?.blockId}
              onUpdate={(content) => handleBlockUpdate(block.id, content)}
              onKeyDown={(e) => handleBlockKeyDown(block.id, e)}
            />
          {:else if block.type === 'heading_3'}
            <BlockHeading
              {block}
              level={3}
              isSelected={block.id === $editorSelection?.blockId}
              onUpdate={(content) => handleBlockUpdate(block.id, content)}
              onKeyDown={(e) => handleBlockKeyDown(block.id, e)}
            />
          {:else if block.type === 'bulleted_list'}
            <BlockListItem
              {block}
              type="bulleted"
              isSelected={block.id === $editorSelection?.blockId}
              onUpdate={(content) => handleBlockUpdate(block.id, content)}
              onKeyDown={(e) => handleBlockKeyDown(block.id, e)}
            />
          {:else if block.type === 'numbered_list'}
            <BlockListItem
              {block}
              type="numbered"
              isSelected={block.id === $editorSelection?.blockId}
              onUpdate={(content) => handleBlockUpdate(block.id, content)}
              onKeyDown={(e) => handleBlockKeyDown(block.id, e)}
            />
          {:else if block.type === 'checklist'}
            <BlockListItem
              {block}
              type="checklist"
              isSelected={block.id === $editorSelection?.blockId}
              onUpdate={(content) => handleBlockUpdate(block.id, content)}
              onToggleChecked={() => {
                const checklistBlock = block as any;
                const updated = { ...checklistBlock, checked: !checklistBlock.checked };
                editorBlocks.update((blocks) => {
                  const updateRecursive = (blocks: Block[]): Block[] => {
                    return blocks.map((b) => (b.id === block.id ? updated : b));
                  };
                  return updateRecursive(blocks);
                });
                editorIsDirty.set(true);
              }}
              onKeyDown={(e) => handleBlockKeyDown(block.id, e)}
            />
          {:else if block.type === 'quote'}
            <BlockQuote
              {block}
              isSelected={block.id === $editorSelection?.blockId}
              onUpdate={(content) => handleBlockUpdate(block.id, content)}
              onKeyDown={(e) => handleBlockKeyDown(block.id, e)}
            />
          {:else if block.type === 'callout'}
            <BlockCallout
              block={block as any}
              isSelected={block.id === $editorSelection?.blockId}
              onUpdate={(content) => handleBlockUpdate(block.id, content)}
              onKeyDown={(e) => handleBlockKeyDown(block.id, e)}
            />
          {:else if block.type === 'code'}
            <BlockCode
              block={block as any}
              isSelected={block.id === $editorSelection?.blockId}
              onUpdate={(content) => handleBlockUpdate(block.id, content)}
              onKeyDown={(e) => handleBlockKeyDown(block.id, e)}
            />
          {:else if block.type === 'divider'}
            <div class="block-divider" />
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  {#if showSlashMenu && currentBlockId}
    <SlashCommandMenu
      query={slashQuery}
      position={slashPosition}
      onSelect={handleSlashCommandSelect}
      onClose={() => (showSlashMenu = false)}
    />
  {/if}

  {#if $editorIsDirty || $editorIsSaving || $canUndo || $canRedo}
    <div class="editor-status" class:saving={$editorIsSaving}>
      <div class="status-left">
        {#if $editorIsSaving}
          <span class="status-indicator">●</span>
          <span>Saving...</span>
        {:else if $editorIsDirty}
          <span class="status-indicator unsaved">●</span>
          <span>Unsaved changes</span>
        {/if}
      </div>

      <div class="status-right">
        <button
          class="history-btn"
          disabled={!$canUndo}
          onclick={undo}
          title="Undo (Cmd+Z)"
          aria-label="Undo"
        >
          <Icons.RotateCcw size={14} />
        </button>
        <button
          class="history-btn"
          disabled={!$canRedo}
          onclick={redo}
          title="Redo (Cmd+Shift+Z)"
          aria-label="Redo"
        >
          <Icons.RotateCw size={14} />
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .block-editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    position: relative;
    padding: 24px;
    gap: 4px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    color: var(--muted);
    height: 100%;
    text-align: center;
  }

  .empty-state p {
    font-size: 16px;
  }

  kbd {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 2px 6px;
    font-family: 'Monaco', 'Courier New', monospace;
    font-size: 12px;
    color: var(--text);
  }

  .blocks-container {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .block-wrapper {
    position: relative;
    transition: background-color 0.15s;
  }

  .block-wrapper:hover {
    background-color: rgba(0, 0, 0, 0.02);
  }

  .block-divider {
    height: 1px;
    background-color: var(--border);
    margin: 8px 0;
  }

  .editor-status {
    position: sticky;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 16px;
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-size: 12px;
    color: var(--muted);
  }

  .status-left {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
  }

  .status-right {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .status-indicator {
    display: inline-block;
    font-size: 8px;
    animation: pulse 1s infinite;
  }

  .status-indicator.unsaved {
    color: var(--warning);
  }

  .history-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 4px;
    background-color: var(--bg);
    color: var(--text);
    cursor: pointer;
    transition: all 0.15s;
  }

  .history-btn:hover:not(:disabled) {
    background-color: rgba(59, 130, 246, 0.1);
    border-color: rgba(59, 130, 246, 0.3);
  }

  .history-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .history-btn:active:not(:disabled) {
    background-color: rgba(59, 130, 246, 0.2);
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
</style>
