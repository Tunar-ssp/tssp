<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { formatRelative } from '$lib/utils/format';
  import type { Workspace } from '$lib/api';

  interface Props {
    workspaces?: Workspace[];
    onOpenWorkspace?: (workspace: Workspace) => void;
    onOpenWorkspaces?: () => void;
  }

  let { workspaces = [], onOpenWorkspace, onOpenWorkspaces }: Props = $props();
</script>

<article class="panel">
  <div class="panel-head">
    <div>
      <h2>Open workspaces</h2>
      <p>Workspace stays honest about editing, not execution.</p>
    </div>
    <button type="button" class="link-btn" onclick={onOpenWorkspaces}>Open Workspace</button>
  </div>

  {#if workspaces.length === 0}
    <div class="empty-card compact">
      <Icons.Code2 size={20} />
      <strong>No workspaces yet</strong>
      <p>Create a workspace to start editing local project files and docs.</p>
    </div>
  {:else}
    <div class="workspace-list">
      {#each workspaces.slice(0, 4) as workspace (workspace.id)}
        <button type="button" class="workspace-row" onclick={() => onOpenWorkspace?.(workspace)}>
          <div class="workspace-icon">{workspace.name.slice(0, 1).toUpperCase()}</div>
          <div class="workspace-copy">
            <strong>{workspace.name}</strong>
            <span>{workspace.language || 'text'} · {formatRelative(workspace.updated_at)}</span>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</article>

<style>
  .panel {
    border: 1px solid rgba(255, 255, 255, 0.07);
    background: linear-gradient(180deg, rgba(20, 22, 29, 0.94), rgba(15, 16, 22, 0.92));
    box-shadow: var(--shadow-card);
    border-radius: 22px;
    padding: 18px;
  }

  .panel-head {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    margin-bottom: 16px;
  }

  .panel-head h2 {
    margin: 0;
    font-size: 17px;
    color: var(--text);
  }

  .panel-head p {
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 13px;
    line-height: 1.5;
  }

  .link-btn {
    margin-left: auto;
    border: none;
    background: none;
    color: var(--blue);
    font-size: 12px;
    cursor: pointer;
  }

  .workspace-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .workspace-row {
    border: 1px solid var(--border);
    background: var(--surface);
    color: inherit;
    cursor: pointer;
    border-radius: 14px;
    padding: 12px;
    display: flex;
    align-items: center;
    gap: 12px;
    text-align: left;
    transition: all 150ms;
  }

  .workspace-row:hover {
    border-color: var(--border-2);
    background: var(--surface-2);
  }

  .workspace-icon {
    width: 34px;
    height: 34px;
    border-radius: 10px;
    background: linear-gradient(135deg, rgba(255, 138, 61, 0.9), rgba(255, 95, 162, 0.55));
    color: rgba(8, 9, 12, 0.86);
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--ff-mono);
    font-size: 13px;
    font-weight: 700;
    flex-shrink: 0;
  }

  .workspace-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .workspace-copy strong {
    font-size: 13px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .workspace-copy span {
    font-size: 11px;
    color: var(--muted);
  }

  .empty-card {
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--surface);
    min-height: 110px;
    justify-content: center;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 14px;
  }

  .empty-card.compact {
    min-height: 110px;
  }

  .empty-card strong {
    display: block;
    color: var(--text);
    font-size: 13px;
  }

  .empty-card p {
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.5;
  }
</style>
