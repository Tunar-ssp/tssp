# TSSP Web v2 — Roadmap & UX Spec

> Single‑source implementation brief. Hand this to your AI coding agent and ship feature‑by‑feature in the order written.
>
> **Product** — TSSP, a local‑first personal cloud OS that runs on Orange Pi. It is *not* an admin dashboard. It is a quiet operating system for your own data.
>
> **North star** — the home screen feels like a launcher; every action is one keystroke (`⌘K`) away; nothing leaves the Pi unless the user shares it.

---

## Table of contents

| # | Phase | What you ship |
|---|---|---|
| P0 | Foundations | Design tokens, brand kit, base primitives, theme engine |
| P1 | Shell | TopBar, Dock, Command palette, Notifications, Settings tray, App routing |
| P2 | Auth & people | Sign‑in, invite codes, sessions, trusted devices, roles, profile |
| P3 | Cloud Drive | Tree, grid/list, upload queue, preview, lenses, trash, tags, pins |
| P4 | Notes | Home, block editor, slash menu, outline, autosave, search |
| P5 | Workspace IDE | Project list, explorer, tabs, code editor, preview, find, status bar |
| P6 | Admin | Overview, diagnostics, users, safe console, activity, maintenance |
| P7 | Sharing | Share modal, QR, expiry, public viewer page, sharing center |
| P8 | Mobile | iOS / Android parity (launcher, drive, notes, palette) |
| P9 | System | Backups, integrity, alerts, storage volumes, telemetry stance |
| P10 | Polish | Motion, empty/loading/error, a11y, perf, i18n, onboarding |

---

## Conventions

**Feature IDs** look like `P3-DRV-04`. Use them in commits and PR titles.

**Status badges**

| Badge | Meaning |
|---|---|
| `MUST` | Blocks the phase from shipping. |
| `SHOULD` | Ship in the phase if time allows. |
| `COULD` | Defer to next phase if needed. |
| `WON'T` | Documented but explicitly out of scope. |

**Every feature** in this doc follows the same template:

- **Where it lives** — exact screen + position
- **UX** — what the user sees and does, step by step
- **States** — loading, empty, error, success, edge cases
- **Acceptance** — checkable criteria
- **Notes** — implementation hints, API shape, gotchas

**Voice rules** for all microcopy:

1. Say "your Pi", not "the cloud" or "the server".
2. Show *local* status before *WAN* status.
3. Errors always propose a fix.
4. Empty states are an opportunity — templates, hints, keyboard tips.

**Keyboard map** (final, do not deviate):

| Key | Action |
|---|---|
| `⌘K` | Command palette |
| `⌘1`–`⌘5` | Switch dock app (Launcher / Drive / Notes / Workspace / Admin) |
| `⌘N` | New (contextual: note / file / workspace) |
| `⌘E` | Share current selection |
| `⌘F` | Find in current view |
| `⌘⇧M` | Move to folder |
| `⌘↑` / `⌘↓` | Navigate breadcrumbs |
| `/` | Slash menu (in editor) / commands‑only filter (in palette) |
| `Space` | Quick preview |
| `Esc` | Close modal / clear selection / dismiss palette |
| `?` | Shortcuts overlay |
| `N` | New note (from anywhere outside text fields) |

---

# P0 — Foundations

> Design tokens, brand identity, theme engine, primitive components. Nothing user‑facing ships in this phase, but everything after depends on it. **Budget: 1 week.**

### P0-FND-01 — Design tokens — `MUST`

- **Where**: `tokens.css` — single file, imported by every page.
- **UX**: invisible. Tokens drive everything.
- **Tokens to define**:
  - **Color**: `bg`, `bg-1`, `surface`, `surface-2`, `surface-3`, `border`, `border-2`, `hairline`, `text`, `text-2`, `muted`, `dim`, `faint`. Brand: `green` (`#5BE39A`), `pink` (`#FF5FA2`), `orange` (`#FF8A3D`). Functional: `blue` (`#6EA8FF`), `cyan` (`#58D6E0`), `violet` (`#A394FF`), `warning` (`#FBBF24`), `danger` (`#FF6B6B`), `success` = `green`.
  - **Spacing** (4‑pt): `s-1` 4 → `s-10` 64.
  - **Radii**: `r-1` 4 → `r-7` 22 + `r-full`.
  - **Typography**: `ff-sans` DM Sans, `ff-display` Bricolage Grotesque, `ff-mono` JetBrains Mono, `ff-hand` Caveat (used **only** for the logo wordmark, never UI). Sizes 12, 13, 14, 15, 16, 18, 20, 24, 32, 40, 56.
  - **Elevation**: `shadow-card`, `shadow-modal`, `shadow-dock`.
- **Acceptance**: no hard‑coded hex in any component; theme flip works by overriding tokens on `.light` class only.

### P0-FND-02 — Theme engine (dark / light) — `MUST`

- **Where**: `<html data-theme="dark|light">` + `.tssp` root.
- **UX**: in Settings tray → Appearance. Three options: System / Dark / Light. System reads `prefers-color-scheme`. Transition is instantaneous (no animation — animating theme swaps looks cheap on a Pi).
- **States**:
  - User has never picked → defaults to **Dark** (Cursor‑adjacent).
  - User picks Light → persists in `localStorage.tssp.theme`.
- **Acceptance**: every screen renders correctly in both themes with no contrast warnings (WCAG AA against `text` on `bg`).

### P0-FND-03 — Brand kit — `MUST`

- **Where**: `brand/` folder, surfaced in any screen via `<Logo size layout>`.
- **Mark**: black squircle. Left half = green eye (3 horizontal strokes + pupil). Right half = pink mouse body with orange cable curling out the top. Strokes only; no fills except the pupil.
- **Wordmark**: handwritten `tssp` in Caveat, gradient `green → pink → orange`. *Caveat appears here and nowhere else in the product.*
- **Lockup**: mark left, wordmark right, 22% gap.
- **Variants**: `mark` only (chip, favicon, dock); `lockup` (login, splash, marketing). Sizes 24 → 140.
- **Acceptance**: same SVG renders crisp at 24px (favicon) and 140px (login).

### P0-FND-04 — Icon set — `MUST`

- **Where**: `<Ico>` primitive in `icons.jsx`. ~50 line icons at 24×24 viewBox, 1.6 stroke.
- **Rules**: monoline, rounded caps, no fills (except dots). Sized in 12 / 13 / 14 / 16 / 18 px.
- **Acceptance**: all icons share visual weight when laid out in a single row.

### P0-FND-05 — Primitive components — `MUST`

Build these once, reuse everywhere. **Do not redesign them per app.**

| Component | Props | Purpose |
|---|---|---|
| `Btn` | `kind` (primary / accent / ghost / solid / danger), `size` (sm/md/lg), `icon`, `iconRight` | Every clickable affordance |
| `Pill` | `tone` (green/pink/blue/orange/violet/warn/danger/neutral) | Status, tags, filters |
| `Kbd` | children | Keyboard hints |
| `Card` | `head`, `foot`, `pad`, `accent` | Surface container |
| `Bar` | `value` 0–100 or `segments[]` | Progress / breakdown |
| `Ring` | `value`, `tone`, `label`, `sub` | Circular metric |
| `StatusDot` | `tone` | 7px online/offline dot |
| `Toast` | `tone`, `title`, `body`, `action` | Non‑blocking feedback |
| `Modal` | `title`, `size`, `onClose` | Centered overlay, backdrop blur |
| `Sheet` | `side` (right/bottom) | Side panel, mobile action sheet |
| `Tooltip` | `delay`, `kbd` | 200ms hover, dark, with optional keystroke |

### P0-FND-06 — Routing & app frame — `MUST`

- **Where**: top‑level `<AppFrame>` that hosts TopBar + content + Dock.
- **Routes** (use `history` API or your stack's router):
  - `/` — Launcher
  - `/drive/:path*` — Drive at a folder
  - `/drive/file/:id` — file preview modal route
  - `/notes` — notes home
  - `/notes/:id` — note editor
  - `/workspace` — project list
  - `/workspace/:project/file/:path` — file in IDE
  - `/admin/:section` — admin section (overview / users / safe-console / …)
  - `/share/:code` — public viewer
  - `/settings/:section`
- **Behavior**: Dock is **always** rendered. Active app is detected from the URL. Switching apps preserves each app's last sub‑path (so jumping Drive → Notes → Drive returns to the same folder).
- **Acceptance**: deep‑link any URL → it renders the right app with correct dock highlight.

---

# P1 — Shell

> The dock, the top bar, the command palette. This is the OS feeling. **Budget: 1.5 weeks.** Everything in P1 must work *before* any app is built.

### P1-SHL-01 — TopBar — `MUST`

- **Where**: fixed top, 52px tall, on every screen except mobile editor full‑screen and public viewer.
- **Layout** (left → right):
  1. Brand mark (24px) + small `tssp` wordmark in hand font + `/` separator + current context (e.g. `Drive`, `Notes / tssp v2 launch`).
  2. **Centered**: command bar (width 460px) — looks like an input, shows search placeholder + `⌘K` chip. Clicking opens the Command palette (P1-SHL-03).
  3. Right: `Upload` button (Drive‑contextual), Notifications bell (with dot when unread), user avatar (initial inside circle), local/WAN status pill (`Local` green dot, `Offline` red dot, `WAN` violet dot if remote access enabled).
- **Behavior**:
  - Clicking the brand mark → returns to Launcher (`/`).
  - Clicking the user avatar → opens Profile + sign‑out sheet.
  - The status pill is informative *and* clickable → opens "Network & access" in Settings tray.
- **Edge**: in a deep route, the path is expanded as crumbs (e.g. `Notes / tssp / v2 launch checklist`). Each crumb is a link.
- **Acceptance**: TopBar is 52px on every page; never scrolls away.

### P1-SHL-02 — Dock — `MUST`

- **Where**: fixed bottom, centered, floating 18px from edge. Always visible (mobile too).
- **Composition**: 5 squircles + divider + system tray (Settings). Apps: Launcher (logo), Drive, Notes, Workspace, Admin.
- **Visual**: 56px icons, 12px gap, 22px outer radius, glass background `rgba(20,22,29,0.62)` + 28px backdrop blur. 1px white inset highlight, 18px black drop shadow.
- **Active indicator**: 16px wide × 4px tall pill under the active icon. Inactive icons get a 4px dot at half opacity. Active icon scales 1.06 and lifts 6px.
- **Hover**: icon lifts 4px, label appears in a small dark chip 28px above. 120ms ease.
- **Keyboard**: `⌘1`–`⌘5` switches apps.
- **States**:
  - **Live indicator**: each icon can show a notification badge (small pink dot top‑right) when its app has new activity.
  - **Active app pulsing**: if a long‑running task is happening (e.g. upload in Drive), the active dot pulses 1× per 2s.
- **Mobile**: dock collapses to bottom inset, icons resize to 42px, system tray (settings) is moved to TopBar avatar menu.
- **Acceptance**: dock renders correctly over any wallpaper; clicking outside the dock never dismisses it.

### P1-SHL-03 — Command palette — `MUST`

- **Trigger**: `⌘K` from anywhere. Mobile: tap search field in TopBar.
- **Layout**:
  - Modal, 720×auto, centered, top offset 110px.
  - Search input (auto‑focus on open).
  - Filter row: `All / Files / Notes / Workspaces / Actions / Settings`.
  - Result list (grouped by category, max 6 per group, "see all" link below if more).
  - Footer with `↑↓ navigate`, `↵ open`, `⌘↵ open in new tab`, `Esc dismiss`.
- **Result row** = icon (file glyph / note dot / action bolt) + title (with highlighted match) + breadcrumb subtitle.
- **Behavior**:
  - Empty query: shows recent searches + 4 suggested actions ("New note", "Upload file", "Show shared", "Run integrity check").
  - Typing 1+ char: live search files (name + tag), notes (title + body), workspaces (name + path), actions (registered command names), settings (key).
  - Each result type maps to an "open" handler. `↵` opens; `⌘↵` opens in a new tab.
  - If the top result has a "create from search" sibling action (`"renewal"` → `Create note "renewal"`), it appears under the Actions group.
  - `Tab` cycles filter chips.
- **Performance**: search must return in <30ms for ≤10k items. Use a memory‑resident index (sqlite FTS5 server‑side, optional local cache).
- **States**:
  - 0 results → "Nothing matches `<query>`" + the 3 most useful actions ("Create note…", "Create file…", "Search inside notes…").
  - Loading → 80ms shimmer rows.
- **Acceptance**: every action in the entire product is reachable here. If you can't get to it via `⌘K`, it doesn't exist.

### P1-SHL-04 — Notifications — `SHOULD`

- **Where**: bell in TopBar opens a right‑side Sheet, 360px, full‑height.
- **Sections**: Unread / All / Muted.
- **Notification shape** = icon + title + body (1 line) + relative time + (optional) actions ("Restore", "Open file").
- **Sources**: upload finished, share viewed, sign‑in from new device, integrity check finished, backup finished, public link expiring in 24h.
- **Behavior**: clicking a notification deep‑links to the relevant screen. Swipe (mobile) or X (desktop) dismisses. "Mark all read" footer button.
- **Toasts**: short‑lived bottom‑left, 5s auto‑dismiss, stack max 3. Used for transient actions (file uploaded, link copied, note saved).
- **Acceptance**: notifications survive page reload (persisted server‑side); unread count syncs across tabs.

### P1-SHL-05 — Settings tray — `MUST`

- **Where**: dock → system tray icon → right‑side Sheet (480px). *Not* a route, *not* the Admin app. Settings = "your personal preferences"; Admin = "the machine".
- **Sections**:
  1. **Account** — name, avatar, change password, sign out.
  2. **Appearance** — Theme (System/Dark/Light), Density (Comfortable/Compact), Accent (Green/Pink/Orange/Blue — defaults Green).
  3. **Editor** — Font (Sans/Mono‑mixed/Serif headings), Line height, Indent size.
  4. **Keyboard** — list of all shortcuts (read‑only in v2; rebindable in v3).
  5. **Network & access** — show LAN address, WAN access toggle (off by default), Tailscale link.
  6. **Notifications** — per‑source toggles, Do‑not‑disturb hours.
  7. **About** — version, build, license, "Open source links".
- **Acceptance**: every preference persists per user, scoped to device when device‑local (e.g. Compact density).

### P1-SHL-06 — Shortcuts overlay — `SHOULD`

- **Trigger**: `?` anywhere.
- **Visual**: centered modal, 720px, grid of categories (Global / Drive / Notes / Workspace / Admin) with `Kbd` chips next to each label.
- **Acceptance**: every entry in the keyboard map renders here.

### P1-SHL-07 — Onboarding (first run) — `SHOULD`

- **Trigger**: first time a user signs in to a fresh tssp instance.
- **Steps** (3 only):
  1. **Welcome** — handwritten "tssp" hero, "Your personal cloud, right here." → "Get started".
  2. **Name your Pi** — input + LAN address auto‑detected (read‑only).
  3. **Create owner** — username, password, optional email.
- **Skip**: not allowed. Owner must exist.
- **After**: lands on Launcher with a pre‑seeded "Welcome" note in Notes and a sample folder in Drive.

---

# P2 — Auth & people

> Who can use the Pi, and how. **Budget: 1 week.**

### P2-AUTH-01 — Sign‑in — `MUST`

- **Route**: `/signin` (no shell, full screen).
- **Layout**: centered card, 380px wide, dark wallpaper.
  - Top: lockup (large mark + handwritten wordmark).
  - Below: username input → password input → "Sign in" primary button.
  - Below that: small "Have a code?" link → opens Access Code flow (P2-AUTH-04).
  - Footer: device fingerprint hash (last 4 chars, in mono) + LAN address.
- **States**:
  - Wrong credentials → red border on password field, "Wrong username or password. Try again." Counter to 5 then 30‑second lockout per IP.
  - Locked → show countdown.
  - First time on this device → after success, prompt "Trust this device?" toggle (default off).

### P2-AUTH-02 — Sessions — `MUST`

- **Where**: stored server‑side. Listed in `/admin/sessions` (P6) **and** in Settings → Account → "Devices" (read‑only for self).
- **Shape**: `{user, device, ip, agent, last_seen, scope (LAN/WAN), trusted, created}`.
- **Behavior**:
  - 30‑day rolling expiry; sliding refresh on each request.
  - "Sign out everywhere" button = revoke all sessions for current user.
  - On revoke, the affected client gets booted to `/signin` within 5 seconds (poll or WS).

### P2-AUTH-03 — Trusted devices — `SHOULD`

- **Where**: Settings → Network & access → "This device" + list of other trusted devices.
- **Behavior**:
  - Trusted device skips re‑auth on each visit within 30 days.
  - Untrusted device requires password every 24h.
  - Owner can revoke any device from Admin.

### P2-AUTH-04 — Invite codes — `MUST`

- **Purpose**: add a person without typing emails — works fully offline.
- **Owner flow**:
  1. Admin → Users → "Invite via code".
  2. Modal: pick role (Viewer / Editor / Admin), pick TTL (24h / 7d / 30d), pick max uses (1 / unlimited).
  3. Generate. Modal shows a **6‑character code** in mono + a **QR** that encodes `tssp://join?code=ABC123&host=tssp.local`.
  4. Owner shows the QR to the recipient (phone) or reads the code aloud.
- **Recipient flow**:
  1. Recipient navigates to `tssp.local` and clicks "Have a code?" on sign‑in.
  2. Enters code → name → password → "Join".
  3. Auto‑signed in, lands on Launcher.
- **States**:
  - Code expired → red error inline.
  - Code used (1‑use codes) → "This code has been used. Ask the owner for a new one."
  - Too many tries → IP lockout same as P2-AUTH-01.
- **Acceptance**: a phone on the same LAN can scan the QR and join in under 30 seconds.

### P2-AUTH-05 — Roles — `MUST`

| Role | Can |
|---|---|
| **Owner** | Everything. Singular. Cannot be deleted; can transfer ownership. |
| **Admin** | Everything except deleting owner / transferring ownership. |
| **Editor** | Upload, edit, share, create notes/workspaces. Cannot manage users or run safe‑console commands. |
| **Viewer** | Read‑only access to whatever's explicitly shared with them. |
| **API** | Token‑based programmatic access. Scoped permissions. |

- **Behavior**: every API endpoint declares required role. UI hides actions the user can't perform (don't gray out; hide).

### P2-AUTH-06 — Profile — `SHOULD`

- **Where**: TopBar avatar → opens a small menu (200px wide):
  - Avatar + name + email
  - "Settings" → opens tray
  - "Devices" → opens tray to Devices
  - "Sign out"
  - "Sign out everywhere"
  - (Owner only) "Switch to Admin" → routes to `/admin`.

---

# P3 — Cloud Drive

> A real file manager that respects how the user organizes things. **Budget: 2.5 weeks.**

### P3-DRV-01 — Drive shell — `MUST`

- **Route**: `/drive/:path*`.
- **Layout**: 3 panes — left sidebar (tree, 240px) / center (content) / optional right sheet (details, 320px, opens on selection).
- **Pane state**: width persisted per user; both panes have a 1px hairline border.

### P3-DRV-02 — Folder tree — `MUST`

- **Where**: left sidebar.
- **Composition**:
  - **Upload** button at top (full‑width accent).
  - "All files" — the root, expands.
  - "Shared with me", "Recents", "Starred", "Public", "Trash" — virtual folders.
  - Below: real folders (alphabetical, lazy expand).
- **Behavior**:
  - Click row → navigate.
  - Right‑click row → context menu (new folder, rename, color label, move, delete).
  - Drag a file onto a folder row → moves it (toast: "Moved 14 files to /docs/legal" with Undo).
  - Drag a row onto another row → reparent.
- **Visual**: 28px row, 14px icon, 12px text. Depth indent 14px. Color labels are 3px left bar (Green/Pink/Orange/Blue/Violet/None).
- **Footer**: storage summary tile (used / total, color‑segmented bar by file type).

### P3-DRV-03 — Breadcrumbs — `MUST`

- **Where**: top of content pane.
- **Behavior**: each crumb is a click; right‑click any crumb → "Open in new tab", "Copy path".
- **Overflow**: collapse middle crumbs to `…` when too long; clicking `…` expands a dropdown.

### P3-DRV-04 — Grid view — `MUST`

- **Where**: content pane, default for image/mixed folders.
- **Card**: 4:3 aspect thumbnail + filename (1 line, truncated middle) + size/modified mono caption. 10px gap. Responsive 6 columns at 1440px, 4 at 1024px, 3 at 768px, 2 at mobile.
- **Thumbnail**:
  - Images: actual thumb (server generates 256px, lazy‑loaded).
  - Videos: poster frame + duration chip bottom‑right.
  - PDFs: first page render.
  - Audio: waveform.
  - Other: file glyph (P0‑FND‑05) centered on `--surface-2`.
- **Affordances per card**:
  - Top‑left: globe icon if public.
  - Top‑right: pin icon if pinned.
  - Bottom‑left: checkbox (visible on hover / when in select mode). Selected = blue border + 3px blue glow.
- **Selection**:
  - Click = select (single).
  - `⌘`/`Ctrl`+click = toggle.
  - `Shift`+click = range select.
  - Drag a rubber band over cards = lasso select.
- **Acceptance**: thumb loads in ≤200ms for cached, ≤2s for new.

### P3-DRV-05 — List view — `MUST`

- **Where**: same content pane, alternative.
- **Columns**: checkbox / Name / Size / Modified / Tags / Sharing / Actions.
- **Behavior**:
  - Click column header → sort. Sort indicator chevron next to label.
  - Hover row → background `var(--surface-2)`; right edge shows tiny `⋮` menu.
  - Selected row = blue tinted background + left border.
- **Acceptance**: lists scroll smoothly at 10k rows (virtualize).

### P3-DRV-06 — Grid/list toggle — `MUST`

- **Where**: right side of the toolbar.
- **Behavior**: segmented control, 2 icons. Persisted **per folder** (not per user). Users want a different default for `/photos` vs `/code`.

### P3-DRV-07 — Toolbar — `MUST`

- **Where**: between breadcrumbs and content.
- **Composition** (left → right): Breadcrumbs (separate row) / Filter chips (Type / Tags / Modified) / Bulk actions (when ≥1 selected) / View toggle.
- **Bulk actions**: appear when N>0 selected: count pill + Download / Share / Rename / Move / Tag / Delete (red).
- **Filter chips**: dropdowns. Multi‑select. Chips stack horizontally, wrap below at 1024px.

### P3-DRV-08 — Upload — `MUST`

- **Trigger**:
  - Upload button (sidebar top, or `Upload` in TopBar).
  - Drag files anywhere in Drive content → full‑bleed dashed border + dropzone hint ("Drop to upload to /docs/legal").
  - `⌘V` paste image from clipboard.
  - Mobile: native file picker via `<input type="file">`.
- **Queue panel**:
  - Floating bottom‑right, 320px, draggable. Survives navigation.
  - Header: "Uploading 4 files" + total throughput pill + minimize / close.
  - Row per file: glyph + name (truncated) + progress bar + state ("queued" / "uploading" / "✓" / "✕").
  - Footer: "Pause all" / time remaining ("~38s").
- **Behavior**:
  - Chunked uploads (4 MB chunks), resumable across page reloads.
  - Conflict on duplicate name: modal "There's already a `lease.pdf` here." → Replace / Keep both / Skip / Apply to all.
  - Folder upload (drag a folder) = recurses, preserves structure.
- **Error**: out of disk → "Out of space on /mnt/data. Free 1.2 GB or attach a new volume." with "Open Volumes" CTA.
- **Acceptance**: a 1 GB file resumes after a reload from where it stopped.

### P3-DRV-09 — File card / row actions — `MUST`

- Open (double‑click / `↵`)
- Quick preview (`Space`)
- Rename (`F2` or double‑click name)
- Move to (`⌘⇧M`) — opens folder picker modal
- Duplicate (`⌘D`)
- Tag (`T`) — opens tag chip popover
- Pin / Unpin (`P`)
- Star (`S`)
- Make public (`⌘⇧P`)
- Share (`⌘E`)
- Copy link (`⌘C`)
- Download (`⌘↓`)
- Archive
- Move to trash (`⌫`)

All available via right‑click context menu (same order, same shortcuts visible as `Kbd`).

### P3-DRV-10 — File preview — `MUST`

- **Trigger**: `Space` or click "Preview" / open.
- **Modal**: 80% viewport, top‑offset 80px, dark backdrop with blur. Right side shows Details panel (320px, can be hidden via `i`).
- **Chrome**: glyph + filename + path + prev/next within folder (`◀ 3/12 ▶`) + Download / Share / Close.
- **Lenses** (renderer per type):
  - **Image** — centered, scroll/pinch to zoom, double‑click 1:1, arrow keys pan.
  - **Video** — `<video>` controls + scrubber. Remembers position per file (localStorage).
  - **PDF** — paginated, sidebar with thumbs.
  - **Code/text** — Monaco/CodeMirror read‑only with syntax highlight, line numbers.
  - **Markdown** — rendered + raw toggle.
  - **CSV/TSV** — table renderer first 1000 rows.
  - **Audio** — waveform + transport.
  - **Other** — "No preview available" + Download CTA.
- **Details panel** sections (collapsible):
  - Type / Size / Modified / Created / Path / Hash
  - Sharing — link + viewers + view count + revoke button
  - Tags — chips, click `+ add`
  - Activity — last 5 events with mono timestamps
- **Acceptance**: opening a 2MB image previews in ≤500ms.

### P3-DRV-11 — Image / video / document lenses (in‑Drive) — `MUST`

- **Where**: sidebar virtual folders: "Photos", "Videos", "Documents", "Music", "Code".
- **Behavior**: each is a filtered cross‑folder view of files matching that type. Same grid/list components, just pre‑filtered. Inside, breadcrumbs read `Photos / 2024 / May` (filter, year, month groupings).

### P3-DRV-12 — Tags — `MUST`

- **Shape**: free‑text strings, lowercase, max 24 chars, no spaces (allow dashes).
- **Where**: per‑file tag chips, and the Tag filter chip in toolbar.
- **Tag editor popover**: 220px, search field, list of all existing tags, "+ create" at bottom. Click chip to toggle.
- **Display**: chips in `var(--surface-2)`, mono small text. Max 3 visible on cards; `+N` overflow chip.

### P3-DRV-13 — Pin / Star — `MUST`

- **Pin** (`P`) = personal, surfaces in "Pinned" sidebar row + top of grid in current folder.
- **Star** (`S`) = personal, surfaces in "Starred" sidebar row.
- **Both** allowed simultaneously. Stars survive moves; pins are folder‑scoped.

### P3-DRV-14 — Move / rename — `MUST`

- **Rename**: inline edit. Selects basename only (extension untouched). `↵` confirms, `Esc` cancels. Error if name collides ("There's already a file with that name").
- **Move**: modal with mini folder tree + breadcrumb of destination + "New folder…" button + confirm. Or drag to sidebar folder.

### P3-DRV-15 — Trash — `MUST`

- **Where**: sidebar bottom row.
- **Behavior**:
  - Deleted files go here for 30 days, then auto‑purge.
  - Trash view = list with extra "Deleted" column.
  - Right‑click: Restore to original / Restore to… / Delete permanently.
  - "Empty trash" button in top toolbar (confirms with count + size).
- **Acceptance**: restoring a file recreates parent folders if needed.

### P3-DRV-16 — Bulk actions — `MUST`

- **Trigger**: ≥1 selected.
- **Position**: toolbar shows count pill ("3 selected") + actions.
- **Actions**: Download (zips on the fly), Share (creates one link for the bunch or N individual links — modal asks), Rename (batch — opens "find/replace in names" with preview), Move, Tag, Star, Pin, Trash.
- **Cancel**: `Esc` or click empty area clears selection.

### P3-DRV-17 — Context menu — `MUST`

- **Trigger**: right‑click on a card / row / folder.
- **Visual**: dark rounded menu, 230px, items 28px tall, dividers between groups, shortcut chips on right.
- **Order**: Preview / Download / Share… / Copy link / —— / Rename / Move / Duplicate / Tags / —— / Make public / Pin / —— / Archive / Move to trash.

### P3-DRV-18 — Empty state — `MUST`

- **Where**: any folder with 0 items.
- **Visual**: large icon, "Nothing here yet", subtitle ("Drag files anywhere to upload, or paste a folder. Everything stays on this Pi."), Upload + New folder buttons.
- **Special root**: if it's the user's first Drive visit, show a 30s "How Drive works" 3‑step illustrated card *above* the empty state (dismissible).

### P3-DRV-19 — Loading state — `MUST`

- **Grid**: 12 skeleton cards with shimmer animation (1.6s linear).
- **List**: 8 skeleton rows.
- **Tree**: 6 skeleton rows.

### P3-DRV-20 — Error state — `MUST`

- Failed to load folder → centered card with icon + cause ("Couldn't reach `tssp.local`") + Retry + Open Diagnostics.

---

# P4 — Notes

> Pages and blocks. Not markdown. **Budget: 2 weeks.**

### P4-NOT-01 — Notes shell — `MUST`

- **Route**: `/notes` (home) and `/notes/:id` (editor).
- **Layout**: 3 panes — left sidebar (workspaces / folders / tags, 240px) / middle list (note list, 280px, collapsible) / right (note canvas, fluid).
- **Mobile**: single column with back‑swipe.

### P4-NOT-02 — Sidebar — `MUST`

- **Sections**: New note button (accent, full‑width). Workspaces group (All / Pinned / Recent / Archive). Folders group (user folders, color‑tagged). Tags group (chips list, click to filter).
- **Folders** are personal, free‑form, not nested deeper than 3 levels.

### P4-NOT-03 — Notes home (cards grid) — `MUST`

- **Where**: `/notes` content pane.
- **Composition**: section "Pinned" (4‑col grid), section "All" (4‑col grid). Each card 156px tall: color left‑bar / title / preview text (4‑line clamp) / tag chips + relative time.
- **Behavior**: click card → opens editor. Right‑click → menu (Open, Duplicate, Pin, Color, Archive, Trash).
- **Sort**: Modified (default), Created, Title, Color.

### P4-NOT-04 — Editor canvas — `MUST`

- **Where**: `/notes/:id`.
- **Layout**:
  - Top bar with breadcrumbs + tag chips + save status (`● Saving…` / `✓ Saved 12s ago`) + Share + `⋮`.
  - Title input (huge, 36px, display font, autofocus on new note).
  - Meta strip: last edited, word count, block count.
  - Block list (main).
  - Right rail (outline + linked items + "Linked" widget).
- **Behavior**: editor is block‑based. Each block has a hover handle on the left (`⋮⋮` drag + `+` insert). Click handle → block menu (Convert to, Duplicate, Color, Delete).
- **Blocks supported in v1**:
  1. Paragraph
  2. Heading 1 / 2 / 3
  3. Bullet list
  4. Numbered list
  5. Checklist (`[ ]` / `[x]`)
  6. Quote
  7. Callout (with color tone)
  8. Code block (language picker, copy button, line numbers)
  9. Divider
  10. Image (from Drive, paste, or upload)
  11. File embed (Drive picker, shows file glyph + name + open button)
  12. Link card (URL → fetched OG card with title/desc/favicon)
  13. Table (basic, no formulas)
- **Acceptance**: typing latency <16ms on a 1000‑block note.

### P4-NOT-05 — Slash menu — `MUST`

- **Trigger**: `/` at the start of an empty line or on a new line.
- **Layout**: 320px popover anchored to caret. Header "Insert block · /co" (echoes filter). List of block types with icon + label + description + shortcut. `↑↓` selects, `↵` inserts, `Esc` dismisses.
- **Filtering**: typing after `/` filters (e.g. `/co` matches Callout, Code). Fuzzy match.
- **Acceptance**: every block type listed in P4‑NOT‑04 is insertable via slash.

### P4-NOT-06 — Markdown compatibility — `MUST`

- **Inline**: `**bold**`, `*italic*`, `` `code` ``, `~~strike~~`, `[label](url)` auto‑convert as user types.
- **Blocks**: `# `, `## `, `### `, `- `, `1. `, `> `, `[]` for checkbox, ``` ``` ``` for code (with language hint), `---` for divider, all convert on space.
- **Paste**: pasting markdown → renders as blocks, not raw text.
- **Export**: per‑note "Export as Markdown" in `⋮` menu (`.md` file written to current Drive folder or downloaded).

### P4-NOT-07 — Outline — `SHOULD`

- **Where**: right rail, 200px wide.
- **Behavior**: shows H1/H2/H3 nesting. Click a heading → smooth‑scroll to block. Current block's heading is highlighted with blue left bar.
- **Mobile**: outline opens as a sheet via title tap.

### P4-NOT-08 — Linked items — `SHOULD`

- **Where**: right rail under outline.
- **Behavior**: shows files / notes / workspaces referenced inside the note (file embeds, link cards). Click → open.

### P4-NOT-09 — Autosave & status — `MUST`

- **Behavior**: debounced save 800ms after last keystroke. Status indicator in top bar: `Saving…` → `Saved` → idle. On focus loss, force save.
- **Conflict**: if the note was edited on another device, show a non‑modal toast "This note was edited on Mira's iPhone — merging" with Diff button. Conflict resolution = last‑write‑wins per block, but show both versions if both changed in the last 30s.
- **Acceptance**: zero data loss across reload, navigation, or browser crash (use IndexedDB shadow copy).

### P4-NOT-10 — Search inside notes — `MUST`

- **Trigger**: `⌘F` inside editor (in‑note); `⌘K` global cross‑note.
- **In‑note**: top‑right widget, search field + result counter + ↑↓ to step. Matches highlight in yellow on the canvas.
- **Cross‑note**: each result row shows note title + matched snippet (bold around match) + folder breadcrumb.

### P4-NOT-11 — Duplicate / Archive / Delete — `MUST`

- **Duplicate**: creates `<title> copy` in same folder.
- **Archive**: moves to "Archive" virtual folder; hidden from All but searchable.
- **Delete**: trash, recoverable from `/notes/trash` for 30 days.

### P4-NOT-12 — Templates — `SHOULD`

- **Where**: New note button → menu: Blank / Daily / Meeting / Project / Reading list.
- **Behavior**: each template is itself a regular note with the right blocks pre‑filled. User can edit templates in Settings → Editor → "Manage templates".

### P4-NOT-13 — Mobile editor — `MUST`

- **Layout**: single column. Title at top, blocks below. Toolbar fixed above keyboard with: `+` add block / `H` heading / `•` list / `☑` checklist / `</>` code / `📷` image / `/` slash.
- **Acceptance**: works in iOS Safari at 100% function (drag handles fall back to long‑press menu).

### P4-NOT-14 — Empty state (no notes) — `MUST`

- Large notes icon, "Start your first note", subtitle "Hit `N` from anywhere", three big template chips (Daily / Meeting / Project).

---

# P5 — Workspace IDE

> A real, light IDE. **Budget: 2 weeks.**

### P5-WKS-01 — Project list — `MUST`

- **Route**: `/workspace`.
- **Visual**: grid of project cards: color square (first letter mono) + project name + branch + file count + dirty file count pill + last opened.
- **Top bar**: "New workspace" accent button. Open workspace from path (modal: enter path under `/mnt/data` or upload zip).
- **Empty state**: "No projects yet" + Create + Open existing.

### P5-WKS-02 — IDE shell — `MUST`

- **Route**: `/workspace/:project/file/:path*`.
- **Layout**:
  - 48px activity rail (Files / Search / Source / Terminal / History)
  - 240px explorer
  - main editor area (tabs + path bar + editor + minimap)
  - 340px right pane (Preview / Outline / Terminal — tabbed)
  - bottom 26px status bar
  - dock below
- **Theme**: editor honors global theme. Code colors:
  - keyword pink
  - string green
  - type cyan
  - function blue
  - variable text
  - comment dim
  - tag/jsx violet

### P5-WKS-03 — Explorer — `MUST`

- **Behavior**: tree of files in the project. `+` button for new file (input row inline) and new folder. Right‑click for full menu (Rename, Move, Delete, Duplicate, Reveal in Drive).
- **Dirty indicator**: orange 6px dot to the right of unsaved files.
- **Icons**: from file type glyph (P0‑FND‑05).
- **Acceptance**: a 5,000‑file tree renders smoothly (virtualize).

### P5-WKS-04 — Tabs — `MUST`

- **Behavior**: opening a file adds a tab; double‑click pinned. Unsaved = italic name + orange dot. Hover tab shows X.
- **Pin**: right‑click → Pin. Pinned tabs cluster left.
- **Limit**: 12 tabs visible; overflow becomes `⌄` menu.

### P5-WKS-05 — Editor — `MUST`

- **Engine**: Monaco (or CodeMirror 6).
- **Required features**: syntax highlight (auto from extension), line numbers, indent guides, wrap toggle, find/replace, multi‑cursor, code folding, bracket match, autocomplete (from open file + same project), format on save (if a formatter is configured).
- **Status bar** shows: branch / sync state / language / encoding / line ending / cursor `L11:Col28` / problems / warnings / sandbox notice / version.
- **Acceptance**: opens a 50k LOC file in <1s, scrolls at 60fps.

### P5-WKS-06 — Markdown preview — `MUST`

- **Where**: right pane, Preview tab. When current file is `.md`, shows rendered HTML with GFM.
- **Sync scroll**: top‑aligned to editor scroll (best‑effort).
- **Acceptance**: image refs from same folder resolve correctly.

### P5-WKS-07 — Find in file / project — `MUST`

- **In file**: `⌘F` widget top‑right of editor: input, prev/next, count, case/word/regex toggles, close.
- **Replace**: `⌘⌥F` adds a replace row + "Replace" / "Replace all" buttons.
- **Project**: activity rail → Search → input + scope + case/regex toggles + tree of results with snippets, click to jump.

### P5-WKS-08 — Status bar — `MUST`

- **Items**: branch (pink dot if dirty) / sync status / language / encoding / line ending / cursor / problems / warnings / sandbox notice / tssp version.
- **Click**: clicking any item opens its detail in the right pane.

### P5-WKS-09 — Sandbox notice — `MUST`

- **Visible**: status bar shows "Sandbox · run requires unlock".
- **Behavior**: there is no shell. Files are read/write only. "Run" buttons are visible but disabled with tooltip "Sandboxed — unlock in Admin → Maintenance → Run mode".
- **Acceptance**: no path exists for arbitrary code execution from Editor.

### P5-WKS-10 — Live preview (component / file) — `COULD`

- **Where**: right pane → Preview tab when the current file matches a known UI framework (`.tsx`, `.jsx`, `.svelte`, `.vue`).
- **Renderer**: server‑side bundle on save → renders in iframe. With Props panel below.
- **Acceptance**: hot reload <500ms after save.

### P5-WKS-11 — Workspace from Drive — `SHOULD`

- **Behavior**: any folder in Drive can be "Open as workspace" from its context menu, which routes to `/workspace/<path>`. The same files are visible in both apps; edits sync.

### P5-WKS-12 — Empty state — `MUST`

- "No file open" → message with shortcuts (`⌘P` to open file, `⌘N` to create).

---

# P6 — Admin

> Operations center. Not a long scroll page. **Budget: 2 weeks.**

### P6-ADM-01 — Admin shell — `MUST`

- **Route**: `/admin/:section`.
- **Layout**: left sidebar (sectioned nav: System / People / Content / Storage). Right content.
- **Visual cue**: sidebar uses violet accent (instead of brand green) so users always know they're in Admin.

### P6-ADM-02 — Overview — `MUST`

- **Layout**:
  - 4 big stat tiles: CPU / Memory / Disk / Network. Each has value, 14‑day trend label, sparkline.
  - Storage breakdown card: segmented bar by type + legend grid.
  - Active sessions card: list of 5 most recent with user/device/IP/last‑seen + Revoke.
  - Recent activity table (last 7 events).
- **Behavior**: refresh every 5s for stat tiles (websocket if available, fallback poll).

### P6-ADM-03 — Diagnostics — `MUST`

- **Where**: `/admin/diagnostics`.
- **Sections**:
  - Real‑time charts (CPU / Mem / Disk I/O / Net) over 1h / 6h / 24h / 7d (range picker).
  - Temperature + fan speed (with thresholds).
  - Top processes (PID, command, %CPU, %Mem) — read‑only.
  - Logs viewer (filterable by service, tail mode).
- **No shell access** even here.

### P6-ADM-04 — Users — `MUST`

- **Layout**: stat tiles (Total / Active sessions / Pending invites / API keys) + filter row + table.
- **Table cols**: Avatar / Name / Email / Role / Last seen / Sessions / Actions (⋮).
- **Right rail**: clicking a user opens detail (avatar, role pill, usage bar, devices list, recent activity).
- **Actions**: Change role / Reset password / Revoke sessions / Disable / Delete.

### P6-ADM-05 — Roles management — `SHOULD`

- **Where**: `/admin/roles`.
- **Behavior**: read‑only matrix of role × capability. Custom roles deferred to v3.

### P6-ADM-06 — Access codes — `MUST`

- **Where**: `/admin/access-codes`.
- **Layout**: list of currently active codes with code / role / TTL / uses / created. Generate new (modal).
- **Revoke** = delete code; future redeems fail.

### P6-ADM-07 — Sessions — `MUST`

- **Where**: `/admin/sessions`.
- **Table cols**: User / Device / IP / Scope (LAN/WAN) / Last seen / Trusted / Actions.
- **Bulk** = revoke all WAN sessions, revoke for user X.

### P6-ADM-08 — Trusted devices — `MUST`

- **Where**: `/admin/devices`.
- **Cards**: device name + last seen + IP + user + Revoke.

### P6-ADM-09 — Files (admin lens) — `SHOULD`

- **Where**: `/admin/files`.
- **Purpose**: cross‑user file table. Owner can audit any file's path, size, owner, public status.
- **Actions**: Force‑private, Delete, Reveal in Drive.

### P6-ADM-10 — Public links — `MUST`

- **Where**: `/admin/public-links`.
- **Same as Sharing center (P7‑SHR‑05)** but with cross‑user scope.

### P6-ADM-11 — Storage — `MUST`

- **Where**: `/admin/storage`.
- **Sections**:
  - Volumes (each: mount path, capacity bar by type, smart status, mount/unmount).
  - Quota per user (table with usage bars).
  - Largest folders / files (top 20).

### P6-ADM-12 — Backups — `MUST`

- **Where**: `/admin/backups`.
- **Sections**:
  - Schedule (daily / weekly / off) + retention (last N snapshots).
  - Target (local volume / external USB / S3‑compatible URL).
  - History table (when, size, duration, status).
- **Actions**: Run now, Restore from snapshot (opens explorer of the snapshot's contents).

### P6-ADM-13 — Maintenance — `MUST`

- **Where**: `/admin/maintenance`.
- **Cards** (each = title + description + button):
  - Clear temp uploads
  - Cleanup expired sessions
  - Rebuild search index
  - Vacuum database
  - Rotate logs
  - Run mode toggle (Sandbox / Full) — guarded behind owner password.

### P6-ADM-14 — Safe console — `MUST`

- **Where**: `/admin/safe-console`.
- **Layout**: 360px command list (left) + run panel (right).
- **Command list** entries: name (mono, color by risk) / category pill / risk pill (low/medium/high) / description / ETA.
- **Run panel**:
  - Top: name + risk + category + Run button.
  - Description prose.
  - Args inputs (typed: enum dropdown / int / string / path picker).
  - Output console (terminal‑styled, monospace, timestamps in dim, status symbols in brand colors).
  - Run history table.
- **Behavior**:
  - Owner enters args → Run → confirm modal for medium/high risk → command executes server‑side.
  - Output streams via SSE/WS, ANSI color preserved.
  - Done → exit code shown, summary line, append to Activity log.
- **Initial command set**: `integrity.check`, `session.revoke`, `backup.run`, `cleanup.tmp`, `cleanup.sessions`, `reindex.search`, `service.restart`, `key.rotate`.
- **Acceptance**: no command outputs raw shell escape sequences that could break the page; no command can be added by users.

### P6-ADM-15 — Activity log — `MUST`

- **Where**: `/admin/activity`.
- **Table cols**: Timestamp / User / Action (pill) / Detail / `⋮`.
- **Filters**: user, action category, time range, severity.
- **Export**: CSV / JSON, scoped to current filter.
- **Retention**: 90 days minimum.

---

# P7 — Sharing

> Public links that the user controls. **Budget: 1 week.**

### P7-SHR-01 — Share modal — `MUST`

- **Trigger**: `⌘E` on selected file(s), or right‑click → Share.
- **Modal** (380px):
  - File chip at top (glyph + name + path).
  - Visibility segmented control: Private / Code / Public.
  - Link row: `tssp.local/s/<code>` + Copy.
  - Expiry dropdown: Never / 1h / 24h / 7d / 30d.
  - Download toggle: Allow / View only.
  - QR card (P7‑SHR‑02).
  - Bottom note: "All shares are logged in Admin → Activity".
- **States**:
  - Private (default for new files) → link section dimmed.
  - Code → reveals "Set access code" input (6+ chars).
  - Public → link active.
- **Acceptance**: changing visibility takes effect immediately; viewer page reflects change.

### P7-SHR-02 — QR card — `MUST`

- **Where**: bottom of share modal; also standalone via `Show QR` button.
- **Visual**: 80×80px QR (white card), label "Scan to open", actions: Save PNG / Send via WhatsApp / Print.
- **Encoding**: same URL as Copy link, no tracking.

### P7-SHR-03 — Public viewer page — `MUST`

- **Route**: `/s/:code` (no shell, no dock).
- **Layout**:
  - Centered minimal header: small brand mark + "Shared from tssp".
  - File header: glyph + filename + size + "Shared by Juan".
  - Preview (same lens as P3‑DRV‑10, read‑only).
  - Footer: Download button (if allowed) + expiry note ("Expires in 6 days") + "Report this content" link.
- **Auth**: only required if visibility is "Code".
- **No telemetry**: no analytics scripts, no tracking pixels. View count is a server‑side increment, period.
- **Acceptance**: page loads fully without JS (server‑rendered) so it works on any device.

### P7-SHR-04 — Expiry — `MUST`

- **Behavior**: server checks expiry on every request. Expired link → 410 Gone with "This link expired — ask the owner for a fresh one."
- **Pre‑expiry**: 24h before expiry, owner gets a notification "3 public links expire tomorrow" with quick‑renew button.

### P7-SHR-05 — Sharing center — `MUST`

- **Route**: `/drive/sharing` (Drive sub‑section).
- **Layout**:
  - 4 stat tiles (Public links / Total views / Expiring soon / Bandwidth).
  - Table of every shared item: file / link / views / expires / status / actions (QR, Share, Revoke).
- **Bulk**: select N → revoke all selected.

### P7-SHR-06 — Revoke — `MUST`

- **Trigger**: per‑link or bulk.
- **Behavior**: one click. Confirm dialog only if N>5. Revoking dead‑links the URL immediately.

### P7-SHR-07 — WhatsApp / Telegram share hook — `COULD`

- **Where**: share modal `Send via …` button.
- **Behavior**: deep‑links the platform's share intent with the URL pre‑filled. No API integration in v2; pure URL handoff.

---

# P8 — Mobile

> iOS and Android parity. **Budget: 1 week (mostly responsive tuning).**

### P8-MOB-01 — Mobile launcher — `MUST`

- **Layout**: greeting hero + search + system status card (3 rings) + recent files (2‑col) + recent notes list.
- **Dock**: bottom inset, 5 squircles 42px, same active indicator pattern.

### P8-MOB-02 — Mobile Drive — `MUST`

- **Behavior**:
  - Tap folder → push view.
  - Long‑press card → action sheet (Preview / Share / Download / Rename / Trash).
  - Multi‑select via top‑bar "Select" or 2‑finger tap.
  - Pull‑to‑refresh.

### P8-MOB-03 — Mobile Notes editor — `MUST`

- **Layout**: title + meta + canvas + sticky keyboard toolbar.
- **Toolbar**: `+ / H / • / ☑ / </> / 📷 / /`.

### P8-MOB-04 — Mobile palette — `MUST`

- **Trigger**: tap search field.
- **Layout**: full‑width modal sliding from top; same filter chips + grouped results.

### P8-MOB-05 — Responsive breakpoints — `MUST`

| Width | Behavior |
|---|---|
| ≥1440 | desktop, 3 panes |
| 1024–1439 | desktop, right pane collapses on demand |
| 768–1023 | tablet, sidebars collapse to hamburger |
| <768 | phone, single column, dock 42px |

### P8-MOB-06 — Drag‑upload fallback — `MUST`

- iOS Safari has no drag‑and‑drop. Upload button opens the native picker; multi‑select supported. Pasting an image still works.

### P8-MOB-07 — Offline behavior — `SHOULD`

- **State**: shell renders cached app shell + local data; banner "You're offline — uploads queue locally" pink dot in TopBar.
- **Queue**: pending uploads stored in IndexedDB, retried when LAN returns.

---

# P9 — System

> Backups, integrity, alerts, multi‑volume, telemetry stance.

### P9-SYS-01 — Backups (engine) — `MUST`

- **Strategy**: snapshot‑based, daily by default, kept 7 days. Implementation: filesystem snapshots (btrfs/zfs if available, else rsync to a target).
- **UI**: P6‑ADM‑12.

### P9-SYS-02 — Integrity check — `MUST`

- **What**: walk every file, recompute SHA‑256, compare to stored hash.
- **UI**: `integrity.check` in safe console + auto‑run weekly.
- **Mismatch handling**: file is moved to `/quarantine/` and surfaced in Admin → Files with a `Quarantined` pill.

### P9-SYS-03 — Alerts — `MUST`

- **Sources**: disk >80% / backup overdue 2d / failed sign‑in 5x in 1h / public link expiring 24h / integrity mismatch / volume offline.
- **Surface**:
  - Launcher → Alerts card (top‑right).
  - Notifications bell.
  - Mobile push (if user opts in).
- **Severity**: info (blue) / warn (orange) / danger (red).

### P9-SYS-04 — Multi‑volume — `SHOULD`

- **Behavior**: Pi can have multiple drives. Each is a "volume" mounted under `/mnt/<id>`. Drive root visually groups by volume (chips at top).
- **Behavior**: users don't choose where files go; system spreads them automatically. Power users can force a volume per‑folder.

### P9-SYS-05 — No‑telemetry stance — `MUST`

- **Rule**: tssp ships with **zero** outbound analytics, error reporting, or update pings *enabled by default*. There may be a single opt‑in toggle in Settings → About → "Help improve tssp" (off by default).
- **Acceptance**: a network monitor on the Pi at idle shows zero outbound connections.

---

# P10 — Polish

### P10-POL-01 — Motion vocabulary — `MUST`

- **Durations**: 80ms (taps), 200ms (sheets, modals), 350ms (page transitions). Easing: `cubic-bezier(0.22, 1, 0.36, 1)`.
- **Dock hover**: lift 4px @ 160ms.
- **Dock active**: lift 6px + scale 1.06 @ 200ms.
- **Modal**: backdrop blur fade 200ms + content scale 0.97→1 + fade.
- **Toast**: slide up + fade 180ms.
- **Skeleton shimmer**: 1.6s linear infinite.
- **No bounce**, no springs that wobble, no parallax. Calm.

### P10-POL-02 — Empty / loading / error baseline — `MUST`

- **Every** route and pane must define its empty, loading, and error states. PR checklist enforces this.

### P10-POL-03 — Accessibility — `MUST`

- All interactive elements reachable by keyboard.
- Focus rings: 2px `var(--blue)` with `box-shadow: 0 0 0 4px rgba(110,168,255,.2)`.
- Color contrast: text on bg ≥ 4.5:1; UI text ≥ 3:1.
- All icons have `aria-label` or are decorative + `aria-hidden`.
- Reduced motion: respect `prefers-reduced-motion`; replace transitions with instant changes.
- Screen reader: command palette announces "X results, use arrow keys".

### P10-POL-04 — Performance — `MUST`

- First paint <1s on 1440p desktop over LAN; <2s on 4G.
- Bundle: per‑route splitting; first bundle ≤200kB gzipped.
- Virtualized lists for >100 items.
- Server‑side thumbnail cache with HTTP cache headers.

### P10-POL-05 — Internationalization — `COULD`

- **Strings**: all in a single dictionary, keyed. v2 ships English. Spanish + French ready for v2.1.
- **RTL**: not yet, but no `text-align: left` hard‑coded — use `start`.

### P10-POL-06 — Print stylesheet — `COULD`

- Notes and Drive previews print cleanly. Hide chrome (TopBar, Dock, sidebars) when printing.

---

# Cross‑cutting UX rules (final, don't bend these)

1. **Dock is permanent.** Every screen shows it. ESC always returns to the active app's root.
2. **Command palette is law.** Every action reachable via `⌘K`. If it's not in the palette, it doesn't exist.
3. **Local before WAN.** Show LAN status first. Say "your Pi", not "the cloud".
4. **Safe console, never raw.** Every operation is a typed, audited command with declared risk.
5. **Sharing is loud, revoking is one click.** Every public link wears a status pill.
6. **Empty states are an opportunity.** Templates, hints, keyboard tips. Never just a sad icon.
7. **Errors propose a fix.** "Out of disk" → "Open Volumes". Never just "Try later".
8. **Mobile is parity, not subset.** Same apps, same dock, same palette.
9. **Brand accents earn their pixels.** Green = OK / Drive. Pink = pinned / shared. Orange = warn / queued. Violet = system / safe console. Blue = interaction.
10. **Animation is mechanical, not decorative.** Nothing wows; everything feels physical.

---

# Component inventory (must exist before P3 starts)

`TopBar`, `Dock`, `AppIcon`, `Logo`, `CommandPalette`, `FileGlyph`, `DriveSide`, `DriveCard`, `UploadQueue`, `ContextMenu`, `ShareModal`, `QRCard`, `NoteCard`, `SlashMenu`, `Block(callout|check|code|h2|list|q|table|embed|link)`, `Outline`, `ExplorerRow`, `Tab`, `Minimap`, `StatusBar`, `RunPanel`, `AdminSide`, `BigStat`, `Ring`, `Bar`, `Pill`, `StatusDot`, `Kbd`, `Btn`, `Card`, `StateCard`, `Toast`, `Modal`, `Sheet`, `Tooltip`, `MobileDock`.

---

# Suggested execution order for the coding agent

1. P0 in full (no exceptions; everything depends on tokens + primitives).
2. P1‑SHL‑01, 02, 03, 05, 06 (TopBar / Dock / Palette / Settings / Shortcuts).
3. P2 in full.
4. P3 in order; ship grid before list, list before preview, preview before lenses.
5. P4 in order; ship home before editor, editor before slash menu polish.
6. P5 in order; ship explorer + editor before find / preview.
7. P6 in order; overview first, safe console last (highest review bar).
8. P7 in full.
9. P8 — apply responsive breakpoints, ship per‑screen.
10. P9 + P10 in parallel.

---

# Definitions

- **Pi** — the Orange Pi device running tssp.
- **LAN** — the user's local network.
- **WAN** — anywhere outside the LAN, opt‑in only.
- **Owner** — singular, all‑powerful user. There is exactly one.
- **Quarantine** — isolation path for files with bad hashes.
- **Sandbox** — the default run mode where no arbitrary code executes.

End of spec. Build it in order; ship in phases; everything reachable via `⌘K`.
