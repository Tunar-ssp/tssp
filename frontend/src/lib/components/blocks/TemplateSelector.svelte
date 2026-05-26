<script lang="ts">
  /**
   * Template selector modal for creating notes from templates
   */

  import { listTemplates } from '$lib/blocks/templates';
  import * as Icons from 'lucide-svelte';

  interface Props {
    onSelect?: (templateId: string) => void;
    onClose?: () => void;
  }

  let { onSelect, onClose }: Props = $props();

  const templates = listTemplates();

  function handleSelect(id: string) {
    onSelect?.(id);
  }
</script>

<div class="template-selector">
  <div class="selector-header">
    <h2>Start with a Template</h2>
    <button
      class="close-btn"
      onclick={onClose}
      aria-label="Close"
    >
      <Icons.X size={20} />
    </button>
  </div>

  <div class="templates-grid">
    {#each templates as [id, template]}
      <button
        class="template-card"
        onclick={() => handleSelect(id)}
      >
        <div class="template-icon">{template.icon}</div>
        <h3>{template.name}</h3>
        <p>{template.description}</p>
      </button>
    {/each}

    <button class="template-card blank" onclick={() => handleSelect('blank')}>
      <div class="template-icon">📝</div>
      <h3>Blank Note</h3>
      <p>Start from scratch</p>
    </button>
  </div>
</div>

<style>
  .template-selector {
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding: 32px;
    max-height: 80vh;
    overflow-y: auto;
  }

  .selector-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .selector-header h2 {
    margin: 0;
    font-size: 24px;
    font-weight: 600;
    color: var(--text);
  }

  .close-btn {
    padding: 0;
    border: none;
    background: none;
    cursor: pointer;
    color: var(--muted);
    transition: color 0.2s;
  }

  .close-btn:hover {
    color: var(--text);
  }

  .templates-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 16px;
  }

  .template-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 20px;
    border: 2px solid var(--border);
    border-radius: 8px;
    background: var(--bg);
    cursor: pointer;
    transition: all 0.2s;
    text-align: center;
    color: var(--text);
    font-family: inherit;
  }

  .template-card:hover {
    border-color: rgba(59, 130, 246, 0.5);
    background-color: rgba(59, 130, 246, 0.05);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .template-card.blank {
    opacity: 0.7;
  }

  .template-card.blank:hover {
    opacity: 1;
  }

  .template-icon {
    font-size: 32px;
  }

  .template-card h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .template-card p {
    margin: 0;
    font-size: 12px;
    color: var(--muted);
  }
</style>
