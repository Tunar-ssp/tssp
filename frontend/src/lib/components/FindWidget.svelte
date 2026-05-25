<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    isOpen?: boolean;
    onClose?: () => void;
    onFind?: (query: string, options: { matchCase: boolean; wholeWord: boolean }) => void;
  }

  let {
    isOpen = false,
    onClose = () => {},
    onFind = () => {},
  }: $$Props = $props();

  let searchQuery = $state('');
  let matchCase = $state(false);
  let wholeWord = $state(false);
  let findInput = $state<HTMLInputElement | null>(null);

  function handleSearch() {
    onFind(searchQuery, { matchCase, wholeWord });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleSearch();
    } else if (e.key === 'Escape') {
      onClose();
    }
  }

  $effect(() => {
    if (isOpen && findInput) {
      findInput.focus();
    }
  });
</script>

{#if isOpen}
  <div class="find-widget">
    <div class="find-input-group">
      <Icons.Search size={16} />
      <input
        bind:this={findInput}
        type="text"
        class="find-input"
        placeholder="Find in file..."
        bind:value={searchQuery}
        onkeydown={handleKeydown}
      />
    </div>

    <div class="find-options">
      <button
        type="button"
        class="find-option"
        class:active={matchCase}
        onclick={() => matchCase = !matchCase}
        title="Match case"
      >
        <Icons.CaseSensitive size={14} />
      </button>
      <button
        type="button"
        class="find-option"
        class:active={wholeWord}
        onclick={() => wholeWord = !wholeWord}
        title="Match whole word"
      >
        <Icons.Type size={14} />
      </button>
    </div>

    <button
      type="button"
      class="find-close"
      onclick={onClose}
      title="Close (Escape)"
    >
      <Icons.X size={16} />
    </button>
  </div>
{/if}

<style>
  .find-widget {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    flex-shrink: 0;
  }

  .find-input-group {
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

  .find-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--fs-12);
    outline: none;
  }

  .find-input::placeholder {
    color: var(--muted);
  }

  .find-options {
    display: flex;
    gap: var(--s-1);
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
