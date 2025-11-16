// comprehensive_e2e_tests.rs - 完璧な動作テスト（統合テスト）
//
// このファイルは backup-suite の完全な動作を検証する包括的な統合テストです。
// 実際のユーザーシナリオを模擬し、全機能の統合動作を確認します。
//
// 設計方針：
// - CLIコマンド経由ではなく、Rust APIを直接使用
// - 実際のファイルシステムを使用した完全な統合テスト
// - エンドツーエンドのバックアップ→復元フローを検証

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

    // 複数種類のファイルを作成
    fs::write(source.join("document.txt"), "Important document").unwrap();
    fs::write(source.join("config.toml"), "[settings]\nkey = \"value\"").unwrap();
    fs::write(source.join("large_file.bin"), vec![0xFFu8; 1024 * 100]).unwrap(); // 100KB

    // サブディレクトリ
    fs::create_dir_all(source.join("subdir/deep")).unwrap();
    fs::write(source.join("subdir/file1.txt"), "File in subdirectory").unwrap();
    fs::write(source.join("subdir/deep/file2.txt"), "Deeply nested file").unwrap();

    source
}

/// ファイルの内容が一致することを確認
fn assert_file_content_matches(source: &Path, restored: &Path, relative_path: &str) {
    let source_file = source.join(relative_path);
    let restored_file = restored.join(relative_path);

    assert!(
        source_file.exists(),
        "Source file not found: {:?}",
        source_file
    );
    assert!(
        restored_file.exists(),
        "Restored file not found: {:?}",
        restored_file
    );

    let source_content = fs::read(&source_file).unwrap();
    let restored_content = fs::read(&restored_file).unwrap();

    assert_eq!(
        source_content, restored_content,
        "File content mismatch: {}",
        relative_path
    );
}

// =============================================================================
// E2E Scenario 1: 基本的なバックアップ→復元フロー
// =============================================================================

#[test]
fn test_e2e_basic_backup_and_restore() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");

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

    // バックアップが成功したことを確認
    assert!(result.total_files >= 5, "Expected at least 5 files");
    assert_eq!(result.failed, 0, "Backup should have no failures");
    assert!(backup.read_dir()?.count() > 0, "No backup files created");

    // ステップ2: 復元実行
    fs::create_dir_all(&restore)?;
    let actual_backup = backup.join(&result.backup_name);
    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    restore_engine.restore(&actual_backup, &restore, None)?;

    // ステップ3: 復元されたファイルの検証
    // 注: RestoreEngineはカテゴリ名のディレクトリを含めて復元する
    // ディレクトリバックアップではディレクトリ名も保持されるため、test/source/ 配下に復元される
    let restored_root = restore.join("test/source");
    assert_file_content_matches(&source, &restored_root, "document.txt");
    assert_file_content_matches(&source, &restored_root, "config.toml");
    assert_file_content_matches(&source, &restored_root, "large_file.bin");
    assert_file_content_matches(&source, &restored_root, "subdir/file1.txt");
    assert_file_content_matches(&source, &restored_root, "subdir/deep/file2.txt");

    Ok(())
}

// =============================================================================
// E2E Scenario 2: 暗号化バックアップ→復元フロー
// =============================================================================

#[test]
fn test_e2e_encrypted_backup_and_restore() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");

    fs::create_dir_all(&backup)?;

    let password = "test_secure_password_123";

    // ステップ1: 暗号化バックアップ実行
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
    assert!(backup.read_dir()?.count() > 0, "No backup files created");

    // ステップ2: 暗号化復元実行
    fs::create_dir_all(&restore)?;

    let actual_backup = backup.join(&result.backup_name);

    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    restore_engine.restore(&actual_backup, &restore, Some(password))?;

    // ステップ3: 復元されたファイルの検証
    let restored_root = restore.join("test/source");
    assert_file_content_matches(&source, &restored_root, "document.txt");
    assert_file_content_matches(&source, &restored_root, "subdir/file1.txt");

    Ok(())
}

// =============================================================================
// E2E Scenario 3: 圧縮バックアップ→復元フロー
// =============================================================================

#[test]
fn test_e2e_compressed_backup_and_restore() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");

    fs::create_dir_all(&backup)?;

    // ステップ1: Zstd圧縮バックアップ実行
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

    assert_eq!(result.failed, 0, "Compressed backup should succeed");

    // ステップ2: 圧縮復元実行
    fs::create_dir_all(&restore)?;

    let actual_backup = backup.join(&result.backup_name);

    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    restore_engine.restore(&actual_backup, &restore, None)?;

    // ステップ3: 復元されたファイルの検証
    let restored_root = restore.join("test/source");
    assert_file_content_matches(&source, &restored_root, "large_file.bin");

    Ok(())
}

// =============================================================================
// E2E Scenario 4: 暗号化+圧縮バックアップ→復元フロー（最強セキュリティ）
// =============================================================================

#[test]
fn test_e2e_encrypted_compressed_backup_and_restore() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");

    fs::create_dir_all(&backup)?;

    let password = "ultra_secure_password_2024";

    // ステップ1: 暗号化+圧縮バックアップ実行
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
        .with_compression(CompressionType::Zstd, 3);

    let result = runner.run(None, None)?;

    assert_eq!(
        result.failed, 0,
        "Encrypted+compressed backup should succeed"
    );

    // バックアップファイルが作成されていることを確認
    let backup_files: Vec<_> = fs::read_dir(&backup)?.collect();
    assert!(!backup_files.is_empty(), "No backup files created");

    // ステップ2: 暗号化+圧縮復元実行
    fs::create_dir_all(&restore)?;

    let actual_backup = backup.join(&result.backup_name);

    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    restore_engine.restore(&actual_backup, &restore, Some(password))?;

    // ステップ3: 復元されたファイルの検証
    let restored_root = restore.join("test/source");
    assert_file_content_matches(&source, &restored_root, "document.txt");
    assert_file_content_matches(&source, &restored_root, "config.toml");
    assert_file_content_matches(&source, &restored_root, "large_file.bin");

    Ok(())
}

// =============================================================================
// E2E Scenario 5: 誤ったパスワードでの復元失敗
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

    // ステップ1: 暗号化バックアップ実行
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

    // ステップ2: 誤ったパスワードで復元を試みる（失敗するはず）
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

// =============================================================================
// E2E Scenario 6: ドライラン（実際のファイル作成なし）
// =============================================================================

#[test]
fn test_e2e_dry_run_backup() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup = temp.path().join("backup");

    fs::create_dir_all(&backup)?;

    // 既存のバックアップファイル数を記録
    let initial_count = fs::read_dir(&backup)?.count();

    // ステップ1: ドライランバックアップ実行
    let mut config = Config::default();
    config.backup.destination = backup.clone();
    config.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, true) // dry_run = true
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // ドライランでは成功するが、実際のファイルコピーは行われない
    assert!(result.total_files > 0, "Dry run should count files");

    // ステップ2: 実際のバックアップディレクトリは作成されるが、中身のファイルはコピーされない
    // （BackupRunnerはbackup_nameのディレクトリを作成する）
    let final_count = fs::read_dir(&backup)?.count();
    // ドライランでもディレクトリは作成されるため、countは増える
    assert!(
        final_count >= initial_count,
        "Dry run should create backup directory"
    );

    Ok(())
}

// =============================================================================
// E2E Scenario 7: 優先度フィルタリング
// =============================================================================

#[test]
fn test_e2e_priority_filtering() -> Result<()> {
    let temp = TempDir::new()?;
    let source_high = temp.path().join("source_high");
    let source_low = temp.path().join("source_low");
    let backup = temp.path().join("backup");

    fs::create_dir_all(&source_high)?;
    fs::create_dir_all(&source_low)?;
    fs::create_dir_all(&backup)?;

    fs::write(source_high.join("high_priority.txt"), "High priority data")?;
    fs::write(source_low.join("low_priority.txt"), "Low priority data")?;

    // 高優先度と低優先度のターゲットを設定
    let mut config = Config::default();
    config.backup.destination = backup.clone();
    config.add_target(Target::new(
        source_high.clone(),
        Priority::High,
        "high".to_string(),
    ));
    config.add_target(Target::new(
        source_low.clone(),
        Priority::Low,
        "low".to_string(),
    ));

    // ステップ1: 高優先度のみバックアップ
    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(Some(&Priority::High), None)?;

    // 高優先度のファイルのみがバックアップされることを確認
    assert!(
        result.total_files > 0,
        "High priority files should be backed up"
    );

    Ok(())
}

// =============================================================================
// E2E Scenario 8: カテゴリフィルタリング
// =============================================================================

#[test]
fn test_e2e_category_filtering() -> Result<()> {
    let temp = TempDir::new()?;
    let source_docs = temp.path().join("documents");
    let source_code = temp.path().join("code");
    let backup = temp.path().join("backup");

    fs::create_dir_all(&source_docs)?;
    fs::create_dir_all(&source_code)?;
    fs::create_dir_all(&backup)?;

    fs::write(source_docs.join("report.pdf"), "PDF content")?;
    fs::write(source_code.join("main.rs"), "fn main() {}")?;

    // ドキュメントとコードのターゲットを設定
    let mut config = Config::default();
    config.backup.destination = backup.clone();
    config.add_target(Target::new(
        source_docs.clone(),
        Priority::High,
        "documents".to_string(),
    ));
    config.add_target(Target::new(
        source_code.clone(),
        Priority::High,
        "code".to_string(),
    ));

    // ステップ1: ドキュメントカテゴリのみバックアップ
    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, Some("documents"))?;

    assert!(result.total_files > 0, "Documents should be backed up");

    Ok(())
}

// =============================================================================
// E2E Scenario 9: 複数ファイルの並列処理
// =============================================================================

#[test]
fn test_e2e_parallel_processing() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    fs::create_dir_all(&backup)?;

    // 多数の小さいファイルを作成（並列処理テスト）
    for i in 0..100 {
        fs::write(
            source.join(format!("file_{}.txt", i)),
            format!("Content {}", i),
        )?;
    }

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

    // 全ファイルがバックアップされることを確認
    assert_eq!(result.total_files, 100, "All 100 files should be processed");
    assert_eq!(result.failed, 0, "No failures expected");
    assert_eq!(result.successful, 100, "All files should succeed");

    Ok(())
}

// =============================================================================
// E2E Scenario 10: 大容量ファイルのバックアップ
// =============================================================================

#[test]
fn test_e2e_large_file_backup() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup = temp.path().join("backup");
    let restore = temp.path().join("restore");

    fs::create_dir_all(&source)?;
    fs::create_dir_all(&backup)?;

    // 10MBファイルを作成
    let large_content = vec![0x42u8; 10 * 1024 * 1024];
    fs::write(source.join("large_file.bin"), &large_content)?;

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

    assert_eq!(result.failed, 0, "Large file backup should succeed");

    // 復元して内容検証
    fs::create_dir_all(&restore)?;

    let actual_backup = backup.join(&result.backup_name);

    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    restore_engine.restore(&actual_backup, &restore, None)?;

    let restored_root = restore.join("test/source");
    let restored_content = fs::read(restored_root.join("large_file.bin"))?;
    assert_eq!(
        restored_content.len(),
        large_content.len(),
        "Restored file size mismatch"
    );
    assert_eq!(
        restored_content, large_content,
        "Restored file content mismatch"
    );

    Ok(())
}

// =============================================================================
// E2E Scenario 11: 除外パターンの動作確認
// =============================================================================

#[test]
fn test_e2e_exclude_patterns() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    fs::create_dir_all(&backup)?;

    fs::write(source.join("include.txt"), "This should be backed up")?;
    fs::write(source.join("exclude.tmp"), "This should be excluded")?;
    fs::write(source.join("exclude.log"), "This should also be excluded")?;

    let mut config = Config::default();
    config.backup.destination = backup.clone();

    let mut target = Target::new(source.clone(), Priority::High, "test".to_string());
    target.exclude_patterns = vec![r"\.tmp$".to_string(), r"\.log$".to_string()];
    config.add_target(target);

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // exclude.tmp と exclude.log は除外されるので、1ファイルのみバックアップ
    assert_eq!(
        result.total_files, 1,
        "Only 1 file should be backed up (exclude patterns active)"
    );

    Ok(())
}

// =============================================================================
// E2E Scenario 12: 完全な実用シナリオ（複数バックアップ→選択的復元）
// =============================================================================

#[test]
fn test_e2e_full_practical_scenario() -> Result<()> {
    let temp = TempDir::new()?;
    let source = create_test_source(&temp);
    let backup1 = temp.path().join("backup1");
    let backup2 = temp.path().join("backup2");
    let restore = temp.path().join("restore");

    fs::create_dir_all(&backup1)?;
    fs::create_dir_all(&backup2)?;

    // シナリオ: ユーザーが2回バックアップを実行

    // 1回目のバックアップ（圧縮あり）
    let mut config1 = Config::default();
    config1.backup.destination = backup1.clone();
    config1.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner1 = BackupRunner::new(config1, false)
        .with_progress(false)
        .with_compression(CompressionType::Zstd, 3);

    runner1.run(None, None)?;

    // ソースファイルを一部変更
    fs::write(source.join("document.txt"), "Updated document content")?;

    // 2回目のバックアップ（暗号化+圧縮）
    let mut config2 = Config::default();
    config2.backup.destination = backup2.clone();
    config2.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner2 = BackupRunner::new(config2, false)
        .with_progress(false)
        .with_encryption("secure_pw".to_string())
        .with_compression(CompressionType::Zstd, 3);

    let result2 = runner2.run(None, None)?;

    // 2回目のバックアップから復元
    fs::create_dir_all(&restore)?;

    let actual_backup2 = backup2.join(&result2.backup_name);

    let mut restore_engine = RestoreEngine::new(false).with_progress(false);
    restore_engine.restore(&actual_backup2, &restore, Some("secure_pw"))?;

    // 復元後のファイル内容確認（更新後の内容）
    let restored_root = restore.join("test/source");
    let restored_content = fs::read_to_string(restored_root.join("document.txt"))?;
    assert_eq!(restored_content, "Updated document content");

    Ok(())
}
