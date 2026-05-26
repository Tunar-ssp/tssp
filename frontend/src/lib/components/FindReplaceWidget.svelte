<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    isOpen?: boolean;
    onClose?: () => void;
    onFind?: (query: string, options: FindOptions) => void;
    onReplace?: (query: string, replacement: string, options: FindOptions) => void;
    onReplaceAll?: (query: string, replacement: string, options: FindOptions) => void;
    matchCount?: number;
    currentMatchIndex?: number;
  }

  interface FindOptions {
    matchCase: boolean;
    wholeWord: boolean;
    regex: boolean;
  }

  let {
    isOpen = false,
    onClose = () => {},
    onFind = () => {},
    onReplace = () => {},
    onReplaceAll = () => {},
    matchCount = 0,
    currentMatchIndex = 0,
  }: $$Props = $props();

  let searchQuery = $state('');
  let replaceQuery = $state('');
  let matchCase = $state(false);
  let wholeWord = $state(false);
  let regex = $state(false);
  let showReplace = $state(false);
  let findInput = $state<HTMLInputElement | null>(null);
  let replaceInput = $state<HTMLInputElement | null>(null);

  function getSearchOptions(): FindOptions {
    return { matchCase, wholeWord, regex };
  }

  function handleSearch() {
    if (searchQuery) {
      onFind(searchQuery, getSearchOptions());
    }
  }

  function handleReplace() {
    if (searchQuery) {
      onReplace(searchQuery, replaceQuery, getSearchOptions());
    }
  }

  function handleReplaceAll() {
    if (searchQuery) {
      onReplaceAll(searchQuery, replaceQuery, getSearchOptions());
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      if (e.shiftKey) {
        // Shift+Enter to replace
        handleReplace();
      } else {
        // Enter to find next
        handleSearch();
      }
    } else if (e.key === 'Escape') {
      onClose();
    }
  }

  function handleReplaceKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      if (e.ctrlKey || e.metaKey) {
        handleReplaceAll();
      } else {
        handleReplace();
      }
    }
  }

  $effect(() => {
    if (isOpen && findInput) {
      findInput.focus();
    }
  });
</script>

{#if isOpen}
  <div class="find-replace-widget">
    <!-- Find Section -->
    <div class="find-section">
      <div class="find-input-group">
        <Icons.Search size={16} />
        <input
          bind:this={findInput}
          type="text"
          class="find-input"
          placeholder="Find..."
          bind:value={searchQuery}
          onkeydown={handleKeydown}
          aria-label="Find in file"
        />
        {#if matchCount > 0}
          <span class="match-counter">
            {currentMatchIndex + 1}/{matchCount}
          </span>
        {/if}
      </div>

      <div class="find-options">
        <button
          type="button"
          class="find-option"
          class:active={matchCase}
          onclick={() => matchCase = !matchCase}
          title="Match case (Alt+C)"
          aria-label="Toggle case sensitivity"
        >
          <Icons.CaseSensitive size={14} />
        </button>
        <button
          type="button"
          class="find-option"
          class:active={wholeWord}
          onclick={() => wholeWord = !wholeWord}
          title="Match whole word (Alt+W)"
          aria-label="Toggle whole word"
        >
          <Icons.Type size={14} />
        </button>
        <button
          type="button"
          class="find-option"
          class:active={regex}
          onclick={() => regex = !regex}
          title="Use regular expression (Alt+R)"
          aria-label="Toggle regex"
        >
          <span class="regex-icon">.*</span>
        </button>
        <button
          type="button"
          class="find-toggle-replace"
          class:expand-open={showReplace}
          onclick={() => showReplace = !showReplace}
          title="Toggle replace (Ctrl+H)"
          aria-label="Toggle replace"
        >
          <Icons.ChevronRight size={16} />
        </button>
      </div>
    </div>

    <!-- Replace Section -->
    {#if showReplace}
      <div class="replace-section">
        <div class="replace-input-group">
          <Icons.Replace size={16} />
          <input
            bind:this={replaceInput}
            type="text"
            class="replace-input"
            placeholder="Replace with..."
            bind:value={replaceQuery}
            onkeydown={handleReplaceKeydown}
            aria-label="Replace with"
          />
        </div>

        <div class="replace-actions">
          <button
            type="button"
            class="replace-btn"
            onclick={handleReplace}
            title="Replace (Shift+Enter)"
            aria-label="Replace current match"
          >
            <Icons.CheckCircle size={14} />
            Replace
          </button>
          <button
            type="button"
            class="replace-btn replace-all"
            onclick={handleReplaceAll}
            title="Replace all (Ctrl+Alt+Enter)"
            aria-label="Replace all matches"
          >
            <Icons.CheckSquare size={14} />
            Replace All
          </button>
        </div>
      </div>
    {/if}

    <!-- Close Button -->
    <button
      type="button"
      class="find-close"
      onclick={onClose}
      title="Close (Escape)"
      aria-label="Close find/replace"
    >
      <Icons.X size={16} />
    </button>
  </div>
{/if}

<style>
  .find-replace-widget {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    flex-shrink: 0;
  }

  .find-section,
  .replace-section {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .find-input-group,
  .replace-input-group {
    flex: 1;
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--bg);
    color: var(--muted);
  }

  .find-input,
  .replace-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-12);
    outline: none;
  }

  .find-input::placeholder,
  .replace-input::placeholder {
    color: var(--muted);
  }

  .match-counter {
    font-size: var(--fs-11);
    color: var(--text-2);
    padding: 0 var(--s-2);
    border-left: 1px solid var(--border);
  }

  .find-options {
    display: flex;
    gap: var(--s-1);
    align-items: center;
  }

  .find-option {
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--r-1);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .find-option:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .find-option.active {
    background: var(--blue-subtle);
    border-color: var(--blue);
    color: var(--blue);
  }

  .regex-icon {
    font-family: var(--ff-mono);
    font-size: 10px;
    font-weight: bold;
  }

  .find-toggle-replace {
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--r-1);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .find-toggle-replace:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .find-toggle-replace :global(svg) {
    transition: transform var(--duration-quick) var(--ease-smooth);
  }

  .find-toggle-replace.expand-open :global(svg) {
    transform: rotate(90deg);
  }

  .replace-actions {
    display: flex;
    gap: var(--s-1);
  }

  .replace-btn {
    display: flex;
    align-items: center;
    gap: var(--s-1);
    padding: var(--s-1) var(--s-2);
    border: 1px solid var(--border);
    border-radius: var(--r-1);
    background: var(--surface-2);
    color: var(--text-2);
    font-size: var(--fs-12);
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
    white-space: nowrap;
  }

  .replace-btn:hover {
    background: var(--blue-soft);
    border-color: var(--blue);
    color: var(--blue);
  }

  .replace-btn.replace-all {
    background: var(--surface-2);
  }

  .replace-btn.replace-all:hover {
    background: var(--green-soft);
    border-color: var(--green);
    color: var(--green);
  }

  .find-close {
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    border-radius: var(--r-1);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .find-close:hover {
    background: var(--surface-2);
    color: var(--text);
  }
</style>
