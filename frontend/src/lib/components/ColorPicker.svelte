<script lang="ts">
  interface $$Props {
    color?: string;
    onChange?: (color: string) => void;
    colors?: string[];
    class?: string;
  }

  const defaultColors = [
    { name: 'gray', value: '#7c8190' },
    { name: 'red', value: '#ff6b6b' },
    { name: 'orange', value: '#ff8a3d' },
    { name: 'yellow', value: '#fbbf24' },
    { name: 'green', value: '#34d399' },
    { name: 'blue', value: '#6ea8ff' },
    { name: 'purple', value: '#a394ff' },
    { name: 'pink', value: '#ff5fa2' },
  ];

  let {
    color = 'gray',
    onChange,
    colors = defaultColors,
    class: className,
  }: $$Props = $props();

  function handleColorSelect(selectedColor: string) {
    if (onChange) {
      onChange(selectedColor);
    }
  }
</script>

<div class="color-picker {className || ''}">
  {#each colors as c (c.value)}
    <button
      class="color-option"
      class:selected={color === c.value}
      style="background: {c.value}"
      on:click={() => handleColorSelect(c.value)}
      title={c.name}
      aria-label={c.name}
    />
  {/each}
</div>

<style>
  .color-picker {
    display: flex;
    gap: var(--s-2);
  }

  .color-option {
    width: 32px;
    height: 32px;
    border-radius: var(--r-full);
    border: 2px solid transparent;
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
    flex-shrink: 0;
  }

  .color-option:hover {
    transform: scale(1.1);
    box-shadow: 0 0 12px rgba(0, 0, 0, 0.3);
  }

  .color-option.selected {
    border-color: var(--text);
    box-shadow: 0 0 0 3px rgba(0, 0, 0, 0.2);
  }
</style>
