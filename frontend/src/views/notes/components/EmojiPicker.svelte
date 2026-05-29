<script lang="ts">
  interface Props {
    onPick: (emoji: string) => void;
    onRemove?: () => void;
    onClose: () => void;
  }
  let { onPick, onRemove, onClose }: Props = $props();

  const EMOJIS = [
    '📄','📝','📒','📕','📗','📘','📙','📚','📖','🗒️','📋','🗂️','📁','📂','🗃️','🏷️',
    '⭐','🔥','💡','✅','🎯','🚀','🧠','💼','📊','📈','📉','🗓️','⏰','🔔','📌','🔖',
    '❤️','💙','💚','💛','💜','🧡','🤍','🖤','🌟','✨','🎉','🎨','🛠️','⚙️','🔧','🧩',
    '🐛','🌱','🌍','💻','📱','🔐','🔑','📦','🧪','🧬','📡','🛰️','🏗️','🏠','🍎','☕',
  ];

  let query = $state('');
  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={handleKey} />
<button type="button" class="emoji-overlay" aria-label="Close" onclick={onClose}></button>
<div class="emoji-pop" role="dialog" aria-label="Choose an icon">
  <div class="emoji-head">
    <span>Choose an icon</span>
    {#if onRemove}
      <button type="button" class="remove-btn" onclick={() => { onRemove?.(); onClose(); }}>Remove</button>
    {/if}
  </div>
  <div class="emoji-grid">
    {#each EMOJIS as emoji (emoji)}
      <button type="button" class="emoji-cell" onclick={() => { onPick(emoji); onClose(); }}>{emoji}</button>
    {/each}
  </div>
</div>

<style>
  .emoji-overlay {
    position: fixed;
    inset: 0;
    background: transparent;
    border: none;
    z-index: 300;
    cursor: default;
  }
  .emoji-pop {
    position: absolute;
    z-index: 301;
    top: 100%;
    left: 0;
    margin-top: 6px;
    width: 296px;
    background: var(--surface, #14181f);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 14px 40px rgba(0, 0, 0, 0.5);
    padding: 10px;
  }
  .emoji-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 12px;
    color: var(--muted);
    margin-bottom: 8px;
  }
  .remove-btn {
    border: none;
    background: transparent;
    color: var(--accent, #6ea8fe);
    cursor: pointer;
    font-size: 12px;
  }
  .emoji-grid {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: 2px;
    max-height: 220px;
    overflow-y: auto;
  }
  .emoji-cell {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 32px;
    border: none;
    background: transparent;
    border-radius: 6px;
    cursor: pointer;
    font-size: 18px;
  }
  .emoji-cell:hover {
    background: var(--surface-2, rgba(255, 255, 255, 0.08));
  }
</style>
