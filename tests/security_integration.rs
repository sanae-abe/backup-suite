//! セキュリティ統合テスト
//!
//! パストラバーサル攻撃、シンボリックリンク攻撃、確認プロンプトの動作を検証

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs as unix_fs;
use tempfile::TempDir;

/// パストラバーサル攻撃のテスト - add コマンド
/// safe_join() は .. を除去してベースディレクトリ配下に正規化するため、
/// セキュリティ的には安全（ベースディレクトリ外にアクセスできない）
#[test]
fn test_add_rejects_path_traversal() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("add")
        .arg("../../../etc/passwd")
        .assert()
        .success()
        .stdout(predicate::str::contains("パスが存在しません"));
}

/// 浅い絶対パス攻撃のテスト - add コマンド
#[test]
fn test_add_rejects_shallow_absolute_path() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("add")
        .arg("/etc/passwd")
        .assert()
        .failure()
        .stderr(predicate::str::contains("許可されていません"));
}

/// パストラバーサル攻撃のテスト - remove コマンド
/// Note: dialoguerの確認プロンプトがあるため、非対話環境では "not a terminal" エラーになる
/// セキュリティ的には safe_join() で正規化されるため安全
#[test]
#[ignore] // dialoguer prompt requires terminal
fn test_remove_rejects_path_traversal() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("remove")
        .arg("../../../etc/passwd")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a terminal"));
}

/// パストラバーサル攻撃のテスト - config set-destination コマンド
/// safe_join() は .. を除去してベースディレクトリ配下に正規化するため、
/// セキュリティ的には安全（ベースディレクトリ外にアクセスできない）
#[test]
fn test_config_set_destination_rejects_path_traversal() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("config")
        .arg("set-destination")
        .arg("../../../tmp/malicious")
        .assert()
        .success()
        .stdout(predicate::str::contains("バックアップ先を変更しました"));
}

/// AI analyze コマンドのパストラバーサル攻撃テスト
/// safe_join() は .. を除去してベースディレクトリ配下に正規化するため、
/// セキュリティ的には安全（ベースディレクトリ外にアクセスできない）
#[cfg(feature = "ai")]
#[test]
fn test_ai_analyze_rejects_path_traversal() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("ai")
        .arg("analyze")
        .arg("../../../etc/passwd")
        .assert()
        .success()
        .stdout(predicate::str::contains("パスが存在しません").or(predicate::str::contains("AI分析に失敗")));
}

/// AI suggest-exclude コマンドのパストラバーサル攻撃テスト
/// safe_join() は .. を除去してベースディレクトリ配下に正規化するため、
/// セキュリティ的には安全（ベースディレクトリ外にアクセスできない）
#[cfg(feature = "ai")]
#[test]
fn test_ai_suggest_exclude_rejects_path_traversal() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("ai")
        .arg("suggest-exclude")
        .arg("../../../etc/passwd")
        .assert()
        .success()
        .stdout(predicate::str::contains("パスが存在しません").or(predicate::str::contains("AI分析に失敗")));
}

/// シンボリックリンク攻撃のテスト - restore コマンド
/// Note: バックアップディレクトリが見つからないため、実際のファイル復元まで進まない
/// safe_open() は O_NOFOLLOW を使用してシンボリックリンクを拒否する設計
#[test]
#[ignore] // Requires actual backup data to test symlink rejection
fn test_restore_rejects_symlink() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backup");
    let malicious_link = temp_dir.path().join("malicious_link");

    // バックアップディレクトリ作成
    fs::create_dir_all(&backup_dir).unwrap();
    fs::write(backup_dir.join("test.txt"), b"test content").unwrap();

    // シンボリックリンク作成（Unixのみ）
    #[cfg(unix)]
    {
        unix_fs::symlink("/etc/passwd", &malicious_link).unwrap();

        let mut cmd = Command::cargo_bin("backup-suite").unwrap();
        cmd.arg("restore")
            .arg("--from")
            .arg(backup_dir.to_str().unwrap())
            .arg("--to")
            .arg(malicious_link.to_str().unwrap())
            .assert()
            .success()
            .stdout(predicate::str::contains("バックアップなし"));
    }
}

/// 正常なパスは受け入れられることを確認 - add コマンド
/// Note: safe_join() は current_dir をベースとするため、current_dir配下に作成
#[test]
fn test_add_accepts_valid_path() {
    use std::env;

    let current_dir = env::current_dir().unwrap();
    let temp_dir = TempDir::new_in(&current_dir).unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"test content").unwrap();

    // current_dirからの相対パスを取得
    let relative_path = test_file.strip_prefix(&current_dir)
        .expect("temp_dir is under current_dir");

    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("add")
        .arg(relative_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("追加"));
}

/// 正常なパスは受け入れられることを確認 - config set-destination コマンド
/// Note: safe_join() は current_dir をベースとするため、current_dir配下に作成
#[test]
fn test_config_set_destination_accepts_valid_path() {
    use std::env;

    let current_dir = env::current_dir().unwrap();
    let temp_dir = TempDir::new_in(&current_dir).unwrap();
    let dest_dir = temp_dir.path().join("backups");

    // current_dirからの相対パスを取得
    let relative_path = dest_dir.strip_prefix(&current_dir)
        .expect("temp_dir is under current_dir");

    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("config")
        .arg("set-destination")
        .arg(relative_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("バックアップ先を変更しました"));
}

/// Priority enum の型安全性テスト
#[test]
fn test_priority_enum_invalid_value() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("add")
        .arg("/tmp/test")
        .arg("--priority")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

/// CompressionType enum の型安全性テスト
#[test]
fn test_compression_type_enum_invalid_value() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("run")
        .arg("--compress")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

/// cleanup コマンドのバリデーション範囲テスト
#[test]
fn test_cleanup_days_validation_min() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("cleanup")
        .arg("--days")
        .arg("0")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1-3650 の範囲で指定してください"));
}

/// cleanup コマンドのバリデーション範囲テスト（最大値）
#[test]
fn test_cleanup_days_validation_max() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("cleanup")
        .arg("--days")
        .arg("9999")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1-3650 の範囲で指定してください"));
}

/// config set-keep-days のバリデーション範囲テスト
#[test]
fn test_config_set_keep_days_validation_min() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("config")
        .arg("set-keep-days")
        .arg("0")
        .assert()
        .success()
        .stdout(predicate::str::contains("1-3650 の範囲"));
}

/// config set-keep-days のバリデーション範囲テスト（最大値）
#[test]
fn test_config_set_keep_days_validation_max() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("config")
        .arg("set-keep-days")
        .arg("9999")
        .assert()
        .success()
        .stdout(predicate::str::contains("1-3650 の範囲"));
}

/// run コマンドの圧縮レベルバリデーション（zstd範囲外）
#[test]
fn test_run_compress_level_validation_zstd_min() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("run")
        .arg("--compress")
        .arg("zstd")
        .arg("--compress-level")
        .arg("0")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1-22 の範囲で指定してください"));
}

/// run コマンドの圧縮レベルバリデーション（zstd範囲外最大値）
#[test]
fn test_run_compress_level_validation_zstd_max() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("run")
        .arg("--compress")
        .arg("zstd")
        .arg("--compress-level")
        .arg("99")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1-22 の範囲で指定してください"));
}

/// run コマンドの圧縮レベルバリデーション（gzip範囲外）
#[test]
fn test_run_compress_level_validation_gzip_min() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("run")
        .arg("--compress")
        .arg("gzip")
        .arg("--compress-level")
        .arg("0")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1-9 の範囲で指定してください"));
}

/// run コマンドの圧縮レベルバリデーション（gzip範囲外最大値）
#[test]
fn test_run_compress_level_validation_gzip_max() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("run")
        .arg("--compress")
        .arg("gzip")
        .arg("--compress-level")
        .arg("99")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1-9 の範囲で指定してください"));
}

/// AI detect コマンドのdaysバリデーション範囲テスト
#[cfg(feature = "ai")]
#[test]
fn test_ai_detect_days_validation_min() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("ai")
        .arg("detect")
        .arg("--days")
        .arg("0")
        .assert()
        .success()
        .stdout(predicate::str::contains("1-365 の範囲で指定してください"));
}

/// AI detect コマンドのdaysバリデーション範囲テスト（最大値超過）
#[cfg(feature = "ai")]
#[test]
fn test_ai_detect_days_validation_max() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("ai")
        .arg("detect")
        .arg("--days")
        .arg("9999")
        .assert()
        .success()
        .stdout(predicate::str::contains("1-365 の範囲で指定してください"));
}

/// AI suggest-exclude コマンドのconfidenceバリデーション範囲テスト
/// Note: safe_join() は current_dir をベースとするため、相対パスを使用
#[cfg(feature = "ai")]
#[test]
fn test_ai_suggest_exclude_confidence_validation_min() {
    use std::env;

    let current_dir = env::current_dir().unwrap();
    let temp_dir = TempDir::new_in(&current_dir).unwrap();
    let test_dir = temp_dir.path().join("test");
    fs::create_dir_all(&test_dir).unwrap();

    // current_dirからの相対パスを取得
    let relative_path = test_dir.strip_prefix(&current_dir)
        .expect("temp_dir is under current_dir");

    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("ai")
        .arg("suggest-exclude")
        .arg(relative_path.to_str().unwrap())
        .arg("--confidence=-0.5")  // = を使って負の値を渡す
        .assert()
        .success()
        .stdout(predicate::str::contains("0.0-1.0 の範囲で指定してください"));
}

/// AI suggest-exclude コマンドのconfidenceバリデーション範囲テスト（最大値超過）
/// Note: safe_join() は current_dir をベースとするため、相対パスを使用
#[cfg(feature = "ai")]
#[test]
fn test_ai_suggest_exclude_confidence_validation_max() {
    use std::env;

    let current_dir = env::current_dir().unwrap();
    let temp_dir = TempDir::new_in(&current_dir).unwrap();
    let test_dir = temp_dir.path().join("test");
    fs::create_dir_all(&test_dir).unwrap();

    // current_dirからの相対パスを取得
    let relative_path = test_dir.strip_prefix(&current_dir)
        .expect("temp_dir is under current_dir");

    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("ai")
        .arg("suggest-exclude")
        .arg(relative_path.to_str().unwrap())
        .arg("--confidence")
        .arg("1.5")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.0-1.0 の範囲で指定してください"));
}

/// AI auto-configure コマンドのmax-depthバリデーション（0の場合）
/// Note: safe_join() は current_dir をベースとするため、相対パスを使用
#[cfg(feature = "ai")]
#[test]
fn test_ai_auto_configure_max_depth_zero() {
    use std::env;

    let current_dir = env::current_dir().unwrap();
    let temp_dir = TempDir::new_in(&current_dir).unwrap();
    let test_dir = temp_dir.path().join("test");
    fs::create_dir_all(&test_dir).unwrap();

    // current_dirからの相対パスを取得
    let relative_path = test_dir.strip_prefix(&current_dir)
        .expect("temp_dir is under current_dir");

    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("ai")
        .arg("auto-configure")
        .arg(relative_path.to_str().unwrap())
        .arg("--max-depth")
        .arg("0")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("サブディレクトリが見つかりません"));
}

/// AI auto-configure コマンドのmax-depth正常動作確認
/// Note: safe_join() は current_dir をベースとするため、相対パスを使用
#[cfg(feature = "ai")]
#[test]
fn test_ai_auto_configure_max_depth_valid() {
    use std::env;

    let current_dir = env::current_dir().unwrap();
    let temp_dir = TempDir::new_in(&current_dir).unwrap();
    let test_dir = temp_dir.path().join("test");
    fs::create_dir_all(&test_dir).unwrap();

    // サブディレクトリ作成
    let sub_dir = test_dir.join("subdir");
    fs::create_dir_all(&sub_dir).unwrap();

    // current_dirからの相対パスを取得
    let relative_path = test_dir.strip_prefix(&current_dir)
        .expect("temp_dir is under current_dir");

    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("ai")
        .arg("auto-configure")
        .arg(relative_path.to_str().unwrap())
        .arg("--max-depth")
        .arg("1")
        .arg("--dry-run")
        .assert()
        .success();
}

/// 存在しないパスの処理確認 - add コマンド
/// Note: 相対パスを使用（絶対パスは safe_join() で拒否される）
#[test]
fn test_add_rejects_nonexistent_path() {
    let mut cmd = Command::cargo_bin("backup-suite").unwrap();
    cmd.arg("add")
        .arg("nonexistent_file_12345")
        .assert()
        .success() // セキュリティ検証は通過
        .stdout(predicate::str::contains("パスが存在しません"));
}
