//! Parallel JSONL pipeline worker runtime.

use std::{
    io::{BufRead, BufReader},
    thread::{self, JoinHandle},
    time::Duration,
};

use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use rayon::prelude::*;
use tunex_core::types::RawSample;

use crate::{
    compression::DatasetInputReader,
    jsonl::SimdJsonlParser,
    pipeline::{
        batch::{LineChunk, RawLine, SampleBatch},
        queue::{send_with_backpressure, BoundedQueue},
        scheduler::{ChunkScheduler, PipelineOptions},
    },
    DatasetError, DatasetResult, JsonlDatasetConfig, ValidateDatasetConfig,
};

type ChunkMessage = DatasetResult<LineChunk>;
type BatchMessage = DatasetResult<SampleBatch>;

/// Runtime statistics for a parallel pipeline.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PipelineStats {
    /// Parsed samples.
    pub samples: u64,
    /// Raw bytes processed.
    pub bytes: u64,
    /// Line chunks scheduled.
    pub chunks: u64,
    /// Producer queue stall time.
    pub queue_stall_time: Duration,
}

/// Parallel JSONL pipeline.
pub struct ParallelJsonlPipeline {
    batches: Receiver<BatchMessage>,
    reader_handle: Option<JoinHandle<PipelineStats>>,
    parser_handle: Option<JoinHandle<PipelineStats>>,
}

impl ParallelJsonlPipeline {
    /// Opens and starts a parallel JSONL pipeline.
    ///
    /// # Errors
    ///
    /// Returns an error when config is invalid.
    pub fn open(config: JsonlDatasetConfig, options: PipelineOptions) -> DatasetResult<Self> {
        validate_config(&config)?;
        if options.chunk_lines == 0
            || options.chunk_queue_capacity == 0
            || options.batch_queue_capacity == 0
        {
            return Err(DatasetError::InvalidConfig(
                "pipeline queue capacities and chunk size must be greater than zero".to_owned(),
            ));
        }

        let chunk_queue = BoundedQueue::new(options.chunk_queue_capacity);
        let batch_queue = BoundedQueue::new(options.batch_queue_capacity);
        let chunk_sender = chunk_queue.sender();
        let chunk_receiver = chunk_queue.receiver();
        let batch_sender = batch_queue.sender();
        let batch_receiver = batch_queue.receiver();

        let reader_config = config.clone();
        let reader_handle =
            thread::spawn(move || read_chunks(reader_config, options, chunk_sender));
        let parser_handle =
            thread::spawn(move || parse_chunks(config, options, chunk_receiver, batch_sender));

        Ok(Self {
            batches: batch_receiver,
            reader_handle: Some(reader_handle),
            parser_handle: Some(parser_handle),
        })
    }

    /// Returns the next parsed sample batch.
    ///
    /// # Errors
    ///
    /// Returns a pipeline or parsing error from worker threads.
    pub fn next_batch(&self) -> DatasetResult<Option<SampleBatch>> {
        match self.batches.recv() {
            Ok(Ok(batch)) => Ok(Some(batch)),
            Ok(Err(error)) => Err(error),
            Err(_) => Ok(None),
        }
    }

    /// Joins worker threads and returns aggregate stats.
    ///
    /// # Errors
    ///
    /// Returns an error when a worker panicked.
    pub fn join(&mut self) -> DatasetResult<PipelineStats> {
        let reader = join_stats(self.reader_handle.take())?;
        let parser = join_stats(self.parser_handle.take())?;
        Ok(PipelineStats {
            samples: parser.samples,
            bytes: reader.bytes,
            chunks: reader.chunks,
            queue_stall_time: reader.queue_stall_time + parser.queue_stall_time,
        })
    }
}

fn validate_config(config: &JsonlDatasetConfig) -> DatasetResult<()> {
    let report = config.validate();
    if report.is_valid() {
        return Ok(());
    }

    let message = report
        .errors
        .into_iter()
        .map(|error| format!("{}: {}", error.path, error.message))
        .collect::<Vec<_>>()
        .join("; ");
    Err(DatasetError::InvalidConfig(message))
}

impl Drop for ParallelJsonlPipeline {
    fn drop(&mut self) {
        let _ = self.join();
    }
}

fn read_chunks(
    config: JsonlDatasetConfig,
    options: PipelineOptions,
    chunks: Sender<ChunkMessage>,
) -> PipelineStats {
    match read_chunks_inner(config, options, &chunks) {
        Ok(stats) => stats,
        Err(error) => {
            let _ = chunks.send(Err(error));
            PipelineStats::default()
        }
    }
}

fn read_chunks_inner(
    config: JsonlDatasetConfig,
    options: PipelineOptions,
    chunks: &Sender<ChunkMessage>,
) -> DatasetResult<PipelineStats> {
    let input = DatasetInputReader::open(&config.path, config.compression)?;
    let mut reader = BufReader::new(input);
    let scheduler = ChunkScheduler::new(options);
    let mut stats = PipelineStats::default();
    let mut line_number = 0_u64;
    let mut emitted = 0_usize;
    let mut buffer = Vec::new();
    let mut chunk = LineChunk::with_capacity(options.chunk_lines);

    loop {
        if config.max_samples.is_some_and(|limit| emitted >= limit) {
            break;
        }

        buffer.clear();
        let read = reader
            .read_until(b'\n', &mut buffer)
            .map_err(|error| DatasetError::io(&config.path, error))?;
        if read == 0 {
            break;
        }
        line_number = line_number.saturating_add(1);
        trim_newline(&mut buffer);
        if buffer.is_empty() {
            continue;
        }

        stats.bytes = stats
            .bytes
            .saturating_add(u64::try_from(buffer.len()).unwrap_or(u64::MAX));
        chunk.push(RawLine {
            line: line_number,
            bytes: buffer.clone(),
        });
        emitted = emitted.saturating_add(1);

        if scheduler.should_schedule(chunk.lines.len()) {
            stats.queue_stall_time += send_chunk(chunks, &mut chunk, options.send_timeout)?;
            stats.chunks = stats.chunks.saturating_add(1);
        }
    }

    if !chunk.is_empty() {
        stats.queue_stall_time += send_chunk(chunks, &mut chunk, options.send_timeout)?;
        stats.chunks = stats.chunks.saturating_add(1);
    }

    Ok(stats)
}

fn send_chunk(
    chunks: &Sender<ChunkMessage>,
    chunk: &mut LineChunk,
    timeout: Duration,
) -> DatasetResult<Duration> {
    let mut next = LineChunk::with_capacity(chunk.lines.capacity().max(1));
    std::mem::swap(chunk, &mut next);
    send_with_backpressure(chunks, Ok(next), timeout)
        .map_err(|_| DatasetError::Pipeline("chunk queue closed".to_owned()))
}

fn parse_chunks(
    config: JsonlDatasetConfig,
    options: PipelineOptions,
    chunks: Receiver<ChunkMessage>,
    batches: Sender<BatchMessage>,
) -> PipelineStats {
    let parser = SimdJsonlParser::new(&config);
    let mut stats = PipelineStats::default();

    while let Ok(message) = chunks.recv() {
        let chunk = match message {
            Ok(chunk) => chunk,
            Err(error) => {
                let _ = batches.send(Err(error));
                break;
            }
        };
        let first_line = chunk.lines.first().map(|line| line.line);
        let last_line = chunk.lines.last().map(|line| line.line);
        let bytes = chunk.bytes;
        let parsed = parse_chunk(&parser, chunk);

        match parsed {
            Ok(samples) => {
                stats.samples = stats
                    .samples
                    .saturating_add(u64::try_from(samples.len()).unwrap_or(u64::MAX));
                stats.bytes = stats
                    .bytes
                    .saturating_add(u64::try_from(bytes).unwrap_or(u64::MAX));
                let batch = SampleBatch::new(samples, bytes, first_line, last_line);
                match send_with_backpressure(&batches, Ok(batch), options.send_timeout) {
                    Ok(stall) => stats.queue_stall_time += stall,
                    Err(_) => break,
                }
            }
            Err(error) => {
                let _ = batches.send(Err(error));
                break;
            }
        }
    }

    stats
}

fn parse_chunk(parser: &SimdJsonlParser, chunk: LineChunk) -> DatasetResult<Vec<RawSample>> {
    chunk
        .lines
        .into_par_iter()
        .map(|line| {
            parser
                .parse_bytes(line.line, Bytes::from(line.bytes))
                .map(|record| match record.id {
                    Some(id) => RawSample::with_id(id, record.text),
                    None => RawSample::new(record.text),
                })
        })
        .collect()
}

fn join_stats(handle: Option<JoinHandle<PipelineStats>>) -> DatasetResult<PipelineStats> {
    let Some(handle) = handle else {
        return Ok(PipelineStats::default());
    };
    handle
        .join()
        .map_err(|_| DatasetError::Pipeline("pipeline worker panicked".to_owned()))
}

fn trim_newline(buffer: &mut Vec<u8>) {
    while matches!(buffer.last(), Some(b'\n' | b'\r')) {
        let _ = buffer.pop();
    }
}
