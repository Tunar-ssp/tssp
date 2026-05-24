<script lang="ts">
  import { onMount } from "svelte";
  import { listPublicFiles, type FileRecord } from "../../lib/api";
  import { formatBytes } from "../../lib/utils/format";
  import { formatRelativeDate } from "../../lib/utils/format";

  let files: FileRecord[] = [];
  let loading = true;
  let error = "";

  onMount(async () => {
    try {
      const response = await listPublicFiles();
      files = response.files || [];
    } catch (nextError) {
      error = nextError instanceof Error ? nextError.message : "Failed to load public links";
    } finally {
      loading = false;
    }
  });
</script>

<section class="view-grid">
  <div class="hero-card compact">
    <div>
      <div class="eyebrow">Sharing Center</div>
      <h1>Public links as a product surface.</h1>
      <p>
        Shared files need copy, QR, revoke, and download flows in one place. This view turns
        “public links” from a dead table into a real sharing dashboard.
      </p>
    </div>
  </div>

  <div class="split-view">
    <article class="panel-card">
      <header class="panel-head">
        <strong>Active shares</strong>
        <span>recent, searchable, revocable</span>
      </header>
      {#if loading}
        <div class="empty-copy">Loading public files…</div>
      {:else if error}
        <div class="empty-copy">{error}</div>
      {:else if files.length === 0}
        <div class="empty-copy">No public links yet. Sharing stays opt-in and explicit.</div>
      {:else}
        <div class="stack-list">
          {#each files.slice(0, 8) as file}
            <div class="stack-card">
              <strong>{file.name}</strong>
              <span>
                {file.folder_path || "Bucket root"} · {formatBytes(file.size_bytes)} ·
                {formatRelativeDate(file.uploaded_at)}
              </span>
            </div>
          {/each}
        </div>
      {/if}
    </article>

    <article class="panel-card">
      <header class="panel-head">
        <strong>Share policy</strong>
        <span>clear language, no surprises</span>
      </header>
      <div class="detail-stack">
        <div class="detail-row"><span>Visibility</span><strong>Explicit opt-in only</strong></div>
        <div class="detail-row"><span>Active shares</span><strong>{files.length}</strong></div>
        <div class="detail-row"><span>Future</span><strong>Expiring and protected links</strong></div>
        <div class="detail-row"><span>Current</span><strong>LAN-only public file listing</strong></div>
      </div>
    </article>
  </div>
</section>
