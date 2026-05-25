<script lang="ts">
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';

  interface Props {
    data: string;
    size?: number;
    level?: 'L' | 'M' | 'Q' | 'H';
  }

  let { data, size = 256, level = 'M' }: Props = $props();

  let canvas = $state<HTMLCanvasElement | undefined>(undefined);
  let qrCode = $state<any>(null);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      const QRCode = (await import('qrcode')).default;

      if (canvas) {
        await QRCode.toCanvas(canvas, data, {
          errorCorrectionLevel: level,
          width: size,
          margin: 2,
          color: {
            dark: '#000000',
            light: '#ffffff',
          },
        });

        qrCode = QRCode;
        isLoading = false;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to generate QR code';
      isLoading = false;
    }
  });

  function downloadQRCode() {
    if (!canvas) return;

    const link = document.createElement('a');
    link.href = canvas.toDataURL('image/png');
    link.download = `qr-code-${Date.now()}.png`;
    link.click();
  }

  function copyToClipboard() {
    if (!canvas) return;

    canvas.toBlob((blob) => {
      if (blob) {
        navigator.clipboard.write([
          new ClipboardItem({ 'image/png': blob }),
        ]);
      }
    });
  }
</script>

<div class="qr-container">
  {#if isLoading}
    <div class="loading">
      <div class="spinner-wrap">
        <Icons.Loader2 size={24} />
      </div>
      <p>Generating QR code...</p>
    </div>
  {:else if error}
    <div class="error">
      <Icons.AlertCircle size={24} />
      <p>{error}</p>
    </div>
  {:else}
    <div class="qr-content">
      <canvas bind:this={canvas} class="qr-canvas"></canvas>
      <div class="qr-actions">
        <button class="btn btn-primary" title="Download QR code" onclick={downloadQRCode}>
          <Icons.Download size={14} />
          Download
        </button>
        <button class="btn btn-secondary" title="Copy to clipboard" onclick={copyToClipboard}>
          <Icons.Copy size={14} />
          Copy
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .qr-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-4);
    background: var(--surface);
    border-radius: var(--r-2);
    border: 1px solid var(--border);
  }

  .loading,
  .error {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-4);
  }

  .spinner-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .loading p,
  .error p {
    margin: 0;
    color: var(--text-2);
    font-size: var(--fs-12);
  }

  .error {
    color: var(--danger);
  }

  .error p {
    color: var(--danger);
  }

  .qr-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-3);
  }

  .qr-canvas {
    border: 2px solid var(--border);
    border-radius: var(--r-1);
    background: white;
    padding: 8px;
  }

  .qr-actions {
    display: flex;
    gap: var(--s-2);
  }

  .btn {
    display: flex;
    align-items: center;
    gap: var(--s-1);
    padding: var(--s-2) var(--s-3);
    border-radius: var(--r-1);
    border: none;
    font-size: var(--fs-12);
    font-weight: 600;
    cursor: pointer;
    transition: all var(--duration-quick);
  }

  .btn-primary {
    background: var(--blue);
    color: white;
  }

  .btn-primary:hover {
    background: var(--blue);
    opacity: 0.9;
  }

  .btn-secondary {
    background: var(--surface-2);
    color: var(--text);
    border: 1px solid var(--border);
  }

  .btn-secondary:hover {
    background: var(--surface);
    border-color: var(--blue);
  }
</style>
