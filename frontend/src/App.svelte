<script lang="ts">
  import { onMount } from "svelte";
  import Shell from "./lib/components/Shell.svelte";
  import { route, type RouteId } from "./lib/router";
  import { hydrateAuth } from "./lib/stores/auth";
  import DriveView from "./views/drive/DriveView.svelte";
  import NotesView from "./views/notes/NotesView.svelte";
  import OperationsView from "./views/operations/OperationsView.svelte";
  import PublicView from "./views/public/PublicView.svelte";
  import SearchView from "./views/search/SearchView.svelte";
  import WorkspaceView from "./views/workspace/WorkspaceView.svelte";

  const lensAlias: Record<RouteId, RouteId> = {
    drive: "drive",
    images: "drive",
    videos: "drive",
    documents: "drive",
    sharing: "sharing",
    notes: "notes",
    workspace: "workspace",
    operations: "operations",
    search: "search",
  };

  onMount(() => {
    void hydrateAuth();
  });
</script>

<Shell route={$route}>
  {#if lensAlias[$route] === "drive"}
    <DriveView lens={$route === "images" || $route === "videos" || $route === "documents" ? $route : "drive"} />
  {:else if $route === "notes"}
    <NotesView />
  {:else if $route === "workspace"}
    <WorkspaceView />
  {:else if $route === "sharing"}
    <PublicView />
  {:else if $route === "search"}
    <SearchView />
  {:else}
    <OperationsView />
  {/if}
</Shell>
