// restore_engine_tests.rs - RestoreEngine の統合テスト

use backup_suite::core::restore::RestoreEngine;
use backup_suite::crypto::{EncryptionEngine, KeyManager};
use std::fs;
use std::io::Write as IoWrite;
use tempfile::TempDir;

/// テスト用のバックアップディレクトリを作成
fn create_backup_dir(temp: &TempDir, files: Vec<(&str, &[u8])>) -> std::path::PathBuf {
    let backup_dir = temp.path().join("backup");
    fs::create_dir_all(&backup_dir).unwrap();

    for (name, content) in files {
        let file_path = backup_dir.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&file_path, content).unwrap();
    }

    backup_dir
}

/// 暗号化されたバックアップファイルを作成
fn create_encrypted_backup(
    temp: &TempDir,
    password: &str,
    files: Vec<(&str, &[u8])>,
) -> std::path::PathBuf {
    let backup_dir = temp.path().join("encrypted_backup");
    fs::create_dir_all(&backup_dir).unwrap();

    // マスターキー生成
    let key_manager = KeyManager::default();
    let (master_key, salt) = key_manager.create_master_key(password).unwrap();

    // 暗号化エンジン
    let encryption_engine = EncryptionEngine::default();

    for (name, content) in files {
        let file_path = backup_dir.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        // データを暗号化
        let encrypted_data = encryption_engine
            .encrypt(content, &master_key, salt)
            .unwrap();
        let serialized = encrypted_data.to_bytes();
        fs::write(&file_path, serialized).unwrap();
    }

    backup_dir
}

/// Zstd圧縮されたバックアップファイルを作成
fn create_compressed_backup_zstd(temp: &TempDir, files: Vec<(&str, &[u8])>) -> std::path::PathBuf {
    let backup_dir = temp.path().join("compressed_backup_zstd");
    fs::create_dir_all(&backup_dir).unwrap();

    for (name, content) in files {
        let file_path = backup_dir.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        // Zstd圧縮
        let compressed = zstd::encode_all(content, 3).unwrap();
        fs::write(&file_path, compressed).unwrap();
    }

    backup_dir
}

/// Gzip圧縮されたバックアップファイルを作成
fn create_compressed_backup_gzip(temp: &TempDir, files: Vec<(&str, &[u8])>) -> std::path::PathBuf {
    let backup_dir = temp.path().join("compressed_backup_gzip");
    fs::create_dir_all(&backup_dir).unwrap();

    for (name, content) in files {
        let file_path = backup_dir.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        // Gzip圧縮
        use flate2::write::GzEncoder;
        use flate2::Compression;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(content).unwrap();
        let compressed = encoder.finish().unwrap();
        fs::write(&file_path, compressed).unwrap();
    }

    backup_dir
}

// =============================================================================
// Test 1: 暗号化ファイルの復元
// =============================================================================

#[test]
fn test_restore_encrypted_files() {
    let temp = TempDir::new().unwrap();
    let password = "test_password_123";

    // 暗号化バックアップを作成
    let backup_dir = create_encrypted_backup(
        &temp,
        password,
        vec![
            ("file1.txt", b"encrypted content 1"),
            ("subdir/file2.txt", b"encrypted content 2"),
        ],
    );

    let restore_dir = temp.path().join("restore");

    // 復元実行
    let mut engine = RestoreEngine::new(false)
        .with_progress(false)
        .with_verification(false);
    let result = engine
        .restore(&backup_dir, &restore_dir, Some(password))
        .unwrap();

    // 検証
    assert_eq!(result.total_files, 2);
    assert_eq!(result.restored, 2);
    assert_eq!(result.failed, 0);
    assert_eq!(result.encrypted_files, 2);

    // 復元されたファイルの内容を確認
    let content1 = fs::read_to_string(restore_dir.join("file1.txt")).unwrap();
    assert_eq!(content1, "encrypted content 1");

    let content2 = fs::read_to_string(restore_dir.join("subdir/file2.txt")).unwrap();
    assert_eq!(content2, "encrypted content 2");
}

// =============================================================================
// Test 2: 暗号化ファイルでパスワード未指定エラー
// =============================================================================

#[test]
fn test_restore_encrypted_no_password_fails() {
    let temp = TempDir::new().unwrap();
    let password = "test_password_123";

    let backup_dir = create_encrypted_backup(&temp, password, vec![("file1.txt", b"test")]);
    let restore_dir = temp.path().join("restore");

    let mut engine = RestoreEngine::new(false)
        .with_progress(false)
        .with_verification(false);
    let result = engine.restore(&backup_dir, &restore_dir, None).unwrap();

    // パスワード未指定のため失敗
    assert_eq!(result.failed, 1);
    assert_eq!(result.restored, 0);
    assert!(!result.errors.is_empty());
}

// =============================================================================
// Test 3: 暗号化ファイルで間違ったパスワード
// =============================================================================

#[test]
fn test_restore_encrypted_wrong_password_fails() {
    let temp = TempDir::new().unwrap();
    let correct_password = "correct_password";
    let wrong_password = "wrong_password";

    let backup_dir = create_encrypted_backup(&temp, correct_password, vec![("file1.txt", b"test")]);
    let restore_dir = temp.path().join("restore");

    let mut engine = RestoreEngine::new(false)
        .with_progress(false)
        .with_verification(false);
    let result = engine
        .restore(&backup_dir, &restore_dir, Some(wrong_password))
        .unwrap();

    // 復号化失敗
    assert_eq!(result.failed, 1);
    assert_eq!(result.restored, 0);
}

// =============================================================================
// Test 4: Zstd圧縮ファイルの復元
// =============================================================================

#[test]
fn test_restore_zstd_compressed() {
    let temp = TempDir::new().unwrap();

    let backup_dir =
        create_compressed_backup_zstd(&temp, vec![("file1.txt", b"zstd compressed content")]);
    let restore_dir = temp.path().join("restore");

    let mut engine = RestoreEngine::new(false)
        .with_progress(false)
        .with_verification(false);
    let result = engine.restore(&backup_dir, &restore_dir, None).unwrap();

    assert_eq!(result.total_files, 1);
    assert_eq!(result.restored, 1);
    assert_eq!(result.failed, 0);

    let content = fs::read_to_string(restore_dir.join("file1.txt")).unwrap();
    assert_eq!(content, "zstd compressed content");
}

// =============================================================================
// Test 5: Gzip圧縮ファイルの復元
// =============================================================================

#[test]
fn test_restore_gzip_compressed() {
    let temp = TempDir::new().unwrap();

    let backup_dir =
        create_compressed_backup_gzip(&temp, vec![("file1.txt", b"gzip compressed content")]);
    let restore_dir = temp.path().join("restore");

    let mut engine = RestoreEngine::new(false)
        .with_progress(false)
        .with_verification(false);
    let result = engine.restore(&backup_dir, &restore_dir, None).unwrap();

    assert_eq!(result.total_files, 1);
    assert_eq!(result.restored, 1);
    assert_eq!(result.failed, 0);

    let content = fs::read_to_string(restore_dir.join("file1.txt")).unwrap();
    assert_eq!(content, "gzip compressed content");
}

// =============================================================================
// Test 6: 存在しないバックアップディレクトリ
// =============================================================================

#[test]
fn test_restore_nonexistent_backup_dir() {
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("nonexistent");
    let restore_dir = temp.path().join("restore");

    let mut engine = RestoreEngine::new(false).with_progress(false);
    let result = engine.restore(&backup_dir, &restore_dir, None);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("存在しません"));
}

// =============================================================================
// Test 7: with_progress設定
// =============================================================================

#[test]
fn test_restore_with_progress_enabled() {
    let temp = TempDir::new().unwrap();
    let backup_dir = create_backup_dir(&temp, vec![("file1.txt", b"test")]);
    let restore_dir = temp.path().join("restore");

    let mut engine = RestoreEngine::new(false).with_progress(true);
    let result = engine.restore(&backup_dir, &restore_dir, None).unwrap();

    assert_eq!(result.total_files, 1);
    assert_eq!(result.restored, 1);
}

// =============================================================================
// Test 8: with_verification設定（メタデータなし）
// =============================================================================

#[test]
fn test_restore_with_verification_no_metadata() {
    let temp = TempDir::new().unwrap();
    let backup_dir = create_backup_dir(&temp, vec![("file1.txt", b"test")]);
    let restore_dir = temp.path().join("restore");

    let mut engine = RestoreEngine::new(false)
        .with_progress(false)
        .with_verification(true);
    let result = engine.restore(&backup_dir, &restore_dir, None).unwrap();

    assert_eq!(result.total_files, 1);
    assert_eq!(result.restored, 1);
    assert_eq!(result.verified_files, 0); // メタデータなしのため0
}

// =============================================================================
// Test 9: 複数ファイルの復元
// =============================================================================

#[test]
fn test_restore_multiple_files() {
    let temp = TempDir::new().unwrap();
    let backup_dir = create_backup_dir(
        &temp,
        vec![
            ("file1.txt", b"content1"),
            ("file2.txt", b"content2"),
            ("dir1/file3.txt", b"content3"),
            ("dir1/dir2/file4.txt", b"content4"),
        ],
    );
    let restore_dir = temp.path().join("restore");

    let mut engine = RestoreEngine::new(false)
        .with_progress(false)
        .with_verification(false);
    let result = engine.restore(&backup_dir, &restore_dir, None).unwrap();

    assert_eq!(result.total_files, 4);
    assert_eq!(result.restored, 4);
    assert_eq!(result.failed, 0);

    // すべてのファイルが正しく復元されているか確認
    assert_eq!(
        fs::read_to_string(restore_dir.join("file1.txt")).unwrap(),
        "content1"
    );
    assert_eq!(
        fs::read_to_string(restore_dir.join("file2.txt")).unwrap(),
        "content2"
    );
    assert_eq!(
        fs::read_to_string(restore_dir.join("dir1/file3.txt")).unwrap(),
        "content3"
    );
    assert_eq!(
        fs::read_to_string(restore_dir.join("dir1/dir2/file4.txt")).unwrap(),
        "content4"
    );
}

// =============================================================================
// Test 10: 暗号化+圧縮の復元
// =============================================================================

#[test]
fn test_restore_encrypted_and_compressed() {
    let temp = TempDir::new().unwrap();
    let password = "test_password";

    // データをZstd圧縮してから暗号化
    let original_content = b"This is a test content for compression and encryption";
    let compressed = zstd::encode_all(&original_content[..], 3).unwrap();

    // 暗号化
    let key_manager = KeyManager::default();
    let (master_key, salt) = key_manager.create_master_key(password).unwrap();
    let encryption_engine = EncryptionEngine::default();
    let encrypted_data = encryption_engine
        .encrypt(&compressed, &master_key, salt)
        .unwrap();

    // バックアップディレクトリに保存
    let backup_dir = temp.path().join("encrypted_compressed_backup");
    fs::create_dir_all(&backup_dir).unwrap();
    fs::write(backup_dir.join("file.txt"), encrypted_data.to_bytes()).unwrap();

    let restore_dir = temp.path().join("restore");

    // 復元
    let mut engine = RestoreEngine::new(false)
        .with_progress(false)
        .with_verification(false);
    let result = engine
        .restore(&backup_dir, &restore_dir, Some(password))
        .unwrap();

    assert_eq!(result.total_files, 1);
    assert_eq!(result.restored, 1);
    assert_eq!(result.encrypted_files, 1);

    // 復元されたファイルが元のコンテンツと一致するか確認
    let restored_content = fs::read(restore_dir.join("file.txt")).unwrap();
    assert_eq!(restored_content, original_content);
}

// =============================================================================
// Test 11: 空のバックアップディレクトリ
// =============================================================================

#[test]
fn test_restore_empty_backup_dir() {
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("empty_backup");
    fs::create_dir_all(&backup_dir).unwrap();

    let restore_dir = temp.path().join("restore");

    let mut engine = RestoreEngine::new(false).with_progress(false);
    let result = engine.restore(&backup_dir, &restore_dir, None).unwrap();

    assert_eq!(result.total_files, 0);
    assert_eq!(result.restored, 0);
    assert_eq!(result.failed, 0);
}

// =============================================================================
// Test 12: ドライランモード（複数ファイル）
// =============================================================================

#[test]
fn test_restore_dry_run_multiple_files() {
    let temp = TempDir::new().unwrap();
    let backup_dir = create_backup_dir(
        &temp,
        vec![
            ("file1.txt", b"content1"),
            ("file2.txt", b"content2"),
            ("dir/file3.txt", b"content3"),
        ],
    );
    let restore_dir = temp.path().join("restore");

    let mut engine = RestoreEngine::new(true).with_progress(false);
    let result = engine.restore(&backup_dir, &restore_dir, None).unwrap();

    assert_eq!(result.total_files, 3);
    assert_eq!(result.restored, 0); // ドライランなので0
    assert_eq!(result.failed, 0);
    assert!(!restore_dir.exists()); // ディレクトリは作成されない
}

// =============================================================================
// Test 13: .integrityファイルの除外確認
// =============================================================================

#[test]
fn test_restore_excludes_integrity_file() {
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("backup");
    fs::create_dir_all(&backup_dir).unwrap();

    // 通常のファイルと.integrityファイルを作成
    fs::write(backup_dir.join("file1.txt"), b"content1").unwrap();
    fs::write(backup_dir.join(".integrity"), b"integrity metadata").unwrap();

    let restore_dir = temp.path().join("restore");

    let mut engine = RestoreEngine::new(false).with_progress(false);
    let result = engine.restore(&backup_dir, &restore_dir, None).unwrap();

    // .integrityファイルは除外されるため1ファイルのみ
    assert_eq!(result.total_files, 1);
    assert_eq!(result.restored, 1);

    // .integrityファイルは復元されていないことを確認
    assert!(!restore_dir.join(".integrity").exists());
    assert!(restore_dir.join("file1.txt").exists());
}

// =============================================================================
// Test 14: エラー発生時のエラーリスト確認
// =============================================================================

#[test]
fn test_restore_error_list_populated() {
    let temp = TempDir::new().unwrap();
    let password = "correct_password";

    // 暗号化バックアップを作成
    let backup_dir = create_encrypted_backup(&temp, password, vec![("file1.txt", b"test")]);
    let restore_dir = temp.path().join("restore");

    // パスワード未指定で復元（エラーが発生）
    let mut engine = RestoreEngine::new(false).with_progress(false);
    let result = engine.restore(&backup_dir, &restore_dir, None).unwrap();

    assert_eq!(result.failed, 1);
    assert!(!result.errors.is_empty());
    assert!(result.errors[0].contains("パスワードが未指定"));
}
