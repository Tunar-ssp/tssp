# Changelog

All notable changes to this project are documented here.

The format follows Keep a Changelog, and this project uses Semantic Versioning.

## [0.1.0] - Unreleased

### Added

- Structural split of `tsspd` HTTP layer (`router`, `state`, `http_tests`,
  `web/assets`, `runner`) and `tssp-app` upload tests module for maintainability.
- Full-page note editor, search filter bar (kind, tag, MIME, visibility, pinned),
  and workspace execution disclaimer in the web dashboard.
- CLI `tssp admin users` and `tssp admin devices` subcommands aligned with admin APIs.

### Changed

- Web notes use a dedicated editor view instead of a modal dialog.

- Initial Rust workspace with pinned toolchain and strict lint configuration.
- Domain validation for hashes, filenames, tags, pagination, timestamps, and QR
  transfer sessions.
- Port traits for storage, metadata, clocks, and file id generation.
- Upload application service with cleanup when metadata commit fails.
- Filesystem blob adapter with streaming BLAKE3 hashing, fanout storage paths,
  deduplication, and cleanup.
- SQLite metadata adapter with embedded migrations, WAL setup, integrity checks,
  file/tag persistence, status counts, and conflict mapping.
- System adapter for UTC timestamps, UUIDv7 file ids, and random 128-bit
  session tokens.
- Clap-based CLI command surface and completion generation.
- Functional `tssp status` command using reqwest/rustls with pretty and JSON
  output plus network/server exit-code mapping.
- Minimal Axum daemon with health, readiness, status, and web fallback routes.
- Daemon startup wiring for data directory creation, SQLite metadata, blob
  storage initialization, and repository-backed status counts.
- HTTP single-file upload endpoint backed by the application upload service,
  filesystem blob storage, SQLite metadata, and content-hash deduplication.
- HTTP recent-file list endpoint backed by SQLite metadata ordering and bounded
  limit validation.
- HTTP single-file metadata endpoint for `/api/v1/files/{id}`.
- HTTP file content endpoint for `/api/v1/files/{id}/content`, including
  download headers, single byte-range support, invalid-range handling, and
  `410 Gone` for metadata records whose blobs are missing.
- Functional default `tssp <file>` upload command that streams regular files to
  the daemon and reports id, size, deduplication, duration, and throughput.
- Functional `tssp list` and `tssp last` commands for recent-file listing.
- Functional `tssp info` command for single-file metadata lookup.
- Functional exact-id `tssp pull` command that downloads file content, protects
  existing files by default, supports `--output`, and honors daemon filenames.
- HTTP `DELETE /api/v1/files/{id}` endpoint with idempotency headers,
  final-reference blob cleanup, and SQLite-backed metadata removal.
- Functional `tssp remove <id> --yes` command with stable exit-code mapping and
  JSON/human output.
- HTTP tag listing and idempotent file tag add/remove endpoints backed by
  SQLite tag counts and join-table cleanup.
- Functional `tssp tag` and `tssp untag` commands with stable exit-code mapping
  and JSON/human output.
- Workspace CRUD stabilization with validated names/languages/body sizes,
  collision-resistant ids, owner-preserving admin updates, admin delete support,
  and web create/edit/delete actions.
- Unified search improvements with server-side limit/tag/type/kind/pinned/
  visibility filters, deterministic ranking, SQLite indexes for bounded fuzzy
  candidates, owner-scoped workspace matches, and CLI query parameter
  forwarding.
- Web dashboard upgraded with object selection, bulk actions, object details,
  media previews, public-link copy controls, full-page note editing, admin
  user/device/file controls, and a corrected modular asset cache.
- Initial project documentation and CI workflow.
