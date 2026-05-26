<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface CodeSnippet {
    id: string;
    title: string;
    description?: string;
    code: string;
    language: string;
    tags?: string[];
    createdAt: number;
    favorite?: boolean;
  }

  interface $$Props {
    snippets?: CodeSnippet[];
    selectedSnippetId?: string | null;
    onSelectSnippet?: (snippet: CodeSnippet) => void;
    onDeleteSnippet?: (snippetId: string) => void;
    onToggleFavorite?: (snippetId: string) => void;
  }

  let {
    snippets = [],
    selectedSnippetId = null,
    onSelectSnippet = () => {},
    onDeleteSnippet = () => {},
    onToggleFavorite = () => {},
  }: $$Props = $props();

  let searchQuery = $state('');
  let filterLanguage = $state('');

  let filteredSnippets = $derived.by(() => {
    return snippets.filter((snippet) => {
      const matchesSearch =
        !searchQuery ||
        snippet.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        snippet.description?.toLowerCase().includes(searchQuery.toLowerCase()) ||
        snippet.tags?.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase()));

      const matchesLanguage = !filterLanguage || snippet.language === filterLanguage;

      return matchesSearch && matchesLanguage;
    });
  });

  let languages = $derived(Array.from(new Set(snippets.map((s) => s.language))));

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp);
    return date.toLocaleDateString();
  }
</script>

<div class="snippet-library">
  <div class="library-header">
    <h3>Code Snippets</h3>
  </div>

  <div class="library-controls">
    <div class="search-box">
      <Icons.Search size={14} />
      <input
        type="text"
        placeholder="Search snippets..."
        bind:value={searchQuery}
      />
    </div>

    <select bind:value={filterLanguage} class="language-filter">
      <option value="">All languages</option>
      {#each languages as language}
        <option value={language}>{language}</option>
      {/each}
    </select>
  </div>

  {#if filteredSnippets.length === 0}
    <div class="empty-state">
      <Icons.Code size={24} />
      <p>{searchQuery || filterLanguage ? 'No snippets found' : 'No snippets yet'}</p>
    </div>
  {:else}
    <div class="snippets-list">
      {#each filteredSnippets as snippet (snippet.id)}
        <button
          type="button"
          class="snippet-item"
          class:selected={selectedSnippetId === snippet.id}
          onclick={() => onSelectSnippet(snippet)}
        >
          <div class="snippet-head">
            <span class="snippet-title">{snippet.title}</span>
            <span class="snippet-lang">{snippet.language}</span>
          </div>
          {#if snippet.description}
            <p class="snippet-desc">{snippet.description}</p>
          {/if}
          {#if snippet.tags && snippet.tags.length > 0}
            <div class="snippet-tags">
              {#each snippet.tags.slice(0, 2) as tag}
                <span class="tag">{tag}</span>
              {/each}
              {#if snippet.tags.length > 2}
                <span class="tag more">+{snippet.tags.length - 2}</span>
              {/if}
            </div>
          {/if}
          <div class="snippet-footer">
            <span class="snippet-date">{formatDate(snippet.createdAt)}</span>
            <div class="snippet-actions">
              <button
                type="button"
                class="action-btn"
                class:favorited={snippet.favorite}
                onclick={(e) => {
                  e.stopPropagation();
                  onToggleFavorite(snippet.id);
                }}
                title="Toggle favorite"
              >
                <Icons.Star size={12} />
              </button>
              <button
                type="button"
                class="action-btn delete"
                onclick={(e) => {
                  e.stopPropagation();
                  onDeleteSnippet(snippet.id);
                }}
                title="Delete snippet"
              >
                <Icons.Trash2 size={12} />
              </button>
            </div>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .snippet-library {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
  }

  .library-header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .library-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text);
  }

  .library-controls {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: var(--bg);
    border: 1px solid var(--hairline);
    border-radius: 6px;
    color: var(--muted);
  }

  .search-box input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    outline: none;
    font-size: 12px;
  }

  .language-filter {
    padding: 6px 10px;
    background: var(--bg);
    border: 1px solid var(--hairline);
    border-radius: 6px;
    color: var(--text);
    font-size: 12px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 40px 16px;
    color: var(--muted);
    text-align: center;
  }

  .snippets-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .snippet-item {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    padding: 12px;
    border: 1px solid var(--hairline);
    background: transparent;
    border-radius: 8px;
    text-align: left;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .snippet-item:hover {
    background: var(--bg);
    border-color: var(--border);
  }

  .snippet-item.selected {
    background: var(--blue-soft);
    border-color: var(--blue);
  }

  .snippet-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .snippet-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
  }

  .snippet-lang {
    font-size: 10px;
    color: var(--muted);
    background: var(--surface-2);
    padding: 2px 6px;
    border-radius: 3px;
    font-family: var(--ff-mono);
  }

  .snippet-desc {
    margin: 0;
    font-size: 12px;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .snippet-tags {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .tag {
    font-size: 10px;
    color: var(--text-2);
    background: var(--surface-2);
    padding: 2px 6px;
    border-radius: 3px;
  }

  .tag.more {
    color: var(--muted);
  }

  .snippet-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11px;
  }

  .snippet-date {
    color: var(--dim);
  }

  .snippet-actions {
    display: flex;
    gap: 4px;
  }

  .action-btn {
    width: 20px;
    height: 20px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .action-btn:hover {
    color: var(--text);
  }

  .action-btn.favorited {
    color: var(--orange);
  }

  .action-btn.delete:hover {
    color: var(--danger);
  }
</style>
