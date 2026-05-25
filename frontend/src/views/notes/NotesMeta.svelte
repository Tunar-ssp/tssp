<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    createdAt: number;
    updatedAt: number;
    wordCount: number;
    blockCount: number;
  }

  let { createdAt, updatedAt, wordCount, blockCount }: $$Props = $props();

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }
</script>

<div class="notes-meta">
  <div class="meta-card">
    <span class="meta-label">Created</span>
    <strong>{formatDate(createdAt)}</strong>
  </div>
  <div class="meta-card">
    <span class="meta-label">Updated</span>
    <strong>{formatDate(updatedAt)}</strong>
  </div>
  <div class="meta-card">
    <span class="meta-label">Words</span>
    <strong>{wordCount}</strong>
  </div>
  <div class="meta-card">
    <span class="meta-label">Blocks</span>
    <strong>{blockCount}</strong>
  </div>

  <div class="meta-note">
    <Icons.Info size={16} />
    <p>Note sharing and linked references are not available from the backend yet, so this editor only exposes real storage-backed actions.</p>
  </div>
</div>

<style>
  .notes-meta {
    flex: 1;
    padding: 16px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .meta-card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--surface-2);
  }

  .meta-label {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .meta-card strong {
    font-size: 18px;
    color: var(--text);
  }

  .meta-note {
    display: flex;
    gap: 8px;
    padding: 12px;
    margin-top: auto;
    border-radius: 12px;
    border: 1px solid rgba(110, 168, 255, 0.14);
    background: rgba(110, 168, 255, 0.06);
    color: var(--text-2);
    font-size: 12px;
  }

  .meta-note :global(svg) {
    flex-shrink: 0;
    color: var(--blue);
  }

  .meta-note p {
    margin: 0;
    line-height: 1.5;
  }
</style>
