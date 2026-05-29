<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import * as Icons from 'lucide-svelte';
  import { get } from 'svelte/store';
  import {
    notes,
    activeNote,
    loadNotes,
    setActiveNote,
    updateActiveNote,
    createNewNote,
    deleteNote,
    duplicateNote,
    toggleNotePin,
    replaceActiveNoteTags,
    moveNoteToParent,
    setNoteIcon,
    saveNote,
    isSaving,
  } from '$lib/stores/notes';
  import { success, error } from '$lib/stores/notifications';
  import { consumeSelectionIntent } from '$lib/stores/ui';
  import { registerKeyboardShortcuts } from '$lib/utils';
  import { buildNoteTree, collectSubtreeIds, ancestorIds, type NoteTreeNode } from '$lib/notes/tree';
  import type { Note } from '$lib/api';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import NotesTreeItem from './components/NotesTreeItem.svelte';
  import NotesEditor from './NotesEditor.svelte';
  import NoteTagsPanel from './NoteTagsPanel.svelte';

  // ---- sidebar layout state (persisted) ----
  const readLS = (key: string, fallback: string) =>
    typeof localStorage !== 'undefined' ? localStorage.getItem(key) ?? fallback : fallback;

  let collapsed = $state(readLS('notes-sidebar-collapsed', 'false') === 'true');
  let sidebarWidth = $state(Number(readLS('notes-sidebar-width', '280')));
  let expanded = $state<Set<string>>(
    new Set(JSON.parse(readLS('notes-expanded', '[]')) as string[]),
  );

  let searchQuery = $state('');
  let isLoading = $state(true);
  let titleDraft = $state('');
  let bodyDraft = $state('');
  let tagDraft = $state('');
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let lastActiveId = $state<string | null>(null);
  let contextMenu = $state({ visible: false, x: 0, y: 0, note: null as Note | null });
  let sidebarEl: HTMLElement | undefined;

  onMount(async () => {
    await loadNotes();
    const intent = consumeSelectionIntent();
    if (intent?.kind === 'note') {
      openNote(intent.id);
    }
    isLoading = false;
  });

  onDestroy(() => {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
      if (lastActiveId) void saveNote(lastActiveId, { title: titleDraft, body: bodyDraft });
    }
  });

  // persist layout
  $effect(() => {
    if (typeof localStorage === 'undefined') return;
    localStorage.setItem('notes-sidebar-collapsed', String(collapsed));
    localStorage.setItem('notes-sidebar-width', String(sidebarWidth));
  });
  $effect(() => {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('notes-expanded', JSON.stringify([...expanded]));
    }
  });

  // keyboard shortcuts
  $effect(() => {
    if (typeof window === 'undefined') return;
    return registerKeyboardShortcuts(
      [
        { key: 'b', ctrl: true, handler: () => (collapsed = !collapsed) },
        { key: 'n', ctrl: true, handler: (e) => { e.preventDefault(); void handleCreate(null); } },
      ],
      window,
    );
  });

  // sync drafts when active note changes; flush pending save for the old note
  $effect(() => {
    const current = $activeNote;
    if (current) {
      if (current.id !== lastActiveId) {
        if (saveTimer && lastActiveId) {
          clearTimeout(saveTimer);
          saveTimer = null;
          void saveNote(lastActiveId, { title: titleDraft, body: bodyDraft });
        }
        titleDraft = current.title;
        bodyDraft = current.body;
        tagDraft = '';
        lastActiveId = current.id;
        // reveal ancestors so the open page is visible in the tree
        for (const id of ancestorIds(get(notes), current.id)) expanded.add(id);
        expanded = new Set(expanded);
      }
    } else if (lastActiveId) {
      if (saveTimer) {
        clearTimeout(saveTimer);
        saveTimer = null;
        void saveNote(lastActiveId, { title: titleDraft, body: bodyDraft });
      }
      titleDraft = '';
      bodyDraft = '';
      lastActiveId = null;
    }
  });

  let tree = $derived(buildNoteTree($notes));
  let searchResults = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return [] as Note[];
    return $notes
      .filter(
        (n) =>
          n.title.toLowerCase().includes(q) ||
          n.body.toLowerCase().includes(q) ||
          (n.tags || []).some((t) => t.toLowerCase().includes(q)),
      )
      .slice(0, 50);
  });

  let breadcrumb = $derived.by(() => {
    if (!$activeNote) return [] as Note[];
    const byId = new Map($notes.map((n) => [n.id, n] as const));
    return ancestorIds($notes, $activeNote.id)
      .map((id) => byId.get(id))
      .filter((n): n is Note => !!n)
      .reverse();
  });

  function openNote(id: string) {
    setActiveNote(id);
  }
  function toggleExpand(id: string) {
    if (expanded.has(id)) expanded.delete(id);
    else expanded.add(id);
    expanded = new Set(expanded);
  }
  const isExpanded = (id: string) => expanded.has(id);

  function canDrop(dragId: string, targetId: string): boolean {
    if (dragId === targetId) return false;
    return !collectSubtreeIds($notes, dragId).has(targetId);
  }

  async function handleCreate(parentId: string | null) {
    try {
      const note = await createNewNote(parentId);
      if (parentId) {
        expanded.add(parentId);
        expanded = new Set(expanded);
      }
      success('Page created');
      return note;
    } catch (err) {
      error('Create failed', err instanceof Error ? err.message : 'Could not create page');
    }
  }

  async function handleMove(dragId: string, targetId: string | null) {
    try {
      await moveNoteToParent(dragId, targetId);
    } catch (err) {
      error('Move failed', err instanceof Error ? err.message : 'Could not move page');
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Delete this page? Child pages move up to its parent.')) return;
    try {
      await deleteNote(id);
      success('Page deleted');
    } catch (err) {
      error('Delete failed', err instanceof Error ? err.message : 'Could not delete page');
    }
  }

  async function handleDuplicate(id: string) {
    try {
      await duplicateNote(id);
      success('Page duplicated');
    } catch (err) {
      error('Duplicate failed', err instanceof Error ? err.message : 'Could not duplicate');
    }
  }

  async function handlePin(note: Note) {
    try {
      await toggleNotePin(note.id, !!note.pinned_at);
    } catch (err) {
      error('Pin failed', err instanceof Error ? err.message : 'Could not update pin');
    }
  }

  async function handleSetIcon(icon: string | null) {
    if (!$activeNote) return;
    try {
      await setNoteIcon($activeNote.id, icon);
    } catch (err) {
      error('Icon failed', err instanceof Error ? err.message : 'Could not set icon');
    }
  }

  // ---- autosave ----
  function scheduleSave() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      saveTimer = null;
      if (!$activeNote) return;
      void updateActiveNote({ title: titleDraft, body: bodyDraft }).catch((err) =>
        error('Save failed', err instanceof Error ? err.message : 'Could not save'),
      );
    }, 1000);
  }

  function onTitleChange(title: string) {
    titleDraft = title;
    // optimistic: keep the tree label in sync while typing
    if ($activeNote) {
      notes.update((list) =>
        list.map((n) => (n.id === $activeNote!.id ? { ...n, title } : n)),
      );
    }
    scheduleSave();
  }
  function onBodyChange(body: string) {
    bodyDraft = body;
    scheduleSave();
  }

  // ---- tags ----
  async function addTag() {
    if (!$activeNote || !tagDraft.trim()) return;
    const next = Array.from(new Set([...($activeNote.tags || []), tagDraft.trim()]));
    try {
      await replaceActiveNoteTags(next);
      tagDraft = '';
    } catch (err) {
      error('Tag failed', err instanceof Error ? err.message : 'Could not update tags');
    }
  }
  async function removeTag(tag: string) {
    if (!$activeNote) return;
    try {
      await replaceActiveNoteTags(($activeNote.tags || []).filter((t) => t !== tag));
    } catch (err) {
      error('Tag failed', err instanceof Error ? err.message : 'Could not update tags');
    }
  }

  // ---- context menu ----
  function openContext(event: MouseEvent, node: NoteTreeNode) {
    event.preventDefault();
    event.stopPropagation();
    contextMenu = { visible: true, x: event.clientX, y: event.clientY, note: node.note };
  }
  function openTopbarMenu(event: MouseEvent) {
    if (!$activeNote) return;
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    contextMenu = { visible: true, x: rect.right - 200, y: rect.bottom + 4, note: $activeNote };
  }
  let contextItems = $derived.by(() => {
    const n = contextMenu.note;
    if (!n) return [];
    return [
      { label: 'Add page inside', icon: Icons.Plus, action: () => void handleCreate(n.id) },
      { label: n.pinned_at ? 'Unpin' : 'Pin', icon: Icons.Pin, action: () => void handlePin(n) },
      { label: 'Duplicate', icon: Icons.Copy, action: () => void handleDuplicate(n.id) },
      ...(n.parent_id
        ? [{ label: 'Move to top level', icon: Icons.CornerUpLeft, action: () => void handleMove(n.id, null) }]
        : []),
      { label: 'Delete', icon: Icons.Trash2, action: () => void handleDelete(n.id), danger: true },
    ];
  });

  // ---- resize ----
  function startResize(e: PointerEvent) {
    e.preventDefault();
    const startLeft = sidebarEl?.getBoundingClientRect().left ?? 0;
    const move = (ev: PointerEvent) => {
      sidebarWidth = Math.min(460, Math.max(220, ev.clientX - startLeft));
    };
    const up = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  }

  // root drop zone (drop here to move a page to the top level)
  let rootOver = $state(false);
</script>

<div class="notes-app" class:collapsed>
  {#if !collapsed}
    <aside class="sidebar" style="width: {sidebarWidth}px" bind:this={sidebarEl}>
      <header class="side-head">
        <div class="brand"><Icons.NotebookPen size={16} /> <span>Notes</span></div>
        <button class="icon-btn" title="Collapse (Ctrl+B)" onclick={() => (collapsed = true)}>
          <Icons.PanelLeftClose size={16} />
        </button>
      </header>

      <div class="search-row">
        <Icons.Search size={14} />
        <input placeholder="Search pages…" bind:value={searchQuery} />
        {#if searchQuery}
          <button class="clear" onclick={() => (searchQuery = '')} aria-label="Clear"><Icons.X size={13} /></button>
        {/if}
      </div>

      <button class="new-page" onclick={() => void handleCreate(null)}>
        <Icons.Plus size={15} /> New page
      </button>

      <div
        class="tree-scroll"
        class:root-over={rootOver}
        role="tree"
        tabindex="-1"
        ondragover={(e) => { e.preventDefault(); rootOver = true; }}
        ondragleave={() => (rootOver = false)}
        ondrop={(e) => {
          rootOver = false;
          const id = e.dataTransfer?.getData('text/plain');
          if (id) { e.preventDefault(); void handleMove(id, null); }
        }}
      >
        {#if searchQuery.trim()}
          <div class="section-label">Results</div>
          {#each searchResults as note (note.id)}
            <button class="flat-row" class:active={$activeNote?.id === note.id} onclick={() => openNote(note.id)}>
              <span class="flat-icon">{#if note.icon}{note.icon}{:else}<Icons.FileText size={14} />{/if}</span>
              <span class="flat-title">{note.title || 'Untitled'}</span>
            </button>
          {:else}
            <div class="empty-hint">No pages match “{searchQuery}”.</div>
          {/each}
        {:else}
          <div class="section-label">Private</div>
          {#each tree as node (node.note.id)}
            <NotesTreeItem
              {node}
              activeId={$activeNote?.id ?? null}
              {isExpanded}
              onToggle={toggleExpand}
              onSelect={openNote}
              onCreateChild={(id) => void handleCreate(id)}
              onContext={openContext}
              onMoveInto={(dragId, targetId) => void handleMove(dragId, targetId)}
              {canDrop}
            />
          {:else}
            <div class="empty-hint">No pages yet. Create your first one.</div>
          {/each}
        {/if}
      </div>

      <button
        class="resize-handle"
        aria-label="Resize sidebar"
        onpointerdown={startResize}
      ></button>
    </aside>
  {:else}
    <button class="reveal" title="Open sidebar (Ctrl+B)" onclick={() => (collapsed = false)}>
      <Icons.PanelLeftOpen size={16} />
    </button>
  {/if}

  <main class="stage">
    {#if $activeNote}
      <header class="topbar">
        <nav class="crumbs">
          {#each breadcrumb as crumb (crumb.id)}
            <button class="crumb" onclick={() => openNote(crumb.id)}>
              {#if crumb.icon}<span class="ci">{crumb.icon}</span>{/if}{crumb.title || 'Untitled'}
            </button>
            <span class="crumb-sep">/</span>
          {/each}
          <span class="crumb current">
            {#if $activeNote.icon}<span class="ci">{$activeNote.icon}</span>{/if}{$activeNote.title || 'Untitled'}
          </span>
        </nav>
        <div class="topbar-right">
          {#if $isSaving}
            <span class="save-state"><Icons.Loader2 size={13} class="spin" /> Saving</span>
          {:else}
            <span class="save-state saved"><Icons.Check size={13} /> Saved</span>
          {/if}
          <button class="icon-btn" title="More" onclick={openTopbarMenu}>
            <Icons.MoreHorizontal size={18} />
          </button>
        </div>
      </header>

      <div class="page-scroll">
        <article class="page">
          <NotesEditor
            note={$activeNote}
            {titleDraft}
            {bodyDraft}
            isSaving={$isSaving}
            {onTitleChange}
            {onBodyChange}
            onIconChange={handleSetIcon}
          />
          <NoteTagsPanel
            tags={$activeNote.tags}
            {tagDraft}
            onTagDraftChange={(v) => (tagDraft = v)}
            onAddTag={addTag}
            onRemoveTag={removeTag}
          />
        </article>
      </div>
    {:else}
      <div class="stage-empty">
        <Icons.NotebookPen size={40} />
        <h2>Your notes</h2>
        <p>Select a page from the sidebar, or create a new one.</p>
        <button class="primary" onclick={() => void handleCreate(null)}>
          <Icons.Plus size={16} /> New page
        </button>
      </div>
    {/if}
  </main>
</div>

<ContextMenu
  bind:visible={contextMenu.visible}
  x={contextMenu.x}
  y={contextMenu.y}
  items={contextItems}
  onClose={() => (contextMenu.visible = false)}
/>

<style>
  .notes-app {
    flex: 1;
    height: 100vh;
    display: flex;
    min-width: 0;
    background: var(--bg, #0b0d12);
    color: var(--text);
  }

  .sidebar {
    position: relative;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--surface-1, #0e1117);
    border-right: 1px solid var(--border);
    min-width: 0;
  }

  .side-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 12px 8px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    font-size: 14px;
    color: var(--text);
  }
  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border: none;
    background: transparent;
    color: var(--muted);
    border-radius: 7px;
    cursor: pointer;
  }
  .icon-btn:hover {
    background: var(--surface-2, rgba(255, 255, 255, 0.06));
    color: var(--text);
  }

  .search-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 10px 8px;
    padding: 6px 10px;
    background: var(--surface-2, rgba(255, 255, 255, 0.04));
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--muted);
  }
  .search-row input {
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    outline: none;
  }
  .search-row .clear {
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    display: flex;
  }

  .new-page {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 10px 6px;
    padding: 8px 10px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text-2, #b8c0cc);
    font-size: 13px;
    cursor: pointer;
  }
  .new-page:hover {
    background: var(--surface-2, rgba(255, 255, 255, 0.06));
    color: var(--text);
  }

  .tree-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 4px 8px 16px;
    scrollbar-width: thin;
  }
  .tree-scroll.root-over {
    box-shadow: inset 0 0 0 2px rgba(110, 168, 254, 0.3);
    border-radius: 8px;
  }
  .section-label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
    padding: 8px 6px 4px;
  }
  .empty-hint {
    color: var(--muted);
    font-size: 12.5px;
    padding: 10px 6px;
  }

  .flat-row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: 28px;
    padding: 0 6px;
    border: none;
    background: transparent;
    border-radius: 6px;
    cursor: pointer;
    color: var(--text-2, #b8c0cc);
    text-align: left;
  }
  .flat-row:hover { background: var(--surface-2, rgba(255,255,255,0.05)); color: var(--text); }
  .flat-row.active { background: var(--surface-2, rgba(110,168,254,0.16)); color: var(--text); }
  .flat-icon { display: inline-flex; width: 18px; justify-content: center; color: var(--muted); }
  .flat-title { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 13.5px; }

  .resize-handle {
    position: absolute;
    top: 0;
    right: -3px;
    width: 6px;
    height: 100%;
    border: none;
    background: transparent;
    cursor: col-resize;
    padding: 0;
  }
  .resize-handle:hover {
    background: linear-gradient(to right, transparent, var(--accent, #6ea8fe), transparent);
    opacity: 0.5;
  }

  .reveal {
    position: absolute;
    top: 14px;
    left: 12px;
    z-index: 50;
    width: 32px;
    height: 32px;
    border: 1px solid var(--border);
    background: var(--surface, #14181f);
    color: var(--text-2);
    border-radius: 8px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .stage {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg, #0b0d12);
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 48px;
    padding: 0 16px 0 56px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .crumbs {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
    overflow: hidden;
    font-size: 13px;
  }
  .crumb {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    padding: 3px 6px;
    border-radius: 6px;
    white-space: nowrap;
  }
  .crumb:hover { background: var(--surface-2, rgba(255,255,255,0.06)); color: var(--text); }
  .crumb.current { color: var(--text); font-weight: 500; }
  .crumb .ci, .crumb.current .ci { font-size: 14px; }
  .crumb-sep { color: var(--muted); opacity: 0.6; }

  .topbar-right { display: flex; align-items: center; gap: 8px; }
  .save-state {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    color: var(--muted);
  }
  .save-state.saved { color: #57b97c; }
  :global(.save-state .spin) { animation: spin 0.9s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .page-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
  .page {
    max-width: 820px;
    margin: 0 auto;
    padding: 56px 64px 120px;
    display: flex;
    flex-direction: column;
    gap: 18px;
    min-height: 100%;
  }

  .stage-empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    color: var(--muted);
  }
  .stage-empty h2 { color: var(--text); margin: 6px 0 0; font-size: 20px; }
  .stage-empty p { margin: 0; font-size: 14px; }
  .primary {
    margin-top: 12px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 9px 16px;
    border: none;
    border-radius: 9px;
    background: var(--accent, #6ea8fe);
    color: #07131f;
    font-weight: 600;
    cursor: pointer;
  }

  @media (max-width: 820px) {
    .page { padding: 40px 22px 100px; }
  }
</style>
