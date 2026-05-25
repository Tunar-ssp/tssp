<script lang="ts">
  interface $$Props {
    class?: string;
  }

  let { class: className, children, ...rest }: $$Props = $props();
</script>

<kbd class="kbd {className || ''}" {...rest}>
  {@render children?.()}
</kbd>

<style>
  .kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 2px 6px;
    min-width: 20px;
    height: 20px;
    font-family: var(--ff-mono);
    font-size: var(--fs-11);
    font-weight: 500;
    line-height: 1;
    background: var(--surface-2);
    color: var(--text-2);
    border: 1px solid var(--border);
    border-radius: var(--r-1);
    box-shadow: inset 0 -1px 0 rgba(0, 0, 0, 0.2);
  }
</style>
