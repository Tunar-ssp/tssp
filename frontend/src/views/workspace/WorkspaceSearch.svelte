<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    searchQuery: string;
    onSearchChange?: (query: string) => void;
    onSearch?: () => void;
  }

  let { searchQuery, onSearchChange, onSearch }: $$Props = $props();
</script>

<div class="workspace-search">
  <label class="search-field">
    <Icons.Search size={16} />
    <input
      type="text"
      value={searchQuery}
      oninput={(e) => onSearchChange?.((e.target as HTMLInputElement).value)}
      placeholder="Search workspaces..."
    />
  </label>
  <button type="button" class="search-button" onclick={onSearch} title="Search">
    <Icons.ArrowRight size={16} />
  </button>
</div>

<style>
  .workspace-search {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px;
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
  }

  .search-field {
    flex: 1;
    height: 40px;
    padding: 0 12px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--surface);
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

  .search-button {
    height: 40px;
    width: 40px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .search-button:hover {
    background: var(--surface-3);
    color: var(--text);
  }
</style>
