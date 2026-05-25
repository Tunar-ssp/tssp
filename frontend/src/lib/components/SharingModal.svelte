<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import QRCodeGenerator from './QRCodeGenerator.svelte';
  import Btn from './Btn.svelte';
  import { api, type VisibilityResponse } from '$lib/api';
  import { success, error } from '$lib/stores/notifications';

  interface $$Props {
    file?: any;
    isOpen?: boolean;
    onClose?: () => void;
    onShare?: (fileId: string, isPublic: boolean) => Promise<VisibilityResponse | null | void>;
    class?: string;
  }

  let {
    file,
    isOpen = false,
    onClose,
    onShare,
    class: className,
  }: $$Props = $props();

  let isPublic = $state(false);
  let shareLink = $state('');
  let showQR = $state(false);
  let isUpdating = $state(false);
  let statusMessage = $state('');

  $effect(() => {
    if (isOpen && file) {
      isPublic = file.public || file.visibility === 'public';
      shareLink = file.public_token ? `${window.location.origin}/p/${file.public_token}` : '';
      statusMessage = '';
      showQR = false;
      if (isPublic) void loadShareLink(file);
    }
  });

  async function togglePublic() {
    if (!file || !onShare || isUpdating) return;

    isUpdating = true;
    statusMessage = '';
    try {
      const nextPublic = !isPublic;
      const result = await onShare(file.id, nextPublic);
      isPublic = nextPublic;
      file = result?.file ?? { ...file, public: nextPublic, visibility: nextPublic ? 'public' : 'private' };
      shareLink = result?.public_url || '';

      if (nextPublic && !shareLink) {
        await loadShareLink(file);
      }

      if (!nextPublic) {
        showQR = false;
        statusMessage = 'This file is private again.';
      }
    } catch (err) {
      statusMessage = err instanceof Error ? err.message : 'Could not update sharing.';
      error('Share Failed', statusMessage);
    } finally {
      isUpdating = false;
    }
  }

  async function loadShareLink(targetFile: any) {
    if (!targetFile?.id || !(targetFile.public || targetFile.visibility === 'public')) return;

    try {
      statusMessage = 'Loading public link...';
      const response = await api.getFileShare(targetFile.id);
      shareLink = response.public_url;
      statusMessage = '';
    } catch (err) {
      statusMessage = err instanceof Error ? err.message : 'Public link is not ready yet.';
    }
  }

  async function copyLink() {
    if (!shareLink) return;
    await navigator.clipboard.writeText(shareLink);
    success('Copied', 'Public link copied to clipboard');
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && onClose) {
      onClose();
    }
  }

  function handleBackdropKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && onClose) onClose();
  }
</script>

{#if isOpen && file}
  <div
    class="sharing-backdrop"
    role="presentation"
    tabindex="-1"
    onclick={handleBackdropClick}
    onkeydown={handleBackdropKeydown}
  >
    <div class="sharing-modal {className || ''}" role="dialog" aria-modal="true" aria-labelledby="sharing-title">
      <div class="sharing-header">
        <h2 id="sharing-title">Share "{file.name}"</h2>
        {#if onClose}
          <button type="button" class="sharing-close" onclick={onClose} aria-label="Close">
            <Icons.X size={18} />
          </button>
        {/if}
      </div>

      <div class="sharing-body">
        <div class="share-option">
          <div class="option-header">
            <div>
              <h3>Make Public</h3>
              <p>Anyone with the link can view and download</p>
            </div>
            <button
              type="button"
              class="toggle-btn"
              class:active={isPublic}
              aria-label={isPublic ? 'Make file private' : 'Make file public'}
              aria-pressed={isPublic}
              disabled={isUpdating}
              onclick={togglePublic}
            >
              <div class="toggle-slider"></div>
            </button>
          </div>

          {#if statusMessage}
            <p class="share-status">{statusMessage}</p>
          {/if}

          {#if isPublic && shareLink}
            <div class="share-link-container">
              <div class="share-link">
                <input
                  type="text"
                  value={shareLink}
                  readonly
                  class="link-input"
                />
                <button type="button" class="copy-btn" onclick={copyLink} title="Copy link">
                  <Icons.Copy size={16} />
                </button>
              </div>

              <button
                type="button"
                class="qr-toggle-btn"
                onclick={() => (showQR = !showQR)}
              >
                <Icons.QrCode size={14} />
                {showQR ? 'Hide' : 'Show'} QR Code
              </button>

              {#if showQR}
                <div class="qr-generator">
                  <QRCodeGenerator data={shareLink} size={256} />
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <div class="share-option">
          <div class="option-header">
            <div>
              <h3>Security</h3>
              <p>Public links are tokenized and can be revoked by making the file private.</p>
            </div>
          </div>
        </div>
      </div>

      <div class="sharing-footer">
        {#if onClose}
          <Btn kind="ghost" onclick={onClose}>Done</Btn>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .sharing-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
    animation: fadeIn var(--duration-normal) var(--ease-smooth);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .sharing-modal {
    width: 90%;
    max-width: 500px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-3);
    box-shadow: var(--shadow-modal);
    display: flex;
    flex-direction: column;
    animation: modalSlideIn var(--duration-normal) var(--ease-smooth);
  }

  @keyframes modalSlideIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .sharing-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-6);
    border-bottom: 1px solid var(--border);
  }

  .sharing-header h2 {
    margin: 0;
    font-size: var(--fs-18);
    font-weight: 600;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sharing-close {
    width: 32px;
    height: 32px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-2);
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .sharing-close:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .sharing-close:focus-visible,
  .toggle-btn:focus-visible,
  .copy-btn:focus-visible,
  .qr-toggle-btn:focus-visible {
    outline: 2px solid var(--blue);
    outline-offset: 2px;
  }

  .sharing-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-6);
    display: flex;
    flex-direction: column;
    gap: var(--s-6);
  }

  .share-option {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .share-option h3 {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
  }

  .share-option p {
    margin: 0;
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .option-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-4);
  }

  .toggle-btn {
    width: 44px;
    height: 24px;
    padding: 2px;
    border: none;
    background: var(--surface-2);
    border-radius: var(--r-full);
    cursor: pointer;
    display: flex;
    align-items: center;
    transition: all var(--duration-quick) var(--ease-smooth);
    flex-shrink: 0;
  }

  .toggle-btn:disabled {
    cursor: progress;
    opacity: 0.65;
  }

  .toggle-btn.active {
    background: var(--green);
  }

  .toggle-slider {
    width: 20px;
    height: 20px;
    background: white;
    border-radius: 50%;
    transition: transform var(--duration-quick) var(--ease-smooth);
  }

  .toggle-btn.active .toggle-slider {
    transform: translateX(20px);
  }

  .share-status {
    margin: 0;
    color: var(--muted);
    font-size: var(--fs-12);
  }

  .share-link-container {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .share-link {
    display: flex;
    gap: var(--s-2);
  }

  .link-input {
    flex: 1;
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    border-radius: var(--r-2);
    font-family: var(--ff-mono);
    font-size: var(--fs-12);
  }

  .copy-btn {
    width: 36px;
    height: 36px;
    padding: 0;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    border-radius: var(--r-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .copy-btn:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .qr-toggle-btn {
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    border-radius: var(--r-2);
    font-size: var(--fs-12);
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--s-2);
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .qr-toggle-btn:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .qr-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-6);
    background: var(--surface-2);
    border-radius: var(--r-2);
  }

  .qr-box {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 150px;
    height: 150px;
    background: white;
    border-radius: var(--r-2);
    color: var(--text);
  }

  .qr-placeholder p {
    margin: 0;
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .expiry-inputs {
    display: flex;
    gap: var(--s-3);
  }

  .expiry-select {
    flex: 1;
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text);
    border-radius: var(--r-2);
    font-family: var(--ff-sans);
    font-size: var(--fs-13);
    cursor: pointer;
  }

  .expiry-select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .shortcut-display {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-3);
    background: var(--surface-2);
    border-radius: var(--r-2);
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .sharing-footer {
    display: flex;
    justify-content: flex-end;
    padding: var(--s-4) var(--s-6);
    border-top: 1px solid var(--border);
  }
</style>
