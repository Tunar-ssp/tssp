<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import StatCard from '$lib/components/StatCard.svelte';
  import Bar from '$lib/components/Bar.svelte';
  import StatusDot from '$lib/components/StatusDot.svelte';

  interface Props {
    fileCount: number;
    noteCount: number;
    workspaceCount: number;
    usedStorage: number;
    uptime: number;
    corruptFileCount: number;
  }

  let { fileCount, noteCount, workspaceCount, usedStorage, uptime, corruptFileCount }: Props =
    $props();

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }

  function formatUptime(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (hours > 24) {
      const days = Math.floor(hours / 24);
      return `${days}d ${hours % 24}h`;
    }
    return `${hours}h ${minutes}m`;
  }
</script>

<div class="stats-grid">
  <StatCard title="Total Files" value={fileCount} icon={Icons.Files} meta="{noteCount} notes · {workspaceCount} workspaces" />
  <StatCard title="Storage" value={formatBytes(usedStorage)} icon={Icons.HardDrive}>
    <Bar value={usedStorage > 0 ? 100 : 0} />
    <div class="stat-meta-bottom">Indexed object storage</div>
  </StatCard>
  <StatCard title="Uptime" value={formatUptime(uptime)} icon={Icons.Zap}>
    <div style="display: flex; align-items: center; gap: var(--s-2);">
      <StatusDot tone="ok" />
      <span style="font-size: var(--fs-12); color: var(--muted);">System healthy</span>
    </div>
  </StatCard>
  <StatCard title="Database" value={corruptFileCount} icon={Icons.Database} meta="Corrupt indexed blobs" />
</div>

<style>
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: var(--s-4);
    margin-bottom: var(--s-6);
  }

  :global(.stat-meta-bottom) {
    font-size: var(--fs-12);
    color: var(--muted);
  }
</style>
