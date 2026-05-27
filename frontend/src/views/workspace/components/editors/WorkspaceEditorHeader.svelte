<script lang="ts">
  interface $$Props {
    name: string;
    selectedLanguage: string;
    isSaving: boolean;
    languages: Array<{ id: string; label: string; ext: string }>;
    onNameChange: (value: string) => void;
    onLanguageChange: (value: string) => void;
  }

  let {
    name,
    selectedLanguage,
    isSaving,
    languages,
    onNameChange,
    onLanguageChange,
  }: $$Props = $props();
</script>

<div class="editor-header">
  <input
    type="text"
    class="name-input"
    placeholder="Untitled workspace"
    value={name}
    oninput={(e) => onNameChange((e.target as HTMLInputElement).value)}
    aria-label="Workspace name"
  />

  <div class="editor-actions">
    <select
      class="language-select"
      value={selectedLanguage}
      onchange={(event) => onLanguageChange((event.currentTarget as HTMLSelectElement).value)}
    >
      {#each languages as language}
        <option value={language.id}>{language.label}</option>
      {/each}
    </select>

    {#if isSaving}
      <span class="saving">Saving...</span>
    {/if}
  </div>
</div>

<style>
  .editor-header {
    padding: 14px 18px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    background: rgba(15, 17, 23, 0.96);
  }

  .name-input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font-size: 28px;
    line-height: 1.1;
    font-weight: 700;
    letter-spacing: -0.03em;
  }

  .name-input::placeholder {
    color: var(--dim);
  }

  .editor-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .language-select {
    height: 38px;
    padding: 0 12px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: rgba(12, 14, 20, 0.98);
    color: var(--text);
  }

  .saving {
    color: var(--warning);
    font-size: 13px;
  }
</style>
