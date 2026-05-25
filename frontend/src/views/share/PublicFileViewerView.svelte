<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import FilePreviewModal from '$lib/components/FilePreviewModal.svelte';
  import Btn from '$lib/components/Btn.svelte';

  interface SharedFile {
    id: string;
    name: string;
    mime_type: string;
    size_bytes: number;
    created_at: number;
    shared_by: string;
    expires_at?: number;
  }

  let file = $state<SharedFile | null>(null);
  let isLoading = $state(true);
  let error = $state('');
  let isExpired = $state(false);
  let showPreview = $state(true);

  async function loadSharedFile(shareId: string) {
    isLoading = true;
    error = '';

    try {
      const response = await fetch(`/api/share/${shareId}`);
      if (!response.ok) {
        if (response.status === 404) {
          error = 'Shared file not found or has been deleted';
        } else if (response.status === 410) {
          error = 'This shared file has expired';
          isExpired = true;
        } else {
          error = 'Unable to load shared file';
        }
        return;
      }

      file = await response.json();

      if (file?.expires_at && file.expires_at < Date.now() / 1000) {
        error = 'This shared file has expired';
        isExpired = true;
      }
    } catch (e) {
      error = 'Failed to load shared file';
    } finally {
      isLoading = false;
    }
  }

  async function downloadFile() {
    if (!file) return;

    try {
      const response = await fetch(`/api/files/${file.id}/content`);
      if (!response.ok) throw new Error('Failed to download');

      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = file.name;
      a.click();
      window.URL.revokeObjectURL(url);
    } catch (e) {
      error = 'Failed to download file';
    }
  }

  function formatBytes(bytes: number) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }

  function formatDate(timestamp: number) {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  // Load file from URL params (would be handled by routing in actual app)
  if (typeof window !== 'undefined') {
    const shareId = window.location.pathname.split('/').pop();
    if (shareId) {
      loadSharedFile(shareId);
    }
  }
</script>

<div class="public-viewer">
  {#if isLoading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading shared file...</p>
    </div>
  {:else if error}
    <div class="error-state">
      <Icons.AlertCircle size={48} />
      <h2>Unable to Load File</h2>
      <p>{error}</p>
      {#if isExpired}
        <p class="expired-info">The file owner may be able to re-share it with you.</p>
      {/if}
      <Btn kind="ghost" on:click={() => (window.location.href = '/')}>
        <Icons.Home size={14} />
        Return Home
      </Btn>
    </div>
  {:else if file}
    <div class="viewer-container">
      <div class="viewer-header">
        <div class="file-info">
          <h1>{file.name}</h1>
          <div class="file-meta">
            <span>{formatBytes(file.size_bytes)}</span>
            <span>•</span>
            <span>Shared by {file.shared_by}</span>
            <span>•</span>
            <span>{formatDate(file.created_at)}</span>
          </div>
          {#if file.expires_at}
            <div class="expiry-notice">
              <Icons.Clock size={14} />
              Expires {formatDate(file.expires_at)}
            </div>
          {/if}
        </div>
        <Btn kind="primary" on:click={downloadFile}>
          <Icons.Download size={14} />
          Download
        </Btn>
      </div>

      {#if showPreview}
        <div class="preview-section">
          <FilePreviewModal
            {file}
            isOpen={true}
            onDownload={downloadFile}
          />
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .public-viewer {
    width: 100%;
    height: 100%;
    background: var(--bg);
    color: var(--text);
    font-family: var(--ff-sans);
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: var(--s-3);
    color: var(--muted);
  }

  .spinner {
    width: 48px;
    height: 48px;
    border: 3px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: var(--s-4);
    color: var(--muted);
    padding: var(--s-6);
    text-align: center;
  }

  .error-state h2 {
    margin: 0;
    font-size: var(--fs-24);
    color: var(--text);
  }

  .error-state p {
    margin: 0;
    font-size: var(--fs-14);
    color: var(--muted);
  }

  .expired-info {
    font-size: var(--fs-12);
    color: var(--orange);
  }

  .viewer-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: var(--s-6);
    gap: var(--s-6);
  }

  .viewer-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-6);
    padding-bottom: var(--s-6);
    border-bottom: 1px solid var(--border);
  }

  .file-info {
    flex: 1;
  }

  .file-info h1 {
    margin: 0;
    font-size: var(--fs-32);
    font-weight: 700;
    color: var(--text);
    word-break: break-word;
  }

  .file-meta {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-top: var(--s-3);
    font-size: var(--fs-13);
    color: var(--muted);
  }

  .expiry-notice {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-top: var(--s-3);
    padding: var(--s-3);
    background: rgba(251, 191, 36, 0.1);
    border-radius: var(--r-2);
    color: var(--warning);
    font-size: var(--fs-12);
  }

  .preview-section {
    flex: 1;
    overflow: auto;
  }
</style>
