<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Activity {
    id: string;
    type: 'file' | 'note' | 'workspace';
    action: string;
    title: string;
    timestamp: number;
  }

  interface $$Props {
    activities: Activity[];
  }

  let { activities = [] }: $$Props = $props();

  function formatTime(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const hours = diff / (1000 * 60 * 60);
    const days = diff / (1000 * 60 * 60 * 24);

    if (hours < 1) return 'just now';
    if (hours < 24) return `${Math.floor(hours)}h ago`;
    if (days < 7) return `${Math.floor(days)}d ago`;
    return date.toLocaleDateString();
  }

  function getActivityIcon(type: string): string {
    switch (type) {
      case 'file':
        return '📎';
      case 'note':
        return '📋';
      case 'workspace':
        return '💻';
      default:
        return '📌';
    }
  }
</script>

<section class="launcher-activity">
  <h3>Recent Activity</h3>
  {#if activities.length === 0}
    <div class="activity-empty">
      <Icons.History size={24} />
      <p>No activity yet</p>
    </div>
  {:else}
    <div class="activity-list">
      {#each activities.slice(0, 8) as activity (activity.id)}
        <div class="activity-item">
          <span class="activity-icon">{getActivityIcon(activity.type)}</span>
          <div class="activity-content">
            <span class="activity-action">{activity.action}</span>
            <span class="activity-title">{activity.title}</span>
          </div>
          <span class="activity-time">{formatTime(activity.timestamp)}</span>
        </div>
      {/each}
    </div>
  {/if}
</section>

<style>
  .launcher-activity {
    padding: 28px;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(18, 21, 29, 0.96);
  }

  .launcher-activity h3 {
    margin: 0 0 20px;
    font-size: 18px;
    font-weight: 600;
    color: var(--text);
  }

  .activity-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 160px;
    color: var(--muted);
    gap: 12px;
  }

  .activity-empty p {
    margin: 0;
    font-size: 14px;
  }

  .activity-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .activity-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-radius: 8px;
    border: 1px solid rgba(110, 168, 255, 0.1);
    background: rgba(110, 168, 255, 0.02);
    font-size: 13px;
  }

  .activity-icon {
    font-size: 18px;
    flex-shrink: 0;
  }

  .activity-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .activity-action {
    color: var(--text);
    font-weight: 500;
  }

  .activity-title {
    color: var(--muted);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .activity-time {
    color: var(--muted);
    font-size: 12px;
    white-space: nowrap;
  }
</style>
