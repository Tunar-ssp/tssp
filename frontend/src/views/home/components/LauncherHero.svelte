<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { openCommandPalette } from '$lib/stores/ui';
  import { api } from '$lib/api';

  type Status = Awaited<ReturnType<typeof api.getStatus>>;

  interface Props {
    greeting?: string;
    dateLabel?: string;
    userName?: string;
    status?: Status;
    onUpload?: () => void;
    onNewNote?: () => void;
    onOpenWorkspace?: () => void;
    onOpenShare?: () => void;
  }

  let { greeting = 'Good morning', dateLabel = 'TODAY', userName = 'operator', status, onUpload, onNewNote, onOpenWorkspace, onOpenShare }: Props = $props();

  const quickActions = [
    { id: 'upload', label: 'Upload', icon: Icons.Upload, action: onUpload },
    { id: 'note', label: 'New note', icon: Icons.FileText, action: onNewNote },
    { id: 'workspace', label: 'Open workspace', icon: Icons.Code2, action: onOpenWorkspace },
    { id: 'share', label: 'Public links', icon: Icons.Share2, action: onOpenShare },
    { id: 'command', label: 'Command palette', icon: Icons.Search, action: () => openCommandPalette() },
  ];
</script>

<section class="hero">
  <div class="hero-copy">
    <div class="eyebrow">{dateLabel}</div>
    <h1>{greeting}, <span>{userName}</span></h1>
    <p>
      {#if status}
        Local cloud is available. <strong>{status.storage_bytes_used.toLocaleString()} bytes</strong> in use across
        <strong> {status.file_count}</strong> files, <strong>{status.note_count}</strong> notes.
      {:else}
        Restore your local cloud, notes, workspaces, and operations from one shell.
      {/if}
    </p>

    <button type="button" class="launcher-search" onclick={() => openCommandPalette()}>
      <Icons.Search size={18} />
      <span>Search files, notes, workspaces, public links</span>
      <kbd>⌘K</kbd>
    </button>

    <div class="quick-actions">
      {#each quickActions as action (action.id)}
        {@const Icon = action.icon}
        <button type="button" class="quick-action" onclick={action.action} disabled={!action.action}>
          <Icon size={14} />
          <span>{action.label}</span>
        </button>
      {/each}
    </div>
  </div>
</section>

<style>
  .hero {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .hero-copy {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .eyebrow {
    font-size: 11px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .hero-copy h1 {
    margin: 0;
    font-family: var(--ff-display);
    font-size: clamp(2.2rem, 4vw, 3rem);
    line-height: 0.95;
    letter-spacing: -0.04em;
  }

  .hero-copy h1 span {
    color: var(--green);
  }

  .hero-copy p {
    margin: 0;
    max-width: 680px;
    color: var(--text-2);
    font-size: 15px;
    line-height: 1.6;
  }

  .hero-copy strong {
    color: var(--text);
  }

  .launcher-search {
    height: 54px;
    width: min(100%, 640px);
    padding: 0 16px;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(12, 13, 18, 0.78);
    color: var(--text-2);
    display: flex;
    align-items: center;
    gap: 12px;
    box-shadow: 0 18px 42px rgba(0, 0, 0, 0.34);
    cursor: pointer;
  }

  .launcher-search span {
    flex: 1;
    text-align: left;
    font-size: 15px;
  }

  .launcher-search kbd {
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    padding: 4px 9px;
    font-size: 11px;
    font-family: var(--ff-mono);
  }

  .launcher-search:hover {
    border-color: var(--border-2);
    background: rgba(18, 20, 27, 0.9);
    color: var(--text);
  }

  .quick-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .quick-action {
    height: 34px;
    padding: 0 13px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: rgba(12, 13, 18, 0.65);
    color: var(--text-2);
    display: inline-flex;
    align-items: center;
    gap: 7px;
    cursor: pointer;
    transition: all 150ms;
  }

  .quick-action:hover:not(:disabled) {
    border-color: var(--border-2);
    background: rgba(18, 20, 27, 0.9);
    color: var(--text);
  }

  .quick-action:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
