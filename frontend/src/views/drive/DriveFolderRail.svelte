<script lang="ts">
  import type { FolderEntry } from "../../lib/utils/files";

  export let folders: FolderEntry[] = [];
  export let activeFolder = "";
  export let onSelectFolder: (folder: string) => void;
</script>

<article class="panel-card">
  <header class="panel-head">
    <strong>Folders</strong>
    <span>bucket tree</span>
  </header>

  <div class="tree-list">
    {#each folders as folder}
      <button
        type="button"
        class:active={activeFolder === folder.path}
        class="tree-item"
        on:click={() => onSelectFolder(folder.path)}
      >
        <span>{folder.label}</span>
        <span class="nav-link-subtitle">{folder.count} item(s)</span>
      </button>
    {/each}
  </div>
</article>
