<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { formatRelative } from '$lib/utils/format';
  import { api, type AdminActivityItem } from '$lib/api';

  type Status = Awaited<ReturnType<typeof api.getStatus>>;

  interface Props {
    isAdmin?: boolean;
    activityItems?: AdminActivityItem[];
    status?: Status;
    workspaceCount?: number;
    onOpenAdmin?: () => void;
  }

  let { isAdmin = false, activityItems = [], status, workspaceCount = 0, onOpenAdmin }: Props = $props();
</script>

<article class="panel activity-panel">
  <div class="panel-head">
    <div>
      <h2>{isAdmin ? 'Admin activity' : 'System posture'}</h2>
      <p>{isAdmin ? 'Real audit events from the backend.' : 'Your local-first cloud is healthy and ready.'}</p>
    </div>
    {#if isAdmin}
      <button type="button" class="link-btn" onclick={onOpenAdmin}>Open Admin</button>
    {/if}
  </div>

  {#if isAdmin && activityItems.length > 0}
    <div class="activity-list">
      {#each activityItems as item, i (`${item.id}-${item.occurred_at}-${i}`)}
        <div class="activity-row">
          <div class="activity-glyph">
            <Icons.Activity size={13} />
          </div>
          <div class="activity-copy">
            <strong>{item.title}</strong>
            <p>{item.detail}</p>
          </div>
          <span>{formatRelative(item.occurred_at)}</span>
        </div>
      {/each}
    </div>
  {:else}
    <div class="health-stack">
      <div class="health-card">
        <Icons.HardDrive size={18} />
        <div>
          <strong>Drive ready</strong>
          <p>{status?.file_count || 0} files indexed for Drive, media, public, and search.</p>
        </div>
      </div>
      <div class="health-card">
        <Icons.BookText size={18} />
        <div>
          <strong>Notes synced</strong>
          <p>{status?.note_count || 0} notes available with local autosave and tags.</p>
        </div>
      </div>
      <div class="health-card">
        <Icons.Code2 size={18} />
        <div>
          <strong>Workspace shell</strong>
          <p>{workspaceCount} active workspaces available for editing. Execution stays disabled unless supported.</p>
        </div>
      </div>
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

  .activity-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-height: 420px;
    overflow-y: auto;
    padding-right: 4px;
  }

  .activity-row {
    display: grid;
    grid-template-columns: 28px minmax(0, 1fr) auto;
    align-items: start;
    gap: 10px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--surface);
  }

  .activity-glyph {
    color: var(--blue);
  }

  .activity-copy strong {
    display: block;
    color: var(--text);
    font-size: 13px;
  }

  .activity-copy p {
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.5;
  }

  .activity-row > span {
    color: var(--muted);
    font-size: 11px;
    font-family: var(--ff-mono);
  }

  .health-stack {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .health-card {
    padding: 14px;
    display: flex;
    gap: 12px;
    align-items: flex-start;
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--surface);
  }

  .health-card strong {
    display: block;
    color: var(--text);
    font-size: 13px;
  }

  .health-card p {
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.5;
  }

  .activity-panel {
    min-height: 100%;
  }
</style>
