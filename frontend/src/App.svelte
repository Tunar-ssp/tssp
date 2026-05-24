<script lang="ts">
  import { onMount } from "svelte";
  import Shell from "./lib/components/Shell.svelte";
  import DriveView from "./views/drive/DriveView.svelte";
  import NotesView from "./views/notes/NotesView.svelte";
  import WorkspaceView from "./views/workspace/WorkspaceView.svelte";
  import OperationsView from "./views/operations/OperationsView.svelte";
  import { appRoute, driveLensRoute } from "./lib/router";
  import { hydrateAuth } from "./lib/stores/auth";
  import { registerShortcuts } from "./lib/utils/keyboard";

  onMount(() => {
    void hydrateAuth();
    return registerShortcuts();
  });
</script>

<Shell app={$appRoute}>
  {#if $appRoute === "drive"}
    <DriveView lens={$driveLensRoute} />
  {:else if $appRoute === "knowledge"}
    <NotesView />
  {:else if $appRoute === "workspace"}
    <WorkspaceView />
  {:else if $appRoute === "operations"}
    <OperationsView />
  {/if}
</Shell>
