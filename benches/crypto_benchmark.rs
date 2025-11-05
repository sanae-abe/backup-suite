//! # 暗号化・圧縮パフォーマンスベンチマーク
//!
//! backup-suiteの暗号化・圧縮機能の性能を測定します。

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use backup_suite::crypto::{EncryptionEngine, KeyManager};
use backup_suite::compression::{CompressionEngine, CompressionType, CompressionConfig};

/// 暗号化性能ベンチマーク
fn bench_encryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("encryption");

    for size_kb in [1, 10, 100, 1000] {
        group.throughput(Throughput::Bytes((size_kb * 1024) as u64));
        group.bench_with_input(
            BenchmarkId::new("aes_256_gcm", format!("{}KB", size_kb)),
            &size_kb,
            |b, &size_kb| {
                let data = vec![0u8; size_kb * 1024];
                let password = "test_password_for_benchmark";
                let key_manager = KeyManager::default();
                let (master_key, salt) = key_manager.create_master_key(password).unwrap();
                let engine = EncryptionEngine::default();

                b.iter(|| {
                    black_box(engine.encrypt(black_box(&data), black_box(&master_key), black_box(salt)).unwrap())
                });
            },
        );
    }

    group.finish();
}

/// 復号化性能ベンチマーク
fn bench_decryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("decryption");

    for size_kb in [1, 10, 100, 1000] {
        group.throughput(Throughput::Bytes((size_kb * 1024) as u64));
        group.bench_with_input(
            BenchmarkId::new("aes_256_gcm", format!("{}KB", size_kb)),
            &size_kb,
            |b, &size_kb| {
                let data = vec![0u8; size_kb * 1024];
                let password = "test_password_for_benchmark";
                let key_manager = KeyManager::default();
                let (master_key, salt) = key_manager.create_master_key(password).unwrap();
                let engine = EncryptionEngine::default();
                let encrypted_data = engine.encrypt(&data, &master_key, salt).unwrap();

                b.iter(|| {
                    black_box(engine.decrypt(black_box(&encrypted_data), black_box(&master_key)).unwrap())
                });
            },
        );
    }

    group.finish();
}

/// 圧縮性能ベンチマーク
fn bench_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");

    for size_kb in [1, 10, 100, 1000] {
        group.throughput(Throughput::Bytes((size_kb * 1024) as u64));

        for comp_type in [CompressionType::Zstd, CompressionType::Gzip] {
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", comp_type), format!("{}KB", size_kb)),
                &size_kb,
                |b, &size_kb| {
                    let data = vec![42u8; size_kb * 1024]; // 繰り返しパターンで圧縮効果を確認
                    let config = match comp_type {
                        CompressionType::Zstd => CompressionConfig::zstd_default(),
                        CompressionType::Gzip => CompressionConfig::gzip_default(),
                        CompressionType::None => CompressionConfig::none(),
                    };
                    let engine = CompressionEngine::new(comp_type, config);

                    b.iter(|| {
                        black_box(engine.compress(black_box(&data)).unwrap())
                    });
                },
            );
        }
    }

    group.finish();
}

/// 展開性能ベンチマーク
fn bench_decompression(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompression");

    for size_kb in [1, 10, 100, 1000] {
        group.throughput(Throughput::Bytes((size_kb * 1024) as u64));

        for comp_type in [CompressionType::Zstd, CompressionType::Gzip] {
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", comp_type), format!("{}KB", size_kb)),
                &size_kb,
                |b, &size_kb| {
                    let data = vec![42u8; size_kb * 1024];
                    let config = match comp_type {
                        CompressionType::Zstd => CompressionConfig::zstd_default(),
                        CompressionType::Gzip => CompressionConfig::gzip_default(),
                        CompressionType::None => CompressionConfig::none(),
                    };
                    let engine = CompressionEngine::new(comp_type, config);
                    let compressed_data = engine.compress(&data).unwrap();

                    b.iter(|| {
                        black_box(engine.decompress(black_box(&compressed_data)).unwrap())
                    });
                },
            );
        }
    }

    group.finish();
}

/// キー導出性能ベンチマーク
fn bench_key_derivation(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_derivation");

    let passwords = ["short", "medium_length_password", "very_long_password_with_many_characters_for_testing"];

    for password in passwords {
        group.bench_with_input(
            BenchmarkId::new("argon2", format!("{}chars", password.len())),
            &password,
            |b, &password| {
                let key_manager = KeyManager::default();

                b.iter(|| {
                    black_box(key_manager.create_master_key(black_box(password)).unwrap())
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_encryption,
    bench_decryption,
    bench_compression,
    bench_decompression,
    bench_key_derivation,
);

criterion_main!(benches);