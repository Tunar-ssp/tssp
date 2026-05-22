# TSSP

TSSP is a self-hosted transfer and storage system for local networks. It is
designed for an Orange Pi class device and ships as:

- `tsspd`: the backend daemon.
- `tssp`: the command-line client.
- `tssp-web`: an embedded static web dashboard.

The v1 trust model is local-network only. There is no authentication yet, so do
not expose the daemon to the public internet.

## Current Status

This repository is in active implementation against [ROADMAP.md](ROADMAP.md).
The foundation currently includes:

- A pinned Rust workspace.
- Pure domain validation for hashes, filenames, tags, pagination, timestamps,
  and QR transfer sessions.
- Ports for storage, metadata, clocks, and ID generation.
- An application upload service with cleanup-on-commit-failure behavior.
- A Clap-based CLI command surface with shell completion generation.
- A minimal Axum daemon exposing `/healthz`, `/readyz`, `/api/v1/status`, and an
  embedded placeholder web shell, backed by real metadata status counts when
  started from the binary.
- A filesystem blob adapter that streams uploads into content-addressed BLAKE3
  storage with fanout directories and deduplication.
- A SQLite metadata adapter with embedded migrations, WAL configuration,
  integrity checking, prepared statements, tag joins, and duplicate-id conflict
  handling, plus aggregate counts for status reporting.
- A system adapter for UTC time, UUIDv7 file IDs, and 128-bit URL-safe session
  tokens.

## Build

```sh
cargo build --workspace
```

## Test

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

## Run The Daemon

```sh
cargo run -p tsspd -- --bind 127.0.0.1 --port 8421 --data-dir data
```

Then check:

```sh
curl http://127.0.0.1:8421/healthz
curl http://127.0.0.1:8421/api/v1/status
```

## CLI

```sh
cargo run -p tssp -- --help
cargo run -p tssp -- completions bash
```

Most command handlers are not connected to the daemon yet. The command surface is
present so generated documentation, completions, and future handlers share one
source of truth.

## Documentation

- [ROADMAP.md](ROADMAP.md): authoritative specification.
- [ARCHITECTURE.md](ARCHITECTURE.md): architecture and layering.
- [API.md](API.md): current HTTP API reference.
- [CLI.md](CLI.md): current CLI reference.
- [CONFIGURATION.md](CONFIGURATION.md): config keys and defaults.
- [INSTALL.md](INSTALL.md): installation notes.
- [CONTRIBUTING.md](CONTRIBUTING.md): development workflow.
- [SECURITY.md](SECURITY.md): v1 security model.
- [CHANGELOG.md](CHANGELOG.md): release history.
