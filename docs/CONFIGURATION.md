# Configuration

## Daemon (`tsspd`)

Layered configuration (highest wins): **CLI flags → `TSSPD_*` env → `data-dir/tssp.toml` → built-in defaults**.

### Example `data/tssp.toml`

```toml
bind = "0.0.0.0"
port = 8421
public_url = "https://cloud.example.com"
trust_forwarded = true
mdns = true
metrics = true
web = true
session_ttl_seconds = 86400
storage_reserve_bytes = 524288000   # 500 MiB
storage_reserve_percent = 1
max_upload_bytes = 0                # 0 = unlimited
```

### Keys

| Key | Default | Environment | Description |
| --- | --- | --- | --- |
| `bind` | `127.0.0.1` | `TSSPD_BIND` | Listen address |
| `port` | `8421` | `TSSPD_PORT` | TCP port |
| `data_dir` | `data` | `TSSPD_DATA_DIR` | Metadata + blob storage |
| `public_url` | (derived) | `TSSPD_PUBLIC_URL` | Public base URL for QR/session links |
| `trust_forwarded` | `false` | `TSSPD_TRUST_FORWARDED` | Trust `X-Forwarded-For` for auth |
| `mdns` | `true` | `TSSPD_MDNS` | Advertise `_tssp._tcp.local` |
| `metrics` | `true` | `TSSPD_METRICS` | Expose `/metrics` |
| `web` | `true` | `TSSPD_WEB` | Serve dashboard at `/` |
| `session_ttl_seconds` | `86400` | `TSSPD_SESSION_TTL_SECONDS` | Default session lifetime |
| `storage_reserve_bytes` | `524288000` | `TSSPD_STORAGE_RESERVE_BYTES` | Minimum free disk bytes |
| `storage_reserve_percent` | `1` | `TSSPD_STORAGE_RESERVE_PERCENT` | Minimum free disk percent |
| `max_upload_bytes` | `0` | `TSSPD_MAX_UPLOAD_BYTES` | Per-upload size cap (`0` = none) |

Auth password: `TSSPD_AUTH_PASSWORD` or `TSSPD_AUTH_PASSWORD_HASH` (see README).

Run `tsspd --check-config` to print the effective settings and exit.

## CLI (`tssp`)

Config file: `~/.config/tssp/config.json`

| Key | Environment | Description |
| --- | --- | --- |
| `host` | `TSSP_HOST` | Daemon hostname |
| `port` | `TSSP_PORT` | Daemon port |
| `url` | — | Full base URL override |
| `scheme` | — | `http` or `https` when not using `url` |
| `token` | — | Bearer token for remote access |
| `discovery` | — | Use mDNS when host is default (`true`) |

```sh
tssp config set host tsspd.local
tssp config set url https://cloud.example.com
tssp login
```
