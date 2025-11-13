// integrity_verification_e2e_tests.rs - 整合性検証E2Eテスト
//
// Phase 1: クリティカル機能（Priority: High）
// 実装期限: 2025-11-15
//
// このファイルはSHA-256ハッシュベースの整合性検証機能を検証します。
//
// テストシナリオ:
// 1. 整合性検証成功 - バックアップ後の.integrityファイル生成と検証
// 2. 改ざん検出 - バックアップファイル改ざん時の検出

use anyhow::Result;
use backup_suite::compression::CompressionType;
use backup_suite::core::integrity::BackupMetadata;
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

    fs::write(source.join("file1.txt"), "Content of file 1").unwrap();
    fs::write(source.join("file2.txt"), "Content of file 2").unwrap();
    fs::write(source.join("file3.txt"), "Content of file 3").unwrap();

    source
}

/// ファイルの内容が一致することを確認
fn assert_file_content(path: &Path, expected: &str) {
    assert!(path.exists(), "File not found: {:?}", path);
    let content = fs::read_to_string(path).unwrap();
    assert_eq!(content, expected, "File content mismatch: {:?}", path);
}

// =============================================================================
// E2E Scenario 1: 整合性検証成功
// =============================================================================

#[test]
fn test_e2e_integrity_verification_success() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");
    fs::create_dir_all(&backup)?;

    // バックアップ実行
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
    assert_eq!(result.failed, 0, "Backup should succeed");

    // 検証1: .integrityファイルが作成されていることを確認
    let backup_dir = backup.join(&result.backup_name);
    let integrity_file = backup_dir.join(".integrity");
    assert!(
        integrity_file.exists(),
        ".integrity file should be created: {:?}",
        integrity_file
    );

    // 検証2: メタデータを読み込み、正しい情報が含まれることを確認
    let metadata = BackupMetadata::load(&backup_dir)?;
    assert_eq!(metadata.version, "1.0", "Metadata version should be 1.0");
    assert!(
        !metadata.file_hashes.is_empty(),
        "File hashes should not be empty"
    );
    assert!(
        metadata.file_hashes.len() >= 3,
        "At least 3 files should have hashes"
    );

    // 検証3: 復元前の整合性検証（正常ケース）
    fs::create_dir_all(&restore)?;
    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    let restore_result = restore_engine.restore(&backup_dir, &restore, None)?;

    assert_eq!(
        restore_result.failed, 0,
        "Restore should succeed with valid integrity"
    );
    assert!(
        restore_result.errors.is_empty(),
        "No errors should occur during restore"
    );

    // 検証4: 復元されたファイルの内容確認
    let restored_root = restore.join("test");
    assert_file_content(&restored_root.join("file1.txt"), "Content of file 1");
    assert_file_content(&restored_root.join("file2.txt"), "Content of file 2");
    assert_file_content(&restored_root.join("file3.txt"), "Content of file 3");

    Ok(())
}

// =============================================================================
// E2E Scenario 2: 改ざん検出（バックアップファイル改ざん）
// =============================================================================

#[test]
fn test_e2e_integrity_verification_tampered() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");
    fs::create_dir_all(&backup)?;

    // バックアップ実行
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
    assert_eq!(result.failed, 0, "Backup should succeed");

    let backup_dir = backup.join(&result.backup_name);

    // バックアップファイルを意図的に改ざん
    let backup_file_dir = backup_dir.join("test");
    let tampered_file = backup_file_dir.join("file1.txt");

    if tampered_file.exists() {
        fs::write(&tampered_file, "TAMPERED CONTENT")?;

        // 復元を試みる
        fs::create_dir_all(&restore)?;
        let mut restore_engine = RestoreEngine::new(false).with_progress(false);
        let _restore_result = restore_engine.restore(&backup_dir, &restore, None)?;

        // 検証: 改ざんされたファイルは復元されるが、整合性チェックで検出される可能性がある
        // （現在の実装では復元後にverify_fileを手動で呼ぶ必要がある）

        // メタデータを読み込んで手動で検証
        let metadata = BackupMetadata::load(&backup_dir)?;

        // 復元されたファイルを検証
        let restored_root = restore.join("test");
        let restored_file = restored_root.join("file1.txt");

        let is_valid =
            metadata.verify_file(&std::path::PathBuf::from("test/file1.txt"), &restored_file)?;

        // 検証: 改ざんされたファイルは整合性検証で失敗するはず
        assert!(
            !is_valid,
            "Tampered file should fail integrity verification"
        );

        // ファイル内容が改ざんされていることを確認
        let content = fs::read_to_string(&restored_file)?;
        assert_eq!(
            content, "TAMPERED CONTENT",
            "File should contain tampered content"
        );
    }

    Ok(())
}

// =============================================================================
// E2E Scenario 3: メタデータファイル自体の改ざん検出
// =============================================================================

#[test]
fn test_e2e_integrity_metadata_tampered() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup = temp.path().join("backup");
    fs::create_dir_all(&backup)?;

    // バックアップ実行
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
    assert_eq!(result.failed, 0, "Backup should succeed");

    let backup_dir = backup.join(&result.backup_name);
    let integrity_file = backup_dir.join(".integrity");

    // .integrityファイルを意図的に破損
    if integrity_file.exists() {
        fs::write(&integrity_file, "INVALID JSON CONTENT")?;

        // メタデータ読み込みを試みる（失敗するはず）
        let load_result = BackupMetadata::load(&backup_dir);

        // 検証: 破損したメタデータはエラーになるはず
        assert!(
            load_result.is_err(),
            "Loading tampered .integrity file should fail"
        );

        if let Err(e) = load_result {
            let error_msg = e.to_string();
            // JSONパースエラーが含まれることを確認
            assert!(
                error_msg.contains("JSON")
                    || error_msg.contains("parse")
                    || error_msg.contains("deserialize"),
                "Error should mention JSON parsing failure: {}",
                error_msg
            );
        }
    }

    Ok(())
}

// =============================================================================
// E2E Scenario 4: 暗号化バックアップの整合性検証
// =============================================================================

#[test]
fn test_e2e_encrypted_integrity_verification() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");
    fs::create_dir_all(&backup)?;

    let password = "integrity_test_password";

    // 暗号化バックアップ実行
    let mut config = Config::default();
    config.backup.destination = backup.clone();
    config.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_encryption(password.to_string())
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;
    assert_eq!(result.failed, 0, "Encrypted backup should succeed");

    // 検証1: .integrityファイルが暗号化バックアップでも作成される
    let backup_dir = backup.join(&result.backup_name);
    let integrity_file = backup_dir.join(".integrity");
    assert!(
        integrity_file.exists(),
        ".integrity file should exist for encrypted backups"
    );

    // 検証2: 暗号化バックアップの復元と整合性確認
    fs::create_dir_all(&restore)?;
    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    let restore_result = restore_engine.restore(&backup_dir, &restore, Some(password))?;

    assert_eq!(restore_result.failed, 0, "Encrypted restore should succeed");

    // 検証3: 復元されたファイルの内容確認
    let restored_root = restore.join("test");
    assert_file_content(&restored_root.join("file1.txt"), "Content of file 1");
    assert_file_content(&restored_root.join("file2.txt"), "Content of file 2");
    assert_file_content(&restored_root.join("file3.txt"), "Content of file 3");

    Ok(())
}
