<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import FolderTree from '$lib/components/FolderTree.svelte';

  interface Props {
    folders: string[];
    onSelectFolder: (path: string) => void;
  }

  let { folders, onSelectFolder }: Props = $props();
</script>

<aside class="folder-rail">
  <div class="rail-header">
    <h3>Folders</h3>
    <Icons.FolderOpen size={16} />
  </div>
  <FolderTree {folders} onSelectFolder={onSelectFolder} />
</aside>

<style>
  .folder-rail {
    flex-shrink: 0;
    width: 240px;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border-right: 1px solid var(--border);
    overflow: hidden;
  }

  .rail-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-4);
    border-bottom: 1px solid var(--border);
  }

  .rail-header h3 {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
  }
</style>
