<script lang="ts">
  import { runSearch, type SearchResult } from "../../lib/api";

  const recentQueries = ["orange pi", "workspace", "public links"];
  let query = "";
  let loading = false;
  let error = "";
  let results: SearchResult[] = [];

  async function submitSearch() {
    if (!query.trim()) {
      results = [];
      error = "";
      return;
    }
    loading = true;
    error = "";
    try {
      const response = await runSearch(query.trim());
      results = response.results || [];
    } catch (nextError) {
      error = nextError instanceof Error ? nextError.message : "Search failed";
    } finally {
      loading = false;
    }
  }

  $: grouped = {
    files: results.filter((result) => result.type === "file"),
    notes: results.filter((result) => result.type === "note"),
    workspaces: results.filter((result) => result.type === "workspace"),
  };
</script>

<section class="view-grid">
  <div class="hero-card compact">
    <div>
      <div class="eyebrow">Global Search</div>
      <h1>One search system across files, notes, workspaces, and commands.</h1>
      <p>
        Command palette and search page share the same mental model. Recent searches,
        grouped results, and direct actions live in one place.
      </p>
    </div>
  </div>

  <div class="split-view">
    <article class="panel-card">
      <header class="panel-head">
        <strong>Recent searches</strong>
        <span>persisted locally</span>
      </header>
      <div class="chip-row">
        {#each recentQueries as recent}
          <button type="button" class="chip" on:click={() => (query = recent)}>{recent}</button>
        {/each}
      </div>
    </article>

    <article class="panel-card">
      <header class="panel-head">
        <strong>Grouped results</strong>
        <span>files, notes, workspaces</span>
      </header>
      <form class="search-form" on:submit|preventDefault={submitSearch}>
        <input bind:value={query} class="command-input" type="search" placeholder="Search the live backend" />
        <button class="btn btn-primary" type="submit">Search</button>
      </form>

      {#if loading}
        <div class="empty-copy">Searching…</div>
      {:else if error}
        <div class="empty-copy">{error}</div>
      {:else if !query.trim()}
        <div class="empty-copy">Enter a query to exercise the shared search API.</div>
      {:else if results.length === 0}
        <div class="empty-copy">No results matched “{query}”.</div>
      {:else}
        <div class="stack-list">
          <div class="stack-card">
            <strong>Files ({grouped.files.length})</strong>
            <span>{grouped.files[0]?.title || grouped.files[0]?.name || "No file hits"}</span>
          </div>
          <div class="stack-card">
            <strong>Notes ({grouped.notes.length})</strong>
            <span>{grouped.notes[0]?.title || grouped.notes[0]?.name || "No note hits"}</span>
          </div>
          <div class="stack-card">
            <strong>Workspaces ({grouped.workspaces.length})</strong>
            <span>{grouped.workspaces[0]?.title || grouped.workspaces[0]?.name || "No workspace hits"}</span>
          </div>
        </div>
      {/if}
    </article>
  </div>
</section>
