<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    searchQuery: string;
    activeTag: string | null;
    onSearchChange?: (query: string) => void;
    onTagClear?: () => void;
  }

  let {
    searchQuery,
    activeTag,
    onSearchChange,
    onTagClear,
  }: $$Props = $props();
</script>

<div class="notes-search">
  <label class="search-field">
    <Icons.Search size={16} />
    <input
      type="text"
      value={searchQuery}
      oninput={(e) => onSearchChange?.((e.target as HTMLInputElement).value)}
      placeholder="Search notes..."
    />
  </label>
  {#if activeTag}
    <button type="button" class="active-tag-chip" onclick={onTagClear}>
      {activeTag}
      <Icons.X size={12} />
    </button>
  {/if}
</div>

<style>
  .notes-search {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px;
    border-bottom: 1px solid var(--border);
  }

  .search-field {
    flex: 1;
    height: 40px;
    padding: 0 12px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--muted);
  }

  .search-field input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font-size: 14px;
  }

  .search-field input::placeholder {
    color: var(--muted);
  }

  .active-tag-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: rgba(110, 168, 255, 0.14);
    color: var(--text-2);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .active-tag-chip:hover {
    border-color: rgba(110, 168, 255, 0.28);
    background: rgba(110, 168, 255, 0.24);
    color: var(--text);
  }
</style>
