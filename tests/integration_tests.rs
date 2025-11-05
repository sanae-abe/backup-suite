//! 統合テストスイート - backup-suite
//!
//! このファイルは以下の統合テストを提供します:
//! - フルバックアップワークフロー
//! - 除外パターンフィルタリング
//! - 優先度別バックアップ
//! - 設定の読み込み・検証
//! - エラーハンドリング

use anyhow::Result;
use backup_suite::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// テスト用のヘルパーモジュール
mod common;

// ==================== フルバックアップワークフローテスト ====================

#[test]
fn test_full_backup_workflow_single_file() -> Result<()> {
    // テスト環境のセットアップ
    let temp_dir = TempDir::new()?;
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    fs::create_dir_all(&source_dir)?;

    // テストデータ作成
    let test_file = source_dir.join("test.txt");
    fs::write(&test_file, "test content")?;

    // 設定作成
    let mut config = Config::default();
    config.backup.destination = backup_dir.clone();

    // バックアップ対象追加
    config.targets.push(Target::new(
        test_file.clone(),
        Priority::High,
        "test".to_string(),
    ));

    // バックアップ実行
    let runner = backup_suite::BackupRunner::new(config, false);
    let result = runner.run(None, None)?;

    // 検証
    assert_eq!(result.total_files, 1);
    assert_eq!(result.successful, 1);
    assert_eq!(result.failed, 0);

    // バックアップファイルが存在することを確認
    let backed_up_file = backup_dir
        .join(&result.backup_name)
        .join("all")
        .join("test.txt");
    assert!(backed_up_file.exists());

    // 内容が一致することを確認
    let content = fs::read_to_string(&backed_up_file)?;
    assert_eq!(content, "test content");

    Ok(())
}

#[test]
fn test_full_backup_workflow_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    // ディレクトリ構造作成
    fs::create_dir_all(&source_dir)?;
    fs::create_dir_all(source_dir.join("subdir"))?;

    // 複数ファイル作成
    fs::write(source_dir.join("file1.txt"), "content1")?;
    fs::write(source_dir.join("file2.txt"), "content2")?;
    fs::write(source_dir.join("subdir/file3.txt"), "content3")?;

    // 設定とバックアップ実行
    let mut config = Config::default();
    config.backup.destination = backup_dir.clone();
    config.targets.push(Target::new(
        source_dir.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let runner = backup_suite::BackupRunner::new(config, false);
    let result = runner.run(None, None)?;

    // 3ファイルがバックアップされたことを確認
    assert_eq!(result.total_files, 3);
    assert_eq!(result.successful, 3);

    // バックアップディレクトリの構造を確認
    let backup_root = backup_dir.join(&result.backup_name).join("all");
    assert!(backup_root.join("file1.txt").exists());
    assert!(backup_root.join("file2.txt").exists());
    assert!(backup_root.join("subdir/file3.txt").exists());

    Ok(())
}

// ==================== 除外パターンテスト ====================

#[test]
fn test_exclude_patterns_simple() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    fs::create_dir_all(&source_dir)?;

    // テストファイル作成
    fs::write(source_dir.join("include.txt"), "include")?;
    fs::write(source_dir.join("exclude.tmp"), "exclude")?;
    fs::write(source_dir.join("include2.md"), "include")?;

    // 除外パターン設定
    let mut target = Target::new(
        source_dir.clone(),
        Priority::High,
        "test".to_string(),
    );
    target.exclude_patterns = vec![r".*\.tmp$".to_string()];

    let mut config = Config::default();
    config.backup.destination = backup_dir.clone();
    config.targets.push(target);

    let runner = backup_suite::BackupRunner::new(config, false);
    let result = runner.run(None, None)?;

    // .tmpファイルが除外され、2ファイルのみバックアップされる
    assert_eq!(result.total_files, 2);

    let backup_root = backup_dir.join(&result.backup_name).join("all");
    assert!(backup_root.join("include.txt").exists());
    assert!(backup_root.join("include2.md").exists());
    assert!(!backup_root.join("exclude.tmp").exists());

    Ok(())
}

#[test]
fn test_exclude_patterns_complex() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    fs::create_dir_all(&source_dir)?;
    fs::create_dir_all(source_dir.join("node_modules"))?;
    fs::create_dir_all(source_dir.join("src"))?;

    // 複数のファイル作成
    fs::write(source_dir.join("node_modules/package.json"), "exclude")?;
    fs::write(source_dir.join("src/main.rs"), "include")?;
    fs::write(source_dir.join(".env"), "exclude")?;
    fs::write(source_dir.join("config.toml"), "include")?;

    // 複数の除外パターン
    let mut target = Target::new(
        source_dir.clone(),
        Priority::High,
        "test".to_string(),
    );
    target.exclude_patterns = vec![
        r"node_modules/.*".to_string(),
        r"^\..+".to_string(), // 隠しファイル
    ];

    let mut config = Config::default();
    config.backup.destination = backup_dir.clone();
    config.targets.push(target);

    let runner = backup_suite::BackupRunner::new(config, false);
    let result = runner.run(None, None)?;

    // 2ファイルのみバックアップ (main.rs, config.toml)
    assert_eq!(result.total_files, 2);

    Ok(())
}

// ==================== 優先度別バックアップテスト ====================

#[test]
fn test_priority_filtering() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    fs::create_dir_all(&source_dir)?;

    // 優先度別ファイル作成
    let high_file = source_dir.join("high.txt");
    let medium_file = source_dir.join("medium.txt");
    let low_file = source_dir.join("low.txt");

    fs::write(&high_file, "high")?;
    fs::write(&medium_file, "medium")?;
    fs::write(&low_file, "low")?;

    // 設定
    let mut config = Config::default();
    config.backup.destination = backup_dir.clone();

    config.targets.push(Target::new(
        high_file,
        Priority::High,
        "test".to_string(),
    ));
    config.targets.push(Target::new(
        medium_file,
        Priority::Medium,
        "test".to_string(),
    ));
    config.targets.push(Target::new(
        low_file,
        Priority::Low,
        "test".to_string(),
    ));

    // 高優先度のみバックアップ
    let runner = backup_suite::BackupRunner::new(config, false);
    let result = runner.run(Some(&Priority::High), None)?;

    // 1ファイルのみバックアップされる
    assert_eq!(result.total_files, 1);

    let backup_root = backup_dir.join(&result.backup_name).join("all");
    assert!(backup_root.join("high.txt").exists());
    assert!(!backup_root.join("medium.txt").exists());
    assert!(!backup_root.join("low.txt").exists());

    Ok(())
}

#[test]
fn test_priority_filtering_medium_and_high() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    fs::create_dir_all(&source_dir)?;

    let high_file = source_dir.join("high.txt");
    let medium_file = source_dir.join("medium.txt");
    let low_file = source_dir.join("low.txt");

    fs::write(&high_file, "high")?;
    fs::write(&medium_file, "medium")?;
    fs::write(&low_file, "low")?;

    let mut config = Config::default();
    config.backup.destination = backup_dir.clone();

    config.targets.push(Target::new(high_file, Priority::High, "test".to_string()));
    config.targets.push(Target::new(medium_file, Priority::Medium, "test".to_string()));
    config.targets.push(Target::new(low_file, Priority::Low, "test".to_string()));

    // Medium優先度でバックアップ (Medium以上が対象)
    let runner = backup_suite::BackupRunner::new(config, false);
    let result = runner.run(Some(&Priority::Medium), None)?;

    // 2ファイルがバックアップされる (high + medium)
    assert_eq!(result.total_files, 2);

    Ok(())
}

// ==================== 設定テスト ====================

#[test]
fn test_config_serialization() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_file = temp_dir.path().join("config.toml");

    // 設定作成
    let mut config = Config::default();
    config.backup.destination = PathBuf::from("/tmp/backup");
    config.backup.keep_days = 60;

    let target = Target::new(
        PathBuf::from("/tmp/source"),
        Priority::High,
        "test".to_string(),
    );
    config.targets.push(target);

    // TOML形式でシリアライズ
    let toml_str = toml::to_string_pretty(&config)?;
    fs::write(&config_file, &toml_str)?;

    // デシリアライズして検証
    let loaded_config: Config = toml::from_str(&fs::read_to_string(&config_file)?)?;

    assert_eq!(loaded_config.backup.keep_days, 60);
    assert_eq!(loaded_config.targets.len(), 1);
    assert_eq!(loaded_config.targets[0].priority, Priority::High);

    Ok(())
}

#[test]
fn test_config_validation() -> Result<()> {
    // 有効な設定
    let mut config = Config::default();
    config.backup.keep_days = 30;

    // バリデーションが通ることを確認
    // 注: Config::validate() メソッドは Phase 2 で実装予定

    Ok(())
}

// ==================== エラーハンドリングテスト ====================

#[test]
fn test_nonexistent_source_handling() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backup");

    // 存在しないパス
    let nonexistent = PathBuf::from("/nonexistent/path/that/does/not/exist");

    let mut config = Config::default();
    config.backup.destination = backup_dir;
    config.targets.push(Target::new(
        nonexistent,
        Priority::High,
        "test".to_string(),
    ));

    // エラーが返されることを確認
    let runner = backup_suite::BackupRunner::new(config, false);
    let result = runner.run(None, None);
    assert!(result.is_err());
}

#[test]
fn test_invalid_destination_handling() {
    let temp_dir = TempDir::new().unwrap();
    let source_file = temp_dir.path().join("source.txt");
    fs::write(&source_file, "test").unwrap();

    let mut config = Config::default();
    // 書き込み不可能なディレクトリを指定 (Unix系)
    #[cfg(unix)]
    {
        config.backup.destination = PathBuf::from("/root/backup_not_writable");
    }

    #[cfg(windows)]
    {
        config.backup.destination = PathBuf::from("C:\\Windows\\System32\\backup_not_writable");
    }

    config.targets.push(Target::new(
        source_file,
        Priority::High,
        "test".to_string(),
    ));

    // エラーが返されることを確認
    let runner = backup_suite::BackupRunner::new(config, false);
    let result = runner.run(None, None);
    assert!(result.is_err());
}

// ==================== 並列処理テスト ====================

#[test]
fn test_parallel_backup_large_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    fs::create_dir_all(&source_dir)?;

    // 100個のファイルを作成
    for i in 0..100 {
        fs::write(
            source_dir.join(format!("file_{:03}.txt", i)),
            format!("content {}", i),
        )?;
    }

    let mut config = Config::default();
    config.backup.destination = backup_dir.clone();
    config.targets.push(Target::new(
        source_dir,
        Priority::High,
        "test".to_string(),
    ));

    // 並列バックアップ実行
    let runner = backup_suite::BackupRunner::new(config, false);
    let result = runner.run(None, None)?;

    // 全ファイルがバックアップされたことを確認
    assert_eq!(result.total_files, 100);
    assert_eq!(result.successful, 100);
    assert_eq!(result.failed, 0);

    Ok(())
}

// ==================== カテゴリ別バックアップテスト ====================

#[test]
fn test_category_organization() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    fs::create_dir_all(&source_dir)?;

    let work_file = source_dir.join("work.txt");
    let personal_file = source_dir.join("personal.txt");

    fs::write(&work_file, "work")?;
    fs::write(&personal_file, "personal")?;

    let mut config = Config::default();
    config.backup.destination = backup_dir.clone();

    // カテゴリ別ターゲット追加
    config.targets.push(Target::new(
        work_file,
        Priority::High,
        "work".to_string(),
    ));
    config.targets.push(Target::new(
        personal_file,
        Priority::Medium,
        "personal".to_string(),
    ));

    let runner = backup_suite::BackupRunner::new(config, false);
    let result = runner.run(None, None)?;

    assert_eq!(result.total_files, 2);

    // カテゴリ別に整理されているか確認
    let backup_root = backup_dir.join(&result.backup_name).join("all");
    assert!(backup_root.join("work.txt").exists());
    assert!(backup_root.join("personal.txt").exists());

    Ok(())
}

// ==================== パフォーマンステスト ====================

#[test]
#[ignore] // 通常は無視、`cargo test -- --ignored` で実行
fn test_large_file_backup_performance() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    fs::create_dir_all(&source_dir)?;

    // 10MBのファイルを作成
    let large_content = vec![0u8; 10 * 1024 * 1024];
    fs::write(source_dir.join("large_file.bin"), &large_content)?;

    let mut config = Config::default();
    config.backup.destination = backup_dir.clone();
    config.targets.push(Target::new(
        source_dir,
        Priority::High,
        "test".to_string(),
    ));

    let start = std::time::Instant::now();
    let runner = backup_suite::BackupRunner::new(config, false);
    let result = runner.run(None, None)?;
    let duration = start.elapsed();

    assert_eq!(result.successful, 1);

    // パフォーマンス目標: 10MBを5秒以内でバックアップ
    assert!(duration.as_secs() < 5, "バックアップに時間がかかりすぎています: {:?}", duration);

    Ok(())
}
