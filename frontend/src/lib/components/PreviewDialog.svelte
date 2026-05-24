<script lang="ts">
  import type { FileRecord } from "../api";
  import { fileContentUrl, fileDownloadUrl } from "../api";
  import { formatBytes } from "../utils/format";

  export let file: FileRecord | null;
  export let files: FileRecord[] = [];
  export let onClose: () => void;
  export let onShare: (file: FileRecord) => void;

  let current: FileRecord | null = null;
  $: current = file;

  $: index = current ? files.findIndex((f) => f.id === current!.id) : -1;
  $: prev = index > 0 ? files[index - 1] : null;
  $: next = index >= 0 && index < files.length - 1 ? files[index + 1] : null;

  function keydown(ev: KeyboardEvent) {
    if (ev.key === "Escape") onClose();
    if (ev.key === "ArrowLeft" && prev) current = prev;
    if (ev.key === "ArrowRight" && next) current = next;
  }

  function shareCurrent() {
    if (current) onShare(current);
  }
</script>

<svelte:window on:keydown={keydown} />

{#if current}
  <div class="preview-backdrop" role="presentation" on:click={onClose} on:keydown={() => undefined}>
    <div class="preview-dialog" role="dialog" aria-modal="true" tabindex="-1" on:click|stopPropagation on:keydown={() => undefined}>
      <header class="preview-head">
        <h2>{current.name}</h2>
        <button type="button" class="btn btn-ghost" on:click={onClose}>✕</button>
      </header>
      <div class="preview-body">
        {#if current.mime_type?.startsWith("image/")}
          <img class="preview-media" src={fileContentUrl(current.id)} alt={current.name} />
        {:else if current.mime_type?.startsWith("video/")}
          <video class="preview-media" src={fileContentUrl(current.id)} controls>
            <track kind="captions" />
          </video>
        {:else if current.mime_type?.startsWith("audio/")}
          <audio class="preview-audio" src={fileContentUrl(current.id)} controls></audio>
        {:else if current.mime_type === "application/pdf"}
          <iframe class="preview-frame" title={current.name} src={fileContentUrl(current.id)}></iframe>
        {:else if current.mime_type?.startsWith("text/") || current.name.match(/\.(md|txt|json|rs|js|ts|css)$/i)}
          <iframe class="preview-frame" title={current.name} src={fileContentUrl(current.id)}></iframe>
        {:else}
          <div class="empty-state">
            <strong>Preview not available</strong>
            <p>{current.mime_type || "Unknown type"} · {formatBytes(current.size_bytes)}</p>
          </div>
        {/if}
      </div>
      <footer class="preview-foot">
        <button type="button" class="btn btn-sm" disabled={!prev} on:click={() => prev && (current = prev)}>← Prev</button>
        <button type="button" class="btn btn-sm" disabled={!next} on:click={() => next && (current = next)}>Next →</button>
        <a class="btn btn-sm" href={fileDownloadUrl(current.id)} download>Download</a>
        <button type="button" class="btn btn-sm btn-primary" on:click={shareCurrent}>Share</button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .preview-backdrop {
    position: fixed;
    inset: 0;
    z-index: 60;
    background: rgba(0, 0, 0, 0.75);
    display: grid;
    place-items: center;
    padding: 24px;
  }
  .preview-dialog {
    width: min(960px, 100%);
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }
  .preview-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
  }
  .preview-head h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .preview-body {
    flex: 1;
    min-height: 200px;
    overflow: auto;
    display: grid;
    place-items: center;
    background: #000;
  }
  .preview-media {
    max-width: 100%;
    max-height: 70vh;
    object-fit: contain;
  }
  .preview-frame {
    width: 100%;
    height: 70vh;
    border: none;
    background: var(--bg-card);
  }
  .preview-audio {
    width: 100%;
    padding: 24px;
  }
  .preview-foot {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
  }
</style>
