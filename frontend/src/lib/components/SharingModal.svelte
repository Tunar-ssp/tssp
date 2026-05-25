<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import QRCodeGenerator from './QRCodeGenerator.svelte';
  import Btn from './Btn.svelte';
  import Kbd from './Kbd.svelte';

  interface $$Props {
    file?: any;
    isOpen?: boolean;
    onClose?: () => void;
    onShare?: (fileId: string, isPublic: boolean) => void;
    class?: string;
  }

  let {
    file,
    isOpen = false,
    onClose,
    onShare,
    class: className,
  } = $props<$$Props>();

  let isPublic = $state(false);
  let expiryDays = $state(7);
  let shareLink = $state('');
  let showQR = $state(false);

  $effect(() => {
    if (file) {
      isPublic = file.public || false;
      if (isPublic) {
        shareLink = `${window.location.origin}/share/${file.id}`;
      }
    }
  });

  async function togglePublic() {
    if (onShare) {
      onShare(file.id, !isPublic);
      isPublic = !isPublic;
      if (isPublic) {
        shareLink = `${window.location.origin}/share/${file.id}`;
      }
    }
  }

  function copyLink() {
    if (shareLink) {
      navigator.clipboard.writeText(shareLink);
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && onClose) {
      onClose();
    }
  }
</script>

{#if isOpen && file}
  <div class="sharing-backdrop" on:click={handleBackdropClick}>
    <div class="sharing-modal {className || ''}">
      <div class="sharing-header">
        <h2>Share "{file.name}"</h2>
        {#if onClose}
          <button class="sharing-close" on:click={onClose} aria-label="Close">
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
              class="toggle-btn"
              class:active={isPublic}
              on:click={togglePublic}
            >
              <div class="toggle-slider"></div>
            </button>
          </div>

          {#if isPublic && shareLink}
            <div class="share-link-container">
              <div class="share-link">
                <input
                  type="text"
                  value={shareLink}
                  readonly
                  class="link-input"
                />
                <button class="copy-btn" on:click={copyLink} title="Copy link">
                  <Icons.Copy size={16} />
                </button>
              </div>

              <button
                class="qr-toggle-btn"
                on:click={() => (showQR = !showQR)}
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
              <h3>Expiry Date</h3>
              <p>Automatically revoke access after</p>
            </div>
          </div>

          <div class="expiry-inputs">
            <select bind:value={expiryDays} disabled={!isPublic} class="expiry-select">
              <option value={1}>1 day</option>
              <option value={7}>7 days</option>
              <option value={30}>30 days</option>
              <option value={90}>90 days</option>
              <option value={0}>Never</option>
            </select>
          </div>
        </div>

        <div class="share-option">
          <h3>Keyboard Shortcut</h3>
          <p>Share or unshare this file</p>
          <div class="shortcut-display">
            <Kbd>Ctrl</Kbd>
            <span>+</span>
            <Kbd>Shift</Kbd>
            <span>+</span>
            <Kbd>S</Kbd>
          </div>
        </div>
      </div>

      <div class="sharing-footer">
        {#if onClose}
          <Btn kind="ghost" on:click={onClose}>Done</Btn>
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
