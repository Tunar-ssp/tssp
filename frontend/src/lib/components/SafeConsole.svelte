<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { activeOverlays } from '$lib/stores/ui';

  interface ConsoleCommand {
    id: string;
    name: string;
    description?: string;
    category: string;
  }

  interface $$Props {
    isOpen?: boolean;
    onClose?: () => void;
    onExecuteCommand?: (command: string) => Promise<string>;
    commands?: ConsoleCommand[];
  }

  let {
    isOpen = false,
    onClose = () => {},
    onExecuteCommand = async () => '',
    commands = [],
  }: $$Props = $props();

  $effect(() => {
    if (isOpen) {
      activeOverlays.push('modal');
      return () => activeOverlays.remove('modal');
    }
  });

  let input = $state('');
  let history: Array<{ type: 'input' | 'output' | 'error'; text: string }> = $state([]);
  let isExecuting = $state(false);
  let consoleElement = $state<HTMLDivElement | undefined>(undefined);
  let inputElement = $state<HTMLInputElement | undefined>(undefined);

  let availableCommands = $derived(
    commands && commands.length > 0
      ? commands.map((cmd: any) => ({
          id: cmd.name || cmd.id,
          name: cmd.name || cmd.id,
          description: cmd.description || 'Safe system command',
          category: cmd.category || 'general',
        }))
      : []
  );

  function scrollToBottom() {
    if (consoleElement) {
      consoleElement.scrollTop = consoleElement.scrollHeight;
    }
  }

  async function executeCommand(cmd: string) {
    const trimmed = cmd.trim();
    if (!trimmed) return;

    history = [...history, { type: 'input', text: trimmed }];
    input = '';
    isExecuting = true;

    try {
      const output = await onExecuteCommand(trimmed);

      // Try to parse as JSON and format it nicely
      let displayOutput = output;
      try {
        const json = JSON.parse(output);
        if (json && typeof json === 'object') {
          displayOutput = JSON.stringify(json, null, 2);
        }
      } catch {
        // Not JSON, use raw output
      }

      history = [...history, { type: 'output', text: displayOutput }];
    } catch (err) {
      history = [...history, { type: 'error', text: `Error: ${err instanceof Error ? err.message : 'Unknown error'}` }];
    } finally {
      isExecuting = false;
      scrollToBottom();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      executeCommand(input);
    } else if (e.key === 'Escape') {
      onClose();
    }
  }

  function insertCommand(cmd: string) {
    input = cmd + ' ';
    if (inputElement) inputElement.focus();
  }

  $effect(() => {
    if (isOpen && inputElement) {
      inputElement.focus();
    }
  });

  $effect(() => {
    scrollToBottom();
  });
</script>

{#if isOpen}
  <div class="safe-console">
    <div class="console-header">
      <h3>Safe Console</h3>
      <p class="subtitle">Execute whitelisted commands</p>
      <button
        class="console-close"
        onclick={onClose}
        title="Close (Escape)"
      >
        <Icons.X size={16} />
      </button>
    </div>

    <div class="console-quick-commands">
      {#each availableCommands.slice(0, 4) as cmd (cmd.id)}
        <button
          class="quick-cmd"
          onclick={() => insertCommand(cmd.name)}
          title={cmd.description}
        >
          <Icons.Terminal size={12} />
          <span>{cmd.name}</span>
        </button>
      {/each}
    </div>

    <div class="console-output" bind:this={consoleElement}>
      {#each history as item, i (i)}
        <div class="console-line" class:input={item.type === 'input'} class:error={item.type === 'error'}>
          {#if item.type === 'input'}
            <span class="prompt">$</span>
          {:else if item.type === 'error'}
            <span class="error-icon">!</span>
          {:else}
            <span class="output-icon">›</span>
          {/if}
          <span class="line-text">{item.text}</span>
        </div>
      {/each}
      {#if isExecuting}
        <div class="console-line executing">
          <span class="prompt">$</span>
          <span class="spinner"></span>
        </div>
      {/if}
    </div>

    <div class="console-input-area">
      <span class="prompt">$</span>
      <input
        bind:this={inputElement}
        type="text"
        class="console-input"
        bind:value={input}
        onkeydown={handleKeydown}
        placeholder="Type command or press Escape to close"
        disabled={isExecuting}
      />
    </div>

    {#if availableCommands.length > 4}
      <div class="console-commands">
        <h4>Available Commands:</h4>
        <div class="commands-grid">
          {#each availableCommands as cmd (cmd.id)}
            <div class="command-item">
              <button
                class="cmd-button"
                onclick={() => insertCommand(cmd.name)}
                disabled={isExecuting}
              >
                {cmd.name}
              </button>
              <span class="cmd-desc">{cmd.description}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .safe-console {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: 60vh;
    background: var(--bg);
    border-top: 1px solid var(--border);
    border-left: 1px solid var(--border);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    font-family: var(--ff-mono);
    font-size: var(--fs-12);
    z-index: 1000;
    box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.15);
  }

  .console-header {
    padding: var(--s-3) var(--s-4);
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    align-items: center;
    gap: var(--s-3);
    flex-shrink: 0;
  }

  .console-header h3 {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
  }

  .subtitle {
    margin: 0;
    font-size: var(--fs-11);
    color: var(--muted);
  }

  .console-close {
    margin-left: auto;
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    border-radius: var(--r-1);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
    flex-shrink: 0;
  }

  .console-close:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .console-quick-commands {
    display: flex;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-4);
    border-bottom: 1px solid var(--hairline);
    background: var(--surface);
    flex-shrink: 0;
  }

  .quick-cmd {
    display: flex;
    align-items: center;
    gap: var(--s-1);
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: var(--r-1);
    background: transparent;
    color: var(--blue);
    font-size: var(--fs-11);
    font-family: var(--ff-mono);
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .quick-cmd:hover {
    background: var(--blue-subtle);
    border-color: var(--blue);
  }

  .console-output {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-3) var(--s-4);
    background: var(--bg);
    line-height: 1.5;
  }

  .console-line {
    display: flex;
    gap: var(--s-2);
    margin-bottom: var(--s-1);
    word-break: break-all;
  }

  .console-line.input {
    color: var(--blue);
    font-weight: 500;
  }

  .console-line.error {
    color: var(--danger);
  }

  .console-line.executing {
    color: var(--muted);
  }

  .prompt {
    color: var(--muted);
    font-weight: 600;
    flex-shrink: 0;
  }

  .error-icon {
    color: var(--danger);
    font-weight: bold;
    flex-shrink: 0;
  }

  .output-icon {
    color: var(--muted);
    flex-shrink: 0;
  }

  .line-text {
    flex: 1;
    color: var(--text);
  }

  .spinner {
    display: inline-block;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--text-2);
    animation: blink 1s ease-in-out infinite;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  .console-input-area {
    padding: var(--s-2) var(--s-4);
    border-top: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    align-items: center;
    gap: var(--s-2);
    flex-shrink: 0;
  }

  .console-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-family: var(--ff-mono);
    font-size: var(--fs-12);
    outline: none;
  }

  .console-input::placeholder {
    color: var(--muted);
  }

  .console-commands {
    padding: var(--s-3) var(--s-4);
    border-top: 1px solid var(--hairline);
    background: var(--surface-2);
    max-height: 40%;
    overflow-y: auto;
    flex-shrink: 0;
  }

  .console-commands h4 {
    margin: 0 0 var(--s-2) 0;
    font-size: var(--fs-11);
    font-weight: 600;
    color: var(--text-2);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .commands-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: var(--s-2);
  }

  .command-item {
    display: flex;
    flex-direction: column;
    gap: var(--s-1);
  }

  .cmd-button {
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: var(--r-1);
    background: var(--bg);
    color: var(--blue);
    font-family: var(--ff-mono);
    font-size: var(--fs-11);
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .cmd-button:hover:not(:disabled) {
    background: var(--blue-subtle);
    border-color: var(--blue);
  }

  .cmd-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .cmd-desc {
    font-size: var(--fs-10);
    color: var(--muted);
  }
</style>
