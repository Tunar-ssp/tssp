# Development Guide

## Prerequisites

- Rust (stable, via rustup)
- Node.js (for frontend syntax checking)

## Common Commands

```sh
# Build everything
cargo build --workspace

# Run all tests
cargo test --workspace

# Lint (errors on warnings)
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --check

# Format (apply)
cargo fmt --all

# Run the daemon against a local data directory
cargo run -p tsspd -- --data-dir /tmp/tssp-dev

# Validate config without starting
cargo run -p tsspd -- --data-dir /tmp/tssp-dev --check-config
```

## Project Layout

```
crates/
  tssp-domain/         Value objects and validation (no I/O)
  tssp-ports/          Port traits: FileRepository, BlobStore, Clock, etc.
  tssp-app/            Application services (orchestrate domain + ports)
  tssp-adapter-sqlite/ SQLite implementation of FileRepository + NoteRepository
    src/
      lib.rs           Struct + impl FileRepository + error type + tests
      connection.rs    Pragma setup and integrity check
      cursor.rs        Pagination cursor encoding/decoding
      migrations.rs    Schema migrations v1–v10
      query.rs         Row mapping and SQL helper functions
      notes.rs         NoteRepository impl
      sessions.rs      SessionRepository impl
  tssp-adapter-fs/     Filesystem blob store
  tssp-adapter-system/ System clock, UUIDv7 IDs, random tokens
  tssp-cli-core/       CLI exit codes and output formatting
  tssp/                CLI binary (upload, download, search, etc.)
  tsspd/               HTTP daemon binary
    src/
      lib.rs           Public re-exports
      router.rs        Route registration
      state.rs         HttpState builder
      upload.rs        File upload handlers
      search.rs        Search endpoint
      workspaces.rs    Workspace CRUD
      http_tests/
        mod.rs         Integration tests (real HTTP stack via oneshot)
        common.rs      Shared mock providers and request builders
```

## Running Tests

Tests use real SQLite (in-memory) and real filesystem storage. No database
setup needed — `cargo test --workspace` is sufficient.

HTTP integration tests in `tsspd/src/http_tests/mod.rs` exercise the full axum
router via `tower::ServiceExt::oneshot` (no TCP socket required).

## Adding a Migration

1. Pick the next version number (`SELECT MAX(version) FROM schema_migrations` to find current).
2. Add a `migrate_<feature>` function in `crates/tssp-adapter-sqlite/src/migrations.rs`.
3. Guard it with `migration_applied(connection, N)?`.
4. Call it from `run_migrations`.

## Architecture Invariants

- Domain (`tssp-domain`) has zero dependencies on adapters.
- Ports (`tssp-ports`) defines traits only — no implementations.
- Application services (`tssp-app`) depend on ports, not adapters.
- HTTP handlers (`tsspd`) are thin: parse request → call application service → serialize response.
