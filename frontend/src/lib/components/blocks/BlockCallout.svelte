<script lang="ts">
  import type { CalloutBlock, CalloutColor } from '$lib/blocks/types';
  import { onMount } from 'svelte';

  interface Props {
    block: CalloutBlock;
    isSelected?: boolean;
    onUpdate?: (content: string) => void;
    onColorChange?: (color: CalloutColor) => void;
    onKeyDown?: (e: KeyboardEvent) => void;
  }

  let { block, isSelected = false, onUpdate, onColorChange, onKeyDown }: Props = $props();

  let contentElement: HTMLDivElement;

  const colors: Record<CalloutColor, { bg: string; icon: string; label: string }> = {
    blue: { bg: 'rgba(59, 130, 246, 0.1)', icon: '💡', label: 'Blue' },
    red: { bg: 'rgba(239, 68, 68, 0.1)', icon: '🔴', label: 'Red' },
    yellow: { bg: 'rgba(251, 191, 36, 0.1)', icon: '⚠️', label: 'Yellow' },
    green: { bg: 'rgba(34, 197, 94, 0.1)', icon: '✅', label: 'Green' },
    purple: { bg: 'rgba(168, 85, 247, 0.1)', icon: '💜', label: 'Purple' },
    gray: { bg: 'rgba(107, 114, 128, 0.1)', icon: '⚙️', label: 'Gray' },
  };

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

<div class="block-callout" style:background-color={colors[block.color].bg} class:selected={isSelected}>
  <div class="callout-icon">{colors[block.color].icon}</div>
  <div
    bind:this={contentElement}
    role="textbox"
    tabindex={isSelected ? 0 : -1}
    contenteditable
    class="callout-content"
    oninput={handleInput}
    onkeydown={onKeyDown}
    data-placeholder="Callout text..."
  >
    {block.content}
  </div>
</div>

<style>
  .block-callout {
    display: flex;
    gap: 12px;
    padding: 12px 16px;
    border-radius: 6px;
    margin: 4px 0;
    transition: opacity 0.2s;
  }

  .block-callout.selected {
    opacity: 0.9;
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
  }

  .callout-icon {
    flex-shrink: 0;
    font-size: 20px;
  }

  .callout-content {
    flex: 1;
    outline: none;
    font-size: 15px;
    word-wrap: break-word;
    white-space: pre-wrap;
    overflow-wrap: break-word;
  }

  .callout-content:empty::before {
    content: attr(data-placeholder);
    color: var(--muted);
    pointer-events: none;
  }
</style>
