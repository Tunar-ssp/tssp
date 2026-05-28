<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { formatBytes } from '$lib/utils/formatters';
  import { api } from '$lib/api';

  type Status = Awaited<ReturnType<typeof api.getStatus>>;
  type AdminSystem = Awaited<ReturnType<typeof api.getAdminSystem>>;

  interface Props {
    status?: Status;
    system?: AdminSystem;
  }

  let { status, system }: Props = $props();

  const percent = (used: number, total: number) =>
    total > 0 ? Math.round((used / total) * 100) : 0;

  let memoryPercent = $derived(
    system ? percent(system.total_memory_bytes - system.available_memory_bytes, system.total_memory_bytes) : null
  );
  let diskPercent = $derived(
    system ? percent(system.data_dir_total_bytes - system.data_dir_free_bytes, system.data_dir_total_bytes) : null
  );
  let loadAverage = $derived(system ? system.load_average_1m.toFixed(2) : null);

  const formatUptime = (seconds = 0) => {
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    if (hours < 48) return `${hours}h ${minutes % 60}m`;
    return `${Math.floor(hours / 24)}d ${hours % 24}h`;
  };
</script>

<div class="status-stack">
  <article class="status-card">
    <div class="status-head">
      <div class="status-pill">
        <span class="status-dot"></span>
        <span>orange-pi</span>
      </div>
      <span class="status-version">{status?.version || 'tssp'}</span>
      <span class="status-uptime">up {formatUptime(status?.uptime_seconds || 0)}</span>
    </div>

    <div class="status-rings">
      <div class="ring green">
        <strong>{loadAverage ?? '—'}</strong>
        <span>Load</span>
      </div>
      <div class="ring blue">
        <strong>{memoryPercent !== null ? `${memoryPercent}%` : '—'}</strong>
        <span>Memory</span>
      </div>
      <div class="ring orange">
        <strong>{diskPercent !== null ? `${diskPercent}%` : '—'}</strong>
        <span>Disk</span>
      </div>
    </div>

    <div class="status-foot">
      <div class="status-line">
        <Icons.Globe size={13} />
        <span>{status?.public_url || 'LAN only'}</span>
      </div>
      <div class="status-line">
        <Icons.Activity size={13} />
        <span>{status?.recent_upload_count_24h || 0} uploads in 24h</span>
      </div>
    </div>
  </article>

  <div class="mini-grid">
    <article class="mini-card">
      <div class="mini-label">Storage</div>
      <div class="mini-value">{formatBytes(status?.storage_bytes_used || 0)}</div>
      <div class="mini-sub">{status?.file_count || 0} files tracked</div>
    </article>
    <article class="mini-card">
      <div class="mini-label">Pinned</div>
      <div class="mini-value">{status?.pinned_count || 0}</div>
      <div class="mini-sub">{status?.tag_count || 0} tags in use</div>
    </article>
  </div>
</div>

<style>
  .status-stack {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .status-card,
  .mini-card {
    border: 1px solid rgba(255, 255, 255, 0.07);
    background: linear-gradient(180deg, rgba(20, 22, 29, 0.94), rgba(15, 16, 22, 0.92));
    box-shadow: var(--shadow-card);
  }

  .status-card {
    border-radius: 20px;
    padding: 18px;
  }

  .status-head,
  .status-foot,
  .status-line {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-head {
    margin-bottom: 18px;
  }

  .status-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 28px;
    padding: 0 12px;
    border-radius: 999px;
    background: rgba(17, 20, 27, 0.9);
    border: 1px solid var(--border);
    font-size: 12px;
    color: var(--text);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: var(--success);
    box-shadow: 0 0 14px rgba(52, 211, 153, 0.7);
  }

  .status-version,
  .status-uptime,
  .status-line {
    font-size: 12px;
    color: var(--muted);
  }

  .status-version {
    margin-left: auto;
  }

  .status-rings {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
    margin-bottom: 16px;
  }

  .ring {
    min-height: 98px;
    border-radius: 18px;
    border: 1px solid var(--border);
    background:
      radial-gradient(circle at top, rgba(255, 255, 255, 0.06), transparent 58%),
      var(--surface);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    gap: 4px;
  }

  .ring strong {
    font-size: 20px;
    color: var(--text);
  }

  .ring span {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.14em;
  }

  .ring.green strong {
    color: var(--green);
  }

  .ring.blue strong {
    color: var(--blue);
  }

  .ring.orange strong {
    color: var(--orange);
  }

  .status-foot {
    justify-content: space-between;
    gap: 14px;
    flex-wrap: wrap;
    padding-top: 14px;
    border-top: 1px dashed var(--hairline);
  }

  .mini-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .mini-card {
    border-radius: 16px;
    padding: 14px;
  }

  .mini-label {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.14em;
  }

  .mini-value {
    margin-top: 8px;
    font-size: 22px;
    font-weight: 600;
    color: var(--text);
  }

  .mini-sub {
    margin-top: 4px;
    font-size: 12px;
    color: var(--muted);
  }
</style>
