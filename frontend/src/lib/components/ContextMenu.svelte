<script lang="ts">
  import type { FileRecord } from "../api";

  export let x = 0;
  export let y = 0;
  export let file: FileRecord | null = null;
  export let onClose: () => void;
  export let onPreview: (file: FileRecord) => void;
  export let onShare: (file: FileRecord) => void;
  export let onRename: (file: FileRecord) => void;
  export let onDelete: (file: FileRecord) => void;
  export let onToggleVisibility: (file: FileRecord) => void;

  function act(fn: () => void) {
    fn();
    onClose();
  }
</script>

{#if file}
  <button class="ctx-backdrop" type="button" aria-label="Close context menu" on:click={onClose} on:contextmenu|preventDefault={onClose}></button>
  <div
    class="ctx-menu"
    role="menu"
    tabindex="-1"
    style={`left:${x}px;top:${y}px`}
    on:click|stopPropagation
    on:keydown|stopPropagation
  >
    <button type="button" role="menuitem" on:click={() => act(() => onPreview(file!))}>Preview</button>
    <button type="button" role="menuitem" on:click={() => act(() => onShare(file!))}>Share / QR</button>
    <button type="button" role="menuitem" on:click={() => act(() => onRename(file!))}>Rename</button>
    <button type="button" role="menuitem" on:click={() => act(() => onToggleVisibility(file!))}>
      {file.visibility === "public" ? "Make private" : "Make public"}
    </button>
    <button type="button" role="menuitem" class="danger" on:click={() => act(() => onDelete(file!))}>Delete</button>
  </div>
{/if}

<style>
  .ctx-backdrop { position: fixed; inset: 0; z-index: 70; border: 0; background: transparent; padding: 0; }
  .ctx-menu {
    position: fixed;
    margin: 0;
    padding: 6px;
    list-style: none;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.45);
    min-width: 160px;
    display: grid;
    gap: 2px;
  }
  .ctx-menu button {
    width: 100%;
    text-align: left;
    border: none;
    background: transparent;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    color: var(--text);
  }
  .ctx-menu button:hover { background: var(--bg-hover); }
  .ctx-menu button.danger { color: var(--red); }
</style>
