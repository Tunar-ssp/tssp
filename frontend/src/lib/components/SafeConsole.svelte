<script lang="ts">
  import * as Icons from 'lucide-svelte';

  let commands: any[] = [];
  let selectedCommand: string = '';
  let output: string = '';
  let running = false;
  let loading = true;

  async function loadCommands() {
    try {
      const res = await fetch('/api/v1/admin/console/commands', {
        credentials: 'same-origin',
      });
      if (res.ok) {
        const data = await res.json();
        commands = data.commands || [];
      }
    } finally {
      loading = false;
    }
  }

  async function runCommand() {
    if (!selectedCommand || running) return;
    running = true;
    output = 'Running...\n';

    try {
      const res = await fetch('/api/v1/admin/console/run', {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command_id: selectedCommand }),
      });

      if (res.ok) {
        const data = await res.json();
        output = data.output || 'Command completed';
      } else {
        output = 'Error: ' + res.statusText;
      }
    } catch (err: any) {
      output = 'Error: ' + (err.message || 'Failed to run command');
    } finally {
      running = false;
    }
  }

  const command = commands.find(c => c.id === selectedCommand);

  onMount(loadCommands);

  import { onMount } from 'svelte';
</script>

<div class="safe-console">
  <div class="console-header">
    <h3>Safe Console</h3>
    <p class="subtitle">Run whitelisted diagnostic commands</p>
  </div>

  {#if loading}
    <div class="loading">
      <div class="spinner" />
      Loading commands...
    </div>
  {:else}
    <div class="console-content">
      <div class="command-selector">
        <label>Select Command:</label>
        <select bind:value={selectedCommand}>
          <option value="">Choose a command...</option>
          {#each commands as cmd (cmd.id)}
            <option value={cmd.id}>
              {cmd.label}
            </option>
          {/each}
        </select>

        {#if command}
          <div class="command-info">
            <p class="info-label">Description:</p>
            <p class="info-text">{command.description}</p>
            <p class="info-label">Command:</p>
            <p class="info-text mono">{command.command}</p>
          </div>
        {/if}

        <button
          class="run-btn"
          disabled={!selectedCommand || running}
          on:click={runCommand}
        >
          {#if running}
            <div class="spinner-small" />
            Running...
          {:else}
            <Icons.Play size={14} />
            Run Command
          {/if}
        </button>
      </div>

      {#if output}
        <div class="output-panel">
          <div class="output-header">Output:</div>
          <pre class="output-text">{output}</pre>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .safe-console {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 20px;
  }

  .console-header h3 {
    margin: 0;
    font-size: var(--fs-20);
    color: var(--text);
  }

  .subtitle {
    margin: 4px 0 0;
    font-size: var(--fs-12);
    color: var(--text-2);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--muted);
    flex: 1;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .spinner-small {
    width: 12px;
    height: 12px;
    border: 1px solid var(--surface-3);
    border-top-color: #0a1228;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .console-content {
    display: flex;
    flex-direction: column;
    gap: 16px;
    flex: 1;
    overflow: hidden;
  }

  .command-selector {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .command-selector label {
    font-size: var(--fs-12);
    font-weight: 500;
    color: var(--text);
  }

  .command-selector select {
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface);
    color: var(--text);
    font-size: var(--fs-13);
    outline: none;
  }

  .command-selector select:focus {
    border-color: var(--blue);
  }

  .command-info {
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface-2);
  }

  .info-label {
    margin: 0 0 4px;
    font-size: 11px;
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
  }

  .info-text {
    margin: 0 0 8px;
    font-size: var(--fs-12);
    color: var(--text-2);
  }

  .info-text.mono {
    font-family: var(--ff-mono);
    background: var(--bg);
    padding: 6px 8px;
    border-radius: 4px;
    color: var(--green);
  }

  .run-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 8px 16px;
    border-radius: var(--r-2);
    border: 1px solid var(--border);
    background: var(--blue);
    color: #0a1228;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }

  .run-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .run-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .output-panel {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--bg);
  }

  .output-header {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    font-size: var(--fs-12);
    font-weight: 500;
    color: var(--text-2);
  }

  .output-text {
    flex: 1;
    margin: 0;
    padding: 12px;
    font-family: var(--ff-mono);
    font-size: var(--fs-12);
    color: var(--green);
    overflow: auto;
    white-space: pre-wrap;
    word-wrap: break-word;
  }
</style>
