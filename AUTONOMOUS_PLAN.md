# AUTONOMOUS_PLAN

This file tracks the autonomous execution loop against `docs/ROADMAP.md`.
Tasks are intentionally small, verifiable, and ordered by dependency.

## Checkpoints

- [x] Checkpoint the verified baseline, refresh coverage artifacts, and record the autonomous plan in git

## Query And Retrieval Surface

- [x] Implement full `GET /api/v1/files` query model in the daemon: multi-tag filtering, MIME prefix, name substring, time bounds, pinned-only, sorting, and cursor pagination
- [x] Wire CLI `tssp list` to the full query model, including repeated tags, sort selection, pinned filter, and cursor paging
- [x] Implement CLI `tssp today` on top of the list query model
- [x] Extend `tssp pull` to resolve `id|name`, warn on ambiguous filename matches, and honor `--all`

## Upload And Metadata Surface

- [x] Implement `POST /api/v1/files/batch` with per-file outcomes and tests
- [ ] Wire CLI multi-file upload reporting to batch semantics where practical
READ AFTER LINE 55 also remove this line after yoy read this line 
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
# Autonomous Execution Contract

The agent must continue working until the entire roadmap is completed.

Do not stop after:
- small edits
- partial implementations
- a single completed task
- 1–2 file changes
- intermediate checkpoints
- successful compilation
- partial test success

The default behavior is CONTINUOUS EXECUTION.

The agent should automatically:
- pick the next dependency task
- implement it
- refactor related code if needed
- batch related changes together before validation
- avoid interrupting workflow with excessive testing after every tiny edit
- run verification after meaningful implementation milestones, subsystem completion, or coordinated change sets
- run tests/lints strategically instead of after every single file modification
- fix failures
- continue to the next task
- repeat until the roadmap is fully completed

The agent must not pause to summarize progress unnecessarily.

The agent must not ask for confirmation between roadmap tasks unless:
- credentials are required
- destructive actions are required
- requirements are fundamentally ambiguous
- external services/accounts are needed

Expected behavior:
- work for long uninterrupted execution cycles
- make many coordinated codebase changes
- finish connected subsystems in one run
- continuously repair broken code introduced during implementation
- maintain build stability throughout execution
- avoid inefficient edit → test → edit → test loops for tiny changes
- prefer larger coherent implementation passes before full verification

The stopping condition is:
- all roadmap tasks completed
- tests passing
- verification commands passing
- documentation synchronized
- workspace in shippable state

## Anti-Early-Exit Rules

Do not interpret:
- "good progress"
- "partial completion"
- "MVP state"
- "core functionality works"
- "enough for now"

as completion conditions.

Only stop when the roadmap itself is complete or a hard blocker exists.

## Validation Strategy

Do not run:
- full test suites
- full lint passes
- full verification pipelines

after every 1-file or tiny change.

Instead:
- accumulate meaningful related changes
- complete a logical implementation block
- then run validation

Prefer:
- implementation momentum
- grouped verification
- subsystem-level testing

over constant interruption from excessive validation cycles.
