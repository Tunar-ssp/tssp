<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { formatBytes } from '$lib/utils';

  type AdminSection = 'overview' | 'users' | 'sessions' | 'devices' | 'public' | 'activity' | 'maintenance';

  interface NavItem {
    id: AdminSection;
    label: string;
  }

  interface Props {
    activeSection?: AdminSection;
    navItems?: NavItem[];
    status?: any;
    sessions?: Array<any>;
    loading?: boolean;
    onRefresh?: () => void;
    onShowConsole?: () => void;
  }

  let {
    activeSection = 'overview',
    navItems = [],
    status,
    sessions = [],
    loading = false,
    onRefresh,
    onShowConsole,
  }: Props = $props();

  function sectionTitle(section: AdminSection) {
    return navItems.find((item) => item.id === section)?.label || 'Admin';
  }

  function healthTone() {
    const statusValue = status?.status?.toLowerCase() || '';
    if (statusValue.includes('ok') || statusValue.includes('healthy') || statusValue.includes('ready')) {
      return 'good';
    }
    if (statusValue.includes('warn') || statusValue.includes('degraded')) {
      return 'warn';
    }
    return 'neutral';
  }

  function healthLabel() {
    return status?.status || 'Status unavailable';
  }
</script>

<header class="admin-header">
  <div>
    <div class="eyebrow">Operations</div>
    <h1>{sectionTitle(activeSection)}</h1>
    <p>Organized system control for users, sessions, sharing, maintenance, and diagnostics.</p>
    <div class="header-meta">
      <span class={`health-pill ${healthTone()}`}>
        <span class="health-dot"></span>
        {healthLabel()}
      </span>
      <span class="meta-pill">{formatBytes(status?.disk_used || 0)} disk used</span>
      <span class="meta-pill">{sessions.length} live sessions</span>
    </div>
  </div>

  <div class="header-actions">
    <button type="button" class="ghost-btn" onclick={onRefresh} disabled={loading}>
      <Icons.RefreshCcw size={14} />
      Refresh
    </button>
    <button type="button" class="accent-btn" onclick={onShowConsole}>
      <Icons.Terminal size={14} />
      Safe console
    </button>
  </div>
</header>

<style>
  .admin-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .eyebrow {
    font-size: 11px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--dim);
    font-family: var(--ff-mono);
  }

  h1 {
    margin: 8px 0 0;
    font-size: 34px;
    line-height: 1;
    letter-spacing: -0.04em;
    font-family: var(--ff-display);
    color: var(--text);
  }

  p {
    margin: 10px 0 0;
    color: var(--muted);
    font-size: 14px;
  }

  .header-meta {
    margin-top: 14px;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .health-pill,
  .meta-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 32px;
    padding: 0 12px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.03);
    color: var(--text-2);
    font-size: 12px;
    font-weight: 500;
  }

  .health-pill.good {
    border-color: rgba(91, 227, 154, 0.22);
    background: rgba(91, 227, 154, 0.1);
    color: var(--green);
  }

  .health-pill.warn {
    border-color: rgba(251, 191, 36, 0.24);
    background: rgba(251, 191, 36, 0.1);
    color: var(--warning);
  }

  .health-dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: currentColor;
    box-shadow: 0 0 10px currentColor;
  }

  .header-actions {
    display: flex;
    gap: 10px;
  }

  .ghost-btn,
  .accent-btn {
    height: 36px;
    padding: 0 14px;
    border-radius: 12px;
    border: 1px solid var(--border);
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-family: inherit;
  }

  .ghost-btn {
    background: var(--surface);
    color: var(--text-2);
  }

  .accent-btn {
    background: linear-gradient(135deg, rgba(163, 148, 255, 0.28), rgba(110, 168, 255, 0.22));
    color: var(--text);
    border-color: rgba(163, 148, 255, 0.18);
  }

  .ghost-btn:disabled,
  .accent-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  @media (max-width: 760px) {
    .admin-header {
      flex-direction: column;
    }

    h1 {
      font-size: 28px;
    }

    p {
      font-size: 13px;
    }

    .header-actions {
      width: 100%;
      justify-content: stretch;
    }

    .header-actions button {
      flex: 1;
      justify-content: center;
    }
  }
</style>
