/**
 * Common dialog and modal utilities
 * Eliminates redundant modal/dialog code across components
 */

export interface DialogState {
  isOpen: boolean;
  isLoading?: boolean;
  error?: string | null;
}

/**
 * Modal backdrop click handler
 * Shared logic for closing modals on backdrop click
 */
export function handleBackdropClick(
  event: MouseEvent,
  onClose: () => void
): void {
  if (event.target === event.currentTarget) {
    onClose();
  }
}

/**
 * Modal keyboard handler
 * Shared logic for closing modals on Escape key
 */
export function handleModalKeydown(
  event: KeyboardEvent,
  onClose: () => void
): void {
  if (event.key === 'Escape') {
    onClose();
  }
}
