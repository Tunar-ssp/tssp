<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import FileIcon from '$lib/components/FileIcon.svelte';
  import TrashView from './TrashView.svelte';
  import type { FileRecord } from '$lib/api';
  import { formatBytes, formatRelative } from '$lib/utils';

  interface FolderItem {
    path: string;
    name: string;
    fileCount: number;
  }

  interface Props {
    files?: FileRecord[];
    trash?: FileRecord[];
    folders?: FolderItem[];
    isLoading?: boolean;
    isTrashView?: boolean;
    viewMode?: 'grid' | 'list';
    selectedFileId?: string;
    selectedFileIds?: Set<string>;
    clipboardFileIds?: Set<string>;
    clipboardOperation?: 'copy' | 'cut' | null;
    hasMore?: boolean;
    isLoadingMore?: boolean;
    showThumbnails?: boolean;
    renameTargetId?: string | null;
    onRenameStarted?: () => void;
    onSelectFile?: (file: FileRecord, event?: MouseEvent) => void;
    onPreviewFile?: (file: FileRecord) => void;
    onContextMenu?: (event: MouseEvent, file: FileRecord) => void;
    onFolderContextMenu?: (event: MouseEvent, path: string, name: string) => void;
    onOpenFolder?: (path: string) => void;
    onLoadMore?: () => void;
    onRestore?: (file: FileRecord) => void;
    onDelete?: (file: FileRecord) => void;
    onPurgeTrash?: () => void;
    onUpload?: () => void;
    onDownload?: (file: FileRecord) => void;
    onCopy?: (file: FileRecord) => void;
    onRename?: (file: FileRecord, nextName: string) => void;
    onDropFiles?: (fileIds: string[], targetFileId: string) => Promise<void>;
  }

  let {
    files = [],
    trash = [],
    folders = [],
    isLoading = false,
    isTrashView = false,
    viewMode = 'grid',
    selectedFileId,
    selectedFileIds = new Set(),
    clipboardFileIds = new Set(),
    clipboardOperation = null,
    hasMore = false,
    isLoadingMore = false,
    showThumbnails = true,
    renameTargetId = null,
    onRenameStarted = () => {},
    onSelectFile = () => {},
    onPreviewFile = () => {},
    onContextMenu = () => {},
    onFolderContextMenu = (_event: MouseEvent, _path: string, _name: string) => {},
    onOpenFolder = () => {},
    onLoadMore = () => {},
    onRestore = () => {},
    onDelete = () => {},
    onPurgeTrash = () => {},
    onUpload = () => {},
    onDownload = () => {},
    onCopy = () => {},
    onRename = () => {},
    onDropFiles = async (_fileIds: string[], _targetFileId: string) => {},
  }: Props = $props();

  // Deduplicate to prevent each_key_duplicate errors
  let displayFiles = $derived(
    Array.from(new Map((isTrashView ? trash : files).map((f) => [f.id, f])).values())
  );
  let displayFolders = $derived(isTrashView ? [] : folders);
  let draggedOverFile = $state<string | null>(null);
  let renamingFileId = $state<string | null>(null);
  let renameValue = $state<string>('');
  // Thumbnail loading is staged per file: the lightweight /thumbnail endpoint
  // first (cheap for the Orange Pi), then full /content if that 404s because
  // the stored mime_type isn't image/* (backend mime detection is unreliable),
  // then the type icon if even that fails.
  let thumbStage = $state<Map<string, 'thumb' | 'content' | 'failed'>>(new Map());

  const IMG_THUMB = /\.(jpg|jpeg|png|gif|webp|avif|bmp|ico|svg)$/i;
  const VID_THUMB = /\.(mp4|webm|mov|mkv|avi|flv|wmv|m4v)$/i;

  function stageOf(id: string): 'thumb' | 'content' | 'failed' {
    return thumbStage.get(id) ?? 'thumb';
  }

  function thumbSrc(id: string): string {
    const base = `/api/v1/files/${encodeURIComponent(id)}`;
    return stageOf(id) === 'content' ? `${base}/content?disposition=inline` : `${base}/thumbnail`;
  }

  function onThumbError(id: string) {
    const current = stageOf(id);
    const next = current === 'thumb' ? 'content' : 'failed';
    thumbStage.set(id, next);
    thumbStage = new Map(thumbStage);
  }

  // Focus + select an input on mount without triggering the browser's
  // "Autofocus processing was blocked" warning that bare `autofocus` causes.
  function autofocusSelect(node: HTMLInputElement) {
    queueMicrotask(() => {
      node.focus();
      node.select();
    });
  }

  function handleFolderDblClick(path: string) {
    onOpenFolder?.(path);
  }

  // React to external rename triggers (e.g. F2 key, context menu)
  $effect(() => {
    const target = renameTargetId;
    if (!target) return;
    const allFiles = isTrashView ? trash : files;
    const file = allFiles.find((f) => f.id === target);
    if (file) {
      renamingFileId = file.id;
      renameValue = file.name;
    }
    onRenameStarted();
  });

  function startRename(file: FileRecord) {
    renamingFileId = file.id;
    renameValue = file.name;
  }

  function finishRename(file: FileRecord) {
    const next = renameValue.trim();
    if (next && next !== file.name) {
      onRename?.(file, next);
    }
    renamingFileId = null;
    renameValue = '';
  }

  function handleRenameKeydown(e: KeyboardEvent, file: FileRecord) {
    if (e.key === 'Enter') {
      e.preventDefault();
      finishRename(file);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      renamingFileId = null;
      renameValue = '';
    }
  }

  function handleDragStart(e: DragEvent, file: FileRecord) {
    if (!e.dataTransfer) return;
    const filesToDrag = selectedFileIds.has(file.id) ? Array.from(selectedFileIds) : [file.id];
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('application/json', JSON.stringify({ fileIds: filesToDrag }));
  }

  function handleDragOver(e: DragEvent, file: FileRecord) {
    if (!e.dataTransfer) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    draggedOverFile = file.id;
  }

  function handleDragLeave(e: DragEvent, file: FileRecord) {
    if (draggedOverFile === file.id) {
      draggedOverFile = null;
    }
  }

  async function handleDrop(e: DragEvent, targetFile: FileRecord) {
    if (!e.dataTransfer) return;
    e.preventDefault();
    draggedOverFile = null;

    try {
      const data = e.dataTransfer.getData('application/json');
      const { fileIds } = JSON.parse(data) as { fileIds: string[] };
      if (fileIds.length > 0) {
        await onDropFiles?.(fileIds, targetFile.id);
      }
    } catch {
      // Ignore invalid drop data
    }
  }
</script>

<div class="drive-content">
  <div class="content-main">
    {#if isTrashView}
      <TrashView
        files={trash}
        {isLoading}
        onRestore={onRestore}
        onDelete={onDelete}
        onEmptyTrash={onPurgeTrash}
        onContextMenu={onContextMenu}
      />
    {:else if isLoading}
      <div class="loading-panel">
        <div class="spinner"></div>
        <strong>Loading Drive</strong>
        <p>Fetching files, folders, and storage state.</p>
      </div>
    {:else if displayFiles.length === 0 && displayFolders.length === 0}
      <div class="empty-panel">
        <Icons.UploadCloud size={36} />
        <strong>Drop files here to upload</strong>
        <p>Or use the button to browse — drag-and-drop works anywhere in Drive.</p>
        <button type="button" class="accent-btn" onclick={onUpload}>
          <Icons.Upload size={15} />
          Choose files
        </button>
      </div>
    {:else if viewMode === 'grid'}
      <div class="file-grid">
        {#each displayFolders as folder (folder.path)}
          <div
            class="file-card folder-card"
            ondblclick={() => handleFolderDblClick(folder.path)}
            onclick={(e) => e.currentTarget.focus()}
            oncontextmenu={(e) => { e.preventDefault(); onFolderContextMenu?.(e, folder.path, folder.name); }}
            role="button"
            tabindex="0"
            onkeydown={(e) => { if (e.key === 'Enter') handleFolderDblClick(folder.path); }}
          >
            <div class="file-surface folder-surface">
              <Icons.Folder size={44} class="folder-glyph-big" />
            </div>
            <div class="file-copy">
              <strong class="filename-text">{folder.name}</strong>
              <span>{folder.fileCount} item{folder.fileCount !== 1 ? 's' : ''}</span>
            </div>
          </div>
        {/each}
        {#each displayFiles as file (file.id)}
          <div
            class="file-card"
            class:selected={selectedFileId === file.id}
            class:multi-selected={selectedFileIds.has(file.id)}
            class:clipboard-copy={clipboardFileIds.has(file.id) && clipboardOperation === 'copy'}
            class:clipboard-cut={clipboardFileIds.has(file.id) && clipboardOperation === 'cut'}
            class:drag-over={draggedOverFile === file.id}
            onclick={(e) => onSelectFile?.(file, e)}
            ondblclick={() => onPreviewFile?.(file)}
            oncontextmenu={(event) => onContextMenu?.(event, file)}
            ondragstart={(e) => handleDragStart(e, file)}
            ondragover={(e) => handleDragOver(e, file)}
            ondragleave={(e) => handleDragLeave(e, file)}
            ondrop={(e) => void handleDrop(e, file)}
            draggable="true"
            role="button"
            tabindex="0"
            onkeydown={(e) => {
              if (e.key === 'Enter') onSelectFile?.(file);
            }}
          >
            <div class="file-surface">
              {#if showThumbnails && (IMG_THUMB.test(file.name) || VID_THUMB.test(file.name)) && stageOf(file.id) !== 'failed'}
                <img
                  src={thumbSrc(file.id)}
                  alt={file.name}
                  class="file-thumbnail"
                  loading="lazy"
                  onerror={() => onThumbError(file.id)}
                />
                {#if VID_THUMB.test(file.name) && stageOf(file.id) === 'thumb'}
                  <div class="video-play-overlay">
                    <Icons.Play size={20} />
                  </div>
                {/if}
              {:else if showThumbnails && VID_THUMB.test(file.name) && stageOf(file.id) === 'failed'}
                <div class="video-thumbnail">
                  <Icons.Play size={24} />
                </div>
              {:else}
                <FileIcon mimeType={file.mime_type} name={file.name} size={40} />
              {/if}
              {#if file.visibility === 'public'}
                <span class="inline-badge public">Public</span>
              {/if}
              {#if file.pinned_at}
                <span class="inline-badge pinned"><Icons.Pin size={10} /></span>
              {/if}
              <div class="quick-actions">
                <button
                  type="button"
                  class="quick-action"
                  title="Preview"
                  onclick={(e) => {
                    e.stopPropagation();
                    onPreviewFile?.(file);
                  }}
                >
                  <Icons.Eye size={14} />
                </button>
                <button
                  type="button"
                  class="quick-action"
                  title="Download"
                  onclick={(e) => {
                    e.stopPropagation();
                    onDownload?.(file);
                  }}
                >
                  <Icons.Download size={14} />
                </button>
                <button
                  type="button"
                  class="quick-action"
                  title="Copy"
                  onclick={(e) => {
                    e.stopPropagation();
                    onCopy?.(file);
                  }}
                >
                  <Icons.Copy size={14} />
                </button>
              </div>
            </div>
            <div class="file-copy">
              {#if renamingFileId === file.id}
                <input
                  type="text"
                  class="rename-input"
                  bind:value={renameValue}
                  onblur={() => finishRename(file)}
                  onkeydown={(e) => handleRenameKeydown(e, file)}
                  onclick={(e) => e.stopPropagation()}
                  use:autofocusSelect
                />
              {:else}
                <strong
                  ondblclick={(e) => { e.stopPropagation(); startRename(file); }}
                  class="filename-text"
                >
                  {file.name}
                </strong>
              {/if}
              <span>{formatBytes(file.size_bytes)} · {formatRelative(file.updated_at || file.uploaded_at)}</span>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="file-list">
        <div class="list-head">
          <span>Name</span>
          <span>Size</span>
          <span>Updated</span>
          <span>Location</span>
          <span>State</span>
        </div>

        {#each displayFolders as folder (folder.path)}
          <button
            type="button"
            class="list-row folder-row"
            ondblclick={() => handleFolderDblClick(folder.path)}
            oncontextmenu={(e) => { e.preventDefault(); onFolderContextMenu?.(e, folder.path, folder.name); }}
          >
            <div class="list-row-content">
              <div class="name-cell">
                <div class="list-thumb folder-thumb">
                  <Icons.Folder size={18} class="folder-list-icon" />
                </div>
                <span class="filename-text folder-name">{folder.name}</span>
              </div>
              <span class="muted-state">{folder.fileCount} item{folder.fileCount !== 1 ? 's' : ''}</span>
              <span></span>
              <span class="muted-state">Folder</span>
              <span></span>
            </div>
            <div class="list-row-actions"></div>
          </button>
        {/each}

        {#each displayFiles as file (file.id)}
          <button
            type="button"
            class="list-row"
            class:selected={selectedFileId === file.id}
            class:multi-selected={selectedFileIds.has(file.id)}
            class:clipboard-copy={clipboardFileIds.has(file.id) && clipboardOperation === 'copy'}
            class:clipboard-cut={clipboardFileIds.has(file.id) && clipboardOperation === 'cut'}
            class:drag-over={draggedOverFile === file.id}
            onclick={(e) => onSelectFile?.(file, e)}
            ondblclick={() => onPreviewFile?.(file)}
            oncontextmenu={(event) => onContextMenu?.(event, file)}
            ondragstart={(e) => handleDragStart(e, file)}
            ondragover={(e) => handleDragOver(e, file)}
            ondragleave={(e) => handleDragLeave(e, file)}
            ondrop={(e) => void handleDrop(e, file)}
            draggable="true"
          >
            <div
              class="list-row-content"
            >
              <div class="name-cell">
                <div class="list-thumb">
                  {#if showThumbnails && (IMG_THUMB.test(file.name) || VID_THUMB.test(file.name)) && stageOf(file.id) !== 'failed'}
                    <img
                      src={thumbSrc(file.id)}
                      alt={file.name}
                      class="list-thumbnail"
                      loading="lazy"
                      onerror={() => onThumbError(file.id)}
                    />
                  {:else}
                    <FileIcon mimeType={file.mime_type} name={file.name} size={18} />
                  {/if}
                </div>
                {#if renamingFileId === file.id}
                  <input
                    type="text"
                    class="rename-input"
                    bind:value={renameValue}
                    onblur={() => finishRename(file)}
                    onkeydown={(e) => handleRenameKeydown(e, file)}
                    onclick={(e) => e.stopPropagation()}
                    use:autofocusSelect
                  />
                {:else}
                  <span
                    role="button"
                    tabindex="-1"
                    ondblclick={(e) => { e.stopPropagation(); startRename(file); }}
                    class="filename-text"
                  >
                    {file.name}
                  </span>
                {/if}
              </div>
              <span>{formatBytes(file.size_bytes)}</span>
              <span>{formatRelative(file.updated_at || file.uploaded_at)}</span>
              <span>{file.folder_path || 'Bucket root'}</span>
              <div class="state-cell">
                {#if file.visibility === 'public'}
                  <span class="inline-badge public">Public</span>
                {/if}
                {#if file.pinned_at}
                  <span class="inline-badge pinned">Pinned</span>
                {/if}
                {#if file.visibility !== 'public' && !file.pinned_at}
                  <span class="muted-state">Private</span>
                {/if}
              </div>
            </div>
            <div class="list-row-actions">
              <div
                class="row-action"
                role="button"
                tabindex="0"
                title="Preview"
                onclick={(e) => {
                  e.stopPropagation();
                  onPreviewFile?.(file);
                }}
                onkeydown={(e) => {
                  if (e.key === 'Enter') onPreviewFile?.(file);
                }}
              >
                <Icons.Eye size={14} />
              </div>
              <div
                class="row-action"
                role="button"
                tabindex="0"
                title="Download"
                onclick={(e) => {
                  e.stopPropagation();
                  onDownload?.(file);
                }}
                onkeydown={(e) => {
                  if (e.key === 'Enter') onDownload?.(file);
                }}
              >
                <Icons.Download size={14} />
              </div>
              <div
                class="row-action"
                role="button"
                tabindex="0"
                title="Copy"
                onclick={(e) => {
                  e.stopPropagation();
                  onCopy?.(file);
                }}
                onkeydown={(e) => {
                  if (e.key === 'Enter') onCopy?.(file);
                }}
              >
                <Icons.Copy size={14} />
              </div>
            </div>
          </button>
        {/each}
      </div>
    {/if}

    {#if !isTrashView && hasMore}
      <div class="load-more-row">
        <button type="button" class="ghost-btn" onclick={onLoadMore} disabled={isLoadingMore}>
          {#if isLoadingMore}
            <div class="spinner small"></div>
            Loading more
          {:else}
            <Icons.ChevronsDown size={14} />
            Load more
          {/if}
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .drive-content {
    flex: 1;
    overflow: hidden;
    display: grid;
    grid-template-columns: 1fr;
    min-height: 0;
  }

  .content-main {
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    padding: 0 24px 24px;
  }

  .loading-panel,
  .empty-panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 60px 24px;
    color: var(--muted);
    text-align: center;
  }

  .empty-panel strong,
  .loading-panel strong {
    color: var(--text);
  }

  .accent-btn {
    padding: 8px 14px;
    background: var(--blue);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: background 0.2s;
    margin-top: 12px;
  }

  .accent-btn:hover {
    background: var(--blue-hover);
  }

  .file-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 12px;
    padding: 12px 0;
  }

  .file-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-secondary);
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
    position: relative;
    user-select: none;
  }

  .file-card:hover {
    border-color: var(--blue-soft);
    background: rgba(59, 130, 246, 0.05);
  }

  .file-card.selected {
    border-color: var(--blue);
    background: rgba(59, 130, 246, 0.1);
  }

  .file-card.multi-selected {
    border-color: var(--blue);
    background: rgba(59, 130, 246, 0.15);
    box-shadow: inset 0 0 0 2px rgba(59, 130, 246, 0.4);
  }

  .file-card.clipboard-copy {
    border-color: var(--green-soft, #5be39a);
    background: rgba(91, 227, 154, 0.08);
    opacity: 0.85;
  }

  .file-card.clipboard-cut {
    border-color: var(--orange-soft, #ff8a3d);
    background: rgba(255, 138, 61, 0.08);
    opacity: 0.7;
  }

  .file-card.drag-over {
    border-color: var(--blue);
    background: rgba(59, 130, 246, 0.15);
    box-shadow: inset 0 0 0 2px var(--blue);
  }

  .file-surface {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 110px;
    background: var(--bg);
    border-radius: 6px;
    position: relative;
    overflow: hidden;
  }

  .folder-surface {
    background: rgba(250, 176, 5, 0.08);
    border: 1px solid rgba(250, 176, 5, 0.18);
  }

  .folder-card {
    border-color: rgba(250, 176, 5, 0.25) !important;
  }

  .folder-card:hover {
    border-color: rgba(250, 176, 5, 0.5) !important;
    background: rgba(250, 176, 5, 0.06) !important;
  }

  .folder-surface :global(.folder-glyph-big) {
    color: #f59e0b;
    filter: drop-shadow(0 2px 6px rgba(245, 158, 11, 0.35));
  }

  .quick-actions {
    position: absolute;
    bottom: 6px;
    left: 6px;
    display: flex;
    gap: 4px;
    opacity: 0;
    transition: opacity 150ms;
  }

  .file-card:hover .quick-actions {
    opacity: 1;
  }

  .quick-action {
    padding: 6px;
    background: rgba(0, 0, 0, 0.6);
    border: none;
    border-radius: 4px;
    color: white;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: background 150ms;
  }

  .quick-action:hover {
    background: rgba(0, 0, 0, 0.8);
  }

  .inline-badge {
    position: absolute;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    background: rgba(0, 0, 0, 0.6);
    color: white;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
    bottom: 6px;
    right: 6px;
  }

  .inline-badge.public {
    background: rgba(91, 227, 154, 0.2);
    color: #5be39a;
  }

  .inline-badge.pinned {
    background: rgba(255, 193, 7, 0.2);
    color: #ffc107;
  }

  .file-copy {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .file-copy strong {
    font-size: 13px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .filename-text {
    cursor: text;
    transition: color 150ms;
  }

  .filename-text:hover {
    color: var(--blue);
  }

  .rename-input {
    width: 100%;
    padding: 2px 4px;
    font-size: 13px;
    border: 1px solid var(--blue);
    border-radius: 3px;
    background: var(--bg);
    color: var(--text);
    font-weight: 500;
  }

  .rename-input:focus {
    outline: none;
    border-color: var(--blue);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
  }

  .file-copy span {
    font-size: 11px;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-list {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .list-head {
    display: grid;
    grid-template-columns: 2fr 1fr 1.5fr 1.5fr 1fr;
    gap: 12px;
    padding: 12px 16px;
    background: var(--surface-2);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .list-row {
    display: flex;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--hairline);
    background: transparent;
    cursor: pointer;
    transition: background 0.15s;
    text-align: left;
    align-items: center;
  }

  .list-row:hover {
    background: var(--surface-2);
  }

  .list-row.selected {
    background: rgba(59, 130, 246, 0.1);
  }

  .list-row.multi-selected {
    background: rgba(59, 130, 246, 0.15);
    border-left: 3px solid var(--blue);
    padding-left: 13px;
  }

  .list-row.clipboard-copy {
    background: rgba(91, 227, 154, 0.08);
    border-left: 3px solid #5be39a;
    padding-left: 13px;
    opacity: 0.85;
  }

  .list-row.clipboard-cut {
    background: rgba(255, 138, 61, 0.08);
    border-left: 3px solid #ff8a3d;
    padding-left: 13px;
    opacity: 0.7;
  }

  .list-row.drag-over {
    background: rgba(59, 130, 246, 0.15);
    border-left: 3px solid var(--blue);
    padding-left: 13px;
  }

  .list-row:last-child {
    border-bottom: none;
  }

  .list-row-content {
    display: grid;
    grid-template-columns: 2fr 1fr 1.5fr 1.5fr 1fr;
    gap: 12px;
    flex: 1;
    min-width: 0;
    padding: 0;
    text-align: left;
    align-items: center;
  }

  .list-row-actions {
    display: flex;
    gap: 8px;
    opacity: 0;
    transition: opacity 150ms;
    flex-shrink: 0;
  }

  .list-row:hover .list-row-actions {
    opacity: 1;
  }

  .row-action {
    padding: 6px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-2);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: all 150ms;
  }

  .row-action:hover {
    border-color: var(--blue);
    color: var(--blue);
    background: rgba(59, 130, 246, 0.05);
  }

  .name-cell,
  .state-cell {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .name-cell span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .list-thumb {
    width: 32px;
    height: 32px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 5px;
    overflow: hidden;
    background: var(--surface);
  }

  .list-thumbnail {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .folder-thumb {
    background: rgba(250, 176, 5, 0.1);
  }

  .folder-thumb :global(.folder-list-icon) {
    color: #f59e0b;
  }

  .folder-row {
    border-left: 2px solid rgba(250, 176, 5, 0.3);
  }

  .folder-row:hover {
    background: rgba(250, 176, 5, 0.04) !important;
  }

  .folder-name {
    font-weight: 500;
  }

  .load-more-row {
    display: flex;
    justify-content: center;
    padding: 18px 0 0;
  }

  .ghost-btn {
    padding: 8px 12px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--text-2);
    transition: all 0.2s;
  }

  .ghost-btn:hover:not(:disabled) {
    border-color: var(--blue);
    color: var(--blue);
  }

  .ghost-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spinner {
    width: 22px;
    height: 22px;
    border-radius: 999px;
    border: 2px solid rgba(255, 255, 255, 0.12);
    border-top-color: var(--blue);
    animation: spin 0.8s linear infinite;
  }

  .spinner.small {
    width: 14px;
    height: 14px;
    border-width: 2px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .file-thumbnail {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 0;
  }

  .video-thumbnail {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, rgba(30, 30, 30, 0.8), rgba(50, 50, 50, 0.8));
    color: var(--text-2);
    border-radius: 4px;
  }

  .video-play-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 4px;
    color: white;
    pointer-events: none;
  }

  .muted-state {
    font-size: 11px;
    color: var(--muted);
  }

  @media (max-width: 768px) {
    .file-grid {
      grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    }

    .list-head {
      display: none;
    }

    .list-row {
      grid-template-columns: 1fr;
      gap: 8px;
    }
  }
</style>
