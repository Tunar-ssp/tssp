# Next Steps: Frontend + Backend Integration

## Immediate Next Actions (Pick One)

### Option A: Wire Backend Routes ⚡ (30-45 min)
**Goal**: Connect frontend API calls to Rust backend
**Impact**: Unblocks all data operations

1. Open `crates/tsspd/src/main.rs`
2. Add route handlers:
   ```rust
   use axum::Router;
   mod workspace_files;
   mod safe_console;
   mod file_ops;
   mod admin_telemetry;

   let app = Router::new()
     .route("/api/v1/files/:id/move", ...) // from file_ops
     .route("/api/v1/admin/console/commands", ...) // from safe_console
     // ... add all routes from the 4 modules
   ```
3. Test each endpoint with `curl` or Postman
4. Verify frontend types match backend response shapes

### Option B: Implement Home View (20-30 min)
**Goal**: Create the launcher/home dashboard
**What to build**:
- Grid of 4 app cards (Drive, Notes, Workspace, Admin)
- Quick stats: files, notes, workspaces, storage
- Recent items from each app
- Quick action buttons (new file, new note, etc.)
- `frontend/src/views/home/HomeView.svelte`

### Option C: Add Command Palette (45-60 min)
**Goal**: Global Ctrl+K search across all content
**What to build**:
- Modal overlay component
- Real-time search across files/notes/workspaces
- Result previews with badges (File, Note, Workspace)
- Keyboard navigation (arrow keys, enter, escape)
- Action hints (⌘K, ESC, Enter)
- Hook into TopBar search input

### Option D: Database Migrations (30-45 min)
**Goal**: Create all necessary schema
**What to do**:
1. Create migrations:
   - `create_workspace_files_table`
   - `add_folder_path_to_files`
   - `add_soft_delete_to_files`
   - `create_audit_events_table`
2. Update Rust models to match
3. Run migrations and verify schema

## Validation Checklist Before Each Task

- [ ] Build succeeds: `npm run build` or `cargo build`
- [ ] Frontend dev server starts: `npm run dev`
- [ ] No TypeScript errors (strict mode)
- [ ] No compiler warnings
- [ ] Components render without console errors
- [ ] State updates propagate (check React/Svelte DevTools)

## Architecture Reminders

### Frontend Signal Flow
```
User Action → Component Event Handler
  → Store method (async if API call)
  → fetch() to /api/v1/...
  → Store update (writable or derived)
  → Component re-render (Svelte reactivity)
```

### Naming Conventions
- Components: PascalCase (FileGrid, NotesList)
- Stores: camelCase (activeNote, currentFolder)
- API methods: camelCase (listFiles, createNote)
- CSS classes: kebab-case (.file-grid, .note-list)

### State Lifecycle
1. **Writable**: Primary data (files[], notes[])
2. **Derived**: Computed from writeables (activeNote, filteredNotes)
3. **Component State**: Local to component only ($state in svelte 5)
4. **API**: Async operations trigger writeables

### Error Handling Strategy
- Try/catch in async store methods
- Set error store on failure
- Show banner from ui.ts: `showBanner('error message', 'error')`
- Always unblock loading state in finally

## Key Files to Understand

- `frontend/src/lib/api.ts` - All backend contracts (update here when adding new endpoints)
- `frontend/src/views/drive/DriveView.svelte` - 3-panel pattern (template for future views)
- `crates/tsspd/src/main.rs` - Backend router (where routes get wired)
- `crates/tssp-adapter-sqlite/src/migrations.rs` - Database schema

## Testing Flow

1. **Unit**: Test individual components in isolation
   ```bash
   # Create a test.svelte file with the component
   # Import and pass props
   # Check console for errors
   ```

2. **Integration**: Test component + store interaction
   ```typescript
   // In dev console:
   // import { activeNote } from '$lib/stores/notes'
   // activeNote.subscribe(n => console.log(n))
   // Call API methods and watch updates
   ```

3. **E2E**: Test full flows in browser
   - Create file → search for it → download
   - Write note → search → filter by tag
   - New workspace → edit → save → see changes

## Performance Notes

- Stores use Svelte `derived()` for automatic memoization
- Don't manually track computed state (that's what derived is for)
- 1000+ files: consider pagination or virtual scrolling
- Auto-save: 1-1.5s debounce (configured in stores)
- Search: real-time OK for <10K items

## Common Patterns

### Creating a Store
```typescript
import { writable, derived } from 'svelte/store';
import { api } from '../api';

export const items = writable<Item[]>([]);
export const selected = writable<string | null>(null);

export const active = derived(
  [items, selected],
  ([$items, $selected]) => $items.find(i => i.id === $selected)
);

export async function load() {
  const data = await api.listItems();
  items.set(data.items);
}
```

### Auto-save Pattern
```typescript
let timeout: number;
function scheduleAutoSave() {
  clearTimeout(timeout);
  timeout = window.setTimeout(() => {
    updateActiveItem(changes).catch(console.error);
  }, 1000); // 1 second debounce
}
```

### Subscription in Component
```typescript
import { onDestroy } from 'svelte';
const unsub = myStore.subscribe(value => {
  // react to changes
});
onDestroy(unsub);
```

## Quick Commands

```bash
# Frontend development
cd frontend
npm run dev          # Start dev server on :5173
npm run build        # Build to ../crates/tsspd/assets/web-v2

# Backend development
cargo build          # Compile Rust backend
cargo run            # Run tsspd on :8080
cargo test           # Run tests

# Database
sqlite3 tssp.db      # Inspect database
# Run migrations: handled by backend on startup
```

## Debugging Tips

- **Console logs in Svelte**: They're visible in browser DevTools like any JavaScript
- **Store subscriptions**: Use `console.log($storeName)` directly (with $ prefix)
- **API errors**: Check Network tab in DevTools, look at request/response bodies
- **CSS not applying**: Check that component imports tokens.css or uses --var names
- **State not updating**: Verify store update is called (not just store.value = x)

## Known Issues / Gotchas

1. **CSS Modules not imported**: Components must import `..lib/tokens.css` or define CSS-in-JS
2. **Lucide icon not found**: Check icon name is exported (e.g., Grid3 doesn't exist, use Grid2x2)
3. **TypeScript strict mode**: Required, no implicit any (e.g., Event → (e: Event))
4. **Svelte auto-subscribe $**: Only works at component root level, not in functions
5. **Relative imports**: Use $lib alias for cleaner imports, avoid ../../../../../../

## Success Metrics

After completing next steps, the app should:
- [ ] Load initial data from backend API
- [ ] Create/edit/delete items (files, notes, workspaces)
- [ ] Persist changes to database
- [ ] Show real-time system status
- [ ] Execute safe console commands
- [ ] Search across all content
- [ ] Handle errors gracefully
- [ ] Auto-save with visual feedback
- [ ] Sort/filter content correctly
- [ ] Responsive layout on mobile (bonus)

