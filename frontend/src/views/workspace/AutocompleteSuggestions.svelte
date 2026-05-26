<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Suggestion {
    id: string;
    label: string;
    kind: 'function' | 'variable' | 'class' | 'keyword' | 'snippet';
    detail?: string;
    documentation?: string;
  }

  interface $$Props {
    suggestions?: Suggestion[];
    selectedIndex?: number;
    isOpen?: boolean;
    onSelect?: (suggestion: Suggestion) => void;
    onClose?: () => void;
  }

  let {
    suggestions = [],
    selectedIndex = 0,
    isOpen = false,
    onSelect = () => {},
    onClose = () => {},
  }: $$Props = $props();

  function getIcon(kind: string) {
    switch (kind) {
      case 'function':
        return Icons.Function;
      case 'variable':
        return Icons.Variable;
      case 'class':
        return Icons.Braces;
      case 'keyword':
        return Icons.Key;
      case 'snippet':
        return Icons.Code;
      default:
        return Icons.Circle;
    }
  }

  function getKindColor(kind: string) {
    switch (kind) {
      case 'function':
        return 'var(--green)';
      case 'variable':
        return 'var(--blue)';
      case 'class':
        return 'var(--violet)';
      case 'keyword':
        return 'var(--orange)';
      case 'snippet':
        return 'var(--cyan)';
      default:
        return 'var(--text-2)';
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, suggestions.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        break;
      case 'Enter':
        e.preventDefault();
        if (suggestions[selectedIndex]) {
          onSelect(suggestions[selectedIndex]);
        }
        break;
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen && suggestions.length > 0}
  <div class="autocomplete-popup">
    <div class="suggestions-list">
      {#each suggestions as suggestion, index (suggestion.id)}
        <button
          type="button"
          class="suggestion-item"
          class:selected={index === selectedIndex}
          onclick={() => onSelect(suggestion)}
        >
          <span class="suggestion-icon" style="--kind-color: {getKindColor(suggestion.kind)}">
            <svelte:component this={getIcon(suggestion.kind)} size={14} />
          </span>
          <div class="suggestion-content">
            <span class="suggestion-label">{suggestion.label}</span>
            {#if suggestion.detail}
              <span class="suggestion-detail">{suggestion.detail}</span>
            {/if}
          </div>
        </button>
      {/each}
    </div>

    {#if suggestions[selectedIndex]?.documentation}
      <div class="suggestion-docs">
        <div class="docs-content">
          {suggestions[selectedIndex].documentation}
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .autocomplete-popup {
    position: fixed;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    max-width: 500px;
    max-height: 400px;
    overflow: hidden;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 500;
    display: flex;
    flex-direction: column;
  }

  .suggestions-list {
    flex: 1;
    overflow-y: auto;
    max-height: 300px;
  }

  .suggestion-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
    font-size: 13px;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .suggestion-item:hover {
    background: var(--surface-2);
  }

  .suggestion-item.selected {
    background: var(--blue-soft);
  }

  .suggestion-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    color: var(--kind-color);
    flex-shrink: 0;
  }

  .suggestion-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .suggestion-label {
    font-weight: 500;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .suggestion-detail {
    font-size: 11px;
    color: var(--muted);
    font-family: var(--ff-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .suggestion-docs {
    padding: 12px;
    border-top: 1px solid var(--border);
    background: var(--bg);
    max-height: 100px;
    overflow-y: auto;
  }

  .docs-content {
    font-size: 11px;
    color: var(--text-2);
    line-height: 1.4;
  }
</style>
