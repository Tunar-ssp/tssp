<script lang="ts">
  import { formatRelative } from '$lib/utils';
  import type { AdminActivityItem } from '$lib/api';

  interface Props {
    items?: AdminActivityItem[];
  }

  let { items = [] }: Props = $props();
</script>

<div class="admin-content">
  <article class="panel table-panel">
    <div class="panel-head">
      <h2>Audit events</h2>
      <span class="panel-meta">{items.length} recent items</span>
    </div>
    <div class="activity-log">
      {#each items as item, i (`${item.id}-${item.occurred_at}-${i}`)}
        <div class="activity-card">
          <div>
            <strong>{item.title}</strong>
            <p>{item.detail}</p>
          </div>
          <span>{formatRelative(item.occurred_at)}</span>
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
    padding: 16px;
  }

  .table-panel {
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

  .panel-meta {
    color: var(--muted);
    font-size: 12px;
    margin-left: auto;
  }

  .activity-log {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .activity-card {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 0;
    border-bottom: 1px solid var(--hairline);
    color: var(--text-2);
    font-size: 13px;
  }

  .activity-card:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .activity-card strong {
    color: var(--text);
    display: block;
    margin-bottom: 2px;
  }

  .activity-card p {
    color: var(--muted);
    font-size: 12px;
    margin: 0;
  }

  .activity-card span {
    color: var(--muted);
    font-size: 12px;
    flex-shrink: 0;
  }

  @media (max-width: 760px) {
    .panel {
      padding: 12px;
    }

    .activity-card {
      padding: 10px 0;
      gap: 8px;
    }

    .activity-card p,
    .activity-card span {
      font-size: 11px;
    }
  }
</style>
