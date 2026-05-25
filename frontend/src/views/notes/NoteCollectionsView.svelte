<script lang="ts">
  import * as Icons from 'lucide-svelte';
  import { success, error as showError } from '$lib/stores/notifications';
  import Btn from '$lib/components/Btn.svelte';
  import Card from '$lib/components/Card.svelte';
  import ColorPicker from '$lib/components/ColorPicker.svelte';

  interface Collection {
    id: string;
    name: string;
    color: string;
    noteCount: number;
    createdAt: number;
  }

  let collections = $state<Collection[]>([]);
  let isLoading = $state(false);
  let showCreateForm = $state(false);
  let newCollectionName = $state('');
  let newCollectionColor = $state('#6ea8ff');

  async function loadCollections() {
    isLoading = true;
    try {
      const response = await fetch('/api/notes/collections');
      if (!response.ok) throw new Error('Failed to load collections');
      collections = await response.json();
    } catch (e) {
      showError(e instanceof Error ? e.message : 'Failed to load collections');
    } finally {
      isLoading = false;
    }
  }

  async function createCollection() {
    if (!newCollectionName.trim()) {
      showError('Collection name required');
      return;
    }

    try {
      const response = await fetch('/api/notes/collections', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: newCollectionName,
          color: newCollectionColor,
        }),
      });

      if (!response.ok) throw new Error('Failed to create collection');
      const newCollection = await response.json();
      collections = [newCollection, ...collections];
      success(`Collection "${newCollectionName}" created`);
      newCollectionName = '';
      newCollectionColor = '#6ea8ff';
      showCreateForm = false;
    } catch (e) {
      showError(e instanceof Error ? e.message : 'Failed to create collection');
    }
  }

  async function deleteCollection(collectionId: string, name: string) {
    if (!confirm(`Delete collection "${name}"?`)) return;

    try {
      const response = await fetch(`/api/notes/collections/${collectionId}`, {
        method: 'DELETE',
      });
      if (!response.ok) throw new Error('Failed to delete collection');
      collections = collections.filter((c) => c.id !== collectionId);
      success('Collection deleted');
    } catch (e) {
      showError(e instanceof Error ? e.message : 'Failed to delete collection');
    }
  }

  $effect(() => {
    loadCollections();
  });
</script>

<div class="collections-view">
  <div class="view-header">
    <div>
      <h2>Collections</h2>
      <p class="subtitle">Organize your notes into collections</p>
    </div>
    <Btn kind="primary" on:click={() => (showCreateForm = !showCreateForm)}>
      <Icons.Plus size={14} />
      New Collection
    </Btn>
  </div>

  {#if showCreateForm}
    <div class="create-form">
      <Card>
        <div class="form-content">
          <div class="form-group">
            <label for="name">Collection Name</label>
            <input
              id="name"
              type="text"
              placeholder="e.g., Ideas, Projects, Reading"
              bind:value={newCollectionName}
            />
          </div>

          <div class="form-group">
            <label>Color</label>
            <ColorPicker
              color={newCollectionColor}
              onChange={(c) => (newCollectionColor = c)}
            />
          </div>

          <div class="form-actions">
            <Btn kind="primary" on:click={createCollection}>Create</Btn>
            <Btn kind="ghost" on:click={() => (showCreateForm = false)}>
              Cancel
            </Btn>
          </div>
        </div>
      </Card>
    </div>
  {/if}

  {#if isLoading}
    <div class="loading">
      <div class="spinner"></div>
      Loading collections...
    </div>
  {:else if collections.length === 0}
    <div class="empty">
      <Icons.Folder size={48} />
      <h3>No collections yet</h3>
      <p>Create a collection to organize your notes</p>
    </div>
  {:else}
    <div class="collections-grid">
      {#each collections as collection (collection.id)}
        <Card
          accent={collection.color}
          on:click={() => null}
          class="collection-card"
        >
          <div class="card-header">
            <div class="card-color" style="background: {collection.color}"></div>
            <h3>{collection.name}</h3>
          </div>
          <div class="card-body">
            <p class="note-count">{collection.noteCount} note{collection.noteCount !== 1
              ? 's'
              : ''}</p>
          </div>
          <div class="card-footer">
            <button
              class="delete-btn"
              on:click={() => deleteCollection(collection.id, collection.name)}
            >
              <Icons.Trash2 size={14} />
            </button>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>

<style>
  .collections-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .view-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s-6);
    border-bottom: 1px solid var(--border);
  }

  .view-header h2 {
    margin: 0;
    font-size: var(--fs-24);
    color: var(--text);
  }

  .subtitle {
    margin: var(--s-2) 0 0;
    font-size: var(--fs-13);
    color: var(--muted);
  }

  .create-form {
    padding: 0 var(--s-6) var(--s-4);
  }

  .form-content {
    display: flex;
    flex-direction: column;
    gap: var(--s-4);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .form-group label {
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text);
  }

  .form-group input {
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text);
    border-radius: var(--r-2);
    font-family: var(--ff-sans);
    font-size: var(--fs-13);
  }

  .form-actions {
    display: flex;
    gap: var(--s-3);
  }

  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    color: var(--muted);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--surface-3);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    color: var(--muted);
  }

  .empty h3 {
    margin: 0;
    color: var(--text-2);
  }

  .empty p {
    margin: 0;
    font-size: var(--fs-12);
  }

  .collections-grid {
    flex: 1;
    overflow-y: auto;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--s-4);
    padding: var(--s-6);
  }

  :global(.collection-card) {
    cursor: pointer;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  :global(.collection-card:hover) {
    transform: translateY(-2px);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.12);
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-bottom: var(--s-3);
  }

  .card-color {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .card-header h3 {
    margin: 0;
    font-size: var(--fs-14);
    font-weight: 600;
    color: var(--text);
  }

  .card-body {
    margin-bottom: var(--s-4);
  }

  .note-count {
    margin: 0;
    font-size: var(--fs-12);
    color: var(--muted);
  }

  .card-footer {
    display: flex;
    justify-content: flex-end;
  }

  .delete-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    border-radius: var(--r-1);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-quick) var(--ease-smooth);
  }

  .delete-btn:hover {
    background: rgba(255, 107, 107, 0.1);
    color: var(--danger);
  }
</style>
