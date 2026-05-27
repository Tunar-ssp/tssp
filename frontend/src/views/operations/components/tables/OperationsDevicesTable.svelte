<script lang="ts">
  import { formatRelative } from '$lib/utils';

  interface Device {
    id: string;
    token: string;
    trusted_at?: number;
  }

  interface Props {
    devices?: Device[];
    onRevoke?: (token: string) => void;
  }

  let { devices = [], onRevoke }: Props = $props();
</script>

<div class="admin-content">
  <article class="panel table-panel">
    <div class="panel-head">
      <h2>Trusted devices</h2>
      <span class="panel-meta">{devices.length} enrolled</span>
    </div>
    <div class="table">
      <div class="table-head devices">
        <span>Device</span>
        <span>Token</span>
        <span>Trusted</span>
        <span></span>
      </div>
      {#each devices as device (device.token)}
        <div class="table-row devices">
          <strong>{device.id}</strong>
          <span class="mono">{device.token}</span>
          <span>{formatRelative(device.trusted_at)}</span>
          <button type="button" class="inline-action" onclick={() => onRevoke?.(device.token)}>Revoke</button>
        </div>
      {/each}
    </div>
  </article>
</div>

<style>
  .admin-content {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .panel {
    border: 1px solid var(--border);
    background: linear-gradient(180deg, rgba(20, 22, 29, 0.95), rgba(14, 15, 21, 0.93));
    box-shadow: var(--shadow-card);
    border-radius: 18px;
  }

  .table-panel {
    padding: 0;
    overflow: hidden;
  }

  .table {
    display: flex;
    flex-direction: column;
  }

  .table-head,
  .table-row {
    display: grid;
    gap: 12px;
    align-items: center;
    padding: 12px 16px;
  }

  .table-head {
    border-top: 1px solid var(--hairline);
    border-bottom: 1px solid var(--hairline);
    font-size: 10px;
    color: var(--dim);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    font-family: var(--ff-mono);
  }

  .table-row {
    border-bottom: 1px solid var(--hairline);
    color: var(--text-2);
    font-size: 13px;
  }

  .table-row:last-child {
    border-bottom: none;
  }

  .table-head.devices,
  .table-row.devices {
    grid-template-columns: 1fr 1.2fr 0.8fr 120px;
  }

  .table-row strong {
    color: var(--text);
  }

  .mono {
    font-family: var(--ff-mono);
  }

  .inline-action {
    height: 30px;
    padding: 0 10px;
    border: 1px solid rgba(255, 107, 107, 0.18);
    border-radius: 10px;
    background: rgba(255, 107, 107, 0.08);
    color: var(--danger);
    cursor: pointer;
    font-family: inherit;
  }

  .panel-head {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 0;
    padding: 16px 16px 0;
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

  @media (max-width: 760px) {
    .table-head.devices,
    .table-row.devices {
      grid-template-columns: 1fr 0.8fr 0.6fr auto;
      font-size: 11px;
      gap: 8px;
    }

    .inline-action {
      height: 28px;
      padding: 0 8px;
      font-size: 11px;
    }
  }
</style>
