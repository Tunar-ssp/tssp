<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    note: any;
    isSaving: boolean;
    showInspector?: boolean;
    onPin: () => void;
    onDuplicate: () => void;
    onDelete: () => void;
    onToggleInspector?: () => void;
  }

  let { note, isSaving, showInspector = false, onPin, onDuplicate, onDelete, onToggleInspector }: $$Props = $props();
</script>

<header class="stage-header">
  <div class="stage-path">
    <button type="button" class="ghost-link" onclick={() => window.location.hash = '#notes'}>Notes</button>
    <span>/</span>
    {#each note.tags.slice(0, 2) as tag}
      <span class="path-chip">{tag}</span>
    {/each}
    <strong>{note.title || 'Untitled note'}</strong>
  </div>

  <div class="stage-actions">
    {#if isSaving}
      <span class="save-state saving"><span class="status-dot"></span>Saving</span>
    {:else}
      <span class="save-state"><span class="status-dot"></span>Saved</span>
    {/if}
    <button type="button" class="action-btn" onclick={onPin}>
      <Icons.Pin size={14} />
      {note.pinned_at ? 'Pinned' : 'Pin'}
    </button>
    {#if onToggleInspector}
      <button type="button" class="action-btn" class:active={showInspector} onclick={onToggleInspector} title="Toggle details panel">
        <Icons.Eye size={14} />
        Details
      </button>
    {/if}
    <button type="button" class="action-btn" onclick={onDuplicate}>
      <Icons.Copy size={14} />
      Duplicate
    </button>
    <button type="button" class="icon-btn danger" title="Delete note" onclick={onDelete}>
      <Icons.Trash2 size={14} />
    </button>
  </div>
</header>

<style>
  .stage-header {
    min-height: 76px;
    padding: 16px 24px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    background: rgba(12, 14, 20, 0.98);
  }

  .stage-path {
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--muted);
    flex-wrap: wrap;
  }

  .stage-path strong {
    color: var(--text);
    font-size: 16px;
  }

  .path-chip {
    height: 34px;
    padding: 0 14px;
    border-radius: 999px;
    background: rgba(41, 30, 18, 0.96);
    color: var(--orange);
    border: 1px solid rgba(255, 138, 61, 0.22);
    display: inline-flex;
    align-items: center;
  }

  .stage-actions {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .ghost-link {
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    font-size: 14px;
  }

  .ghost-link:hover {
    color: var(--text);
  }

  .save-state {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--green);
    font-size: 14px;
  }

  .save-state.saving {
    color: var(--warning);
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 999px;
    background: currentColor;
    box-shadow: 0 0 18px currentColor;
  }

  .action-btn,
  .icon-btn {
    height: 40px;
    padding: 0 14px;
    border-radius: 14px;
    background: rgba(18, 22, 31, 0.96);
    border: 1px solid var(--border);
    color: var(--text-2);
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .action-btn:hover,
  .icon-btn:hover {
    color: var(--text);
    background: rgba(25, 29, 40, 0.96);
  }

  .action-btn.active {
    color: var(--text);
    background: rgba(110, 168, 255, 0.15);
    border-color: rgba(110, 168, 255, 0.3);
  }

  .icon-btn {
    width: 40px;
    justify-content: center;
    padding: 0;
  }

  .icon-btn.danger {
    color: var(--danger);
  }

  @media (max-width: 960px) {
    .stage-header {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
