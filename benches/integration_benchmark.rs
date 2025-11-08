use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;
use tempfile::TempDir;

// 統合シナリオのベンチマーク

fn create_test_files(dir: &std::path::Path, count: usize, size: usize) {
    for i in 0..count {
        let path = dir.join(format!("file_{i}.txt"));
        let content = vec![b'x'; size];
        fs::write(path, content).unwrap();
    }
}

fn create_nested_structure(dir: &std::path::Path, depth: usize, files_per_dir: usize) {
    if depth == 0 {
        create_test_files(dir, files_per_dir, 1024);
        return;
    }

    for i in 0..3 {
        let subdir = dir.join(format!("level_{i}"));
        fs::create_dir_all(&subdir).unwrap();
        create_nested_structure(&subdir, depth - 1, files_per_dir);
    }
}

fn bench_small_files_backup(c: &mut Criterion) {
    let file_counts = vec![10, 50, 100, 500];

    let mut group = c.benchmark_group("small_files_backup");

    for count in file_counts {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let temp_dir = TempDir::new().unwrap();
            let source = temp_dir.path().join("source");
            let dest = temp_dir.path().join("dest");
            fs::create_dir_all(&source).unwrap();
            fs::create_dir_all(&dest).unwrap();

            create_test_files(&source, count, 1024); // 1KB each

            b.iter(|| {
                for entry in fs::read_dir(&source).unwrap() {
                    let entry = entry.unwrap();
                    let dest_path = dest.join(entry.file_name());
                    fs::copy(entry.path(), black_box(dest_path)).unwrap();
                }
            });
        });
    }

    group.finish();
}

fn bench_large_files_backup(c: &mut Criterion) {
    let sizes = vec![
        (1, 1024 * 1024, "1MB"),
        (1, 10 * 1024 * 1024, "10MB"),
        (1, 50 * 1024 * 1024, "50MB"),
    ];

    let mut group = c.benchmark_group("large_files_backup");

    for (count, size, name) in sizes {
        group.throughput(Throughput::Bytes((count * size) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(count, size),
            |b, &(count, size)| {
                let temp_dir = TempDir::new().unwrap();
                let source = temp_dir.path().join("source");
                let dest = temp_dir.path().join("dest");
                fs::create_dir_all(&source).unwrap();
                fs::create_dir_all(&dest).unwrap();

                create_test_files(&source, count, size);

                b.iter(|| {
                    for entry in fs::read_dir(&source).unwrap() {
                        let entry = entry.unwrap();
                        let dest_path = dest.join(entry.file_name());
                        fs::copy(entry.path(), black_box(dest_path)).unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_nested_directory_backup(c: &mut Criterion) {
    let depths = vec![2, 3, 4];

    let mut group = c.benchmark_group("nested_directory_backup");

    for depth in depths {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("depth_{depth}")),
            &depth,
            |b, &depth| {
                let temp_dir = TempDir::new().unwrap();
                let source = temp_dir.path().join("source");
                let dest = temp_dir.path().join("dest");
                fs::create_dir_all(&source).unwrap();
                fs::create_dir_all(&dest).unwrap();

                create_nested_structure(&source, depth, 5);

                b.iter(|| {
                    copy_recursive(black_box(&source), black_box(&dest)).unwrap();
                });
            },
        );
    }

    group.finish();
}

fn copy_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }

    Ok(())
}

fn bench_mixed_workload(c: &mut Criterion) {
    c.bench_function("mixed_workload", |b| {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source");
        let dest = temp_dir.path().join("dest");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&dest).unwrap();

        // 混合ワークロード作成
        create_test_files(&source, 50, 1024); // 50個の1KBファイル
        create_test_files(&source, 10, 1024 * 1024); // 10個の1MBファイル

        let subdir = source.join("nested");
        fs::create_dir_all(&subdir).unwrap();
        create_test_files(&subdir, 20, 512); // 20個の512Bファイル

        b.iter(|| {
            copy_recursive(black_box(&source), black_box(&dest)).unwrap();
        });
    });
}

fn bench_incremental_backup(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source");
    let dest = temp_dir.path().join("dest");
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&dest).unwrap();

    // 初期ファイル作成
    create_test_files(&source, 100, 1024);

    // 初回バックアップ
    copy_recursive(&source, &dest).unwrap();

    let mut group = c.benchmark_group("incremental_backup");

    group.bench_function("no_changes", |b| {
        b.iter(|| {
            // 変更なしの場合（スキップ処理のテスト）
            for entry in fs::read_dir(&source).unwrap() {
                let entry = entry.unwrap();
                let dest_path = dest.join(entry.file_name());

                if dest_path.exists() {
                    let src_meta = fs::metadata(entry.path()).unwrap();
                    let dst_meta = fs::metadata(&dest_path).unwrap();

                    if src_meta.modified().unwrap() == dst_meta.modified().unwrap() {
                        black_box(());
                        continue;
                    }
                }

                fs::copy(entry.path(), black_box(dest_path)).unwrap();
            }
        });
    });

    group.bench_function("partial_changes", |b| {
        // 10%のファイルを変更
        for i in (0..100).step_by(10) {
            let path = source.join(format!("file_{i}.txt"));
            fs::write(&path, vec![b'y'; 1024]).unwrap();
        }

        b.iter(|| {
            for entry in fs::read_dir(&source).unwrap() {
                let entry = entry.unwrap();
                let dest_path = dest.join(entry.file_name());

                if dest_path.exists() {
                    let src_meta = fs::metadata(entry.path()).unwrap();
                    let dst_meta = fs::metadata(&dest_path).unwrap();

                    if src_meta.modified().unwrap() == dst_meta.modified().unwrap() {
                        black_box(());
                        continue;
                    }
                }

                fs::copy(entry.path(), black_box(dest_path)).unwrap();
            }
        });
    });

    group.finish();
}

fn bench_parallel_backup(c: &mut Criterion) {
    use rayon::prelude::*;

    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source");
    let dest = temp_dir.path().join("dest");
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&dest).unwrap();

    create_test_files(&source, 100, 10 * 1024); // 100個の10KBファイル

    let mut group = c.benchmark_group("parallel_backup");
    group.throughput(Throughput::Elements(100));

    group.bench_function("sequential", |b| {
        b.iter(|| {
            for entry in fs::read_dir(&source).unwrap() {
                let entry = entry.unwrap();
                let dest_path = dest.join(entry.file_name());
                fs::copy(entry.path(), black_box(&dest_path)).unwrap();
            }
        });
    });

    group.bench_function("parallel", |b| {
        b.iter(|| {
            let entries: Vec<_> = fs::read_dir(&source)
                .unwrap()
                .filter_map(|e| e.ok())
                .collect();

            entries.par_iter().for_each(|entry| {
                let dest_path = dest.join(entry.file_name());
                fs::copy(entry.path(), black_box(&dest_path)).unwrap();
            });
        });
    });

    group.finish();
}

fn bench_filtering_performance(c: &mut Criterion) {
    use regex::Regex;

    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source");
    fs::create_dir_all(&source).unwrap();

    // 様々な拡張子のファイルを作成
    for i in 0..50 {
        fs::write(source.join(format!("file_{i}.txt")), "text").unwrap();
        fs::write(source.join(format!("file_{i}.log")), "log").unwrap();
        fs::write(source.join(format!("file_{i}.tmp")), "tmp").unwrap();
        fs::write(source.join(format!("file_{i}.dat")), "data").unwrap();
    }

    let exclude_pattern = Regex::new(r"\.(log|tmp)$").unwrap();

    let mut group = c.benchmark_group("filtering_performance");
    group.throughput(Throughput::Elements(200));

    group.bench_function("with_filter", |b| {
        b.iter(|| {
            let entries: Vec<_> = fs::read_dir(&source)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|entry| {
                    let path = entry.path();
                    let path_str = path.to_string_lossy();
                    !exclude_pattern.is_match(&path_str)
                })
                .collect();
            black_box(entries);
        });
    });

    group.bench_function("without_filter", |b| {
        b.iter(|| {
            let entries: Vec<_> = fs::read_dir(&source)
                .unwrap()
                .filter_map(|e| e.ok())
                .collect();
            black_box(entries);
        });
    });

    group.finish();
}

fn bench_metadata_collection(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source");
    fs::create_dir_all(&source).unwrap();

    create_test_files(&source, 100, 1024);

    let mut group = c.benchmark_group("metadata_collection");
    group.throughput(Throughput::Elements(100));

    group.bench_function("basic_metadata", |b| {
        b.iter(|| {
            let metadata: Vec<_> = fs::read_dir(&source)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter_map(|entry| {
                    fs::metadata(entry.path())
                        .ok()
                        .map(|meta| (entry.file_name(), meta.len()))
                })
                .collect();
            black_box(metadata);
        });
    });

    group.bench_function("full_metadata", |b| {
        b.iter(|| {
            let metadata: Vec<_> = fs::read_dir(&source)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter_map(|entry| {
                    fs::metadata(entry.path()).ok().map(|meta| {
                        (
                            entry.file_name(),
                            meta.len(),
                            meta.modified().ok(),
                            meta.is_dir(),
                        )
                    })
                })
                .collect();
            black_box(metadata);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_small_files_backup,
    bench_large_files_backup,
    bench_nested_directory_backup,
    bench_mixed_workload,
    bench_incremental_backup,
    bench_parallel_backup,
    bench_filtering_performance,
    bench_metadata_collection,
);

criterion_main!(benches);
