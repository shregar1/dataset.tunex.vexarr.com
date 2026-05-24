# Dataset Benchmarks

TuneX dataset performance claims must be reproducible. Record the hardware,
dataset shape, compression, worker count, and feature flags for every result.

## Suite

Run all Rust benchmarks:

```bash
scripts/benchmark_all.sh
```

Run one benchmark:

```bash
cargo bench --bench simd_parse
```

Available benches:

- `jsonl_parse`: JSONL parser throughput and allocations/sample
- `mmap_read`: mmap random access vs buffered non-mmap reader
- `simd_parse`: SIMD parser vs `serde_json::Value`
- `compression`: plain, zstd, and lz4 read throughput
- `batching`: batch build, prefetch latency, allocation counter
- `shuffle`: deterministic shuffle and rank-aware sharding overhead
- `cache`: manifest/cache lookup hit ratio baseline
- `distributed`: distributed assignment and partition planning
- `end_to_end`: JSONL read, filter, batch, stats, manifest path

## Required Metadata

Record these fields with each published result:

- CPU model, core count, and frequency governor
- RAM size and storage device type
- OS, kernel, Rust version, and target CPU flags
- Dataset size, sample count, and average text length
- Compression type and compression level
- Worker count and queue capacities
- SIMD enabled or disabled
- mmap enabled or disabled

## External Baselines

Compare against:

- Hugging Face `datasets`
- Python line-by-line `json` loader
- Rust `serde_json::Value`
- Non-mmap TuneX reader
- Single-thread parser path

The Rust suite includes direct baselines for `serde_json::Value`, non-mmap
reading, and single-process paths. Python and HF comparisons should use the same
fixture files generated for the Rust runs and must report wall time, peak RSS,
samples/sec, and MB/sec.

## Profiling

Generate a flamegraph:

```bash
scripts/flamegraph.sh end_to_end
```

Run Linux perf:

```bash
scripts/perf.sh simd_parse
```

Use the profiling output to validate claims such as:

- JSONL parser throughput
- compression throughput
- queue latency and stall time
- cache hit ratio
- parallel scaling efficiency
- resume and distributed assignment overhead

## Claim Format

Use concrete, reproducible claims:

- `N samples/sec` and `N MB/sec` on named hardware
- `X allocations/sample` for parser and batch paths
- `X% cache hit ratio` for warm cache runs
- `Xx faster than Python JSONL loader` on the same fixture
- `Xx lower peak RSS` compared to the same baseline
