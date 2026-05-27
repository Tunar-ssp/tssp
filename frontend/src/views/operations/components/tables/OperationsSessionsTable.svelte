<script lang="ts">
  import { formatRelative } from '$lib/utils';
  import type { AdminSession } from '$lib/api';

  interface Props {
    sessions?: AdminSession[];
  }

  let { sessions = [] }: Props = $props();
</script>

<div class="admin-content">
  <article class="panel table-panel">
    <div class="panel-head">
      <h2>Sessions</h2>
      <span class="panel-meta">{sessions.length} active</span>
    </div>
    <div class="table">
      <div class="table-head sessions">
        <span>User</span>
        <span>Kind</span>
        <span>Created</span>
        <span>Expires</span>
        <span>Token</span>
      </div>
      {#each sessions as session (session.token)}
        <div class="table-row sessions">
          <strong>{session.user_name || session.role || 'session'}</strong>
          <span>{session.kind}</span>
          <span>{formatRelative(session.created_at)}</span>
          <span>{formatRelative(session.expires_at)}</span>
          <span class="mono">{session.token_preview}</span>
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

  .table-head.sessions,
  .table-row.sessions {
    grid-template-columns: 1fr 0.8fr 0.8fr 0.8fr 0.8fr;
  }

  .table-row strong {
    color: var(--text);
  }

  .mono {
    font-family: var(--ff-mono);
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
    .table-head.sessions,
    .table-row.sessions {
      grid-template-columns: 1fr 0.6fr 0.6fr 0.6fr 0.6fr;
      font-size: 11px;
      gap: 8px;
    }
  }
</style>
