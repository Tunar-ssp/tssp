# Architecture

TSSP is built as a Rust workspace with a hexagonal architecture. Domain rules sit
at the center, application services orchestrate use cases, and adapters provide
filesystem, database, network, terminal, and operating-system behavior.

```text
CLI / HTTP delivery
       |
Application services
       |
Ports
       |
Domain core
```

Infrastructure points inward through traits. The domain does not know about
HTTP, SQLite, filesystems, clocks, or operating-system APIs.

## Workspace Layout

```text
crates/
  tssp-domain       Pure validation and value objects.
  tssp-ports        Traits for storage, metadata, clocks, and IDs.
  tssp-app          Backend use-case orchestration.
  tssp-adapter-fs   Content-addressed filesystem blob storage.
  tssp-adapter-sqlite SQLite metadata repository and migrations.
  tssp-adapter-system System clock, UUIDv7 ids, and random tokens.
  tssp-cli-core     CLI exit codes and output policy.
  tssp              CLI binary and Clap command definitions.
  tsspd             Daemon binary and HTTP foundation.
```

## Domain Core

`tssp-domain` owns rules that must be consistent across every delivery mechanism:

- BLAKE3 hash shape.
- User-facing filename preservation and safe storage-component derivation.
- Tag normalization and case-insensitive lookup keys.
- Cursor page-size limits.
- UTC timestamp sanity checks.
- QR session token shape and single-use state transitions.

The domain crate has no I/O. It uses deterministic functions and typed errors so
tests can cover edge cases directly.

## Ports

`tssp-ports` defines the external capabilities required by application services:

- `Clock`
- `IdGenerator`
- `BlobStore`
- `FileRepository`
- `SessionRepository`

These traits are intentionally small. Concrete adapters can be added without
changing the use-case code.

## Application Services

`tssp-app` coordinates domain values and ports. `UploadService` streams bytes
into `BlobStore`, creates metadata through `FileRepository`, and asks storage to
clean up if metadata commit fails. When storage reports duplicate bytes, the
service queries metadata by content hash and returns the oldest existing record
instead of creating a second logical record.

`DeleteFileService` removes one logical metadata record and receives the
remaining content-reference count from the repository transaction. It removes
the content-addressed blob only when no other file record still references the
same hash.

`TagService` lists tag summaries and performs idempotent add/remove mutations.
It normalizes raw tag strings through the domain layer before asking the
repository to update the join table.

`SessionService` manages transfer sessions for file sharing. It creates send
sessions (bound to a specific file) and receive sessions (accepting anonymous
uploads), both with caller-provided TTL. It retrieves sessions by token,
marks sessions as used, and cleans up expired sessions based on the current time.

The application layer owns ordering and consistency rules. It does not contain
HTTP status codes, SQLite statements, or terminal output.

## Filesystem Blob Adapter

`tssp-adapter-fs` implements `BlobStore`. It streams bytes into a temporary file
while computing BLAKE3, fsyncs completed bytes, then renames the file into a
content-addressed fanout layout:

```text
data/
  tmp/
  blobs/
    ab/
      cd/
        abcdef...
```

The fanout keeps large deployments away from single-directory scaling cliffs.
Re-uploading the same bytes returns the same storage handle and marks the write
as deduplicated.

## SQLite Metadata Adapter

`tssp-adapter-sqlite` implements `FileRepository` and `SessionRepository`.
Opening a repository applies the required pragmas, runs `PRAGMA integrity_check`,
and executes embedded forward-only schema setup. The current schema stores files,
tags, file/tag joins, sessions, schema migration records, and an FTS5 table
reserved for search.

All writes use prepared statements through `rusqlite`. The adapter maps busy and
locked database states to `RepositoryError::Busy`, duplicate constraints to
`RepositoryError::Conflict`, and corrupt domain values read from the database to
typed operation failures.

The file repository exposes aggregate counts for status reporting: total files,
distinct tags, pinned files, and recent uploads since a caller-provided UTC
cutoff.

The session repository manages transfer tokens with expiration tracking, session
kind (send or receive), and optional source/destination file references.

## System Adapter

`tssp-adapter-system` contains small host integrations:

- `SystemClock` converts the host clock to the bounded UTC timestamp domain
  type.
- `UuidV7FileIdGenerator` creates sortable opaque file ids.
- `RandomSessionTokenGenerator` uses operating-system randomness and unpadded
  URL-safe base64 to produce 128-bit QR/session tokens.

## Delivery

The CLI command tree lives in `tssp` and is defined with Clap. This gives a
single source for parsing, help text, completions, and future man page
generation.

The daemon foundation lives in `tsspd`. It currently exposes:

- `GET /healthz`
- `GET /readyz`
- `GET /api/v1/status`
- `POST /api/v1/files`
- `GET /api/v1/files`
- `PATCH /api/v1/files/{id}`
- `DELETE /api/v1/files/{id}`
- `GET /api/v1/files/{id}`
- `GET /api/v1/files/{id}/content`
- `POST /api/v1/files/{id}/tags`
- `DELETE /api/v1/files/{id}/tags/{tag}`
- `GET /api/v1/tags`
- `POST /api/v1/sessions/send`
- `POST /api/v1/sessions/receive`
- `GET /api/v1/sessions/{token}`
- `POST /api/v1/sessions/{token}/use`
- `GET /<any-non-api-path>` as a placeholder web shell

The binary initializes the data directory, opens the SQLite metadata repository,
verifies blob storage can be initialized, and wires repository-backed status
counts and upload storage into the HTTP state. The upload route stages multipart
file content to a temporary file while reading chunks, then hands a blocking
`Read` stream to the application upload service. The list route reads recent
metadata through the repository-backed status provider, and the single-file
route uses the same provider for id lookups. The content route combines metadata
lookup with the read-side blob port, applies download headers, and handles
single-range byte requests. The delete route calls the application delete
service and reports idempotent deletes through headers. HTTP upload and delete
storage mutations share a daemon-local async mutex so a final-reference blob
cleanup cannot race with an in-flight upload that is trying to reuse the same
content-addressed bytes. The tag routes use the application tag service and keep
the metadata join-table rules out of HTTP handlers. Other handlers will call
application services as capabilities are added.

## Dependency Direction

```text
tssp-domain
  ^
  |
tssp-ports
  ^
  |
tssp-app
```

`tssp` and `tsspd` sit at the edge. They may depend on application services and
adapters, but inner crates never depend on delivery crates.

## Reliability Decisions

Uploads are designed around streaming. The application service receives a
`Read` stream and passes it to storage without buffering the whole file. The
filesystem adapter writes to a temporary file, fsyncs completed bytes, and
renames into a content-addressed path.

Downloads validate metadata before opening storage and return `410 Gone` when a
metadata record points at a missing blob. Range requests are validated before
reading bytes and rejected with `416 Range Not Satisfiable` plus
`Content-Range: bytes */<size>` when the requested range cannot exist.

The metadata commit happens only after blob storage succeeds. If metadata commit
fails, the app calls `cleanup_unreferenced` on storage. This creates an explicit
recovery point for crash-safe adapter implementations.

Deletes commit metadata first, then clean the blob only after the repository has
reported zero remaining references for that hash. Deleting an already absent id
is a success and surfaces `x-tssp-already-gone: true` to clients.

## Security Boundary

The v1 daemon is local-network trusted. Tokens are still high entropy and
single-use, but there is no user authentication in v1. The daemon sets
`X-Content-Type-Options: nosniff` and a restrictive CSP for the embedded web
fallback.

## Testing Strategy

Each layer gets tests at its own boundary:

- Domain tests cover validation and state transitions without I/O.
- Application tests use in-memory fakes for ports.
- Filesystem adapter tests use temporary directories and real file operations.
- SQLite adapter tests use in-memory and temporary on-disk databases.
- System adapter tests verify timestamp bounds, UUIDv7 id shape, and token
  uniqueness over a small deterministic sample size.
- Daemon tests exercise HTTP routes through the router.
- CLI tests verify the command tree and completion generation.

As adapters are added, integration tests will use temporary directories and real
SQLite databases.
