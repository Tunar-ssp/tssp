<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { workspaceApi, type WorkspaceFileEntry } from '$lib/api';
  import { findMatches } from '$lib/services/workspaceSearchService';
  import { error } from '$lib/stores/notifications';

  interface Props {
    workspaceId: string;
    onOpenMatch?: (path: string) => void;
  }

  let { workspaceId, onOpenMatch = () => {} }: Props = $props();

  function autofocusNode(node: HTMLInputElement) {
    queueMicrotask(() => node.focus());
  }

  interface FileMatch {
    path: string;
    matches: Array<{ line: number; preview: string }>;
  }

  let query = $state('');
  let caseSensitive = $state(false);
  let useRegex = $state(false);
  let wholeWord = $state(false);
  let isSearching = $state(false);
  let results = $state<FileMatch[]>([]);
  let totalMatches = $state(0);
  let scannedFiles = $state(0);
  let abortFlag = false;

  const TEXT_EXTENSIONS = new Set([
    'ts', 'tsx', 'js', 'jsx', 'mjs', 'cjs',
    'py', 'rs', 'go', 'java', 'kt', 'swift', 'rb', 'php', 'c', 'h', 'cpp', 'hpp', 'cs',
    'md', 'mdx', 'txt', 'json', 'yaml', 'yml', 'toml', 'ini', 'env',
    'html', 'htm', 'svelte', 'vue', 'css', 'scss', 'less',
    'sh', 'bash', 'zsh', 'fish', 'sql', 'graphql', 'proto',
  ]);

  function isText(path: string): boolean {
    const ext = path.split('.').pop()?.toLowerCase() ?? '';
    return TEXT_EXTENSIONS.has(ext);
  }

  async function* walkTree(path = ''): AsyncGenerator<string> {
    const res = await workspaceApi.listWorkspaceFiles(workspaceId, path);
    for (const entry of (res.entries || []) as WorkspaceFileEntry[]) {
      if (abortFlag) return;
      if (entry.is_dir) {
        yield* walkTree(entry.path);
      } else if (isText(entry.path)) {
        yield entry.path;
      }
    }
  }

  function previewLine(content: string, matchStart: number): { line: number; preview: string } {
    const before = content.slice(0, matchStart);
    const line = before.split('\n').length;
    const lineStart = before.lastIndexOf('\n') + 1;
    const lineEnd = content.indexOf('\n', matchStart);
    const preview = content.slice(lineStart, lineEnd === -1 ? content.length : lineEnd).trim();
    return { line, preview: preview.slice(0, 200) };
  }

  async function runSearch() {
    if (!query.trim()) {
      results = [];
      totalMatches = 0;
      return;
    }
    abortFlag = false;
    isSearching = true;
    results = [];
    totalMatches = 0;
    scannedFiles = 0;
    const opts = { matchCase: caseSensitive, regex: useRegex, wholeWord };
    const found: FileMatch[] = [];
    try {
      for await (const path of walkTree('')) {
        if (abortFlag) break;
        scannedFiles += 1;
        try {
          const { content } = await workspaceApi.readWorkspaceFile(workspaceId, path);
          const matches = findMatches(content || '', query, opts);
          if (matches.length > 0) {
            const previews = matches.slice(0, 50).map((m) => previewLine(content, m.matchStart));
            found.push({ path, matches: previews });
            totalMatches += matches.length;
            results = [...found];
          }
        } catch {
          // skip unreadable files
        }
      }
    } catch (err) {
      error('Search Failed', err instanceof Error ? err.message : 'Could not complete search');
    } finally {
      isSearching = false;
    }
  }

  function cancelSearch() {
    abortFlag = true;
    isSearching = false;
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      if (isSearching) cancelSearch();
      else void runSearch();
    } else if (e.key === 'Escape') {
      cancelSearch();
    }
  }
</script>

<aside class="search-panel">
  <header class="hdr">
    <div class="title">
      <Icons.Search size={14} />
      <span>SEARCH</span>
    </div>
  </header>

  <div class="controls">
    <div class="input-row">
      <input
        type="text"
        placeholder="Search across workspace..."
        bind:value={query}
        onkeydown={onKeydown}
        use:autofocusNode
      />
      {#if isSearching}
        <button type="button" class="ibtn danger" title="Cancel" onclick={cancelSearch}>
          <Icons.X size={14} />
        </button>
      {:else}
        <button type="button" class="ibtn" title="Search (Enter)" onclick={runSearch} disabled={!query.trim()}>
          <Icons.ArrowRight size={14} />
        </button>
      {/if}
    </div>
    <div class="toggle-row">
      <button type="button" class:on={caseSensitive} onclick={() => caseSensitive = !caseSensitive} title="Match case">Aa</button>
      <button type="button" class:on={wholeWord} onclick={() => wholeWord = !wholeWord} title="Match whole word">ab</button>
      <button type="button" class:on={useRegex} onclick={() => useRegex = !useRegex} title="Use regex">.*</button>
    </div>
  </div>

  <div class="status">
    {#if isSearching}
      <span class="spin"></span>
      <span>Scanned {scannedFiles} files… {totalMatches} matches</span>
    {:else if query.trim() && results.length === 0}
      <span>No results in {scannedFiles} files</span>
    {:else if results.length > 0}
      <span>{totalMatches} matches in {results.length} files</span>
    {/if}
  </div>

  <div class="results">
    {#each results as file (file.path)}
      <details open class="file-result">
        <summary>
          <Icons.FileText size={12} />
          <span class="path">{file.path}</span>
          <span class="count">{file.matches.length}</span>
        </summary>
        {#each file.matches as match}
          <button class="match" type="button" onclick={() => onOpenMatch(file.path)} title="Open {file.path}">
            <span class="line">{match.line}</span>
            <span class="preview">{match.preview}</span>
          </button>
        {/each}
      </details>
    {/each}
  </div>
</aside>

<style>
  .search-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 320px;
    flex-shrink: 0;
    background: rgba(14, 16, 22, 0.98);
    border-right: 1px solid var(--border);
    overflow: hidden;
  }
  .hdr {
    padding: 8px 10px;
    border-bottom: 1px solid var(--border);
  }
  .title {
    display: flex; align-items: center; gap: 6px;
    color: var(--text-2);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
  }
  .controls {
    padding: 10px;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .input-row {
    display: flex;
    gap: 6px;
  }
  .input-row input {
    flex: 1;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 10px;
    color: var(--text);
    font-size: 13px;
    outline: none;
    min-width: 0;
  }
  .input-row input:focus { border-color: var(--blue); }
  .ibtn {
    width: 28px;
    height: 28px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-2);
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .ibtn:hover:not(:disabled) { background: rgba(255,255,255,0.06); color: var(--text); }
  .ibtn:disabled { opacity: 0.4; cursor: not-allowed; }
  .ibtn.danger { color: #ef4444; border-color: rgba(239,68,68,0.3); }
  .toggle-row { display: flex; gap: 4px; }
  .toggle-row button {
    height: 24px;
    padding: 0 8px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-2);
    border-radius: 4px;
    font-size: 11px;
    font-family: monospace;
    cursor: pointer;
  }
  .toggle-row button.on {
    background: rgba(59,130,246,0.18);
    color: var(--text);
    border-color: var(--blue);
  }
  .status {
    padding: 6px 12px;
    font-size: 11px;
    color: var(--muted);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 24px;
  }
  .results {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }
  .file-result {
    border-bottom: 1px solid rgba(255,255,255,0.04);
  }
  .file-result summary {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    color: var(--text);
    font-size: 12px;
    cursor: pointer;
    user-select: none;
    list-style: none;
  }
  .file-result summary::-webkit-details-marker { display: none; }
  .file-result summary:hover { background: rgba(255,255,255,0.04); }
  .file-result .path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: monospace;
  }
  .file-result .count {
    color: var(--muted);
    font-size: 11px;
    padding: 1px 6px;
    border-radius: 10px;
    background: rgba(255,255,255,0.06);
  }
  .match {
    width: 100%;
    display: flex;
    gap: 10px;
    padding: 3px 18px;
    border: none;
    background: transparent;
    color: var(--text-2);
    font-size: 12px;
    cursor: pointer;
    text-align: left;
    font-family: monospace;
    overflow: hidden;
  }
  .match:hover { background: rgba(59,130,246,0.12); color: var(--text); }
  .line {
    color: var(--muted);
    flex-shrink: 0;
    min-width: 30px;
    text-align: right;
  }
  .preview {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }
  .spin {
    width: 10px;
    height: 10px;
    border: 2px solid rgba(255,255,255,0.15);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
