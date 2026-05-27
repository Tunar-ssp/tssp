<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { type AdminSession } from '$lib/api';
  import Card from '$lib/components/Card.svelte';

  interface Props {
    sessions: AdminSession[];
    isLoading?: boolean;
  }

  let { sessions, isLoading = false }: Props = $props();

  function formatTime(timestamp: string): string {
    return new Date(timestamp).toLocaleString();
  }

  function getStatusColor(status: string): string {
    return status === 'active' ? 'var(--green)' : 'var(--muted)';
  }
</script>

<div class="admin-panel">
  {#if sessions.length === 0}
    <div class="empty-state">
      <Icons.Smartphone size={48} />
      <h3>No sessions</h3>
      <p>No active or recent sessions.</p>
    </div>
  {:else}
    <div class="sessions-list">
      {#each sessions as session (session.id)}
        <Card>
          <div class="session-item">
            <div class="session-info">
              <h4>{session.user_agent || 'Unknown Device'}</h4>
              <p class="session-meta">
                {#if session.ip_address}
                  <span>{session.ip_address}</span>
                {/if}
                <span>{formatTime(session.created_at)}</span>
              </p>
            </div>
            <div class="session-status">
              <span class="status-dot" style="background-color: {getStatusColor(session.status)}"></span>
              {session.status}
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>

<style>
  .admin-panel {
    flex: 1;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    padding: var(--s-8);
    text-align: center;
    color: var(--muted);
  }

  .empty-state h3 {
    margin: 0;
    color: var(--text-2);
  }

  .sessions-list {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .session-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-3);
    gap: var(--s-4);
  }

  .session-info h4 {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
    word-break: break-word;
  }

  .session-meta {
    margin: var(--s-1) 0 0;
    font-size: var(--fs-12);
    color: var(--muted);
    display: flex;
    gap: var(--s-2);
    flex-wrap: wrap;
  }

  .session-status {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    font-size: var(--fs-12);
    color: var(--text-2);
    white-space: nowrap;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
</style>
