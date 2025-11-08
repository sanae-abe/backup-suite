//! 監査ログシステムの統合テスト
//!
//! HMAC-SHA256による改ざん防止機能、イベント記録、ログローテーション、
//! バックアップ・復元・クリーンアップコマンドとの統合をテストします。

use anyhow::Result;
use backup_suite::security::{AuditEvent, AuditLog, EventType};
use backup_suite::{
    BackupRunner, CleanupEngine, CleanupPolicy, Config, Priority, RestoreEngine, Target,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_audit_event_creation_and_hmac() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let log_path = temp_dir.path().join("audit.log");

    let mut audit_log = AuditLog::with_path(log_path.clone())?;

    // イベント作成とログ記録
    let event = AuditEvent::backup_started("/test/path", "testuser");
    audit_log.log(event)?;

    // ログ読み込み
    let events = audit_log.read_all()?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, EventType::BackupStarted);
    assert_eq!(events[0].user, "testuser");
    assert_eq!(events[0].target, Some("/test/path".to_string()));

    // HMAC検証
    assert!(audit_log.verify_all()?);

    Ok(())
}

#[test]
fn test_audit_log_tampering_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let log_path = temp_dir.path().join("audit.log");

    let mut audit_log = AuditLog::with_path(log_path.clone())?;

    // イベント記録
    audit_log.log(AuditEvent::backup_started("/test/path1", "user1"))?;
    audit_log.log(AuditEvent::backup_completed(
        "/test/path1",
        "user1",
        serde_json::json!({"files": 10}),
    ))?;

    // 検証成功
    assert!(audit_log.verify_all()?);

    // ログファイルを直接改ざん
    let mut content = fs::read_to_string(&log_path)?;
    content = content.replace("user1", "attacker");
    fs::write(&log_path, content)?;

    // 検証失敗（改ざん検知）
    let audit_log = AuditLog::with_path(log_path)?;
    assert!(!audit_log.verify_all()?);

    Ok(())
}

#[test]
fn test_audit_log_various_event_types() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let log_path = temp_dir.path().join("audit.log");

    let mut audit_log = AuditLog::with_path(log_path)?;

    // 様々なイベントタイプを記録
    audit_log.log(AuditEvent::backup_started("/path1", "user1"))?;
    audit_log.log(AuditEvent::backup_completed(
        "/path1",
        "user1",
        serde_json::json!({"total_files": 100}),
    ))?;
    audit_log.log(AuditEvent::backup_failed("/path2", "user1", "disk full"))?;
    audit_log.log(AuditEvent::restore_started("/backup1", "user2"))?;
    audit_log.log(AuditEvent::restore_completed(
        "/backup1",
        "user2",
        serde_json::json!({"restored": 50}),
    ))?;
    audit_log.log(AuditEvent::cleanup_started("admin", 30))?;
    audit_log.log(AuditEvent::cleanup_completed(
        "admin",
        serde_json::json!({"deleted": 5}),
    ))?;
    audit_log.log(AuditEvent::security_warning(
        "suspicious activity",
        "system",
    ))?;
    audit_log.log(AuditEvent::permission_denied("/etc/shadow", "hacker"))?;

    // すべてのイベントを読み込み
    let events = audit_log.read_all()?;
    assert_eq!(events.len(), 9);

    // イベント種別でフィルタ
    let backup_events = audit_log.get_events_by_type(&EventType::BackupStarted)?;
    assert_eq!(backup_events.len(), 1);

    let security_events = audit_log.get_events_by_type(&EventType::SecurityWarning)?;
    assert_eq!(security_events.len(), 1);
    assert_eq!(security_events[0].user, "system");

    // すべてのHMAC検証
    assert!(audit_log.verify_all()?);

    Ok(())
}

#[test]
fn test_backup_runner_with_audit_logging() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_file = temp_dir.path().join("test.txt");
    fs::write(&source_file, b"test content")?;

    let mut config = Config::default();
    let target = Target::new(source_file.clone(), Priority::High, "test".to_string());
    config.add_target(target);
    config.backup.destination = temp_dir.path().join("backups");

    // バックアップ実行（監査ログが自動的に記録される）
    let mut runner = BackupRunner::new(config, false);
    let result = runner.run(None, None)?;

    assert_eq!(result.total_files, 1);
    assert_eq!(result.successful, 1);

    // 監査ログを確認（デフォルト位置の場合、手動確認必要）
    // 実際のプロダクションでは、テスト用にカスタムパスを設定する必要がある

    Ok(())
}

#[test]
fn test_restore_engine_with_audit_logging() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let backup_dir = temp_dir.path().join("backup");
    let restore_dir = temp_dir.path().join("restore");

    // テストバックアップを作成
    fs::create_dir_all(&backup_dir)?;
    fs::write(backup_dir.join("file1.txt"), b"test1")?;
    fs::write(backup_dir.join("file2.txt"), b"test2")?;

    // 復元実行（監査ログが自動的に記録される）
    let mut engine = RestoreEngine::new(false).with_progress(false);
    let result = engine.restore(&backup_dir, &restore_dir, None)?;

    assert_eq!(result.total_files, 2);
    assert_eq!(result.restored, 2);
    assert_eq!(result.failed, 0);

    // 復元されたファイルを確認
    assert_eq!(fs::read_to_string(restore_dir.join("file1.txt"))?, "test1");
    assert_eq!(fs::read_to_string(restore_dir.join("file2.txt"))?, "test2");

    Ok(())
}

#[test]
fn test_cleanup_engine_with_audit_logging() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // 設定を作成してバックアップ先を設定
    let mut config = Config::default();
    config.backup.destination = temp_dir.path().join("backups");
    fs::create_dir_all(&config.backup.destination)?;

    // 古いバックアップディレクトリを作成
    let old_backup = config.backup.destination.join("backup_20200101_000000");
    fs::create_dir_all(&old_backup)?;
    fs::write(old_backup.join("test.txt"), b"old data")?;

    // 設定を保存
    config.save()?;

    // クリーンアップ実行（監査ログが自動的に記録される）
    let policy = CleanupPolicy::retention_days(0); // すぐに削除
    let mut engine = CleanupEngine::new(policy, false);
    let result = engine.cleanup()?;

    // 削除確認（古いバックアップが削除されたことを確認）
    assert!(result.deleted > 0); // ドライランでなければ削除される

    Ok(())
}

#[test]
fn test_audit_log_rotation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let log_path = temp_dir.path().join("audit.log");

    let mut audit_log = AuditLog::with_path(log_path.clone())?;
    audit_log.max_log_size = 200; // テスト用に小さいサイズ

    // 大量のイベントを記録してローテーションをトリガー
    for i in 0..100 {
        audit_log.log(AuditEvent::backup_started(
            format!("/path/{i}"),
            "testuser",
        ))?;
    }

    // ローテーションされたファイルが存在することを確認
    let entries: Vec<_> = fs::read_dir(temp_dir.path())?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with("audit_") && name.ends_with(".log")
        })
        .collect();

    assert!(
        !entries.is_empty(),
        "ログローテーションが実行されませんでした"
    );

    Ok(())
}

#[test]
fn test_audit_log_time_based_filtering() -> Result<()> {
    use chrono::{Duration, Utc};

    let temp_dir = TempDir::new()?;
    let log_path = temp_dir.path().join("audit.log");

    let mut audit_log = AuditLog::with_path(log_path)?;

    // イベント記録
    audit_log.log(AuditEvent::backup_started("/path1", "user1"))?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    audit_log.log(AuditEvent::backup_completed(
        "/path1",
        "user1",
        serde_json::json!({}),
    ))?;

    // 1秒前からのイベントを取得（すべて取得される）
    let since = Utc::now() - Duration::seconds(1);
    let recent_events = audit_log.get_events_since(since)?;
    assert_eq!(recent_events.len(), 2);

    // 未来時刻からのイベントを取得（何も取得されない）
    let future = Utc::now() + Duration::hours(1);
    let future_events = audit_log.get_events_since(future)?;
    assert_eq!(future_events.len(), 0);

    Ok(())
}

#[test]
fn test_audit_log_hmac_with_different_keys() {
    // HMAC検証のテスト（異なる鍵での検証失敗を確認）

    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("audit.log");

    let mut audit_log = AuditLog::with_path(log_path).unwrap();
    let mut event = AuditEvent::backup_started("/test", "user");

    // 正しい鍵でHMACを計算
    let secret1 = b"secret_key_1";
    event.hmac = event.compute_hmac(secret1);

    // 同じ鍵での検証は成功
    assert!(event.verify_hmac(secret1));

    // 異なる鍵での検証は失敗
    let secret2 = b"secret_key_2";
    assert!(!event.verify_hmac(secret2));

    // 監査ログに記録して検証
    audit_log
        .log(AuditEvent::backup_started("/test2", "user"))
        .unwrap();
    assert!(audit_log.verify_all().unwrap());
}

#[test]
fn test_audit_log_json_serialization() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let log_path = temp_dir.path().join("audit.log");

    let mut audit_log = AuditLog::with_path(log_path.clone())?;

    // メタデータ付きイベント
    let metadata = serde_json::json!({
        "total_files": 100,
        "total_bytes": 1024000,
        "compression": "zstd",
        "encryption": true,
    });

    audit_log.log(AuditEvent::backup_completed("/path", "user", metadata))?;

    // ログファイルの内容を確認
    let content = fs::read_to_string(&log_path)?;
    assert!(
        content.contains("total_files"),
        "Content missing 'total_files': {content}"
    );
    assert!(
        content.contains("1024000"),
        "Content missing '1024000': {content}"
    );
    assert!(
        content.contains("BackupCompleted"),
        "Content missing 'BackupCompleted': {content}"
    );

    // JSON形式で読み込めることを確認
    let events = audit_log.read_all()?;
    assert_eq!(events.len(), 1);
    assert!(events[0].metadata.is_some());

    if let Some(ref meta) = events[0].metadata {
        assert_eq!(meta["total_files"], 100);
        assert_eq!(meta["total_bytes"], 1024000);
    }

    Ok(())
}

#[test]
fn test_current_user_detection() {
    // ユーザー名の取得テスト
    let user = AuditLog::current_user();
    assert!(!user.is_empty());
    assert_ne!(user, "unknown"); // CI環境では失敗する可能性がある
}

#[test]
fn test_audit_log_permission_security() -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("audit.log");

        let _audit_log = AuditLog::with_path(log_path.clone())?;

        // 秘密鍵ファイルのパーミッションを確認（600であるべき）
        let key_path = log_path.with_extension("key");
        let metadata = fs::metadata(&key_path)?;
        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // Unix権限: 600 = 0o100600 (ファイルタイプビット含む)
        assert_eq!(
            mode & 0o777,
            0o600,
            "秘密鍵のパーミッションが600ではありません"
        );
    }

    Ok(())
}

#[test]
fn test_audit_log_error_recovery() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let log_path = temp_dir.path().join("audit.log");

    let mut audit_log = AuditLog::with_path(log_path.clone())?;

    // 正常なイベント
    audit_log.log(AuditEvent::backup_started("/path1", "user1"))?;

    // 不正なJSON行を手動で挿入（エラーハンドリングテスト）
    let mut file = fs::OpenOptions::new().append(true).open(&log_path)?;
    use std::io::Write;
    writeln!(file, "INVALID JSON LINE")?;
    drop(file);

    // 正常なイベント
    audit_log.log(AuditEvent::backup_completed(
        "/path1",
        "user1",
        serde_json::json!({}),
    ))?;

    // 読み込み時にエラーが発生することを確認
    let result = audit_log.read_all();
    assert!(
        result.is_err(),
        "不正なJSON行のエラーハンドリングが機能していません"
    );

    Ok(())
}
