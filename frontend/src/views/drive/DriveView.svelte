<script lang="ts">
  import { get } from 'svelte/store';
  import { onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import {
    api,
    type FileRecord,
    type FolderEntry,
    type VisibilityResponse,
  } from '$lib/api';
  import { driveStateManager } from '$lib/services/driveStateService';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import { isDriveFolder, createWorkspaceFromFolder } from '$lib/services/driveWorkspaceIntegration';
  import FilePreviewModal from '$lib/components/FilePreviewModal.svelte';
  import SharingModal from '$lib/components/SharingModal.svelte';
  import MoveFileDialog from './components/modals/MoveFileDialog.svelte';
  import RenameFileDialog from './components/modals/RenameFileDialog.svelte';
  import DriveDetailsPanel from './components/panels/DriveDetailsPanel.svelte';
  import DriveHeader from './DriveHeader.svelte';
  import DriveSidebar from './DriveSidebar.svelte';
  import DriveContent from './DriveContent.svelte';
  import DriveToolbarRow from './components/toolbar/DriveToolbarRow.svelte';
  import KeyboardHelpModal from '$lib/components/KeyboardHelpModal.svelte';
  import { consumeSelectionIntent, preferences, setDefaultDriveView, selectionIntent } from '$lib/stores/ui';
  import { error, success, info } from '$lib/stores/notifications';
  import { clipboard } from '$lib/stores/clipboard';
  import { formatBytes, formatRelative, registerKeyboardShortcuts } from '$lib/utils';

  type DriveLens = 'all' | 'images' | 'videos' | 'documents' | 'public' | 'trash';
  type Status = Awaited<ReturnType<typeof api.getStatus>>;

  let files = $state<FileRecord[]>([]);
  let trash = $state<FileRecord[]>([]);
  let folderEntries = $state<FolderEntry[]>([]);
  let status = $state<Status | null>(null);
  let selectedFile = $state<FileRecord | null>(null);
  let selectedFileIds = $state<Set<string>>(new Set());
  let lastSelectedIndex = $state<number>(-1);
  let activeTagFilter = $state<string | null>(null);
  let availableTags = $derived.by(() => {
    const tagFreq = new Map<string, number>();
    filteredLibraryFiles.forEach(file => {
      (file.tags || []).forEach(tag => {
        tagFreq.set(tag, (tagFreq.get(tag) || 0) + 1);
      });
    });
    return Array.from(tagFreq.entries())
      .sort((a, b) => b[1] - a[1])
      .map(([tag]) => tag);
  });

  let imageCount = $derived(files.filter(f => (f.mime_type || '').startsWith('image/')).length);
  let videoCount = $derived(files.filter(f => (f.mime_type || '').startsWith('video/')).length);
  let documentCount = $derived(files.filter(f => {
    const mime = f.mime_type || '';
    if (mime.startsWith('image/') || mime.startsWith('video/') || mime.startsWith('audio/')) return false;
    return mime.startsWith('text/') || mime.startsWith('application/') || mime.includes('pdf') || mime.includes('word') || mime.includes('sheet') || mime.includes('presentation');
  }).length);
  let previewFile = $state<FileRecord | null>(null);
  let shareFile = $state<FileRecord | null>(null);
  let moveDialogFile = $state<FileRecord | null>(null);
  let renameDialogFile = $state<FileRecord | null>(null);
  let isMoveDialogOpen = $state(false);
  let isRenameDialogOpen = $state(false);
  let bulkMoveTarget = $state<string | null>(null);
  let isMoving = $state(false);
  let isRenaming = $state(false);
  let isLoading = $state(true);
  let isLoadingMore = $state(false);
  let trashLoading = $state(false);
  let nextCursor = $state<string | undefined>(undefined);
  let hasMore = $state(false);
  let currentFolder = $state('');
  let filterQuery = $state('');
  let activeLens = $state<DriveLens>('all');
  let viewMode = $state<'grid' | 'list'>(get(preferences).defaultDriveView || 'grid');
  let sortBy = $state<'name' | 'date' | 'size'>('date');
  let sortOrder = $state<'asc' | 'desc'>('desc');
  let contextMenu = $state({
    visible: false,
    x: 0,
    y: 0,
    file: null as FileRecord | null,
  });
  let showBulkMoveMenu = $state(false);
  let showKeyboardHelp = $state(false);
  let sidebarOpen = $state(true);
  let detailsPanelOpen = $state(true);

  let clipboardFileIds = $derived(clipboard.getItemIds());
  let clipboardOperation: 'copy' | 'cut' | null = $state(null);

  onMount(() => {
    const handleExternalRefresh = () => {
      void loadLibrary(true);
      if (activeLens === 'trash') {
        void loadTrash();
      }
    };

    if (typeof document !== 'undefined') {
      document.addEventListener('tssp:drive-refresh', handleExternalRefresh as EventListener);
    }

    void loadLibrary(true).then(() => consumeIntent());

    return () => {
      if (typeof document !== 'undefined') {
        document.removeEventListener('tssp:drive-refresh', handleExternalRefresh as EventListener);
      }
    };
  });

  const handleDriveKeydown = (e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'a') {
      e.preventDefault();
      filteredLibraryFiles.forEach(f => selectedFileIds.add(f.id));
      selectedFileIds = new Set(selectedFileIds);
    }
    if (e.key === 'Escape') {
      selectedFileIds.clear();
      selectedFileIds = new Set();
      selectedFile = null;
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'c') {
      e.preventDefault();
      if (selectedFileIds.size > 0) {
        const selectedFiles = filteredLibraryFiles.filter(f => selectedFileIds.has(f.id));
        clipboard.copy(selectedFiles.map(f => ({ id: f.id, name: f.name, type: 'file' })));
        clipboardOperation = 'copy';
        success('Copied', `${selectedFileIds.size} file(s) copied to clipboard`);
      }
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'x') {
      e.preventDefault();
      if (selectedFileIds.size > 0) {
        const selectedFiles = filteredLibraryFiles.filter(f => selectedFileIds.has(f.id));
        clipboard.cut(selectedFiles.map(f => ({ id: f.id, name: f.name, type: 'file' })));
        clipboardOperation = 'cut';
        success('Cut', `${selectedFileIds.size} file(s) cut to clipboard`);
      }
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'v') {
      e.preventDefault();
      void handlePasteFiles();
    }
    if ((e.shiftKey || e.metaKey) && e.key === 'Delete') {
      e.preventDefault();
      if (selectedFileIds.size > 0) {
        void handleBulkDelete();
      }
    }
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      if (filteredLibraryFiles.length === 0) return;
      const currentIndex = selectedFile?.id ? filteredLibraryFiles.findIndex(f => f.id === selectedFile?.id) : -1;
      let nextIndex = currentIndex;
      if (e.key === 'ArrowDown') {
        nextIndex = currentIndex < filteredLibraryFiles.length - 1 ? currentIndex + 1 : 0;
      } else {
        nextIndex = currentIndex > 0 ? currentIndex - 1 : filteredLibraryFiles.length - 1;
      }
      selectFile(filteredLibraryFiles[nextIndex]);
    }
    if (e.key === 'Enter' && selectedFile?.id) {
      e.preventDefault();
      openPreview(selectedFile);
    }
    if ((e.ctrlKey || e.metaKey) && e.key === '?') {
      e.preventDefault();
      showKeyboardHelp = true;
    }
  };

  $effect(() => {
    if (typeof document === 'undefined') return;

    const cleanup = registerKeyboardShortcuts(
      [
        { key: 'a', ctrl: true, handler: handleDriveKeydown },
        { key: 'Escape', handler: handleDriveKeydown },
        { key: 'c', ctrl: true, handler: handleDriveKeydown },
        { key: 'x', ctrl: true, handler: handleDriveKeydown },
        { key: 'v', ctrl: true, handler: handleDriveKeydown },
        { key: 'Delete', shift: true, handler: handleDriveKeydown },
        { key: 'ArrowDown', handler: handleDriveKeydown },
        { key: 'ArrowUp', handler: handleDriveKeydown },
        { key: 'Enter', handler: handleDriveKeydown },
        { key: '?', ctrl: true, handler: handleDriveKeydown },
      ],
      document
    );

    return cleanup;
  });

  let isTrashView = $derived(activeLens === 'trash');

  let filteredLibraryFiles = $derived.by(() =>
    files
      .filter((file) => {
        if (currentFolder && (file.folder_path || '') !== currentFolder) return false;

        const mime = file.mime_type || '';
        if (activeLens === 'images' && !mime.startsWith('image/')) return false;
        if (activeLens === 'videos' && !mime.startsWith('video/')) return false;
        if (activeLens === 'documents') {
          if (mime.startsWith('image/') || mime.startsWith('video/') || mime.startsWith('audio/')) return false;
          const isDocument =
            mime.startsWith('text/') ||
            mime.startsWith('application/') ||
            mime.includes('pdf') ||
            mime.includes('word') ||
            mime.includes('sheet') ||
            mime.includes('presentation') ||
            mime.includes('json') ||
            mime.includes('xml') ||
            mime.includes('code') ||
            ['.md', '.doc', '.docx', '.txt', '.pdf', '.xls', '.xlsx', '.ppt', '.pptx', '.csv'].some(ext => file.name.toLowerCase().endsWith(ext));
          if (!isDocument) return false;
        }
        if (activeLens === 'public' && file.visibility !== 'public') return false;

        if (activeTagFilter && !(file.tags || []).includes(activeTagFilter)) return false;

        const query = filterQuery.trim().toLowerCase();
        if (!query) return true;
        return (
          file.name.toLowerCase().includes(query) ||
          (file.folder_path || '').toLowerCase().includes(query) ||
          (file.tags || []).some((tag) => tag.toLowerCase().includes(query))
        );
      })
      .sort((left, right) => {
        // Pinned files always come first
        if (left.pinned_at && !right.pinned_at) return -1;
        if (!left.pinned_at && right.pinned_at) return 1;

        let comparison = 0;
        if (sortBy === 'name') {
          comparison = left.name.localeCompare(right.name);
        } else if (sortBy === 'size') {
          comparison = (left.size_bytes || 0) - (right.size_bytes || 0);
        } else {
          // date (default)
          comparison = (left.updated_at || left.uploaded_at) - (right.updated_at || right.uploaded_at);
        }

        return sortOrder === 'asc' ? comparison : -comparison;
      })
  );

  let filteredTrash = $derived.by(() =>
    trash
      .filter((file) => {
        const query = filterQuery.trim().toLowerCase();
        if (!query) return true;
        return (
          file.name.toLowerCase().includes(query) ||
          (file.folder_path || '').toLowerCase().includes(query) ||
          (file.tags || []).some((tag) => tag.toLowerCase().includes(query))
        );
      })
      .sort((left, right) => (right.updated_at || right.uploaded_at) - (left.updated_at || left.uploaded_at))
  );

  function dedupeFiles(items: FileRecord[]) {
    return Array.from(new Map(items.map((file) => [file.id, file])).values());
  }

  async function loadLibrary(reset = false) {
    if (reset) {
      isLoading = true;
    } else {
      isLoadingMore = true;
    }

    try {
      if (reset) {
        const [fileData, folderData, statusData] = await Promise.all([
          api.listFiles(72),
          api.listFolders(),
          api.getStatus(),
        ]);

        files = fileData.files || [];
        nextCursor = fileData.nextCursor;
        hasMore = !!fileData.nextCursor;
        folderEntries = folderData.folders || [];
        status = statusData;
      } else if (nextCursor) {
        const fileData = await api.listFiles(72, nextCursor);
        files = dedupeFiles([...files, ...(fileData.files || [])]);
        nextCursor = fileData.nextCursor;
        hasMore = !!fileData.nextCursor;
      }
    } catch (cause) {
      error('Drive Failed', cause instanceof Error ? cause.message : 'Could not load files');
    } finally {
      isLoading = false;
      isLoadingMore = false;
    }
  }

  async function loadTrash() {
    trashLoading = true;
    try {
      const data = await api.listTrash();
      trash = data.files || [];
    } catch (cause) {
      error('Trash Failed', cause instanceof Error ? cause.message : 'Could not load trash');
    } finally {
      trashLoading = false;
    }
  }

  async function consumeIntent() {
    const intent = consumeSelectionIntent();
    if (!intent || intent.kind !== 'file') return;

    let target = files.find((file) => file.id === intent.id) || null;
    if (!target) {
      try {
        target = await api.getFile(intent.id);
        files = dedupeFiles([target, ...files]);
      } catch {
        return;
      }
    }

    activeLens = 'all';
    currentFolder = target.folder_path || '';
    selectedFile = target;
  }

  function setLens(lens: DriveLens) {
    activeLens = lens;
    if (lens === 'trash') {
      void loadTrash();
    }
  }

  function setViewMode(mode: 'grid' | 'list') {
    viewMode = mode;
    setDefaultDriveView(mode);
  }

  function requestUpload() {
    if (typeof document !== 'undefined') {
      document.dispatchEvent(new CustomEvent('tssp:request-upload'));
    }
  }

  function folderCount(path: string) {
    return folderEntries.find((entry) => entry.path === path)?.file_count || 0;
  }

  function selectFile(file: FileRecord, event?: MouseEvent) {
    selectedFile = file;

    if (!event) {
      selectedFileIds.clear();
      selectedFileIds.add(file.id);
      selectedFileIds = new Set(selectedFileIds);
      return;
    }

    if (event.ctrlKey || event.metaKey) {
      if (selectedFileIds.has(file.id)) {
        selectedFileIds.delete(file.id);
      } else {
        selectedFileIds.add(file.id);
      }
      selectedFileIds = new Set(selectedFileIds);
      lastSelectedIndex = filteredLibraryFiles.findIndex(f => f.id === file.id);
    } else if (event.shiftKey && lastSelectedIndex >= 0) {
      const currentIndex = filteredLibraryFiles.findIndex(f => f.id === file.id);
      const start = Math.min(lastSelectedIndex, currentIndex);
      const end = Math.max(lastSelectedIndex, currentIndex);
      selectedFileIds.clear();
      for (let i = start; i <= end; i++) {
        selectedFileIds.add(filteredLibraryFiles[i].id);
      }
      selectedFileIds = new Set(selectedFileIds);
    } else {
      selectedFileIds.clear();
      selectedFileIds.add(file.id);
      selectedFileIds = new Set(selectedFileIds);
      lastSelectedIndex = filteredLibraryFiles.findIndex(f => f.id === file.id);
    }
  }

  function openPreview(file: FileRecord) {
    selectedFile = file;
    previewFile = file;
  }

  function updateFileInState(nextFile: FileRecord) {
    files = files.map((file) => (file.id === nextFile.id ? nextFile : file));
    if (selectedFile?.id === nextFile.id) selectedFile = nextFile;
    if (shareFile?.id === nextFile.id) shareFile = nextFile;
    if (previewFile?.id === nextFile.id) previewFile = nextFile;
    if (moveDialogFile?.id === nextFile.id) moveDialogFile = nextFile;
    if (renameDialogFile?.id === nextFile.id) renameDialogFile = nextFile;
  }

  function downloadFile(file: FileRecord) {
    window.location.assign(`/api/v1/files/${encodeURIComponent(file.id)}/content`);
  }

  function handleQuickCopy(file: FileRecord) {
    clipboard.copy([{ id: file.id, name: file.name, type: 'file' }]);
    clipboardOperation = 'copy';
    success('Copied', `${file.name} copied to clipboard`);
  }

  async function handleDropFiles(fileIds: string[], targetFileId: string) {
    const targetFile = files.find(f => f.id === targetFileId);
    if (!targetFile || !targetFile.folder_path) {
      error('Drop Failed', 'Target is not a folder');
      return;
    }

    try {
      const success_op = await driveStateManager.moveFiles(fileIds, currentFolder, targetFile.folder_path);
      if (success_op) {
        await refreshMetadata();
        success('Moved', `${fileIds.length} file(s) moved to ${targetFile.name}`);
      }
    } catch (cause) {
      error('Drop Failed', cause instanceof Error ? cause.message : 'Could not move files');
    }
  }

  async function handleRefresh() {
    await loadLibrary(true);
    if (isTrashView) {
      await loadTrash();
    }
    info('Refreshed', 'Drive state reloaded from the server');
  }

  async function handleRename(file: FileRecord, nextName: string) {
    if (!nextName || nextName === file.name) return;

    try {
      isRenaming = true;
      const success_op = await driveStateManager.renameFile(file, nextName);
      if (success_op) {
        updateFileInState({ ...file, name: nextName });
        success('Renamed', `${file.name} is now ${nextName}`);
      }
    } catch (cause) {
      error('Rename Failed', cause instanceof Error ? cause.message : 'Could not rename file');
    } finally {
      isRenaming = false;
    }
  }

  async function handlePin(file: FileRecord) {
    try {
      const success_op = await driveStateManager.togglePin(file);
      if (success_op) {
        const nextPinState = !file.pinned_at;
        const updated = { ...file, pinned_at: nextPinState ? Math.floor(Date.now() / 1000) : undefined };
        updateFileInState(updated);
        success(nextPinState ? 'Pinned' : 'Unpinned', nextPinState ? `${file.name} will stay at the top` : `${file.name} moved out of pinned`);
      }
    } catch (cause) {
      error('Pin Failed', cause instanceof Error ? cause.message : 'Could not update pin state');
    }
  }


  async function handleDelete(file: FileRecord) {
    if (!confirm(`Move "${file.name}" to trash?`)) return;

    try {
      const success_op = await driveStateManager.deleteFile(file);
      if (success_op) {
        files = files.filter((item) => item.id !== file.id);
        if (selectedFile?.id === file.id) selectedFile = null;
        await Promise.all([loadTrash(), refreshMetadata()]);
        success('Moved to Trash', `${file.name} can be restored from trash`);
      }
    } catch (cause) {
      error('Delete Failed', cause instanceof Error ? cause.message : 'Could not delete file');
    }
  }

  async function handlePasteFiles() {
    const clipboardItems = clipboard.paste();
    if (clipboardItems.length === 0) {
      info('Clipboard Empty', 'No files to paste');
      return;
    }

    const fileIds = clipboardItems.map(item => item.id);
    const operation = clipboardItems[0]?.operation;

    try {
      if (operation === 'cut') {
        const success_op = await driveStateManager.moveFiles(fileIds, '', currentFolder);
        if (success_op) {
          clipboardOperation = null;
          success('Files Moved', `${fileIds.length} file(s) moved to ${currentFolder || 'root'}`);
        } else {
          error('Move Failed', 'Could not move files');
        }
      } else if (operation === 'copy') {
        const success_op = await driveStateManager.copyFiles(fileIds, currentFolder);
        if (success_op) {
          success('Files Copied', `${fileIds.length} file(s) copied to ${currentFolder || 'root'}`);
        } else {
          error('Copy Failed', 'Could not copy files');
        }
      }
    } catch (cause) {
      error('Paste Failed', cause instanceof Error ? cause.message : 'Could not paste files');
    }
  }

  async function handleRestore(file: FileRecord) {
    try {
      const success_op = await driveStateManager.restoreFile(file);
      if (success_op) {
        trash = trash.filter((item) => item.id !== file.id);
        await Promise.all([loadLibrary(true), loadTrash()]);
        success('Restored', `${file.name} is back in Drive`);
      }
    } catch (cause) {
      error('Restore Failed', cause instanceof Error ? cause.message : 'Could not restore file');
    }
  }

  async function handlePermanentDelete(file: FileRecord) {
    if (!confirm(`Permanently purge "${file.name}" from trash?`)) return;

    try {
      const success_op = await driveStateManager.permanentlyDeleteFile(file);
      if (success_op) {
        trash = trash.filter((item) => item.id !== file.id);
        if (selectedFile?.id === file.id) selectedFile = null;
        success('Purged', `${file.name} was permanently deleted`);
      }
    } catch (cause) {
      error('Purge Failed', cause instanceof Error ? cause.message : 'Could not purge file');
    }
  }

  async function handleBulkDelete() {
    const fileCount = selectedFileIds.size;
    if (fileCount === 0) return;
    if (!confirm(`Move ${fileCount} file(s) to trash?`)) return;

    try {
      const success_op = await driveStateManager.deleteFiles(Array.from(selectedFileIds));
      if (success_op) {
        selectedFileIds.clear();
        selectedFileIds = new Set();
        selectedFile = null;
        await Promise.all([loadTrash(), refreshMetadata()]);
        success('Moved to Trash', `${fileCount} file(s) moved to trash`);
      }
    } catch (cause) {
      error('Delete Failed', cause instanceof Error ? cause.message : 'Could not delete files');
    }
  }

  async function handleBulkMove(targetFolder: string) {
    const fileCount = selectedFileIds.size;
    if (fileCount === 0) return;

    try {
      isMoving = true;
      const success_op = await driveStateManager.moveFiles(Array.from(selectedFileIds), currentFolder, targetFolder);
      if (success_op) {
        selectedFileIds.clear();
        selectedFileIds = new Set();
        selectedFile = null;
        await refreshMetadata();
        success('Moved', `${fileCount} file(s) moved to ${targetFolder || 'Bucket root'}`);
      }
    } catch (cause) {
      error('Move Failed', cause instanceof Error ? cause.message : 'Could not move files');
    } finally {
      isMoving = false;
    }
  }

  async function handlePurgeExpiredTrash() {
    const confirmed = confirm('Purge only trash items older than the configured retention period?');
    if (!confirmed) return;

    try {
      const success_op = await driveStateManager.emptyTrash();
      if (success_op) {
        await loadTrash();
        success('Expired Trash Purged', 'Only items older than retention were removed');
      }
    } catch (cause) {
      error('Trash Purge Failed', cause instanceof Error ? cause.message : 'Could not purge expired trash');
    }
  }

  async function handleMove(fileId: string, folderPath: string) {
    try {
      isMoving = true;
      const file = files.find(f => f.id === fileId);
      if (!file) {
        error('Move Failed', 'File not found');
        return;
      }
      const success_op = await driveStateManager.moveFile(file, folderPath);
      if (success_op) {
        await refreshMetadata();
        success('Moved', `File moved to ${folderPath || 'Bucket root'}`);
      }
    } catch (cause) {
      error('Move Failed', cause instanceof Error ? cause.message : 'Could not move file');
    } finally {
      isMoving = false;
    }
  }

  async function handleShareChange(fileId: string, isPublic: boolean): Promise<FileRecord | null> {
    try {
      const response = await driveStateManager.setFileVisibility(fileId, isPublic);
      if (response) {
        success(isPublic ? 'Public Link Enabled' : 'Made Private', response.name || 'File');
      }
      return response;
    } catch (cause) {
      error('Share Failed', cause instanceof Error ? cause.message : 'Could not update visibility');
      return null;
    }
  }

  async function handleToggleVisibility(isPublic: boolean) {
    if (!selectedFile) return;
    await handleShareChange(selectedFile.id, isPublic);
  }

  async function refreshMetadata() {
    try {
      const [folderData, statusData] = await Promise.all([api.listFolders(), api.getStatus()]);
      folderEntries = folderData.folders || [];
      status = statusData;
    } catch {
      // Metadata refresh should not replace the main action result with another error.
    }
  }

  function showContextMenu(event: MouseEvent, file: FileRecord) {
    event.preventDefault();
    contextMenu = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      file,
    };
  }

  async function handleOpenAsWorkspace(file: FileRecord) {
    try {
      const workspaceId = await createWorkspaceFromFolder(file.name, file.folder_path || '');
      success('Workspace Created', `Opened "${file.name}" as a workspace`);
      selectionIntent.set({ kind: 'workspace', id: workspaceId });
    } catch (cause) {
      error('Failed to Open Workspace', cause instanceof Error ? cause.message : 'Could not create workspace');
    }
  }

  interface MenuItem {
    icon?: any;
    label: string;
    action: () => void | Promise<void>;
    danger?: boolean;
  }

  function getContextItems(file: FileRecord): MenuItem[] {
    if (isTrashView) {
      return [
        { icon: Icons.RotateCcw, label: 'Restore', action: () => handleRestore(file) },
        { icon: Icons.Trash2, label: 'Purge permanently', action: () => handlePermanentDelete(file), danger: true },
      ];
    }

    const items: MenuItem[] = [
      { icon: Icons.Eye, label: 'Preview', action: () => openPreview(file) },
      { icon: Icons.Download, label: 'Download', action: () => downloadFile(file) },
      { icon: Icons.Share2, label: file.visibility === 'public' ? 'Manage sharing' : 'Share', action: () => { selectedFile = file; shareFile = file; } },
      { icon: Icons.Pin, label: file.pinned_at ? 'Unpin' : 'Pin', action: () => handlePin(file) },
      { icon: Icons.Pencil, label: 'Rename', action: () => { selectedFile = file; renameDialogFile = file; isRenameDialogOpen = true; } },
      { icon: Icons.FolderOpen, label: 'Move', action: () => { selectedFile = file; moveDialogFile = file; isMoveDialogOpen = true; } },
    ];

    if (isDriveFolder(file)) {
      items.push({ icon: Icons.Code, label: 'Open as Workspace', action: () => handleOpenAsWorkspace(file) });
    }

    items.push({ icon: Icons.Trash2, label: 'Move to trash', action: () => handleDelete(file), danger: true });

    return items;
  }

  const libraryFilters = [
    { id: 'all' as const, label: 'All files', icon: Icons.HardDrive },
    { id: 'images' as const, label: 'Images', icon: Icons.Image },
    { id: 'videos' as const, label: 'Videos', icon: Icons.Video },
    { id: 'documents' as const, label: 'Documents', icon: Icons.FileText },
    { id: 'public' as const, label: 'Public', icon: Icons.Globe },
    { id: 'trash' as const, label: 'Trash', icon: Icons.Trash2 },
  ];
</script>

<div class="drive-shell">
  {#if sidebarOpen}
    <DriveSidebar
      filters={libraryFilters}
      {activeLens}
      folders={folderEntries}
      {currentFolder}
      publicCount={files.filter((f) => f.visibility === 'public').length}
      trashCount={trash.length}
      usedBytes={status?.storage_bytes_used || 0}
      totalObjects={status?.file_count || files.length}
      onLensChange={setLens}
      onFolderChange={(path) => {
        currentFolder = path;
        if (activeLens === 'trash') activeLens = 'all';
      }}
    />
  {/if}

  <section class="drive-main" class:sidebar-closed={!sidebarOpen}>
    <button class="sidebar-toggle" onclick={() => sidebarOpen = !sidebarOpen} title={sidebarOpen ? 'Hide sidebar' : 'Show sidebar'}>
      <Icons.Menu size={18} />
    </button>
    <DriveHeader
      title={isTrashView ? 'Trash' : 'Cloud Drive'}
      {currentFolder}
      {isTrashView}
      fileCount={status?.file_count || files.length}
      visibleCount={isTrashView ? filteredTrash.length : filteredLibraryFiles.length}
      pinnedCount={status?.pinned_count || files.filter((f) => f.pinned_at).length}
      publicCount={files.filter((f) => f.visibility === 'public').length}
      selectedCount={selectedFileIds.size}
      {imageCount}
      {videoCount}
      {documentCount}
      onRefresh={handleRefresh}
      onUpload={requestUpload}
      onPurgeTrash={handlePurgeExpiredTrash}
      trashEmpty={trash.length === 0}
    />

    <div class="toolbar-with-panel-toggle">
      <DriveToolbarRow
        filterQuery={filterQuery}
        isTrashView={isTrashView}
        viewMode={viewMode}
        {sortBy}
        {sortOrder}
        onFilterChange={(query) => filterQuery = query}
        onViewModeChange={setViewMode}
        onSortChange={(newSortBy) => sortBy = newSortBy}
        onSortOrderChange={(newOrder) => sortOrder = newOrder}
      />
      <button class="panel-toggle-btn" title={detailsPanelOpen ? 'Hide details' : 'Show details'} onclick={() => detailsPanelOpen = !detailsPanelOpen}>
        <Icons.ChevronRight size={16} />
      </button>
    </div>

    {#if !isTrashView && availableTags.length > 0}
      <div class="tag-filter-bar">
        <span class="tag-label">Filter by tag:</span>
        <div class="tag-list">
          {#each availableTags as tag}
            <button
              type="button"
              class="tag-btn"
              class:active={activeTagFilter === tag}
              onclick={() => activeTagFilter = activeTagFilter === tag ? null : tag}
            >
              {tag}
            </button>
          {/each}
          {#if activeTagFilter}
            <button
              type="button"
              class="tag-clear"
              onclick={() => activeTagFilter = null}
              title="Clear tag filter"
            >
              <Icons.X size={12} />
            </button>
          {/if}
        </div>
      </div>
    {/if}

    {#if selectedFileIds.size > 0}
      <div class="bulk-actions-bar">
        <div class="bulk-info">
          <strong>{selectedFileIds.size} file(s) selected</strong>
          <button type="button" class="clear-selection" onclick={() => {
            selectedFileIds.clear();
            selectedFileIds = new Set();
            selectedFile = null;
          }}>
            <Icons.X size={16} />
            Clear
          </button>
        </div>
        <div class="bulk-actions">
          <div class="move-dropdown">
            <button type="button" class="action-btn" onclick={() => showBulkMoveMenu = !showBulkMoveMenu}>
              <Icons.FolderOpen size={14} />
              Move to
              <Icons.ChevronDown size={12} />
            </button>
            {#if showBulkMoveMenu}
              <div class="move-menu">
                <button
                  type="button"
                  onclick={() => {
                    void handleBulkMove('');
                    showBulkMoveMenu = false;
                  }}
                  disabled={isMoving || currentFolder === ''}
                >
                  <Icons.HardDrive size={12} />
                  Bucket root
                </button>
                {#each folderEntries as folder}
                  <button
                    type="button"
                    onclick={() => {
                      void handleBulkMove(folder.path);
                      showBulkMoveMenu = false;
                    }}
                    disabled={isMoving || currentFolder === folder.path}
                  >
                    <Icons.Folder size={12} />
                    {folder.path}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
          <button type="button" class="action-btn danger" onclick={handleBulkDelete}>
            <Icons.Trash2 size={14} />
            Delete selected
          </button>
        </div>
      </div>
    {/if}

    <DriveContent
      files={filteredLibraryFiles}
      trash={filteredTrash}
      {isLoading}
      {isTrashView}
      {viewMode}
      selectedFileId={selectedFile?.id}
      {selectedFileIds}
      clipboardFileIds={new Set(clipboardFileIds)}
      {clipboardOperation}
      {hasMore}
      {isLoadingMore}
      onSelectFile={selectFile}
      onPreviewFile={openPreview}
      onContextMenu={showContextMenu}
      onLoadMore={() => loadLibrary(false)}
      onRestore={handleRestore}
      onDelete={handlePermanentDelete}
      onPurgeTrash={handlePurgeExpiredTrash}
      onUpload={requestUpload}
      onDownload={downloadFile}
      onCopy={handleQuickCopy}
      onDropFiles={handleDropFiles}
    />

    {#if selectedFile && detailsPanelOpen}
      <DriveDetailsPanel
        file={selectedFile}
        onToggleVisibility={handleToggleVisibility}
        onShare={() => {
          if (selectedFile) shareFile = selectedFile;
        }}
        onMove={() => {
          if (selectedFile) {
            moveDialogFile = selectedFile;
            isMoveDialogOpen = true;
          }
        }}
      />
    {/if}
  </section>
</div>

<ContextMenu
  bind:visible={contextMenu.visible}
  x={contextMenu.x}
  y={contextMenu.y}
  items={contextMenu.file ? getContextItems(contextMenu.file) : []}
  onClose={() => contextMenu.visible = false}
/>

<FilePreviewModal
  file={previewFile}
  isOpen={previewFile !== null}
  onClose={() => (previewFile = null)}
  onDownload={(fileId) => {
    const target = files.find((file) => file.id === fileId) || previewFile;
    if (target) downloadFile(target);
  }}
/>

<SharingModal
  file={shareFile}
  isOpen={shareFile !== null}
  onClose={() => (shareFile = null)}
  onShare={handleShareChange}
/>

<RenameFileDialog
  file={renameDialogFile}
  isOpen={isRenameDialogOpen}
  isRenaming={isRenaming}
  onRename={handleRename}
  onClose={() => {
    isRenameDialogOpen = false;
    renameDialogFile = null;
  }}
/>

<MoveFileDialog
  file={moveDialogFile}
  folders={folderEntries.map((entry) => entry.path)}
  isOpen={isMoveDialogOpen}
  isMoving={isMoving}
  onMove={handleMove}
  onClose={() => {
    isMoveDialogOpen = false;
    moveDialogFile = null;
  }}
/>

<KeyboardHelpModal
  isOpen={showKeyboardHelp}
  onClose={() => (showKeyboardHelp = false)}
/>

<style>
  .drive-shell {
    flex: 1;
    display: grid;
    grid-template-columns: 240px minmax(0, 1fr);
    min-height: 0;
  }

  .drive-main {
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }

  .sidebar-toggle {
    position: absolute;
    top: 12px;
    left: 12px;
    width: 36px;
    height: 36px;
    padding: 0;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 150ms;
    z-index: 50;
  }

  .sidebar-toggle:hover {
    background: var(--surface-2);
    color: var(--text);
    border-color: var(--border-2);
  }

  .drive-main.sidebar-closed .sidebar-toggle {
    display: none;
  }

  .toolbar-with-panel-toggle {
    display: flex;
    gap: 12px;
    align-items: center;
    padding-right: 12px;
  }

  .panel-toggle-btn {
    padding: 8px 12px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 150ms;
  }

  .panel-toggle-btn:hover {
    border-color: var(--border-2);
    color: var(--text);
  }

  .tag-filter-bar {
    margin: 0 24px 12px;
    padding: 8px 12px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    gap: 12px;
    align-items: center;
    flex-wrap: wrap;
  }

  .tag-label {
    font-size: 12px;
    color: var(--muted);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .tag-list {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    align-items: center;
  }

  .tag-btn {
    padding: 4px 10px;
    border: 1px solid var(--border);
    border-radius: 16px;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    transition: all 150ms;
    white-space: nowrap;
  }

  .tag-btn:hover {
    border-color: var(--blue);
    color: var(--blue);
    background: rgba(59, 130, 246, 0.05);
  }

  .tag-btn.active {
    background: rgba(59, 130, 246, 0.15);
    border-color: var(--blue);
    color: var(--blue);
  }

  .tag-clear {
    padding: 2px 6px;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    transition: color 150ms;
  }

  .tag-clear:hover {
    color: var(--text);
  }

  .bulk-actions-bar {
    margin: 0 24px 12px;
    padding: 12px 14px;
    border-radius: 12px;
    border: 1px solid var(--blue-soft, rgba(59, 130, 246, 0.3));
    background: rgba(59, 130, 246, 0.05);
    display: flex;
    gap: 16px;
    align-items: center;
    justify-content: space-between;
  }

  .bulk-info {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .bulk-info strong {
    color: var(--text);
    font-size: 14px;
  }

  .clear-selection {
    padding: 4px 8px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-2);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    transition: all 150ms;
  }

  .clear-selection:hover {
    border-color: var(--text-2);
    color: var(--text);
  }

  .bulk-actions {
    display: flex;
    gap: 8px;
  }

  .action-btn {
    padding: 6px 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-2);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    font-weight: 500;
    transition: all 150ms;
  }

  .action-btn:hover:not(:disabled) {
    border-color: var(--border-2);
    color: var(--text);
  }

  .action-btn.danger {
    background: var(--danger-soft);
    border: 1px solid var(--danger-soft);
    color: var(--danger);
  }

  .action-btn.danger:hover:not(:disabled) {
    background: var(--danger);
    color: white;
    border-color: var(--danger);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .move-dropdown {
    position: relative;
  }

  .move-menu {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 4px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    z-index: 100;
    min-width: 180px;
    max-height: 300px;
    overflow-y: auto;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.3);
  }

  .move-menu button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: 13px;
    transition: background 150ms;
    text-align: left;
  }

  .move-menu button:first-child {
    border-radius: 7px 7px 0 0;
  }

  .move-menu button:last-child {
    border-radius: 0 0 7px 7px;
  }

  .move-menu button:hover:not(:disabled) {
    background: var(--surface);
    color: var(--text);
  }

  .move-menu button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @media (max-width: 1200px) {
    .drive-shell {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 760px) {
    .drive-shell {
      grid-template-columns: 1fr;
    }
  }
</style>
