<script lang="ts">
  interface $$Props {
    language?: string;
    lines?: number;
    columns?: number;
    cursorLine?: number;
    cursorColumn?: number;
    wordCount?: number;
    charCount?: number;
    isDirty?: boolean;
  }

  let {
    language = 'text',
    lines = 0,
    columns = 0,
    cursorLine = 1,
    cursorColumn = 1,
    wordCount = 0,
    charCount = 0,
    isDirty = false,
  } = $props<$$Props>();
</script>

<div class="status-bar">
  <div class="status-left">
    <span class="status-item language">{language.toUpperCase()}</span>
    <span class="status-divider">•</span>
    <span class="status-item">Ln {cursorLine}, Col {cursorColumn}</span>
  </div>

  <div class="status-center">
    {#if isDirty}
      <span class="status-item unsaved">● Unsaved</span>
    {:else}
      <span class="status-item saved">✓ Saved</span>
    {/if}
  </div>

  <div class="status-right">
    <span class="status-item">{lines} lines</span>
    <span class="status-divider">•</span>
    <span class="status-item">{wordCount} words</span>
    <span class="status-divider">•</span>
    <span class="status-item">{charCount} chars</span>
  </div>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-2) var(--s-4);
    border-top: 1px solid var(--border);
    background: var(--surface);
    font-size: var(--fs-11);
    color: var(--text-2);
    flex-shrink: 0;
    height: 28px;
  }

  .status-left,
  .status-center,
  .status-right {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .status-item {
    white-space: nowrap;
  }

  .status-divider {
    color: var(--muted);
  }

  .status-item.language {
    font-weight: 600;
    color: var(--blue);
  }

  .status-item.unsaved {
    color: var(--orange);
    font-weight: 500;
  }

  .status-item.saved {
    color: var(--green);
    font-weight: 500;
  }
</style>
