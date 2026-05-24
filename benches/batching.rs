use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicUsize, Ordering},
};

use bytes::Bytes;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tunex_core::types::RawSample;
use tunex_dataset::{BatchBuilder, BatchSampler, PrefetchBatches};

const SAMPLE_COUNT: usize = 65_536;
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

struct CountingAllocator;

// SAFETY: This allocator delegates every operation to the system allocator and
// only increments an atomic counter before allocation.
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

fn samples() -> Vec<RawSample> {
    (0..SAMPLE_COUNT)
        .map(|index| RawSample::with_id(index.to_string(), Bytes::from_static(b"hello world")))
        .collect()
}

fn bench_batching(c: &mut Criterion) {
    let input = samples();
    let bytes = u64::try_from(SAMPLE_COUNT * b"hello world".len()).expect("bytes should fit u64");
    let mut group = c.benchmark_group("batching");
    group.throughput(Throughput::Bytes(bytes));

    group.bench_function("single_thread_batching", |b| {
        b.iter(|| {
            let mut builder = BatchBuilder::with_capacity(1024);
            let mut batches = 0_usize;
            for sample in input.iter().cloned() {
                builder.push(sample);
                if builder.len() == 1024 {
                    batches = batches.saturating_add(1);
                    black_box(builder.finish());
                }
            }
            black_box((batches, builder.finish()));
        });
    });

    group.bench_function("prefetch_latency", |b| {
        b.iter(|| {
            let mut prefetch = PrefetchBatches::spawn(input.clone(), BatchSampler::fixed(1024), 4);
            let mut batches = 0_usize;
            while let Some(batch) = prefetch.next_batch().expect("batch should read") {
                batches = batches.saturating_add(1);
                black_box(batch);
            }
            black_box((batches, prefetch.join().expect("worker should join")));
        });
    });

    group.bench_function("memory_usage_allocation_counter", |b| {
        b.iter(|| {
            ALLOCATIONS.store(0, Ordering::Relaxed);
            let mut builder = BatchBuilder::with_capacity(1024);
            for sample in input.iter().take(1024).cloned() {
                builder.push(sample);
            }
            let batch = builder.finish().expect("batch should build");
            black_box((batch.bytes, ALLOCATIONS.load(Ordering::Relaxed)));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_batching);
criterion_main!(benches);
