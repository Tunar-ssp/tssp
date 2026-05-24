<script lang="ts">
  import { onMount } from "svelte";
  import {
    getFileShare,
    listFiles,
    moveFileToFolder,
    setFileVisibility,
    uploadFilesBatch,
    type FileRecord,
    type ShareInfo,
  } from "../../lib/api";
  import { buildFolderEntries, type DriveLens, matchesDriveLens } from "../../lib/utils/files";
  import { formatBytes } from "../../lib/utils/format";
  import DriveDetailsPanel from "./DriveDetailsPanel.svelte";
  import DriveFileList from "./DriveFileList.svelte";
  import DriveFolderRail from "./DriveFolderRail.svelte";
  import DriveUploadBar from "./DriveUploadBar.svelte";

  export let lens: DriveLens = "drive";

  let files: FileRecord[] = [];
  let loading = true;
  let error = "";
  let selectedFileId: string | null = null;
  let folderPath = "";
  let shareInfo: ShareInfo | null = null;
  let shareStatus = "";
  let moveStatus = "";
  let uploadStatus = "";

  const copy = {
    drive: {
      eyebrow: "Cloud Drive",
      title: "One storage system, multiple lenses.",
      description:
        "Files, images, videos, and documents stay in the same object store. The new frontend models that as one product surface instead of disconnected pages.",
    },
    images: {
      eyebrow: "Drive Lens",
      title: "Images are a filtered view of the same cloud drive.",
      description:
        "Photos and screenshots stay inside the same storage model, with the lens focused on image-first browsing and preview flow.",
    },
    videos: {
      eyebrow: "Drive Lens",
      title: "Videos stay inside the same drive, with playback-first browsing.",
      description:
        "This lens narrows the object catalog to playable assets while preserving the same folder, sharing, and metadata model.",
    },
    documents: {
      eyebrow: "Drive Lens",
      title: "Documents are still drive objects, just viewed through a document lens.",
      description:
        "Text, PDF, and source files remain part of the same storage graph instead of being split into a fake separate product.",
    },
  } as const;

  async function refreshFiles() {
    loading = true;
    error = "";
    try {
      const response = await listFiles({ folder: folderPath || undefined, limit: 300 });
      files = (response.files || []).filter((file) => matchesDriveLens(file, lens));
      if (!selectedFileId || !files.some((file) => file.id === selectedFileId)) {
        selectedFileId = files[0]?.id || null;
      }
    } catch (nextError) {
      error = nextError instanceof Error ? nextError.message : "Failed to load files";
    } finally {
      loading = false;
    }
  }

  async function loadShareInfo(file: FileRecord) {
    shareStatus = "Loading public link...";
    try {
      shareInfo = await getFileShare(file.id);
      shareStatus = "Share metadata loaded";
    } catch (nextError) {
      shareInfo = null;
      shareStatus = nextError instanceof Error ? nextError.message : "Failed to load share metadata";
    }
  }

  async function handleToggleVisibility(file: FileRecord) {
    try {
      const visibility = file.visibility === "public" ? "private" : "public";
      const response = await setFileVisibility(file.id, visibility);
      files = files.map((item) => (item.id === file.id ? response.file : item));
      if (selectedFileId === file.id) {
        await loadShareInfo(response.file);
      }
      moveStatus = `Visibility updated to ${visibility}`;
    } catch (nextError) {
      moveStatus = nextError instanceof Error ? nextError.message : "Failed to update visibility";
    }
  }

  async function handleMoveSelected(file: FileRecord) {
    if (!folderPath.trim()) {
      moveStatus = "Enter a target folder path first";
      return;
    }
    try {
      const response = await moveFileToFolder(file.id, folderPath.trim());
      files = files.map((item) => (item.id === file.id ? response.file : item));
      moveStatus = `Moved to ${folderPath.trim()}`;
    } catch (nextError) {
      moveStatus = nextError instanceof Error ? nextError.message : "Failed to move file";
    }
  }

  async function handleCopyShare() {
    if (!shareInfo?.public_url) {
      shareStatus = "Load a public share first";
      return;
    }
    try {
      await navigator.clipboard.writeText(shareInfo.public_url);
      shareStatus = "Public link copied";
    } catch {
      shareStatus = shareInfo.public_url;
    }
  }

  async function handleUpload(filesToUpload: FileList | null) {
    if (!filesToUpload || filesToUpload.length === 0) return;
    uploadStatus = "Uploading...";
    try {
      const response = await uploadFilesBatch(Array.from(filesToUpload), {
        folderPath: folderPath.trim() || undefined,
      });
      const uploaded = response.results
        .map((item) => item.file)
        .filter((file): file is FileRecord => Boolean(file));
      if (uploaded.length > 0) {
        files = [...uploaded, ...files];
        selectedFileId = uploaded[0].id;
      }
      uploadStatus = `${uploaded.length} file(s) uploaded`;
    } catch (nextError) {
      uploadStatus = nextError instanceof Error ? nextError.message : "Upload failed";
    }
  }

  onMount(() => {
    void refreshFiles();
  });

  $: filteredFiles = files.filter((file) => matchesDriveLens(file, lens));
  $: folders = buildFolderEntries(filteredFiles);
  $: selectedFile = filteredFiles.find((file) => file.id === selectedFileId) || null;
  $: totalBytes = filteredFiles.reduce((sum, file) => sum + Number(file.size_bytes || 0), 0);
</script>

<section class="view-grid drive-layout">
  <div class="hero-card">
    <div>
      <div class="eyebrow">{copy[lens].eyebrow}</div>
      <h1>{copy[lens].title}</h1>
      <p>{copy[lens].description}</p>
    </div>
    <div class="hero-actions">
      <button class="btn btn-primary" type="button" on:click={refreshFiles}>Refresh</button>
      <button class="btn btn-secondary" type="button" on:click={() => selectedFile && loadShareInfo(selectedFile)} disabled={!selectedFile}>
        Share details
      </button>
    </div>
  </div>

  <div class="metric-row">
    <article class="metric-card">
      <span>Objects</span>
      <strong>{filteredFiles.length}</strong>
    </article>
    <article class="metric-card">
      <span>Public</span>
      <strong>{filteredFiles.filter((file) => file.visibility === "public").length}</strong>
    </article>
    <article class="metric-card">
      <span>Storage</span>
      <strong>{formatBytes(totalBytes)}</strong>
    </article>
    <article class="metric-card">
      <span>Upload</span>
      <strong>{uploadStatus || "idle"}</strong>
    </article>
  </div>

  <div class="three-panel">
    <DriveFolderRail
      folders={folders}
      activeFolder={folderPath}
      onSelectFolder={(nextFolder) => {
        folderPath = nextFolder;
        void refreshFiles();
      }}
    />

    <div class="panel-stack">
      <DriveUploadBar
        folderPath={folderPath}
        onFolderPathChange={(value) => (folderPath = value)}
        onUpload={(value) => void handleUpload(value)}
      />
      <DriveFileList
        files={filteredFiles}
        selectedFileId={selectedFileId}
        loading={loading}
        error={error}
        onSelectFile={(file) => {
          selectedFileId = file.id;
          void loadShareInfo(file);
        }}
      />
    </div>

    <DriveDetailsPanel
      file={selectedFile}
      folderPath={folderPath}
      shareInfo={shareInfo}
      shareStatus={shareStatus}
      moveStatus={moveStatus}
      onMoveFolder={() => selectedFile && void handleMoveSelected(selectedFile)}
      onToggleVisibility={() => selectedFile && void handleToggleVisibility(selectedFile)}
      onCopyShare={() => void handleCopyShare()}
      onRefreshShare={() => selectedFile && void loadShareInfo(selectedFile)}
    />
  </div>
</section>
