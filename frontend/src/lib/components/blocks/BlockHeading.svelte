<script lang="ts">
  import type { Block } from '$lib/blocks/types';
  import { onMount } from 'svelte';

  interface Props {
    block: Block;
    level: 1 | 2 | 3;
    isSelected?: boolean;
    onUpdate?: (content: string) => void;
    onKeyDown?: (e: KeyboardEvent) => void;
  }

  let { block, level, isSelected = false, onUpdate, onKeyDown }: Props = $props();

  let contentElement: HTMLHeadingElement;

  onMount(() => {
    if (isSelected && contentElement) {
      contentElement.focus();
    }
  });

  function handleInput(e: Event) {
    const content = (e.target as HTMLHeadingElement).textContent || '';
    onUpdate?.(content);
  }

  const Tag = `h${level}` as 'h1' | 'h2' | 'h3';
</script>

<svelte:element
  this={Tag}
  bind:this={contentElement}
  role="textbox"
  tabindex={isSelected ? 0 : -1}
  contenteditable
  class="block-heading"
  class:selected={isSelected}
  class:level-{level}
  onInput={handleInput}
  onKeyDown={onKeyDown}
  data-placeholder={`Heading ${level}...`}
>
  {block.content}
</svelte:element>

<style>
  .block-heading {
    outline: none;
    font-weight: 600;
    line-height: 1.4;
    word-wrap: break-word;
    white-space: pre-wrap;
    overflow-wrap: break-word;
    margin: 0.5em 0;
  }

  .block-heading.level-1 {
    font-size: 32px;
  }

  .block-heading.level-2 {
    font-size: 24px;
  }

  .block-heading.level-3 {
    font-size: 20px;
  }

  .block-heading:empty::before {
    content: attr(data-placeholder);
    color: var(--muted);
    pointer-events: none;
  }

  .block-heading.selected {
    background-color: rgba(59, 130, 246, 0.05);
    border-radius: 4px;
    padding: 0 4px;
  }
</style>
