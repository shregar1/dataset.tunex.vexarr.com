#!/usr/bin/env bash
set -euo pipefail

BENCH="${1:-end_to_end}"

command -v cargo-flamegraph >/dev/null 2>&1 || {
  echo "cargo-flamegraph is required. Install with: cargo install flamegraph" >&2
  exit 1
}

cargo flamegraph --bench "${BENCH}"
