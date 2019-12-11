extern crate gelf;
extern crate rand;
extern crate criterion;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput, BenchmarkGroup, BenchmarkId};

use rand::{thread_rng, Rng, RngCore};
use rand::distributions::Alphanumeric;
use gelf::{Message, UdpBackend, MessageCompression, Logger};
use std::iter;
use criterion::measurement::WallTime;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn messages_iterator<'a>(characters: usize) -> impl std::iter::Iterator<Item=Message<'a>> {
    let short_message: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(characters)
        .collect();

    iter::repeat_with(move || Message::new(short_message.clone()))
}

fn random_messages_iterator<'a>() -> impl std::iter::Iterator<Item=Message<'a>> {

    iter::repeat_with(|| {
        let characters = thread_rng().next_u32();

        let short_message: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(characters as usize)
            .collect();

        Message::new(short_message)
    })
}

fn generate_messages<'a>(size: usize, characters: usize) -> Vec<Message<'a>> {
   messages_iterator(characters).take(size).collect()
}

fn log_message_characters_benchmark(logger: Logger, group: &mut BenchmarkGroup<WallTime>) {
    for size in [100, 200, 500, 1000].iter() {
        let mut iterator = messages_iterator(*size);

        group.throughput(Throughput::Bytes(*size as u64))
            .bench_function(BenchmarkId::new("Log message", format!("{} Bytes", size)),  |b| {
            b.iter(|| {
                let next = iterator.next().expect("New item");

                black_box(logger.log_message(black_box(next)))
            })
        });
    }
}

fn log_compression_none_benchmark(c: &mut Criterion) {
    let compressor = MessageCompression::None;

    let mut backed = UdpBackend::new("127.0.0.1:5659").unwrap();

    backed.set_compression(compressor);

    let logger = Logger::new(Box::new(backed)).expect("Should create with success");

    let mut group = c.benchmark_group("Log message without compression");

    log_message_characters_benchmark(logger, &mut group)
}

fn log_compression_benchmark(compression: MessageCompression, group: &mut BenchmarkGroup<WallTime>) {
    let mut backend = UdpBackend::new("127.0.0.1:5659").unwrap();
    backend.set_compression(compression);

    let logger = Logger::new(Box::new(backend)).expect("Should create with success");

    log_message_characters_benchmark(logger, group)
}

fn log_compression_gzip_benchmark(c: &mut Criterion) {
    for level in 1..=12 {
        let compression = MessageCompression::Gzip { level };

        let mut group = c.benchmark_group(format!("Log message with compression using Gzip level {}", level));

        log_compression_benchmark(compression, &mut group)
    }

}

fn log_compression_zlib_benchmark(c: &mut Criterion) {
    for level in 1..=12 {
        let compression = MessageCompression::Zlib { level };

        let mut group = c.benchmark_group(format!("Log message with compression using Zlib level {}", level));

        log_compression_benchmark(compression, &mut group)
    }
}

criterion_group!(benches, log_compression_none_benchmark, log_compression_zlib_benchmark, log_compression_gzip_benchmark);
criterion_main!(benches);