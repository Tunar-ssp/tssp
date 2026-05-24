<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { currentFolder } from '$lib/stores/drive';

  export let onUpload: (files: FileList, folder: string) => void = () => {};

  let isDragging = false;
  let fileInput: HTMLInputElement;

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;

    const files = e.dataTransfer?.files;
    if (files) {
      onUpload(files, $currentFolder);
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  function handleDragLeave() {
    isDragging = false;
  }

  function handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files) {
      onUpload(input.files, $currentFolder);
      input.value = '';
    }
  }
</script>

<div
  class="upload-area"
  class:dragging={isDragging}
  on:drop={handleDrop}
  on:dragover={handleDragOver}
  on:dragleave={handleDragLeave}
  role="button"
  tabindex="0"
>
  <div class="upload-content">
    <Icons.Upload size={28} />
    <div class="upload-text">
      <p class="upload-title">Drop files here to upload</p>
      <p class="upload-subtitle">or click to browse</p>
    </div>
  </div>

  <input
    bind:this={fileInput}
    type="file"
    multiple
    on:change={handleFileSelect}
    style="display: none"
  />
  <button on:click={() => fileInput?.click()} style="display: none" />
</div>

<style>
  .upload-area {
    flex: 1;
    border: 2px dashed var(--border-2);
    border-radius: var(--r-3);
    padding: 32px 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s;
    background: var(--surface);
  }

  .upload-area:hover {
    border-color: var(--border);
    background: var(--surface-2);
  }

  .upload-area.dragging {
    border-color: var(--blue);
    background: var(--blue-soft);
  }

  .upload-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    color: var(--muted);
  }

  .upload-area:hover .upload-content {
    color: var(--text-2);
  }

  .upload-area.dragging .upload-content {
    color: var(--blue);
  }

  .upload-text {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .upload-title {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 500;
  }

  .upload-subtitle {
    margin: 0;
    font-size: var(--fs-12);
    color: var(--muted);
  }
</style>
