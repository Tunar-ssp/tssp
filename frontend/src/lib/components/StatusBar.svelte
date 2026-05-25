<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    fileName?: string;
    language?: string;
    lineNumber?: number;
    columnNumber?: number;
    characterCount?: number;
    wordCount?: number;
    isSaved?: boolean;
    class?: string;
  }

  let {
    fileName = 'Untitled',
    language = 'Plain Text',
    lineNumber = 1,
    columnNumber = 1,
    characterCount = 0,
    wordCount = 0,
    isSaved = true,
    class: className,
  } = $props<$$Props>();
</script>

<div class="status-bar {className || ''}">
  <div class="status-left">
    {#if !isSaved}
      <span class="status-item unsaved">
        <Icons.Circle size={8} />
        Unsaved
      </span>
    {:else}
      <span class="status-item">
        <Icons.Check size={12} />
        Saved
      </span>
    {/if}
    <span class="status-divider">·</span>
    <span class="status-item">{language}</span>
  </div>

  <div class="status-center">
    <span class="status-item" title="Line and column">
      Ln {lineNumber}, Col {columnNumber}
    </span>
  </div>

  <div class="status-right">
    <span class="status-item" title="Character count">
      {characterCount} {characterCount === 1 ? 'character' : 'characters'}
    </span>
    <span class="status-divider">·</span>
    <span class="status-item" title="Word count">
      {wordCount} {wordCount === 1 ? 'word' : 'words'}
    </span>
  </div>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 24px;
    padding: 0 var(--s-3);
    background: var(--surface-2);
    border-top: 1px solid var(--border);
    font-size: var(--fs-11);
    color: var(--muted);
    flex-shrink: 0;
  }

  .status-left,
  .status-center,
  .status-right {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .status-left {
    flex: 1;
  }

  .status-right {
    flex: 1;
    justify-content: flex-end;
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }

  .status-item.unsaved {
    color: var(--orange);
  }

  .status-divider {
    color: var(--border);
    opacity: 0.5;
  }
</style>
