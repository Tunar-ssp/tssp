<script lang="ts">
  import Outline from '$lib/components/Outline.svelte';

  interface $$Props {
    content: string;
    onSelectItem?: () => void;
  }

  let { content, onSelectItem }: $$Props = $props();
</script>

<div class="notes-outline">
  <Outline {content} onSelectItem={() => onSelectItem?.()} />
</div>

<style>
  .notes-outline {
    flex: 1;
    padding: 16px;
    overflow-y: auto;
  }
</style>
