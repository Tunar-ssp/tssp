<script lang="ts">
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import { api } from '$lib/api';
  import { currentView } from '$lib/stores/ui';

  type Status = Awaited<ReturnType<typeof api.getStatus>>;

  let status: Status | null = null;
  let loading = true;
  let error = '';

  const apps = [
    {
      id: 'drive',
      label: 'Cloud',
      title: 'Cloud Drive',
      description: 'Files, folders, previews, uploads, and public sharing.',
      icon: Icons.Cloud,
      accent: '#6ea8ff',
      shortcut: 'D',
    },
    {
      id: 'notes',
      label: 'Notes',
      title: 'Notes',
      description: 'Private pages, tags, pinned notes, and fast capture.',
      icon: Icons.BookOpen,
      accent: '#fbbf24',
      shortcut: 'N',
    },
    {
      id: 'workspace',
      label: 'Workspace',
      title: 'Workspace',
      description: 'Project documents, editor tabs, and local IDE workflows.',
      icon: Icons.Code2,
      accent: '#5be39a',
      shortcut: 'W',
    },
    {
      id: 'operations',
      label: 'Admin',
      title: 'Admin Console',
      description: 'Users, sessions, storage, health, diagnostics, and safe ops.',
      icon: Icons.Shield,
      accent: '#a394ff',
      shortcut: 'A',
    },
  ];

  onMount(async () => {
    try {
      status = await api.getStatus();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Could not load system status';
    } finally {
      loading = false;
    }
  });

  function openApp(id: string) {
    currentView.set(id);
  }

  function formatBytes(bytes = 0): string {
    if (bytes <= 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    return `${(bytes / 1024 ** index).toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
  }

  function formatUptime(seconds = 0): string {
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    if (hours < 48) return `${hours}h ${minutes % 60}m`;
    return `${Math.floor(hours / 24)}d ${hours % 24}h`;
  }

  $: healthLabel = status?.corrupt_file_count
    ? `${status.corrupt_file_count} missing blob${status.corrupt_file_count === 1 ? '' : 's'}`
    : 'Healthy';
</script>

<section class="launcher" aria-label="TSSP app launcher">
  <div class="ambient ambient-blue"></div>
  <div class="ambient ambient-green"></div>
  <div class="ambient ambient-pink"></div>

  <header class="hero">
    <div class="hero-copy">
      <div class="eyebrow">
        <span class="pulse"></span>
        Local-first cloud OS
      </div>
      <h1>TSSP</h1>
      <p>
        Four focused apps for your private storage system: cloud files, notes,
        workspace, and admin operations.
      </p>
    </div>

    <aside class="status-card" aria-live="polite">
      {#if loading}
        <span class="status-kicker">Checking daemon</span>
        <strong>Loading status</strong>
        <small>Reading real backend health metrics.</small>
      {:else if error}
        <span class="status-kicker danger">Status unavailable</span>
        <strong>Backend error</strong>
        <small>{error}</small>
      {:else if status}
        <span class="status-kicker ok">Daemon online</span>
        <strong>{healthLabel}</strong>
        <small>v{status.version} · uptime {formatUptime(status.uptime_seconds)}</small>
      {/if}
    </aside>
  </header>

  <div class="desktop">
    <div class="launcher-panel">
      <div class="panel-head">
        <span>Choose an app</span>
        <small>Bottom dock stays available everywhere</small>
      </div>

      <div class="app-grid">
        {#each apps as app (app.id)}
          <button
            type="button"
            class="app-tile"
            style={`--accent:${app.accent}`}
            on:click={() => openApp(app.id)}
            aria-label={`Open ${app.title}`}
          >
            <span class="app-icon" aria-hidden="true">
              <svelte:component this={app.icon} size={38} strokeWidth={2.2} />
            </span>
            <span class="app-copy">
              <strong>{app.title}</strong>
              <span>{app.description}</span>
            </span>
            <kbd>{app.shortcut}</kbd>
          </button>
        {/each}
      </div>
    </div>

    <div class="metrics-panel">
      <div class="metric-card">
        <Icons.Files size={18} />
        <span>Files</span>
        <strong>{status?.file_count ?? '—'}</strong>
      </div>
      <div class="metric-card">
        <Icons.BookMarked size={18} />
        <span>Notes</span>
        <strong>{status?.note_count ?? '—'}</strong>
      </div>
      <div class="metric-card">
        <Icons.Database size={18} />
        <span>Storage</span>
        <strong>{status ? formatBytes(status.storage_bytes_used) : '—'}</strong>
      </div>
      <div class="metric-card">
        <Icons.Tags size={18} />
        <span>Tags</span>
        <strong>{status?.tag_count ?? '—'}</strong>
      </div>
    </div>
  </div>
</section>

<style>
  .launcher {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: clamp(28px, 5vw, 64px) clamp(18px, 5vw, 72px) 130px;
    background:
      radial-gradient(circle at 20% 16%, rgba(110, 168, 255, 0.14), transparent 28%),
      radial-gradient(circle at 82% 10%, rgba(91, 227, 154, 0.12), transparent 24%),
      linear-gradient(145deg, #07080c 0%, #0c0f16 48%, #07080c 100%);
    isolation: isolate;
  }

  .ambient {
    position: fixed;
    width: 280px;
    height: 280px;
    border-radius: 999px;
    filter: blur(70px);
    opacity: 0.34;
    pointer-events: none;
    z-index: -1;
  }

  .ambient-blue {
    left: 8vw;
    top: 20vh;
    background: #356dff;
  }

  .ambient-green {
    right: 12vw;
    top: 14vh;
    background: #21d07a;
  }

  .ambient-pink {
    right: 24vw;
    bottom: 14vh;
    background: #b23cff;
  }

  .hero {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(260px, 340px);
    gap: 28px;
    align-items: end;
    max-width: 1180px;
    margin: 0 auto 34px;
  }

  .hero-copy {
    display: grid;
    gap: 12px;
  }

  .eyebrow,
  .status-kicker {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--text-2);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  .pulse {
    width: 9px;
    height: 9px;
    border-radius: 999px;
    background: var(--green);
    box-shadow: 0 0 18px rgba(91, 227, 154, 0.75);
  }

  h1 {
    margin: 0;
    font-size: clamp(56px, 11vw, 126px);
    line-height: 0.82;
    letter-spacing: -0.09em;
    font-family: var(--ff-display);
  }

  p {
    max-width: 640px;
    margin: 0;
    color: var(--text-2);
    font-size: clamp(15px, 1.8vw, 18px);
    line-height: 1.65;
  }

  .status-card,
  .launcher-panel,
  .metrics-panel {
    border: 1px solid rgba(255, 255, 255, 0.1);
    background:
      linear-gradient(180deg, rgba(255,255,255,0.075), rgba(255,255,255,0.025)),
      rgba(16, 18, 24, 0.72);
    box-shadow: var(--shadow-card);
    backdrop-filter: blur(24px);
  }

  .status-card {
    display: grid;
    gap: 8px;
    padding: 18px;
    border-radius: 22px;
  }

  .status-card strong {
    font-size: 22px;
  }

  .status-card small {
    color: var(--muted);
    line-height: 1.45;
  }

  .status-kicker.ok {
    color: var(--green);
  }

  .status-kicker.danger {
    color: var(--danger);
  }

  .desktop {
    max-width: 1180px;
    margin: 0 auto;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 310px;
    gap: 18px;
    align-items: start;
  }

  .launcher-panel {
    border-radius: 30px;
    padding: 18px;
  }

  .panel-head {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    padding: 4px 4px 16px;
    color: var(--text);
    font-weight: 700;
  }

  .panel-head small {
    color: var(--muted);
    font-weight: 500;
  }

  .app-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 14px;
  }

  .app-tile {
    min-height: 180px;
    display: grid;
    grid-template-rows: auto 1fr auto;
    gap: 18px;
    padding: 18px;
    border: 1px solid rgba(255,255,255,0.09);
    border-radius: 24px;
    background:
      radial-gradient(circle at 28% 18%, color-mix(in srgb, var(--accent) 34%, transparent), transparent 38%),
      linear-gradient(160deg, rgba(255,255,255,0.08), rgba(255,255,255,0.025));
    color: var(--text);
    text-align: left;
    box-shadow: 0 14px 38px rgba(0,0,0,.28);
    transition:
      transform var(--duration-normal) var(--ease-smooth),
      border-color var(--duration-normal) var(--ease-smooth),
      background var(--duration-normal) var(--ease-smooth);
  }

  .app-tile:hover {
    transform: translateY(-5px);
    border-color: color-mix(in srgb, var(--accent) 52%, white 10%);
    background:
      radial-gradient(circle at 28% 18%, color-mix(in srgb, var(--accent) 48%, transparent), transparent 42%),
      linear-gradient(160deg, rgba(255,255,255,0.105), rgba(255,255,255,0.035));
  }

  .app-icon {
    width: 74px;
    height: 74px;
    display: grid;
    place-items: center;
    color: white;
    border-radius: 22px;
    background:
      linear-gradient(145deg, color-mix(in srgb, var(--accent) 86%, white 8%), color-mix(in srgb, var(--accent) 48%, black 24%));
    box-shadow:
      0 16px 28px color-mix(in srgb, var(--accent) 26%, transparent),
      0 1px 0 rgba(255,255,255,.38) inset;
  }

  .app-copy {
    display: grid;
    gap: 8px;
  }

  .app-copy strong {
    font-size: 22px;
    letter-spacing: -0.02em;
  }

  .app-copy span {
    max-width: 34ch;
    color: var(--text-2);
    font-size: 13px;
    line-height: 1.55;
  }

  kbd {
    justify-self: end;
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border: 1px solid rgba(255,255,255,0.12);
    border-radius: 9px;
    background: rgba(0,0,0,0.22);
    color: var(--muted);
    font-family: var(--ff-mono);
    font-size: 12px;
  }

  .metrics-panel {
    display: grid;
    gap: 10px;
    border-radius: 26px;
    padding: 14px;
  }

  .metric-card {
    display: grid;
    grid-template-columns: 22px 1fr auto;
    align-items: center;
    gap: 10px;
    min-height: 58px;
    padding: 12px;
    border: 1px solid rgba(255,255,255,0.06);
    border-radius: 16px;
    background: rgba(255,255,255,0.035);
  }

  .metric-card :global(svg) {
    color: var(--blue);
  }

  .metric-card span {
    color: var(--text-2);
    font-size: 13px;
  }

  .metric-card strong {
    font-size: 16px;
  }

  @media (max-width: 920px) {
    .hero,
    .desktop {
      grid-template-columns: 1fr;
    }

    .metrics-panel {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 640px) {
    .launcher {
      padding: 26px 14px 112px;
    }

    .app-grid,
    .metrics-panel {
      grid-template-columns: 1fr;
    }

    .app-tile {
      min-height: 154px;
    }

    .panel-head {
      display: grid;
    }
  }
</style>
