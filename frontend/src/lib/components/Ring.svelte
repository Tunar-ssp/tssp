<script lang="ts">
  interface $$Props {
    value?: number;
    tone?: 'ok' | 'warn' | 'err' | 'info';
    label?: string;
    sub?: string;
    size?: number;
    strokeWidth?: number;
    class?: string;
  }

  let {
    value = 65,
    tone = 'info',
    label,
    sub,
    size = 120,
    strokeWidth = 8,
    class: className,
    ...rest
  }: $$Props = $props();

  const toneColors = {
    ok: 'var(--success)',
    warn: 'var(--warning)',
    err: 'var(--danger)',
    info: 'var(--blue)',
  };

  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference - (Math.min(value, 100) / 100) * circumference;
  const bgColor = toneColors[tone];
</script>

<div class="ring-wrapper {className || ''}" {...rest}>
  <svg {size} {size} class="ring-svg" viewBox={`0 0 ${size} ${size}`}>
    <circle
      cx={size / 2}
      cy={size / 2}
      {radius}
      fill="none"
      stroke="var(--surface-2)"
      stroke-width={strokeWidth}
    />
    <circle
      cx={size / 2}
      cy={size / 2}
      {radius}
      fill="none"
      stroke={bgColor}
      stroke-width={strokeWidth}
      stroke-dasharray={circumference}
      stroke-dashoffset={offset}
      stroke-linecap="round"
      class="ring-progress"
    />
  </svg>

  {#if label || sub}
    <div class="ring-text">
      {#if label}
        <div class="ring-label">{label}</div>
      {/if}
      {#if sub}
        <div class="ring-sub">{sub}</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .ring-wrapper {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    position: relative;
  }

  .ring-svg {
    transform: rotate(-90deg);
  }

  .ring-progress {
    transition: stroke-dashoffset var(--duration-normal) var(--ease-smooth);
  }

  .ring-text {
    position: absolute;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
  }

  .ring-label {
    font-size: var(--fs-16);
    font-weight: 500;
    color: var(--text);
  }

  .ring-sub {
    font-size: var(--fs-12);
    color: var(--muted);
    margin-top: 2px;
  }
</style>
