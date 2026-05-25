<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface MenuItem {
    id: string;
    label: string;
    description?: string;
    icon: any;
    action: () => void;
  }

  interface $$Props {
    items?: MenuItem[];
    isOpen?: boolean;
    x?: number;
    y?: number;
    onClose?: () => void;
    class?: string;
  }

  let {
    items = [],
    isOpen = false,
    x = 0,
    y = 0,
    onClose,
    class: className,
  } = $props<$$Props>();

  let selectedIndex = $state(0);

  const defaultItems: MenuItem[] = [
    {
      id: 'heading1',
      label: 'Heading 1',
      description: 'Large title',
      icon: Icons.Heading1,
      action: () => insertText('# '),
    },
    {
      id: 'heading2',
      label: 'Heading 2',
      description: 'Medium title',
      icon: Icons.Heading2,
      action: () => insertText('## '),
    },
    {
      id: 'bold',
      label: 'Bold',
      description: 'Bold text',
      icon: Icons.Bold,
      action: () => insertText('**text**'),
    },
    {
      id: 'italic',
      label: 'Italic',
      description: 'Italic text',
      icon: Icons.Italic,
      action: () => insertText('*text*'),
    },
    {
      id: 'code',
      label: 'Code',
      description: 'Code block',
      icon: Icons.Code2,
      action: () => insertText('```\n\n```'),
    },
    {
      id: 'quote',
      label: 'Quote',
      description: 'Block quote',
      icon: Icons.Quote,
      action: () => insertText('> '),
    },
    {
      id: 'list',
      label: 'List',
      description: 'Bullet list',
      icon: Icons.List,
      action: () => insertText('- '),
    },
    {
      id: 'checkbox',
      label: 'Checkbox',
      description: 'Task checkbox',
      icon: Icons.CheckSquare2,
      action: () => insertText('- [ ] '),
    },
  ];

  let menuItems = $derived(items.length > 0 ? items : defaultItems);

  function insertText(text: string) {
    // This would be implemented by the parent component
    const event = new CustomEvent('insert', { detail: { text } });
    window.dispatchEvent(event);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, menuItems.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      menuItems[selectedIndex]?.action();
      if (onClose) onClose();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      if (onClose) onClose();
    }
  }
</script>

{#if isOpen}
  <div
    class="slash-menu {className || ''}"
    style="left: {x}px; top: {y}px"
    role="menu"
    tabindex="-1"
    onkeydown={handleKeydown}
  >
    <div class="menu-items">
      {#each menuItems as item, idx (item.id)}
        {@const Icon = item.icon}
        <button
          type="button"
          role="menuitem"
          class="menu-item"
          class:selected={idx === selectedIndex}
          onclick={() => {
            item.action();
            if (onClose) onClose();
          }}
        >
          <div class="item-icon">
            <Icon size={16} />
          </div>
          <div class="item-text">
            <div class="item-label">{item.label}</div>
            {#if item.description}
              <div class="item-description">{item.description}</div>
            {/if}
          </div>
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .slash-menu {
    position: fixed;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    box-shadow: var(--shadow-card);
    z-index: 1000;
    max-width: 250px;
    animation: slideUp var(--duration-quick) var(--ease-smooth);
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .menu-items {
    display: flex;
    flex-direction: column;
  }

  .menu-item {
    display: flex;
    align-items: flex-start;
    gap: var(--s-3);
    padding: var(--s-2) var(--s-3);
    border: none;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .menu-item:hover,
  .menu-item.selected {
    background: var(--surface-2);
  }

  .item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    color: var(--text-2);
    margin-top: 2px;
  }

  .item-text {
    flex: 1;
  }

  .item-label {
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text);
  }

  .item-description {
    font-size: var(--fs-11);
    color: var(--muted);
    margin-top: 2px;
  }
</style>
