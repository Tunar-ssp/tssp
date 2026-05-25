<script lang="ts">
  export let value: number = 0;
  export let tone: 'blue' | 'green' | 'pink' | 'orange' | 'violet' | 'warn' = 'blue';
  export let height: number = 6;
  export let segments: any[] | null = null;

  const colors: Record<string, string> = {
    blue: 'var(--blue)',
    green: 'var(--green)',
    pink: 'var(--pink)',
    orange: 'var(--orange)',
    violet: 'var(--violet)',
    warn: 'var(--warning)',
  };

  const c = colors[tone];
</script>

{#if segments}
  <div class="bar-segmented" style="height: {height}px">
    {#each segments as s, i}
      <div class="segment" style="width: {s.pct}%; background: {s.color || c}; border-right: {i < segments.length - 1 ? '1px solid var(--bg)' : 'none'}"></div>
    {/each}
  </div>
{:else}
  <div class="bar-container" style="height: {height}px">
    <div class="bar-fill" style="width: {value}%; background: {c}; border-radius: {height / 2}px"></div>
  </div>
{/if}

<style>
  .bar-container {
    width: 100%;
    border-radius: 8px;
    background: var(--surface-3);
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    transition: width 0.3s ease;
  }

  .bar-segmented {
    width: 100%;
    display: flex;
    border-radius: 4px;
    overflow: hidden;
    background: var(--surface-3);
  }

  .segment {
    height: 100%;
  }
</style>
