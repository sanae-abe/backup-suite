//! # backup-suite パフォーマンスベンチマーク
//!
//! このファイルは backup-suite の主要コンポーネントのパフォーマンステストを実装します。
//! Criterionを使用して、継続的な性能監視と回帰検出を行います。

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use backup_suite::{
    BackupRunner, Config, Target, Priority,
    security::{safe_join, validate_path_safety},
    core::{CopyEngine, FileFilter},
    ui::BackupProgress,
};

/// テストファイル作成ヘルパー
fn create_test_files(dir: &Path, count: usize, size_kb: usize) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let content = vec![b'A'; size_kb * 1024];

    for i in 0..count {
        let file_path = dir.join(format!("test_file_{:04}.dat", i));
        let mut file = File::create(&file_path)?;
        file.write_all(&content)?;
        files.push(file_path);
    }

    Ok(files)
}

/// テストディレクトリ構造作成
fn create_nested_structure(base: &Path, depth: usize, files_per_dir: usize) -> std::io::Result<()> {
    if depth == 0 {
        return Ok(());
    }

    for i in 0..3 {
        let dir = base.join(format!("level_{}", i));
        create_dir_all(&dir)?;

        // 各ディレクトリにファイルを作成
        create_test_files(&dir, files_per_dir, 1)?;

        // 再帰的にサブディレクトリを作成
        create_nested_structure(&dir, depth - 1, files_per_dir)?;
    }

    Ok(())
}

/// 1. ファイルコピー性能ベンチマーク
fn bench_file_copy(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_copy");

    // 異なるファイルサイズでのテスト
    for size_kb in [1, 10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Bytes((*size_kb * 1024) as u64));

        group.bench_with_input(
            BenchmarkId::new("copy_engine", format!("{}KB", size_kb)),
            size_kb,
            |b, &size_kb| {
                let temp_dir = TempDir::new().unwrap();
                let source = temp_dir.path().join("source.dat");
                let dest = temp_dir.path().join("dest.dat");

                // テストファイル作成
                let content = vec![b'X'; size_kb * 1024];
                std::fs::write(&source, &content).unwrap();

                let copy_engine = CopyEngine::new();

                b.iter(|| {
                    // 前回のコピー結果を削除
                    let _ = std::fs::remove_file(&dest);

                    black_box(copy_engine.copy_file(
                        black_box(&source),
                        black_box(&dest)
                    ).unwrap())
                });
            },
        );
    }

    group.finish();
}

/// 2. 並列バックアップ性能ベンチマーク
fn bench_parallel_backup(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_backup");

    // 異なるファイル数でのテスト
    for file_count in [10, 50, 100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*file_count as u64));

        group.bench_with_input(
            BenchmarkId::new("backup_files", format!("{}_files", file_count)),
            file_count,
            |b, &file_count| {
                let temp_dir = TempDir::new().unwrap();
                let source_dir = temp_dir.path().join("source");
                let backup_dir = temp_dir.path().join("backup");

                create_dir_all(&source_dir).unwrap();
                create_dir_all(&backup_dir).unwrap();

                // テストファイル作成（各1KB）
                create_test_files(&source_dir, file_count, 1).unwrap();

                let mut config = Config::default();
                config.backup.destination = backup_dir;

                let target = Target::new(
                    source_dir,
                    Priority::High,
                    "test".to_string()
                );
                config.add_target(target);

                let runner = BackupRunner::new(config, false)
                    .with_progress(false); // ベンチマーク中はプログレス無効

                b.iter(|| {
                    black_box(runner.run(None, None).unwrap())
                });
            },
        );
    }

    group.finish();
}

/// 3. セキュリティ機能性能ベンチマーク
fn bench_security_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("security");

    // safe_join性能テスト
    group.bench_function("safe_join_simple", |b| {
        let base = Path::new("/safe/backup/dir");
        let child = Path::new("documents/file.txt");

        b.iter(|| {
            black_box(safe_join(
                black_box(base),
                black_box(child)
            ).unwrap())
        });
    });

    // 複雑なパスでのsafe_join
    group.bench_function("safe_join_complex", |b| {
        let base = Path::new("/very/long/base/directory/path/for/backup/storage");
        let child = Path::new("deeply/nested/subdirectory/structure/with/many/levels/file.txt");

        b.iter(|| {
            black_box(safe_join(
                black_box(base),
                black_box(child)
            ).unwrap())
        });
    });

    // パス検証性能
    group.bench_function("validate_path_safety", |b| {
        let safe_path = Path::new("documents/projects/backup-suite/file.txt");

        b.iter(|| {
            validate_path_safety(black_box(safe_path)).unwrap()
        });
    });

    group.finish();
}

/// 4. ファイルフィルタリング性能ベンチマーク
fn bench_file_filtering(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_filtering");

    // 除外パターン数による性能影響
    for pattern_count in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("filter_patterns", format!("{}_patterns", pattern_count)),
            pattern_count,
            |b, &pattern_count| {
                let patterns: Vec<String> = (0..pattern_count)
                    .map(|i| format!(r".*\.tmp{}", i))
                    .collect();

                let filter = FileFilter::new(&patterns).unwrap();
                let test_file = Path::new("/path/to/document.txt");

                b.iter(|| {
                    black_box(filter.should_exclude(black_box(test_file)))
                });
            },
        );
    }

    group.finish();
}

/// 5. 設定管理性能ベンチマーク
fn bench_config_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("config");

    // 設定ロード性能
    group.bench_function("config_load", |b| {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        // 設定ファイルを作成
        let mut config = Config::default();
        for i in 0..100 {
            let target = Target::new(
                PathBuf::from(format!("/test/path/{}", i)),
                Priority::Medium,
                format!("category_{}", i)
            );
            config.add_target(target);
        }
        config.save().unwrap();

        b.iter(|| {
            black_box(Config::load().unwrap())
        });
    });

    // 設定保存性能
    group.bench_function("config_save", |b| {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let mut config = Config::default();
        for i in 0..100 {
            let target = Target::new(
                PathBuf::from(format!("/test/path/{}", i)),
                Priority::Medium,
                format!("category_{}", i)
            );
            config.add_target(target);
        }

        b.iter(|| {
            config.save().unwrap()
        });
    });

    group.finish();
}

/// 6. ディレクトリ走査性能ベンチマーク
fn bench_directory_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory_traversal");

    // 異なる深度・ファイル数での走査性能
    for (depth, files_per_dir) in [(3, 10), (4, 5), (5, 3)].iter() {
        group.bench_with_input(
            BenchmarkId::new("walkdir", format!("depth_{}_files_{}", depth, files_per_dir)),
            &(depth, files_per_dir),
            |b, &(depth, files_per_dir)| {
                let temp_dir = TempDir::new().unwrap();
                let test_dir = temp_dir.path().join("nested");
                create_dir_all(&test_dir).unwrap();

                // 入れ子構造作成
                create_nested_structure(&test_dir, *depth, *files_per_dir).unwrap();

                b.iter(|| {
                    use walkdir::WalkDir;
                    let mut count = 0;
                    for entry in WalkDir::new(black_box(&test_dir)).into_iter().filter_map(|e| e.ok()) {
                        if entry.file_type().is_file() {
                            count += 1;
                        }
                    }
                    black_box(count)
                });
            },
        );
    }

    group.finish();
}

/// 7. プログレスバー性能ベンチマーク
fn bench_progress_display(c: &mut Criterion) {
    let mut group = c.benchmark_group("progress_display");

    // プログレスバー更新性能
    group.bench_function("progress_update", |b| {
        let progress = BackupProgress::new(1000);

        b.iter(|| {
            progress.inc(black_box(1))
        });
    });

    // メッセージ付きプログレス更新
    group.bench_function("progress_with_message", |b| {
        let progress = BackupProgress::new(1000);

        b.iter(|| {
            progress.set_message(black_box("Processing file.txt"));
            progress.inc(black_box(1))
        });
    });

    group.finish();
}

/// 8. 統合性能ベンチマーク（End-to-End）
fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");

    // 小規模バックアップ（現実的なユースケース）
    group.bench_function("small_backup_realistic", |b| {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("documents");
        let backup_dir = temp_dir.path().join("backup");

        create_dir_all(&source_dir).unwrap();
        create_dir_all(&backup_dir).unwrap();

        // 現実的なファイル構成
        create_test_files(&source_dir, 50, 10).unwrap(); // 50ファイル、各10KB

        // サブディレクトリ作成
        let sub_dir = source_dir.join("projects");
        create_dir_all(&sub_dir).unwrap();
        create_test_files(&sub_dir, 30, 5).unwrap(); // 30ファイル、各5KB

        let mut config = Config::default();
        config.backup.destination = backup_dir;

        let target = Target::new(
            source_dir,
            Priority::High,
            "documents".to_string()
        );
        config.add_target(target);

        let runner = BackupRunner::new(config, false)
            .with_progress(false);

        b.iter(|| {
            black_box(runner.run(None, None).unwrap())
        });
    });

    group.finish();
}

/// メモリ使用量ベンチマーク
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    // 大量ファイル処理時のメモリ効率
    group.bench_function("large_file_set_memory", |b| {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("large_set");
        let backup_dir = temp_dir.path().join("backup");

        create_dir_all(&source_dir).unwrap();
        create_dir_all(&backup_dir).unwrap();

        // 大量の小ファイル（メモリ効率テスト）
        create_test_files(&source_dir, 2000, 1).unwrap(); // 2000ファイル、各1KB

        let mut config = Config::default();
        config.backup.destination = backup_dir;

        let target = Target::new(
            source_dir,
            Priority::High,
            "large_set".to_string()
        );
        config.add_target(target);

        let runner = BackupRunner::new(config, false)
            .with_progress(false);

        b.iter(|| {
            black_box(runner.run(None, None).unwrap())
        });
    });

    group.finish();
}

// ベンチマークグループ定義
criterion_group!(
    benches,
    bench_file_copy,
    bench_parallel_backup,
    bench_security_features,
    bench_file_filtering,
    bench_config_operations,
    bench_directory_traversal,
    bench_progress_display,
    bench_end_to_end,
    bench_memory_usage
);

criterion_main!(benches);