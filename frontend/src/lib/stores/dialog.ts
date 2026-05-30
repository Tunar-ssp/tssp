import { writable } from 'svelte/store';

/**
 * Global, promise-based confirm/prompt dialogs.
 *
 * Replaces native window.confirm/window.prompt so product flows never trigger
 * blocking browser chrome. A single <AppDialog /> instance (mounted in App.svelte)
 * renders whatever is in this store and resolves the pending promise.
 */

export type DialogKind = 'confirm' | 'prompt';
export type DialogTone = 'default' | 'danger';

export interface DialogRequest {
  id: number;
  kind: DialogKind;
  title: string;
  message?: string;
  confirmLabel: string;
  cancelLabel: string;
  tone: DialogTone;
  // prompt-only
  placeholder?: string;
  defaultValue?: string;
  resolve: (value: boolean | string | null) => void;
}

export const activeDialog = writable<DialogRequest | null>(null);

let counter = 0;

export interface ConfirmOptions {
  title: string;
  message?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  tone?: DialogTone;
}

export interface PromptOptions {
  title: string;
  message?: string;
  placeholder?: string;
  defaultValue?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  tone?: DialogTone;
}

/** Resolve any open dialog as cancelled and clear it (used on route changes). */
export function dismissDialog() {
  activeDialog.update((current) => {
    if (current) {
      current.resolve(current.kind === 'confirm' ? false : null);
    }
    return null;
  });
}

export function confirmDialog(options: ConfirmOptions): Promise<boolean> {
  return new Promise<boolean>((resolve) => {
    // Replace any in-flight dialog (resolve it as cancelled first).
    dismissDialog();
    activeDialog.set({
      id: ++counter,
      kind: 'confirm',
      title: options.title,
      message: options.message,
      confirmLabel: options.confirmLabel ?? 'Confirm',
      cancelLabel: options.cancelLabel ?? 'Cancel',
      tone: options.tone ?? 'default',
      resolve: (value) => resolve(value === true),
    });
  });
}

export function promptDialog(options: PromptOptions): Promise<string | null> {
  return new Promise<string | null>((resolve) => {
    dismissDialog();
    activeDialog.set({
      id: ++counter,
      kind: 'prompt',
      title: options.title,
      message: options.message,
      placeholder: options.placeholder,
      defaultValue: options.defaultValue ?? '',
      confirmLabel: options.confirmLabel ?? 'Confirm',
      cancelLabel: options.cancelLabel ?? 'Cancel',
      tone: options.tone ?? 'default',
      resolve: (value) => resolve(typeof value === 'string' ? value : null),
    });
  });
}

/** Internal: called by the dialog component to settle and clear. */
export function settleDialog(request: DialogRequest, value: boolean | string | null) {
  request.resolve(value);
  activeDialog.update((current) => (current && current.id === request.id ? null : current));
}
