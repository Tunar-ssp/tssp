# TSSP

TSSP is a self-hosted transfer and storage system for local networks. It is
designed for an Orange Pi class device and ships as:

- `tsspd`: the backend daemon.
- `tssp`: the command-line client.
- `tssp-web`: an embedded static web dashboard.

On your LAN, access is open by default. When you expose the daemon on the
internet, set a password so remote clients must authenticate (see below).

## Current Status

This repository is in active implementation against [ROADMAP.md](docs/ROADMAP.md).
The foundation currently includes:

- A pinned Rust workspace.
- Pure domain validation for hashes, filenames, tags, pagination, timestamps,
  and QR transfer sessions.
- Ports for storage, metadata, clocks, and ID generation.
- An application upload service with cleanup-on-commit-failure behavior.
- An application delete service that removes metadata and cleans final-reference
  blobs.
- An application tag service for tag listing and idempotent tag mutation.
- An application pin service for pinning, unpinning, listing, and reordering
  favorites.
- A Clap-based CLI command surface with shell completion generation and wired
  `status`, regular-file upload, `list`, `last`, `search`, `info`, `pull`,
  `tag`, `untag`, `pin`, `unpin`, `pins`, and `remove` commands.
- A minimal Axum daemon exposing `/healthz`, `/readyz`, `/api/v1/status`,
  `POST /api/v1/files`, `GET /api/v1/files`, `GET /api/v1/files/{id}`,
  `GET /api/v1/files/{id}/content`, `DELETE /api/v1/files/{id}`,
  `GET /api/v1/tags`, `POST /api/v1/files/{id}/tags`,
  `DELETE /api/v1/files/{id}/tags/{tag}`, `GET /api/v1/pins`,
  `PUT`/`DELETE /api/v1/files/{id}/pin`, `POST /api/v1/pins/reorder`,
  `GET /api/v1/search`, notes APIs, workspace CRUD, dual-mode auth
  (`/api/v1/auth/*`), and a GCS-style web dashboard at `/`, backed by real
  metadata status counts, tag mutation, pin mutation, search, workspace
  editing, object selection/bulk actions, media previews, admin user/device
  management, and upload/download/delete storage when started from the binary.
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
open http://127.0.0.1:8421/
curl http://127.0.0.1:8421/api/v1/status
curl -F 'tag=Docs' -F 'file=@README.md;type=text/markdown' \
  http://127.0.0.1:8421/api/v1/files
curl -OJ http://127.0.0.1:8421/api/v1/files/<file-id>/content
curl -H 'Content-Type: application/json' -d '["Docs"]' \
  http://127.0.0.1:8421/api/v1/files/<file-id>/tags
curl http://127.0.0.1:8421/api/v1/tags
curl -X DELETE http://127.0.0.1:8421/api/v1/files/<file-id>
```

## Remote authentication

Enable a shared password for clients outside your LAN:

```sh
export TSSPD_AUTH_PASSWORD='your-secret'
cargo run -p tsspd -- --data-dir data
```

Or store a bcrypt hash:

```sh
export TSSPD_AUTH_PASSWORD_HASH='$2b$12$...'
```

CLI login for remote hosts:

```sh
cargo run -p tssp -- --host cloud.example.com login
```

Behind a reverse proxy, pass `--trust-forwarded` (or `TSSPD_TRUST_FORWARDED=1`) so
client IP detection uses `X-Forwarded-For`.

## CLI

```sh
cargo run -p tssp -- --help
cargo run -p tssp -- completions bash
cargo run -p tssp -- --host 127.0.0.1 --port 8421 status
cargo run -p tssp -- --host 127.0.0.1 --port 8421 --tag Docs README.md
cargo run -p tssp -- --host 127.0.0.1 --port 8421 list --limit 25
cargo run -p tssp -- --host 127.0.0.1 --port 8421 search report --tag Docs --limit 10
cargo run -p tssp -- --host 127.0.0.1 --port 8421 info <file-id>
cargo run -p tssp -- --host 127.0.0.1 --port 8421 pull <file-id>
cargo run -p tssp -- --host 127.0.0.1 --port 8421 tag <file-id> Docs
cargo run -p tssp -- --host 127.0.0.1 --port 8421 untag <file-id> Docs
cargo run -p tssp -- --host 127.0.0.1 --port 8421 pin <file-id>
cargo run -p tssp -- --host 127.0.0.1 --port 8421 pins list
cargo run -p tssp -- --host 127.0.0.1 --port 8421 remove <file-id> --yes
```

The default regular-file upload action plus `status`, `list`, `last`, `search`,
`info`, exact-id `pull`, `tag`, `untag`, `pin`, `unpin`, `pins`, and `remove`
are connected to the daemon. The remaining command surface is present so
generated documentation, completions, and future handlers share one source of
truth.

## Documentation

- [ROADMAP.md](docs/ROADMAP.md): authoritative specification.
- [ARCHITECTURE.md](docs/ARCHITECTURE.md): architecture and layering.
- [API.md](docs/API.md): current HTTP API reference.
- [CLI.md](docs/CLI.md): current CLI reference.
- [CONFIGURATION.md](docs/CONFIGURATION.md): config keys and defaults.
- [INSTALL.md](docs/INSTALL.md): installation notes.
- [CONTRIBUTING.md](CONTRIBUTING.md): development workflow.
- [SECURITY.md](docs/SECURITY.md): v1 security model.
- [CHANGELOG.md](docs/CHANGELOG.md): release history.
