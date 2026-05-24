# TSSP Web v2 — Product Roadmap
**Stack:** Svelte 5 + Vite + TypeScript
**Last updated:** 2026-05-25
**Status:** Frontend scaffold started in `frontend/`

---

## 1. Why We Are Rebuilding

The current vanilla HTML/JS/CSS frontend is unmaintainable:

- One 700-line `index.html`, one 800-line `notes.js`, global `window.Tssp` namespace — no type safety, no encapsulation
- Every feature requires touching 4–6 files and coordinating by hand
- CSS has no co-location — a component's styles are scattered across 6 separate files
- No build step means no tree-shaking, no dead-code elimination, no optimization
- Impossible to refactor without breaking something invisible
- New developer (or future-you in 6 months) has no idea where anything is

**The result is a site that looks and behaves like AI-generated slop because the codebase forces you to work like that.**

---

## 2. Tech Stack Decision

### Chosen: **Svelte 5 + Vite + TypeScript**

| Concern | Choice | Reason |
|---|---|---|
| Framework | Svelte 5 | Compiles away — zero runtime on Orange Pi. No virtual DOM overhead. |
| Language | TypeScript | Type-safe API calls, component props, store state. Catch bugs at build time. |
| Build tool | Vite 6 | Instant HMR in dev. Optimized, tree-shaken output for prod. |
| Styling | CSS Modules + design tokens | Scoped per component, no global collision. Same token system, modernized. |
| State | Svelte stores + runes (`$state`) | Replaces `window.Tssp.*` namespace. Reactive, typed, tree-shakeable. |
| Routing | Client-side hash router (custom, minimal) | No SvelteKit needed — single-page app, no SSR required. |
| Icons | Lucide Svelte | MIT, tree-shakeable, consistent design language. |

### Why NOT React / Vue / SvelteKit
- **React:** Ships 150KB+ runtime. Heavier than needed for a local tool. JSX overhead for little gain.
- **Vue:** Fine, but heavier than Svelte and less aligned with "zero-overhead" goal for Pi.
- **SvelteKit:** We don't need SSR or file-based routing. Adds complexity. Plain Svelte + Vite is correct.
- **Astro / Remix:** Wrong paradigm for a real-time dashboard app.

### Build contract with Rust backend
- **Dev:** `npm run dev` on your workstation — proxies API calls to the Rust backend running locally
- **Migration build:** `npm run build` currently outputs to `crates/tsspd/assets/web-v2/` and uses `/app-v2/` as its base path so the new app can evolve without clobbering the live legacy dashboard
- **Cutover build:** once parity is reached, flip output to `crates/tsspd/assets/web/`
- **Orange Pi:** Never runs Node.js. It only serves the compiled static bundle.
- **Rust backend:** Unchanged API surface. The frontend just has a better client for it.

---

## 3. The Vision: Local Cloud OS

TSSP is not a "dashboard." It is a **Local Cloud OS** — four distinct apps in one shell, each with a focused, native-feeling UX:

| App | Inspiration | Core job |
|---|---|---|
| **Cloud Drive** | Google Drive + Finder | Store, browse, preview, share files |
| **Knowledge** | Notion | Block-based notes, linked pages, quick capture |
| **Workspace** | VS Code / Cursor | Multi-file code editor with syntax highlighting |
| **Operations** | AWS Console + htop | Admin, diagnostics, system control, safe terminal |

The shell (top bar + left rail) is shared. Switching apps completely changes the sidebar context and main area — not just a tab swap.

---

## 4. Design System

### Color tokens (unchanged from current, enforced by TypeScript constants)
```
--bg-root:      #000000
--bg-base:      #0B0B0D
--bg-elevated:  #111114
--bg-surface:   #16161A
--bg-card:      #1A1A1F
--bg-hover:     rgba(255,255,255,0.05)
--border:       #2C2E35
--border-strong:#3A3D47
--text:         #F1F1F3
--text-muted:   #8B8D97
--text-dim:     #5A5C66
--brand:        #2563EB
--blue:         #60A5FA
--cyan:         #22D3EE
--green:        #4ADE80
--yellow:       #FCD34D
--orange:       #FB923C
--red:          #F87171
--violet:       #A78BFA
```

### Typography
- UI: `"Inter", "Geist", system-ui, sans-serif`
- Code/mono: `"JetBrains Mono", "Fira Code", "Cascadia Code", monospace`
- Font sizes: 11px (label/micro), 12px (meta), 13px (body), 14px (default), 16px (h4), 20px (h3), 28px (h1)

### Component design rules
1. Every interactive element has a visible focus ring (brand blue)
2. No purple anywhere — that was the old `--accent` token, it is gone
3. Hover states are always `var(--bg-hover)` or `var(--brand-dim)`
4. Danger actions are always red, never orange or yellow
5. Success states are always green
6. Loading states use skeleton shimmer, never spinners for page-level loads

---

## 5. Project Structure

```
tssp/
├── frontend/                    ← NEW: Svelte + Vite project
│   ├── src/
│   │   ├── app.svelte           ← Root shell: top bar + left rail + router outlet
│   │   ├── main.ts              ← Entry point, mounts app
│   │   ├── lib/
│   │   │   ├── api.ts           ← Typed fetch client (all API calls)
│   │   │   ├── stores/
│   │   │   │   ├── auth.ts      ← User, role, session state
│   │   │   │   ├── drive.ts     ← Files, folders, selection, upload
│   │   │   │   ├── notes.ts     ← Notes list, active note, autosave
│   │   │   │   ├── workspace.ts ← Open tabs, active file, IDE state
│   │   │   │   └── ui.ts        ← Banner, command palette, modal state
│   │   │   ├── components/      ← Shared UI components
│   │   │   │   ├── Shell.svelte
│   │   │   │   ├── TopBar.svelte
│   │   │   │   ├── SideNav.svelte
│   │   │   │   ├── Banner.svelte
│   │   │   │   ├── CommandPalette.svelte
│   │   │   │   ├── PreviewDialog.svelte
│   │   │   │   ├── ContextMenu.svelte
│   │   │   │   └── ui/          ← Primitives: Button, Input, Badge, etc.
│   │   │   └── utils/
│   │   │       ├── format.ts    ← formatBytes, formatDate, formatRelativeTime
│   │   │       ├── mime.ts      ← fileKind, mimeToIcon
│   │   │       └── keyboard.ts  ← Global shortcut registry
│   │   └── views/
│   │       ├── drive/
│   │       │   ├── DriveView.svelte
│   │       │   ├── FolderTree.svelte
│   │       │   ├── FileGrid.svelte
│   │       │   ├── FileTable.svelte
│   │       │   ├── FileCard.svelte
│   │       │   ├── DetailsPanel.svelte
│   │       │   ├── UploadZone.svelte
│   │       │   └── lenses/      ← Images.svelte, Videos.svelte, Docs.svelte
│   │       ├── notes/
│   │       │   ├── NotesView.svelte
│   │       │   ├── NoteCard.svelte
│   │       │   ├── NoteEditor.svelte
│   │       │   ├── BlockToolbar.svelte
│   │       │   ├── NoteOutline.svelte
│   │       │   └── NotePreview.svelte
│   │       ├── workspace/
│   │       │   ├── WorkspaceView.svelte  ← IDE shell
│   │       │   ├── FileExplorer.svelte   ← Left panel
│   │       │   ├── TabBar.svelte
│   │       │   ├── CodeEditor.svelte     ← Textarea + highlighting
│   │       │   └── StatusBar.svelte
│   │       ├── operations/
│   │       │   ├── OperationsView.svelte
│   │       │   ├── OverviewTab.svelte
│   │       │   ├── AccessTab.svelte
│   │       │   ├── FilesTab.svelte
│   │       │   ├── MaintenanceTab.svelte
│   │       │   └── SafeConsole.svelte
│   │       ├── public/
│   │       │   └── PublicView.svelte
│   │       └── search/
│   │           └── SearchView.svelte
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── package.json
├── crates/
│   └── tsspd/
│       └── assets/
│           └── web/             ← Vite build output (git-ignored build artifacts)
└── ...
```

---

## 6. Phase-by-Phase Implementation Plan

### Phase 0: Scaffolding (Start here)
- [x] Create `frontend/` workspace with Vite, Svelte, and TypeScript config
- [x] Configure `vite.config.ts` with API proxy to Rust dev server
- [x] Set up `tsconfig.json` with strict mode
- [ ] Add Lucide Svelte
- [x] Port design tokens to `src/lib/tokens.css` (global)
- [x] Write typed API client `src/lib/api.ts` covering the first set of endpoints
- [x] Write `src/lib/stores/auth.ts` — probe auth on mount, store user/role
- [x] Implement shell: `App.svelte` with hash router, `TopBar.svelte`, `SideNav.svelte`
- [ ] Install dependencies and verify `npm run check`
- [ ] Deploy shell bundle and verify Rust can serve the new output path

### Migration policy
- Legacy `crates/tsspd/assets/web/` remains the production bundle until parity and smoke checks pass.
- New view work should land in `frontend/` first unless it is a required hotfix for the legacy dashboard.
- Legacy JS/CSS cleanup should only happen when it reduces migration friction or prevents a user-facing regression.

### Phase 1: Cloud Drive
**Goal:** Better than Google Drive for a local network

Frontend:
- [ ] `DriveView.svelte` — 3-column layout: FolderTree | FileGrid/Table | DetailsPanel
- [ ] `FolderTree.svelte` — collapsible tree, click to navigate, `+` button for new folder
- [ ] `FileGrid.svelte` — card grid with thumbnails for images, type icons otherwise
- [ ] `FileTable.svelte` — sortable table with checkbox selection
- [ ] `FileCard.svelte` — hover reveals actions: preview, share, pin, rename, delete
- [ ] `DetailsPanel.svelte` — metadata, tags, sharing status, preview thumbnail, download
- [ ] `UploadZone.svelte` — full-window drag-drop overlay, multi-file queue with progress bars
- [ ] `ContextMenu.svelte` — right-click on any file: rename, move, copy link, pin, delete
- [ ] Drive lenses (Images / Videos / Docs) as filter tabs inside DriveView, not separate nav items
- [ ] `PreviewDialog.svelte` — image lightbox, video player, text/code viewer, arrow key navigation

Backend needed:
- [ ] `PATCH /files/:id/move` — move file to folder
- [ ] `GET /files/:id/thumbnail` — generated thumbnail for images (if not already)
- [ ] `PATCH /files/:id/rename` already exists, verify it works

### Phase 2: Knowledge / Notes
**Goal:** Faster and cleaner than Notion for personal notes

Frontend:
- [ ] `NotesView.svelte` — left: notes list with search/filter/tags; right: editor (no separate route needed)
- [ ] `NoteCard.svelte` — title, preview line, tags, relative time, color accent on left border
- [ ] `NoteEditor.svelte` — full-height editor, title at top, body below
  - Markdown textarea with live preview in a side pane
  - `/` command menu (slash commands) for inserting blocks
  - `Ctrl+B`, `Ctrl+I`, `Ctrl+K` for inline formatting (wraps selection in markdown)
  - Outline panel (headings extracted from body)
  - Tag chips inline
  - Autosave debounce 1.5s, status indicator in header
- [ ] `BlockToolbar.svelte` — H1, Todo, List, Code, Table, Rule — minimal, no icons needed
- [ ] Quick-capture: `Ctrl+Shift+N` opens a floating mini-editor anywhere in the app, saves on close
- [ ] Note cover image (upload to set a cover, stored as note metadata)
- [ ] Notes folder/hierarchy (requires backend migration)

Backend needed:
- [ ] `POST /notes/:id/duplicate`
- [ ] `PATCH /notes/:id/archive`
- [ ] Migration: add `folder_path` column to `notes` table
- [ ] `GET /notes?folder=X` filter

### Phase 3: Workspace / IDE
**Goal:** Actually usable like VS Code, not a toy form

Frontend:
- [ ] `WorkspaceView.svelte` — full-height IDE shell (no page header, no padding)
- [ ] `FileExplorer.svelte` — list of workspace files as a file tree
  - Click to open in tab
  - Right-click: rename, delete
  - `+` button: create new file (name infers language from extension)
  - Folder grouping (e.g. `src/`, `tests/`)
- [ ] `TabBar.svelte` — open files as tabs, dirty dot, close button, Ctrl+W to close
- [ ] `CodeEditor.svelte`
  - `<textarea>` with monospace font, Tab key → 2 spaces, Ctrl+S saves
  - Syntax highlighting via **Highlight.js** (loaded lazily per language, ~8KB per lang)
  - Line numbers (CSS counter trick, no heavy library)
  - Ctrl+/ for line comments
  - Word wrap toggle
- [ ] `StatusBar.svelte` — language badge, cursor position, char count, save state (VSCode-style blue bar)
- [ ] Workspace folders: group files inside a workspace project

Backend needed:
- [ ] Migration: rename `workspaces` to workspace projects; add `workspace_files` table (id, workspace_id, name, language, body, folder_path, created_at, updated_at)
- [ ] `GET /workspaces/:id/files` — list files in a workspace
- [ ] `POST /workspaces/:id/files` — create file in workspace
- [ ] `PUT /workspaces/:id/files/:file_id` — update file body
- [ ] `DELETE /workspaces/:id/files/:file_id`
- [ ] `PATCH /workspaces/:id/files/:file_id/rename`

### Phase 4: Operations Console
**Goal:** Real system control, not a flat card page

Frontend:
- [ ] `OperationsView.svelte` — tab-based layout (same as current but rebuilt)
- [ ] `OverviewTab.svelte`
  - Live-updating metrics (polling every 5s via `/status`)
  - CPU / memory / disk charts (simple SVG bars, no library needed)
  - Recent uploads list
  - Active sessions count
  - Quick actions
- [ ] `AccessTab.svelte` — users table, create/delete/reset; active sessions; trusted devices
- [ ] `FilesTab.svelte` — admin file browser (search all files, bulk delete, force-make-private)
- [ ] `MaintenanceTab.svelte` — cleanup buttons, integrity check, vacuum DB
- [ ] `SafeConsole.svelte`
  - Pre-defined allowlisted commands (not arbitrary shell)
  - Commands: `df -h`, `free -m`, `uptime`, `top -bn1`, `ls /tmp`, integrity check, DB stats
  - Output rendered in a terminal-style pane
  - Command history in session
  - Each command defined server-side with an ID — client just sends `POST /admin/console/run {command: "disk_usage"}`
  - **No arbitrary input. No raw shell.**
- [ ] System metrics live chart (sparkline for CPU, optional)

Backend needed:
- [ ] `POST /admin/console/run` — runs a pre-approved command by enum key
- [ ] `GET /admin/console/commands` — returns list of allowed commands with descriptions
- [ ] `GET /status` improvements: add `active_session_count`, `upload_count_7d`, `top_tags`
- [ ] `POST /admin/integrity` — run file integrity check, return report

### Phase 5: Polish & Performance
- [ ] Service worker for offline shell (app loads even if Pi is momentarily unreachable)
- [ ] `CommandPalette.svelte` — Ctrl+K, searches files + notes + workspaces + actions
- [ ] Keyboard shortcut help overlay (`?` key)
- [ ] Mobile layout (responsive sidebar collapse, touch-friendly cards)
- [ ] `robots.txt` and security headers via Rust middleware
- [ ] Bundle analysis — keep total JS under 150KB gzipped
- [ ] Lazy-load view components (code splitting per view)

---

## 7. API Client Design

All API calls go through `src/lib/api.ts` — a typed thin wrapper:

```typescript
// src/lib/api.ts
const BASE = "/api/v1";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(BASE + path, {
    credentials: "same-origin",
    headers: { "Content-Type": "application/json", ...init?.headers },
    ...init,
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err?.error?.message || err?.error || `HTTP ${res.status}`);
  }
  return res.json();
}

export const api = {
  get:    <T>(path: string) => request<T>(path),
  post:   <T>(path: string, body: unknown) => request<T>(path, { method: "POST", body: JSON.stringify(body) }),
  put:    <T>(path: string, body: unknown) => request<T>(path, { method: "PUT", body: JSON.stringify(body) }),
  patch:  <T>(path: string, body: unknown) => request<T>(path, { method: "PATCH", body: JSON.stringify(body) }),
  delete: <T>(path: string) => request<T>(path, { method: "DELETE" }),
};
```

All response types are typed interfaces in `src/lib/types.ts`.

---

## 8. State Management

Use Svelte 5 runes (`$state`, `$derived`) for component-local state.
Use Svelte stores for shared cross-component state:

```typescript
// src/lib/stores/drive.ts
import { writable, derived } from "svelte/store";
import type { FileRecord } from "$lib/types";

export const files = writable<FileRecord[]>([]);
export const selectedIds = writable<Set<string>>(new Set());
export const currentFolder = writable<string>("");
export const isLoading = writable(false);

export const selectedCount = derived(selectedIds, ($s) => $s.size);
```

No `window.Tssp` global. No `data-*` click handlers. Components own their events.

---

## 9. Migration Strategy

**Do not migrate incrementally.** The current codebase is too entangled. Clean-room rewrite by view, keeping the Rust API unchanged.

Order of migration:
1. Scaffold + shell + auth (Phase 0)
2. Drive (most-used feature, biggest visual impact)
3. Notes
4. Workspace/IDE
5. Operations
6. Retire old `assets/web/` HTML/JS/CSS

During the rewrite, old frontend remains in place. Once new build output overwrites `assets/web/`, migration is complete.

**Rust backend is not touched** unless a feature requires a new endpoint (documented per phase above).

---

## 10. Build & Deploy Workflow

```bash
# Development (on your workstation)
cd frontend
npm run dev          # Vite dev server at :5173, proxies /api → :8080

# Production build
npm run build        # outputs to ../crates/tsspd/assets/web/

# Deploy to Orange Pi (example)
cargo build --release --bin tsspd
scp target/aarch64-*/release/tsspd pi@orange:/usr/local/bin/tsspd
# The binary embeds the web assets via include_dir or serves the directory
```

---

## 11. Security Constraints (Non-negotiable)

These carry over from the original roadmap and are enforced in every phase:

- No arbitrary shell execution exposed to the browser — ever
- Admin-only routes protected by RBAC middleware in Rust
- Public file links never expose private files
- All path parameters sanitized server-side
- Session and device controls are the Rust backend's responsibility
- If sandboxed execution is added later, it goes through a dedicated container/process — not a web textarea
