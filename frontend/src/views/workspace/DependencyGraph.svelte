<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface Dependency {
    id: string;
    from: string;
    to: string;
    type: 'import' | 'require' | 'reference';
  }

  interface $$Props {
    dependencies?: Dependency[];
    selectedFile?: string;
    onSelectFile?: (file: string) => void;
  }

  let {
    dependencies = [],
    selectedFile = '',
    onSelectFile = () => {},
  }: $$Props = $props();

  let viewMode = $state<'incoming' | 'outgoing'>('outgoing');

  let filteredDeps = $derived.by(() => {
    if (!selectedFile) return [];
    if (viewMode === 'outgoing') {
      return dependencies.filter((d) => d.from === selectedFile);
    } else {
      return dependencies.filter((d) => d.to === selectedFile);
    }
  });

  let uniqueFiles = $derived(
    Array.from(
      new Set(
        filteredDeps.map((d) => (viewMode === 'outgoing' ? d.to : d.from))
      )
    )
  );

  function getTypeIcon(type: string) {
    switch (type) {
      case 'import':
        return Icons.Download;
      case 'require':
        return Icons.Package;
      case 'reference':
        return Icons.Link;
      default:
        return Icons.Circle;
    }
  }

  function getTypeLabel(type: string): string {
    switch (type) {
      case 'import':
        return 'Import';
      case 'require':
        return 'Require';
      case 'reference':
        return 'Reference';
      default:
        return 'Dependency';
    }
  }
</script>

<div class="dependency-graph">
  <div class="graph-header">
    <h3>Dependencies</h3>
  </div>

  {#if !selectedFile}
    <div class="empty-state">
      <Icons.GitGraph size={24} />
      <p>Select a file to view dependencies</p>
    </div>
  {:else}
    <div class="graph-controls">
      <div class="file-selected">
        <Icons.File size={12} />
        <span>{selectedFile}</span>
      </div>
      <div class="view-mode">
        <button
          type="button"
          class:active={viewMode === 'outgoing'}
          onclick={() => (viewMode = 'outgoing')}
          title="Files this module depends on"
        >
          <Icons.ArrowRight size={12} />
          Out
        </button>
        <button
          type="button"
          class:active={viewMode === 'incoming'}
          onclick={() => (viewMode = 'incoming')}
          title="Files that depend on this module"
        >
          <Icons.ArrowLeft size={12} />
          In
        </button>
      </div>
    </div>

    {#if uniqueFiles.length === 0}
      <div class="empty-state">
        <Icons.Slack size={24} />
        <p>No {viewMode} dependencies</p>
      </div>
    {:else}
      <div class="deps-list">
        {#each filteredDeps as dep (dep.id)}
          <div class="dep-item">
            <div class="dep-type">
              <svelte:component this={getTypeIcon(dep.type)} size={12} />
              <span class="type-label">{getTypeLabel(dep.type)}</span>
            </div>
            <button
              type="button"
              class="dep-file"
              onclick={() =>
                onSelectFile(viewMode === 'outgoing' ? dep.to : dep.from)}
            >
              <Icons.File size={12} />
              <span>{viewMode === 'outgoing' ? dep.to : dep.from}</span>
            </button>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .dependency-graph {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
  }

  .graph-header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .graph-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 40px 16px;
    color: var(--muted);
    text-align: center;
  }

  .empty-state p {
    margin: 0;
    font-size: 13px;
  }

  .graph-controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    gap: 8px;
    flex-shrink: 0;
  }

  .file-selected {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text);
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .view-mode {
    display: flex;
    gap: 4px;
  }

  .view-mode button {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    color: var(--text-2);
    font-size: 11px;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .view-mode button:hover {
    background: var(--surface-2);
  }

  .view-mode button.active {
    background: var(--blue-soft);
    border-color: var(--blue);
    color: var(--blue);
  }

  .deps-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .dep-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px;
    background: var(--bg);
    border-radius: 8px;
    border: 1px solid var(--hairline);
  }

  .dep-type {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 10px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .type-label {
    display: none;
  }

  @media (min-width: 300px) {
    .type-label {
      display: inline;
    }
  }

  .dep-file {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 12px;
    cursor: pointer;
    text-align: left;
    overflow: hidden;
    min-width: 0;
  }

  .dep-file span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dep-file:hover {
    color: var(--blue);
  }
</style>
