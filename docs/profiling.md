# Dataset Profiling

TuneX dataset performance work must be backed by measurements. Use the metrics
contracts in `src/metrics` to separate I/O, parsing, batching, queue pressure,
cache behavior, and worker utilization.

## Perf

Use `perf` on Linux for CPU hotspots:

```bash
perf record --call-graph dwarf -- cargo bench
perf report
```

Focus on JSON parsing, decompression, Arrow decoding, and queue stall time.

## Flamegraph

Use flamegraphs to compare implementation changes:

```bash
cargo flamegraph --bench pipeline
```

Check for accidental row-by-row deserialization, excessive allocation, and lock
contention.

## Heap Profiling

Use platform heap profilers such as `heaptrack`, `dhat`, or Instruments on
macOS. Watch for per-record allocation churn and deep clones in batch paths.

## Allocation Tracking

Benchmark allocation-sensitive paths separately:

```bash
cargo bench --bench jsonl_parse
cargo bench --bench batching
```

The target is batch-level allocation reuse, not one allocation per record.

## SIMD Benchmarking

Compare parser fast paths against fallbacks with representative JSONL lines:

```bash
cargo bench --bench jsonl_parse
```

Track lines/sec, MB/sec, malformed-line cost, and escaped-string fallback cost.

## Required Signals

Each dataset backend should report:

- samples/sec and MB/sec
- JSON parse latency
- mmap read latency
- decompression latency
- queue stall time
- batch build latency
- cache hit and miss ratios
- CPU, memory, and worker utilization
