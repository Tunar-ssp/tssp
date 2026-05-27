<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { type AdminActivityItem } from '$lib/api';
  import Card from '$lib/components/Card.svelte';

  interface Props {
    activities: AdminActivityItem[];
    isLoading?: boolean;
  }

  let { activities, isLoading = false }: Props = $props();

  function formatTime(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  }

  function getActivityIcon(action: string) {
    const iconMap: Record<string, any> = {
      create: Icons.Plus,
      update: Icons.Edit,
      delete: Icons.Trash2,
      upload: Icons.Upload,
      download: Icons.Download,
      share: Icons.Share2,
    };
    return iconMap[action] || Icons.Activity;
  }

  function getActivityColor(action: string): string {
    if (action === 'delete') return 'var(--danger)';
    if (action === 'create' || action === 'upload') return 'var(--green)';
    if (action === 'share') return 'var(--blue)';
    return 'var(--text-2)';
  }
</script>

<div class="admin-panel">
  {#if activities.length === 0}
    <div class="empty-state">
      <Icons.Activity size={48} />
      <h3>No activity</h3>
      <p>No recent activity.</p>
    </div>
  {:else}
    <div class="activity-list">
      {#each activities as activity (activity.id)}
        <Card>
          <div class="activity-item">
            <div class="activity-icon" style="color: {getActivityColor(activity.action)}">
              <svelte:component this={getActivityIcon(activity.action)} size={20} />
            </div>
            <div class="activity-content">
              <h4>{activity.action}</h4>
              <p class="activity-details">
                {#if activity.user_id}
                  <span>By {activity.user_id}</span>
                {/if}
                {#if activity.target}
                  <span>{activity.target}</span>
                {/if}
              </p>
            </div>
            <time class="activity-time">{formatTime(activity.timestamp)}</time>
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

  .activity-list {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .activity-item {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-3);
  }

  .activity-icon {
    flex-shrink: 0;
  }

  .activity-content {
    flex: 1;
    min-width: 0;
  }

  .activity-content h4 {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
    text-transform: capitalize;
  }

  .activity-details {
    margin: var(--s-1) 0 0;
    font-size: var(--fs-12);
    color: var(--muted);
    display: flex;
    gap: var(--s-2);
    flex-wrap: wrap;
  }

  .activity-time {
    font-size: var(--fs-12);
    color: var(--muted);
    white-space: nowrap;
  }
</style>
