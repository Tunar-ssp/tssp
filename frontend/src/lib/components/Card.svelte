<script lang="ts">
  export let accent: string | null = null;
  export let head: any = null;
  export let foot: any = null;
  export let pad: number = 16;
  export let style: string = '';
</script>

<div class="card" {style}>
  {#if accent}<div class="accent" style="background: {accent}" />{/if}
  {#if head}<div class="head">{head}</div>{/if}
  <div class="body" style="padding: {pad}px"><slot /></div>
  {#if foot}<div class="foot">{foot}</div>{/if}
</div>

<style>
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-3);
    position: relative;
    overflow: hidden;
    box-shadow: var(--shadow-card);
  }

  .accent {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
  }

  .head {
    padding: 12px 16px;
    border-bottom: 1px solid var(--hairline);
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .body {
    overflow: hidden;
  }

  .foot {
    padding: 12px 16px;
    border-top: 1px solid var(--hairline);
    display: flex;
    align-items: center;
    gap: 12px;
  }
</style>
