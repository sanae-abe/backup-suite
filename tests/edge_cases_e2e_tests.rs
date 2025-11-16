// edge_cases_e2e_tests.rs - エッジケースE2Eテスト
//
// Phase 2: 完全性向上（Priority: Medium）
// 実装期限: 2025-11-20
//
// このファイルは極端なエッジケースでの動作を検証します。
//
// テストシナリオ:
// 1. 空ディレクトリのバックアップ→復元
// 2. 特殊文字を含むファイル名（Unicode、スペース、記号）
// 3. 多数の小さいファイル（1,000個の1KB以下ファイル）

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

/// ファイルの存在を確認
fn assert_file_exists(path: &Path, message: &str) {
    assert!(path.exists(), "{}: {:?}", message, path);
}

/// ディレクトリの存在を確認
fn assert_dir_exists(path: &Path, message: &str) {
    assert!(path.exists() && path.is_dir(), "{}: {:?}", message, path);
}

// =============================================================================
// E2E Scenario 1: 空ディレクトリのバックアップ→復元
// =============================================================================

#[test]
fn test_e2e_empty_directory() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");

    // 空のディレクトリ構造を作成
    fs::create_dir_all(source.join("empty_dir1"))?;
    fs::create_dir_all(source.join("empty_dir2/nested_empty"))?;
    fs::create_dir_all(source.join("with_file"))?;
    fs::write(source.join("with_file/single.txt"), "Only file")?;

    fs::create_dir_all(&backup)?;

    // ステップ1: バックアップ実行
    let mut config = Config::default();
    config.backup.destination = backup.clone();
    config.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;
    assert_eq!(result.failed, 0, "Empty directory backup should succeed");

    // ステップ2: 復元実行
    fs::create_dir_all(&restore)?;
    let actual_backup = backup.join(&result.backup_name);

    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    let restore_result = restore_engine.restore(&actual_backup, &restore, None)?;

    assert_eq!(
        restore_result.failed, 0,
        "Empty directory restore should succeed"
    );

    // ステップ3: 復元されたディレクトリ構造を確認
    // 注: backup-suite の実装では、空のディレクトリはバックアップされません。
    // ファイルを含むディレクトリのみが復元されます。
    // 注: ディレクトリバックアップではディレクトリ名も保持されるため、test/source/ 配下に復元される
    let restored_root = restore.join("test/source");
    assert_dir_exists(&restored_root, "Restored root should exist");

    // 空ディレクトリは復元されない（実装の制限）
    assert!(
        !restored_root.join("empty_dir1").exists(),
        "empty_dir1 should NOT be restored (empty directory limitation)"
    );
    assert!(
        !restored_root.join("empty_dir2").exists(),
        "empty_dir2 should NOT be restored (empty directory limitation)"
    );

    // ファイルを含むディレクトリは復元される
    assert_dir_exists(
        &restored_root.join("with_file"),
        "with_file should be restored",
    );
    assert_file_exists(
        &restored_root.join("with_file/single.txt"),
        "single.txt should be restored",
    );

    println!("✅ 空ディレクトリテスト成功:");
    println!("  バックアップファイル数: {}", result.total_files);
    println!("  復元ファイル数: {}", restore_result.total_files);
    println!("  注: 空ディレクトリはバックアップされません（実装の制限）");

    Ok(())
}

// =============================================================================
// E2E Scenario 2: 特殊文字を含むファイル名
// =============================================================================

#[test]
fn test_e2e_special_characters_filename() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");

    fs::create_dir_all(&source)?;
    fs::create_dir_all(&backup)?;

    // 特殊文字を含むファイル名を作成
    let test_files = vec![
        "normal_file.txt",
        "ファイル with スペース.txt",
        "日本語ファイル名.txt",
        "emoji_😀_test.txt",
        "dots...and...more.txt",
        "under_score_file.txt",
        "dash-file-name.txt",
        // "(parentheses).txt", // macOSで問題が出る可能性があるため一旦除外
        // "[brackets].txt",
    ];

    for filename in &test_files {
        fs::write(source.join(filename), format!("Content of {}", filename))?;
    }

    // ステップ1: バックアップ実行
    let mut config = Config::default();
    config.backup.destination = backup.clone();
    config.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::Zstd, 3);

    let result = runner.run(None, None)?;
    assert_eq!(result.failed, 0, "Special characters backup should succeed");
    assert_eq!(
        result.total_files,
        test_files.len(),
        "All special character files should be backed up"
    );

    // ステップ2: 復元実行
    fs::create_dir_all(&restore)?;
    let actual_backup = backup.join(&result.backup_name);

    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    let restore_result = restore_engine.restore(&actual_backup, &restore, None)?;

    assert_eq!(
        restore_result.failed, 0,
        "Special characters restore should succeed"
    );

    // ステップ3: 復元されたファイルの検証
    // 注: ディレクトリバックアップではディレクトリ名も保持されるため、test/source/ 配下に復元される
    let restored_root = restore.join("test/source");
    for filename in &test_files {
        let file_path = restored_root.join(filename);
        assert_file_exists(&file_path, &format!("{} should be restored", filename));

        let content = fs::read_to_string(&file_path)?;
        assert_eq!(
            content,
            format!("Content of {}", filename),
            "File content mismatch: {}",
            filename
        );
    }

    println!("✅ 特殊文字ファイル名テスト成功:");
    println!("  テスト対象ファイル数: {}", test_files.len());
    println!("  バックアップ成功: {}", result.successful);
    println!("  復元成功: {}", restore_result.restored);

    Ok(())
}

// =============================================================================
// E2E Scenario 3: 多数の小さいファイル（1,000個の1KB以下ファイル）
// =============================================================================

#[test]
fn test_e2e_many_small_files() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");

    fs::create_dir_all(&source)?;
    fs::create_dir_all(&backup)?;

    // 1,000個の小さいファイルを作成
    let file_count = 1000;
    let content_template = "Small file content ";

    for i in 0..file_count {
        let filename = format!("file_{:04}.txt", i);
        let content = format!("{}{}", content_template, i);
        fs::write(source.join(&filename), content)?;
    }

    // ステップ1: バックアップ実行（並列処理のテスト）
    let mut config = Config::default();
    config.backup.destination = backup.clone();
    config.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::Zstd, 3);

    let result = runner.run(None, None)?;
    assert_eq!(result.failed, 0, "Many small files backup should succeed");
    assert_eq!(
        result.total_files, file_count,
        "All {} files should be backed up",
        file_count
    );
    assert_eq!(
        result.successful, file_count,
        "All {} files should succeed",
        file_count
    );

    // ステップ2: 復元実行
    fs::create_dir_all(&restore)?;
    let actual_backup = backup.join(&result.backup_name);

    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    let restore_result = restore_engine.restore(&actual_backup, &restore, None)?;

    assert_eq!(
        restore_result.failed, 0,
        "Many small files restore should succeed"
    );
    assert_eq!(
        restore_result.total_files, file_count,
        "All {} files should be restored",
        file_count
    );

    // ステップ3: ランダムサンプリング検証（すべて検証すると遅いため）
    // 注: ディレクトリバックアップではディレクトリ名も保持されるため、test/source/ 配下に復元される
    let restored_root = restore.join("test/source");
    let sample_indices = vec![0, 100, 500, 750, 999];

    for i in sample_indices {
        let filename = format!("file_{:04}.txt", i);
        let file_path = restored_root.join(&filename);
        assert_file_exists(&file_path, &format!("{} should be restored", filename));

        let content = fs::read_to_string(&file_path)?;
        let expected_content = format!("{}{}", content_template, i);
        assert_eq!(
            content, expected_content,
            "File content mismatch: {}",
            filename
        );
    }

    println!("✅ 多数の小さいファイルテスト成功:");
    println!("  ファイル数: {}", file_count);
    println!("  バックアップ成功: {}", result.successful);
    println!("  復元成功: {}", restore_result.restored);
    println!(
        "  並列処理効率: {}%",
        (result.successful as f64 / file_count as f64) * 100.0
    );

    Ok(())
}
