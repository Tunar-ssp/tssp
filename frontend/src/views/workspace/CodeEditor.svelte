<script lang="ts">
  export let value = "";
  export let language = "text";
  export let dirty = false;
  export let onChange: (value: string) => void;
  export let onSave: () => void;
  export let onCloseTab: () => void;

  let textarea: HTMLTextAreaElement | null = null;
  let findQuery = "";
  let showFind = false;

  $: lines = value.split("\n");
  $: lineCount = lines.length;
  $: charCount = value.length;
  $: wordCount = value.trim() ? value.trim().split(/\s+/).length : 0;

  function handleKeydown(ev: KeyboardEvent) {
    if ((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === "s") {
      ev.preventDefault();
      onSave();
    }
    if ((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === "w") {
      ev.preventDefault();
      onCloseTab();
    }
    if ((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === "f") {
      ev.preventDefault();
      showFind = !showFind;
    }
    if (ev.key === "Tab" && textarea) {
      ev.preventDefault();
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      value = `${value.slice(0, start)}  ${value.slice(end)}`;
      onChange(value);
      queueMicrotask(() => {
        if (textarea) {
          textarea.selectionStart = start + 2;
          textarea.selectionEnd = start + 2;
        }
      });
    }
  }

  function applyFind() {
    if (!textarea || !findQuery) return;
    const idx = value.toLowerCase().indexOf(findQuery.toLowerCase());
    if (idx >= 0) {
      textarea.focus();
      textarea.setSelectionRange(idx, idx + findQuery.length);
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="editor">
  {#if showFind}
    <div class="find-bar">
      <input type="search" placeholder="Find in file…" bind:value={findQuery} on:keydown={(e) => e.key === "Enter" && applyFind()} />
      <button type="button" class="btn btn-sm" on:click={applyFind}>Find</button>
    </div>
  {/if}
  <div class="editor-scroll">
    <div class="gutter" aria-hidden="true">
      {#each lines as _, i}
        <div class="ln">{i + 1}</div>
      {/each}
    </div>
    <textarea
      bind:this={textarea}
      class="code-area mono"
      spellcheck="false"
      {value}
      on:input={(e) => onChange(e.currentTarget.value)}
    ></textarea>
  </div>
  <footer class="status-bar">
    <span class="lang">{language}</span>
    <span>{lineCount} lines</span>
    <span>{wordCount} words</span>
    <span>{charCount} chars</span>
    <span class="save">{dirty ? "● unsaved" : "saved"}</span>
  </footer>
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    min-height: 0;
    flex: 1;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-card);
    overflow: hidden;
  }
  .find-bar {
    display: flex;
    gap: 8px;
    padding: 8px;
    border-bottom: 1px solid var(--border);
  }
  .find-bar input {
    flex: 1;
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
  }
  .editor-scroll {
    display: grid;
    grid-template-columns: 48px minmax(0, 1fr);
    flex: 1;
    min-height: 280px;
    overflow: auto;
  }
  .gutter {
    background: var(--bg-elevated);
    border-right: 1px solid var(--border);
    padding: 12px 0;
    text-align: right;
    color: var(--text-dim);
    font-size: 12px;
    line-height: 1.5;
    user-select: none;
  }
  .ln {
    padding-right: 10px;
  }
  .code-area {
    width: 100%;
    min-height: 100%;
    border: none;
    resize: none;
    padding: 12px;
    background: var(--bg-card);
    color: var(--text);
    font-size: 13px;
    line-height: 1.5;
    outline: none;
  }
  .status-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    padding: 6px 12px;
    border-top: 1px solid var(--border);
    background: var(--brand);
    color: #fff;
    font-size: 11px;
  }
  .lang {
    font-weight: 600;
    text-transform: uppercase;
  }
  .save {
    margin-left: auto;
  }
</style>
