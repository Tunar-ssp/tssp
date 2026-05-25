<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import Kbd from './Kbd.svelte';

  interface $$Props {
    isOpen?: boolean;
    onClose?: () => void;
    onFind?: (query: string, caseSensitive: boolean, wholeWord: boolean) => void;
    class?: string;
  }

  let {
    isOpen = false,
    onClose,
    onFind,
    class: className,
  } = $props<$$Props>();

  let searchQuery = $state('');
  let caseSensitive = $state(false);
  let wholeWord = $state(false);
  let searchInput: HTMLInputElement;

  $effect(() => {
    if (isOpen && searchInput) {
      searchInput.focus();
    }
  });

  function handleSearch() {
    if (onFind) {
      onFind(searchQuery, caseSensitive, wholeWord);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (onClose) onClose();
    } else if (e.key === 'Enter') {
      handleSearch();
    }
  }
</script>

{#if isOpen}
  <div class="find-widget {className || ''}">
    <div class="find-controls">
      <input
        bind:this={searchInput}
        type="text"
        placeholder="Find..."
        bind:value={searchQuery}
        on:keydown={handleKeydown}
        on:input={handleSearch}
        class="find-input"
      />

      <div class="find-toggles">
        <button
          class="toggle-btn"
          class:active={caseSensitive}
          on:click={() => (caseSensitive = !caseSensitive)}
          title="Match case"
        >
          <Icons.CaseSensitive size={14} />
        </button>
        <button
          class="toggle-btn"
          class:active={wholeWord}
          on:click={() => (wholeWord = !wholeWord)}
          title="Match whole word"
        >
          <Icons.ALargeSmall size={14} />
        </button>
      </div>

      <div class="find-shortcuts">
        <span class="shortcut">
          <Kbd>Enter</Kbd>
          <span>Next</span>
        </span>
        <span class="shortcut">
          <Kbd>Esc</Kbd>
          <span>Close</span>
        </span>
      </div>

      <button
        class="close-btn"
        on:click={onClose}
        title="Close (Esc)"
      >
        <Icons.X size={16} />
      </button>
    </div>
  </div>
{/if}

<style>
  .find-widget {
    display: flex;
    align-items: center;
    height: 40px;
    padding: 0 var(--s-3);
    background: var(--surface-2);
    border-bottom: 1px solid var(--border);
    gap: var(--s-3);
    flex-shrink: 0;
  }

  .find-controls {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    width: 100%;
  }

  .find-input {
    flex: 1;
    max-width: 300px;
    padding: var(--s-1) var(--s-2);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    border-radius: var(--r-1);
    font-size: var(--fs-12);
    font-family: var(--ff-sans);
  }

  .find-input:focus {
    outline: none;
    border-color: var(--blue);
    box-shadow: 0 0 0 2px rgba(110, 168, 255, 0.1);
  }

  .find-toggles {
    display: flex;
    gap: var(--s-1);
  }

  .toggle-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    border-radius: var(--r-1);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .toggle-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .toggle-btn.active {
    background: var(--blue-subtle);
    color: var(--blue);
    border-color: var(--blue);
  }

  .find-shortcuts {
    display: flex;
    gap: var(--s-4);
    margin-left: auto;
  }

  .shortcut {
    display: flex;
    align-items: center;
    gap: var(--s-1);
    font-size: var(--fs-11);
    color: var(--muted);
  }

  .close-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    border-radius: var(--r-1);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .close-btn:hover {
    background: var(--surface);
    color: var(--text);
  }
</style>
