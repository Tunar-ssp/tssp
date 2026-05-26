<script lang="ts">
  import type { CodeBlock } from '$lib/blocks/types';
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';

  interface Props {
    block: CodeBlock;
    isSelected?: boolean;
    onUpdate?: (content: string) => void;
    onLanguageChange?: (language: string) => void;
    onKeyDown?: (e: KeyboardEvent) => void;
  }

  let { block, isSelected = false, onUpdate, onLanguageChange, onKeyDown }: Props = $props();

  let contentElement = $state<HTMLDivElement>();
  let language = $state('');

  $effect(() => {
    language = block.language || 'javascript';
  });

  const languages = ['javascript', 'typescript', 'python', 'rust', 'sql', 'html', 'css', 'json'];

  onMount(() => {
    if (isSelected && contentElement) {
      contentElement.focus();
    }
  });

  function handleInput(e: Event) {
    const content = (e.target as HTMLDivElement).textContent || '';
    onUpdate?.(content);
  }

  function handleLanguageChange(e: Event) {
    const lang = (e.target as HTMLSelectElement).value;
    language = lang;
    onLanguageChange?.(lang);
  }
</script>

<div class="block-code" class:selected={isSelected}>
  <div class="code-header">
    <select value={language} onchange={handleLanguageChange} class="language-select">
      {#each languages as lang}
        <option value={lang}>{lang}</option>
      {/each}
    </select>
    <Icons.Copy size={16} class="copy-icon" />
  </div>

  <div
    bind:this={contentElement}
    role="textbox"
    aria-multiline="true"
    tabindex={isSelected ? 0 : -1}
    contenteditable
    class="code-content"
    oninput={handleInput}
    onkeydown={onKeyDown}
    data-placeholder="Enter code..."
    spellcheck="false"
  >
    {block.content}
  </div>
</div>

<style>
  .block-code {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
    margin: 4px 0;
    font-family: 'Monaco', 'Courier New', monospace;
    font-size: 13px;
  }

  .block-code.selected {
    border-color: rgba(59, 130, 246, 0.5);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
  }

  .code-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    background-color: rgba(0, 0, 0, 0.02);
  }

  .language-select {
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: 3px;
    font-size: 12px;
    background-color: var(--bg);
    color: var(--text);
    cursor: pointer;
  }

  :global(.copy-icon) {
    cursor: pointer;
    color: var(--muted);
    transition: color 0.2s;
  }

  :global(.copy-icon:hover) {
    color: var(--text);
  }

  .code-content {
    padding: 12px;
    outline: none;
    color: var(--text);
    line-height: 1.5;
    white-space: pre;
    overflow-x: auto;
    word-wrap: break-word;
    overflow-wrap: break-word;
    min-height: 60px;
    max-height: 400px;
    overflow-y: auto;
  }

  .code-content:empty::before {
    content: attr(data-placeholder);
    color: var(--muted);
    pointer-events: none;
  }
</style>
