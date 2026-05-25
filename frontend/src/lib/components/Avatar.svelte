<script lang="ts">
  interface $$Props {
    initials?: string;
    url?: string;
    size?: 'sm' | 'md' | 'lg';
    name?: string;
    color?: string;
    class?: string;
  }

  let {
    initials = '?',
    url,
    size = 'md',
    name,
    color = 'var(--blue)',
    class: className,
  }: $$Props = $props();

  const sizeMap = {
    sm: 32,
    md: 40,
    lg: 56,
  };

  const fontSize = {
    sm: 12,
    md: 14,
    lg: 20,
  };
</script>

<div
  class="avatar avatar-{size} {className || ''}"
  title={name}
  style="--avatar-color: {color}; width: {sizeMap[size]}px; height: {sizeMap[size]}px; font-size: {fontSize[size]}px;"
>
  {#if url}
    <img src={url} alt={name || 'Avatar'} />
  {:else}
    <span class="initials">{initials}</span>
  {/if}
</div>

<style>
  .avatar {
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: var(--avatar-color);
    color: white;
    font-weight: 600;
    flex-shrink: 0;
    overflow: hidden;
  }

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .initials {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
  }
</style>
