use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicUsize, Ordering},
};

use bytes::Bytes;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tunex_dataset::{JsonlDatasetConfig, SimdJsonlParser};

const LINE: &[u8] = br#"{"id":"sample-1","text":"hello world from tunex"}"#;
const LINE_COUNT: usize = 16_384;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

struct CountingAllocator;

// SAFETY: This allocator delegates every allocation operation to the standard
// system allocator and only increments an atomic counter before allocation.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: The layout is forwarded unchanged to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: The pointer and layout are forwarded unchanged to the system
        // allocator that originally allocated the memory.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn bench_jsonl_parse(c: &mut Criterion) {
    let parser = SimdJsonlParser::new(&JsonlDatasetConfig::default());
    let mut lines = Vec::with_capacity(LINE_COUNT);
    for _ in 0..LINE_COUNT {
        lines.push(Bytes::copy_from_slice(LINE));
    }
    let bytes = u64::try_from(LINE.len() * LINE_COUNT).expect("benchmark size should fit u64");

    let mut group = c.benchmark_group("jsonl_parse");
    group.throughput(Throughput::Bytes(bytes));
    group.bench_function("simd_fast_path_lines_per_sec", |b| {
        b.iter(|| {
            for (index, line) in lines.iter().enumerate() {
                let line_number = u64::try_from(index + 1).expect("line should fit u64");
                let sample = parser
                    .parse_bytes(line_number, line.clone())
                    .expect("line should parse");
                black_box(sample);
            }
        });
    });
    group.bench_function("simd_batch_parse", |b| {
        b.iter(|| {
            let batch = parser
                .parse_batch(
                    lines
                        .iter()
                        .enumerate()
                        .map(|(index, line)| (u64::try_from(index + 1).unwrap(), line.clone())),
                )
                .expect("batch should parse");
            black_box(batch);
        });
    });
    group.bench_function("allocations_per_sample", |b| {
        b.iter(|| {
            ALLOCATIONS.store(0, Ordering::Relaxed);
            for (index, line) in lines.iter().enumerate() {
                let line_number = u64::try_from(index + 1).expect("line should fit u64");
                let sample = parser
                    .parse_bytes(line_number, line.clone())
                    .expect("line should parse");
                black_box(sample);
            }
            let allocations = ALLOCATIONS.load(Ordering::Relaxed);
            black_box(allocations as f64 / LINE_COUNT as f64);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_jsonl_parse);
criterion_main!(benches);
