<script lang="ts">
  import { workspaces, activeWorkspace, loadWorkspaces, setActiveWorkspace } from '$lib/stores/workspace';
  import WorkspaceList from '$lib/components/WorkspaceList.svelte';
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import { onMount } from 'svelte';

  onMount(() => {
    loadWorkspaces();
  });

  function handleSelectWorkspace(id: string) {
    setActiveWorkspace(id);
  }
</script>

<div class="workspace-view">
  <WorkspaceList workspaces={$workspaces} onSelectWorkspace={handleSelectWorkspace} />
  <CodeEditor workspace={$activeWorkspace} />
</div>

<style>
  .workspace-view {
    flex: 1;
    display: flex;
    overflow: hidden;
    background: var(--bg);
  }
</style>
