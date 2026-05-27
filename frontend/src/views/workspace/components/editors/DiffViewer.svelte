<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    originalContent?: string;
    modifiedContent?: string;
    fileName?: string;
    onClose?: () => void;
  }

  let {
    originalContent = '',
    modifiedContent = '',
    fileName = 'file.txt',
    onClose = () => {},
  }: $$Props = $props();

  function getDiffLines() {
    const originalLines = originalContent.split('\n');
    const modifiedLines = modifiedContent.split('\n');
    const lines: Array<{ type: 'added' | 'removed' | 'unchanged'; line: string; lineNumber: number }> = [];

    const maxLines = Math.max(originalLines.length, modifiedLines.length);

    for (let i = 0; i < maxLines; i++) {
      const origLine = originalLines[i];
      const modLine = modifiedLines[i];

      if (origLine === modLine) {
        if (origLine !== undefined) {
          lines.push({ type: 'unchanged', line: origLine, lineNumber: i + 1 });
        }
      } else {
        if (origLine !== undefined) {
          lines.push({ type: 'removed', line: origLine, lineNumber: i + 1 });
        }
        if (modLine !== undefined) {
          lines.push({ type: 'added', line: modLine, lineNumber: i + 1 });
        }
      }
    }

    return lines;
  }

  let diffLines = $derived(getDiffLines());
</script>

<div class="diff-viewer">
  <div class="diff-header">
    <div class="diff-title">
      <Icons.GitCompare size={18} />
      <span>{fileName}</span>
    </div>
    <button
      type="button"
      class="close-btn"
      onclick={onClose}
      aria-label="Close diff"
    >
      <Icons.X size={18} />
    </button>
  </div>

  <div class="diff-content">
    {#each diffLines as item (item.lineNumber)}
      <div
        class="diff-line"
        class:added={item.type === 'added'}
        class:removed={item.type === 'removed'}
        class:unchanged={item.type === 'unchanged'}
      >
        <span class="line-number">{item.lineNumber}</span>
        <span class="diff-marker">
          {#if item.type === 'added'}
            +
          {:else if item.type === 'removed'}
            -
          {:else}
            <span style="opacity: 0;">-</span>
          {/if}
        </span>
        <code>{item.line}</code>
      </div>
    {/each}
  </div>
</div>

<style>
  .diff-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .diff-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .diff-title {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--text);
    font-weight: 500;
  }

  .close-btn {
    width: 32px;
    height: 32px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .close-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .diff-content {
    flex: 1;
    overflow-y: auto;
    font-family: var(--ff-mono);
    font-size: 12px;
    line-height: 1.5;
  }

  .diff-line {
    display: flex;
    align-items: flex-start;
    border-bottom: 1px solid var(--hairline);
  }

  .diff-line.added {
    background: color-mix(in srgb, var(--green) 8%, transparent);
  }

  .diff-line.added:hover {
    background: color-mix(in srgb, var(--green) 12%, transparent);
  }

  .diff-line.removed {
    background: color-mix(in srgb, var(--danger) 8%, transparent);
  }

  .diff-line.removed:hover {
    background: color-mix(in srgb, var(--danger) 12%, transparent);
  }

  .diff-line.unchanged {
    background: transparent;
  }

  .diff-line.unchanged:hover {
    background: var(--surface);
  }

  .line-number {
    display: inline-block;
    width: 50px;
    padding: 4px 12px;
    text-align: right;
    color: var(--muted);
    background: var(--surface);
    border-right: 1px solid var(--hairline);
    flex-shrink: 0;
    user-select: none;
  }

  .diff-marker {
    display: inline-block;
    width: 30px;
    padding: 4px 12px;
    text-align: center;
    color: var(--text-2);
    flex-shrink: 0;
    user-select: none;
  }

  .diff-line.added .diff-marker {
    color: var(--green);
  }

  .diff-line.removed .diff-marker {
    color: var(--danger);
  }

  code {
    flex: 1;
    padding: 4px 12px;
    overflow-x: auto;
    white-space: pre;
    word-break: break-all;
  }
</style>
