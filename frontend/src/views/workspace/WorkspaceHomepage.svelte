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
    recentWorkspaces,
    onCreateWorkspace = () => {},
    onSelectWorkspace = () => {},
  }: $$Props = $props();

  function languageAccent(language: string) {
    if (language === 'rust') return 'var(--orange)';
    if (language === 'typescript' || language === 'javascript') return 'var(--blue)';
    if (language === 'markdown') return 'var(--green)';
    if (language === 'python') return 'var(--warning)';
    return 'var(--violet)';
  }
</script>

<section class="welcome">
  <div class="welcome-inner">
    <header class="welcome-head">
      <div class="brand-mark"><Icons.Code2 size={30} /></div>
      <div>
        <h1>Workspace</h1>
        <p>Edit code with a file tree, Monaco editor, multi-file tabs and project search — all served from your local cloud.</p>
      </div>
    </header>

    <div class="welcome-grid">
      <div class="col">
        <h2 class="col-title">Start</h2>
        <button type="button" class="start-row" onclick={onCreateWorkspace}>
          <Icons.FilePlus2 size={18} />
          <span class="start-label">New workspace<small>Create an empty project</small></span>
        </button>
        <button
          type="button"
          class="start-row"
          disabled={recentWorkspaces.length === 0}
          onclick={() => recentWorkspaces[0] && onSelectWorkspace(recentWorkspaces[0].id)}
        >
          <Icons.FolderOpen size={18} />
          <span class="start-label">Open latest<small>{recentWorkspaces[0]?.name || 'No workspaces yet'}</small></span>
        </button>
      </div>

      <div class="col">
        <h2 class="col-title">Recent</h2>
        {#if recentWorkspaces.length === 0}
          <div class="empty">
            <Icons.Inbox size={18} />
            <span>No recent workspaces. Create one to get started.</span>
          </div>
        {:else}
          <ul class="recent-list">
            {#each recentWorkspaces as workspace (workspace.id)}
              <li>
                <button type="button" class="recent-row" onclick={() => onSelectWorkspace(workspace.id)}>
                  <span class="lang-dot" style="background: {languageAccent(workspace.language)}"></span>
                  <span class="recent-name">{workspace.name || 'untitled'}</span>
                  <span class="recent-meta">{workspace.language}</span>
                  <span class="recent-time">{formatRelative(workspace.updated_at)}</span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </div>

    <footer class="welcome-foot">
      <Icons.Info size={13} />
      <span>{workspaces.length} workspace{workspaces.length === 1 ? '' : 's'} · Autosave on · Terminal execution stays sandboxed until safe sandboxing lands.</span>
    </footer>
  </div>
</section>

<style>
  .welcome {
    flex: 1;
    min-width: 0;
    height: 100%;
    overflow: auto;
    background:
      radial-gradient(circle at 50% -10%, rgba(110, 168, 255, 0.08), transparent 55%),
      #0c0d12;
    display: flex;
    justify-content: center;
  }

  .welcome-inner {
    width: min(880px, 100%);
    padding: 64px 40px 48px;
  }

  .welcome-head {
    display: flex;
    align-items: flex-start;
    gap: 18px;
    margin-bottom: 40px;
  }

  .brand-mark {
    width: 56px;
    height: 56px;
    flex-shrink: 0;
    border-radius: 16px;
    display: grid;
    place-items: center;
    color: var(--blue);
    background: rgba(110, 168, 255, 0.12);
    border: 1px solid rgba(110, 168, 255, 0.22);
  }

  .welcome-head h1 {
    margin: 0;
    font-size: 32px;
    letter-spacing: -0.02em;
  }

  .welcome-head p {
    margin: 8px 0 0;
    max-width: 560px;
    color: var(--muted);
    font-size: 14px;
    line-height: 1.6;
  }

  .welcome-grid {
    display: grid;
    grid-template-columns: minmax(0, 280px) minmax(0, 1fr);
    gap: 48px;
  }

  .col-title {
    margin: 0 0 12px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--dim);
    font-family: var(--ff-mono);
  }

  .start-row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 10px;
    border: none;
    background: transparent;
    color: var(--blue);
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    transition: background 0.15s;
  }

  .start-row:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.04);
  }

  .start-row:disabled {
    color: var(--dim);
    cursor: default;
  }

  .start-label {
    display: flex;
    flex-direction: column;
    line-height: 1.3;
    font-size: 14px;
    font-weight: 500;
  }

  .start-label small {
    color: var(--muted);
    font-weight: 400;
    font-size: 12px;
  }

  .recent-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
  }

  .recent-row {
    width: 100%;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto auto;
    align-items: center;
    gap: 12px;
    padding: 9px 10px;
    border: none;
    background: transparent;
    color: var(--text);
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    transition: background 0.15s;
  }

  .recent-row:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .lang-dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
  }

  .recent-name {
    font-size: 14px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .recent-meta {
    font-size: 11px;
    color: var(--muted);
    font-family: var(--ff-mono);
  }

  .recent-time {
    font-size: 11px;
    color: var(--dim);
    font-family: var(--ff-mono);
  }

  .empty {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px;
    border-radius: 10px;
    border: 1px dashed var(--border);
    color: var(--muted);
    font-size: 13px;
  }

  .welcome-foot {
    margin-top: 44px;
    padding-top: 18px;
    border-top: 1px solid var(--hairline);
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--dim);
    font-size: 12px;
  }

  @media (max-width: 720px) {
    .welcome-inner {
      padding: 36px 20px;
    }

    .welcome-grid {
      grid-template-columns: 1fr;
      gap: 28px;
    }
  }
</style>
