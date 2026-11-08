#!/usr/bin/env bash
# Starts the locked Makas dev build with a fresh, empty CODEX_HOME (K-20), so every run goes
# through the first-run flow: gateway key entry, then model choice (K-30).
#
# Usage: makas/scripts/start-dev.sh [codex args...]
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${repo_root}/codex-rs"

# Incremental caches grow the target directory quickly; the dev loop does not need them.
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"

codex_home="$(mktemp -d)"
trap 'rm -rf "${codex_home}"' EXIT
export CODEX_HOME="${codex_home}"

echo "start-dev: CODEX_HOME=${codex_home} (removed on exit)" >&2
cargo run --bin codex -- "$@"
