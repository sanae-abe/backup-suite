// incremental_backup_e2e_tests.rs - 増分バックアップE2Eテスト
//
// Phase 1: クリティカル機能（Priority: High）
// 実装期限: 2025-11-15
//
// このファイルは増分バックアップ機能の完全な動作を検証します。
//
// テストシナリオ:
// 1. 増分バックアップ基本動作 - Day 1フル, Day 2増分, Day 3増分
// 2. 増分バックアップからの最新状態復元
// 3. ポイント・イン・タイム復元 - 特定世代への復元

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
fn create_test_source(temp: &TempDir, scenario: &str) -> std::path::PathBuf {
    let source = temp.path().join("source");
    fs::create_dir_all(&source).unwrap();

    match scenario {
        "day1" => {
            // Day 1: 初期ファイル作成
            fs::write(source.join("file1.txt"), "Day 1: file1 original").unwrap();
            fs::write(source.join("file2.txt"), "Day 1: file2 original").unwrap();
            fs::write(source.join("config.toml"), "[settings]\nday = 1").unwrap();
        }
        "day2" => {
            // Day 2: file1を変更、file3を追加
            fs::write(source.join("file1.txt"), "Day 2: file1 modified").unwrap();
            fs::write(source.join("file2.txt"), "Day 1: file2 original").unwrap(); // 変更なし
            fs::write(source.join("config.toml"), "[settings]\nday = 1").unwrap(); // 変更なし
            fs::write(source.join("file3.txt"), "Day 2: file3 new").unwrap();
        }
        "day3" => {
            // Day 3: file2を変更、file4を追加
            fs::write(source.join("file1.txt"), "Day 2: file1 modified").unwrap(); // 変更なし
            fs::write(source.join("file2.txt"), "Day 3: file2 modified").unwrap();
            fs::write(source.join("config.toml"), "[settings]\nday = 3").unwrap();
            fs::write(source.join("file3.txt"), "Day 2: file3 new").unwrap(); // 変更なし
            fs::write(source.join("file4.txt"), "Day 3: file4 new").unwrap();
        }
        _ => panic!("Unknown scenario: {}", scenario),
    }

    source
}

/// ファイルの内容が一致することを確認
fn assert_file_content(path: &Path, expected: &str) {
    assert!(path.exists(), "File not found: {:?}", path);
    let content = fs::read_to_string(path).unwrap();
    assert_eq!(content, expected, "File content mismatch: {:?}", path);
}

// =============================================================================
// E2E Scenario 1: 増分バックアップ基本動作
// =============================================================================

#[test]
fn test_e2e_incremental_backup_basic() -> Result<()> {
    let temp = TempDir::new()?;
    let backup = temp.path().join("backup");
    fs::create_dir_all(&backup)?;

    // Day 1: フルバックアップ（初回）
    let source = create_test_source(&temp, "day1");

    let mut config = Config::default();
    config.backup.destination = backup.clone();
    config.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner1 = BackupRunner::new(config, false)
        .with_progress(false)
        .with_incremental(true) // 増分バックアップ有効化
        .with_compression(CompressionType::None, 0);

    let result1 = runner1.run(None, None)?;

    // 検証: フルバックアップが実行されたこと
    assert_eq!(result1.total_files, 3, "Day 1: 3 files should be backed up");
    assert_eq!(result1.failed, 0, "Day 1: No failures");

    // タイムスタンプ衝突回避のため1秒待機
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Day 2: 増分バックアップ（file1変更, file3新規追加）
    fs::remove_dir_all(&source)?;
    let source = create_test_source(&temp, "day2");

    let mut config2 = Config::default();
    config2.backup.destination = backup.clone();
    config2.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner2 = BackupRunner::new(config2, false)
        .with_progress(false)
        .with_incremental(true)
        .with_compression(CompressionType::None, 0);

    let result2 = runner2.run(None, None)?;

    // 検証: 増分バックアップで変更ファイルのみ
    // file1（変更）+ file3（新規）= 2ファイル
    assert_eq!(
        result2.total_files, 2,
        "Day 2: Only 2 changed files should be backed up"
    );
    assert_eq!(result2.failed, 0, "Day 2: No failures");

    // タイムスタンプ衝突回避のため1秒待機
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Day 3: 増分バックアップ（file2変更, config変更, file4新規追加）
    fs::remove_dir_all(&source)?;
    let source = create_test_source(&temp, "day3");

    let mut config3 = Config::default();
    config3.backup.destination = backup.clone();
    config3.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner3 = BackupRunner::new(config3, false)
        .with_progress(false)
        .with_incremental(true)
        .with_compression(CompressionType::None, 0);

    let result3 = runner3.run(None, None)?;

    // 検証: 増分バックアップで変更ファイルのみ
    // file2（変更）+ config.toml（変更）+ file4（新規）= 3ファイル
    assert_eq!(
        result3.total_files, 3,
        "Day 3: Only 3 changed files should be backed up"
    );
    assert_eq!(result3.failed, 0, "Day 3: No failures");

    // 検証: 3つのバックアップが作成されていること
    let backup_count = fs::read_dir(&backup)?.count();
    assert_eq!(backup_count, 3, "3 backups should exist");

    Ok(())
}

// =============================================================================
// E2E Scenario 2: 増分バックアップからの最新状態復元
// =============================================================================

#[test]
fn test_e2e_incremental_restore_latest() -> Result<()> {
    let temp = TempDir::new()?;
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");
    fs::create_dir_all(&backup)?;

    // Day 1: フルバックアップ
    let source = create_test_source(&temp, "day1");

    let mut config = Config::default();
    config.backup.destination = backup.clone();
    config.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner1 = BackupRunner::new(config, false)
        .with_progress(false)
        .with_incremental(true)
        .with_compression(CompressionType::None, 0);

    runner1.run(None, None)?;

    // タイムスタンプ衝突回避のため1秒待機
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Day 2: 増分バックアップ
    fs::remove_dir_all(&source)?;
    let source = create_test_source(&temp, "day2");

    let mut config2 = Config::default();
    config2.backup.destination = backup.clone();
    config2.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner2 = BackupRunner::new(config2, false)
        .with_progress(false)
        .with_incremental(true)
        .with_compression(CompressionType::None, 0);

    runner2.run(None, None)?;

    // タイムスタンプ衝突回避のため1秒待機
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Day 3: 増分バックアップ
    fs::remove_dir_all(&source)?;
    create_test_source(&temp, "day3");

    let mut config3 = Config::default();
    config3.backup.destination = backup.clone();
    config3.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner3 = BackupRunner::new(config3, false)
        .with_progress(false)
        .with_incremental(true)
        .with_compression(CompressionType::None, 0);

    let result3 = runner3.run(None, None)?;

    // 最新のバックアップから復元
    fs::create_dir_all(&restore)?;
    let latest_backup = backup.join(&result3.backup_name);

    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    restore_engine.restore(&latest_backup, &restore, None)?;

    // 検証: Day 3の最新状態が復元されていること
    let restored_root = restore.join("test");
    assert_file_content(
        &restored_root.join("file1.txt"),
        "Day 2: file1 modified", // Day 2で変更、Day 3で変更なし
    );
    assert_file_content(
        &restored_root.join("file2.txt"),
        "Day 3: file2 modified", // Day 3で変更
    );
    assert_file_content(
        &restored_root.join("config.toml"),
        "[settings]\nday = 3", // Day 3で変更
    );
    assert_file_content(&restored_root.join("file3.txt"), "Day 2: file3 new"); // Day 2で追加
    assert_file_content(&restored_root.join("file4.txt"), "Day 3: file4 new"); // Day 3で追加

    Ok(())
}

// =============================================================================
// E2E Scenario 3: ポイント・イン・タイム復元 - Day 2の状態に復元
// =============================================================================

#[test]
fn test_e2e_incremental_point_in_time_restore() -> Result<()> {
    let temp = TempDir::new()?;
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");
    fs::create_dir_all(&backup)?;

    // Day 1: フルバックアップ
    let source = create_test_source(&temp, "day1");

    let mut config = Config::default();
    config.backup.destination = backup.clone();
    config.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner1 = BackupRunner::new(config, false)
        .with_progress(false)
        .with_incremental(true)
        .with_compression(CompressionType::None, 0);

    runner1.run(None, None)?;

    // タイムスタンプ衝突回避のため1秒待機
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Day 2: 増分バックアップ
    fs::remove_dir_all(&source)?;
    let source = create_test_source(&temp, "day2");

    let mut config2 = Config::default();
    config2.backup.destination = backup.clone();
    config2.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner2 = BackupRunner::new(config2, false)
        .with_progress(false)
        .with_incremental(true)
        .with_compression(CompressionType::None, 0);

    let result2 = runner2.run(None, None)?;

    // タイムスタンプ衝突回避のため1秒待機
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Day 3: 増分バックアップ（これは使わない）
    fs::remove_dir_all(&source)?;
    create_test_source(&temp, "day3");

    let mut config3 = Config::default();
    config3.backup.destination = backup.clone();
    config3.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner3 = BackupRunner::new(config3, false)
        .with_progress(false)
        .with_incremental(true)
        .with_compression(CompressionType::None, 0);

    runner3.run(None, None)?;

    // Day 2の時点に復元（ポイント・イン・タイム復元）
    fs::create_dir_all(&restore)?;
    let day2_backup = backup.join(&result2.backup_name);

    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    restore_engine.restore(&day2_backup, &restore, None)?;

    // 検証: Day 2の状態が復元されていること
    let restored_root = restore.join("test");
    assert_file_content(
        &restored_root.join("file1.txt"),
        "Day 2: file1 modified", // Day 2で変更
    );
    assert_file_content(
        &restored_root.join("file2.txt"),
        "Day 1: file2 original", // Day 2では変更なし
    );
    assert_file_content(
        &restored_root.join("config.toml"),
        "[settings]\nday = 1", // Day 2では変更なし
    );
    assert_file_content(&restored_root.join("file3.txt"), "Day 2: file3 new"); // Day 2で追加

    // Day 3で追加されたfile4は存在しないはず
    assert!(
        !restored_root.join("file4.txt").exists(),
        "file4 should not exist in Day 2 state"
    );

    Ok(())
}
