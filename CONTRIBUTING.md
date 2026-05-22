# Contributing

## Development Setup

Install the Rust toolchain selected by `rust-toolchain.toml`, then run:

```sh
cargo build --workspace
cargo test --workspace
```

## Quality Gates

Before submitting a change, run:

```sh
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Code Style

- Keep domain logic free of I/O.
- Add typed errors at library boundaries.
- Avoid `unwrap` and `expect` outside tests.
- Keep functions focused and named for behavior.
- Add module docs for every crate module.
- Prefer small traits at infrastructure boundaries.

## Tests

New behavior should have tests at the lowest useful layer. Domain rules should be
tested in `tssp-domain`; orchestration should be tested with fake ports in
`tssp-app`; HTTP behavior should be tested through the Axum router.
