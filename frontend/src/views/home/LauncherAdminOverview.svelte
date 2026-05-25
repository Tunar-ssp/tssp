<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Stats {
    totalUsers?: number;
    totalFiles?: number;
    totalNotes?: number;
    totalWorkspaces?: number;
  }

  interface $$Props {
    stats: Stats;
  }

  let { stats }: $$Props = $props();
</script>

<section class="admin-overview">
  <h3>Admin Overview</h3>
  <div class="stats-grid">
    <div class="stat-card">
      <Icons.Users size={20} />
      <div>
        <span class="stat-label">Total Users</span>
        <span class="stat-value">{stats.totalUsers || 0}</span>
      </div>
    </div>
    <div class="stat-card">
      <Icons.FileStack size={20} />
      <div>
        <span class="stat-label">Total Files</span>
        <span class="stat-value">{stats.totalFiles || 0}</span>
      </div>
    </div>
    <div class="stat-card">
      <Icons.BookOpen size={20} />
      <div>
        <span class="stat-label">Total Notes</span>
        <span class="stat-value">{stats.totalNotes || 0}</span>
      </div>
    </div>
    <div class="stat-card">
      <Icons.Code size={20} />
      <div>
        <span class="stat-label">Total Workspaces</span>
        <span class="stat-value">{stats.totalWorkspaces || 0}</span>
      </div>
    </div>
  </div>
</section>

<style>
  .admin-overview {
    padding: 28px;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(110, 168, 255, 0.06);
  }

  .admin-overview h3 {
    margin: 0 0 20px;
    font-size: 18px;
    font-weight: 600;
    color: var(--text);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 16px;
  }

  .stat-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    border-radius: 12px;
    border: 1px solid rgba(110, 168, 255, 0.2);
    background: rgba(18, 21, 29, 0.96);
  }

  .stat-card :global(svg) {
    flex-shrink: 0;
    color: var(--blue);
  }

  .stat-label {
    display: block;
    font-size: 12px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 4px;
  }

  .stat-value {
    display: block;
    font-size: 24px;
    font-weight: 700;
    color: var(--text);
  }
</style>
