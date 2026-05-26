<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Shortcut {
    keys: string;
    action: string;
    category: string;
  }

  interface $$Props {
    isOpen?: boolean;
    onClose?: () => void;
  }

  let {
    isOpen = false,
    onClose = () => {},
  }: $$Props = $props();

  const shortcuts: Shortcut[] = [
    // File operations
    { category: 'File Operations', keys: 'Ctrl+S / Cmd+S', action: 'Save file' },
    { category: 'File Operations', keys: 'Ctrl+N / Cmd+N', action: 'New file' },
    { category: 'File Operations', keys: 'Ctrl+O / Cmd+O', action: 'Open file' },

    // Search and Replace
    { category: 'Search', keys: 'Ctrl+F / Cmd+F', action: 'Find in file' },
    { category: 'Search', keys: 'Ctrl+H / Cmd+H', action: 'Find and replace' },
    { category: 'Search', keys: 'Ctrl+Shift+F / Cmd+Shift+F', action: 'Search workspace' },

    // Navigation
    { category: 'Navigation', keys: 'Ctrl+G / Cmd+G', action: 'Go to line' },
    { category: 'Navigation', keys: 'Ctrl+K Ctrl+O / Cmd+K Cmd+O', action: 'Open in workspace' },
    { category: 'Navigation', keys: 'Alt+Left / Opt+Left', action: 'Back' },
    { category: 'Navigation', keys: 'Alt+Right / Opt+Right', action: 'Forward' },

    // Editor
    { category: 'Editor', keys: 'Tab', action: 'Indent' },
    { category: 'Editor', keys: 'Shift+Tab', action: 'Outdent' },
    { category: 'Editor', keys: 'Ctrl+/ / Cmd+/', action: 'Toggle comment' },
    { category: 'Editor', keys: 'Alt+Up / Opt+Up', action: 'Move line up' },
    { category: 'Editor', keys: 'Alt+Down / Opt+Down', action: 'Move line down' },

    // Interface
    { category: 'Interface', keys: 'Ctrl+B / Cmd+B', action: 'Toggle sidebar' },
    { category: 'Interface', keys: 'Ctrl+J / Cmd+J', action: 'Toggle panel' },
    { category: 'Interface', keys: 'Ctrl+K / Cmd+K', action: 'Command palette' },
  ];

  let categorized = $derived.by(() => {
    const groups = new Map<string, Shortcut[]>();
    shortcuts.forEach((shortcut) => {
      if (!groups.has(shortcut.category)) {
        groups.set(shortcut.category, []);
      }
      groups.get(shortcut.category)!.push(shortcut);
    });
    return Array.from(groups.entries());
  });
</script>

{#if isOpen}
  <div class="shortcuts-overlay" onclick={onClose}>
    <div class="shortcuts-dialog" onclick={(e) => e.stopPropagation()}>
      <div class="dialog-header">
        <h2>Keyboard Shortcuts</h2>
        <button
          type="button"
          class="close-btn"
          onclick={onClose}
          aria-label="Close"
        >
          <Icons.X size={20} />
        </button>
      </div>

      <div class="shortcuts-grid">
        {#each categorized as [category, items] (category)}
          <div class="shortcut-group">
            <h3>{category}</h3>
            <div class="shortcut-list">
              {#each items as shortcut (shortcut.action)}
                <div class="shortcut-item">
                  <kbd class="shortcut-keys">{shortcut.keys}</kbd>
                  <span class="shortcut-action">{shortcut.action}</span>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>

      <div class="dialog-footer">
        <p>Press <kbd>?</kbd> to open this help menu</p>
      </div>
    </div>
  </div>
{/if}

<style>
  .shortcuts-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
  }

  .shortcuts-dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 16px;
    max-width: 600px;
    width: 90vw;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 24px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .dialog-header h2 {
    margin: 0;
    font-size: 24px;
    color: var(--text);
  }

  .close-btn {
    width: 40px;
    height: 40px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .close-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .shortcuts-grid {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 24px;
  }

  .shortcut-group h3 {
    margin: 0 0 12px;
    font-size: 14px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-2);
  }

  .shortcut-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .shortcut-item {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .shortcut-keys {
    font-family: var(--ff-mono);
    font-size: 11px;
    font-weight: 600;
    padding: 4px 8px;
    background: var(--bg);
    border: 1px solid var(--hairline);
    border-radius: 4px;
    color: var(--text);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .shortcut-action {
    font-size: 13px;
    color: var(--text-2);
  }

  .dialog-footer {
    padding: 16px 24px;
    border-top: 1px solid var(--border);
    text-align: center;
    color: var(--muted);
    flex-shrink: 0;
  }

  .dialog-footer p {
    margin: 0;
    font-size: 12px;
  }

  .dialog-footer kbd {
    font-family: var(--ff-mono);
    font-size: 11px;
    padding: 2px 6px;
    background: var(--bg);
    border: 1px solid var(--hairline);
    border-radius: 3px;
  }
</style>
