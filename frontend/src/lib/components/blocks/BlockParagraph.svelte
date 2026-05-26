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

<div
  bind:this={contentElement}
  role="textbox"
  tabindex={isSelected ? 0 : -1}
  contenteditable
  class="block-paragraph"
  class:selected={isSelected}
  onInput={handleInput}
  onKeyDown={onKeyDown}
  data-placeholder="Type or use / for commands..."
>
  {block.content}
</div>

<style>
  .block-paragraph {
    min-height: 1.5em;
    line-height: 1.6;
    font-size: 16px;
    color: var(--text);
    outline: none;
    word-wrap: break-word;
    white-space: pre-wrap;
    overflow-wrap: break-word;
  }

  .block-paragraph:empty::before {
    content: attr(data-placeholder);
    color: var(--muted);
    pointer-events: none;
  }

  .block-paragraph.selected {
    background-color: rgba(59, 130, 246, 0.05);
    border-radius: 4px;
    padding: 0 4px;
  }
</style>
