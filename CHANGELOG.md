# Changelog

All notable changes to this project are documented here.

The format follows Keep a Changelog, and this project uses Semantic Versioning.

## [0.1.0] - Unreleased

### Added

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
- Minimal Axum daemon with health, readiness, status, and web fallback routes.
- Daemon startup wiring for data directory creation, SQLite metadata, blob
  storage initialization, and repository-backed status counts.
- Initial project documentation and CI workflow.
