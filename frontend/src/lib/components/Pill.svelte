<script lang="ts">
  export let tone: 'neutral' | 'green' | 'pink' | 'blue' | 'orange' | 'violet' | 'warn' | 'danger' =
    'neutral';

  const tones = {
    neutral: { bg: 'rgba(255,255,255,.05)', fg: 'var(--text-2)', bd: 'var(--border)' },
    green: { bg: 'rgba(91,227,154,.10)', fg: 'var(--green)', bd: 'rgba(91,227,154,.25)' },
    pink: { bg: 'rgba(255,95,162,.10)', fg: 'var(--pink)', bd: 'rgba(255,95,162,.25)' },
    blue: { bg: 'rgba(110,168,255,.10)', fg: 'var(--blue)', bd: 'rgba(110,168,255,.25)' },
    orange: { bg: 'rgba(255,138,61,.10)', fg: 'var(--orange)', bd: 'rgba(255,138,61,.25)' },
    violet: { bg: 'rgba(163,148,255,.10)', fg: 'var(--violet)', bd: 'rgba(163,148,255,.25)' },
    warn: { bg: 'rgba(251,191,36,.10)', fg: 'var(--warning)', bd: 'rgba(251,191,36,.25)' },
    danger: { bg: 'rgba(255,107,107,.10)', fg: 'var(--danger)', bd: 'rgba(255,107,107,.25)' },
  };

  const theme = tones[tone];
</script>

<span style:background={theme.bg} style:color={theme.fg} style:border-color={theme.bd} class="pill">
  <slot />
</span>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    font-weight: 500;
    padding: 2px 8px;
    border-radius: var(--r-full);
    border: 1px solid;
    line-height: 1.4;
  }
</style>
