<script lang="ts">
  /**
   * Keyboard shortcuts help dialog for block editor
   */

  interface Shortcut {
    keys: string;
    action: string;
    category: string;
  }

  let { onClose }: { onClose?: () => void } = $props();

  const shortcuts: Shortcut[] = [
    // Slash commands
    { keys: '/', action: 'Open command palette', category: 'Insertion' },

    // Block operations
    { keys: 'Enter', action: 'Create new block below', category: 'Block Editing' },
    { keys: 'Shift+Enter', action: 'Soft line break', category: 'Block Editing' },
    { keys: 'Backspace', action: 'Delete block if empty', category: 'Block Editing' },
    { keys: 'Cmd/Ctrl+Backspace', action: 'Delete block', category: 'Block Editing' },
    { keys: 'Alt+↑', action: 'Move block up', category: 'Block Editing' },
    { keys: 'Alt+↓', action: 'Move block down', category: 'Block Editing' },

    // Formatting
    { keys: 'Cmd/Ctrl+B', action: 'Bold', category: 'Formatting' },
    { keys: 'Cmd/Ctrl+I', action: 'Italic', category: 'Formatting' },

    // History
    { keys: 'Cmd/Ctrl+Z', action: 'Undo', category: 'History' },
    { keys: 'Cmd/Ctrl+Shift+Z', action: 'Redo', category: 'History' },
    { keys: 'Cmd/Ctrl+Y', action: 'Redo (alternate)', category: 'History' },

    // Navigation
    { keys: 'Tab', action: 'Indent block (future)', category: 'Navigation' },
  ];

  const categories = [...new Set(shortcuts.map(s => s.category))];

  function handleEscape(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose?.();
    }
  }
</script>

<div
  class="shortcuts-overlay"
  onclick={() => onClose?.()}
  onkeydown={handleEscape}
  role="dialog"
  aria-modal="true"
  aria-labelledby="shortcuts-title"
  tabindex="-1"
>
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="shortcuts-modal"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="document"
    tabindex="-1"
  >
    <div class="shortcuts-header">
      <h2 id="shortcuts-title">Keyboard Shortcuts</h2>
      <button
        class="close-btn"
        onclick={() => onClose?.()}
        aria-label="Close"
      >
        ✕
      </button>
    </div>

    <div class="shortcuts-content">
      {#each categories as category}
        <div class="shortcut-section">
          <h3>{category}</h3>
          <div class="shortcuts-list">
            {#each shortcuts.filter(s => s.category === category) as shortcut}
              <div class="shortcut-row">
                <kbd class="shortcut-keys">{shortcut.keys}</kbd>
                <span class="shortcut-action">{shortcut.action}</span>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>

    <div class="shortcuts-footer">
      <p>Press <kbd>Esc</kbd> to close</p>
    </div>
  </div>
</div>

<style>
  .shortcuts-overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
    padding: 16px;
  }

  .shortcuts-modal {
    background-color: var(--bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    max-width: 600px;
    width: 100%;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .shortcuts-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
  }

  .shortcuts-header h2 {
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
    background: none;
    color: var(--muted);
    cursor: pointer;
    font-size: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.2s;
  }

  .close-btn:hover {
    color: var(--text);
  }

  .shortcuts-content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }

  .shortcut-section {
    margin-bottom: 24px;
  }

  .shortcut-section:last-child {
    margin-bottom: 0;
  }

  .shortcut-section h3 {
    margin: 0 0 12px 0;
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

  .shortcut-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    border-radius: 6px;
    background-color: rgba(0, 0, 0, 0.02);
  }

  .shortcut-keys {
    min-width: 140px;
    padding: 4px 8px;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-family: 'Monaco', 'Courier New', monospace;
    font-size: 12px;
    color: var(--text);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .shortcut-action {
    font-size: 13px;
    color: var(--text);
  }

  .shortcuts-footer {
    padding: 12px 24px;
    border-top: 1px solid var(--border);
    background-color: rgba(0, 0, 0, 0.02);
    font-size: 12px;
    color: var(--muted);
    text-align: center;
  }

  .shortcuts-footer p {
    margin: 0;
  }

  .shortcuts-footer kbd {
    padding: 2px 6px;
    background-color: var(--bg);
    border: 1px solid var(--border);
    border-radius: 3px;
  }

  @media (max-width: 600px) {
    .shortcuts-modal {
      max-height: 90vh;
    }

    .shortcut-row {
      flex-direction: column;
      align-items: flex-start;
    }

    .shortcut-keys {
      min-width: auto;
    }
  }
</style>
