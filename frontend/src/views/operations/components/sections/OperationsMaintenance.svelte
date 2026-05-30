<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Command {
    id: string;
    name: string;
    description?: string;
  }

  interface Props {
    commands?: Command[];
    commandOutput?: string;
    executing?: boolean;
    onRunCommand?: (name: string) => void;
  }

  let { commands = [], commandOutput = '', executing = false, onRunCommand }: Props = $props();

  import { api } from '$lib/api';
  import { success, error } from '$lib/stores/notifications';
  import { promptDialog } from '$lib/stores/dialog';

  let localExecuting = $state(false);
  let localOutput = $state('');

  async function handleVacuum() {
    localExecuting = true;
    localOutput = 'Compacting database...';
    try {
      const res = await api.vacuumDatabase();
      localOutput = res.message;
      success('Database Vacuumed', 'Optimization complete');
    } catch (err) {
      localOutput = err instanceof Error ? err.message : 'Vacuum failed';
      error('Vacuum Failed', localOutput);
    } finally {
      localExecuting = false;
    }
  }

  async function handlePruneLogs() {
    const days = await promptDialog({
      title: 'Prune audit logs',
      message: 'Remove log entries older than the given number of days.',
      placeholder: 'Days',
      defaultValue: '30',
      confirmLabel: 'Prune',
      tone: 'danger',
    });
    if (!days) return;
    const d = parseInt(days, 10);
    if (isNaN(d) || d < 0) {
      error('Invalid Value', 'Enter a valid number of days');
      return;
    }

    localExecuting = true;
    localOutput = `Pruning logs older than ${d} days...`;
    try {
      const res = await api.pruneAuditLogs(d);
      localOutput = `Successfully removed ${res.removed_count} log entries.`;
      success('Logs Pruned', localOutput);
    } catch (err) {
      localOutput = err instanceof Error ? err.message : 'Prune failed';
      error('Prune Failed', localOutput);
    } finally {
      localExecuting = false;
    }
  }
</script>

<div class="admin-content">
  <div class="split-grid">
    <article class="panel">
      <div class="panel-head">
        <h2>Maintenance actions</h2>
      </div>
      <div class="command-grid">
        <button type="button" class="command-card" onclick={handleVacuum} disabled={executing || localExecuting}>
          <div>
            <strong>Vacuum database</strong>
            <p>Reclaim unused space and optimize SQLite metadata.</p>
          </div>
          <Icons.Zap size={16} />
        </button>
        <button type="button" class="command-card" onclick={handlePruneLogs} disabled={executing || localExecuting}>
          <div>
            <strong>Prune audit logs</strong>
            <p>Remove old activity history to save storage.</p>
          </div>
          <Icons.Trash2 size={16} />
        </button>
        {#each commands as command (command.id)}
          <button type="button" class="command-card" onclick={() => onRunCommand?.(command.name)} disabled={executing || localExecuting}>
            <div>
              <strong>{command.name}</strong>
              <p>{command.description || 'Safe backend maintenance command'}</p>
            </div>
            <Icons.ChevronRight size={16} />
          </button>
        {/each}
      </div>
    </article>

    <article class="panel">
      <div class="panel-head">
        <h2>Command output</h2>
      </div>
      <pre class="command-output">{localOutput || commandOutput || 'Run a maintenance command to inspect output here.'}</pre>
    </article>
  </div>
</div>

<style>
  .admin-content {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .split-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 14px;
  }

  .panel {
    border: 1px solid var(--border);
    background: linear-gradient(180deg, rgba(20, 22, 29, 0.95), rgba(14, 15, 21, 0.93));
    box-shadow: var(--shadow-card);
    border-radius: 18px;
    padding: 16px;
  }

  .panel-head {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 14px;
  }

  .panel-head h2 {
    margin: 0;
    color: var(--text);
    font-size: 16px;
  }

  .command-grid {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .command-card {
    width: 100%;
    padding: 14px;
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--surface);
    color: inherit;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    text-align: left;
    cursor: pointer;
    font-family: inherit;
  }

  .command-card:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .command-card strong {
    color: var(--text);
    display: block;
    margin-bottom: 2px;
  }

  .command-card p {
    color: var(--muted);
    font-size: 12px;
    margin: 0;
  }

  .command-output {
    min-height: 220px;
    margin: 0;
    padding: 14px;
    border-radius: 14px;
    background: rgba(7, 8, 12, 0.92);
    border: 1px solid var(--hairline);
    color: var(--text-2);
    white-space: pre-wrap;
    font-family: var(--ff-mono);
    font-size: 12px;
    word-break: break-word;
    overflow-y: auto;
    max-height: 400px;
  }

  @media (max-width: 1180px) {
    .split-grid {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 760px) {
    .panel {
      padding: 12px;
    }

    .panel-head {
      margin-bottom: 10px;
    }

    .command-card {
      padding: 12px;
    }

    .command-output {
      min-height: 180px;
      padding: 12px;
      font-size: 11px;
    }
  }
</style>
