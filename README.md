# TuneX Dataset

High-performance Rust dataset loading engine for TuneX.

This crate provides dataset input contracts and implementations for JSONL,
memory-mapped JSONL, compressed JSONL, Arrow/Parquet batches, deterministic
sampling, distributed sharding, filtering, manifests, resume cursors, metrics,
and benchmark tooling.

## Status

`0.1.0` is the first unstable dataset module release. APIs are intended for the
TuneX v0.1 ecosystem and may change before `1.0.0`.

## Quick Start

```rust
use tunex_dataset::prelude::*;

let config = DatasetConfig::new("./data/train.jsonl");
let mut dataset = JsonlDataset::from_config(config)?;
let batch = dataset.next_batch()?;
# Ok::<(), tunex_dataset::DatasetError>(())
```

## Features

- Buffered JSONL reader
- Memory-mapped JSONL reader
- SIMD-optimized JSONL parsing
- zstd and lz4 streaming input
- Parallel bounded pipeline
- Arrow/Parquet reader contracts
- Batch generation and prefetch
- Deterministic shuffle and sampling
- Distributed sharding contracts
- Resume cursor snapshots
- Dataset filters and quality rules
- Dataset manifests and fingerprints
- Criterion benchmark suite

## Examples

Run examples with:

```bash
cargo run --example read_jsonl
cargo run --example read_mmap_jsonl
cargo run --example read_compressed
cargo run --example parallel_batches
cargo run --example shuffle_dataset
cargo run --example generate_manifest
cargo run --example plugin_register
```

## Quality Gates

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --all-features --no-deps
cargo bench --no-run
```

Security gates:

```bash
cargo audit
cargo deny check
```

## License

Licensed under either MIT or Apache-2.0, at your option.
