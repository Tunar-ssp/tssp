# TSSP Frontend Migration

This directory is the maintainable replacement for the legacy dashboard under
`crates/tsspd/assets/web/`.

## Why this exists

The legacy app is difficult to evolve because:

- markup, behavior, and styling are spread across many global files
- there is no typed client boundary around the API
- product areas are not isolated into reusable components
- every meaningful change requires hand-coordinating `index.html`, CSS, state,
  and global event listeners

The new frontend fixes that with:

- Svelte component boundaries
- TypeScript API contracts
- a routed app shell
- source code that maps directly to product areas

## Current contract

- `frontend/` is the source tree for the new app
- `npm run build` writes to `crates/tsspd/assets/web-v2/`
- built assets use the `/app-v2/` base path
- the legacy dashboard remains the live bundle until parity is reached
- once parity is good enough, Rust asset serving can be pointed at the new build

## Commands

```sh
npm install
npm run dev
npm run check
npm run build
```

Preview targets:

- Vite dev: `http://127.0.0.1:5173/app-v2/`
- Rust-served built preview: `http://127.0.0.1:8421/app-v2/`

## Source layout

```text
src/
  App.svelte                 route switch + shell mounting
  lib/
    api.ts                   typed HTTP client
    router.ts                hash router + nav model
    stores/                  auth, UI, drive, notes, workspace state
    components/              shell-level UI
    utils/                   shared formatting helpers
  views/
    drive/                   storage product area
    notes/                   knowledge product area
    workspace/               IDE product area
    operations/              admin product area
    public/                  sharing product area
    search/                  search/command product area
```

## Migration rules

1. New frontend work should land here first.
2. Legacy dashboard edits should be limited to hotfixes or migration support.
3. API shape changes must be reflected in `src/lib/api.ts`.
4. Product goals stay in `docs/WEBROADMAP.md`.
5. Architecture and cutover plan stay in `docs/ROADMAPWEB.md`.
