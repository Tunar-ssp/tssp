<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { api, type Note } from '$lib/api';
  import NotionBlock from './NotionBlock.svelte';
  import { type Block, type BlockType, createBlock } from './NotionBlockTypes';

  interface Props {
    note?: Note;
    onSave?: (noteData: { note: Note; blocks: Block[] }) => void;
    onDelete?: (id: string) => void;
  }

  let { note, onSave, onDelete }: Props = $props();

  let title = $state('');
  let blocks = $state<Block[]>([createBlock()]);
  let isSaving = $state(false);
  let searchQuery = $state('');
  let selectedBlockId: string | null = null;

  $effect(() => {
    if (note) {
      title = note.title;
      blocks = [createBlock()];
    }
  });

  const filteredBlocks = $derived(
    searchQuery
      ? blocks.filter(b => b.content.toLowerCase().includes(searchQuery.toLowerCase()))
      : blocks
  );

  async function saveNote() {
    if (!onSave || !note) return;
    isSaving = true;
    try {
      const noteData = {
        note: {
          ...note,
          title,
          body: blocks.map(b => b.content).join('\n'),
        },
        blocks,
      };
      onSave(noteData);
    } finally {
      isSaving = false;
    }
  }

  function handleBlockUpdate(updatedBlock: Block) {
    const index = blocks.findIndex(b => b.id === updatedBlock.id);
    if (index > -1) {
      blocks[index] = updatedBlock;
      blocks = blocks;
    }
  }

  function handleBlockDelete(blockId: string) {
    blocks = blocks.filter(b => b.id !== blockId);
    if (blocks.length === 0) {
      blocks = [createBlock()];
    }
  }

  function handleCreateBlock(afterId: string, type: BlockType) {
    const newBlock = createBlock(type);
    const index = blocks.findIndex(b => b.id === afterId);
    if (index > -1) {
      blocks.splice(index + 1, 0, newBlock);
      blocks = blocks;
    }
  }

  function handleToggleCollapse(blockId: string) {
    const block = blocks.find(b => b.id === blockId);
    if (block) {
      block.collapsed = !block.collapsed;
      blocks = blocks;
    }
  }
</script>

<div class="notes-editor">
  <div class="editor-header">
    <input
      type="text"
      class="note-title"
      placeholder="Note title..."
      value={title}
      onchange={(e) => title = (e.target as HTMLInputElement).value}
    />
    <div class="editor-actions">
      <input
        type="text"
        class="search-input"
        placeholder="Search blocks..."
        value={searchQuery}
        onchange={(e) => searchQuery = (e.target as HTMLInputElement).value}
      />
      <button class="save-btn" disabled={isSaving} onclick={saveNote}>
        {isSaving ? 'Saving...' : 'Save'}
      </button>
      <button class="delete-btn" onclick={() => note && onDelete?.(note.id)} disabled={!note}>
        <Icons.Trash2 size={16} />
      </button>
    </div>
  </div>

  <div class="blocks-container">
    {#each filteredBlocks as block (block.id)}
      <NotionBlock
        {block}
        isSelected={selectedBlockId === block.id}
        onUpdate={handleBlockUpdate}
        onDelete={handleBlockDelete}
        onCreateBlock={handleCreateBlock}
        onToggleCollapse={handleToggleCollapse}
      />
    {/each}
  </div>

  <div class="editor-footer">
    <small>
      {blocks.length} blocks · Last saved {note && note.updated_at ? new Date(note.updated_at).toLocaleDateString() : 'never'}
    </small>
  </div>
</div>

<style>
  .notes-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg);
  }

  .editor-header {
    padding: 20px;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .note-title {
    font-size: 32px;
    font-weight: 700;
    border: none;
    background: transparent;
    color: var(--text);
    outline: none;
    font-family: var(--ff-display);
  }

  .editor-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .search-input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    color: var(--text);
    font-size: 13px;
    outline: none;
  }

  .search-input:focus {
    border-color: var(--blue);
  }

  .save-btn,
  .delete-btn {
    padding: 8px 14px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface-2);
    color: var(--text);
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    transition: all 150ms;
  }

  .save-btn:hover:not(:disabled) {
    border-color: var(--blue);
    background: var(--blue);
    color: white;
  }

  .save-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .delete-btn {
    color: var(--danger);
  }

  .delete-btn:hover {
    background: rgba(255, 107, 107, 0.1);
  }

  .blocks-container {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .editor-footer {
    padding: 12px 20px;
    border-top: 1px solid var(--border);
    color: var(--muted);
    font-size: 12px;
  }
</style>
