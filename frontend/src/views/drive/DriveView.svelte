<script lang="ts">
  import { onMount } from "svelte";
  import {
    addFileTags,
    bulkSetVisibility,
    deleteFile,
    deleteFolder,
    getFileShare,
    listAdminFolders,
    listFiles,
    moveFileToFolder,
    moveFolder,
    pinFile,
    renameFile,
    setFileVisibility,
    unpinFile,
    uploadFilesBatch,
    fileContentUrl,
    fileDownloadUrl,
    type FileRecord,
    type FolderEntry,
    type ShareInfo,
  } from "../../lib/api";
  import {
    driveFiles,
    driveFolder,
    driveLens,
    driveLoading,
    driveError,
    driveViewMode,
    driveQuery,
    driveSort,
    focusedFileId,
    previewFileId,
    selectedIds,
    shareInfo,
    uploadDragOver,
    toggleSelection,
    clearSelection,
    selectAll,
  } from "../../lib/stores/drive";
  import type { DriveLens } from "../../lib/router";
  import { navigateDriveLens } from "../../lib/router";
  import { matchesDriveLens } from "../../lib/utils/files";
  import { formatAbsoluteDate, formatBytes } from "../../lib/utils/format";
  import PreviewDialog from "../../lib/components/PreviewDialog.svelte";

  export let lens: DriveLens = "all";

  let folders: FolderEntry[] = [];
  let localQuery = "";
  let statusMsg = "";
  let showShareModal = false;
  let shareQr = "";
  let shareUrl = "";
  let uploadInput: HTMLInputElement | null = null;

  async function refresh() {
    driveLoading.set(true);
    driveError.set("");
    try {
      const [fileRes, folderRes] = await Promise.all([
        listFiles({ folder: $driveFolder || undefined, limit: 400 }),
        listAdminFolders().catch(() => ({ folders: [] as FolderEntry[] })),
      ]);
      driveFiles.set(fileRes.files || []);
      folders = folderRes.folders || [];
    } catch (e) {
      driveError.set(e instanceof Error ? e.message : "Failed to load drive");
    } finally {
      driveLoading.set(false);
    }
  }

  $: driveLens.set(lens);
  $: filtered = $driveFiles
    .filter((f) => matchesDriveLens(f, lens))
    .filter((f) => {
      const q = ($driveQuery || localQuery).trim().toLowerCase();
      if (!q) return true;
      return (
        f.name.toLowerCase().includes(q) ||
        (f.tags || []).some((t) => t.toLowerCase().includes(q))
      );
    })
    .sort((a, b) => {
      if ($driveSort === "name") return a.name.localeCompare(b.name);
      if ($driveSort === "size") return b.size_bytes - a.size_bytes;
      return b.uploaded_at - a.uploaded_at;
    });

  $: breadcrumb = $driveFolder
    ? ["Bucket", ...$driveFolder.split("/").filter(Boolean)]
    : ["Bucket"];

  async function handleUpload(files: FileList | File[] | null) {
    if (!files || files.length === 0) return;
    statusMsg = "Uploading…";
    try {
      const res = await uploadFilesBatch(Array.from(files), {
        folderPath: $driveFolder || undefined,
      });
      const ok = res.results.filter((r) => r.file).length;
      statusMsg = `${ok} file(s) uploaded`;
      await refresh();
    } catch (e) {
      statusMsg = e instanceof Error ? e.message : "Upload failed";
    }
  }

  async function openShare(file: FileRecord) {
    try {
      if (file.visibility !== "public") {
        const res = await setFileVisibility(file.id, "public");
        file = res.file;
      }
      const info = await getFileShare(file.id);
      shareUrl = info.public_url;
      shareQr = info.qr_terminal;
      showShareModal = true;
    } catch (e) {
      statusMsg = e instanceof Error ? e.message : "Share failed";
    }
  }

  async function handleDelete(file: FileRecord) {
    if (!confirm(`Delete "${file.name}"?`)) return;
    await deleteFile(file.id);
    await refresh();
  }

  async function handleRename(file: FileRecord) {
    const name = prompt("New name", file.name);
    if (!name || name === file.name) return;
    await renameFile(file.id, name);
    await refresh();
  }

  async function handleMoveHere(file: FileRecord) {
    await moveFileToFolder(file.id, $driveFolder);
    await refresh();
  }

  async function createFolder() {
    const name = prompt("New folder name");
    if (!name?.trim()) return;
    const path = $driveFolder ? `${$driveFolder}/${name.trim()}` : name.trim();
    statusMsg = `Folder "${path}" — upload a file here to create it`;
    driveFolder.set(path);
  }

  async function renameFolder() {
    if (!$driveFolder) return;
    const to = prompt("Rename folder to", $driveFolder);
    if (!to || to === $driveFolder) return;
    await moveFolder($driveFolder, to);
    driveFolder.set(to);
    await refresh();
  }

  async function removeFolder() {
    if (!$driveFolder) return;
    if (!confirm(`Remove folder "${$driveFolder}"? Files move to root.`)) return;
    await deleteFolder($driveFolder);
    driveFolder.set("");
    await refresh();
  }

  async function bulkAction(action: string) {
    const ids = [...$selectedIds];
    if (!ids.length) return;
    if (action === "delete") {
      if (!confirm(`Delete ${ids.length} files?`)) return;
      for (const id of ids) await deleteFile(id);
    } else if (action === "public" || action === "private") {
      await bulkSetVisibility(ids, action);
    } else if (action === "pin") {
      for (const id of ids) await pinFile(id);
    } else if (action === "unpin") {
      for (const id of ids) await unpinFile(id);
    }
    clearSelection();
    await refresh();
  }

  function onDrop(ev: DragEvent) {
    ev.preventDefault();
    uploadDragOver.set(false);
    void handleUpload(ev.dataTransfer?.files || null);
  }

  onMount(() => {
    void refresh();
  });
</script>

<section
  class="drive"
  on:dragover|preventDefault={() => uploadDragOver.set(true)}
  on:dragleave={() => uploadDragOver.set(false)}
  on:drop={onDrop}
>
  {#if $uploadDragOver}
    <div class="drop-overlay">Drop files to upload</div>
  {/if}

  <header class="drive-toolbar">
    <nav class="breadcrumb" aria-label="Folder path">
      {#each breadcrumb as part, i}
        {#if i > 0}<span class="sep">/</span>{/if}
        <button
          type="button"
          class="crumb"
          on:click={() => {
            if (i === 0) driveFolder.set("");
            else driveFolder.set(breadcrumb.slice(1, i + 1).join("/"));
            void refresh();
          }}
        >{part}</button>
      {/each}
    </nav>

    <div class="toolbar-actions">
      <input
        class="search-input"
        type="search"
        placeholder="Filter in folder…"
        bind:value={localQuery}
        on:input={() => driveQuery.set(localQuery)}
      />
      <select class="select" bind:value={$driveSort} on:change={() => driveSort.set($driveSort)}>
        <option value="date">Date</option>
        <option value="name">Name</option>
        <option value="size">Size</option>
      </select>
      <button type="button" class="btn btn-sm" class:active={$driveViewMode === "grid"} on:click={() => driveViewMode.set("grid")}>Grid</button>
      <button type="button" class="btn btn-sm" class:active={$driveViewMode === "table"} on:click={() => driveViewMode.set("table")}>List</button>
      <button type="button" class="btn btn-primary btn-sm" on:click={() => uploadInput?.click()}>Upload</button>
      <input bind:this={uploadInput} type="file" multiple hidden on:change={(e) => void handleUpload(e.currentTarget.files)} />
    </div>
  </header>

  <div class="lens-tabs">
    {#each [
      { id: "all", label: "All" },
      { id: "images", label: "Images" },
      { id: "videos", label: "Videos" },
      { id: "documents", label: "Documents" },
    ] as tab}
      <button
        type="button"
        class="lens-tab"
        class:active={lens === tab.id}
        on:click={() => navigateDriveLens(tab.id as DriveLens)}
      >{tab.label}</button>
    {/each}
  </div>

  <div class="drive-body">
    <aside class="folder-panel">
      <div class="panel-head">
        <strong>Folders</strong>
        <button type="button" class="btn btn-ghost btn-sm" on:click={createFolder}>+</button>
      </div>
      <button type="button" class="folder-row" class:active={!$driveFolder} on:click={() => { driveFolder.set(""); void refresh(); }}>
        Bucket root
      </button>
      {#each folders.filter((f) => f.path) as folder}
        <button
          type="button"
          class="folder-row"
          class:active={$driveFolder === folder.path}
          on:click={() => { driveFolder.set(folder.path); void refresh(); }}
        >
          <span>{folder.path}</span>
          <span class="count">{folder.file_count}</span>
        </button>
      {/each}
      {#if $driveFolder}
        <div class="folder-actions">
          <button type="button" class="btn btn-sm" on:click={renameFolder}>Rename</button>
          <button type="button" class="btn btn-sm btn-danger" on:click={removeFolder}>Delete</button>
        </div>
      {/if}
    </aside>

    <div class="main-panel">
      {#if $selectedIds.size > 0}
        <div class="bulk-bar">
          <span>{$selectedIds.size} selected</span>
          <button type="button" class="btn btn-sm" on:click={() => bulkAction("public")}>Public</button>
          <button type="button" class="btn btn-sm" on:click={() => bulkAction("private")}>Private</button>
          <button type="button" class="btn btn-sm" on:click={() => bulkAction("pin")}>Pin</button>
          <button type="button" class="btn btn-sm btn-danger" on:click={() => bulkAction("delete")}>Delete</button>
          <button type="button" class="btn btn-ghost btn-sm" on:click={clearSelection}>Clear</button>
        </div>
      {/if}

      {#if $driveLoading}
        <div class="skeleton-grid">
          {#each Array(8) as _}
            <div class="skeleton card-skeleton"></div>
          {/each}
        </div>
      {:else if $driveError}
        <div class="empty-state"><strong>Could not load drive</strong>{$driveError}</div>
      {:else if filtered.length === 0}
        <div class="empty-state">
          <strong>No files here</strong>
          Upload files or change folder. Drag and drop anywhere on this view.
        </div>
      {:else if $driveViewMode === "grid"}
        <div class="file-grid">
          {#each filtered as file}
            <article class="file-card" class:focused={$focusedFileId === file.id}>
              <label class="select-check">
                <input type="checkbox" checked={$selectedIds.has(file.id)} on:change={(e) => toggleSelection(file.id, e.currentTarget.checked)} />
              </label>
              <button type="button" class="card-preview" on:click={() => previewFileId.set(file.id)} on:dblclick={() => previewFileId.set(file.id)}>
                {#if file.mime_type?.startsWith("image/")}
                  <img src={fileContentUrl(file.id)} alt="" loading="lazy" />
                {:else}
                  <span class="ext">{file.name.split(".").pop()?.slice(0, 4) || "file"}</span>
                {/if}
              </button>
              <div class="card-meta">
                <button type="button" class="card-name" on:click={() => focusedFileId.set(file.id)}>{file.name}</button>
                <span>{formatBytes(file.size_bytes)}</span>
                <span class="badge" class:public={file.visibility === "public"}>{file.visibility}</span>
              </div>
            </article>
          {/each}
        </div>
      {:else}
        <table class="file-table">
          <thead>
            <tr>
              <th><input type="checkbox" on:change={(e) => e.currentTarget.checked ? selectAll(filtered.map((f) => f.id)) : clearSelection()} /></th>
              <th>Name</th>
              <th>Size</th>
              <th>Modified</th>
              <th>Visibility</th>
            </tr>
          </thead>
          <tbody>
            {#each filtered as file}
              <tr class:focused={$focusedFileId === file.id} on:click={() => focusedFileId.set(file.id)}>
                <td><input type="checkbox" checked={$selectedIds.has(file.id)} on:change={(e) => toggleSelection(file.id, e.currentTarget.checked)} /></td>
                <td><button type="button" class="linkish" on:click={() => previewFileId.set(file.id)}>{file.name}</button></td>
                <td>{formatBytes(file.size_bytes)}</td>
                <td>{formatAbsoluteDate(file.uploaded_at)}</td>
                <td><span class="badge" class:public={file.visibility === "public"}>{file.visibility}</span></td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>

    <aside class="details-panel">
      {#if filtered.find((f) => f.id === $focusedFileId)}
        {@const file = filtered.find((f) => f.id === $focusedFileId)!}
        <h3>{file.name}</h3>
        <dl class="meta">
          <dt>Size</dt><dd>{formatBytes(file.size_bytes)}</dd>
          <dt>Type</dt><dd class="mono">{file.mime_type || "—"}</dd>
          <dt>Folder</dt><dd>{file.folder_path || "Bucket root"}</dd>
          <dt>Uploaded</dt><dd>{formatAbsoluteDate(file.uploaded_at)}</dd>
          <dt>Tags</dt><dd>{#each file.tags || [] as t}<span class="tag">{t}</span>{/each}{#if !file.tags?.length}—{/if}</dd>
        </dl>
        <div class="detail-actions">
          <button type="button" class="btn btn-sm btn-primary" on:click={() => previewFileId.set(file.id)}>Preview</button>
          <a class="btn btn-sm" href={fileDownloadUrl(file.id)} download>Download</a>
          <button type="button" class="btn btn-sm" on:click={() => openShare(file)}>Share / QR</button>
          <button type="button" class="btn btn-sm" on:click={() => handleMoveHere(file)}>Move here</button>
          <button type="button" class="btn btn-sm" on:click={() => handleRename(file)}>Rename</button>
          <button type="button" class="btn btn-sm" on:click={() => file.pinned ? unpinFile(file.id).then(refresh) : pinFile(file.id).then(refresh)}>{file.pinned ? "Unpin" : "Pin"}</button>
          <button type="button" class="btn btn-sm btn-danger" on:click={() => handleDelete(file)}>Delete</button>
        </div>
      {:else}
        <div class="empty-state"><strong>No selection</strong>Select a file to inspect metadata and actions.</div>
      {/if}
    </aside>
  </div>

  {#if statusMsg}<p class="status-line">{statusMsg}</p>{/if}
</section>

{#if $previewFileId}
  <PreviewDialog file={filtered.find((f) => f.id === $previewFileId) || null} files={filtered} onClose={() => previewFileId.set(null)} onShare={openShare} />
{/if}

{#if showShareModal}
  <div class="modal-backdrop" role="presentation" on:click={() => (showShareModal = false)}>
    <div class="modal" role="dialog" on:click|stopPropagation>
      <h3>Public link</h3>
      <p class="mono share-url">{shareUrl}</p>
      <pre class="qr">{shareQr}</pre>
      <div class="detail-actions">
        <button type="button" class="btn btn-primary" on:click={() => navigator.clipboard.writeText(shareUrl)}>Copy link</button>
        <button type="button" class="btn" on:click={() => (showShareModal = false)}>Close</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .drive { position: relative; display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .drop-overlay { position: absolute; inset: 0; z-index: 20; display: grid; place-items: center; background: rgba(37,99,235,0.15); border: 2px dashed var(--brand); font-weight: 600; }
  .drive-toolbar { display: flex; flex-wrap: wrap; gap: 12px; align-items: center; justify-content: space-between; padding: 12px 16px; border-bottom: 1px solid var(--border); }
  .breadcrumb { display: flex; flex-wrap: wrap; align-items: center; gap: 4px; }
  .crumb { background: none; border: none; color: var(--blue); font-size: 13px; padding: 2px 4px; }
  .sep { color: var(--text-dim); }
  .toolbar-actions { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
  .search-input, .select { border: 1px solid var(--border); background: var(--bg-card); border-radius: var(--radius-sm); padding: 6px 10px; font-size: 13px; }
  .lens-tabs { display: flex; gap: 4px; padding: 8px 16px; border-bottom: 1px solid var(--border); }
  .lens-tab { border: 1px solid transparent; background: transparent; padding: 6px 12px; border-radius: var(--radius-sm); font-size: 12px; color: var(--text-muted); }
  .lens-tab.active { background: var(--brand-dim); color: var(--text); border-color: rgba(37,99,235,0.35); }
  .drive-body { flex: 1; min-height: 0; display: grid; grid-template-columns: 220px minmax(0, 1fr) var(--details-width); }
  .folder-panel, .details-panel, .main-panel { min-height: 0; overflow: auto; border-right: 1px solid var(--border); padding: 12px; }
  .details-panel { border-right: none; border-left: 1px solid var(--border); }
  .panel-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; font-size: 12px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.08em; }
  .folder-row { width: 100%; display: flex; justify-content: space-between; gap: 8px; border: none; background: transparent; text-align: left; padding: 8px 10px; border-radius: var(--radius-sm); font-size: 13px; }
  .folder-row:hover, .folder-row.active { background: var(--bg-hover); }
  .count { color: var(--text-dim); font-size: 11px; }
  .folder-actions { display: flex; gap: 6px; margin-top: 12px; }
  .bulk-bar { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; padding-bottom: 10px; border-bottom: 1px solid var(--border); margin-bottom: 10px; font-size: 13px; }
  .file-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 12px; }
  .file-card { position: relative; border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--bg-card); overflow: hidden; }
  .file-card.focused { border-color: var(--brand); }
  .select-check { position: absolute; top: 8px; left: 8px; z-index: 2; }
  .card-preview { width: 100%; aspect-ratio: 1; border: none; background: var(--bg-surface); display: grid; place-items: center; padding: 0; }
  .card-preview img { width: 100%; height: 100%; object-fit: cover; }
  .ext { font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; }
  .card-meta { padding: 8px 10px; display: grid; gap: 4px; font-size: 11px; color: var(--text-muted); }
  .card-name { border: none; background: none; text-align: left; color: var(--text); font-weight: 600; font-size: 12px; padding: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .file-table { width: 100%; border-collapse: collapse; font-size: 13px; }
  .file-table th, .file-table td { padding: 8px 10px; border-bottom: 1px solid var(--border); text-align: left; }
  .file-table tr.focused { background: var(--brand-dim); }
  .linkish { background: none; border: none; color: var(--blue); padding: 0; }
  .meta { display: grid; grid-template-columns: auto 1fr; gap: 6px 12px; font-size: 13px; }
  .meta dt { color: var(--text-muted); }
  .detail-actions { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 16px; }
  .skeleton-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 12px; }
  .card-skeleton { height: 160px; }
  .status-line { padding: 8px 16px; font-size: 12px; color: var(--text-muted); border-top: 1px solid var(--border); margin: 0; }
  .modal-backdrop { position: fixed; inset: 0; z-index: 50; background: rgba(0,0,0,0.6); display: grid; place-items: center; padding: 16px; }
  .modal { width: min(480px, 100%); background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--radius-lg); padding: 20px; }
  .share-url { word-break: break-all; font-size: 12px; }
  .qr { background: #fff; color: #000; padding: 12px; font-size: 0.45rem; line-height: 1; overflow: auto; border-radius: var(--radius-sm); }
  .btn.active { border-color: var(--brand); background: var(--brand-dim); }
  @media (max-width: 1100px) { .drive-body { grid-template-columns: 1fr; } .folder-panel, .details-panel { display: none; } }
</style>
