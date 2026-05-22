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

Implemented parser flags:

- `--tag <NAME>`
- `--pin`
- `--rename <NEW_NAME>`
- `--parallel <N>`
- `-r, --recursive <FOLDER>`
- `-a, --all`

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
remove
info
status
init
config
completions
```

`status` is wired to the daemon. Other command execution is still being wired to
backend services. Parsing and shell completion generation are implemented for the
full command surface.

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
