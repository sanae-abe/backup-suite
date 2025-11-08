use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// セキュリティ関連の関数（実装を想定）
fn safe_join(base: &Path, child: &Path) -> Result<PathBuf, String> {
    use std::path::Component;

    // 相対パスから .. を除去
    let normalized: PathBuf = child
        .components()
        .filter(|c| !matches!(c, Component::ParentDir))
        .collect();

    let result = base.join(&normalized);

    // 結果がベースディレクトリ配下にあることを確認
    result
        .canonicalize()
        .ok()
        .and_then(|canonical| {
            base.canonicalize().ok().and_then(|base_canonical| {
                if canonical.starts_with(&base_canonical) {
                    Some(result)
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| format!("不正なパス: {child:?}"))
}

fn sanitize_path_component(name: &str) -> String {
    name.chars()
        .filter(|&c| c.is_alphanumeric() || "-_.".contains(c))
        .collect()
}

fn check_read_permission(path: &Path) -> Result<(), String> {
    let metadata = std::fs::metadata(path).map_err(|e| format!("メタデータ取得失敗: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        if mode & 0o400 == 0 {
            return Err(format!("読み取り権限がありません: {path:?}"));
        }
    }

    Ok(())
}

fn validate_path_security(path: &Path) -> Result<(), String> {
    // パスの文字列表現を取得
    let path_str = path.to_string_lossy();

    // 危険なパターンをチェック
    let dangerous_patterns = ["..", "~", "/etc/", "/root/", "C:\\Windows\\"];

    for pattern in &dangerous_patterns {
        if path_str.contains(pattern) {
            return Err(format!("危険なパターン検出: {pattern}"));
        }
    }

    Ok(())
}

// ベンチマーク関数

fn bench_safe_join(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    let test_cases = vec![
        ("simple.txt", "単純なファイル名"),
        ("dir/file.txt", "ディレクトリ付き"),
        ("deep/nested/path/file.txt", "深い階層"),
        ("file-with-special_chars.txt", "特殊文字含む"),
    ];

    let mut group = c.benchmark_group("safe_join");

    for (child, name) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &child, |b, child| {
            b.iter(|| {
                let child_path = PathBuf::from(child);
                safe_join(black_box(base), black_box(&child_path))
            });
        });
    }

    group.finish();
}

fn bench_safe_join_malicious(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    let malicious_cases = vec![
        ("../../../etc/passwd", "パストラバーサル"),
        ("..\\..\\..\\windows\\system32\\config\\sam", "Windows攻撃"),
        ("/absolute/path/attack", "絶対パス"),
        ("normal/../../../etc/shadow", "混合パス"),
    ];

    let mut group = c.benchmark_group("safe_join_malicious");

    for (child, name) in malicious_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &child, |b, child| {
            b.iter(|| {
                let child_path = PathBuf::from(child);
                let _ = safe_join(black_box(base), black_box(&child_path));
            });
        });
    }

    group.finish();
}

fn bench_sanitize_path_component(c: &mut Criterion) {
    let test_cases = vec![
        ("normal-file.txt", "正常なファイル名"),
        ("file_with_spaces   .txt", "スペース含む"),
        ("file@#$%^&*().txt", "特殊記号含む"),
        ("日本語ファイル名.txt", "日本語含む"),
        (
            "very-long-filename-with-many-characters-to-test-performance.txt",
            "長いファイル名",
        ),
    ];

    let mut group = c.benchmark_group("sanitize_path_component");

    for (input, name) in test_cases {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &input, |b, input| {
            b.iter(|| sanitize_path_component(black_box(input)));
        });
    }

    group.finish();
}

fn bench_check_read_permission(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test content").unwrap();

    c.bench_function("check_read_permission", |b| {
        b.iter(|| check_read_permission(black_box(&file_path)));
    });
}

fn bench_validate_path_security(c: &mut Criterion) {
    let test_paths = vec![
        (PathBuf::from("/home/user/file.txt"), "安全なパス"),
        (PathBuf::from("/tmp/test.txt"), "一時ディレクトリ"),
        (PathBuf::from("relative/path.txt"), "相対パス"),
    ];

    let mut group = c.benchmark_group("validate_path_security");

    for (path, name) in test_paths {
        group.bench_with_input(BenchmarkId::from_parameter(name), &path, |b, path| {
            b.iter(|| validate_path_security(black_box(path)));
        });
    }

    group.finish();
}

fn bench_security_full_pipeline(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();
    let file_path = base.join("test.txt");
    fs::write(&file_path, "test content").unwrap();

    c.bench_function("security_full_pipeline", |b| {
        b.iter(|| {
            let child = PathBuf::from("test.txt");

            // 1. パス結合
            let joined = safe_join(black_box(base), black_box(&child));

            if let Ok(path) = joined {
                // 2. セキュリティ検証
                let _ = validate_path_security(black_box(&path));

                // 3. 権限チェック
                let _ = check_read_permission(black_box(&path));
            }
        });
    });
}

fn bench_concurrent_security_checks(c: &mut Criterion) {
    use rayon::prelude::*;

    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    // 100個のテストファイルを作成
    let files: Vec<_> = (0..100)
        .map(|i| {
            let path = base.join(format!("file_{i}.txt"));
            fs::write(&path, format!("content {i}")).unwrap();
            PathBuf::from(format!("file_{i}.txt"))
        })
        .collect();

    let mut group = c.benchmark_group("concurrent_security_checks");
    group.throughput(Throughput::Elements(100));

    group.bench_function("sequential", |b| {
        b.iter(|| {
            for file in &files {
                let _ = safe_join(black_box(base), black_box(file));
            }
        });
    });

    group.bench_function("parallel", |b| {
        b.iter(|| {
            files.par_iter().for_each(|file| {
                let _ = safe_join(black_box(base), black_box(file));
            });
        });
    });

    group.finish();
}

fn bench_path_canonicalization(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();
    let nested = base.join("a").join("b").join("c");
    fs::create_dir_all(&nested).unwrap();
    let file = nested.join("test.txt");
    fs::write(&file, "test").unwrap();

    let mut group = c.benchmark_group("path_canonicalization");

    group.bench_function("shallow", |b| {
        let shallow = base.join("test.txt");
        b.iter(|| shallow.canonicalize());
    });

    group.bench_function("deep", |b| {
        b.iter(|| file.canonicalize());
    });

    group.finish();
}

fn bench_regex_pattern_matching(c: &mut Criterion) {
    use regex::Regex;

    let dangerous_pattern = Regex::new(r"\.\.|~|/etc/|/root/|C:\\Windows\\").unwrap();

    let test_cases = vec![
        ("/home/user/file.txt", "安全なパス"),
        ("../../../etc/passwd", "危険なパス"),
        ("/tmp/test.txt", "一時ディレクトリ"),
    ];

    let mut group = c.benchmark_group("regex_pattern_matching");

    for (path, name) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &path, |b, path| {
            b.iter(|| dangerous_pattern.is_match(black_box(path)));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_safe_join,
    bench_safe_join_malicious,
    bench_sanitize_path_component,
    bench_check_read_permission,
    bench_validate_path_security,
    bench_security_full_pipeline,
    bench_concurrent_security_checks,
    bench_path_canonicalization,
    bench_regex_pattern_matching,
);

criterion_main!(benches);
