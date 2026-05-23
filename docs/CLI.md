# CLI Reference

The CLI command surface is defined in Rust with Clap and is the source for help
text and completion generation.

## Global Flags

- `--json`
- `--quiet`
- `--verbose`
- `--no-color`
- `--host <HOST>`
- `--port <PORT>`

## Default Upload

```sh
tssp [OPTIONS] <FILE>...
tssp -r <FOLDER>
tssp -a
```

Implemented for regular files:

- `--tag <NAME>`
- `--pin`
- `--rename <NEW_NAME>`

The command streams the local file to `POST /api/v1/files`, prints the assigned
id, byte size, duration, throughput, returned location, and whether the daemon
deduplicated the upload. `--json` emits one JSON object per uploaded file.

`-r, --recursive`, `-a, --all`, and `--parallel <N>` are parsed but not wired
yet.

## Commands

```text
send
receive
paste
copy
pull
list
last
today
search
tag
untag
pin
unpin
pins
remove
info
status
init
config
completions
```

The default upload action, `status`, `list`, `last`, `search`, `info`,
exact-id `pull`, `tag`, `untag`, `pin`, `unpin`, `pins`, and `remove` are
wired to the daemon. Other command execution is still being wired to backend
services. Parsing and shell completion generation are implemented for the full
command surface.

## `tssp list`

```sh
tssp list
tssp list --limit 25
tssp list --tag Docs --limit 25
tssp --json list
```

Calls `/api/v1/files` and prints recent files in daemon upload order. The current
implementation supports `--limit` and a single `--tag` filter. Additional tag,
MIME, time, pinned, sort, and cursor filters are parsed but return a usage error
until the daemon supports them.

## `tssp last`

```sh
tssp last
tssp last 20
```

Calls `/api/v1/files?limit=<count>` using the command count as the page size.

## `tssp search`

```sh
tssp search report
tssp search report --limit 10
tssp search report --tag Docs
```

Calls `/api/v1/search?q=<query>`. The current implementation applies `--tag`
matching case-insensitively against the daemon result set and truncates the
final output with `--limit`. Empty queries, empty tag filters, and `--limit 0`
return exit code `2`.

## `tssp info`

```sh
tssp info <id>
tssp --json info <id>
```

Calls `/api/v1/files/{id}` and prints metadata for one file. Missing files return
exit code `6`; malformed ids return exit code `2`.

## `tssp pull`

```sh
tssp pull <id>
tssp pull <id> --output ./downloaded.bin
tssp pull <id> --overwrite
```

Downloads file content from `/api/v1/files/{id}/content`. The current
implementation treats the argument as an exact file id. If `--output` is a
directory, the daemon-provided filename is used inside that directory. Existing
files are not overwritten unless `--overwrite` is supplied.

## `tssp remove`

```sh
tssp remove <id> --yes
tssp --json remove <id> --yes
```

Calls `DELETE /api/v1/files/{id}`. The command requires `--yes` when stdin is
not a terminal, which keeps scripts deterministic and prevents accidental
interactive prompts in pipelines. A successful idempotent delete exits `0`;
malformed ids return exit code `2`; daemon `5xx` responses return exit code `5`.

## `tssp tag`

```sh
tssp tag <id> Docs
tssp --json tag <id> Docs Family
```

Calls `POST /api/v1/files/{id}/tags` with a JSON array of tag names. The command
is idempotent: adding an existing tag succeeds and reports `0` changes for that
association. Missing files return exit code `6`; invalid ids or tags return exit
code `2`.

## `tssp untag`

```sh
tssp untag <id> Docs
tssp --json untag <id> Docs Family
```

Calls `DELETE /api/v1/files/{id}/tags/{tag}` once per tag argument and reports
the total changed associations. Removing an absent tag association succeeds with
`0` changes. Missing files return exit code `6`; invalid ids or tags return exit
code `2`.

## `tssp pin`

```sh
tssp pin <id>
tssp pin <id> --position 1
```

Calls `PUT /api/v1/files/{id}/pin`. A successful pin exits `0`; malformed ids
return exit code `2`; missing files return exit code `6`; daemon `5xx`
responses return exit code `5`.

## `tssp unpin`

```sh
tssp unpin <id>
```

Calls `DELETE /api/v1/files/{id}/pin` with the same exit-code contract as
`tssp pin`.

## `tssp pins`

```sh
tssp pins list
tssp pins reorder <id-1> <id-2>
```

`list` calls `GET /api/v1/pins` and prints pin order. `reorder` calls
`POST /api/v1/pins/reorder` with a JSON body containing the new ordered id
list.

## `tssp status`

```sh
tssp --host 127.0.0.1 --port 8421 status
tssp --json status
```

Calls `/api/v1/status` with a 5 second connect timeout and 60 second total
timeout. Network failures return exit code `4`; daemon `5xx` responses return
exit code `5`.

## Completions

```sh
tssp completions bash
tssp completions zsh
tssp completions fish
tssp completions powershell
```
