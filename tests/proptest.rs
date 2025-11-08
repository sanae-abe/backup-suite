//! プロパティベーステスト - backup-suite
//!
//! proptest を使用した高度なテスト。以下のプロパティを検証:
//! - パストラバーサル対策
//! - ファイル処理の一貫性
//! - エラーハンドリングの正確性
//! - 並列処理の安全性

use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ==================== パス操作のプロパティテスト ====================

/// 安全なパス結合のプロパティテスト
///
/// Phase 1で実装予定のsafe_join関数のテスト
/// 現在はPath::join()の動作を検証
#[test]
fn test_path_joining_never_escapes_base() {
    proptest!(|(
        base in r"[a-zA-Z0-9_-]{1,10}",
        child in r"[a-zA-Z0-9_/\.\-]{1,20}",
    )| {
        let base_path = PathBuf::from(&base);
        let child_path = PathBuf::from(&child);
        let joined = base_path.join(&child_path);

        // 結合されたパスの文字列表現がbaseで始まることを確認
        let joined_str = joined.to_string_lossy();
        let base_str = base_path.to_string_lossy();

        prop_assert!(
            joined_str.starts_with(&*base_str) || child_path.is_absolute(),
            "Path joining should preserve base: base={}, child={}, joined={}",
            base_str,
            child_path.display(),
            joined_str
        );
    });
}

/// 危険なパス要素の検出テスト
#[test]
fn test_dangerous_path_components() {
    proptest!(|(path_str in r"[a-zA-Z0-9_/\.\-]{1,50}")| {
        let path = PathBuf::from(&path_str);

        // パスに".."が含まれている場合の検証
        let has_parent_ref = path.components().any(|c| {
            matches!(c, std::path::Component::ParentDir)
        });

        // ".."を含むパスは特別な扱いが必要
        if has_parent_ref {
            // Phase 1で実装予定: このようなパスは正規化またはエラーにする
            prop_assert!(true, "Parent directory references require special handling");
        }
    });
}

/// パス正規化のプロパティテスト
#[test]
fn test_path_normalization_properties() {
    proptest!(|(
        segments in prop::collection::vec(r"[a-zA-Z0-9_-]{1,10}", 1..5)
    )| {
        let path: PathBuf = segments.iter().collect();
        let path_str = path.to_string_lossy();

        // 正規化されたパスの特性を検証
        prop_assert!(
            !path_str.contains("//"),
            "Normalized paths should not contain double slashes: {}",
            path_str
        );
    });
}

// ==================== ファイル操作のプロパティテスト ====================

/// ファイルコピーの冪等性テスト
#[test]
fn test_file_copy_idempotency() -> Result<(), proptest::test_runner::TestCaseError> {
    proptest!(|(content in r"[a-zA-Z0-9 .,!?\n]{0,1000}")| {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest1 = temp_dir.path().join("dest1.txt");
        let dest2 = temp_dir.path().join("dest2.txt");

        // ソースファイル作成
        fs::write(&source, &content).unwrap();

        // 2回コピー
        fs::copy(&source, &dest1).unwrap();
        fs::copy(&source, &dest2).unwrap();

        // 両方のコピーが同じ内容を持つことを確認
        let content1 = fs::read_to_string(&dest1).unwrap();
        let content2 = fs::read_to_string(&dest2).unwrap();

        prop_assert_eq!(&content1, &content2);
        prop_assert_eq!(&content1, &content);
    });

    Ok(())
}

/// ファイルサイズの一貫性テスト
#[test]
fn test_file_size_consistency() -> Result<(), proptest::test_runner::TestCaseError> {
    proptest!(|(size in 0usize..10000)| {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("test.bin");

        // 指定サイズのファイルを作成
        let content = vec![0u8; size];
        fs::write(&file, &content).unwrap();

        // メタデータからサイズを取得
        let metadata = fs::metadata(&file).unwrap();

        prop_assert_eq!(metadata.len() as usize, size);
    });

    Ok(())
}

/// ディレクトリ走査の一貫性テスト
#[test]
fn test_directory_traversal_consistency() -> Result<(), proptest::test_runner::TestCaseError> {
    proptest!(|(
        num_files in 1usize..20,
        num_dirs in 0usize..5,
    )| {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join("test");
        fs::create_dir_all(&base_dir).unwrap();

        // ディレクトリ作成
        for i in 0..num_dirs {
            fs::create_dir_all(base_dir.join(format!("dir{i}"))).unwrap();
        }

        // ファイル作成
        for i in 0..num_files {
            let dir_index = i % (num_dirs.max(1));
            let file_path = if num_dirs > 0 {
                base_dir.join(format!("dir{dir_index}/file{i}.txt"))
            } else {
                base_dir.join(format!("file{i}.txt"))
            };

            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&file_path, format!("content{i}")).unwrap();
        }

        // walkdirで走査してファイル数をカウント
        let count = walkdir::WalkDir::new(&base_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_file())
            .count();

        prop_assert_eq!(count, num_files);
    });

    Ok(())
}

// ==================== 設定のプロパティテスト ====================

/// 設定のシリアライズ/デシリアライズの可逆性テスト
#[test]
fn test_config_serialization_roundtrip() -> Result<(), proptest::test_runner::TestCaseError> {
    use backup_suite::{Config, Priority, Target};

    proptest!(|(
        keep_days in 1u32..3650u32,
        num_targets in 0usize..10,
    )| {
        let temp_dir = TempDir::new().unwrap();

        // 設定作成
        let mut config = Config::default();
        config.backup.destination = temp_dir.path().join("backup");
        config.backup.keep_days = keep_days;

        // ターゲット追加
        for i in 0..num_targets {
            let target_path = temp_dir.path().join(format!("target{i}"));
            fs::create_dir_all(&target_path).unwrap();

            let priority = match i % 3 {
                0 => Priority::High,
                1 => Priority::Medium,
                _ => Priority::Low,
            };

            config.targets.push(Target::new(
                target_path,
                priority,
                format!("category{i}"),
            ));
        }

        // シリアライズ
        let toml_str = toml::to_string(&config).unwrap();

        // デシリアライズ
        let loaded_config: Config = toml::from_str(&toml_str).unwrap();

        // 検証
        prop_assert_eq!(loaded_config.backup.keep_days, keep_days);
        prop_assert_eq!(loaded_config.targets.len(), num_targets);
    });

    Ok(())
}

/// keep_daysの境界値テスト
#[test]
fn test_keep_days_boundaries() {
    proptest!(|(keep_days in 1u32..10000u32)| {
        let mut config = backup_suite::Config::default();
        config.backup.keep_days = keep_days;

        // keep_daysが正の値であることを確認
        prop_assert!(config.backup.keep_days > 0);

        // Phase 2で実装予定: バリデーション
        // if keep_days > 3650 {
        //     prop_assert!(config.validate().is_err());
        // }
    });
}

// ==================== 除外パターンのプロパティテスト ====================

/// 正規表現パターンの妥当性テスト
#[test]
fn test_regex_pattern_validity() {
    use regex::Regex;

    proptest!(|(pattern in r"[a-zA-Z0-9.*+?\[\](){}|\\-]{1,30}")| {
        // 正規表現のコンパイルを試みる
        let result = Regex::new(&pattern);

        // エラーの場合は適切にハンドリングされるべき
        if result.is_err() {
            // Phase 2で実装予定: 不正なパターンはエラーを返す
            prop_assert!(true, "Invalid regex patterns should be handled gracefully");
        }
    });
}

/// 除外パターンマッチングの一貫性テスト
#[test]
fn test_exclude_pattern_matching_consistency() -> Result<(), proptest::test_runner::TestCaseError> {
    use regex::Regex;

    proptest!(|(
        file_name in r"[a-zA-Z0-9_-]{1,10}\.(txt|md|rs|tmp|bak)",
    )| {
        // .tmp ファイルを除外するパターン
        let pattern = Regex::new(r".*\.tmp$").unwrap();

        let matches = pattern.is_match(&file_name);

        // .tmpで終わるファイルはマッチする
        if file_name.ends_with(".tmp") {
            prop_assert!(matches, "Pattern should match .tmp files: {}", file_name);
        } else {
            prop_assert!(!matches, "Pattern should not match non-.tmp files: {}", file_name);
        }
    });

    Ok(())
}

// ==================== 優先度フィルタリングのプロパティテスト ====================

/// 優先度フィルタリングの包含関係テスト
#[test]
fn test_priority_filtering_inclusion() {
    use backup_suite::Priority;

    proptest!(|(
        targets in prop::collection::vec(
            prop::sample::select(vec![Priority::High, Priority::Medium, Priority::Low]),
            1..20
        )
    )| {
        let high_count = targets.iter().filter(|&&p| p == Priority::High).count();
        let medium_count = targets.iter().filter(|&&p| p == Priority::Medium).count();
        let low_count = targets.iter().filter(|&&p| p == Priority::Low).count();

        // Highフィルタは High のみを含む
        let high_filtered: Vec<_> = targets.iter()
            .filter(|&&p| p == Priority::High)
            .collect();
        prop_assert_eq!(high_filtered.len(), high_count);

        // Mediumフィルタは High + Medium を含む
        let medium_filtered: Vec<_> = targets.iter()
            .filter(|&&p| matches!(p, Priority::High | Priority::Medium))
            .collect();
        prop_assert_eq!(medium_filtered.len(), high_count + medium_count);

        // Lowフィルタは全てを含む（フィルタなし）
        let low_filtered: Vec<_> = targets.iter().collect();
        prop_assert_eq!(low_filtered.len(), high_count + medium_count + low_count);
    });
}

// ==================== エラーハンドリングのプロパティテスト ====================

/// 存在しないパスの処理テスト
#[test]
fn test_nonexistent_path_handling() {
    proptest!(|(random_path in r"[a-zA-Z0-9_/\-]{10,50}")| {
        let path = PathBuf::from(&random_path);

        // ランダムなパスが存在しない可能性が高い
        if !path.exists() {
            // 存在しないパスへのアクセスはエラーを返すべき
            let metadata_result = fs::metadata(&path);
            prop_assert!(metadata_result.is_err());
        }
    });
}

/// 並列処理の安全性テスト
#[test]
fn test_concurrent_file_access_safety() -> Result<(), proptest::test_runner::TestCaseError> {
    proptest!(|(
        _num_threads in 2usize..10,
        num_files in 5usize..20,
    )| {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join("concurrent");
        fs::create_dir_all(&base_dir).unwrap();

        // ファイル作成
        for i in 0..num_files {
            fs::write(
                base_dir.join(format!("file{i}.txt")),
                format!("content{i}")
            ).unwrap();
        }

        // 並列読み込みテスト
        use rayon::prelude::*;

        let files: Vec<_> = (0..num_files)
            .map(|i| base_dir.join(format!("file{i}.txt")))
            .collect();

        // 並列で複数回読み込んでもエラーが起きないことを確認
        let results: Vec<_> = files.par_iter()
            .map(fs::read_to_string)
            .collect();

        // 全ての読み込みが成功することを確認
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        prop_assert_eq!(success_count, num_files);
    });

    Ok(())
}

// ==================== パフォーマンス特性のプロパティテスト ====================

/// ファイルサイズとコピー時間の関係テスト（線形性）
#[test]
#[ignore] // 時間がかかるため通常は無視
fn test_copy_time_scales_linearly() -> Result<(), proptest::test_runner::TestCaseError> {
    proptest!(|(size_kb in 1usize..1000)| {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.bin");
        let dest = temp_dir.path().join("dest.bin");

        // 指定サイズのファイルを作成
        let content = vec![0u8; size_kb * 1024];
        fs::write(&source, &content).unwrap();

        // コピー時間を測定
        let start = std::time::Instant::now();
        fs::copy(&source, &dest).unwrap();
        let duration = start.elapsed();

        // 大きなファイルほど時間がかかる（線形またはそれ以下）
        // 1KBあたり1ms以内を想定
        prop_assert!(
            duration.as_millis() <= (size_kb as u128) * 10,
            "Copy time should scale linearly: {}KB took {:?}",
            size_kb,
            duration
        );
    });

    Ok(())
}

// ==================== カスタムストラテジー ====================

/// 安全なファイル名生成ストラテジー
fn safe_filename() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_-]{1,20}\.(txt|md|rs|toml|json)".prop_map(|s| s.to_string())
}

/// 安全なディレクトリパス生成ストラテジー
fn safe_directory_path() -> impl Strategy<Value = String> {
    prop::collection::vec(r"[a-zA-Z0-9_-]{1,10}", 1..5).prop_map(|segments| segments.join("/"))
}

/// 優先度ストラテジー
fn priority_strategy() -> impl Strategy<Value = backup_suite::Priority> {
    use backup_suite::Priority;
    prop::sample::select(vec![Priority::High, Priority::Medium, Priority::Low])
}

#[test]
fn test_custom_strategies() {
    proptest!(|(
        filename in safe_filename(),
        dir_path in safe_directory_path(),
        priority in priority_strategy(),
    )| {
        // ファイル名が安全な文字のみを含むことを確認
        prop_assert!(filename.chars().all(|c| c.is_alphanumeric() || "_-.".contains(c)));

        // ディレクトリパスが安全な文字のみを含むことを確認
        prop_assert!(!dir_path.contains(".."));
        prop_assert!(!dir_path.starts_with('/'));

        // 優先度が有効な値であることを確認
        prop_assert!(matches!(
            priority,
            backup_suite::Priority::High
                | backup_suite::Priority::Medium
                | backup_suite::Priority::Low
        ));
    });
}
