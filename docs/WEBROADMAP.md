# TSSP Web Product Roadmap

> Local-first personal cloud. Self-hosted on Orange Pi. Fast, private, serious.

Implementation note:
- Product goals live in this file.
- The maintainable frontend architecture and migration execution live in [ROADMAPWEB.md](./ROADMAPWEB.md).
- New source now starts in `frontend/`, with cutover to the Rust-served bundle after parity.

---

## Product Vision

TSSP should feel like working inside a professional tool, not opening a demo page.  
The closest references: **Cursor** (clean dark IDE shell), **Linear** (tight spacing, purposeful hierarchy), **Google Drive** (structured file management), **Notion** (block-based writing).

Every surface must be usable, not just functional. Every empty state must explain what to do. Every interaction must feel immediate.

---

## Design System

### Color System (done → needs polish)

Current problem: pure black backgrounds + random purple = dead AI dashboard.  
Target: dark slate + professional blue accent.

| Token | Value | Role |
|---|---|---|
| `--bg-root` | `#0B0B0D` | Page background |
| `--bg-base` | `#111114` | App shell |
| `--bg-elevated` | `#18181C` | Panels, sidebar |
| `--bg-card` | `#1E1E23` | Cards, dialogs |
| `--bg-surface` | `#242429` | Inputs, hover targets |
| `--accent-primary` | `#2563EB` | Primary buttons, active states |
| `--accent-secondary` | `#7C3AED` | Secondary highlights, pinned |
| Border | `#2A2A30` | Subtle separators |

### Typography Scale

- Page title: `clamp(1.5rem, 2.5vw, 2rem)`, weight 700
- Section head: `1.1rem`, weight 600
- Body: `14px`, line-height 1.6
- Mono: JetBrains Mono / SF Mono
- Labels/meta: `12px`, muted color

### Spacing

- Page padding: `24px 28px`
- Card padding: `16px`
- Section gap: `24px`
- Grid gap: `12–16px`

### Component Standards

- **Buttons**: `btn-primary` = blue gradient; `btn-secondary` = ghost with border; `btn-danger` = red tint; `btn-text` = no background
- **Cards**: consistent 1px border, subtle background gradient, `border-radius: 10px`
- **Tables**: sticky header, alternating hover, no dead space
- **Empty states**: icon + headline + CTA, centered, minimum 220px height
- **Loading states**: skeleton shimmer, no spinner-only
- **Dialogs**: backdrop blur, max-width 640px, close on Escape + backdrop click

---

## Area 1: Cloud Drive

### Status: Partially built — UX problems remain

**What exists:**
- File list with cards/table toggle
- Folder tree navigation
- Upload with drag-drop
- File detail panel
- Preview dialog (image, video, audio, PDF, text/markdown)
- Bulk selection + delete
- Pin/unpin, visibility toggle
- Public link copy
- Sort + filter controls
- Tags

**What needs work:**

#### P0 — Core UX
- [ ] Folder create / rename / delete via UI (API exists, no button)
- [ ] Move file to folder (drag-and-drop or context menu)
- [ ] Rename file inline
- [ ] Better file cards: thumbnail for images, type icon for others, file name truncation, size + date meta
- [ ] Preview dialog: full-screen option, prev/next keyboard nav ✓ (done)
- [ ] Upload progress bar (per-file), cancel upload
- [ ] Upload queue display
- [ ] QR code for public links

#### P1 — Polish
- [ ] Folder breadcrumb as clickable path
- [ ] Right-click context menu (rename, move, delete, copy link, pin)
- [ ] File search within current folder
- [ ] Bulk move to folder
- [ ] Tag filter pills in file list header
- [ ] "Recently uploaded" section on drive home
- [ ] Drive activity (last modified, last viewed)

#### P2 — Future
- [ ] File version history (content-addressed storage already supports it)
- [ ] File comments / annotations
- [ ] Starred files separate from pinned
- [ ] Shared with me (if multi-user)
- [ ] Offline cache via Service Worker

### Images, Videos, Documents

Current problem: completely separate dead pages with no empty state context.

- [ ] Make images/videos/documents feel like filtered Drive views (same breadcrumb, same actions, same sorting)
- [ ] Image gallery: lazy-load, lightbox with caption, download button
- [ ] Video gallery: thumbnail from first frame (if practical), player dialog
- [ ] Documents: PDF viewer improvement, text file diff view

---

## Area 2: Notes

### Status: Textarea with markdown — NOT a block editor

**What exists:**
- Note cards with color, tags, pin
- Full-page editor (3-panel: outline, textarea, preview)
- Autosave
- Block toolbar (heading, checklist, bullets, code, table, callout buttons that insert markdown shortcuts)
- Tag chips
- Note color picker

**Target: Page-and-blocks UX like Notion Lite**

The key insight: blocks don't require reinventing a rich text editor. They require making the **editing experience feel structured** even if the underlying storage is markdown.

#### P0 — Block-Feels UX (achievable without CRDT or ProseMirror)
- [ ] Slash command menu: type `/` in textarea → floating menu appears with block types (Heading 1/2/3, Bullet list, Numbered list, Checklist, Code block, Callout, Divider, Table, Quote)
- [ ] Live block rendering: render the markdown body inline as styled blocks as you type (split-pane or toggle view, not permanent preview-only)
- [ ] Better block toolbar: icon-based, not text labels, grouped by category
- [ ] Checklist rendering: `- [ ]` renders as actual checkboxes in preview mode, clicking toggles the source
- [ ] Code block with language label
- [ ] Callout block rendering (styled quote with icon)
- [ ] Table rendering
- [ ] Full-width note editor option (no side panels)

#### P1 — Organization
- [ ] Note folders / sections (backend: add `folder` field to notes table)
- [ ] Drag to reorder notes on home page
- [ ] Duplicate note
- [ ] Note templates (empty + starter templates)
- [ ] Note trash / restore (soft-delete, `deleted_at` column)

#### P2 — Integration
- [ ] Embed file preview inside note body (`/file` command inserts `[filename](link)`)
- [ ] Note backlinks (show which notes reference current note)
- [ ] Export to PDF (print CSS + window.print())
- [ ] Import from Markdown file (upload + parse)

### Backend changes needed
- `folder_path: Option<String>` on notes table + migration
- `deleted_at: Option<DateTime>` for soft-delete
- `GET /api/v1/notes/folders` — list note folder names
- `GET /api/v1/notes?folder=x` — filter by folder

---

## Area 3: Workspace / IDE

### Status: Card list with modal editor — NOT an IDE

**What exists:**
- Workspace cards with name, language, line count, date
- Edit button opens full-page editor
- Editor has: syntax textarea, line numbers, find-in-file, auto-close brackets
- Tabs for open workspaces

**Target: Small web IDE**

#### P0 — Real IDE Feel
- [ ] **Workspace = project, not a single file**: each workspace holds multiple files in a virtual tree
  - Backend: add `workspace_files` table (id, workspace_id, path, content, created_at, updated_at)
  - API: `GET/POST/PUT/DELETE /api/v1/workspaces/{id}/files/{path}`
- [ ] File explorer sidebar (tree view) inside workspace view
- [ ] Create file / folder in workspace
- [ ] Rename / delete workspace files
- [ ] Open multiple files as tabs
- [ ] Language inferred from file extension (`.py` → python, `.rs` → rust, etc.)
- [ ] Syntax highlighting (integrate a lightweight highlighter — Highlight.js CDN or Prism.js CDN, lazy-loaded)
- [ ] File save state indicator (dot on tab = unsaved)
- [ ] Keyboard shortcut Ctrl+S to save

#### P1 — Editor Quality
- [ ] Line wrap toggle
- [ ] Font size control (+/-)
- [ ] Soft-tab size setting (2 / 4)
- [ ] Find + replace (not just find)
- [ ] Go to line (Ctrl+G)
- [ ] Markdown preview for .md files
- [ ] Diff view for file changes

#### P2 — Workspace Dashboard
- [ ] Workspace README (show first .md file as workspace description)
- [ ] Recent files list
- [ ] Workspace statistics (file count, total lines, languages used)
- [ ] Share workspace as read-only public view

### Backend changes needed
- `workspace_files` table + migration
- Full CRUD handlers for workspace file tree
- `GET /api/v1/workspaces/{id}` returns file tree metadata
- Content stored as UTF-8 text (no blob storage for workspace files)

---

## Area 4: Admin Control Center

### Status: One page with 5 tabs — not a control center

**What exists:**
- Overview: storage stats, system health bars, version
- Users & sessions: create user, user list, session management, device management
- Files: file table with delete
- Maintenance: cleanup temp/sessions buttons, integrity check
- Safe console: allowlisted commands with history

**Target: Real operations console**

#### P0 — Better Structure
- [ ] Admin sidebar nav (not just top tabs): Overview / Users / Storage / Sessions / Devices / Public Links / Notes / Workspaces / Maintenance / Console / Logs
- [ ] Each section loads independently (lazy fetch)
- [ ] Admin overview dashboard:
  - Health status (color-coded: healthy / warning / critical)
  - Storage ring chart (used / free)
  - User activity (last seen per user)
  - Recent uploads timeline
  - Quick action buttons
- [ ] Admin action confirmation dialogs (before destructive operations)

#### P1 — User Management
- [ ] Reset user access code (with copy button)
- [ ] Force-expire all sessions for a user
- [ ] User storage quota display
- [ ] Last seen / activity for each user
- [ ] Enable/disable user (soft block)

#### P2 — Storage Operations
- [ ] Folder tree in admin (same as user drive but full view)
- [ ] Orphan blob detection + delete
- [ ] Storage breakdown by user
- [ ] Export file manifest as CSV

#### P3 — Console Upgrade
- [ ] Command categories with icons
- [ ] Output syntax highlighting (JSON, plain text)
- [ ] Output copy button
- [ ] Run history persisted in localStorage
- [ ] Favorite commands
- [ ] Keyboard shortcut to run last command again

---

## Area 5: Sharing Center

### Status: Public links page is dead

**What exists:**
- Table of public files with copy link button
- Make private / delete buttons

**Target: Sharing dashboard**

#### P0
- [ ] Card layout (not table): each public file shows thumbnail/icon, filename, link, copy button, QR code button, revoke button
- [ ] QR code generation (inline, SVG-based, no external dependency)
- [ ] Share stats: number of times accessed (if tracked)
- [ ] "Recently shared" sorted view
- [ ] Bulk revoke

#### P1
- [ ] Expiring links (add `expires_at` to files table, checked in public handler)
- [ ] Password-protected links (hash in URL or cookie-based)
- [ ] Share via WhatsApp / Telegram deep link (just a pre-filled URL, no API needed)

---

## Area 6: Search / Command Palette

### Status: Mostly working, UX needs polish

**What exists:**
- Global search bar in top bar
- Ctrl+K opens command palette
- Palette shows: files, notes, workspaces, nav shortcuts, recent searches
- Dedicated search page with kind/tag/type/visibility/pinned filters
- Result count header, snippet display

**Target: Spotlight-level experience**

#### P0
- [ ] Search results grouped visually (Files section, Notes section, Workspaces section)
- [ ] Keyboard navigation (↑↓ to move, Enter to open, Escape to close) ← partially done
- [ ] Recent searches persisted in localStorage
- [ ] Show total count per group ("3 files, 1 note")
- [ ] Search result previews on hover (mini panel)
- [ ] Empty state: recent searches + suggested actions

#### P1
- [ ] Full-text search inside workspace files
- [ ] Search inside note body (not just title)
- [ ] Search by date range
- [ ] Search by file size (> 1MB)

---

## Area 7: Overview Dashboard

### Status: Metric cards only — no real dashboard

**What exists:**
- 8 metric cards (files, notes, storage, public, pinned, tags, uploaded, uptime)
- Health indicator

**Target: Real dashboard**

#### P0
- [ ] Activity feed: last 10 uploads / note edits / workspace changes
- [ ] Storage breakdown visual (ring chart or bar chart, pure CSS or Canvas)
- [ ] Quick access: recently modified files, pinned notes
- [ ] System status: uptime, RAM, CPU, disk — with color-coded thresholds
- [ ] Welcome card (first-time setup guidance)

#### P1
- [ ] Daily / weekly usage sparkline
- [ ] Top tags cloud
- [ ] Quick upload shortcut

---

## Technical Debt

### Frontend
- [ ] `views.js` should be split: `views/drive.js`, `views/images.js`, `views/videos.js`, `views/documents.js`, `views/overview.js`
- [ ] `notes.js` split into `notes/list.js`, `notes/editor.js`, `notes/blocks.js`
- [ ] CSS: `views.css` is 1600+ lines — split into `css/drive.css`, `css/notes.css`, `css/admin.css`, `css/editor.css`
- [ ] Remove dead event listener registrations in `app.js`
- [ ] Consistent error handling: all API calls should show banner on failure
- [ ] Loading skeleton components instead of text "Loading..."
- [ ] `state.js`: add derived state helpers (e.g., `T.selectedFileCount`, `T.isAdminUser`)

### Backend
- [ ] `PUT /api/v1/files/{id}/move` — move file to new folder
- [ ] `PUT /api/v1/files/{id}/rename` — rename file
- [ ] `POST /api/v1/folders` — create folder
- [ ] `DELETE /api/v1/folders/{path}` — delete empty folder
- [ ] `PUT /api/v1/folders/{path}/rename` — rename folder
- [ ] Folder delete cascades files (or requires empty)
- [ ] `GET /api/v1/admin/activity` — recent events log
- [ ] Rate limiting on public file serving
- [ ] Content-Security-Policy headers tightened
- [ ] `GET /api/v1/files?folder=x&recursive=true` — deep folder listing

### Performance
- [ ] Paginate file list (currently loads all at once)
- [ ] Paginate notes list (lazy load more)
- [ ] Image thumbnails generated server-side (or lazy-loaded full images)
- [ ] SQLite indexes verified on: `files.folder_path`, `files.mime_type`, `files.uploaded_at`, `notes.updated_at`
- [ ] `EXPLAIN QUERY PLAN` audit on all search queries

### Security
- [ ] Verify all admin routes require `admin` role
- [ ] Public file handler must not expose private files via path traversal
- [ ] Add `X-Content-Type-Options: nosniff` header
- [ ] Session token rotation on privilege escalation

---

## Release Milestones

### v0.2 — "Feels Real"
Design system overhaul, better drive UX, notes slash commands, admin dashboard.

### v0.3 — "Notes First"  
Block editor feel, note folders, checklist interaction, note search.

### v0.4 — "IDE Foundation"
Multi-file workspace, syntax highlighting, tab system.

### v0.5 — "Sharing Platform"
QR codes, expiring links, sharing dashboard, public link stats.

### v0.6 — "Operations Ready"
Full admin control center, activity logs, bulk operations, storage analytics.

### v1.0 — "Production Stable"
Performance audit, security audit, mobile-first responsive, documentation.
