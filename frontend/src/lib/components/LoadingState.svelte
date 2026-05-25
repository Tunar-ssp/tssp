<script lang="ts">
  interface $$Props {
    message?: string;
    size?: 'sm' | 'md' | 'lg';
  }

  let {
    message = 'Loading...',
    size = 'md',
  }: $$Props = $props();

  const sizeMap = {
    sm: 24,
    md: 40,
    lg: 60,
  };
</script>

<div class="loading-state">
  <div class="spinner" style="--size: {sizeMap[size]}px;">
    <div class="spinner-ring"></div>
  </div>
  <p class="loading-message">{message}</p>
</div>

<style>
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    padding: var(--s-8);
    color: var(--muted);
  }

  .spinner {
    --size: 40px;
    position: relative;
    width: var(--size);
    height: var(--size);
  }

  .spinner-ring {
    position: absolute;
    width: 100%;
    height: 100%;
    border: 3px solid transparent;
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    box-shadow: inset 0 0 0 2px rgba(110, 168, 255, 0.1);
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .loading-message {
    margin: 0;
    font-size: var(--fs-12);
    color: var(--muted);
  }
</style>
