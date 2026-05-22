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

## Web Fallback

`GET /` and non-API paths return the embedded placeholder web shell with:

- `Content-Type: text/html; charset=utf-8`
- `X-Content-Type-Options: nosniff`
- `Content-Security-Policy: default-src 'self'; connect-src 'self'; style-src 'self' 'unsafe-inline'`
