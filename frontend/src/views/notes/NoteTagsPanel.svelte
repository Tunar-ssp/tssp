<script lang="ts">
  import * as Icons from 'lucide-svelte';

  interface $$Props {
    tags: string[];
    tagDraft: string;
    onTagDraftChange: (value: string) => void;
    onAddTag: () => void;
    onRemoveTag: (tag: string) => void;
  }

  let { tags, tagDraft, onTagDraftChange, onAddTag, onRemoveTag }: $$Props = $props();
</script>

<div class="tag-strip">
  <div class="tag-list" aria-label="Note tags">
    {#if tags?.length}
      {#each tags as tag}
        <button type="button" class="tag-chip" onclick={() => onRemoveTag(tag)} title="Remove tag">
          {tag}
          <Icons.X size={12} />
        </button>
      {/each}
    {:else}
      <span class="tag-empty">No tags yet</span>
    {/if}
  </div>

  <form
    class="tag-form"
    onsubmit={(event) => {
      event.preventDefault();
      onAddTag();
    }}
  >
    <input
      bind:value={tagDraft}
      placeholder="Add tag"
      aria-label="Add note tag"
    />
    <button type="submit">Add</button>
  </form>
</div>

<style>
  .tag-strip {
    padding: 16px 18px;
    border-radius: 20px;
    border: 1px solid var(--border);
    background: rgba(15, 17, 23, 0.92);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }

  .tag-chip {
    height: 34px;
    padding: 0 12px;
    border-radius: 999px;
    border: 1px solid rgba(110, 168, 255, 0.16);
    background: rgba(23, 29, 43, 0.9);
    color: var(--blue);
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .tag-empty {
    color: var(--muted);
    font-size: 14px;
  }

  .tag-form {
    display: flex;
    gap: 10px;
  }

  .tag-form input {
    flex: 1;
    height: 40px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: rgba(12, 14, 20, 0.98);
    color: var(--text);
    padding: 0 12px;
  }

  .tag-form button {
    height: 40px;
    padding: 0 14px;
    border-radius: 14px;
    border: 1px solid rgba(110, 168, 255, 0.22);
    background: rgba(110, 168, 255, 0.1);
    color: var(--blue);
    cursor: pointer;
  }
</style>
