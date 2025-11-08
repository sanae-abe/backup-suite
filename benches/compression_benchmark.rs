use backup_suite::compression::{CompressionConfig, CompressionEngine, CompressionType};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use std::io::Cursor;

fn generate_test_data(size: usize) -> Vec<u8> {
    // 実際のファイルに近いランダムデータ生成
    let mut data = Vec::with_capacity(size);
    for i in 0..size {
        data.push((i % 256) as u8);
    }
    data
}

fn benchmark_zstd_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("zstd_compression_levels");
    let data_1mb = generate_test_data(1024 * 1024);

    group.throughput(Throughput::Bytes(data_1mb.len() as u64));

    for level in [1, 3, 5, 7, 9].iter() {
        let config = CompressionConfig {
            level: *level,
            chunk_size: 2 * 1024 * 1024,
            buffer_size: 128 * 1024,
        };
        let engine = CompressionEngine::new(CompressionType::Zstd, config);

        group.bench_with_input(BenchmarkId::new("compress", level), &data_1mb, |b, data| {
            b.iter(|| engine.compress(black_box(data)).unwrap());
        });
    }

    group.finish();
}

fn benchmark_chunk_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("zstd_chunk_sizes");
    let data_10mb = generate_test_data(10 * 1024 * 1024);

    group.throughput(Throughput::Bytes(data_10mb.len() as u64));

    for chunk_size in [512 * 1024, 1024 * 1024, 2 * 1024 * 1024, 4 * 1024 * 1024].iter() {
        let config = CompressionConfig {
            level: 5,
            chunk_size: *chunk_size,
            buffer_size: 128 * 1024,
        };
        let engine = CompressionEngine::new(CompressionType::Zstd, config);

        group.bench_with_input(
            BenchmarkId::new("compress", chunk_size / 1024),
            &data_10mb,
            |b, data| {
                b.iter(|| engine.compress(black_box(data)).unwrap());
            },
        );
    }

    group.finish();
}

fn benchmark_buffer_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("zstd_buffer_sizes");
    let data_5mb = generate_test_data(5 * 1024 * 1024);

    group.throughput(Throughput::Bytes(data_5mb.len() as u64));

    for buffer_size in [64 * 1024, 128 * 1024, 256 * 1024].iter() {
        let config = CompressionConfig {
            level: 5,
            chunk_size: 2 * 1024 * 1024,
            buffer_size: *buffer_size,
        };
        let engine = CompressionEngine::new(CompressionType::Zstd, config);

        group.bench_with_input(
            BenchmarkId::new("stream_compress", buffer_size / 1024),
            &data_5mb,
            |b, _| {
                b.iter(|| {
                    let reader = Cursor::new(&data_5mb);
                    let mut output = Vec::new();
                    engine.compress_stream(reader, &mut output).unwrap()
                });
            },
        );
    }

    group.finish();
}

fn benchmark_compression_ratio(c: &mut Criterion) {
    let group = c.benchmark_group("compression_ratio");
    let data_1mb = generate_test_data(1024 * 1024);

    // 圧縮率の比較（速度は測定しない）
    for level in [3, 5, 7, 9].iter() {
        let config = CompressionConfig {
            level: *level,
            chunk_size: 2 * 1024 * 1024,
            buffer_size: 128 * 1024,
        };
        let engine = CompressionEngine::new(CompressionType::Zstd, config);

        let compressed = engine.compress(&data_1mb).unwrap();
        println!(
            "Zstd level {}: {:.2}% compression ({} → {} bytes)",
            level,
            compressed.compression_percentage(),
            compressed.original_size,
            compressed.compressed_size
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_zstd_levels,
    benchmark_chunk_sizes,
    benchmark_buffer_sizes,
    benchmark_compression_ratio
);
criterion_main!(benches);
