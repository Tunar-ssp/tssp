# TSSP v2 - Comprehensive Feature Checklist

## File: screens-drive.jsx

### FEATURES:
- Cloud Drive file management system
- Grid view file display (6-column layout)
- List view with table headers and sortable columns
- File preview modal with document rendering
- Upload queue with progress tracking
- Sharing center with public link management
- Context menu for file operations
- Bulk file selection and operations
- File tagging system with color-coded tags
- Pin/favorite files functionality
- File search and filtering
- Breadcrumb navigation for folder hierarchy
- Storage quota display with volume breakdown
- Public link sharing with QR code generation
- File expiry controls (time-based link expiration)
- Bandwidth monitoring for shared files
- Trash/delete recovery system

### DATA STRUCTURES:
- driveTree: hierarchical folder structure with counts
- driveFiles: file metadata (type, name, size, modified, tags, public status, share count)
- Upload queue items: (name, progress%, type, queued status)
- Sharing links: (filename, type, view count, expiry, code, visibility)
- File detail metadata: (hash, size, created/modified dates, path, type)
- Storage stats: (used GB, free GB, by-type breakdown)
- Activity log entries: (icon, action, timestamp)

### INTERACTIONS:
- Click folder to expand/collapse hierarchy
- Click file to select (checkbox with blue highlight)
- Drag files to upload (multiple simultaneous)
- Right-click context menu on files
- Keyboard shortcuts in menu (⌘↓, ⌘E, ⌘C, F2, ⌘⇧M, ⌘D, T)
- Filter by file type (Images, Video, Docs, Code)
- Sort by Modified date
- Switch between grid/list view toggle
- Open file preview by clicking
- Copy share link to clipboard
- Generate and scan QR codes
- Set link expiry duration
- Download files from share sheet
- Revoke public links instantly

### COMPONENTS:
- DriveSide (folder tree sidebar)
- DriveToolbar (view toggle, filters, sort)
- DriveBreadcrumbs (path navigation)
- DriveCard (grid item with thumbnail)
- FileGlyph (file type icon)
- UploadQueue (floating status panel)
- ContextMenu (right-click actions)
- DetailRow (metadata label + value)
- StatTile (stat card with icon, value, subtitle, gradient)
- Card (container component)
- Bar (progress bar, segmented)
- Pill (badge component)
- Btn (button with variants)
- Kbd (keyboard shortcut display)
- TopBar (global header)
- Dock (app launcher)
- QRCodeArt (SVG QR code renderer)

---

## File: screens-notes.jsx

### FEATURES:
- Notes home with card grid layout
- Pinned notes section
- Note folder/workspace organization
- Tag cloud with count badges
- Note search and filter
- Rich text editor with slash menu
- Block-based content (headings, checklists, code, callouts, lists)
- Markdown support
- Code syntax highlighting
- Note outline with linked items
- Autosave with status indicator
- Note duplicate, archive, and share
- Full-text search with highlighting
- Cursor-based collaborative editing indicators
- Template suggestions (callout blocks)
- Sharing controls for individual notes

### DATA STRUCTURES:
- noteCards: (title, body, color, pinned, tags[], date)
- Folders/Workspaces: (name, icon, count, color)
- Tags: (name, count, color-tone)
- Block types: callout, h2, check, list, code, q (question)
- Block metadata: (kind, tone, done status, cursor info, language)
- Slash menu items: (icon, title, description, keyboard shortcut)
- Outline entries: (label, depth, active status)
- Linked items: (icon, title)

### INTERACTIONS:
- Click card to open editor
- New note with ⌘N or button
- Filter by tag (clickable pills)
- Switch grid/list view
- Slash menu with / trigger (e.g., /code, /h1, /h2, /li, /todo, /call, /img, /file, /hr)
- Keyboard shortcuts in editor (specific keys for block insertion)
- Check/uncheck checkboxes
- Click "+" tag to add tags
- Cursor shows collaborator activity
- Quick preview with Space key
- Search with ⌘K
- Sort by Modified date

### COMPONENTS:
- NotesSide (folder tree + tag cloud)
- NoteCard (grid card with color bar)
- SideHead (section heading)
- SideRow (sidebar row with count)
- Block (polymorphic block component - callout, h2, check, list, code, q)
- SlashMenu (floating command palette)
- DetailRow (from Drive, reused)
- Pill (badge)
- Kbd (keyboard display)
- StatusDot (save status indicator)
- Btn (button variants)
- Card (container)
- TopBar
- Dock

---

## File: screens-workspace.jsx

### FEATURES:
- Lightweight IDE/workspace for code editing
- Project explorer with file tree
- Tabs for open files with dirty state indicator
- Code editor with syntax highlighting
- Line numbers and status bar
- Find widget (search in file)
- Minimap for large files
- Live preview pane (component preview)
- Output console/terminal panel
- File status tracking (synced, dirty)
- Run history with execution results
- Safe console with curated commands (integrity check, backup, cleanup, etc.)
- Activity rail with file explorer, search, code, terminal views
- Status bar showing git branch, language, encoding, line endings
- Multiple view modes (Explorer, Terminal, Preview)

### DATA STRUCTURES:
- explorer items: (name, icon, open, depth, dirty, active, color)
- code lines: (line number, tokens with colors: [text, type])
- tabs: (name, type, on, dirty)
- tokenColor map: (keyword, string, type, function, variable, comment, tag)
- status items: (icon, text, color)
- Explorer row metadata: (name, depth, open/close state, icon, dirty, active)

### INTERACTIONS:
- Click folder to expand/collapse
- Click file to open in tab
- New file/folder creation
- Tab switching by clicking
- Close tab with X or Ctrl+W
- Search in file with Find widget (Ctrl+F or Cmd+F)
- Navigate with arrow keys in Find
- Keyboard shortcuts (context-aware)
- Click line to position cursor
- Dirty indicator shows unsaved changes
- Synced status in explorer
- View toggle (Explorer, Search, Code, Terminal, History)
- Output panel shows command results
- Run command from Safe console

### COMPONENTS:
- TopBar (context + breadcrumbs)
- ExplorerRow (file/folder row)
- StatusItem (status bar element)
- Dock (app switcher)
- FileGlyph (file type icon)
- Pill (badge)
- Kbd (keyboard shortcut)
- Btn (buttons)
- Card (panels)
- Icons (chevron, cloud, activity, etc.)

---

## File: screens-admin.jsx

### FEATURES:
- Admin overview dashboard with system health
- Big statistics display (CPU, Memory, Disk, Network)
- Sparkline charts for trending data
- Storage breakdown by file type
- Active sessions monitoring
- User management with roles/permissions
- User invitation with QR codes
- Session revocation
- Device trust management
- Safe console for curated operations
- Integrity check command
- Backup operations
- Database reindex
- Service restart controls
- Key rotation
- Activity log with filtering
- Command execution with auditing
- Run history tracking
- Role-based access control (Owner, Editor, Viewer, API, Public)
- Bandwidth cap monitoring
- Link expiry management

### DATA STRUCTURES:
- adminNav: (group, items with name, icon, badge count)
- BigStat data: (label, value, trend, sparkline values)
- Storage breakdown: (category, color, files count, GB size)
- Sessions: (user, device, location, IP, last login, warn status)
- Users: (name, email, role, last seen, session count, api status, revoked status)
- Safe console commands: (name, group, description, risk level, execution time, category)
- Activity log: (timestamp, user, action type, details, action tone)
- Command args: (label, value, options dropdown)
- Run output lines: (timestamp, message, status indicator, progress)
- Run history: (time, command name, executor, status, output summary)

### INTERACTIONS:
- Click admin nav item to switch section
- Click user row to open detail panel
- Change user role from detail panel
- Revoke user sessions
- Search users by name/email/IP
- Filter by role and status
- Invite new user with code generation
- Copy invitation code or QR
- Search safe console commands
- Click command to select and view details
- Configure command arguments
- Run command with confirmation
- View live command output
- Download run history/logs
- Refresh overview stats
- Run integrity check (read-only operation)
- Execute sandboxed commands
- View activity log with event export

### COMPONENTS:
- AdminSide (sidebar with navigation)
- TopBar (header)
- BigStat (stat card with sparkline)
- StatTile (smaller stat card)
- Card (container panels)
- Bar (progress bar, segmented)
- Ring (circular progress indicator)
- Pill (badge/status)
- StatusDot (connection/health indicator)
- Btn (buttons)
- Kbd (keyboard display)
- Icons (various system icons)
- Table components (rows, headers)
- Dock (app launcher)
- ArgRow (command argument editor)

---

## File: screens-launcher.jsx

### FEATURES:
- Launcher/home screen with greeting
- System status summary card
- Storage usage visualization
- Active alerts display
- Recent files rail
- Recent notes rail with pinned section
- Open workspaces rail with dirty file indicator
- Live activity feed
- Command palette (⌘K) with full-text search
- Search result filtering (All, Files, Notes, Workspaces, Actions, Settings)
- Quick action creation from search (create note from search term)
- Keyboard navigation in palette
- Light theme variant
- Wallpaper background with subtle grid
- Spotlight-style search interface
- Quick action chips (Upload, New note, New workspace, Share link, QR)
- System uptime display
- Network reachability status (LAN indicator)
- Free space counter

### DATA STRUCTURES:
- Recent files: (type, name, size, modified)
- Recent notes: (title, body preview, color, pinned, tags[])
- Workspaces: (name, branch, file count, dirty count, color)
- Activity items: (icon, action text, timestamp, color)
- Search results: (type [file/note/action], title, path/subtitle, highlight match)
- Status summary: (uptime, version, online status)
- Storage breakdown: (category color, count, percentage, size)
- Alerts: (icon, message, tone)

### INTERACTIONS:
- Type in search to filter palette
- Navigate search results with arrow keys
- Press Enter/Return to open selection
- Press ⌘Return to open in new tab
- Switch filter tabs (All, Files, Notes, etc.)
- Click to open recent item
- Click rail header "Open X" link
- Quick action buttons for common tasks
- View system status at a glance
- See activity live feed
- Wallpaper gradient background loads background OS
- Light theme toggle switches color scheme
- Command palette auto-shows 7 results

### COMPONENTS:
- TopBar (simplified header, no context)
- Logo (brandmark)
- AppIcon (app launcher icons)
- RailHead (section header with count and action)
- Card (stat cards, recent item containers)
- Ring (progress circle for CPU/RAM/Disk)
- Bar (storage segmented progress)
- Pill (status badge)
- StatusDot (system health indicator)
- Btn (quick action buttons)
- Kbd (keyboard shortcut display)
- FileGlyph (file type icon)
- SectionHead (search result section)
- Row (search result row)
- Dock (app switcher)
- Icons (search, activity, etc.)

---

## File: screens-ia.jsx

### FEATURES:
- Information architecture visualization
- 4-app dock navigation map
- Cross-cutting features (Command palette, Sharing center, Notifications, Settings)
- User flow diagrams (5 common journeys)
- Empty state designs
- Loading state with skeleton/shimmer
- Error state with recovery suggestions
- Offline state with local queue indicator
- UX rules documentation (10 principles)
- Component inventory listing
- Keyboard shortcut map
- Risk levels for admin commands (low, medium, high)

### DATA STRUCTURES:
- App columns: (icon, name, description, feature list items)
- User flows: (title, steps with icon, label, description)
- State cards: (tone, tag, content)
- UX rules: (number, title, description)
- Keyboard map: (key combination, action description)
- Components list: (component names as strings)
- Cross-cutting features: (icon, label, description, sub-feature list)

### INTERACTIONS:
- View IA tree diagram
- Follow user flow steps
- Click to see different state variants
- Review UX rules and principles
- Reference keyboard shortcuts
- See component inventory
- Understand design system relationships

### COMPONENTS:
- Logo (branding)
- AppIcon (app icons)
- AppColumn (IA column card)
- Card (stat/info card)
- Pill (status badge)
- StatusDot (indicator)
- Btn (action buttons)
- Kbd (keyboard display)
- Icons (various)
- FlowRow (user flow diagram)
- StateCard (state example container)
- TopBar
- Dock

---

## File: screens-brand.jsx

### FEATURES:
- Brand identity guidelines
- Logo lockup (primary and icon versions)
- Wordmark with handwritten style (Caveat font)
- Brand voice and tone guidelines
- App icon set (6 icons - drive, notes, workspace, admin, sharing, settings)
- Color palette system
- Typography scale (9 font sizes)
- Spacing scale (10 spacing values)
- Radii system (7 border radius values)
- Shadow/elevation system (3 shadow levels)
- Component showcase (buttons, inputs, badges, file chips)
- Design token documentation
- Font families (DM Sans, Bricolage Grotesque, JetBrains Mono, Caveat)

### DATA STRUCTURES:
- Logo marks: (eye, mouse, cable - brand colors with roles)
- Color groups: (Surfaces, Text, Brand, Function)
- Type scale: (size px, role name)
- Spacing values: (4px increments from 0-64px)
- Radii values: (4px to 9999px full)
- Shadow definitions: (name, CSS shadow value)
- Button variants: (primary, accent, solid, ghost, danger)
- Input states: (default, focused, error)
- Badge/pill tones: (green, pink, blue, orange, violet, warn, danger, neutral)
- File type glyphs: (image, video, doc, pdf, sheet, code, music, zip, file)

### INTERACTIONS:
- View brand guidelines
- Reference color codes (hex values)
- See typography in action
- View component variants
- Check spacing/sizing guide
- Reference shadow definitions

### COMPONENTS:
- Logo (various sizes, layouts)
- AppIcon (all 6 app icons)
- Card (brand guidelines card)
- Pill (badge showcase)
- Btn (button variants)
- StatusDot (status indicator)
- FileGlyph (file type icons)
- Kbd (keyboard key style)
- Bar (progress bar)
- Ring (progress circle)

---

## File: screens-mobile.jsx

### FEATURES:
- Mobile home screen (iPhone)
- Mobile Drive folder view
- Mobile notes editor
- Mobile command palette
- iOS-style bottom dock (variable height)
- Mobile-optimized grid layout (2 columns)
- Keyboard toolbar for note editing
- Action sheet (floating menu)
- Pinch gestures support (implied)
- Touch-friendly spacing (larger tap targets)
- Mobile file preview with thumbnail
- Status indicators adapted for small screen
- Drag-upload fallback (native file picker on iOS Safari)
- Mobile search/command palette
- Account status and quick stats
- Bottom dock with app launcher (5 apps + home)

### DATA STRUCTURES:
- Mobile recent files: (type, name, size, public status)
- Mobile notes list: (title, color, date)
- Mobile workspace rail: (name, file count)
- Activity feed items: (icon, action, timestamp)
- Mobile dock apps: (id, icon/logo element)
- File action sheet items: (icon, label, tone)

### INTERACTIONS:
- Tap file to open
- Tap folder breadcrumb back navigation
- Tap dock icons to switch apps
- Pinch to zoom preview
- Pull up action sheet (context menu)
- Search with ⌘K or search button
- Type in note editor with keyboard toolbar
- Tap tag pill to filter
- Swipe to navigate (implied)
- Mobile gesture support for common operations

### COMPONENTS:
- MobileWrap (iOS device frame wrapper)
- IOSDevice (hardware frame/notch simulation)
- MobileDock (bottom dock for mobile)
- Logo (home indicator)
- AppIcon (mobile app icons)
- Card (mobile cards)
- Ring (CPU/RAM/Disk stats)
- Pill (filter/status pills)
- StatusDot (status indicator)
- FileGlyph (file type icon)
- Btn (buttons)
- Block (note block components, reused from Notes)
- Kbd (keyboard shortcut)
- TopBar (simplified mobile header)
- Icons (various)
- SectionHead, Row (from Launcher, reused for palette)

---

## File: shell.jsx

### FEATURES:
- Top navigation bar with search and user profile
- Bottom app dock with active indicator
- Card component with optional header/footer
- Floating panels and modals
- Progress bars (linear and segmented)
- Progress rings (circular gauges)
- Status indicators (StatusDot)
- Pill/badge component
- Button component with 5 variants (primary, accent, solid, ghost, danger)
- Keyboard shortcut display (Kbd)
- Icons system integration
- Online/offline status indicator
- Notification bell with badge count
- User profile menu entry

### DATA STRUCTURES:
- TopBar props: (context, deepCrumbs[], online status, onlyCenter flag)
- Dock props: (active app id, size, expanded toggle, trash toggle, settings toggle)
- Card props: (children, style, padding, header, footer, accent color)
- Bar props: (value 0-100, tone, height, OR segments array with pct and color)
- Ring props: (value 0-100, size, tone, label, sub)
- Button props: (kind, icon, size, style, children, iconRight)
- Pill props: (tone, style, children)
- Kbd props: (children/shortcut text, style)

### INTERACTIONS:
- Type in command bar (⌘K to focus)
- Click app in dock to switch
- Click notifications bell to view
- Click user profile to open menu
- Buttons trigger actions
- Pills are read-only status indicators
- Cards can be scrollable panels
- Progress bars show completion
- Rings show system metrics

### COMPONENTS:
- TopBar (global header with search, user, online status)
- Dock (app launcher at bottom)
- Card (reusable container)
- Bar (progress bar, linear or segmented)
- Ring (circular progress gauge)
- StatusDot (small status indicator)
- Pill (status badge)
- Kbd (keyboard shortcut display)
- Btn (button with variants)

---

## File: tokens.css

### FEATURES:
- Complete design token system
- Color palette (surfaces, text, brand accents, functional colors)
- Typography system (3 font families)
- Spacing scale (4pt grid)
- Border radius system
- Shadow/elevation system
- Wallpaper background pattern
- Light theme override
- Utility classes (mono, hand, display, tnum, hair, kbd)
- Scrollbar styling
- CSS variables for theme switching
- Glass morphism values for dock
- Selection highlighting

### DATA STRUCTURES:
- Color tokens: --bg-deep, --bg, --bg-1, --surface, --surface-2, --surface-3, --surface-hi, --hairline, --border, --border-2, --text, --text-2, --muted, --dim, --faint, --green, --green-soft, --pink, --pink-soft, --orange, --orange-soft, --blue, --blue-soft, --cyan, --violet, --success, --warning, --danger
- Glass effects: --dock-glass, --dock-edge, --dock-hi, --shadow-dock, --shadow-card, --shadow-modal
- Spacing scale: --s-0 through --s-10 (0px to 64px)
- Radii: --r-1 through --r-7, --r-full (4px to 9999px)
- Font families: --ff-sans, --ff-display, --ff-mono, --ff-hand
- Font sizes: --fs-12 through --fs-56
- Wallpaper patterns: radial gradients, grid pattern, film grain
- Light theme color overrides (complete palette inversion)

### STYLING RULES:
- Tabular numbers for consistency
- OpenType feature flags (ss01, cv11)
- Antialiased font rendering
- Base line-height 1.45
- Subtle letter-spacing adjustments
- Box model reset
- Custom selection color
- Scrollbar styling with hover states
- Wallpaper with grid overlay and grain
- Light theme CSS classes

---

## CROSS-CUTTING DESIGN PATTERNS

### Navigation & Structure:
- Always-visible dock at bottom (persistent 5-app + settings)
- Top bar with search (⌘K) and user profile
- Breadcrumb navigation for hierarchical content
- Sidebar navigation per app (collapsible folders/tags)
- Command palette as primary navigation aid
- Keyboard shortcuts for quick access

### State Management:
- Empty state designs with actionable suggestions
- Loading state with skeleton/shimmer animation
- Error states with recovery path
- Offline mode with local queue
- Dirty file indicators (orange dot or italics)
- Unsaved changes warnings

### Visual Feedback:
- Selection highlighting (blue tint, checkbox)
- Hover states (subtle background change)
- Active states (strong color accent)
- Status indicators (StatusDot colors)
- Progress visualization (Bar, Ring components)
- Toast/notification system implied

### Interactions:
- Drag-and-drop upload everywhere
- Right-click context menus
- Keyboard shortcuts pervasive (⌘ + key)
- Slash menu for content insertion (/)
- Modal/sheet overlays for focused tasks
- Floating action buttons
- Tab switching and document tabs
- Find-in-file search

### Accessibility:
- High contrast text on dark backgrounds
- Semantic color usage (green=ok, red=error, orange=warn)
- Keyboard navigation throughout
- ARIA-implied semantic structure
- Focus states with visible indicators
- Monospace fonts for technical content

### Performance Indicators:
- Sparkline charts for trends
- Progress bars for upload/download
- CPU, RAM, Disk gauges
- Network bandwidth monitoring
- Storage quota visualization
- Session/device counts
- File/note/workspace counts

---

## SUMMARY STATISTICS

**Total Applications:** 5 (Drive, Notes, Workspace, Admin, Launcher)
**Total Screens:** 15+ (grid, list, preview, sharing, users, console, mobile variants, IA, brand, states)
**Unique Components:** 35+ (reusable UI pieces)
**Color Tokens:** 24 (surfaces, text, brand, functional)
**Keyboard Shortcuts:** 15+
**Font Families:** 4 (Sans, Display, Mono, Hand)
**Font Sizes:** 9 levels
**Spacing Increments:** 10 values (4pt grid)
**Border Radii:** 8 values
**Shadow Levels:** 3 (card, dock, modal)
**Button Variants:** 5 (primary, accent, solid, ghost, danger)
**Pill/Badge Tones:** 8 (neutral, green, pink, blue, orange, violet, warn, danger)
**File Types Supported:** 9+ (image, video, doc, pdf, sheet, code, music, zip, file)
**User Roles:** 4 primary (Owner, Editor, Viewer, API) + Public/Anonymous
**Status Indicators:** 4 (ok/green, warn/orange, error/red, info/blue)

---

Generated: May 25, 2026
TSSP Design System v2.0
