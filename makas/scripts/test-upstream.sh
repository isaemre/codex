#!/usr/bin/env bash
# Runs upstream tests with the Makas lock compiled out (K-28).
#
# Upstream tests talk to local mock servers and expect the `openai` provider, so they cannot run
# against a locked build. A separate target directory avoids rebuilding the workspace every time
# the switch changes. build.rs refuses this switch for release builds.
#
# Usage: makas/scripts/test-upstream.sh -p codex-http-client [nextest args]
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${repo_root}/codex-rs"

export MAKAS_UNLOCK_FOR_UPSTREAM_TESTS=1
export CARGO_TARGET_DIR="${repo_root}/codex-rs/target/upstream-tests"
exec just test "$@"
