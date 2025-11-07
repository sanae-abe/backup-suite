//! 整合性検証統合テスト
//!
//! バックアップ→復元のフル流れでSHA-256整合性検証を確認します。

use backup_suite::core::{
    BackupMetadata, BackupRunner, Config, IntegrityChecker, Priority, RestoreEngine, Target,
};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_backup_and_restore_with_integrity_verification() {
    // 一時ディレクトリ作成
    let temp = TempDir::new().unwrap();
    let source_dir = temp.path().join("source");
    let backup_base = temp.path().join("backups");
    let restore_dir = temp.path().join("restore");

    // ソースファイル作成
    fs::create_dir_all(&source_dir).unwrap();
    let file1 = source_dir.join("test1.txt");
    let file2 = source_dir.join("subdir/test2.txt");
    fs::create_dir_all(&source_dir.join("subdir")).unwrap();

    let mut f1 = fs::File::create(&file1).unwrap();
    f1.write_all(b"test content 1").unwrap();
    drop(f1);

    let mut f2 = fs::File::create(&file2).unwrap();
    f2.write_all(b"test content 2 with more data").unwrap();
    drop(f2);

    // バックアップ設定
    let mut config = Config::default();
    config.backup.destination = backup_base.clone();
    config.add_target(Target::new(
        source_dir.clone(),
        Priority::High,
        "test-category".to_string(),
    ));

    // バックアップ実行（整合性検証有効）
    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_verification(true);
    let backup_result = runner.run(None, None).unwrap();

    assert_eq!(backup_result.total_files, 2);
    assert_eq!(backup_result.successful, 2);
    assert_eq!(backup_result.failed, 0);

    // バックアップディレクトリのパスを取得
    let backup_dir = backup_base.join(&backup_result.backup_name);
    assert!(backup_dir.exists());

    // .integrity ファイルが存在することを確認
    let integrity_file = backup_dir.join(".integrity");
    assert!(integrity_file.exists());

    // メタデータを読み込み
    let metadata = BackupMetadata::load(&backup_dir).unwrap();
    assert_eq!(metadata.file_hashes.len(), 2);
    assert_eq!(metadata.version, "1.0");

    // 復元実行（整合性検証有効）
    let mut restore_engine = RestoreEngine::new(false)
        .with_progress(false)
        .with_verification(true);
    let restore_result = restore_engine
        .restore(&backup_dir, &restore_dir, None)
        .unwrap();

    assert_eq!(restore_result.total_files, 2); // 2ファイル（.integrityは除外）
    assert_eq!(restore_result.restored, 2);
    assert_eq!(restore_result.failed, 0);
    assert_eq!(restore_result.verified_files, 2);
    assert_eq!(restore_result.verification_failures, 0);

    // 復元されたファイルの内容を確認
    let restored1 = fs::read_to_string(restore_dir.join("test-category/test1.txt")).unwrap();
    assert_eq!(restored1, "test content 1");

    let restored2 =
        fs::read_to_string(restore_dir.join("test-category/subdir/test2.txt")).unwrap();
    assert_eq!(restored2, "test content 2 with more data");
}

#[test]
fn test_restore_detects_tampered_file() {
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("backup");
    let restore_dir = temp.path().join("restore");
    fs::create_dir_all(&backup_dir).unwrap();

    // テストファイル作成
    let test_file = backup_dir.join("test.txt");
    fs::write(&test_file, b"original content").unwrap();

    // 整合性メタデータ作成
    let mut checker = IntegrityChecker::new();
    let hash = checker.compute_hash(&test_file).unwrap();
    checker.add_file_hash("test.txt".into(), hash);
    checker.save_metadata(&backup_dir).unwrap();

    // バックアップファイルを改ざん
    fs::write(&test_file, b"tampered content").unwrap();

    // 復元実行（整合性検証有効）
    let mut restore_engine = RestoreEngine::new(false)
        .with_progress(false)
        .with_verification(true);
    let restore_result = restore_engine
        .restore(&backup_dir, &restore_dir, None)
        .unwrap();

    // 復元は成功するが、検証失敗を検出
    assert_eq!(restore_result.total_files, 1); // test.txt（.integrityは除外）
    assert_eq!(restore_result.restored, 1);
    assert_eq!(restore_result.failed, 0);
    assert_eq!(restore_result.verified_files, 0);
    assert_eq!(restore_result.verification_failures, 1);

    // エラーメッセージに改ざん検出が含まれることを確認
    assert!(!restore_result.errors.is_empty());
    let error_msg = restore_result.errors.join(" ");
    assert!(error_msg.contains("整合性検証失敗"));
}

#[test]
fn test_backup_without_integrity_verification() {
    let temp = TempDir::new().unwrap();
    let source_dir = temp.path().join("source");
    let backup_base = temp.path().join("backups");
    fs::create_dir_all(&source_dir).unwrap();

    let test_file = source_dir.join("test.txt");
    fs::write(&test_file, b"test content").unwrap();

    // バックアップ設定
    let mut config = Config::default();
    config.backup.destination = backup_base.clone();
    config.add_target(Target::new(
        source_dir.clone(),
        Priority::High,
        "test".to_string(),
    ));

    // バックアップ実行（整合性検証無効）
    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_verification(false);
    let backup_result = runner.run(None, None).unwrap();

    assert_eq!(backup_result.successful, 1);

    // .integrity ファイルが存在しないことを確認
    let backup_dir = backup_base.join(&backup_result.backup_name);
    let integrity_file = backup_dir.join(".integrity");
    assert!(!integrity_file.exists());
}

#[test]
fn test_restore_without_integrity_metadata() {
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("backup");
    let restore_dir = temp.path().join("restore");
    fs::create_dir_all(&backup_dir).unwrap();

    // テストファイル作成（メタデータなし）
    fs::write(backup_dir.join("test.txt"), b"test content").unwrap();

    // 復元実行（整合性検証有効だがメタデータなし）
    let mut restore_engine = RestoreEngine::new(false)
        .with_progress(false)
        .with_verification(true);
    let restore_result = restore_engine
        .restore(&backup_dir, &restore_dir, None)
        .unwrap();

    // 復元は成功するが、検証は実行されない
    assert_eq!(restore_result.total_files, 1);
    assert_eq!(restore_result.restored, 1);
    assert_eq!(restore_result.failed, 0);
    assert_eq!(restore_result.verified_files, 0); // メタデータなし
    assert_eq!(restore_result.verification_failures, 0);
}

#[test]
fn test_integrity_checker_multiple_files() {
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("backup");
    fs::create_dir_all(&backup_dir).unwrap();

    // 複数のテストファイル作成
    let files = vec![
        ("file1.txt", b"content 1"),
        ("file2.txt", b"content 2"),
        ("subdir/file3.txt", b"content 3"),
    ];

    fs::create_dir_all(backup_dir.join("subdir")).unwrap();

    let mut checker = IntegrityChecker::new();

    for (path, content) in &files {
        let file_path = backup_dir.join(path);
        fs::write(&file_path, *content).unwrap();
        let hash = checker.compute_hash(&file_path).unwrap();
        checker.add_file_hash(path.into(), hash);
    }

    assert_eq!(checker.file_count(), 3);

    // メタデータ保存
    checker.save_metadata(&backup_dir).unwrap();

    // メタデータ読み込みと検証
    let metadata = BackupMetadata::load(&backup_dir).unwrap();
    assert_eq!(metadata.file_hashes.len(), 3);

    for (path, _) in &files {
        let file_path = backup_dir.join(path);
        let path_buf = std::path::PathBuf::from(*path);
        let is_valid = metadata.verify_file(&path_buf, &file_path).unwrap();
        assert!(is_valid);
    }
}
