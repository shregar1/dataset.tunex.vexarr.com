#!/usr/bin/env bash
set -euo pipefail

BENCH="${1:-end_to_end}"
OUT="${2:-target/perf-${BENCH}.data}"

command -v perf >/dev/null 2>&1 || {
  echo "perf is required and is currently Linux-only for this script." >&2
  exit 1
}

cargo bench --bench "${BENCH}" --no-run
BIN="$(find target/release/deps -maxdepth 1 -type f -perm -111 -name "${BENCH}-*" | head -n 1)"

perf record --call-graph dwarf -o "${OUT}" -- "${BIN}" --bench
perf report -i "${OUT}"
