<script lang="ts">
  export let value: number = 0;
  export let size: number = 64;
  export let tone: 'blue' | 'green' | 'pink' | 'orange' | 'violet' | 'warn' = 'blue';
  export let label: string = '';
  export let sub: string = '';

  const colors: Record<string, string> = {
    blue: 'var(--blue)',
    green: 'var(--green)',
    pink: 'var(--pink)',
    orange: 'var(--orange)',
    violet: 'var(--violet)',
    warn: 'var(--warning)',
  };

  const r = size / 2 - 4;
  const cir = 2 * Math.PI * r;
  const c = colors[tone];
</script>

<div class="ring-container">
  <svg {size} {size} style="transform: rotate(-90deg)">
    <circle cx={size/2} cy={size/2} {r} fill="none" stroke="var(--surface-3)" stroke-width="4"/>
    <circle cx={size/2} cy={size/2} {r} fill="none" stroke={c} stroke-width="4" stroke-linecap="round"
      stroke-dasharray={cir} stroke-dashoffset={cir - (cir * value) / 100}/>
  </svg>
  <div class="ring-label">
    <div class="label">{label}</div>
    {#if sub}<div class="sub">{sub}</div>{/if}
  </div>
</div>

<style>
  .ring-container {
    width: var(--s-10);
    height: var(--s-10);
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  svg {
    position: absolute;
  }

  .ring-label {
    position: absolute;
    text-align: center;
    pointer-events: none;
  }

  .label {
    font-size: var(--fs-16);
    font-weight: 600;
    color: var(--text);
  }

  .sub {
    font-size: 10px;
    color: var(--muted);
    margin-top: -2px;
  }
</style>
