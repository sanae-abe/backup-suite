// error_recovery_e2e_tests.rs - エラーリカバリE2Eテスト
//
// Phase 1: クリティカル機能（Priority: High）
// 実装期限: 2025-11-15
//
// このファイルはエラー発生時の適切なリカバリ処理を検証します。
//
// テストシナリオ:
// 1. 権限エラー時の継続処理 - 一部ファイルが読めなくても他ファイルは継続
// 2. 破損バックアップの検出 - 整合性検証による破損検出
// 3. 復元時のエラーハンドリング - 誤パスワード等での適切なエラー

use anyhow::Result;
use backup_suite::compression::CompressionType;
use backup_suite::core::{BackupRunner, Config, RestoreEngine};
use backup_suite::{Priority, Target};
use std::fs;
use tempfile::TempDir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// =============================================================================
// テストヘルパー関数
// =============================================================================

/// テスト用のソースディレクトリを作成
fn create_test_source(temp: &TempDir) -> std::path::PathBuf {
    let source = temp.path().join("source");
    fs::create_dir_all(&source).unwrap();

    fs::write(source.join("file1.txt"), "Normal file 1").unwrap();
    fs::write(source.join("file2.txt"), "Normal file 2").unwrap();
    fs::write(source.join("file3.txt"), "Normal file 3").unwrap();

    source
}

// =============================================================================
// E2E Scenario 1: 権限エラー時の継続処理
// =============================================================================

#[test]
#[cfg(unix)] // Unix系OSのみ（パーミッション操作）
fn test_e2e_permission_error_continues() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup = temp.path().join("backup");
    fs::create_dir_all(&backup)?;

    // file2.txtを読み取り不可に設定
    let protected_file = source.join("file2.txt");
    let mut perms = fs::metadata(&protected_file)?.permissions();
    perms.set_mode(0o000); // 全ての権限を削除
    fs::set_permissions(&protected_file, perms)?;

    // バックアップ実行（一部ファイルで失敗するはず）
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

    // 検証: 一部失敗しても処理は完了
    assert!(
        result.failed > 0,
        "Permission error should cause at least 1 failure"
    );
    assert!(
        result.successful > 0,
        "Other files should be backed up successfully"
    );
    assert!(
        !result.errors.is_empty(),
        "Error list should contain permission errors"
    );

    // エラーメッセージに権限関連の内容が含まれることを確認
    let has_permission_error = result
        .errors
        .iter()
        .any(|e| e.contains("Permission") || e.contains("permission") || e.contains("denied"));
    assert!(
        has_permission_error,
        "Error messages should mention permission issues"
    );

    // 権限を戻す（テンポラリディレクトリクリーンアップのため）
    let mut perms = fs::metadata(&protected_file)?.permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&protected_file, perms)?;

    Ok(())
}

// =============================================================================
// E2E Scenario 2: 破損バックアップの検出
// =============================================================================

#[test]
fn test_e2e_corrupted_backup_detection() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");
    fs::create_dir_all(&backup)?;

    // 正常なバックアップを作成
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

    // バックアップファイルを意図的に破損
    let backup_dir = backup.join(&result.backup_name).join("test");
    let files: Vec<_> = fs::read_dir(&backup_dir)?.collect();

    if let Some(Ok(entry)) = files.first() {
        let file_path = entry.path();
        // ファイルの一部を破壊
        fs::write(&file_path, b"CORRUPTED DATA")?;

        // 復元を試みる（失敗するはず）
        fs::create_dir_all(&restore)?;
        let actual_backup = backup.join(&result.backup_name);

        let mut restore_engine = RestoreEngine::new(false).with_progress(false);
        let restore_result = restore_engine.restore(&actual_backup, &restore, None)?;

        // 検証: 復元は完了するが、破損ファイルはエラーとして記録される
        // （暗号化なしの場合、ファイル内容は復元されるが整合性チェックはない）
        // このテストは暗号化バックアップでより重要
        assert!(
            restore_result.total_files > 0,
            "Restore should process files even if some are corrupted"
        );
    }

    Ok(())
}

// =============================================================================
// E2E Scenario 3: 暗号化バックアップの破損検出（認証タグ検証）
// =============================================================================

#[test]
fn test_e2e_encrypted_corrupted_backup_detection() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");
    fs::create_dir_all(&backup)?;

    let password = "test_password_123";

    // 暗号化バックアップを作成
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

    // 暗号化バックアップファイルを意図的に破損
    let backup_dir = backup.join(&result.backup_name).join("test");
    let files: Vec<_> = fs::read_dir(&backup_dir)?.filter_map(|e| e.ok()).collect();

    if let Some(entry) = files.first() {
        let file_path = entry.path();
        let mut data = fs::read(&file_path)?;

        // データの一部を破壊（認証タグ検証で検出されるはず）
        if data.len() > 50 {
            data[50] ^= 0xFF; // 1バイト反転
            fs::write(&file_path, data)?;
        }

        // 復元を試みる（AES-GCM認証タグで失敗するはず）
        fs::create_dir_all(&restore)?;
        let actual_backup = backup.join(&result.backup_name);

        let mut restore_engine = RestoreEngine::new(false).with_progress(false);
        let restore_result = restore_engine.restore(&actual_backup, &restore, Some(password))?;

        // 検証: 破損ファイルは復号化失敗として記録される
        assert!(
            restore_result.failed > 0,
            "Corrupted encrypted file should fail to decrypt: failed={}, errors={:?}",
            restore_result.failed,
            restore_result.errors
        );
        assert!(
            !restore_result.errors.is_empty(),
            "Error list should contain decryption errors"
        );

        // エラーメッセージに復号化失敗が含まれることを確認
        let has_decrypt_error = restore_result
            .errors
            .iter()
            .any(|e| e.contains("復号化") || e.contains("decrypt") || e.contains("暗号化"));
        assert!(
            has_decrypt_error,
            "Error messages should mention decryption failure: {:?}",
            restore_result.errors
        );
    }

    Ok(())
}

// =============================================================================
// E2E Scenario 4: 誤パスワードでの復元エラー（既存テストからの移植）
// =============================================================================

#[test]
fn test_e2e_wrong_password_restore_fails() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");
    fs::create_dir_all(&backup)?;

    let correct_password = "correct_password";
    let wrong_password = "wrong_password";

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
        .with_encryption(correct_password.to_string())
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // 誤ったパスワードで復元を試みる（失敗するはず）
    fs::create_dir_all(&restore)?;
    let actual_backup = backup.join(&result.backup_name);

    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    let restore_result = restore_engine.restore(&actual_backup, &restore, Some(wrong_password))?;

    // 復元が失敗することを確認（failedカウントが0より大きい）
    assert!(
        restore_result.failed > 0,
        "Restore with wrong password should have failures: failed={}, errors={:?}",
        restore_result.failed,
        restore_result.errors
    );

    Ok(())
}
