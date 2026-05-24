<script lang="ts">
  import { bannerState } from "../stores/ui";
</script>

{#if $bannerState}
  <section class={`banner banner-${$bannerState.tone}`} aria-live="polite">
    <div class="banner-copy">
      <strong>{$bannerState.title}</strong>
      {#if $bannerState.detail}
        <span>{$bannerState.detail}</span>
      {/if}
    </div>
  </section>
{/if}
