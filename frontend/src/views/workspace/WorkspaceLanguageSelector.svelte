<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Language {
    id: string;
    label: string;
    ext: string;
  }

  interface $$Props {
    selectedLanguage: string;
    languages: Language[];
    onLanguageChange?: (language: string) => void;
  }

  let { selectedLanguage, languages, onLanguageChange }: $$Props = $props();

  let isOpen = $state(false);

  function handleSelect(langId: string) {
    onLanguageChange?.(langId);
    isOpen = false;
  }

  function getLanguageLabel(id: string): string {
    return languages.find((lang) => lang.id === id)?.label || id;
  }
</script>

<div class="language-selector">
  <button
    type="button"
    class="selector-trigger"
    onclick={() => (isOpen = !isOpen)}
    title="Change language"
  >
    <Icons.Code size={16} />
    <span>{getLanguageLabel(selectedLanguage)}</span>
    <Icons.ChevronDown size={14} />
  </button>

  {#if isOpen}
    <div class="selector-dropdown">
      {#each languages as lang}
        <button
          type="button"
          class="dropdown-item"
          class:active={selectedLanguage === lang.id}
          onclick={() => handleSelect(lang.id)}
        >
          <span>{lang.label}</span>
          <span class="lang-ext">{lang.ext}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .language-selector {
    position: relative;
    display: inline-block;
  }

  .selector-trigger {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 40px;
    padding: 0 14px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
    font-size: 14px;
    transition: all 0.2s;
  }

  .selector-trigger:hover {
    color: var(--text);
    background: var(--surface-3);
  }

  .selector-trigger span {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
  }

  .selector-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    width: 200px;
    max-height: 300px;
    overflow-y: auto;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--surface);
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
    z-index: 100;
  }

  .dropdown-item {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s;
    text-align: left;
  }

  .dropdown-item:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .dropdown-item.active {
    background: rgba(110, 168, 255, 0.14);
    color: var(--blue);
    font-weight: 600;
  }

  .lang-ext {
    color: var(--muted);
    font-family: var(--ff-mono);
    font-size: 11px;
  }
</style>
