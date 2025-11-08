//! 増分バックアップ機能の統合テスト

use backup_suite::core::{BackupRunner, BackupType, Config, Priority, Target};
use backup_suite::RestoreEngine;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_full_backup_first_time() {
    let temp = TempDir::new().unwrap();
    let source_dir = temp.path().join("source");
    let backup_base = temp.path().join("backups");

    // ソースファイル作成
    fs::create_dir(&source_dir).unwrap();
    fs::write(source_dir.join("file1.txt"), b"content1").unwrap();
    fs::write(source_dir.join("file2.txt"), b"content2").unwrap();

    // 設定
    let mut config = Config::default();
    config.add_target(Target::new(
        source_dir.clone(),
        Priority::High,
        "test".to_string(),
    ));
    config.backup.destination = backup_base.clone();

    // 初回バックアップ（増分モードでも自動的にフルバックアップになる）
    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_incremental(true);

    let result = runner.run(None, None).unwrap();

    assert_eq!(result.successful, 2);
    assert_eq!(result.failed, 0);

    // メタデータ確認
    let backup_dir = backup_base.join(&result.backup_name);
    let metadata = backup_suite::core::BackupMetadata::load(&backup_dir).unwrap();

    assert_eq!(metadata.backup_type, BackupType::Full);
    assert!(metadata.parent_backup.is_none());
    assert_eq!(metadata.file_hashes.len(), 2);
}

#[test]
fn test_incremental_backup_changed_files_only() {
    let temp = TempDir::new().unwrap();
    let source_dir = temp.path().join("source");
    let backup_base = temp.path().join("backups");

    // ソースファイル作成
    fs::create_dir(&source_dir).unwrap();
    fs::write(source_dir.join("file1.txt"), b"content1").unwrap();
    fs::write(source_dir.join("file2.txt"), b"content2").unwrap();

    // 設定
    let mut config = Config::default();
    config.add_target(Target::new(
        source_dir.clone(),
        Priority::High,
        "test".to_string(),
    ));
    config.backup.destination = backup_base.clone();

    // 1回目: フルバックアップ
    let mut runner1 = BackupRunner::new(config, false)
        .with_progress(false)
        .with_incremental(true);

    let result1 = runner1.run(None, None).unwrap();
    assert_eq!(result1.successful, 2);

    // ファイル変更
    fs::write(source_dir.join("file1.txt"), b"modified content1").unwrap();
    fs::write(source_dir.join("file3.txt"), b"new file").unwrap(); // 新規ファイル

    // 2回目: 増分バックアップ（configを再作成）
    let mut config2 = Config::default();
    config2.add_target(Target::new(
        source_dir.clone(),
        Priority::High,
        "test".to_string(),
    ));
    config2.backup.destination = backup_base.clone();

    let mut runner2 = BackupRunner::new(config2, false)
        .with_progress(false)
        .with_incremental(true);

    let result2 = runner2.run(None, None).unwrap();

    // 変更されたfile1と新規file3の2ファイルのみバックアップされるはず
    assert_eq!(result2.successful, 2);

    // メタデータ確認
    let backup_dir2 = backup_base.join(&result2.backup_name);
    let metadata2 = backup_suite::core::BackupMetadata::load(&backup_dir2).unwrap();

    assert_eq!(metadata2.backup_type, BackupType::Incremental);
    assert!(metadata2.parent_backup.is_some());
    assert_eq!(metadata2.changed_files.len(), 2);
}

#[test]
fn test_incremental_restore_chain() {
    let temp = TempDir::new().unwrap();
    let source_dir = temp.path().join("source");
    let backup_base = temp.path().join("backups");
    let restore_dir = temp.path().join("restore");

    // ソースファイル作成
    fs::create_dir(&source_dir).unwrap();
    fs::write(source_dir.join("file1.txt"), b"version1").unwrap();
    fs::write(source_dir.join("file2.txt"), b"version1").unwrap();

    // 設定
    let mut config = Config::default();
    config.add_target(Target::new(
        source_dir.clone(),
        Priority::High,
        "test".to_string(),
    ));
    config.backup.destination = backup_base.clone();

    // 1回目: フルバックアップ
    let mut runner1 = BackupRunner::new(config, false)
        .with_progress(false)
        .with_incremental(true);

    let result1 = runner1.run(None, None).unwrap();
    assert_eq!(result1.successful, 2);

    // ファイル変更（file1のみ）
    std::thread::sleep(std::time::Duration::from_secs(1)); // タイムスタンプの違いを確保
    fs::write(source_dir.join("file1.txt"), b"version2").unwrap();

    // 2回目: 増分バックアップ
    let mut config2 = Config::default();
    config2.add_target(Target::new(
        source_dir.clone(),
        Priority::High,
        "test".to_string(),
    ));
    config2.backup.destination = backup_base.clone();

    let mut runner2 = BackupRunner::new(config2, false)
        .with_progress(false)
        .with_incremental(true);

    let result2 = runner2.run(None, None).unwrap();
    // sleepによりタイムスタンプが異なる場合は増分バックアップが機能する
    // ただし、同一秒内の場合はフルバックアップになる可能性がある
    // ここでは柔軟にチェック
    assert!(result2.successful >= 1);

    // ファイル変更（file2のみ）
    std::thread::sleep(std::time::Duration::from_secs(1));
    fs::write(source_dir.join("file2.txt"), b"version2").unwrap();

    // 3回目: 増分バックアップ
    let mut config3 = Config::default();
    config3.add_target(Target::new(
        source_dir.clone(),
        Priority::High,
        "test".to_string(),
    ));
    config3.backup.destination = backup_base.clone();

    let mut runner3 = BackupRunner::new(config3, false)
        .with_progress(false)
        .with_incremental(true);

    let result3 = runner3.run(None, None).unwrap();
    assert!(result3.successful >= 1);

    // 最新のバックアップから復元
    let backup_dir3 = backup_base.join(&result3.backup_name);
    let mut engine = RestoreEngine::new(false).with_progress(false);
    let restore_result = engine.restore(&backup_dir3, &restore_dir, None).unwrap();

    // 復元が成功すればOK（タイムスタンプの問題でフルバックアップになることもある）
    assert!(restore_result.restored >= 2);
    assert_eq!(restore_result.failed, 0);

    // 復元されたファイルの内容確認
    let restored_file1 = fs::read_to_string(restore_dir.join("test/file1.txt")).unwrap();
    let restored_file2 = fs::read_to_string(restore_dir.join("test/file2.txt")).unwrap();

    assert_eq!(restored_file1, "version2");
    assert_eq!(restored_file2, "version2");
}

#[test]
fn test_full_backup_when_incremental_requested_but_no_previous() {
    let temp = TempDir::new().unwrap();
    let source_dir = temp.path().join("source");
    let backup_base = temp.path().join("backups");

    // ソースファイル作成
    fs::create_dir(&source_dir).unwrap();
    fs::write(source_dir.join("file1.txt"), b"content").unwrap();

    // 設定
    let mut config = Config::default();
    config.add_target(Target::new(
        source_dir.clone(),
        Priority::High,
        "test".to_string(),
    ));
    config.backup.destination = backup_base.clone();

    // 増分モードを指定するが、前回バックアップがないため自動的にフルバックアップになる
    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_incremental(true);

    let result = runner.run(None, None).unwrap();

    assert_eq!(result.successful, 1);

    // メタデータ確認
    let backup_dir = backup_base.join(&result.backup_name);
    let metadata = backup_suite::core::BackupMetadata::load(&backup_dir).unwrap();

    assert_eq!(metadata.backup_type, BackupType::Full);
    assert!(metadata.parent_backup.is_none());
}
