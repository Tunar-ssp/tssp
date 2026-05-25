<script lang="ts">
  import type { FileRecord } from "../../lib/api";
  import { fileIconLabel, fileKindLabel } from "../../lib/utils/files";
  import { formatAbsoluteDate, formatBytes } from "../../lib/utils/format";

  export let files: FileRecord[] = [];
  export let selectedFileId: string | null = null;
  export let loading = false;
  export let error = "";
  export let onSelectFile: (file: FileRecord) => void;
</script>

<article class="panel-card">
  <header class="panel-head">
    <strong>Drive browser</strong>
    <span>grid, metadata, filters</span>
  </header>

  {#if loading}
    <div class="empty-copy">Loading files from the Rust API...</div>
  {:else if error}
    <div class="empty-copy">{error}</div>
  {:else if files.length === 0}
    <div class="empty-copy">No files yet. Upload into the bucket to populate the new Drive view.</div>
  {:else}
    <div class="file-card-list">
      {#each files as file}
        <button
          type="button"
          class:selected={selectedFileId === file.id}
          class="file-card"
          on:click={() => onSelectFile(file)}
        >
          <div class="file-icon">{fileIconLabel(file)}</div>
          <div class="file-copy">
            <strong>{file.name}</strong>
            <span>
              {fileKindLabel(file)} · {file.folder_path || "Bucket root"} · {formatBytes(file.size_bytes)} ·
              {formatAbsoluteDate(file.uploaded_at)}
            </span>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</article>
