<script lang="ts">
  import type { Block } from '$lib/blocks/types';
  import { onMount } from 'svelte';

  interface Props {
    block: Block;
    isSelected?: boolean;
    onUpdate?: (content: string) => void;
    onKeyDown?: (e: KeyboardEvent) => void;
  }

  let { block, isSelected = false, onUpdate, onKeyDown }: Props = $props();

  let contentElement: HTMLDivElement;

  onMount(() => {
    if (isSelected && contentElement) {
      contentElement.focus();
    }
  });

  function handleInput(e: Event) {
    const content = (e.target as HTMLDivElement).textContent || '';
    onUpdate?.(content);
  }
</script>

<div class="block-quote" class:selected={isSelected}>
  <div class="quote-bar"></div>
  <div
    bind:this={contentElement}
    role="textbox"
    tabindex={isSelected ? 0 : -1}
    contenteditable
    class="quote-content"
    oninput={handleInput}
    onkeydown={onKeyDown}
    data-placeholder="Quote..."
  >
    {block.content}
  </div>
</div>

<style>
  .block-quote {
    display: flex;
    gap: 8px;
    padding: 8px 12px;
    background-color: var(--bg-secondary);
    border-radius: 4px;
    margin: 4px 0;
  }

  .quote-bar {
    flex-shrink: 0;
    width: 3px;
    background-color: var(--muted);
    border-radius: 2px;
  }

  .quote-content {
    flex: 1;
    outline: none;
    font-style: italic;
    color: var(--muted);
    word-wrap: break-word;
    white-space: pre-wrap;
    overflow-wrap: break-word;
  }

  .quote-content:empty::before {
    content: attr(data-placeholder);
    pointer-events: none;
  }

  .block-quote.selected {
    background-color: rgba(59, 130, 246, 0.1);
  }
</style>
