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

  const levelClass = `level-${level}`;
</script>

{#if level === 1}
  <h1
    bind:this={contentElement}
    role="textbox"
    tabindex={isSelected ? 0 : -1}
    contenteditable
    class="block-heading level-1"
    class:selected={isSelected}
    oninput={handleInput}
    onkeydown={onKeyDown}
    data-placeholder="Heading 1..."
  >
    {block.content}
  </h1>
{:else if level === 2}
  <h2
    bind:this={contentElement}
    role="textbox"
    tabindex={isSelected ? 0 : -1}
    contenteditable
    class="block-heading level-2"
    class:selected={isSelected}
    oninput={handleInput}
    onkeydown={onKeyDown}
    data-placeholder="Heading 2..."
  >
    {block.content}
  </h2>
{:else}
  <h3
    bind:this={contentElement}
    role="textbox"
    tabindex={isSelected ? 0 : -1}
    contenteditable
    class="block-heading level-3"
    class:selected={isSelected}
    oninput={handleInput}
    onkeydown={onKeyDown}
    data-placeholder="Heading 3..."
  >
    {block.content}
  </h3>
{/if}

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
