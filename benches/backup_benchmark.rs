//! パフォーマンスベンチマーク - backup-suite
//!
//! Criterionを使用した詳細なパフォーマンス測定
//!
//! 実行方法:
//! ```bash
//! cargo bench
//! cargo bench -- --baseline baseline  # ベースライン設定
//! cargo bench -- --baseline optimized # 最適化後と比較
//! ```

use backup_suite::{Config, Priority, Target};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;
use tempfile::TempDir;

// ==================== ヘルパー関数 ====================

/// ベンチマーク用のテスト環境を作成
fn setup_benchmark_env(num_files: usize) -> (TempDir, Config) {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    fs::create_dir_all(&source_dir).unwrap();

    // ファイル作成
    for i in 0..num_files {
        fs::write(
            source_dir.join(format!("file_{:04}.txt", i)),
            format!("content for file {}", i),
        )
        .unwrap();
    }

    // 設定作成
    let mut config = Config::default();
    config.backup.destination = backup_dir;
    config.targets.push(Target::new(
        source_dir,
        Priority::High,
        "benchmark".to_string(),
    ));

    (temp_dir, config)
}

/// サブディレクトリ構造を持つ環境を作成
fn setup_nested_structure(depth: usize, files_per_dir: usize) -> (TempDir, Config) {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    // ネストされたディレクトリ構造を作成
    fn create_nested(base: &std::path::Path, current_depth: usize, max_depth: usize, files: usize) {
        if current_depth >= max_depth {
            return;
        }

        fs::create_dir_all(base).unwrap();

        // 各ディレクトリにファイルを作成
        for i in 0..files {
            fs::write(
                base.join(format!("file_{}.txt", i)),
                format!("content at depth {}", current_depth),
            )
            .unwrap();
        }

        // サブディレクトリを作成
        for i in 0..2 {
            let subdir = base.join(format!("subdir_{}", i));
            create_nested(&subdir, current_depth + 1, max_depth, files);
        }
    }

    create_nested(&source_dir, 0, depth, files_per_dir);

    let mut config = Config::default();
    config.backup.destination = backup_dir;
    config.targets.push(Target::new(
        source_dir,
        Priority::High,
        "benchmark".to_string(),
    ));

    (temp_dir, config)
}

/// 大きなファイルを作成
fn setup_large_files(num_files: usize, size_mb: usize) -> (TempDir, Config) {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    fs::create_dir_all(&source_dir).unwrap();

    let content = vec![0u8; size_mb * 1024 * 1024];

    for i in 0..num_files {
        fs::write(source_dir.join(format!("large_file_{}.bin", i)), &content).unwrap();
    }

    let mut config = Config::default();
    config.backup.destination = backup_dir;
    config.targets.push(Target::new(
        source_dir,
        Priority::High,
        "benchmark".to_string(),
    ));

    (temp_dir, config)
}

// ==================== ベンチマーク: 小さなファイル ====================

fn benchmark_small_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_files");

    for num_files in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*num_files as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(num_files),
            num_files,
            |b, &num_files| {
                b.iter_batched(
                    || setup_benchmark_env(num_files),
                    |(_temp_dir, config)| {
                        let result =
                            backup_suite::core::backup::run_backup(&config, None).unwrap();
                        black_box(result)
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// ==================== ベンチマーク: 大きなファイル ====================

fn benchmark_large_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_files");
    group.sample_size(10); // サンプル数を減らす（時間がかかるため）

    for size_mb in [1, 5, 10].iter() {
        group.throughput(Throughput::Bytes((size_mb * 1024 * 1024) as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}MB", size_mb)),
            size_mb,
            |b, &size_mb| {
                b.iter_batched(
                    || setup_large_files(1, size_mb),
                    |(_temp_dir, config)| {
                        let result =
                            backup_suite::core::backup::run_backup(&config, None).unwrap();
                        black_box(result)
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// ==================== ベンチマーク: ネストされたディレクトリ ====================

fn benchmark_nested_directories(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_directories");

    for depth in [2, 3, 4].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("depth_{}", depth)),
            depth,
            |b, &depth| {
                b.iter_batched(
                    || setup_nested_structure(depth, 3),
                    |(_temp_dir, config)| {
                        let result =
                            backup_suite::core::backup::run_backup(&config, None).unwrap();
                        black_box(result)
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// ==================== ベンチマーク: 並列処理スケーラビリティ ====================

fn benchmark_parallel_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_processing");

    // 大量のファイルでの並列処理性能を測定
    for num_files in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*num_files as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(num_files),
            num_files,
            |b, &num_files| {
                b.iter_batched(
                    || setup_benchmark_env(num_files),
                    |(_temp_dir, config)| {
                        let result =
                            backup_suite::core::backup::run_backup(&config, None).unwrap();
                        black_box(result)
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// ==================== ベンチマーク: 優先度フィルタリング ====================

fn benchmark_priority_filtering(c: &mut Criterion) {
    let mut group = c.benchmark_group("priority_filtering");

    let num_files = 100;

    group.bench_function("no_filter", |b| {
        b.iter_batched(
            || setup_benchmark_env(num_files),
            |(_temp_dir, config)| {
                let result = backup_suite::core::backup::run_backup(&config, None).unwrap();
                black_box(result)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("high_priority_only", |b| {
        b.iter_batched(
            || setup_benchmark_env(num_files),
            |(_temp_dir, config)| {
                let result =
                    backup_suite::core::backup::run_backup(&config, Some(Priority::High)).unwrap();
                black_box(result)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ==================== ベンチマーク: 設定の読み込み ====================

fn benchmark_config_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_operations");

    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.toml");

    // テスト用設定作成
    let mut config = Config::default();
    for i in 0..100 {
        config.targets.push(Target::new(
            temp_dir.path().join(format!("target_{}", i)),
            Priority::Medium,
            "test".to_string(),
        ));
    }

    // シリアライズ
    group.bench_function("serialize_config", |b| {
        b.iter(|| {
            let toml_str = toml::to_string(&config).unwrap();
            black_box(toml_str)
        });
    });

    // デシリアライズ用のTOML文字列を準備
    let toml_str = toml::to_string(&config).unwrap();
    fs::write(&config_file, &toml_str).unwrap();

    group.bench_function("deserialize_config", |b| {
        b.iter(|| {
            let loaded: Config = toml::from_str(&toml_str).unwrap();
            black_box(loaded)
        });
    });

    group.finish();
}

// ==================== ベンチマーク: ファイル収集 ====================

fn benchmark_file_collection(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_collection");

    for num_files in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*num_files as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(num_files),
            num_files,
            |b, &num_files| {
                let temp_dir = TempDir::new().unwrap();
                let source_dir = temp_dir.path().join("source");
                fs::create_dir_all(&source_dir).unwrap();

                for i in 0..num_files {
                    fs::write(
                        source_dir.join(format!("file_{}.txt", i)),
                        format!("content {}", i),
                    )
                    .unwrap();
                }

                b.iter(|| {
                    let files: Vec<_> = walkdir::WalkDir::new(&source_dir)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_type().is_file())
                        .map(|e| e.path().to_path_buf())
                        .collect();

                    black_box(files)
                });
            },
        );
    }

    group.finish();
}

// ==================== ベンチマーク: 除外パターンマッチング ====================

fn benchmark_exclude_patterns(c: &mut Criterion) {
    use regex::Regex;

    let mut group = c.benchmark_group("exclude_patterns");

    let patterns = vec![
        Regex::new(r".*\.tmp$").unwrap(),
        Regex::new(r".*\.bak$").unwrap(),
        Regex::new(r"node_modules/.*").unwrap(),
        Regex::new(r"^\..+").unwrap(),
    ];

    let test_files = vec![
        "file.txt",
        "file.tmp",
        "backup.bak",
        "node_modules/package/index.js",
        ".env",
        "src/main.rs",
    ];

    group.bench_function("pattern_matching", |b| {
        b.iter(|| {
            for file in &test_files {
                let should_exclude = patterns.iter().any(|p| p.is_match(file));
                black_box(should_exclude);
            }
        });
    });

    group.finish();
}

// ==================== ベンチマーク: メモリ使用量（参考） ====================

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    // 大量のターゲットを持つ設定
    group.bench_function("large_config_allocation", |b| {
        b.iter(|| {
            let mut config = Config::default();
            for i in 0..10000 {
                config.targets.push(Target::new(
                    format!("/tmp/target_{}", i).into(),
                    Priority::Medium,
                    "test".to_string(),
                ));
            }
            black_box(config)
        });
    });

    group.finish();
}

// ==================== Criterionグループ設定 ====================

criterion_group!(
    benches,
    benchmark_small_files,
    benchmark_large_files,
    benchmark_nested_directories,
    benchmark_parallel_processing,
    benchmark_priority_filtering,
    benchmark_config_operations,
    benchmark_file_collection,
    benchmark_exclude_patterns,
    benchmark_memory_usage,
);

criterion_main!(benches);
