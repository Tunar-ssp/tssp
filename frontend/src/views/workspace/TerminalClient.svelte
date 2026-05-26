<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import { Terminal } from 'xterm';
  import { FitAddon } from 'xterm-addon-fit';
  import 'xterm/css/xterm.css';

  interface $$Props {
    workspaceId: string;
    isAvailable: boolean;
  }

  let {
    workspaceId,
    isAvailable,
  }: $$Props = $props();

  let terminalElement = $state<HTMLDivElement | null>(null);
  let terminal: Terminal;
  let fitAddon: FitAddon;
  let ws: WebSocket | null = null;
  let state = $state('disconnected');
  let errorMessage = $state<string | null>(null);

  onMount(() => {
    if (!isAvailable) return;

    terminal = new Terminal({
      theme: {
        background: '#0E1016',
        foreground: '#E0E0E0',
        cursor: '#E0E0E0',
        black: '#000000',
        red: '#FF5555',
        green: '#55FF55',
        yellow: '#FFFF55',
        blue: '#5555FF',
        magenta: '#FF55FF',
        cyan: '#55FFFF',
        white: '#FFFFFF',
        brightBlack: '#555555',
        brightRed: '#FF8888',
        brightGreen: '#88FF88',
        brightYellow: '#FFFF88',
        brightBlue: '#8888FF',
        brightMagenta: '#FF88FF',
        brightCyan: '#88FFFF',
        brightWhite: '#FFFFFF',
      },
      fontSize: 14,
      fontFamily: 'Courier New, monospace',
      cursorStyle: 'block',
      bellStyle: 'none',
    });

    fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(terminalElement);
    fitAddon.fit();

    connect();

    const resizeObserver = new ResizeObserver(() => {
      if (fitAddon) fitAddon.fit();
    });
    resizeObserver.observe(terminalElement);

    return () => {
      resizeObserver.disconnect();
      disconnect();
      terminal.dispose();
    };
  });

  onDestroy(() => {
    disconnect();
  });

  function connect() {
    if (state === 'connecting' || state === 'connected') return;
    if (!isAvailable) return;

    state = 'connecting';
    errorMessage = null;

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}/api/v1/workspaces/${workspaceId}/terminal/ws`;

    ws = new WebSocket(url);

    ws.onopen = () => {
      state = 'connected';
      terminal.write('\r\n✓ Terminal connected\r\n');
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);

        if (msg.type === 'connected') {
          terminal.write(`✓ Session ${msg.session_id.substring(0, 8)}... active\r\n`);
        } else if (msg.type === 'output') {
          const output = msg.data || '';
          terminal.write(output);
        } else if (msg.type === 'exit') {
          terminal.write(`\r\n✓ Process exited with code ${msg.code}\r\n`);
          state = 'disconnected';
          ws?.close();
        } else if (msg.error) {
          const errMsg = msg.error || 'Unknown error';
          terminal.write(`\r\n✗ Error: ${errMsg}\r\n`);
          state = 'error';
          errorMessage = errMsg;
          setTimeout(() => ws?.close(), 1000);
        }
      } catch (e) {
        terminal.write(`\r\n✗ Protocol error\r\n`);
        state = 'error';
        errorMessage = 'Protocol error';
      }
    };

    ws.onerror = () => {
      state = 'error';
      errorMessage = 'Connection failed';
      terminal.write('\r\n✗ Connection error\r\n');
    };

    ws.onclose = () => {
      if (state === 'connected') {
        state = 'disconnected';
        terminal.write('\r\n✓ Disconnected\r\n');
      }
      ws = null;
    };

    terminal.onData((data) => {
      if (state === 'connected' && ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ input: data }));
      }
    });
  }

  function disconnect() {
    if (ws) {
      ws.close();
      ws = null;
    }
    state = 'disconnected';
    errorMessage = null;
  }

  function reconnect() {
    disconnect();
    connect();
  }

  function kill() {
    disconnect();
  }
</script>

<div class="terminal-container">
  {#if !isAvailable}
    <div class="terminal-unavailable">
      <Icons.AlertTriangle size={20} />
      <p>Terminal is not available for this workspace.</p>
    </div>
  {:else if state === 'connecting'}
    <div class="terminal-status">
      <Icons.Loader size={20} class="spinner" />
      <p>Connecting...</p>
    </div>
  {:else if state === 'error'}
    <div class="terminal-status error">
      <Icons.AlertCircle size={20} />
      <div>
        <p>Connection Error</p>
        {#if errorMessage}
          <code>{errorMessage}</code>
        {/if}
        <button type="button" onclick={reconnect} class="reconnect-btn">
          Reconnect
        </button>
      </div>
    </div>
  {:else}
    <div class="terminal-wrapper">
      <div class="terminal-header">
        <div class="terminal-status-indicator" class:connected={state === 'connected'}>
          {#if state === 'connected'}
            <span class="status-dot"></span>
            Connected
          {:else}
            <span class="status-dot disconnected"></span>
            Disconnected
          {/if}
        </div>
        <div class="terminal-controls">
          {#if state === 'connected'}
            <button type="button" onclick={kill} class="terminal-button" title="Disconnect">
              <Icons.X size={16} />
            </button>
          {/if}
        </div>
      </div>
      <div bind:this={terminalElement} class="terminal-display"></div>
    </div>
  {/if}
</div>

<style>
  .terminal-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: #0E1016;
  }

  .terminal-unavailable,
  .terminal-status {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 24px;
    text-align: center;
    color: var(--muted);
  }

  .terminal-status.error {
    flex-direction: column;
    color: var(--red);
  }

  .terminal-status.error code {
    display: block;
    margin-top: 12px;
    padding: 12px;
    border-radius: 12px;
    background: rgba(255, 85, 85, 0.1);
    color: var(--red);
    font-family: var(--ff-mono);
    font-size: 12px;
    word-break: break-word;
  }

  .reconnect-btn {
    margin-top: 16px;
    padding: 8px 16px;
    border-radius: 12px;
    border: 1px solid var(--red);
    background: transparent;
    color: var(--red);
    cursor: pointer;
    font-size: 12px;
    transition: all 0.2s;
  }

  .reconnect-btn:hover {
    background: rgba(255, 85, 85, 0.1);
  }

  :global(.spinner) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .terminal-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .terminal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(18, 21, 29, 0.5);
  }

  .terminal-status-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--text-2);
  }

  .terminal-status-indicator.connected {
    color: var(--green);
  }

  .status-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--green);
    animation: pulse 2s ease-in-out infinite;
  }

  .status-dot.disconnected {
    background: var(--muted);
    animation: none;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .terminal-controls {
    display: flex;
    gap: 8px;
  }

  .terminal-button {
    padding: 6px;
    border-radius: 8px;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    transition: all 0.2s;
  }

  .terminal-button:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
  }

  .terminal-display {
    flex: 1;
    overflow: hidden;
    background: #0E1016;
  }

  :global(.terminal-display .xterm) {
    width: 100%;
    height: 100%;
  }
</style>
