#!/usr/bin/env bash
set -euo pipefail

export RUSTFLAGS="${RUSTFLAGS:-}"

cargo bench --bench jsonl_parse
cargo bench --bench mmap_read
cargo bench --bench simd_parse
cargo bench --bench compression
cargo bench --bench batching
cargo bench --bench shuffle
cargo bench --bench cache
cargo bench --bench distributed
cargo bench --bench end_to_end
