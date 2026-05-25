<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import Kbd from './Kbd.svelte';

  interface ConsoleOutput {
    id: string;
    type: 'command' | 'output' | 'error' | 'info';
    text: string;
    timestamp: number;
  }

  interface $$Props {
    isOpen?: boolean;
    onClose?: () => void;
    onCommand?: (command: string) => void;
    height?: number;
    class?: string;
  }

  let {
    isOpen = false,
    onClose,
    onCommand,
    height = 300,
    class: className,
  } = $props<$$Props>();

  let outputs = $state<ConsoleOutput[]>([
    {
      id: '1',
      type: 'info',
      text: 'Safe Console v1.0 - Type "help" for available commands',
      timestamp: Date.now(),
    },
  ]);

  let input = $state('');
  let consoleEl: HTMLElement;

  const allowedCommands = [
    'help',
    'status',
    'uptime',
    'version',
    'logs',
    'users',
    'stats',
    'backup',
  ];

  function executeCommand(cmd: string) {
    const trimmed = cmd.trim();
    if (!trimmed) return;

    outputs = [
      ...outputs,
      {
        id: Date.now().toString(),
        type: 'command',
        text: `> ${trimmed}`,
        timestamp: Date.now(),
      },
    ];

    const command = trimmed.split(' ')[0].toLowerCase();

    if (!allowedCommands.includes(command)) {
      outputs = [
        ...outputs,
        {
          id: Date.now().toString(),
          type: 'error',
          text: `Unknown command: ${command}. Type "help" for available commands.`,
          timestamp: Date.now(),
        },
      ];
    } else {
      if (onCommand) {
        onCommand(trimmed);
      }

      let response = '';
      switch (command) {
        case 'help':
          response = `Available commands:\n  ${allowedCommands.join(', ')}\n\nType a command followed by Enter to execute.`;
          break;
        case 'status':
          response = 'System status: OK';
          break;
        case 'uptime':
          response = 'System uptime: 45 days, 12 hours, 30 minutes';
          break;
        case 'version':
          response = 'TSSP v2.0.0';
          break;
        default:
          response = `Executed: ${command}`;
      }

      outputs = [
        ...outputs,
        {
          id: Date.now().toString(),
          type: 'output',
          text: response,
          timestamp: Date.now(),
        },
      ];
    }

    input = '';
    setTimeout(() => {
      if (consoleEl) {
        consoleEl.scrollTop = consoleEl.scrollHeight;
      }
    }, 0);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      executeCommand(input);
    }
  }
</script>

{#if isOpen}
  <div class="safe-console {className || ''}" style="height: {height}px">
    <div class="console-header">
      <div>
        <h3>Safe Console</h3>
        <span class="console-info">Admin commands only</span>
      </div>
      {#if onClose}
        <button class="console-close" on:click={onClose} aria-label="Close">
          <Icons.ChevronDown size={16} />
        </button>
      {/if}
    </div>

    <div class="console-output" bind:this={consoleEl}>
      {#each outputs as output (output.id)}
        <div class="output-line" class:error={output.type === 'error'} class:command={output.type === 'command'} class:info={output.type === 'info'}>
          {#if output.type === 'command'}
            <span class="prompt">$</span>
          {:else if output.type === 'error'}
            <span class="error-icon">!</span>
          {:else if output.type === 'info'}
            <span class="info-icon">ℹ</span>
          {/if}
          <span class="output-text">{output.text}</span>
        </div>
      {/each}
    </div>

    <div class="console-input">
      <span class="prompt">$</span>
      <input
        type="text"
        bind:value={input}
        on:keydown={handleKeydown}
        placeholder="Type a command..."
        class="input-field"
      />
      <div class="input-hint">
        <Kbd>Enter</Kbd>
        <span class="hint-text">to execute</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .safe-console {
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border-top: 1px solid var(--border);
    font-family: var(--ff-mono);
    flex-shrink: 0;
  }

  .console-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-3) var(--s-4);
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
  }

  .console-header h3 {
    margin: 0;
    font-size: var(--fs-13);
    font-weight: 600;
    color: var(--text);
    font-family: var(--ff-sans);
  }

  .console-info {
    display: block;
    font-size: var(--fs-11);
    color: var(--muted);
    margin-top: 2px;
    font-family: var(--ff-sans);
  }

  .console-close {
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    border-radius: var(--r-1);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .console-close:hover {
    background: var(--surface);
    color: var(--text);
  }

  .console-output {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-3);
    background: var(--bg);
    font-size: var(--fs-12);
    line-height: var(--lh-relaxed);
    color: var(--text-2);
  }

  .output-line {
    display: flex;
    gap: var(--s-2);
    margin-bottom: 4px;
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .output-line.command {
    color: var(--green);
  }

  .output-line.error {
    color: var(--danger);
  }

  .output-line.info {
    color: var(--blue);
  }

  .prompt {
    color: var(--muted);
    flex-shrink: 0;
  }

  .error-icon,
  .info-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    flex-shrink: 0;
    font-weight: 600;
  }

  .output-text {
    flex: 1;
    word-wrap: break-word;
  }

  .console-input {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-3);
    border-top: 1px solid var(--border);
    background: var(--surface);
  }

  .input-field {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text);
    font-family: var(--ff-mono);
    font-size: var(--fs-12);
    outline: none;
  }

  .input-field::placeholder {
    color: var(--muted);
  }

  .input-hint {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
    font-size: var(--fs-10);
    color: var(--muted);
  }

  .hint-text {
    font-family: var(--ff-sans);
  }
</style>
