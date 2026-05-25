<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import Kbd from './Kbd.svelte';

  interface $$Props {
    isOpen?: boolean;
    onClose?: () => void;
    class?: string;
  }

  let {
    isOpen = false,
    onClose,
    class: className,
  } = $props<$$Props>();

  const shortcuts = [
    {
      category: 'Navigation',
      items: [
        { keys: ['Ctrl', 'K'], label: 'Command palette' },
        { keys: ['Ctrl', 'Alt', 'S'], label: 'Settings' },
        { keys: ['Ctrl', '?'], label: 'Keyboard shortcuts' },
      ],
    },
    {
      category: 'Files',
      items: [
        { keys: ['Ctrl', 'U'], label: 'Upload files' },
        { keys: ['Ctrl', 'Shift', 'D'], label: 'Download' },
        { keys: ['Delete'], label: 'Delete file' },
        { keys: ['Ctrl', 'R'], label: 'Rename' },
      ],
    },
    {
      category: 'Notes',
      items: [
        { keys: ['Ctrl', 'N'], label: 'New note' },
        { keys: ['Ctrl', 'S'], label: 'Save note' },
        { keys: ['Ctrl', '/'], label: 'Slash menu' },
        { keys: ['Ctrl', 'B'], label: 'Toggle bold' },
      ],
    },
    {
      category: 'Workspace',
      items: [
        { keys: ['Ctrl', 'Shift', 'N'], label: 'New file' },
        { keys: ['Ctrl', 'P'], label: 'Quick open' },
        { keys: ['Ctrl', 'Shift', 'F'], label: 'Find in files' },
        { keys: ['F5'], label: 'Run code' },
      ],
    },
  ];

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && onClose) {
      onClose();
    }
  }
</script>

{#if isOpen}
  <div
    class="shortcuts-backdrop"
    role="presentation"
    tabindex="-1"
    onclick={handleBackdropClick}
    onkeydown={(e) => {
      if (e.key === 'Escape' && onClose) onClose();
    }}
  >
    <div class="shortcuts-modal {className || ''}">
      <div class="shortcuts-header">
        <h2 class="shortcuts-title">Keyboard Shortcuts</h2>
        {#if onClose}
          <button type="button" class="shortcuts-close" onclick={onClose} aria-label="Close">
            <Icons.X size={20} />
          </button>
        {/if}
      </div>

      <div class="shortcuts-content">
        {#each shortcuts as section (section.category)}
          <div class="shortcut-section">
            <h3 class="section-title">{section.category}</h3>
            <div class="shortcuts-list">
              {#each section.items as item (item.label)}
                <div class="shortcut-item">
                  <div class="shortcut-keys">
                    {#each item.keys as key (key)}
                      <Kbd>{key}</Kbd>
                    {/each}
                  </div>
                  <div class="shortcut-label">{item.label}</div>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>

      <div class="shortcuts-footer">
        <p>Press <Kbd>?</Kbd> to toggle shortcuts at any time</p>
      </div>
    </div>
  </div>
{/if}

<style>
  .shortcuts-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2001;
    animation: fadeIn var(--duration-normal) var(--ease-smooth);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .shortcuts-modal {
    width: 90%;
    max-width: 800px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-3);
    box-shadow: var(--shadow-modal);
    max-height: 80vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    animation: modalSlideIn var(--duration-normal) var(--ease-smooth);
  }

  @keyframes modalSlideIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .shortcuts-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-6);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .shortcuts-title {
    margin: 0;
    font-size: var(--fs-20);
    font-weight: 600;
    color: var(--text);
  }

  .shortcuts-close {
    width: 36px;
    height: 36px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-2);
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .shortcuts-close:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .shortcuts-content {
    flex: 1;
    padding: var(--s-6);
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--s-8);
  }

  .shortcut-section {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .section-title {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .shortcut-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-4);
  }

  .shortcut-keys {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .shortcut-label {
    flex: 1;
    font-size: var(--fs-13);
    color: var(--text-2);
  }

  .shortcuts-footer {
    padding: var(--s-4);
    border-top: 1px solid var(--border);
    text-align: center;
    font-size: var(--fs-12);
    color: var(--muted);
    flex-shrink: 0;
  }

  .shortcuts-footer p {
    margin: 0;
  }
</style>
