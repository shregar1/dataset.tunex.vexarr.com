use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    time::{SystemTime, UNIX_EPOCH},
};

use tunex_core::{plugin::TunexPlugin, traits::DatasetSource};
use tunex_dataset::{
    DatasetMode, DatasetPlugin, JsonlDatasetConfig, TunexDatasetSource, DATASET_PLUGIN_NAME,
};

struct NoopWake;

impl Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
}

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(NoopWake));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

fn fixture_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("tunex-plugin-{name}-{nanos}.jsonl"))
}

#[test]
fn dataset_plugin_exposes_core_metadata_and_capabilities() {
    let plugin = DatasetPlugin::new();
    let metadata = plugin.metadata();
    let capabilities = plugin
        .capabilities()
        .iter()
        .map(|capability| capability.as_str())
        .collect::<Vec<_>>();

    assert_eq!(metadata.name.as_str(), DATASET_PLUGIN_NAME);
    assert_eq!(metadata.kind, tunex_core::plugin::PluginKind::Dataset);
    assert!(capabilities.contains(&"jsonl"));
    assert!(capabilities.contains(&"mmap"));
    assert!(capabilities.contains(&"simd-json"));
    assert!(capabilities.contains(&"zstd"));
    assert!(capabilities.contains(&"lz4"));
    assert!(capabilities.contains(&"parallel-read"));
    assert!(capabilities.contains(&"estimated-throughput:750-mib-s"));
}

#[test]
fn tunex_dataset_source_implements_core_dataset_source_for_mmap() {
    let path = fixture_path("mmap");
    fs::write(&path, "{\"text\":\"one\"}\n{\"text\":\"two\"}\n").expect("fixture should write");
    let mut config = JsonlDatasetConfig::new(&path);
    config.mode = DatasetMode::Mmap;
    let source = TunexDatasetSource::open(config).expect("source should open");

    let first = block_on(source.next_sample())
        .expect("first read should succeed")
        .expect("first sample should exist");
    let second = block_on(source.next_sample())
        .expect("second read should succeed")
        .expect("second sample should exist");

    assert_eq!(first.payload.as_ref(), b"one");
    assert_eq!(second.payload.as_ref(), b"two");
    assert!(block_on(source.next_sample())
        .expect("eof should succeed")
        .is_none());

    fs::remove_file(path).expect("fixture should remove");
}
