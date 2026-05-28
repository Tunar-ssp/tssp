<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { formatBytes } from '$lib/utils';

  interface Props {
    isAdmin?: boolean;
    fileCount?: number;
    noteCount?: number;
    workspaceCount?: number;
    storageBytes?: number;
    pinnedCount?: number;
    onOpen?: (view: 'drive' | 'notes' | 'workspace' | 'admin') => void;
  }

  let {
    isAdmin = false,
    fileCount = 0,
    noteCount = 0,
    workspaceCount = 0,
    storageBytes = 0,
    pinnedCount = 0,
    onOpen = () => {},
  }: Props = $props();

  type AppCard = {
    id: 'drive' | 'notes' | 'workspace' | 'admin';
    name: string;
    desc: string;
    icon: any;
    accent: string;
    stat: string;
    sub: string;
  };

  let apps = $derived.by<AppCard[]>(() => {
    const cards: AppCard[] = [
      {
        id: 'drive',
        name: 'Cloud Drive',
        desc: 'Store, preview and share files',
        icon: Icons.Cloud,
        accent: '#5b8cff',
        stat: `${fileCount}`,
        sub: fileCount === 1 ? 'file' : 'files',
      },
      {
        id: 'notes',
        name: 'Notes',
        desc: 'Docs, knowledge and writing',
        icon: Icons.NotebookPen,
        accent: '#5be39a',
        stat: `${noteCount}`,
        sub: noteCount === 1 ? 'note' : 'notes',
      },
      {
        id: 'workspace',
        name: 'Workspace',
        desc: 'Edit code with a real editor',
        icon: Icons.SquareCode,
        accent: '#ff8a3d',
        stat: `${workspaceCount}`,
        sub: workspaceCount === 1 ? 'project' : 'projects',
      },
    ];
    if (isAdmin) {
      cards.push({
        id: 'admin',
        name: 'Operations',
        desc: 'System, users and audit log',
        icon: Icons.Gauge,
        accent: '#a394ff',
        stat: formatBytes(storageBytes),
        sub: 'used',
      });
    }
    return cards;
  });
</script>

<section class="apps">
  <div class="apps-head">
    <h2>Apps</h2>
    {#if pinnedCount > 0}<span>{pinnedCount} pinned</span>{/if}
  </div>
  <div class="apps-grid">
    {#each apps as app (app.id)}
      {@const Icon = app.icon}
      <button
        type="button"
        class="app-card"
        style="--app-accent: {app.accent}"
        onclick={() => onOpen(app.id)}
      >
        <div class="app-icon">
          <Icon size={26} strokeWidth={1.9} />
        </div>
        <div class="app-body">
          <div class="app-name">{app.name}</div>
          <div class="app-desc">{app.desc}</div>
        </div>
        <div class="app-foot">
          <span class="app-stat"><strong>{app.stat}</strong> {app.sub}</span>
          <Icons.ArrowUpRight size={16} class="app-arrow" />
        </div>
      </button>
    {/each}
  </div>
</section>

<style>
  .apps {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .apps-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }

  .apps-head h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--text);
  }

  .apps-head span {
    font-size: 12px;
    color: var(--muted);
  }

  .apps-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(232px, 1fr));
    gap: 14px;
  }

  .app-card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 18px;
    min-height: 168px;
    border: 1px solid var(--border);
    border-radius: var(--r-6);
    background:
      radial-gradient(120% 90% at 100% 0%, color-mix(in srgb, var(--app-accent) 10%, transparent), transparent 60%),
      var(--bg-secondary);
    color: var(--text);
    text-align: left;
    cursor: pointer;
    overflow: hidden;
    transition: transform 0.16s var(--ease-smooth), border-color 0.16s, box-shadow 0.16s;
  }

  .app-card::before {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    border-top: 2px solid color-mix(in srgb, var(--app-accent) 60%, transparent);
    opacity: 0;
    transition: opacity 0.16s;
    pointer-events: none;
  }

  .app-card:hover {
    transform: translateY(-3px);
    border-color: color-mix(in srgb, var(--app-accent) 45%, var(--border));
    box-shadow: 0 14px 34px rgba(0, 0, 0, 0.34);
  }

  .app-card:hover::before {
    opacity: 1;
  }

  .app-icon {
    display: grid;
    place-items: center;
    width: 50px;
    height: 50px;
    border-radius: 14px;
    color: var(--app-accent);
    background: color-mix(in srgb, var(--app-accent) 16%, transparent);
    border: 1px solid color-mix(in srgb, var(--app-accent) 26%, transparent);
  }

  .app-body {
    flex: 1;
  }

  .app-name {
    font-size: 16px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .app-desc {
    margin-top: 4px;
    font-size: 13px;
    color: var(--muted);
    line-height: 1.45;
  }

  .app-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .app-stat {
    font-size: 13px;
    color: var(--text-2);
  }

  .app-stat strong {
    color: var(--text);
    font-size: 15px;
    font-variant-numeric: tabular-nums;
  }

  .app-card :global(.app-arrow) {
    color: var(--dim);
    transition: color 0.16s, transform 0.16s;
  }

  .app-card:hover :global(.app-arrow) {
    color: var(--app-accent);
    transform: translate(2px, -2px);
  }
</style>
