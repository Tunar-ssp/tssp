<script lang="ts">
  import type { Block, ChecklistBlock } from '$lib/blocks/types';
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';

  interface Props {
    block: Block | ChecklistBlock;
    type: 'bulleted' | 'numbered' | 'checklist';
    isSelected?: boolean;
    onUpdate?: (content: string) => void;
    onToggleChecked?: () => void;
    onKeyDown?: (e: KeyboardEvent) => void;
  }

  let { block, type, isSelected = false, onUpdate, onToggleChecked, onKeyDown }: Props = $props();

  let contentElement: HTMLDivElement;
  let isChecked = $state(type === 'checklist' ? (block as ChecklistBlock).checked || false : false);

  onMount(() => {
    if (isSelected && contentElement) {
      contentElement.focus();
    }
  });

  function handleInput(e: Event) {
    const content = (e.target as HTMLDivElement).textContent || '';
    onUpdate?.(content);
  }

  function handleCheckboxChange() {
    isChecked = !isChecked;
    onToggleChecked?.();
  }
</script>

<div class="block-list-item" class:selected={isSelected} class:checked={isChecked && type === 'checklist'}>
  {#if type === 'bulleted'}
    <span class="bullet">•</span>
  {:else if type === 'numbered'}
    <span class="number">1.</span>
  {:else if type === 'checklist'}
    <button
      class="checkbox"
      onclick={handleCheckboxChange}
      aria-label="Toggle checklist item"
    >
      {#if isChecked}
        <Icons.CheckSquare size={18} />
      {:else}
        <Icons.Square size={18} />
      {/if}
    </button>
  {/if}

  <div
    bind:this={contentElement}
    role="textbox"
    tabindex={isSelected ? 0 : -1}
    contenteditable
    class="list-content"
    class:strikethrough={isChecked && type === 'checklist'}
    oninput={handleInput}
    onkeydown={onKeyDown}
    data-placeholder="List item..."
  >
    {block.content}
  </div>
</div>

<style>
  .block-list-item {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    line-height: 1.6;
  }

  .bullet,
  .number {
    flex-shrink: 0;
    width: 24px;
    text-align: center;
    color: var(--muted);
    font-size: 14px;
    margin-top: 3px;
  }

  .checkbox {
    flex-shrink: 0;
    padding: 0;
    margin: 0;
    margin-top: 2px;
    border: none;
    background: none;
    cursor: pointer;
    color: var(--muted);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.2s;
  }

  .checkbox:hover {
    color: var(--text);
  }

  .list-content {
    flex: 1;
    outline: none;
    min-height: 1.5em;
    word-wrap: break-word;
    white-space: pre-wrap;
    overflow-wrap: break-word;
  }

  .list-content:empty::before {
    content: attr(data-placeholder);
    color: var(--muted);
    pointer-events: none;
  }

  .list-content.strikethrough {
    text-decoration: line-through;
    color: var(--muted);
  }

  .block-list-item.selected .list-content {
    background-color: rgba(59, 130, 246, 0.05);
    border-radius: 4px;
    padding: 0 4px;
  }
</style>
