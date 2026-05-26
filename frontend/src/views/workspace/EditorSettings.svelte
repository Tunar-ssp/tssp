<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    fontSize?: number;
    tabSize?: number;
    lineNumbers?: boolean;
    wordWrap?: boolean;
    minimap?: boolean;
    onFontSizeChange?: (size: number) => void;
    onTabSizeChange?: (size: number) => void;
    onLineNumbersChange?: (enabled: boolean) => void;
    onWordWrapChange?: (enabled: boolean) => void;
    onMinimapChange?: (enabled: boolean) => void;
  }

  let {
    fontSize = 13,
    tabSize = 2,
    lineNumbers = true,
    wordWrap = true,
    minimap = false,
    onFontSizeChange = () => {},
    onTabSizeChange = () => {},
    onLineNumbersChange = () => {},
    onWordWrapChange = () => {},
    onMinimapChange = () => {},
  }: $$Props = $props();
</script>

<div class="editor-settings">
  <div class="settings-header">
    <h3>Editor Settings</h3>
    <Icons.Settings size={16} />
  </div>

  <div class="settings-group">
    <div class="setting-item">
      <label for="font-size">Font Size</label>
      <div class="setting-control">
        <input
          id="font-size"
          type="number"
          min="10"
          max="24"
          value={fontSize}
          onchange={(e) => onFontSizeChange(parseInt(e.currentTarget.value))}
        />
        <span class="unit">px</span>
      </div>
    </div>

    <div class="setting-item">
      <label for="tab-size">Tab Size</label>
      <div class="setting-control">
        <input
          id="tab-size"
          type="number"
          min="1"
          max="8"
          value={tabSize}
          onchange={(e) => onTabSizeChange(parseInt(e.currentTarget.value))}
        />
        <span class="unit">spaces</span>
      </div>
    </div>
  </div>

  <div class="settings-group">
    <div class="setting-toggle">
      <label for="line-numbers">Line Numbers</label>
      <input
        id="line-numbers"
        type="checkbox"
        checked={lineNumbers}
        onchange={(e) => onLineNumbersChange(e.currentTarget.checked)}
      />
    </div>

    <div class="setting-toggle">
      <label for="word-wrap">Word Wrap</label>
      <input
        id="word-wrap"
        type="checkbox"
        checked={wordWrap}
        onchange={(e) => onWordWrapChange(e.currentTarget.checked)}
      />
    </div>

    <div class="setting-toggle">
      <label for="minimap">Minimap</label>
      <input
        id="minimap"
        type="checkbox"
        checked={minimap}
        onchange={(e) => onMinimapChange(e.currentTarget.checked)}
      />
    </div>
  </div>
</div>

<style>
  .editor-settings {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
    background: var(--surface);
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }

  .settings-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text);
  }

  .settings-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .setting-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .setting-item label {
    font-size: 13px;
    color: var(--text);
    font-weight: 500;
  }

  .setting-control {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .setting-control input {
    width: 50px;
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg);
    color: var(--text);
    font-size: 12px;
    text-align: center;
  }

  .setting-control input:focus {
    outline: none;
    border-color: var(--blue);
  }

  .unit {
    font-size: 11px;
    color: var(--muted);
    font-family: var(--ff-mono);
  }

  .setting-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px;
    border-radius: 6px;
    background: var(--bg);
    border: 1px solid var(--hairline);
  }

  .setting-toggle label {
    font-size: 13px;
    color: var(--text);
    font-weight: 500;
    cursor: pointer;
    user-select: none;
  }

  .setting-toggle input[type='checkbox'] {
    cursor: pointer;
    accent-color: var(--blue);
  }
</style>
