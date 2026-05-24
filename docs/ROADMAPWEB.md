# TSSP Web & Product Redesign Roadmap

## 1. The Core Problem
The current TSSP web dashboard functions technically, but its UX is unacceptable for a production-grade system.
- **Aesthetics:** It looks like an AI-generated, generic dark-mode admin template with an arbitrary purple accent color. It lacks the professional, premium feel of products like Cursor, Linear, or Google Cloud.
- **Structure:** Features are crammed into a single sidebar as disjointed tabs. "Images", "Videos", "Documents" feel like separate apps rather than views of a unified Cloud Drive.
- **Notes:** The "Notion-like" notes system is currently just a basic markdown textarea with a preview. It lacks the block-based, rich-text experience of true note-taking apps.
- **Workspace:** The IDE feels like a toy form rather than a real workspace (VS Code style). It asks for languages upfront instead of inferring them, and lacks a real file tree.
- **Admin:** The admin panel is a flat scrollable page of cards rather than a powerful, separated operations console.
- **Color Scheme:** "AI-ish" colors. Needs a deep, sophisticated dark mode (true blacks, subtle grays, sharp high-contrast text, minimal accent colors).

## 2. The Vision: App OS (4-5 Distinct Apps)
TSSP should no longer feel like one monolithic dashboard. It should feel like a lightweight Local Cloud OS. When the user logs in, they should be able to switch between 4 distinct "Apps" or "Modes":

1. **Cloud Drive** (Google Drive style)
2. **Knowledge / Notes** (Notion style)
3. **Workspace / IDE** (VS Code / Cursor style)
4. **Operations / Admin Console** (Google Cloud / AWS style)

*The UI will use a unified shell (top bar / left rail), but switching between these apps will completely change the context of the sidebar and main area, providing a focused, native-feeling experience for each.*

## 3. UI/UX Design System Overhaul
**Color Palette (Cursor/Linear Style):**
- **Backgrounds:** True black (`#000000`) for the root, deep grays (`#0E1015`, `#16181D`) for panels and cards.
- **Borders:** Extremely subtle, low-contrast borders (`#2C2E35`) to separate panes.
- **Text:** High contrast white/off-white for primary text, clear readable grays for secondary/muted text.
- **Accents:** Remove the arbitrary purple. Use a sharp, professional blue (e.g., `#3B82F6`) or monochrome white for active states.
- **Typography:** System UI fonts (`Inter`, `Geist`, `SF Pro`) with precise hierarchy, tight kerning, and clean spacing.

## 4. Feature Implementation Plan (Step-by-Step)

### Phase 1: Frontend Architecture & UI Foundation
- [ ] **CSS Rewrite:** Overhaul `tokens.css`, `base.css`, and `layout.css` to implement the new "True Dark" design system.
- [ ] **App Split:** Refactor `index.html` and the JS routing (`app.js`, `views.js`) to support the 4 distinct App Modes.
- [ ] **Component Splitting:** Break the monolithic HTML/JS down into manageable files (`/js/features/drive/`, `/js/features/notes/`, etc.).
- [ ] **Global Command Palette:** Implement a highly responsive `Ctrl+K` global search overlay that overlays everything, finding files, notes, and workspaces.

### Phase 2: Cloud Drive (Google Drive Style)
**Web:**
- [ ] Implement a unified "Files" view. Remove separate "Images/Videos" tabs from the main nav; turn them into smart filters/lenses inside the Drive.
- [ ] Build a robust sidebar for the Drive: "My Drive", "Shared with me" (Public Links), "Starred/Pinned", "Recent".
- [ ] Build a context menu (Right-click) and details pane (Right sidebar) for files (metadata, tags, sharing status).
- [ ] Drag-and-drop area that feels native to the whole window.
- [ ] Folder navigation (Breadcrumbs, double-click to enter, tree view).
**Backend/CLI:**
- [ ] API routes for robust folder creation, renaming, and moving (`mv`, `cp`).
- [ ] Advanced metadata extraction and storage (Exif, dimensions, durations).
- [ ] `tssp share` CLI improvements (expiration times, password protection).

### Phase 3: Knowledge / Notes (Notion Style)
**Web:**
- [ ] Redesign the editor interface to hide the raw markdown where possible, moving toward a WYSIWYG or block-like experience.
- [ ] Implement seamless inline formatting (bold, italic) without exposing markdown asterisks constantly.
- [ ] Add `/` command menu for inserting blocks (Headings, Checklists, Tables, Code).
- [ ] Add page covers, icons, and dynamic color changing for note headers.
- [ ] Fast note creation (floating action button or `Ctrl+Alt+N`).
**Backend/CLI:**
- [ ] Support hierarchical notes (Notes inside Notes / Folders).
- [ ] Atomic autosave endpoints to prevent data loss.
- [ ] Note duplication and archiving APIs.

### Phase 4: Workspace / IDE (VS Code / Cursor Style)
**Web:**
- [ ] Implement a true 3-pane IDE layout: File Explorer (left), Editor Tabs (top), Code Area (center).
- [ ] Language inference from file extensions (remove the dropdown form).
- [ ] Syntax highlighting using a lightweight library (e.g., Prism.js or Monaco if resources permit, but highly optimized).
- [ ] Support opening multiple files in tabs simultaneously.
**Backend/CLI:**
- [ ] Virtual file system APIs for workspaces (nested folders inside workspaces).
- [ ] Future-proofing: Design the backend endpoints to securely attach to a containerized sandbox in the future (for script execution).

### Phase 5: Admin Console (High/Low Level Control)
**Web:**
- [ ] Build a dedicated Admin OS.
- [ ] **Dashboard:** Live server stats (CPU, RAM, Disk), active sessions, recent anomalous activity.
- [ ] **Access Control:** Granular user management, role assignments, device revocation lists.
- [ ] **File Management:** A God-mode file explorer to view, audit, and wipe any file across the system.
- [ ] **Safe Terminal:** A stylized, secure web console that runs pre-approved diagnostic tasks (not arbitrary `sh`).
**Backend/CLI:**
- [ ] Extensive telemetry APIs to feed the Admin dashboard.
- [ ] Strict role-based access control (RBAC) middleware for all admin routes.
- [ ] Batch operation APIs for bulk deletion and cleanup.

## 5. Execution Strategy
1. **Visual Overhaul First:** We will immediately fix the CSS tokens and layout structure so the app *feels* right.
2. **App Segmentation:** We will rewrite the DOM and JS to support the 4-app model.
3. **Feature Depth:** We will iterate through Drive, Notes, Workspace, and Admin, ensuring the backend supports the rich UI features.
4. **No Faking:** If a UI button exists, the backend must support it securely and efficiently (optimized for Orange Pi).
