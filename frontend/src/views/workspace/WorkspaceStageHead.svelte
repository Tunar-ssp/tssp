<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    workspaceName: string;
    onFind: () => void;
    onDelete: () => void;
    onSave: () => void;
  }

  let { workspaceName, onFind, onDelete, onSave }: $$Props = $props();
</script>

<header class="stage-head">
  <div class="stage-breadcrumbs">
    <span>Workspace</span>
    <span>/</span>
    <strong>{workspaceName || 'untitled'}</strong>
  </div>

  <div class="stage-actions">
    <button type="button" class="ghost-btn" onclick={onFind}>
      <Icons.Search size={14} />
      Find
    </button>
    <button type="button" class="ghost-btn" onclick={onDelete}>
      <Icons.Trash2 size={14} />
      Delete
    </button>
    <button type="button" class="primary-btn compact" onclick={onSave}>
      <Icons.Save size={14} />
      Save
    </button>
  </div>
</header>

<style>
  .stage-head {
    min-height: 74px;
    padding: 16px 22px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    background: rgba(12, 14, 20, 0.98);
  }

  .stage-breadcrumbs {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--muted);
    flex-wrap: wrap;
  }

  .stage-breadcrumbs strong {
    color: var(--text);
    font-size: 18px;
  }

  .stage-actions {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .primary-btn,
  .ghost-btn {
    height: 44px;
    padding: 0 16px;
    border-radius: 16px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .primary-btn {
    border: 1px solid rgba(110, 168, 255, 0.24);
    background: rgba(110, 168, 255, 0.14);
    color: var(--blue);
  }

  .primary-btn.compact {
    height: 40px;
  }

  .ghost-btn {
    border: 1px solid var(--border);
    background: rgba(18, 22, 31, 0.96);
    color: var(--text-2);
  }

  .ghost-btn:hover,
  .primary-btn:hover {
    color: var(--text);
  }
</style>
