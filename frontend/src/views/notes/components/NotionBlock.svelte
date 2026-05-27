<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { createBlock, type Block, type BlockType, BLOCK_LABELS, SLASH_COMMANDS } from './NotionBlockTypes';
  import NotionBlock from './NotionBlock.svelte';

  interface Props {
    block?: Block;
    isSelected?: boolean;
    onUpdate?: (block: Block) => void;
    onDelete?: (id: string) => void;
    onCreateBlock?: (afterId: string, type: BlockType) => void;
    onToggleCollapse?: (id: string) => void;
  }

  let {
    block = createBlock(),
    isSelected = false,
    onUpdate,
    onDelete,
    onCreateBlock,
    onToggleCollapse,
  }: Props = $props();

  let showSlashMenu = $state(false);
  let slashQuery = $state('');
  let contentElement: HTMLDivElement;

  const blockIcon = $derived(getBlockIcon(block.type));
  const blockClass = $derived(getBlockClass(block.type));

  function handleInput(e: Event) {
    const target = e.target as HTMLDivElement;
    const newContent = target.innerText;
    onUpdate?.({ ...block, content: newContent });
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === '/') {
      showSlashMenu = true;
      slashQuery = '';
    } else if (showSlashMenu && e.key === 'Escape') {
      showSlashMenu = false;
    } else if (showSlashMenu && (e.key === 'Enter' || e.key === 'Tab')) {
      e.preventDefault();
      selectCommand(slashQuery);
    } else if (e.key === 'Enter' && e.ctrlKey) {
      e.preventDefault();
      onCreateBlock?.(block.id, 'paragraph');
    } else if (e.key === 'Backspace' && !block.content) {
      e.preventDefault();
      onDelete?.(block.id);
    }
  }

  function selectCommand(command: string) {
    const blockType = SLASH_COMMANDS[command] || 'paragraph';
    onUpdate?.({ ...block, type: blockType, content: '' });
    showSlashMenu = false;
    slashQuery = '';
  }

  function getBlockIcon(type: BlockType) {
    return {
      'paragraph': Icons.Type,
      'heading-1': Icons.Heading1,
      'heading-2': Icons.Heading2,
      'heading-3': Icons.Heading3,
      'bulleted-list': Icons.List,
      'numbered-list': Icons.ListOrdered,
      'toggle': Icons.ChevronRight,
      'quote': Icons.Quote,
      'code': Icons.Code,
      'divider': Icons.Minus,
      'callout': Icons.AlertCircle,
      'table': Icons.Grid3x3,
    }[type] || Icons.Type;
  }

  function getBlockClass(type: BlockType): string {
    return {
      'paragraph': 'block-paragraph',
      'heading-1': 'block-heading-1',
      'heading-2': 'block-heading-2',
      'heading-3': 'block-heading-3',
      'bulleted-list': 'block-list',
      'numbered-list': 'block-list',
      'toggle': 'block-toggle',
      'quote': 'block-quote',
      'code': 'block-code',
      'divider': 'block-divider',
      'callout': 'block-callout',
      'table': 'block-table',
    }[type] || 'block-paragraph';
  }
</script>

<div class="notion-block" class:selected={isSelected}>
  <div class="block-handle">
    <svelte:component this={blockIcon} {...{ size: 16 }} />
  </div>

  <div class="block-content">
    {#if block.type === 'divider'}
      <div class="divider-line"></div>
    {:else if block.type === 'toggle'}
      <button class="toggle-btn" onclick={() => onToggleCollapse?.(block.id)}>
        <span class:collapsed={block.collapsed}>
          <Icons.ChevronRight size={16} />
        </span>
      </button>
      <div
        contenteditable="true"
        class={`block-editor ${blockClass}`}
        oninput={handleInput}
        onkeydown={handleKeyDown}
      >
        {block.content}
      </div>
    {:else}
      <div
        contenteditable="true"
        class={`block-editor ${blockClass}`}
        oninput={handleInput}
        onkeydown={handleKeyDown}
        data-placeholder={`Type '/' for commands`}
      >
        {block.content}
      </div>
    {/if}
  </div>

  <div class="block-actions">
    <button class="action-btn" onclick={() => onDelete?.(block.id)}>
      <Icons.Trash2 size={16} />
    </button>
  </div>

  {#if showSlashMenu}
    <div class="slash-menu">
      {#each Object.entries(SLASH_COMMANDS) as [cmd, type]}
        <button class="menu-item" onclick={() => selectCommand(cmd)}>
          <span class="cmd-label">/{cmd}</span>
          <span class="cmd-desc">{BLOCK_LABELS[type]}</span>
        </button>
      {/each}
    </div>
  {/if}

  {#if block.children && block.children.length > 0 && !block.collapsed}
    <div class="block-children">
      {#each block.children as child (child.id)}
        <NotionBlock
          block={child}
          isSelected={isSelected}
          onUpdate={onUpdate}
          onDelete={onDelete}
          onCreateBlock={onCreateBlock}
          onToggleCollapse={onToggleCollapse}
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .notion-block {
    display: flex;
    gap: 8px;
    padding: 4px;
    border-radius: 8px;
    transition: background 150ms;
  }

  .notion-block:hover {
    background: rgba(255, 255, 255, 0.02);
  }

  .notion-block.selected {
    background: rgba(110, 168, 255, 0.1);
    border: 1px solid rgba(110, 168, 255, 0.2);
  }

  .block-handle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    color: var(--muted);
    cursor: grab;
  }

  .block-handle:active {
    cursor: grabbing;
  }

  .block-content {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 24px;
  }

  .block-editor {
    flex: 1;
    outline: none;
    border: none;
    background: transparent;
    color: var(--text);
    font-family: inherit;
    font-size: inherit;
    line-height: 1.5;
    word-break: break-word;
    white-space: pre-wrap;
  }

  .block-editor[contenteditable]:empty::before {
    content: attr(placeholder);
    color: var(--muted);
    pointer-events: none;
  }

  /* Block type styles */
  .block-paragraph {
    font-size: 14px;
  }

  .block-heading-1 {
    font-size: 28px;
    font-weight: 700;
    margin: 8px 0;
  }

  .block-heading-2 {
    font-size: 22px;
    font-weight: 700;
    margin: 6px 0;
  }

  .block-heading-3 {
    font-size: 18px;
    font-weight: 600;
    margin: 4px 0;
  }

  .block-list {
    font-size: 14px;
    margin-left: 16px;
  }

  .block-quote {
    font-size: 14px;
    font-style: italic;
    color: var(--text-2);
    border-left: 3px solid var(--blue);
    padding-left: 12px;
    margin-left: 8px;
  }

  .block-code {
    font-family: var(--ff-mono);
    font-size: 12px;
    background: rgba(0, 0, 0, 0.2);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .block-callout {
    background: rgba(110, 168, 255, 0.1);
    border-left: 3px solid var(--blue);
    padding: 8px 12px;
    border-radius: 4px;
    margin: 4px 0;
  }

  .divider-line {
    height: 1px;
    background: var(--border);
    width: 100%;
    margin: 8px 0;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    padding: 0;
    transition: transform 150ms;
  }

  .toggle-btn span.collapsed {
    display: inline-flex;
    transform: rotate(-90deg);
  }

  .block-actions {
    display: flex;
    gap: 4px;
    opacity: 0;
    transition: opacity 150ms;
  }

  .notion-block:hover .block-actions {
    opacity: 1;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border: none;
    background: transparent;
    color: var(--danger);
    cursor: pointer;
    border-radius: 4px;
    transition: background 150ms;
  }

  .action-btn:hover {
    background: rgba(255, 107, 107, 0.1);
  }

  .slash-menu {
    position: absolute;
    top: 100%;
    left: 0;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    margin-top: 4px;
    max-height: 300px;
    overflow-y: auto;
    z-index: 1000;
    min-width: 200px;
  }

  .menu-item {
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
    font-size: 12px;
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .menu-item:hover {
    background: var(--surface-2);
  }

  .cmd-label {
    font-family: var(--ff-mono);
    font-weight: 500;
    color: var(--blue);
  }

  .cmd-desc {
    color: var(--text-2);
    flex: 1;
  }

  .block-children {
    margin-left: 16px;
    margin-top: 2px;
  }
</style>
