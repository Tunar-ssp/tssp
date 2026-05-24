<script lang="ts">
  import type { FileRecord, ShareInfo } from "../../lib/api";
  import { formatAbsoluteDate, formatBytes } from "../../lib/utils/format";

  export let file: FileRecord | null = null;
  export let folderPath = "";
  export let shareInfo: ShareInfo | null = null;
  export let shareStatus = "";
  export let moveStatus = "";
  export let onMoveFolder: () => void;
  export let onToggleVisibility: () => void;
  export let onCopyShare: () => void;
  export let onRefreshShare: () => void;
</script>

<article class="panel-card">
  <header class="panel-head">
    <strong>Details</strong>
    <span>metadata and sharing</span>
  </header>

  {#if !file}
    <div class="empty-copy">Select a file to inspect its metadata, sharing state, and folder placement.</div>
  {:else}
    <div class="detail-stack">
      <div class="detail-row"><span>Name</span><strong>{file.name}</strong></div>
      <div class="detail-row"><span>Size</span><strong>{formatBytes(file.size_bytes)}</strong></div>
      <div class="detail-row"><span>Folder</span><strong>{file.folder_path || "Bucket root"}</strong></div>
      <div class="detail-row"><span>Uploaded</span><strong>{formatAbsoluteDate(file.uploaded_at)}</strong></div>
      <div class="detail-row"><span>Visibility</span><strong>{file.visibility}</strong></div>
      <div class="detail-row"><span>Tags</span><strong>{file.tags.join(", ") || "none"}</strong></div>
    </div>

    <div class="action-stack">
      <button type="button" class="btn btn-secondary" on:click={onToggleVisibility}>
        {file.visibility === "public" ? "Make private" : "Make public"}
      </button>
      <button type="button" class="btn btn-secondary" on:click={onRefreshShare}>Load share info</button>
      <button type="button" class="btn btn-secondary" on:click={onCopyShare} disabled={!shareInfo}>
        Copy public link
      </button>
      <button type="button" class="btn btn-secondary" on:click={onMoveFolder}>Move file</button>
    </div>

    <div class="detail-stack">
      <div class="detail-row"><span>Move target</span><strong>{folderPath || "enter a folder path"}</strong></div>
      <div class="detail-row"><span>Share</span><strong>{shareInfo?.public_url || shareStatus || "private"}</strong></div>
      <div class="detail-row"><span>QR</span><strong>{shareInfo?.qr_terminal ? "available" : "not loaded"}</strong></div>
      <div class="detail-row"><span>Status</span><strong>{moveStatus || "idle"}</strong></div>
    </div>
  {/if}
</article>
