<script lang="ts">
  export let folderPath = "";
  export let onUpload: (files: FileList | null) => void;
  export let onFolderPathChange: (value: string) => void;
</script>

<article class="panel-card">
  <header class="panel-head">
    <strong>Upload</strong>
    <span>batch upload with logical folder target</span>
  </header>

  <div class="upload-stack">
    <input
      class="command-input"
      type="text"
      placeholder="Target folder path"
      value={folderPath}
      on:input={(event) => onFolderPathChange((event.currentTarget as HTMLInputElement).value)}
    />
    <input class="command-input" type="file" multiple on:change={(event) => onUpload((event.currentTarget as HTMLInputElement).files)} />
  </div>
</article>
