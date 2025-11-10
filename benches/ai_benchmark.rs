//! # AI機能パフォーマンスベンチマーク
//!
//! backup-suiteのAI/ML機能（異常検知、予測、推奨エンジン）の性能を測定します。
//!
//! ## パフォーマンス目標
//!
//! - 異常検知: < 5ms（100件履歴）、< 100ms（1000件履歴）
//! - 重要度評価: < 100μs/ファイル
//! - ファイル分析: < 500ms（1000ファイル）、< 10秒（10,000ファイル）
//! - 統計計算: < 1ms（1000件）
//! - メモリ使用量: < 50MB（10,000件処理時）

#[cfg(feature = "smart")]
use backup_suite::core::history::{BackupHistory, BackupStatus};
#[cfg(feature = "smart")]
use backup_suite::smart::anomaly::{AnomalyDetector, PatternAnalyzer, Predictor};
#[cfg(feature = "smart")]
use backup_suite::smart::recommendation::{ExcludeRecommendationEngine, ImportanceEvaluator};
#[cfg(feature = "smart")]
use backup_suite::smart::types::{BackupSize, DiskCapacity};
#[cfg(feature = "smart")]
use backup_suite::Priority;
#[cfg(feature = "smart")]
use chrono::{Duration, Utc};
#[cfg(feature = "smart")]
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
#[cfg(feature = "smart")]
use std::hint::black_box;
#[cfg(feature = "smart")]
use std::path::PathBuf;
#[cfg(feature = "smart")]
use tempfile::TempDir;

#[cfg(feature = "smart")]
/// モック履歴データ生成ヘルパー
#[allow(clippy::cast_possible_wrap)]
fn create_mock_histories(count: usize, base_size: u64) -> Vec<BackupHistory> {
    let now = Utc::now();
    (0..count)
        .map(|i| {
            let size = base_size + (i as u64 * 1000);
            let mut history = BackupHistory::new(
                PathBuf::from(format!("/tmp/backup/backup-{}", i)),
                100 + i,
                size,
                true,
            );
            history.timestamp = now - Duration::hours(i as i64);
            history.category = Some("default".to_string());
            history.priority = Some(Priority::Medium);
            history.status = BackupStatus::Success;
            history.compressed = false;
            history.encrypted = false;
            history.duration_ms = 10_000;
            history
        })
        .collect()
}

#[cfg(feature = "smart")]
/// 異常検知ベンチマーク（目標: < 5ms）
fn bench_anomaly_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("anomaly_detection");

    for count in [10, 50, 100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("detect_size_anomaly", count),
            &count,
            |b, &count| {
                let detector = AnomalyDetector::default_detector();
                let histories = create_mock_histories(count, 50_000_000);
                let current_size = BackupSize::new(150_000_000); // 3倍の異常値

                b.iter(|| {
                    black_box(
                        detector
                            .detect_size_anomaly(black_box(&histories), black_box(current_size))
                            .unwrap(),
                    )
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "smart")]
/// ディスク容量予測ベンチマーク（目標: < 50ms）
fn bench_disk_prediction(c: &mut Criterion) {
    let mut group = c.benchmark_group("disk_prediction");

    for count in [10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("predict_disk_usage", count),
            &count,
            |b, &count| {
                let predictor = Predictor::new();
                let histories = create_mock_histories(count, 50_000_000);
                let total_capacity = DiskCapacity::new(1_000_000_000_000); // 1TB

                b.iter(|| {
                    black_box(
                        predictor
                            .predict_disk_usage(
                                black_box(&histories),
                                black_box(total_capacity),
                                black_box(30), // 30日先を予測
                            )
                            .unwrap(),
                    )
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "smart")]
/// パターン分析ベンチマーク
fn bench_pattern_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_analysis");

    for count in [50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::new("calculate_failure_rate", count),
            &count,
            |b, &count| {
                let analyzer = PatternAnalyzer::new();
                let mut histories = create_mock_histories(count, 50_000_000);
                // 20%を失敗にする
                for (i, history) in histories.iter_mut().enumerate() {
                    if i % 5 == 0 {
                        history.status = BackupStatus::Failed;
                        history.error_message = Some("Test error".to_string());
                    }
                }

                b.iter(|| {
                    black_box(
                        analyzer
                            .calculate_failure_rate(black_box(&histories))
                            .unwrap(),
                    )
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "smart")]
/// ファイル重要度評価ベンチマーク（目標: < 100μs/ファイル）
fn bench_importance_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("importance_evaluation");

    let evaluator = ImportanceEvaluator::new();
    let test_files = [
        PathBuf::from("/tmp/test/document.pdf"),
        PathBuf::from("/tmp/test/src/main.rs"),
        PathBuf::from("/tmp/test/cache/temp.log"),
        PathBuf::from("/tmp/test/config.toml"),
    ];

    for (i, path) in test_files.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("evaluate", i), path, |b, path| {
            b.iter(|| black_box(evaluator.evaluate(black_box(path)).unwrap()));
        });
    }

    group.finish();
}

#[cfg(feature = "smart")]
/// ファイル重要度評価（キャッシュ）ベンチマーク
fn bench_importance_cached(c: &mut Criterion) {
    let mut group = c.benchmark_group("importance_cached");

    let evaluator = ImportanceEvaluator::new();
    let test_path = PathBuf::from("/tmp/test/document.pdf");

    // ウォームアップ（キャッシュに登録）
    let _ = evaluator.evaluate_cached(&test_path);

    group.bench_function("cache_hit", |b| {
        b.iter(|| black_box(evaluator.evaluate_cached(black_box(&test_path)).unwrap()));
    });

    group.bench_function("cache_miss", |b| {
        let new_path = PathBuf::from("/tmp/test/new_file.txt");
        b.iter(|| black_box(evaluator.evaluate_cached(black_box(&new_path)).unwrap()));
    });

    group.finish();
}

#[cfg(feature = "smart")]
/// 除外パターン推奨ベンチマーク
fn bench_exclude_recommendation(c: &mut Criterion) {
    let mut group = c.benchmark_group("exclude_recommendation");

    // 一時ディレクトリ作成
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // テスト用ファイル作成
    std::fs::create_dir_all(base_path.join("node_modules")).unwrap();
    std::fs::create_dir_all(base_path.join("cache")).unwrap();
    std::fs::write(base_path.join("node_modules/test.js"), b"test").unwrap();
    std::fs::write(base_path.join("cache/temp.log"), b"log").unwrap();

    let engine = ExcludeRecommendationEngine::new();

    group.bench_function("suggest_exclude_patterns", |b| {
        b.iter(|| {
            black_box(
                engine
                    .suggest_exclude_patterns(black_box(base_path))
                    .unwrap(),
            )
        });
    });

    group.finish();
}

#[cfg(feature = "smart")]
/// 統計計算パフォーマンスベンチマーク（目標: < 1ms/1000件）
fn bench_statistics_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics_computation");

    for count in [100, 500, 1000, 5000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("mean_calculation", count),
            &count,
            |b, &count| {
                let histories = create_mock_histories(count, 50_000_000);
                b.iter(|| {
                    let sizes: Vec<f64> = histories.iter().map(|h| h.total_bytes as f64).collect();
                    let mean: f64 = black_box(sizes.iter().sum::<f64>() / sizes.len() as f64);
                    black_box(mean)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("variance_calculation", count),
            &count,
            |b, &count| {
                let histories = create_mock_histories(count, 50_000_000);
                b.iter(|| {
                    let sizes: Vec<f64> = histories.iter().map(|h| h.total_bytes as f64).collect();
                    let mean: f64 = sizes.iter().sum::<f64>() / sizes.len() as f64;
                    let variance: f64 = black_box(
                        sizes
                            .iter()
                            .map(|&x| {
                                let diff = x - mean;
                                diff * diff
                            })
                            .sum::<f64>()
                            / sizes.len() as f64,
                    );
                    black_box(variance)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("std_dev_calculation", count),
            &count,
            |b, &count| {
                let histories = create_mock_histories(count, 50_000_000);
                b.iter(|| {
                    let sizes: Vec<f64> = histories.iter().map(|h| h.total_bytes as f64).collect();
                    let mean: f64 = sizes.iter().sum::<f64>() / sizes.len() as f64;
                    let variance: f64 = sizes
                        .iter()
                        .map(|&x| {
                            let diff = x - mean;
                            diff * diff
                        })
                        .sum::<f64>()
                        / sizes.len() as f64;
                    let std_dev: f64 = black_box(variance.sqrt());
                    black_box(std_dev)
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "smart")]
/// 大規模データ処理ベンチマーク（スケーラビリティ検証）
fn bench_large_scale_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_scale_processing");
    group.sample_size(10); // 大規模データのためサンプルサイズを削減

    for count in [1000, 5000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("anomaly_detection_large", count),
            &count,
            |b, &count| {
                let detector = AnomalyDetector::default_detector();
                let histories = create_mock_histories(count, 50_000_000);
                let current_size = BackupSize::new(150_000_000);

                b.iter(|| {
                    black_box(
                        detector
                            .detect_size_anomaly(black_box(&histories), black_box(current_size))
                            .unwrap(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("pattern_analysis_large", count),
            &count,
            |b, &count| {
                let analyzer = PatternAnalyzer::new();
                let mut histories = create_mock_histories(count, 50_000_000);
                // 20%を失敗にする
                for (i, history) in histories.iter_mut().enumerate() {
                    if i % 5 == 0 {
                        history.status = BackupStatus::Failed;
                        history.error_message = Some("Test error".to_string());
                    }
                }

                b.iter(|| {
                    black_box(
                        analyzer
                            .calculate_failure_rate(black_box(&histories))
                            .unwrap(),
                    )
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "smart")]
/// エンドツーエンド推奨エンジンベンチマーク
fn bench_recommendation_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("recommendation_pipeline");
    group.sample_size(10);

    // 一時ディレクトリ作成
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // 大量のテストファイルを作成（1000ファイル）
    for i in 0..1000 {
        let dir_name = format!("dir_{}", i / 100);
        let file_path = base_path.join(&dir_name).join(format!("file_{}.txt", i));
        std::fs::create_dir_all(file_path.parent().unwrap()).ok();
        std::fs::write(&file_path, format!("test content {}", i)).ok();
    }

    // キャッシュディレクトリ
    std::fs::create_dir_all(base_path.join("node_modules")).unwrap();
    std::fs::create_dir_all(base_path.join("cache")).unwrap();
    for i in 0..100 {
        std::fs::write(
            base_path
                .join("node_modules")
                .join(format!("module_{}.js", i)),
            b"module",
        )
        .ok();
    }

    let engine = ExcludeRecommendationEngine::new();
    let evaluator = ImportanceEvaluator::new();

    group.bench_function("full_scan_and_recommend", |b| {
        b.iter(|| {
            // スキャン
            let recommendations = black_box(
                engine
                    .suggest_exclude_patterns(black_box(base_path))
                    .unwrap(),
            );

            // 重要度評価（サンプル）
            for i in 0..10 {
                let file_path = base_path.join("dir_0").join(format!("file_{}.txt", i));
                if file_path.exists() {
                    black_box(evaluator.evaluate(&file_path).ok());
                }
            }

            black_box(recommendations)
        });
    });

    group.finish();
}

#[cfg(feature = "smart")]
/// メモリ使用量ベンチマーク（定性的評価）
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(10);

    for count in [1000, 5000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("histories_allocation", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    let histories = black_box(create_mock_histories(count, 50_000_000));
                    // メモリ使用量はOS側で測定（ここでは割り当て時間を測定）
                    black_box(histories.len())
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("detector_memory", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    let detector = black_box(AnomalyDetector::default_detector());
                    let histories = black_box(create_mock_histories(count, 50_000_000));
                    let current_size = BackupSize::new(150_000_000);

                    let result = black_box(
                        detector
                            .detect_size_anomaly(&histories, current_size)
                            .unwrap(),
                    );
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "smart")]
/// キャッシュ効率性ベンチマーク
fn bench_cache_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_efficiency");

    let evaluator = ImportanceEvaluator::new();

    // キャッシュヒット率のテスト
    group.bench_function("cache_hit_ratio_test", |b| {
        let paths: Vec<PathBuf> = (0..100)
            .map(|i| PathBuf::from(format!("/tmp/test/file_{}.txt", i)))
            .collect();

        // ウォームアップ（全パスをキャッシュに登録）
        for path in &paths {
            let _ = evaluator.evaluate_cached(path);
        }

        b.iter(|| {
            // 80%キャッシュヒット、20%キャッシュミス
            for (i, path) in paths.iter().enumerate() {
                if i % 5 == 0 {
                    // キャッシュミス（新しいパス）
                    let new_path = PathBuf::from(format!("/tmp/test/new_file_{}.txt", i));
                    black_box(evaluator.evaluate_cached(&new_path).ok());
                } else {
                    // キャッシュヒット
                    black_box(evaluator.evaluate_cached(path).ok());
                }
            }
        });
    });

    group.finish();
}

#[cfg(feature = "smart")]
criterion_group!(
    benches,
    bench_anomaly_detection,
    bench_disk_prediction,
    bench_pattern_analysis,
    bench_importance_evaluation,
    bench_importance_cached,
    bench_exclude_recommendation,
    bench_statistics_computation,
    bench_large_scale_processing,
    bench_recommendation_pipeline,
    bench_memory_usage,
    bench_cache_efficiency
);

#[cfg(feature = "smart")]
criterion_main!(benches);

#[cfg(not(feature = "smart"))]
fn main() {
    eprintln!("AI benchmarks require the 'ai' feature to be enabled");
    eprintln!("Run with: cargo bench --features ai");
}
