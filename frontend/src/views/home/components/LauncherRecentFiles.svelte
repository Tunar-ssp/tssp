<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import FileThumb from '$lib/components/FileThumb.svelte';
  import { formatBytes, formatRelative } from '$lib/utils/formatters';
  import type { FileRecord } from '$lib/api';

  interface Props {
    files?: FileRecord[];
    onOpenFile?: (file: FileRecord) => void;
    onOpenDrive?: () => void;
  }

  let { files = [], onOpenFile, onOpenDrive }: Props = $props();

  function fileAccent(file: FileRecord) {
    if (file.mime_type.startsWith('image/')) return 'var(--green)';
    if (file.mime_type.startsWith('video/')) return 'var(--pink)';
    if (file.mime_type.includes('json') || file.mime_type.includes('javascript') || file.mime_type.includes('text/')) return 'var(--orange)';
    return 'var(--blue)';
  }
</script>

<article class="panel">
  <div class="panel-head">
    <div>
      <h2>Recent files</h2>
      <p>Jump back into Drive from the latest objects.</p>
    </div>
    <button type="button" class="link-btn" onclick={onOpenDrive}>Open Drive</button>
  </div>

  {#if files.length === 0}
    <div class="empty-card">
      <Icons.HardDrive size={24} />
      <strong>No files yet</strong>
      <p>Upload to Drive and the latest objects will appear here.</p>
    </div>
  {:else}
    <div class="file-grid">
      {#each files.slice(0, 8) as file (file.id)}
        <button type="button" class="file-card" onclick={() => onOpenFile?.(file)}>
          <div class="file-preview" style={`--accent:${fileAccent(file)}`}>
            <FileThumb id={file.id} name={file.name} mimeType={file.mime_type} iconSize={30} />
            {#if file.visibility === 'public'}
              <span class="chip">Public</span>
            {/if}
          </div>
          <div class="file-meta">
            <strong>{file.name}</strong>
            <span>{formatBytes(file.size_bytes)} · {formatRelative(file.updated_at || file.uploaded_at)}</span>
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

  .file-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 12px;
  }

  .file-card {
    border: 1px solid var(--border);
    background: var(--surface);
    color: inherit;
    cursor: pointer;
    border-radius: 14px;
    padding: 10px;
    text-align: left;
    transition: all 150ms;
  }

  .file-card:hover {
    border-color: var(--border-2);
    background: var(--surface-2);
  }

  .file-preview {
    position: relative;
    aspect-ratio: 4 / 3;
    border-radius: 10px;
    margin-bottom: 10px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    background:
      linear-gradient(135deg, color-mix(in srgb, var(--accent) 18%, transparent), rgba(30, 32, 40, 0.95)),
      var(--surface-2);
    color: var(--text);
  }

  .chip {
    position: absolute;
    top: 8px;
    left: 8px;
    height: 22px;
    padding: 0 8px;
    border-radius: 999px;
    background: rgba(110, 168, 255, 0.14);
    color: var(--blue);
    display: inline-flex;
    align-items: center;
    font-size: 10px;
    font-family: var(--ff-mono);
  }

  .file-meta {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .file-meta strong {
    font-size: 13px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-meta span {
    font-size: 11px;
    color: var(--muted);
  }

  .empty-card {
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--surface);
    min-height: 140px;
    justify-content: center;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 14px;
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
