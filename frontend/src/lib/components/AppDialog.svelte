<script lang="ts">
  import { activeDialog, settleDialog, type DialogRequest } from '$lib/stores/dialog';
  import { activeOverlays } from '$lib/stores/ui';

  let inputRef: HTMLInputElement | null = $state(null);
  let value = $state('');
  let lastId = $state<number | null>(null);

  // Sync local input value + focus whenever a new dialog appears.
  $effect(() => {
    const dialog = $activeDialog;
    if (dialog && dialog.id !== lastId) {
      lastId = dialog.id;
      value = dialog.defaultValue ?? '';
      activeOverlays.push('modal');
      queueMicrotask(() => {
        if (dialog.kind === 'prompt') {
          inputRef?.focus();
          inputRef?.select();
        }
      });
    } else if (!dialog && lastId !== null) {
      lastId = null;
      activeOverlays.remove('modal');
    }
  });

  function cancel(dialog: DialogRequest) {
    settleDialog(dialog, dialog.kind === 'confirm' ? false : null);
  }

  function accept(dialog: DialogRequest) {
    if (dialog.kind === 'prompt') {
      settleDialog(dialog, value);
    } else {
      settleDialog(dialog, true);
    }
  }

  function onKeydown(e: KeyboardEvent, dialog: DialogRequest) {
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      cancel(dialog);
    } else if (e.key === 'Enter' && dialog.kind === 'prompt') {
      e.preventDefault();
      e.stopPropagation();
      accept(dialog);
    }
  }
</script>

{#if $activeDialog}
  {@const dialog = $activeDialog}
  <div
    class="dialog-backdrop"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) cancel(dialog);
    }}
    onkeydown={(e) => onKeydown(e, dialog)}
  >
    <div class="dialog" role="dialog" aria-modal="true" aria-label={dialog.title}>
      <h2 class="dialog-title">{dialog.title}</h2>
      {#if dialog.message}
        <p class="dialog-message">{dialog.message}</p>
      {/if}
      {#if dialog.kind === 'prompt'}
        <input
          bind:this={inputRef}
          bind:value
          class="dialog-input"
          type="text"
          placeholder={dialog.placeholder ?? ''}
          onkeydown={(e) => onKeydown(e, dialog)}
        />
      {/if}
      <div class="dialog-actions">
        <button type="button" class="dialog-btn ghost" onclick={() => cancel(dialog)}>
          {dialog.cancelLabel}
        </button>
        <button
          type="button"
          class="dialog-btn"
          class:danger={dialog.tone === 'danger'}
          class:primary={dialog.tone !== 'danger'}
          disabled={dialog.kind === 'prompt' && !value.trim()}
          onclick={() => accept(dialog)}
        >
          {dialog.confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2200;
    padding: 24px;
    animation: dialogFade var(--duration-normal, 160ms) ease;
  }

  @keyframes dialogFade {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .dialog {
    width: min(440px, 100%);
    background: var(--surface);
    border: 1px solid var(--border-2);
    border-radius: 16px;
    box-shadow: var(--shadow-modal);
    padding: 22px 22px 18px;
    animation: dialogIn var(--duration-normal, 160ms) ease;
  }

  @keyframes dialogIn {
    from { opacity: 0; transform: translateY(8px) scale(0.98); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  .dialog-title {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
  }

  .dialog-message {
    margin: 8px 0 0;
    font-size: 13.5px;
    line-height: 1.5;
    color: var(--text-2);
  }

  .dialog-input {
    width: 100%;
    margin-top: 16px;
    padding: 10px 12px;
    background: var(--surface-2);
    border: 1px solid var(--border-2);
    border-radius: 10px;
    color: var(--text);
    font-size: 14px;
    outline: none;
  }

  .dialog-input:focus {
    border-color: var(--blue);
  }

  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 20px;
  }

  .dialog-btn {
    padding: 8px 16px;
    border-radius: 10px;
    border: 1px solid var(--border-2);
    background: var(--surface-2);
    color: var(--text);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: filter 120ms ease, background 120ms ease;
  }

  .dialog-btn.ghost:hover {
    background: var(--surface-3);
  }

  .dialog-btn.primary {
    background: var(--blue);
    border-color: var(--blue);
    color: #fff;
  }

  .dialog-btn.danger {
    background: var(--danger);
    border-color: var(--danger);
    color: #fff;
  }

  .dialog-btn.primary:hover,
  .dialog-btn.danger:hover {
    filter: brightness(1.08);
  }

  .dialog-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
