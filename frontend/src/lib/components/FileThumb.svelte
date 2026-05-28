<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import FileIcon from './FileIcon.svelte';

  interface Props {
    id: string;
    name: string;
    mimeType?: string;
    iconSize?: number;
  }

  let { id, name, mimeType = '', iconSize = 34 }: Props = $props();

  const IMG = /\.(jpg|jpeg|png|gif|webp|avif|bmp|ico|svg)$/i;
  const VID = /\.(mp4|webm|mov|mkv|avi|flv|wmv|m4v)$/i;

  let isImage = $derived(IMG.test(name) || mimeType.startsWith('image/'));
  let isVideo = $derived(VID.test(name) || mimeType.startsWith('video/'));

  // Try the cheap /thumbnail endpoint first; if it 404s (stored mime isn't
  // image/*), fall back to the full /content; then to the type icon.
  let stage = $state<'thumb' | 'content' | 'failed'>('thumb');
  let src = $derived(
    stage === 'content'
      ? `/api/v1/files/${encodeURIComponent(id)}/content?disposition=inline`
      : `/api/v1/files/${encodeURIComponent(id)}/thumbnail`
  );

  function onError() {
    stage = stage === 'thumb' ? 'content' : 'failed';
  }
</script>

{#if isImage && stage !== 'failed'}
  <img class="thumb-img" {src} alt={name} loading="lazy" onerror={onError} />
{:else if isVideo}
  <div class="thumb-fallback video">
    <Icons.Play size={iconSize - 8} />
  </div>
{:else}
  <div class="thumb-fallback">
    <FileIcon {mimeType} {name} size={iconSize} />
  </div>
{/if}

<style>
  .thumb-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .thumb-fallback {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .thumb-fallback.video {
    color: var(--pink);
    background: color-mix(in srgb, var(--pink) 12%, transparent);
  }
</style>
