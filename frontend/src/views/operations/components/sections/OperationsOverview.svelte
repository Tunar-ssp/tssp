<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { formatBytes, formatRelative } from '$lib/utils';
  import type { AdminActivityItem } from '$lib/api';

  interface Props {
    overview?: any;
    status?: any;
    users?: Array<any>;
    sessions?: Array<any>;
    devices?: Array<any>;
    publicFiles?: Array<any>;
    activityItems?: AdminActivityItem[];
    primaryCommands?: Array<any>;
    onSectionChange?: (section: string) => void;
    onRunCommand?: (name: string) => void;
    executing?: boolean;
  }

  let {
    overview,
    status,
    users = [],
    sessions = [],
    devices = [],
    publicFiles = [],
    activityItems = [],
    primaryCommands = [],
    onSectionChange,
    onRunCommand,
    executing = false,
  }: Props = $props();

  function healthTone() {
    const statusValue = status?.status?.toLowerCase() || '';
    if (statusValue.includes('ok') || statusValue.includes('healthy') || statusValue.includes('ready')) {
      return 'good';
    }
    if (statusValue.includes('warn') || statusValue.includes('degraded')) {
      return 'warn';
    }
    return 'neutral';
  }

  function healthLabel() {
    return status?.status || 'Status unavailable';
  }

  let memoryPercent = $derived(
    status?.memory_total ? Math.round((status.memory_used / status.memory_total) * 100) : 0
  );
</script>

<div class="admin-content">
  <div class="overview-hero">
    <article class="panel hero-card hero-primary">
      <div class="panel-head hero-head">
        <div>
          <h2>System posture</h2>
          <p class="panel-copy">Real machine health, repository volume, and persistence status.</p>
        </div>
        <span class={`health-pill ${healthTone()}`}>
          <span class="health-dot"></span>
          {healthLabel()}
        </span>
      </div>

      <div class="metrics-grid">
        <article class="metric-card">
          <span>Repository files</span>
          <strong>{overview?.file_count ?? 0}</strong>
          <p>{formatBytes(overview?.storage_bytes_used ?? 0)} in use</p>
        </article>
        <article class="metric-card">
          <span>Notes</span>
          <strong>{overview?.note_count ?? 0}</strong>
          <p>Knowledge base entries</p>
        </article>
        <article class="metric-card">
          <span>Load</span>
          <strong>{(status?.load_average ?? 0).toFixed(2)}</strong>
          <p>1m system load average</p>
        </article>
        <article class="metric-card">
          <span>Memory</span>
          <strong>{memoryPercent}%</strong>
          <p>{formatBytes(status?.memory_used ?? 0)} of {formatBytes(status?.memory_total ?? 0)}</p>
        </article>
      </div>
    </article>

    <article class="panel hero-card hero-secondary">
      <div class="panel-head">
        <h2>Operational lanes</h2>
        <span class="panel-meta">Jump directly into active areas</span>
      </div>
      <div class="snapshot-list">
        <button type="button" class="snapshot-card" onclick={() => onSectionChange?.('users')}>
          <div>
            <strong>Users</strong>
            <p>Access inventory and roles</p>
          </div>
          <span>{users.length}</span>
        </button>
        <button type="button" class="snapshot-card" onclick={() => onSectionChange?.('sessions')}>
          <div>
            <strong>Sessions</strong>
            <p>Browser and CLI presence</p>
          </div>
          <span>{sessions.length}</span>
        </button>
        <button type="button" class="snapshot-card" onclick={() => onSectionChange?.('devices')}>
          <div>
            <strong>Trusted devices</strong>
            <p>Remembered device access</p>
          </div>
          <span>{devices.length}</span>
        </button>
        <button type="button" class="snapshot-card" onclick={() => onSectionChange?.('public')}>
          <div>
            <strong>Public links</strong>
            <p>Externally visible objects</p>
          </div>
          <span>{publicFiles.length}</span>
        </button>
      </div>
    </article>
  </div>

  <div class="split-grid">
    <article class="panel">
      <div class="panel-head">
        <h2>System posture</h2>
      </div>
      <div class="stat-list">
        <div class="stat-row"><span>Status</span><strong>{status?.status || 'unknown'}</strong></div>
        <div class="stat-row"><span>Disk</span><strong>{formatBytes(status?.disk_used || 0)} / {formatBytes(status?.disk_total || 0)}</strong></div>
        <div class="stat-row"><span>Memory</span><strong>{formatBytes(status?.memory_used || 0)} / {formatBytes(status?.memory_total || 0)}</strong></div>
        <div class="stat-row"><span>Database</span><strong>{formatBytes(status?.db_size || 0)} · {status?.db_status || 'unknown'}</strong></div>
        <div class="stat-row"><span>Uptime</span><strong>{status?.uptime_hours || 0}h</strong></div>
      </div>
    </article>

    <article class="panel">
      <div class="panel-head">
        <h2>Recent activity</h2>
      </div>
      <div class="activity-list">
        {#each activityItems.slice(0, 6) as item (item.id)}
          <div class="activity-row">
            <div class="activity-dot"></div>
            <div>
              <strong>{item.title}</strong>
              <p>{item.detail}</p>
            </div>
            <span>{formatRelative(item.occurred_at)}</span>
          </div>
        {/each}
      </div>
    </article>

    <article class="panel">
      <div class="panel-head">
        <h2>Maintenance queue</h2>
        <span class="panel-meta">{primaryCommands.length} safe actions</span>
      </div>
      <div class="command-grid">
        {#each primaryCommands as command (command.id)}
          <button type="button" class="command-card" onclick={() => onRunCommand?.(command.name)} disabled={executing}>
            <div>
              <strong>{command.name}</strong>
              <p>{command.description || 'Safe backend maintenance command'}</p>
            </div>
            <Icons.ChevronRight size={16} />
          </button>
        {/each}
      </div>
    </article>
  </div>
</div>

<style>
  .admin-content {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .overview-hero {
    display: grid;
    grid-template-columns: minmax(0, 1.6fr) minmax(280px, 0.9fr);
    gap: 14px;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 12px;
  }

  .metric-card,
  .panel {
    border: 1px solid var(--border);
    background: linear-gradient(180deg, rgba(20, 22, 29, 0.95), rgba(14, 15, 21, 0.93));
    box-shadow: var(--shadow-card);
    border-radius: 18px;
    padding: 16px;
  }

  .hero-card {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .hero-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .panel-copy {
    margin: 8px 0 0;
    color: var(--muted);
    font-size: 13px;
  }

  .metric-card span {
    font-size: 11px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--muted);
    font-family: var(--ff-mono);
  }

  .metric-card strong {
    margin-top: 10px;
    font-size: 28px;
    color: var(--text);
  }

  .metric-card p {
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 12px;
  }

  .health-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 32px;
    padding: 0 12px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.03);
    color: var(--text-2);
    font-size: 12px;
    font-weight: 500;
  }

  .health-pill.good {
    border-color: rgba(91, 227, 154, 0.22);
    background: rgba(91, 227, 154, 0.1);
    color: var(--green);
  }

  .health-pill.warn {
    border-color: rgba(251, 191, 36, 0.24);
    background: rgba(251, 191, 36, 0.1);
    color: var(--warning);
  }

  .health-dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: currentColor;
    box-shadow: 0 0 10px currentColor;
  }

  .split-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 14px;
  }

  .snapshot-list {
    display: grid;
    gap: 10px;
  }

  .snapshot-card {
    width: 100%;
    padding: 14px;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.02);
    color: inherit;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 14px;
    align-items: center;
    text-align: left;
    cursor: pointer;
    transition: transform var(--duration-quick) var(--ease-smooth), border-color var(--duration-quick) var(--ease-smooth), background var(--duration-quick) var(--ease-smooth);
    font-family: inherit;
  }

  .snapshot-card:hover {
    transform: translateY(-1px);
    border-color: var(--border-2);
    background: rgba(255, 255, 255, 0.04);
  }

  .snapshot-card p {
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 12px;
  }

  .snapshot-card span {
    min-width: 34px;
    height: 34px;
    padding: 0 10px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: rgba(163, 148, 255, 0.12);
    color: var(--violet);
    font-family: var(--ff-mono);
    font-size: 12px;
    font-weight: 600;
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

  .panel-meta {
    color: var(--muted);
    font-size: 12px;
    margin-left: auto;
  }

  .stat-list,
  .activity-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .stat-row,
  .activity-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 0;
    border-bottom: 1px solid var(--hairline);
  }

  .stat-row:last-child,
  .activity-row:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .stat-row span,
  .activity-row p {
    color: var(--muted);
    font-size: 12px;
  }

  .stat-row strong,
  .activity-row strong {
    color: var(--text);
  }

  .activity-dot {
    width: 10px;
    height: 10px;
    border-radius: 999px;
    background: var(--violet);
    margin-top: 5px;
    box-shadow: 0 0 12px rgba(163, 148, 255, 0.45);
    flex-shrink: 0;
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

  .command-card strong {
    color: var(--text);
  }

  .command-card p {
    color: var(--muted);
    font-size: 12px;
    margin: 0;
  }

  @media (max-width: 1180px) {
    .overview-hero,
    .split-grid {
      grid-template-columns: 1fr;
    }

    .metrics-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 760px) {
    .metrics-grid {
      grid-template-columns: 1fr;
    }

    .hero-head {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>
