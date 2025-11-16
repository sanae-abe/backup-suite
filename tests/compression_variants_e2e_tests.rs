// compression_variants_e2e_tests.rs - 圧縮形式バリエーションE2Eテスト
//
// Phase 2: 完全性向上（Priority: Medium）
// 実装期限: 2025-11-20
//
// このファイルはGzip圧縮と圧縮形式自動検出機能を検証します。
//
// テストシナリオ:
// 1. Gzip圧縮バックアップ→復元 - Zstdとの圧縮率比較
// 2. 圧縮形式自動検出 - 復元時の自動判別

use anyhow::Result;
use backup_suite::compression::CompressionType;
use backup_suite::core::{BackupRunner, Config, RestoreEngine};
use backup_suite::{Priority, Target};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// =============================================================================
// テストヘルパー関数
// =============================================================================

/// テスト用のソースディレクトリを作成
fn create_test_source(temp: &TempDir) -> std::path::PathBuf {
    let source = temp.path().join("source");
    fs::create_dir_all(&source).unwrap();

    // 圧縮効果を確認できる繰り返しパターンのファイル
    let repetitive_content = "This is a test file with repetitive content. ".repeat(100);
    fs::write(source.join("file1.txt"), &repetitive_content).unwrap();
    fs::write(source.join("file2.txt"), &repetitive_content).unwrap();
    fs::write(source.join("file3.txt"), &repetitive_content).unwrap();

    // バイナリファイル（圧縮効果が低い）
    let random_content = vec![0xFFu8; 1024 * 10]; // 10KB
    fs::write(source.join("binary.bin"), &random_content).unwrap();

    source
}

/// ファイルの内容が一致することを確認
fn assert_file_content(path: &Path, expected: &str) {
    assert!(path.exists(), "File not found: {:?}", path);
    let content = fs::read_to_string(path).unwrap();
    assert_eq!(content, expected, "File content mismatch: {:?}", path);
}

/// バックアップディレクトリの総サイズを計算
fn calculate_backup_size(backup_dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
    for entry in walkdir::WalkDir::new(backup_dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            total_size += fs::metadata(entry.path())?.len();
        }
    }
    Ok(total_size)
}

// =============================================================================
// E2E Scenario 1: Gzip圧縮バックアップ→復元（Zstdとの比較）
// =============================================================================

#[test]
fn test_e2e_gzip_compression() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup_gzip = temp.path().join("backup_gzip");
    let backup_zstd = temp.path().join("backup_zstd");
    let backup_none = temp.path().join("backup_none");
    let restore = temp.path().join("restore");

    fs::create_dir_all(&backup_gzip)?;
    fs::create_dir_all(&backup_zstd)?;
    fs::create_dir_all(&backup_none)?;

    // ステップ1: Gzip圧縮バックアップ実行
    let mut config_gzip = Config::default();
    config_gzip.backup.destination = backup_gzip.clone();
    config_gzip.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner_gzip = BackupRunner::new(config_gzip, false)
        .with_progress(false)
        .with_compression(CompressionType::Gzip, 6); // デフォルトレベル6

    let result_gzip = runner_gzip.run(None, None)?;
    assert_eq!(result_gzip.failed, 0, "Gzip backup should succeed");

    // ステップ2: Zstd圧縮バックアップ実行（比較用）
    let mut config_zstd = Config::default();
    config_zstd.backup.destination = backup_zstd.clone();
    config_zstd.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner_zstd = BackupRunner::new(config_zstd, false)
        .with_progress(false)
        .with_compression(CompressionType::Zstd, 3);

    let result_zstd = runner_zstd.run(None, None)?;
    assert_eq!(result_zstd.failed, 0, "Zstd backup should succeed");

    // ステップ3: 圧縮なしバックアップ実行（比較用）
    let mut config_none = Config::default();
    config_none.backup.destination = backup_none.clone();
    config_none.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner_none = BackupRunner::new(config_none, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result_none = runner_none.run(None, None)?;
    assert_eq!(
        result_none.failed, 0,
        "Non-compressed backup should succeed"
    );

    // ステップ4: バックアップサイズ比較
    let size_gzip = calculate_backup_size(&backup_gzip.join(&result_gzip.backup_name))?;
    let size_zstd = calculate_backup_size(&backup_zstd.join(&result_zstd.backup_name))?;
    let size_none = calculate_backup_size(&backup_none.join(&result_none.backup_name))?;

    // 圧縮効果の検証（繰り返しパターンのファイルは圧縮されるはず）
    assert!(
        size_gzip < size_none,
        "Gzip should reduce size: gzip={}, none={}",
        size_gzip,
        size_none
    );
    assert!(
        size_zstd < size_none,
        "Zstd should reduce size: zstd={}, none={}",
        size_zstd,
        size_none
    );

    println!("圧縮率比較:");
    println!("  圧縮なし: {} bytes", size_none);
    println!(
        "  Gzip: {} bytes ({:.1}% of original)",
        size_gzip,
        (size_gzip as f64 / size_none as f64) * 100.0
    );
    println!(
        "  Zstd: {} bytes ({:.1}% of original)",
        size_zstd,
        (size_zstd as f64 / size_none as f64) * 100.0
    );

    // ステップ5: Gzip復元実行
    fs::create_dir_all(&restore)?;
    let actual_backup = backup_gzip.join(&result_gzip.backup_name);

    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    let restore_result = restore_engine.restore(&actual_backup, &restore, None)?;

    assert_eq!(restore_result.failed, 0, "Gzip restore should succeed");
    assert_eq!(
        restore_result.total_files, 4,
        "All 4 files should be restored"
    );

    // ステップ6: 復元されたファイルの内容確認
    // 注: ディレクトリバックアップではディレクトリ名も保持されるため、test/source/ 配下に復元される
    let restored_root = restore.join("test/source");
    let repetitive_content = "This is a test file with repetitive content. ".repeat(100);
    assert_file_content(&restored_root.join("file1.txt"), &repetitive_content);
    assert_file_content(&restored_root.join("file2.txt"), &repetitive_content);
    assert_file_content(&restored_root.join("file3.txt"), &repetitive_content);

    // バイナリファイルの内容確認
    let binary_content = fs::read(restored_root.join("binary.bin"))?;
    assert_eq!(binary_content.len(), 1024 * 10, "Binary file size mismatch");

    Ok(())
}

// =============================================================================
// E2E Scenario 2: 圧縮形式自動検出
// =============================================================================

#[test]
fn test_e2e_compression_auto_detection() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup_gzip = temp.path().join("backup_gzip");
    let backup_zstd = temp.path().join("backup_zstd");
    let backup_none = temp.path().join("backup_none");
    let restore_gzip = temp.path().join("restore_gzip");
    let restore_zstd = temp.path().join("restore_zstd");
    let restore_none = temp.path().join("restore_none");

    fs::create_dir_all(&backup_gzip)?;
    fs::create_dir_all(&backup_zstd)?;
    fs::create_dir_all(&backup_none)?;

    // ステップ1: 各圧縮形式でバックアップ作成
    let mut config_gzip = Config::default();
    config_gzip.backup.destination = backup_gzip.clone();
    config_gzip.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner_gzip = BackupRunner::new(config_gzip, false)
        .with_progress(false)
        .with_compression(CompressionType::Gzip, 6);

    let result_gzip = runner_gzip.run(None, None)?;

    let mut config_zstd = Config::default();
    config_zstd.backup.destination = backup_zstd.clone();
    config_zstd.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner_zstd = BackupRunner::new(config_zstd, false)
        .with_progress(false)
        .with_compression(CompressionType::Zstd, 3);

    let result_zstd = runner_zstd.run(None, None)?;

    let mut config_none = Config::default();
    config_none.backup.destination = backup_none.clone();
    config_none.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner_none = BackupRunner::new(config_none, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result_none = runner_none.run(None, None)?;

    // ステップ2: RestoreEngineが圧縮形式を自動検出して復元
    // （圧縮形式を指定しないで復元）
    fs::create_dir_all(&restore_gzip)?;
    fs::create_dir_all(&restore_zstd)?;
    fs::create_dir_all(&restore_none)?;

    let mut restore_engine_gzip = RestoreEngine::new(false).with_progress(false);
    let restore_result_gzip = restore_engine_gzip.restore(
        &backup_gzip.join(&result_gzip.backup_name),
        &restore_gzip,
        None,
    )?;

    let mut restore_engine_zstd = RestoreEngine::new(false).with_progress(false);
    let restore_result_zstd = restore_engine_zstd.restore(
        &backup_zstd.join(&result_zstd.backup_name),
        &restore_zstd,
        None,
    )?;

    let mut restore_engine_none = RestoreEngine::new(false).with_progress(false);
    let restore_result_none = restore_engine_none.restore(
        &backup_none.join(&result_none.backup_name),
        &restore_none,
        None,
    )?;

    // ステップ3: すべての圧縮形式で復元が成功することを確認
    assert_eq!(
        restore_result_gzip.failed, 0,
        "Gzip auto-detection restore should succeed"
    );
    assert_eq!(
        restore_result_zstd.failed, 0,
        "Zstd auto-detection restore should succeed"
    );
    assert_eq!(
        restore_result_none.failed, 0,
        "Non-compressed restore should succeed"
    );

    // ステップ4: 各復元結果の内容確認
    // 注: ディレクトリバックアップではディレクトリ名も保持されるため、test/source/ 配下に復元される
    let repetitive_content = "This is a test file with repetitive content. ".repeat(100);

    let restored_gzip = restore_gzip.join("test/source");
    assert_file_content(&restored_gzip.join("file1.txt"), &repetitive_content);
    assert_file_content(&restored_gzip.join("file2.txt"), &repetitive_content);

    let restored_zstd = restore_zstd.join("test/source");
    assert_file_content(&restored_zstd.join("file1.txt"), &repetitive_content);
    assert_file_content(&restored_zstd.join("file2.txt"), &repetitive_content);

    let restored_none = restore_none.join("test/source");
    assert_file_content(&restored_none.join("file1.txt"), &repetitive_content);
    assert_file_content(&restored_none.join("file2.txt"), &repetitive_content);

    println!("✅ 圧縮形式自動検出テスト成功:");
    println!("  Gzip: {} files restored", restore_result_gzip.total_files);
    println!("  Zstd: {} files restored", restore_result_zstd.total_files);
    println!("  None: {} files restored", restore_result_none.total_files);

    Ok(())
}
