# Installation

The project is currently built from source.

## Requirements

- Rust toolchain pinned by `rust-toolchain.toml`.
- Cargo.

## Build

```sh
cargo build --workspace --release
```

## Run Daemon

```sh
./target/release/tsspd --bind 127.0.0.1 --port 8421
```

## Run CLI

```sh
./target/release/tssp --help
```

Release packages, systemd units, and cross-compiled artifacts will be added as
the implementation reaches the distribution workstream.
