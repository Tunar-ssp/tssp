<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { workspaceApi, type WorkspaceFileEntry } from '$lib/api';
  import FileIcon from '$lib/components/FileIcon.svelte';
  import { activeOverlays } from '$lib/stores/ui';

  interface Props {
    workspaceId: string;
    isOpen: boolean;
    onClose: () => void;
    onOpen: (path: string) => void;
  }

  let { workspaceId, isOpen, onClose, onOpen }: Props = $props();

  let query = $state('');
  let allFiles = $state<string[]>([]);
  let isIndexing = $state(false);
  let selectedIndex = $state(0);
  let inputRef: HTMLInputElement | null = $state(null);

  async function* walk(path = ''): AsyncGenerator<string> {
    const res = await workspaceApi.listWorkspaceFiles(workspaceId, path);
    for (const entry of (res.entries || []) as WorkspaceFileEntry[]) {
      if (entry.is_dir) {
        yield* walk(entry.path);
      } else {
        yield entry.path;
      }
    }
  }

  async function indexFiles() {
    isIndexing = true;
    const paths: string[] = [];
    try {
      for await (const path of walk('')) {
        paths.push(path);
      }
      allFiles = paths;
    } catch {
      allFiles = [];
    } finally {
      isIndexing = false;
    }
  }

  function fuzzyScore(text: string, q: string): number {
    if (!q) return 1;
    const lowerText = text.toLowerCase();
    const lowerQ = q.toLowerCase();
    if (lowerText.includes(lowerQ)) {
      const idx = lowerText.indexOf(lowerQ);
      return 1000 - idx;
    }
    let ti = 0;
    let score = 0;
    for (const c of lowerQ) {
      const found = lowerText.indexOf(c, ti);
      if (found === -1) return 0;
      score += 100 - (found - ti);
      ti = found + 1;
    }
    return score;
  }

  let results = $derived.by(() => {
    const scored = allFiles
      .map((p) => ({ path: p, score: fuzzyScore(p, query) }))
      .filter((x) => x.score > 0)
      .sort((a, b) => b.score - a.score)
      .slice(0, 50);
    return scored;
  });

  $effect(() => {
    if (isOpen) {
      activeOverlays.push('modal');
      return () => activeOverlays.remove('modal');
    }
  });

  $effect(() => {
    if (isOpen) {
      query = '';
      selectedIndex = 0;
      if (allFiles.length === 0 && !isIndexing) void indexFiles();
      setTimeout(() => inputRef?.focus(), 0);
    }
  });

  $effect(() => {
    if (selectedIndex >= results.length) selectedIndex = 0;
  });

  function selectFile(path: string) {
    onOpen(path);
    onClose();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const hit = results[selectedIndex];
      if (hit) selectFile(hit.path);
    }
  }

  function fileName(path: string): string {
    return path.split('/').pop() ?? path;
  }
  function dirName(path: string): string {
    const parts = path.split('/');
    parts.pop();
    return parts.join('/');
  }
</script>

{#if isOpen}
  <div class="overlay" role="presentation" onclick={onClose}>
    <div class="palette" role="dialog" aria-label="Quick open" onclick={(e) => e.stopPropagation()}>
      <div class="search-row">
        <Icons.Search size={16} />
        <input
          bind:this={inputRef}
          bind:value={query}
          onkeydown={onKeydown}
          placeholder="Type a filename… (Esc to close)"
        />
        <span class="hint">{results.length} {isIndexing ? '(indexing…)' : ''}</span>
      </div>
      <div class="results">
        {#if results.length === 0 && !isIndexing}
          <div class="empty">No matching files</div>
        {/if}
        {#each results as r, i (r.path)}
          <button
            type="button"
            class="row"
            class:active={i === selectedIndex}
            onclick={() => selectFile(r.path)}
            onmouseenter={() => selectedIndex = i}
          >
            <FileIcon name={fileName(r.path)} size={14} />
            <span class="name">{fileName(r.path)}</span>
            <span class="dir">{dirName(r.path)}</span>
          </button>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 12vh;
    z-index: 2000;
  }
  .palette {
    width: min(640px, 90vw);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    max-height: 60vh;
  }
  .search-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    color: var(--text-2);
  }
  .search-row input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text);
    font-size: 15px;
  }
  .hint {
    color: var(--muted);
    font-size: 12px;
    font-family: var(--ff-mono);
    flex-shrink: 0;
  }
  .results {
    overflow-y: auto;
    padding: 4px;
  }
  .empty {
    padding: 24px;
    text-align: center;
    color: var(--muted);
    font-size: 13px;
  }
  .row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 10px;
    border: none;
    background: transparent;
    color: var(--text);
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    font-size: 13px;
  }
  .row.active { background: rgba(59, 130, 246, 0.2); }
  .name { flex-shrink: 0; }
  .dir {
    flex: 1;
    color: var(--muted);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-align: right;
    font-family: monospace;
  }
</style>
