<script lang="ts">
  interface $$Props {
    value?: number;
    segments?: Array<{ value: number; color: string }>;
    tone?: 'ok' | 'warn' | 'err';
    class?: string;
  }

  let {
    value = 50,
    segments,
    tone = 'ok',
    class: className,
    ...rest
  } = $props<$$Props>();

  const toneColors = {
    ok: 'var(--success)',
    warn: 'var(--warning)',
    err: 'var(--danger)',
  };

  const bgColor = toneColors[tone];
  const clampedValue = Math.min(Math.max(value, 0), 100);
</script>

<div class="bar {className || ''}" {...rest}>
  {#if segments}
    <div class="bar-segments">
      {#each segments as segment (segment.color)}
        <div
          class="bar-segment"
          style="width: {segment.value}%; background: {segment.color};"
        />
      {/each}
    </div>
  {:else}
    <div class="bar-fill" style="width: {clampedValue}%; background: {bgColor};" />
  {/if}
</div>

<style>
  .bar {
    width: 100%;
    height: 6px;
    background: var(--surface-2);
    border-radius: var(--r-full);
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    border-radius: var(--r-full);
    transition: width var(--duration-normal) var(--ease-smooth);
  }

  .bar-segments {
    display: flex;
    height: 100%;
  }

  .bar-segment {
    height: 100%;
    transition: width var(--duration-normal) var(--ease-smooth);
  }
</style>
