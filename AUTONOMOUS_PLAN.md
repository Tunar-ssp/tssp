# AUTONOMOUS_PLAN

This file tracks the autonomous execution loop against `docs/ROADMAP.md`.
Tasks are intentionally small, verifiable, and ordered by dependency.

## Checkpoints

- [x] Checkpoint the verified baseline, refresh coverage artifacts, and record the autonomous plan in git

## Query And Retrieval Surface

- [x] Implement full `GET /api/v1/files` query model in the daemon: multi-tag filtering, MIME prefix, name substring, time bounds, pinned-only, sorting, and cursor pagination
- [ ] Wire CLI `tssp list` to the full query model, including repeated tags, sort selection, pinned filter, and cursor paging
- [ ] Implement CLI `tssp today` on top of the list query model
- [ ] Extend `tssp pull` to resolve `id|name`, warn on ambiguous filename matches, and honor `--all`

## Upload And Metadata Surface

- [ ] Implement `POST /api/v1/files/batch` with per-file outcomes and tests
- [ ] Wire CLI multi-file upload reporting to batch semantics where practical
- [ ] Implement recursive upload planning for `tssp -r` with hidden-file, exclude, dry-run, and parallel controls
- [ ] Implement `tssp -a` as non-recursive current-directory upload
- [ ] Implement `PATCH /api/v1/files/{id}` for rename and logical move metadata

## Sessions And Sharing

- [ ] Add domain and port types for transfer sessions
- [ ] Add SQLite persistence for send and receive sessions
- [ ] Implement session creation/status/public-consumption HTTP endpoints: `/api/v1/sessions/send`, `/api/v1/sessions/receive`, `/api/v1/sessions/{token}`, `/s/{token}`, `/u/{token}`
- [ ] Add session expiry reaping and startup cleanup for stale session state
- [ ] Implement CLI `send`
- [ ] Implement CLI `receive`
- [ ] Implement CLI `copy --share`

## Clipboard And Local UX

- [ ] Implement CLI `paste` for text, image, and file-list clipboard sources
- [ ] Implement CLI `copy <id>` direct download URL clipboard flow
- [ ] Implement CLI `init` first-run configuration flow

## Daemon Lifecycle And Ops

- [ ] Expand daemon status to include storage usage details from the live data directory
- [ ] Harden daemon startup validation and temp-upload cleanup
- [ ] Implement graceful shutdown behavior around in-flight work
- [ ] Add `OPTIONS` handling and CORS behavior for documented API routes
- [ ] Add optional metrics exposure behind configuration

## Documentation And Quality

- [ ] Sync README, API, CLI, architecture, install, and configuration docs with the implemented feature set
- [ ] Raise measured workspace coverage to at least 80%
- [ ] Run the final workspace verification pass: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo tarpaulin --workspace --out Xml --output-dir coverage`
