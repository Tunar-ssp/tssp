<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    userName?: string;
    isAdmin?: boolean;
    storageUsed?: number;
    storageTotal?: number;
    onQuickAction?: (action: string) => void;
  }

  let { userName, isAdmin = false, storageUsed = 0, storageTotal = 100 }: $$Props = $props();

  const hour = new Date().getHours();
  let greeting = 'Good morning';
  if (hour >= 12 && hour < 18) greeting = 'Good afternoon';
  if (hour >= 18) greeting = 'Good evening';

  function getStoragePercent(): number {
    if (storageTotal === 0) return 0;
    return Math.round((storageUsed / storageTotal) * 100);
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }
</script>

<section class="launcher-hero">
  <div class="hero-content">
    <div class="hero-greeting">
      <h1>{greeting}, {userName || 'User'}</h1>
      <p>Welcome to TSSP, your self-hosted cloud workspace</p>
    </div>

    <div class="hero-status">
      <div class="status-card">
        <Icons.HardDrive size={20} />
        <div class="status-info">
          <span class="status-label">Storage</span>
          <div class="storage-bar">
            <div class="bar-fill" style="width: {getStoragePercent()}%"></div>
          </div>
          <span class="status-value">{formatBytes(storageUsed)} / {formatBytes(storageTotal)}</span>
        </div>
      </div>

      {#if isAdmin}
        <div class="status-card">
          <Icons.Lock size={20} />
          <div class="status-info">
            <span class="status-label">Admin Panel</span>
            <span class="status-value">Manage users & settings</span>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <div class="hero-actions">
    <button class="action-btn primary" onclick={() => {}}>
      <Icons.Plus size={18} />
      <span>New File</span>
    </button>
    <button class="action-btn primary" onclick={() => {}}>
      <Icons.FileText size={18} />
      <span>New Note</span>
    </button>
    <button class="action-btn primary" onclick={() => {}}>
      <Icons.Code size={18} />
      <span>New Workspace</span>
    </button>
  </div>
</section>

<style>
  .launcher-hero {
    padding: 40px;
    margin-bottom: 40px;
    border-radius: 24px;
    background: linear-gradient(
      135deg,
      rgba(110, 168, 255, 0.06),
      rgba(139, 92, 246, 0.06)
    );
    border: 1px solid rgba(110, 168, 255, 0.12);
  }

  .hero-content {
    margin-bottom: 32px;
  }

  .hero-greeting h1 {
    margin: 0;
    font-size: 48px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text);
  }

  .hero-greeting p {
    margin: 8px 0 0;
    font-size: 18px;
    color: var(--muted);
  }

  .hero-status {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 16px;
    margin-top: 28px;
  }

  .status-card {
    display: flex;
    gap: 16px;
    padding: 16px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: rgba(18, 21, 29, 0.96);
  }

  .status-card :global(svg) {
    flex-shrink: 0;
    color: var(--blue);
  }

  .status-info {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .status-label {
    font-size: 12px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .status-value {
    font-size: 14px;
    color: var(--text);
    font-weight: 500;
  }

  .storage-bar {
    width: 100%;
    height: 6px;
    border-radius: 3px;
    background: var(--surface-3);
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--blue), var(--blue));
    transition: width 0.3s ease;
  }

  .hero-actions {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 12px 20px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
    font-size: 14px;
    transition: all 0.2s;
  }

  .action-btn:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .action-btn.primary {
    background: linear-gradient(135deg, rgba(110, 168, 255, 0.96), rgba(95, 149, 233, 0.96));
    border-color: rgba(110, 168, 255, 0.28);
    color: #06101f;
    font-weight: 600;
  }

  .action-btn.primary:hover {
    transform: translateY(-1px);
    box-shadow: 0 8px 20px rgba(110, 168, 255, 0.2);
  }
</style>
