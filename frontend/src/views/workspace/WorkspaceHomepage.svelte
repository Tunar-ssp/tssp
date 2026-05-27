<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import type { Workspace } from '$lib/api';
  import { formatRelative } from '$lib/utils';

  interface $$Props {
    workspaces: Workspace[];
    languageCount: number;
    recentWorkspaces: Workspace[];
    onCreateWorkspace?: () => void;
    onSelectWorkspace?: (id: string) => void;
  }

  let {
    workspaces,
    languageCount,
    recentWorkspaces,
    onCreateWorkspace = () => {},
    onSelectWorkspace = () => {},
  }: $$Props = $props();

  function summarizeBody(text: string) {
    return text.replace(/\s+/g, ' ').trim();
  }

  function languageAccent(language: string) {
    if (language === 'rust') return 'var(--orange)';
    if (language === 'typescript' || language === 'javascript') return 'var(--blue)';
    if (language === 'markdown') return 'var(--green)';
    if (language === 'python') return 'var(--warning)';
    return 'var(--violet)';
  }
</script>

<section class="workspace-home">
  <header class="home-hero">
    <div>
      <div class="eyebrow">Workspace</div>
      <h1>Local editor foundation</h1>
      <p>
        TSSP currently stores single-document workspaces. Nested trees, folder actions, and execution are intentionally
        hidden until backend support exists.
      </p>
    </div>

    <div class="hero-actions">
      <button type="button" class="primary-btn" onclick={onCreateWorkspace}>
        <Icons.Plus size={16} />
        New workspace
      </button>
      <button
        type="button"
        class="ghost-btn"
        onclick={() => recentWorkspaces[0] && onSelectWorkspace(recentWorkspaces[0].id)}
      >
        <Icons.FolderOpen size={16} />
        Open latest
      </button>
    </div>
  </header>

  <div class="hub-metrics">
    <article class="metric-card">
      <span>Documents</span>
      <strong>{workspaces.length}</strong>
      <p>Storage-backed workspace entries</p>
    </article>
    <article class="metric-card">
      <span>Languages</span>
      <strong>{languageCount}</strong>
      <p>Inferred from the saved workspace metadata</p>
    </article>
    <article class="metric-card">
      <span>Sync model</span>
      <strong>Autosave</strong>
      <p>Changes persist to the local backend after short idle windows</p>
    </article>
    <article class="metric-card warning">
      <span>Execution</span>
      <strong>Locked</strong>
      <p>Terminal and run controls remain disabled until a safe sandbox exists</p>
    </article>
  </div>

  <section class="hub-section">
    <div class="section-head">
      <span class="sidebar-label">Recent</span>
    </div>
    <div class="hub-grid">
      {#each recentWorkspaces as workspace (workspace.id)}
        <button type="button" class="hub-card" onclick={() => onSelectWorkspace(workspace.id)}>
          <div class="hub-card-head">
            <strong>{workspace.name || 'untitled'}</strong>
            <span class="lang-chip" style="--tone: {languageAccent(workspace.language)}">{workspace.language}</span>
          </div>
          <p>{summarizeBody(workspace.body).slice(0, 180) || 'No content yet'}</p>
          <small>{formatRelative(workspace.updated_at)}</small>
        </button>
      {/each}
    </div>
  </section>
</section>

<style>
  .workspace-home {
    padding: 28px 32px 36px;
    overflow: auto;
  }

  .home-hero {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
    margin-bottom: 28px;
  }

  .eyebrow {
    font-family: var(--ff-mono);
    font-size: 12px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--dim);
  }

  .home-hero h1 {
    margin: 8px 0 0;
    font-size: clamp(42px, 4vw, 64px);
    line-height: 0.98;
    letter-spacing: -0.04em;
  }

  .home-hero p {
    margin: 14px 0 0;
    max-width: 760px;
    color: var(--muted);
    font-size: 18px;
    line-height: 1.6;
  }

  .hero-actions {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .primary-btn,
  .ghost-btn {
    height: 44px;
    padding: 0 16px;
    border-radius: 16px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .primary-btn {
    border: 1px solid rgba(110, 168, 255, 0.24);
    background: rgba(110, 168, 255, 0.14);
    color: var(--blue);
  }

  .ghost-btn {
    border: 1px solid var(--border);
    background: rgba(18, 22, 31, 0.96);
    color: var(--text-2);
  }

  .ghost-btn:hover,
  .primary-btn:hover {
    color: var(--text);
  }

  .hub-metrics {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 16px;
    margin-bottom: 28px;
  }

  .metric-card {
    padding: 20px;
    border-radius: 24px;
    border: 1px solid var(--border);
    background: rgba(18, 21, 29, 0.98);
  }

  .metric-card.warning {
    border-color: rgba(251, 191, 36, 0.2);
    background: rgba(45, 32, 16, 0.38);
  }

  .metric-card span {
    color: var(--muted);
    font-family: var(--ff-mono);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.16em;
  }

  .metric-card strong {
    display: block;
    margin-top: 18px;
    font-size: 36px;
    line-height: 1;
  }

  .metric-card p {
    margin: 12px 0 0;
    color: var(--muted);
    line-height: 1.5;
  }

  .hub-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .hub-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
  }

  .hub-card {
    padding: 22px;
    border-radius: 24px;
    border: 1px solid var(--border);
    background: rgba(17, 20, 28, 0.98);
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }

  .hub-card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .hub-card-head strong {
    font-size: 20px;
  }

  .hub-card p {
    margin: 16px 0;
    color: var(--muted);
    line-height: 1.55;
  }

  .hub-card small {
    color: var(--dim);
    font-family: var(--ff-mono);
  }

  @media (max-width: 1400px) {
    .hub-metrics {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 960px) {
    .home-hero {
      flex-direction: column;
      align-items: stretch;
    }

    .hub-metrics {
      grid-template-columns: 1fr;
    }
  }
</style>
