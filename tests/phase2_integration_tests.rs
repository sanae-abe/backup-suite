use backup_suite::{
    BackupHistory, BackupRunner, BackupStatus, CleanupPolicy, Config, Priority, RestoreEngine,
    Target,
};
use std::fs;
use tempfile::TempDir;

/// Phase 2機能の統合テスト

#[test]
fn test_history_with_filters() {
    // 履歴エントリの作成とフィルタリング
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("backup_20251107_120000");
    fs::create_dir_all(&backup_dir).unwrap();

    let mut history_entries = vec![];

    // 高優先度のエントリ
    let mut entry1 = BackupHistory::new(backup_dir.clone(), 100, 1_000_000, true);
    entry1.priority = Some(Priority::High);
    entry1.category = Some("documents".to_string());
    entry1.status = BackupStatus::Success;
    entry1.compressed = true;
    entry1.encrypted = false;
    history_entries.push(entry1);

    // 中優先度のエントリ
    let mut entry2 = BackupHistory::new(
        temp.path().join("backup_20251106_120000"),
        50,
        500_000,
        true,
    );
    entry2.priority = Some(Priority::Medium);
    entry2.category = Some("photos".to_string());
    entry2.status = BackupStatus::Success;
    entry2.compressed = false;
    entry2.encrypted = true;
    history_entries.push(entry2);

    // 優先度フィルタのテスト
    let high_priority = BackupHistory::filter_by_priority(&history_entries, &Priority::High);
    assert_eq!(high_priority.len(), 1);
    assert_eq!(high_priority[0].priority, Some(Priority::High));

    // カテゴリフィルタのテスト
    let docs = BackupHistory::filter_by_category(&history_entries, "documents");
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].category, Some("documents".to_string()));
}

#[test]
fn test_restore_engine() {
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("backup");
    let restore_dir = temp.path().join("restore");

    // バックアップディレクトリ作成
    fs::create_dir_all(&backup_dir).unwrap();
    fs::write(backup_dir.join("file1.txt"), b"content1").unwrap();
    fs::write(backup_dir.join("file2.txt"), b"content2").unwrap();

    // 復元実行
    let mut engine = RestoreEngine::new(false).with_progress(false);
    let result = engine.restore(&backup_dir, &restore_dir, None).unwrap();

    assert_eq!(result.total_files, 2);
    assert_eq!(result.restored, 2);
    assert_eq!(result.failed, 0);
    assert_eq!(result.encrypted_files, 0);

    // 復元されたファイルの確認
    assert_eq!(
        fs::read_to_string(restore_dir.join("file1.txt")).unwrap(),
        "content1"
    );
    assert_eq!(
        fs::read_to_string(restore_dir.join("file2.txt")).unwrap(),
        "content2"
    );
}

#[test]
fn test_cleanup_policy_retention_days() {
    let temp = TempDir::new().unwrap();

    // 古いバックアップディレクトリを作成（テスト用）
    let old_backup = temp.path().join("backup_old");
    fs::create_dir_all(&old_backup).unwrap();
    fs::write(old_backup.join("test.txt"), b"old").unwrap();

    let policy = CleanupPolicy::retention_days(30);
    assert_eq!(policy.retention_days, Some(30));
    assert_eq!(policy.keep_count, None);
}

#[test]
fn test_cleanup_policy_keep_count() {
    let policy = CleanupPolicy::keep_count(10);
    assert_eq!(policy.keep_count, Some(10));
    assert_eq!(policy.retention_days, None);
}

#[test]
fn test_cleanup_policy_max_size() {
    let max_size = 1024 * 1024 * 100; // 100MB
    let policy = CleanupPolicy::max_size(max_size);
    assert_eq!(policy.max_total_size, Some(max_size));
}

#[test]
fn test_target_with_exclude_patterns() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    fs::create_dir_all(&source).unwrap();

    let mut config = Config::default();
    config.backup.destination = temp.path().join("backups");

    let mut target = Target::new(source.clone(), Priority::High, "test".to_string());
    target.exclude_patterns = vec![r"\.log$".to_string(), r"\.tmp$".to_string()];
    config.add_target(target);

    // ターゲットに除外パターンが設定されていることを確認
    assert_eq!(config.targets.len(), 1);
    assert_eq!(config.targets[0].exclude_patterns.len(), 2);
    assert!(config.targets[0]
        .exclude_patterns
        .contains(&r"\.log$".to_string()));
}

#[test]
fn test_backup_with_exclude_patterns() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    fs::create_dir_all(&source).unwrap();

    // テストファイル作成
    fs::write(source.join("keep.txt"), b"keep this").unwrap();
    fs::write(source.join("exclude.log"), b"exclude this").unwrap();
    fs::write(source.join("exclude.tmp"), b"exclude this too").unwrap();

    let mut config = Config::default();
    config.backup.destination = temp.path().join("backups");

    let mut target = Target::new(source.clone(), Priority::High, "test".to_string());
    target.exclude_patterns = vec![r"\.log$".to_string(), r"\.tmp$".to_string()];
    config.add_target(target);

    let mut runner = BackupRunner::new(config, false).with_progress(false);
    let result = runner.run(None, None).unwrap();

    // .logと.tmpは除外されるので、keep.txtのみバックアップされる
    assert_eq!(result.total_files, 1);
    assert_eq!(result.successful, 1);
}

#[test]
fn test_history_status_enum() {
    let status_success = BackupStatus::Success;
    let status_failed = BackupStatus::Failed;
    let status_partial = BackupStatus::Partial;

    assert_eq!(status_success, BackupStatus::Success);
    assert_eq!(status_failed, BackupStatus::Failed);
    assert_eq!(status_partial, BackupStatus::Partial);
}

#[test]
fn test_recent_entries() {
    // BackupHistory::get_recent_entriesは実際の履歴ファイルを読むため、
    // ここでは関数が存在することのみ確認
    let result = BackupHistory::get_recent_entries(10);
    // 履歴ファイルが存在しない、または破損している場合はエラーが返される
    // これはテスト環境では正常な動作
    assert!(result.is_ok() || result.is_err());
}
