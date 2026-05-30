<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { activeOverlays } from '$lib/stores/ui';

  interface Props {
    isOpen?: boolean;
    onClose?: () => void;
  }

  let { isOpen = false, onClose }: Props = $props();

  $effect(() => {
    if (isOpen) {
      activeOverlays.push('modal');
      return () => activeOverlays.remove('modal');
    }
  });

  const shortcuts = [
    { group: 'Navigation', items: [
      { keys: '↑ / ↓', description: 'Navigate files' },
      { keys: 'Enter', description: 'Preview selected file' },
      { keys: 'Escape', description: 'Deselect all files' },
    ]},
    { group: 'Selection', items: [
      { keys: 'Ctrl+A', description: 'Select all files' },
      { keys: 'Ctrl+Click', description: 'Multi-select files' },
      { keys: 'Shift+Click', description: 'Range select files' },
    ]},
    { group: 'Operations', items: [
      { keys: 'Ctrl+C', description: 'Copy selected files' },
      { keys: 'Ctrl+X', description: 'Cut selected files' },
      { keys: 'Ctrl+V', description: 'Paste files' },
      { keys: 'Shift+Delete', description: 'Delete selected files' },
    ]},
    { group: 'Hover Actions', items: [
      { keys: '👁', description: 'Preview file (hover)' },
      { keys: '⬇', description: 'Download file (hover)' },
      { keys: '📋', description: 'Copy to clipboard (hover)' },
    ]},
  ];
</script>

{#if isOpen}
  <div class="modal-overlay" onclick={onClose}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>Keyboard Shortcuts</h2>
        <button
          type="button"
          class="close-btn"
          onclick={onClose}
          title="Close"
        >
          <Icons.X size={20} />
        </button>
      </div>

      <div class="shortcuts-grid">
        {#each shortcuts as group}
          <div class="shortcut-group">
            <h3>{group.group}</h3>
            <div class="shortcuts-list">
              {#each group.items as item}
                <div class="shortcut-item">
                  <kbd class="shortcut-keys">{item.keys}</kbd>
                  <span class="shortcut-desc">{item.description}</span>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>

      <div class="modal-footer">
        <p class="hint">Press <kbd>?</kbd> or <kbd>Cmd/?</kbd> anytime to show this help</p>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 150ms ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .modal-content {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    width: 90%;
    max-width: 600px;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
    animation: slideUp 200ms ease-out;
  }

  @keyframes slideUp {
    from {
      transform: translateY(20px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 24px;
    border-bottom: 1px solid var(--border);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--text);
  }

  .close-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    transition: all 150ms;
  }

  .close-btn:hover {
    background: var(--surface);
    color: var(--text);
  }

  .shortcuts-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 24px;
    padding: 24px;
  }

  .shortcut-group h3 {
    margin: 0 0 12px;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
  }

  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .shortcut-item {
    display: flex;
    gap: 12px;
    align-items: flex-start;
  }

  .shortcut-keys {
    padding: 2px 6px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-family: monospace;
    font-size: 11px;
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .shortcut-desc {
    font-size: 13px;
    color: var(--text-2);
    line-height: 1.4;
    padding-top: 3px;
  }

  .modal-footer {
    padding: 16px 24px;
    border-top: 1px solid var(--border);
    background: var(--surface);
    border-radius: 0 0 12px 12px;
  }

  .hint {
    margin: 0;
    font-size: 12px;
    color: var(--muted);
  }

  .hint kbd {
    padding: 2px 6px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 3px;
    font-family: monospace;
    font-size: 11px;
    font-weight: 500;
  }
</style>
