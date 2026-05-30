#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

printf '\nfrontend-build:\n'
(
  cd frontend
  npm run build
)

printf '\nbackend-build:\n'
cargo build -p tsspd

printf '\nbackend:\n'
cargo run -p tsspd
