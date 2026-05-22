# Configuration

## Daemon

Implemented daemon flags:

| Key | Default | Environment | Description |
| --- | --- | --- | --- |
| `bind` | `127.0.0.1` | `TSSPD_BIND` | IP address to bind. |
| `port` | `8421` | `TSSPD_PORT` | TCP port to listen on. |

The daemon also supports `--check-config`, which validates the currently parsed
configuration and exits.

## CLI

Implemented global CLI overrides:

| Key | Environment | Description |
| --- | --- | --- |
| `host` | `TSSP_HOST` | Daemon host override. |
| `port` | `TSSP_PORT` | Daemon port override. |

The full layered configuration model from the roadmap will be added around the
same command and application boundaries.
