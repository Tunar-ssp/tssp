/**
 * Centralized component exports
 * Organized by category for easier discovery and imports
 * See FOLDER_STRUCTURE.md for detailed organization guide
 */

// ============================================================================
// FORM & INPUT COMPONENTS
// ============================================================================
export { default as FormInput } from './FormInput.svelte';
export { default as Button } from './Button.svelte';
export { default as Btn } from './Btn.svelte';
export { default as Badge } from './Badge.svelte';
export { default as Pill } from './Pill.svelte';
export { default as ColorPicker } from './ColorPicker.svelte';

// ============================================================================
// PRIMITIVE UI COMPONENTS
// ============================================================================
export { default as Card } from './Card.svelte';
export { default as Bar } from './Bar.svelte';
export { default as Ring } from './Ring.svelte';
export { default as ProgressBar } from './ProgressBar.svelte';
export { default as ProgressRing } from './ProgressRing.svelte';
export { default as StatusDot } from './StatusDot.svelte';
export { default as Tooltip } from './Tooltip.svelte';
export { default as Kbd } from './Kbd.svelte';
export { default as Avatar } from './Avatar.svelte';
export { default as Breadcrumb } from './Breadcrumb.svelte';

// ============================================================================
// STATE & FEEDBACK COMPONENTS
// ============================================================================
export { default as Toast } from './Toast.svelte';
export { default as Banner } from './Banner.svelte';
export { default as LoadingState } from './LoadingState.svelte';
export { default as ErrorState } from './ErrorState.svelte';
export { default as EmptyState } from './EmptyState.svelte';
export { default as SystemStatus } from './SystemStatus.svelte';
export { default as StatusBar } from './StatusBar.svelte';

// ============================================================================
// MODAL & DIALOG COMPONENTS
// ============================================================================
export { default as Modal } from './Modal.svelte';
export { default as Sheet } from './Sheet.svelte';
export { default as PreviewDialog } from './PreviewDialog.svelte';
export { default as FilePreviewModal } from './FilePreviewModal.svelte';
export { default as SharingModal } from './SharingModal.svelte';
export { default as SafeConsole } from './SafeConsole.svelte';
export { default as QRCodeGenerator } from './QRCodeGenerator.svelte';

// ============================================================================
// NAVIGATION COMPONENTS
// ============================================================================
export { default as TopBar } from './TopBar.svelte';
export { default as Dock } from './Dock.svelte';
export { default as TabBar } from './TabBar.svelte';
export { default as SlashMenu } from './SlashMenu.svelte';
export { default as ProfileMenu } from './ProfileMenu.svelte';
export { default as CommandPalette } from './CommandPalette.svelte';

// ============================================================================
// FILE & FOLDER COMPONENTS
// ============================================================================
export { default as FileIcon } from './FileIcon.svelte';
export { default as FileExplorer } from './FileExplorer.svelte';
export { default as FileGrid } from './FileGrid.svelte';
export { default as FolderTree } from './FolderTree.svelte';
export { default as UploadArea } from './UploadArea.svelte';
export { default as UploadQueue } from './UploadQueue.svelte';
export { default as DeviceManager } from './DeviceManager.svelte';

// ============================================================================
// EDITOR COMPONENTS
// ============================================================================
export { default as CodeEditor } from './CodeEditor.svelte';
export { default as MonacoEditor } from './MonacoEditor.svelte';
export { default as NotesList } from './NotesList.svelte';
export { default as MarkdownPreview } from './MarkdownPreview.svelte';
export { default as FindWidget } from './FindWidget.svelte';
export { default as FindReplaceWidget } from './FindReplaceWidget.svelte';

// ============================================================================
// SHARED/LAYOUT COMPONENTS
// ============================================================================
export { default as ContextMenu } from './ContextMenu.svelte';
export { default as NotificationCenter } from './NotificationCenter.svelte';
export { default as SettingsTray } from './SettingsTray.svelte';
export { default as ShortcutsOverlay } from './ShortcutsOverlay.svelte';
export { default as Outline } from './Outline.svelte';
export { default as WorkspaceList } from './WorkspaceList.svelte';

// ============================================================================
// COMPONENT SUBDIRECTORY EXPORTS
// Organized categories with dedicated index files
// ============================================================================
export * from './editors/index';
export * from './file/index';
export * from './layout/index';
export * from './navigation/index';
export * from './modals/index';
export * from './primitives/index';
export * from './state/index';
export * from './misc/index';
export * from './overlay/index';

// ============================================================================
// TYPE EXPORTS
// ============================================================================
export type { ButtonProps, PillProps, CardProps, StatusDotProps, ModalProps, SheetProps, TooltipProps } from './primitives.svelte';
