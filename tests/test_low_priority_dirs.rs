//! 低優先度ディレクトリの判定テスト

use backup_suite::smart::recommendation::ImportanceEvaluator;
use backup_suite::Priority;
use std::fs;

#[test]
fn test_cache_directory_is_low_priority() {
    let evaluator = ImportanceEvaluator::new();
    let temp_dir = std::env::temp_dir();
    let cache_dir = temp_dir.join("test_low_priority_cache");

    // キャッシュディレクトリを作成
    fs::create_dir_all(&cache_dir).unwrap();

    // 評価
    let result = evaluator.evaluate(&cache_dir).unwrap();

    // 検証
    assert_eq!(
        result.priority(),
        &Priority::Low,
        "cache ディレクトリは Low 優先度であるべき"
    );
    assert_eq!(
        result.score().get(),
        20,
        "cache ディレクトリのスコアは 20 であるべき"
    );

    // クリーンアップ
    let _ = fs::remove_dir_all(&cache_dir);
}

#[test]
fn test_logs_directory_is_low_priority() {
    let evaluator = ImportanceEvaluator::new();
    let temp_dir = std::env::temp_dir();
    let logs_dir = temp_dir.join("test_low_priority_logs");

    fs::create_dir_all(&logs_dir).unwrap();

    let result = evaluator.evaluate(&logs_dir).unwrap();

    assert_eq!(
        result.priority(),
        &Priority::Low,
        "logs ディレクトリは Low 優先度であるべき"
    );
    assert_eq!(
        result.score().get(),
        20,
        "logs ディレクトリのスコアは 20 であるべき"
    );

    let _ = fs::remove_dir_all(&logs_dir);
}

#[test]
fn test_archive_directory_is_low_priority() {
    let evaluator = ImportanceEvaluator::new();
    let temp_dir = std::env::temp_dir();
    let archive_dir = temp_dir.join("test_low_priority_archive");

    fs::create_dir_all(&archive_dir).unwrap();

    let result = evaluator.evaluate(&archive_dir).unwrap();

    assert_eq!(
        result.priority(),
        &Priority::Low,
        "archive ディレクトリは Low 優先度であるべき"
    );
    assert_eq!(
        result.score().get(),
        20,
        "archive ディレクトリのスコアは 20 であるべき"
    );

    let _ = fs::remove_dir_all(&archive_dir);
}

#[test]
fn test_build_directory_is_low_priority() {
    let evaluator = ImportanceEvaluator::new();
    let temp_dir = std::env::temp_dir();
    let build_dir = temp_dir.join("test_low_priority_build");

    fs::create_dir_all(&build_dir).unwrap();

    let result = evaluator.evaluate(&build_dir).unwrap();

    assert_eq!(
        result.priority(),
        &Priority::Low,
        "build ディレクトリは Low 優先度であるべき"
    );
    assert_eq!(result.score().get(), 20);

    let _ = fs::remove_dir_all(&build_dir);
}

#[test]
fn test_target_directory_is_low_priority() {
    let evaluator = ImportanceEvaluator::new();
    let temp_dir = std::env::temp_dir();
    let target_dir = temp_dir.join("test_low_priority_target");

    fs::create_dir_all(&target_dir).unwrap();

    let result = evaluator.evaluate(&target_dir).unwrap();

    assert_eq!(
        result.priority(),
        &Priority::Low,
        "target ディレクトリ（Rustビルド成果物）は Low 優先度であるべき"
    );
    assert_eq!(result.score().get(), 20);

    let _ = fs::remove_dir_all(&target_dir);
}

#[test]
fn test_node_modules_directory_is_low_priority() {
    let evaluator = ImportanceEvaluator::new();
    let temp_dir = std::env::temp_dir();
    let node_modules_dir = temp_dir.join("test_low_priority_node_modules");

    fs::create_dir_all(&node_modules_dir).unwrap();

    let result = evaluator.evaluate(&node_modules_dir).unwrap();

    assert_eq!(
        result.priority(),
        &Priority::Low,
        "node_modules ディレクトリは Low 優先度であるべき"
    );
    assert_eq!(result.score().get(), 20);

    let _ = fs::remove_dir_all(&node_modules_dir);
}

#[test]
fn test_important_directory_is_not_low_priority() {
    use std::io::Write;

    let evaluator = ImportanceEvaluator::new();
    let temp_dir = std::env::temp_dir();
    let docs_dir = temp_dir.join("test_low_priority_documents");

    fs::create_dir_all(&docs_dir).unwrap();

    // 重要なファイルを追加してディレクトリの重要度を上げる
    let report_path = docs_dir.join("report.pdf");
    let mut file = fs::File::create(&report_path).unwrap();
    file.write_all(b"Important report content").unwrap();

    let result = evaluator.evaluate(&docs_dir).unwrap();

    // documents は低優先度パターンに含まれていないので Low ではない
    assert_ne!(
        result.priority(),
        &Priority::Low,
        "documents ディレクトリ（重要ファイル含む）は Low 優先度であるべきではない"
    );
    assert!(
        result.score().get() > 20,
        "documents ディレクトリのスコアは 20 より大きいべき"
    );

    let _ = fs::remove_dir_all(&docs_dir);
}
