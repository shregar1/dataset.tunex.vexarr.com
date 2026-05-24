# Contributing

TuneX Dataset changes should keep the data path deterministic, bounded, and
measurable.

Before opening a pull request:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --all-features --no-deps
cargo bench --no-run
```

Guidelines:

- Prefer batch-level APIs over per-record work in hot paths.
- Keep queues bounded and memory growth explicit.
- Avoid deep cloning payload data.
- Add tests for contracts and edge cases.
- Add or update benchmarks for performance-sensitive changes.
- Document any unsafe code with a `SAFETY:` comment.
