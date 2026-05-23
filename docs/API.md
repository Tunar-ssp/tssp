# HTTP API Reference

This document currently reflects the implemented daemon foundation. The full
target API is defined in [ROADMAP.md](ROADMAP.md).

## `GET /healthz`

Liveness probe.

Response:

```text
ok
```

Status: `200 OK`

## `GET /readyz`

Readiness probe.

Response:

```text
ready
```

Status: `200 OK`

## `POST /api/v1/files`

Uploads one file as `multipart/form-data`.

Fields:

- `file`: required exactly once. The multipart filename is preserved in
  metadata. The part `Content-Type` is stored as the MIME type; missing content
  type defaults to `application/octet-stream`.
- `tag` or `tags`: optional, repeatable initial tags.
- `pin`: optional initial pin flag. Accepted true values are `true`, `1`,
  `yes`, `on`, or an empty field. Accepted false values are `false`, `0`, `no`,
  and `off`. Matching is case-insensitive.
- `destination` or `destination_hint`: optional reserved logical destination
  hint. The current implementation accepts the field and ignores it until
  logical paths are added.

Example:

```sh
curl -F 'tag=Docs' \
  -F 'pin=true' \
  -F 'file=@note.txt;type=text/plain' \
  http://127.0.0.1:8421/api/v1/files
```

Created response:

```json
{
  "schema_version": 1,
  "id": "019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f",
  "name": "note.txt",
  "size_bytes": 12,
  "content_hash": "6caeccdad8d0e6ff73e98a68b77cc62a0e1871f4fb18c6c8e1c12e4f3da10827",
  "mime_type": "text/plain",
  "uploaded_at": 1779494400,
  "tags": ["Docs"],
  "pinned": true
}
```

Status: `201 Created`

Headers:

- `Location: /api/v1/files/{id}`
- `x-tssp-deduplicated: false`

Uploading identical bytes again returns the oldest existing file record for that
content hash.

Status: `200 OK`

Headers:

- `Location: /api/v1/files/{id}`
- `x-tssp-deduplicated: true`

Error responses use the standard error shape. Current upload errors include:

- `400 Bad Request` for missing file fields, multiple file fields, unknown
  multipart fields, malformed multipart data, or invalid metadata.
- `409 Conflict` for metadata conflicts.
- `503 Service Unavailable` when the metadata store is busy or the upload
  service is unavailable.
- `507 Insufficient Storage` when storage reports insufficient free space.
- `500 Internal Server Error` for unexpected server-side failures.

## `GET /api/v1/files`

Lists recent files in descending upload order.

Query parameters:

- `limit`: optional page size. Defaults to `50`; maximum is `500`.
- `tag`: optional tag filter. Matches one normalized tag key.

Example:

```sh
curl 'http://127.0.0.1:8421/api/v1/files?limit=10'
curl 'http://127.0.0.1:8421/api/v1/files?limit=10&tag=Docs'
```

Response:

```json
{
  "schema_version": 1,
  "files": [
    {
      "schema_version": 1,
      "id": "019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f",
      "name": "note.txt",
      "size_bytes": 12,
      "content_hash": "6caeccdad8d0e6ff73e98a68b77cc62a0e1871f4fb18c6c8e1c12e4f3da10827",
      "mime_type": "text/plain",
      "uploaded_at": 1779494400,
      "tags": ["Docs"],
      "pinned": true
    }
  ]
}
```

Status: `200 OK`

Errors:

- `400 Bad Request` when `limit` is `0` or greater than `500`.
- `400 Bad Request` when `tag` is malformed.
- `500 Internal Server Error` when metadata listing fails.

## `GET /api/v1/files/{id}`

Returns metadata for one file id.

Example:

```sh
curl http://127.0.0.1:8421/api/v1/files/019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f
```

Response:

```json
{
  "schema_version": 1,
  "id": "019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f",
  "name": "note.txt",
  "size_bytes": 12,
  "content_hash": "6caeccdad8d0e6ff73e98a68b77cc62a0e1871f4fb18c6c8e1c12e4f3da10827",
  "mime_type": "text/plain",
  "uploaded_at": 1779494400,
  "tags": ["Docs"],
  "pinned": true
}
```

Status: `200 OK`

Errors:

- `400 Bad Request` when the id is malformed.
- `404 Not Found` when the file id is not present in metadata.
- `500 Internal Server Error` when metadata lookup fails.

## `GET /api/v1/files/{id}/content`

Returns stored file bytes.

Query parameters:

- `disposition`: optional. `attachment` is the default; `inline` requests an
  inline `Content-Disposition`.

Supported request headers:

- `Range: bytes=<start>-<end>`
- `Range: bytes=<start>-`
- `Range: bytes=-<suffix-length>`

Example:

```sh
curl -OJ http://127.0.0.1:8421/api/v1/files/019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f/content
curl -H 'Range: bytes=0-1023' \
  http://127.0.0.1:8421/api/v1/files/019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f/content
```

Success headers include:

- `Content-Type`
- `Content-Length`
- `Content-Disposition`
- `Accept-Ranges: bytes`
- `ETag`
- `Last-Modified`
- `Content-Range` for `206 Partial Content`

Status:

- `200 OK` for a full response.
- `206 Partial Content` for a valid byte range.

Errors:

- `400 Bad Request` when the id or disposition is malformed.
- `404 Not Found` when the file id is not present in metadata.
- `410 Gone` when metadata exists but the blob is missing.
- `416 Range Not Satisfiable` when the requested byte range is invalid.
- `500 Internal Server Error` when metadata or blob reads fail.

## `DELETE /api/v1/files/{id}`

Deletes one logical file record. The operation is idempotent: deleting a file
that is already absent still returns success and marks the response as already
gone.

Example:

```sh
curl -X DELETE http://127.0.0.1:8421/api/v1/files/019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f
```

Status: `204 No Content`

Headers:

- `x-tssp-already-gone: false` when a record was deleted.
- `x-tssp-already-gone: true` when the id was already absent.
- `x-tssp-blob-cleaned: true` when the deleted record was the final metadata
  reference and the content-addressed blob was removed.
- `x-tssp-blob-cleaned: false` when bytes are still shared or no record existed.

Errors:

- `400 Bad Request` when the id is malformed.
- `503 Service Unavailable` when the metadata store is busy or the delete
  service is unavailable.
- `500 Internal Server Error` when metadata deletion or final blob cleanup fails.

## `GET /api/v1/tags`

Lists all tags that are attached to at least one file.

Example:

```sh
curl http://127.0.0.1:8421/api/v1/tags
```

Response:

```json
{
  "schema_version": 1,
  "tags": [
    {
      "name": "Docs",
      "file_count": 3
    }
  ]
}
```

Status: `200 OK`

Errors:

- `503 Service Unavailable` when the tag service is unavailable.
- `500 Internal Server Error` when metadata listing fails.

## `POST /api/v1/files/{id}/tags`

Adds tags to one file idempotently. The request body is a JSON array of tag
strings.

Example:

```sh
curl -X POST \
  -H 'Content-Type: application/json' \
  -d '["Docs","Family"]' \
  http://127.0.0.1:8421/api/v1/files/019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f/tags
```

Response:

```json
{
  "schema_version": 1,
  "changed_count": 1
}
```

Status: `200 OK`

Errors:

- `400 Bad Request` when the id is malformed, the body is empty, or a tag is
  invalid.
- `404 Not Found` when the file id is not present in metadata.
- `503 Service Unavailable` when the metadata store is busy or the tag service
  is unavailable.
- `500 Internal Server Error` when metadata mutation fails.

## `DELETE /api/v1/files/{id}/tags/{tag}`

Removes a tag from one file idempotently. Removing a tag association that is
already absent returns success with `changed_count: 0`.

Example:

```sh
curl -X DELETE \
  http://127.0.0.1:8421/api/v1/files/019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f/tags/Docs
```

Response:

```json
{
  "schema_version": 1,
  "changed_count": 1
}
```

Status: `200 OK`

Errors:

- `400 Bad Request` when the id or tag is malformed.
- `404 Not Found` when the file id is not present in metadata.
- `503 Service Unavailable` when the metadata store is busy or the tag service
  is unavailable.
- `500 Internal Server Error` when metadata mutation fails.

## `GET /api/v1/pins`

Lists pinned files in pin order.

Example:

```sh
curl http://127.0.0.1:8421/api/v1/pins
```

Response:

```json
{
  "schema_version": 1,
  "files": [
    {
      "schema_version": 1,
      "id": "019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f",
      "name": "note.txt",
      "size_bytes": 12,
      "content_hash": "6caeccdad8d0e6ff73e98a68b77cc62a0e1871f4fb18c6c8e1c12e4f3da10827",
      "mime_type": "text/plain",
      "uploaded_at": 1779494400,
      "tags": ["Docs"],
      "pinned": true
    }
  ]
}
```

Status: `200 OK`

Errors:

- `503 Service Unavailable` when the metadata store is busy or the pin service
  is unavailable.
- `500 Internal Server Error` when pin listing fails.

## `PUT /api/v1/files/{id}/pin`

Pins a file. The request body is optional. When present it may include a JSON
object with `position`.

Example:

```sh
curl -X PUT http://127.0.0.1:8421/api/v1/files/019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f/pin
curl -X PUT \
  -H 'Content-Type: application/json' \
  -d '{"position":1}' \
  http://127.0.0.1:8421/api/v1/files/019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f/pin
```

Response:

```json
{
  "schema_version": 1,
  "changed": true
}
```

Status: `200 OK`

Errors:

- `400 Bad Request` when the id or JSON body is malformed.
- `404 Not Found` when the file id is not present in metadata.
- `503 Service Unavailable` when the metadata store is busy or the pin service
  is unavailable.
- `500 Internal Server Error` when the pin mutation fails.

## `DELETE /api/v1/files/{id}/pin`

Unpins one file.

Example:

```sh
curl -X DELETE \
  http://127.0.0.1:8421/api/v1/files/019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f/pin
```

Response:

```json
{
  "schema_version": 1,
  "changed": true
}
```

Status: `200 OK`

Errors:

- `400 Bad Request` when the id is malformed.
- `404 Not Found` when the file id is not present in metadata.
- `503 Service Unavailable` when the metadata store is busy or the pin service
  is unavailable.
- `500 Internal Server Error` when the unpin mutation fails.

## `POST /api/v1/pins/reorder`

Reorders currently pinned files.

Example:

```sh
curl -X POST \
  -H 'Content-Type: application/json' \
  -d '{"ids":["019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f"]}' \
  http://127.0.0.1:8421/api/v1/pins/reorder
```

Response:

```json
{
  "schema_version": 1
}
```

Status: `200 OK`

Errors:

- `400 Bad Request` when the body is malformed.
- `404 Not Found` when any supplied file id is not present in the current pin
  set.
- `503 Service Unavailable` when the metadata store is busy or the pin service
  is unavailable.
- `500 Internal Server Error` when the reorder fails.

## `GET /api/v1/search`

Runs full-text search against filenames and indexed tags.

Query parameters:

- `q`: required search string.

Example:

```sh
curl 'http://127.0.0.1:8421/api/v1/search?q=report'
```

Response:

```json
{
  "schema_version": 1,
  "files": [
    {
      "schema_version": 1,
      "id": "019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f",
      "name": "report.pdf",
      "size_bytes": 12,
      "content_hash": "6caeccdad8d0e6ff73e98a68b77cc62a0e1871f4fb18c6c8e1c12e4f3da10827",
      "mime_type": "application/pdf",
      "uploaded_at": 1779494400,
      "tags": ["Docs"],
      "pinned": false
    }
  ]
}
```

Status: `200 OK`

Errors:

- `400 Bad Request` when `q` is empty.
- `500 Internal Server Error` when search fails.

## `GET /api/v1/status`

Returns daemon status.

Example:

```json
{
  "schema_version": 1,
  "version": "0.1.0",
  "status": "ok",
  "uptime_seconds": 12,
  "file_count": 0,
  "tag_count": 0,
  "pinned_count": 0,
  "recent_upload_count_24h": 0
}
```

The count fields are read from the metadata repository. If metadata is
unavailable, the endpoint returns `503 Service Unavailable` with the standard
error shape:

```json
{
  "error": {
    "code": "metadata_unavailable",
    "message": "metadata database is unavailable"
  }
}
```

## `PATCH /api/v1/files/{id}`

Renames a file or updates its metadata.

Request body:

```json
{
  "name": "new-filename.txt"
}
```

Response:

```json
{
  "schema_version": 1,
  "file": {
    "id": "019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f",
    "name": "new-filename.txt",
    "size_bytes": 12,
    "mime_type": "text/plain",
    "tags": ["Docs"],
    "uploaded_at": 1779494400,
    "pinned": true
  }
}
```

Status: `200 OK`

Errors:

- `400 Bad Request` when the id or name is malformed.
- `404 Not Found` when the file id does not exist.
- `500 Internal Server Error` when metadata update fails.

## `POST /api/v1/sessions/send`

Creates a send session for sharing a file via a time-limited transfer token.

Request body:

```json
{
  "file_id": "019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f",
  "ttl_seconds": 3600
}
```

Response:

```json
{
  "schema_version": 1,
  "token": "aaaaaaaaaaaaaaaaaaaaaa",
  "kind": "send",
  "created_at": 1779494400,
  "expires_at": 1779498000,
  "source_file": "019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f"
}
```

Status: `201 Created`

Errors:

- `400 Bad Request` when parameters are invalid.
- `404 Not Found` when the file does not exist.
- `503 Service Unavailable` when the session store is unavailable.

## `POST /api/v1/sessions/receive`

Creates a receive session for accepting a file upload via a time-limited transfer token.

Request body:

```json
{
  "ttl_seconds": 3600
}
```

Response:

```json
{
  "schema_version": 1,
  "token": "aaaaaaaaaaaaaaaaaaaaaa",
  "kind": "receive",
  "created_at": 1779494400,
  "expires_at": 1779498000
}
```

Status: `201 Created`

Errors:

- `400 Bad Request` when parameters are invalid.
- `503 Service Unavailable` when the session store is unavailable.

## `GET /api/v1/sessions/{token}`

Retrieves a session by token.

Response:

```json
{
  "schema_version": 1,
  "token": "aaaaaaaaaaaaaaaaaaaaaa",
  "kind": "send",
  "created_at": 1779494400,
  "expires_at": 1779498000,
  "source_file": "019b2f0f-3b1f-7c20-bd79-7bb4f46e7f9f"
}
```

Status: `200 OK`

Errors:

- `400 Bad Request` when the token is malformed.
- `404 Not Found` when the session does not exist or has expired.
- `503 Service Unavailable` when the session store is unavailable.

## `POST /api/v1/sessions/{token}/use`

Marks a session as used (consumed or accessed).

Response:

```json
{
  "schema_version": 1
}
```

Status: `204 No Content` or `200 OK`

Errors:

- `400 Bad Request` when the token is malformed.
- `404 Not Found` when the session does not exist.
- `503 Service Unavailable` when the session store is unavailable.

## Web Fallback

`GET /` and non-API paths return the embedded placeholder web shell with:

- `Content-Type: text/html; charset=utf-8`
- `X-Content-Type-Options: nosniff`
- `Content-Security-Policy: default-src 'self'; connect-src 'self'; style-src 'self' 'unsafe-inline'`
