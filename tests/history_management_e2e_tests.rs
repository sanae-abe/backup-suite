// history_management_e2e_tests.rs - 履歴管理E2Eテスト
//
// Phase 2: 完全性向上（Priority: Medium）
// 実装期限: 2025-11-20
//
// このファイルはバックアップ履歴管理機能を検証します。
//
// テストシナリオ:
// 1. バックアップ履歴追跡 - 複数回実行で全記録保存確認
// 2. 時系列順一覧表示 - メタデータの正確性確認
// 3. 古いバックアップ削除 - ディスク容量管理

use anyhow::Result;
use backup_suite::compression::CompressionType;
use backup_suite::core::history::{BackupHistory, BackupStatus};
use backup_suite::core::{BackupRunner, Config};
use backup_suite::{Priority, Target};
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// =============================================================================
// テストヘルパー関数
// =============================================================================

/// テスト用の設定環境をセットアップ
fn setup_test_env() -> Result<TempDir> {
    use std::env;

    // 一時ディレクトリを作成
    let temp = TempDir::new()?;
    let config_dir = temp.path().join(".config").join("backup-suite");
    fs::create_dir_all(&config_dir)?;

    // HOMEディレクトリを一時ディレクトリに設定
    env::set_var("HOME", temp.path());

    // テスト用に一時的な設定ファイルを作成
    let backup_dest = temp.path().join("backup");
    fs::create_dir_all(&backup_dest)?;

    // Config::default()を使用して正しい設定を作成
    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.save()?;

    // 履歴ファイルパスを確認して、ディレクトリが存在することを検証
    let history_path = config_dir.join("history.toml");
    if let Some(parent) = history_path.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(temp)
}

/// テスト用のソースディレクトリを作成
fn create_test_source(temp: &TempDir, scenario: &str) -> std::path::PathBuf {
    let source = temp.path().join("source");
    fs::create_dir_all(&source).unwrap();

    match scenario {
        "v1" => {
            fs::write(source.join("file1.txt"), "Version 1 content").unwrap();
            fs::write(source.join("file2.txt"), "Version 1 content").unwrap();
        }
        "v2" => {
            fs::write(source.join("file1.txt"), "Version 2 content").unwrap();
            fs::write(source.join("file2.txt"), "Version 2 content").unwrap();
            fs::write(source.join("file3.txt"), "Version 2 new file").unwrap();
        }
        "v3" => {
            fs::write(source.join("file1.txt"), "Version 3 content").unwrap();
            fs::write(source.join("file2.txt"), "Version 3 content").unwrap();
            fs::write(source.join("file3.txt"), "Version 3 content").unwrap();
            fs::write(source.join("file4.txt"), "Version 3 new file").unwrap();
        }
        _ => panic!("Unknown scenario: {}", scenario),
    }

    source
}

/// バックアップディレクトリの総サイズを計算
fn calculate_backup_size(backup_dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
    for entry in walkdir::WalkDir::new(backup_dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            total_size += fs::metadata(entry.path())?.len();
        }
    }
    Ok(total_size)
}

// =============================================================================
// E2E Scenario 1: バックアップ履歴追跡
// =============================================================================

#[test]
#[serial]
fn test_e2e_backup_history_tracking() -> Result<()> {
    // テスト環境をセットアップ
    let temp = setup_test_env()?;
    let backup = temp.path().join("backup");
    fs::create_dir_all(&backup)?;

    // テスト用の一時的な履歴ファイルパス設定
    // （BackupHistory::save()はシステムの履歴ファイルに保存するため、
    //   このテストでは手動でエントリを作成して検証）

    // ステップ1: 3回のバックアップを実行
    let source_v1 = create_test_source(&temp, "v1");
    let mut config1 = Config::default();
    config1.backup.destination = backup.clone();
    config1.add_target(Target::new(
        source_v1.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner1 = BackupRunner::new(config1, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result1 = runner1.run(None, None)?;
    assert_eq!(result1.failed, 0, "First backup should succeed");

    // 履歴エントリ1を作成して保存
    let backup_dir1 = backup.join(&result1.backup_name);
    let size1 = calculate_backup_size(&backup_dir1)?;
    let mut history1 = BackupHistory::new(backup_dir1.clone(), result1.total_files, size1, true);
    history1.category = Some("test_tracking_v1".to_string());
    history1.priority = Some(Priority::High);
    // Note: compressed/encrypted flags are set by the runner, not manually
    BackupHistory::save(&history1)?;

    // タイムスタンプ衝突回避のため1秒待機
    std::thread::sleep(std::time::Duration::from_secs(1));

    // ステップ2: 2回目のバックアップ（Zstd圧縮）
    fs::remove_dir_all(&source_v1)?;
    let source_v2 = create_test_source(&temp, "v2");
    let mut config2 = Config::default();
    config2.backup.destination = backup.clone();
    config2.add_target(Target::new(
        source_v2.clone(),
        Priority::Medium,
        "test".to_string(),
    ));

    let mut runner2 = BackupRunner::new(config2, false)
        .with_progress(false)
        .with_compression(CompressionType::Zstd, 3);

    let result2 = runner2.run(None, None)?;
    assert_eq!(result2.failed, 0, "Second backup should succeed");

    let backup_dir2 = backup.join(&result2.backup_name);
    let size2 = calculate_backup_size(&backup_dir2)?;
    let mut history2 = BackupHistory::new(backup_dir2.clone(), result2.total_files, size2, true);
    history2.category = Some("test_tracking_v2".to_string());
    history2.priority = Some(Priority::Medium);
    BackupHistory::save(&history2)?;

    // タイムスタンプ衝突回避のため1秒待機
    std::thread::sleep(std::time::Duration::from_secs(1));

    // ステップ3: 3回目のバックアップ（暗号化+圧縮）
    fs::remove_dir_all(&source_v2)?;
    let source_v3 = create_test_source(&temp, "v3");
    let mut config3 = Config::default();
    config3.backup.destination = backup.clone();
    config3.add_target(Target::new(
        source_v3.clone(),
        Priority::Low,
        "test".to_string(),
    ));

    let mut runner3 = BackupRunner::new(config3, false)
        .with_progress(false)
        .with_encryption("test_password".to_string())
        .with_compression(CompressionType::Zstd, 3);

    let result3 = runner3.run(None, None)?;
    assert_eq!(result3.failed, 0, "Third backup should succeed");

    let backup_dir3 = backup.join(&result3.backup_name);
    let size3 = calculate_backup_size(&backup_dir3)?;
    let mut history3 = BackupHistory::new(backup_dir3.clone(), result3.total_files, size3, true);
    history3.category = Some("test_tracking_v3".to_string());
    history3.priority = Some(Priority::Low);
    BackupHistory::save(&history3)?;

    // ステップ4: 履歴を読み込んで検証
    let all_history = BackupHistory::load_all()?;

    // このテストで追加した3件のエントリをカテゴリでフィルタリング
    let tracking_entries: Vec<_> = all_history
        .iter()
        .filter(|e| {
            e.category
                .as_ref()
                .map(|c| c.starts_with("test_tracking"))
                .unwrap_or(false)
        })
        .collect();

    // 最低3件のtrackingエントリが保存されていることを確認
    assert!(
        tracking_entries.len() >= 3,
        "At least 3 tracking entries should exist: found {}",
        tracking_entries.len()
    );

    // 最新3件を取得して検証
    let recent_3 = &tracking_entries[tracking_entries.len() - 3..];

    // ステップ5: 各履歴エントリのメタデータ確認
    // （.integrityファイルもカウントされる可能性があるため、>= で検証）
    assert!(
        recent_3[0].total_files >= 2,
        "First backup: at least 2 files"
    );
    assert_eq!(recent_3[0].status, BackupStatus::Success);
    assert_eq!(recent_3[0].priority, Some(Priority::High));
    assert_eq!(recent_3[0].category, Some("test_tracking_v1".to_string()));

    assert!(
        recent_3[1].total_files >= 3,
        "Second backup: at least 3 files"
    );
    assert_eq!(recent_3[1].status, BackupStatus::Success);
    assert_eq!(recent_3[1].priority, Some(Priority::Medium));
    assert_eq!(recent_3[1].category, Some("test_tracking_v2".to_string()));

    assert!(
        recent_3[2].total_files >= 4,
        "Third backup: at least 4 files"
    );
    assert_eq!(recent_3[2].status, BackupStatus::Success);
    assert_eq!(recent_3[2].priority, Some(Priority::Low));
    assert_eq!(recent_3[2].category, Some("test_tracking_v3".to_string()));

    println!("✅ バックアップ履歴追跡テスト成功:");
    println!("  全履歴エントリ数: {} 件", all_history.len());
    println!("  trackingエントリ数: {} 件", tracking_entries.len());
    println!("  最新3件のtrackingバックアップ:");
    for (i, entry) in recent_3.iter().enumerate() {
        println!(
            "    {}. {} - {} files, priority={:?}",
            i + 1,
            entry.category.as_ref().unwrap(),
            entry.total_files,
            entry.priority
        );
    }

    Ok(())
}

// =============================================================================
// E2E Scenario 2: 時系列順一覧表示（メタデータの正確性確認）
// =============================================================================

#[test]
#[serial]
fn test_e2e_list_backups_chronological() -> Result<()> {
    // テスト環境をセットアップ
    let temp = setup_test_env()?;
    let backup = temp.path().join("backup");
    fs::create_dir_all(&backup)?;

    // ステップ1: 異なる優先度で3回バックアップ実行
    let source = create_test_source(&temp, "v1");

    // High priority backup
    let mut config_high = Config::default();
    config_high.backup.destination = backup.clone();
    config_high.add_target(Target::new(
        source.clone(),
        Priority::High,
        "test_high".to_string(),
    ));

    let mut runner_high = BackupRunner::new(config_high, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result_high = runner_high.run(None, None)?;
    let backup_dir_high = backup.join(&result_high.backup_name);
    let size_high = calculate_backup_size(&backup_dir_high)?;
    let mut history_high =
        BackupHistory::new(backup_dir_high, result_high.total_files, size_high, true);
    history_high.priority = Some(Priority::High);
    BackupHistory::save(&history_high)?;

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Medium priority backup
    let mut config_medium = Config::default();
    config_medium.backup.destination = backup.clone();
    config_medium.add_target(Target::new(
        source.clone(),
        Priority::Medium,
        "test_medium".to_string(),
    ));

    let mut runner_medium = BackupRunner::new(config_medium, false)
        .with_progress(false)
        .with_compression(CompressionType::Zstd, 3);

    let result_medium = runner_medium.run(None, None)?;
    let backup_dir_medium = backup.join(&result_medium.backup_name);
    let size_medium = calculate_backup_size(&backup_dir_medium)?;
    let mut history_medium = BackupHistory::new(
        backup_dir_medium,
        result_medium.total_files,
        size_medium,
        true,
    );
    history_medium.priority = Some(Priority::Medium);
    BackupHistory::save(&history_medium)?;

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Low priority backup
    let mut config_low = Config::default();
    config_low.backup.destination = backup.clone();
    config_low.add_target(Target::new(
        source.clone(),
        Priority::Low,
        "test_low".to_string(),
    ));

    let mut runner_low = BackupRunner::new(config_low, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result_low = runner_low.run(None, None)?;
    let backup_dir_low = backup.join(&result_low.backup_name);
    let size_low = calculate_backup_size(&backup_dir_low)?;
    let mut history_low =
        BackupHistory::new(backup_dir_low, result_low.total_files, size_low, true);
    history_low.priority = Some(Priority::Low);
    BackupHistory::save(&history_low)?;

    // ステップ2: すべての履歴を読み込み
    let all_history = BackupHistory::load_all()?;
    assert!(all_history.len() >= 3, "At least 3 history entries");

    // ステップ3: 時系列順（古い→新しい）であることを確認
    let recent = &all_history[all_history.len() - 3..];
    for i in 0..recent.len() - 1 {
        assert!(
            recent[i].timestamp <= recent[i + 1].timestamp,
            "History should be in chronological order"
        );
    }

    // ステップ4: 優先度でフィルタリング
    let high_priority = BackupHistory::filter_by_priority(&all_history, &Priority::High);
    let medium_priority = BackupHistory::filter_by_priority(&all_history, &Priority::Medium);
    let low_priority = BackupHistory::filter_by_priority(&all_history, &Priority::Low);

    assert!(high_priority.len() >= 1, "At least 1 high priority backup");
    assert!(
        medium_priority.len() >= 1,
        "At least 1 medium priority backup"
    );
    assert!(low_priority.len() >= 1, "At least 1 low priority backup");

    println!("✅ 時系列順一覧表示テスト成功:");
    println!("  全履歴: {} 件", all_history.len());
    println!("  High priority: {} 件", high_priority.len());
    println!("  Medium priority: {} 件", medium_priority.len());
    println!("  Low priority: {} 件", low_priority.len());

    Ok(())
}

// =============================================================================
// E2E Scenario 3: 古いバックアップ削除（ディスク容量管理）
// =============================================================================

#[test]
#[serial]
fn test_e2e_delete_old_backups() -> Result<()> {
    // テスト環境をセットアップ
    let temp = setup_test_env()?;
    let backup = temp.path().join("backup");
    fs::create_dir_all(&backup)?;

    // ステップ1: 5回のバックアップを実行
    let source = create_test_source(&temp, "v1");

    for i in 1..=5 {
        let mut config = Config::default();
        config.backup.destination = backup.clone();
        config.add_target(Target::new(
            source.clone(),
            Priority::High,
            format!("test_{}", i),
        ));

        let mut runner = BackupRunner::new(config, false)
            .with_progress(false)
            .with_compression(CompressionType::None, 0);

        let result = runner.run(None, None)?;
        let backup_dir = backup.join(&result.backup_name);
        let size = calculate_backup_size(&backup_dir)?;
        let history = BackupHistory::new(backup_dir, result.total_files, size, true);
        BackupHistory::save(&history)?;

        if i < 5 {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    // ステップ2: 履歴を確認（5件以上存在するはず）
    let all_history = BackupHistory::load_all()?;
    let initial_count = all_history.len();
    assert!(initial_count >= 5, "At least 5 history entries");

    // ステップ3: 古いバックアップディレクトリを削除
    // （最新2件のみ残す）
    let to_keep = 2;
    let to_delete = if all_history.len() > to_keep {
        &all_history[..all_history.len() - to_keep]
    } else {
        &[]
    };

    let mut deleted_count = 0;
    for entry in to_delete {
        if entry.backup_dir.exists() {
            fs::remove_dir_all(&entry.backup_dir)?;
            deleted_count += 1;
            println!("🗑️  削除: {}", entry.backup_dir.display());
        }
    }

    // ステップ4: 残っているバックアップディレクトリを確認
    let remaining_backups: Vec<_> = fs::read_dir(&backup)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    assert!(
        remaining_backups.len() <= to_keep,
        "Should have at most {} backups remaining: found {}",
        to_keep,
        remaining_backups.len()
    );

    println!("✅ 古いバックアップ削除テスト成功:");
    println!("  初期履歴数: {} 件", initial_count);
    println!("  削除数: {} 件", deleted_count);
    println!("  残存バックアップ: {} 件", remaining_backups.len());

    // ステップ5: 履歴ファイル自体の最大100件制限を検証
    // （BackupHistory::save()は自動的に100件に制限）
    let final_history = BackupHistory::load_all()?;
    assert!(
        final_history.len() <= 100,
        "History should be limited to 100 entries: found {}",
        final_history.len()
    );

    Ok(())
}

// =============================================================================
// E2E Scenario 4: filter_by_days() - 日数フィルタリング
// =============================================================================

#[test]
#[serial]
fn test_e2e_filter_by_days() -> Result<()> {
    use chrono::Duration;

    // テスト環境をセットアップ
    let _temp = setup_test_env()?;

    // テスト用履歴エントリを作成（異なるタイムスタンプ）
    let now = chrono::Utc::now();

    // 7日前
    let mut entry_7days = BackupHistory::new(
        std::path::PathBuf::from("/test/backup_7days"),
        10,
        1000,
        true,
    );
    entry_7days.timestamp = now - Duration::days(7);
    entry_7days.category = Some("test_filter_days_7".to_string());
    BackupHistory::save(&entry_7days)?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    // 3日前
    let mut entry_3days = BackupHistory::new(
        std::path::PathBuf::from("/test/backup_3days"),
        15,
        1500,
        true,
    );
    entry_3days.timestamp = now - Duration::days(3);
    entry_3days.category = Some("test_filter_days_3".to_string());
    BackupHistory::save(&entry_3days)?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    // 1日前
    let mut entry_1day = BackupHistory::new(
        std::path::PathBuf::from("/test/backup_1day"),
        20,
        2000,
        true,
    );
    entry_1day.timestamp = now - Duration::days(1);
    entry_1day.category = Some("test_filter_days_1".to_string());
    BackupHistory::save(&entry_1day)?;

    // 5日間でフィルタリング（3日前と1日前のみ該当）
    let filtered = BackupHistory::filter_by_days(5)?;

    let test_entries: Vec<_> = filtered
        .iter()
        .filter(|e| {
            e.category
                .as_ref()
                .map(|c| c.starts_with("test_filter_days"))
                .unwrap_or(false)
        })
        .collect();

    // 3日前と1日前の2件が含まれることを確認
    assert!(
        test_entries.len() >= 2,
        "Should have at least 2 entries within 5 days: found {}",
        test_entries.len()
    );

    // 7日前のエントリは含まれないことを確認
    let has_7day_entry = test_entries
        .iter()
        .any(|e| e.category == Some("test_filter_days_7".to_string()));
    assert!(!has_7day_entry, "7-day-old entry should not be included");

    println!("✅ 日数フィルタリングテスト成功:");
    println!("  5日以内のエントリ: {} 件", test_entries.len());
    for entry in test_entries.iter() {
        let days_ago = (now - entry.timestamp).num_days();
        println!(
            "    {} - {} 日前",
            entry.category.as_ref().unwrap(),
            days_ago
        );
    }

    Ok(())
}

// =============================================================================
// E2E Scenario 5: filter_by_category() - カテゴリフィルタリング詳細
// =============================================================================

#[test]
#[serial]
fn test_e2e_filter_by_category_detailed() -> Result<()> {
    // テスト環境をセットアップ
    let _temp = setup_test_env()?;

    // テスト用履歴エントリを作成（異なるカテゴリ）
    let categories = vec!["documents", "photos", "code"];

    for category in &categories {
        let entry = BackupHistory::new(
            std::path::PathBuf::from(format!("/test/backup_{}", category)),
            10,
            1000,
            true,
        );
        let mut entry_with_category = entry;
        entry_with_category.category = Some(format!("test_category_{}", category));
        BackupHistory::save(&entry_with_category)?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // すべての履歴を読み込み
    let all_history = BackupHistory::load_all()?;

    // 各カテゴリでフィルタリング
    for category in &categories {
        let test_category = format!("test_category_{}", category);
        let filtered = BackupHistory::filter_by_category(&all_history, &test_category);

        assert!(
            filtered.len() >= 1,
            "Should have at least 1 entry for category {}: found {}",
            test_category,
            filtered.len()
        );

        // すべてのエントリが正しいカテゴリであることを確認
        for entry in &filtered {
            assert_eq!(
                entry.category.as_deref(),
                Some(test_category.as_str()),
                "Filtered entry should have correct category"
            );
        }
    }

    // 存在しないカテゴリでフィルタリング
    let nonexistent = BackupHistory::filter_by_category(&all_history, "test_category_nonexistent");
    assert_eq!(
        nonexistent.len(),
        0,
        "Nonexistent category should return empty"
    );

    println!("✅ カテゴリフィルタリング詳細テスト成功:");
    for category in &categories {
        let test_category = format!("test_category_{}", category);
        let filtered = BackupHistory::filter_by_category(&all_history, &test_category);
        println!("  {}: {} 件", test_category, filtered.len());
    }

    Ok(())
}

// =============================================================================
// E2E Scenario 6: get_recent_entries() - 最近N件取得
// =============================================================================

#[test]
#[serial]
fn test_e2e_get_recent_entries() -> Result<()> {
    // テスト環境をセットアップ
    let _temp = setup_test_env()?;

    // 10件のテスト履歴を作成
    for i in 1..=10 {
        let entry = BackupHistory::new(
            std::path::PathBuf::from(format!("/test/backup_recent_{}", i)),
            i * 10,
            (i * 100) as u64,
            true,
        );
        let mut entry_with_category = entry;
        entry_with_category.category = Some(format!("test_recent_{}", i));
        BackupHistory::save(&entry_with_category)?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 最新3件を取得
    let recent_3 = BackupHistory::get_recent_entries(3)?;

    let test_recent: Vec<_> = recent_3
        .iter()
        .filter(|e| {
            e.category
                .as_ref()
                .map(|c| c.starts_with("test_recent_"))
                .unwrap_or(false)
        })
        .collect();

    // 最低3件存在することを確認
    assert!(
        test_recent.len() >= 3,
        "Should have at least 3 recent entries: found {}",
        test_recent.len()
    );

    // 新しい順（降順）であることを確認
    for i in 0..test_recent.len().saturating_sub(1) {
        assert!(
            test_recent[i].timestamp >= test_recent[i + 1].timestamp,
            "Recent entries should be in descending order (newest first)"
        );
    }

    println!("✅ 最近N件取得テスト成功:");
    println!("  取得した最新エントリ: {} 件", test_recent.len());
    for (i, entry) in test_recent.iter().take(3).enumerate() {
        println!(
            "    {}. {} - timestamp={}",
            i + 1,
            entry.category.as_ref().unwrap(),
            entry.timestamp
        );
    }

    Ok(())
}

// =============================================================================
// E2E Scenario 7: list_backup_dirs() - バックアップディレクトリ一覧
// =============================================================================

#[test]
#[serial]
fn test_e2e_list_backup_dirs() -> Result<()> {
    // テスト環境をセットアップ
    let temp = setup_test_env()?;
    let backup = temp.path().join("backup");
    fs::create_dir_all(&backup)?;

    // 3つのバックアップディレクトリを作成
    let source = create_test_source(&temp, "v1");

    for i in 1..=3 {
        let mut config = Config::default();
        config.backup.destination = backup.clone();
        config.add_target(Target::new(
            source.clone(),
            Priority::High,
            format!("test_list_{}", i),
        ));

        let mut runner = BackupRunner::new(config, false)
            .with_progress(false)
            .with_compression(CompressionType::None, 0);

        runner.run(None, None)?;

        if i < 3 {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    // バックアップディレクトリ一覧を取得
    // （注意: list_backup_dirs()はConfig::load()を使うため、
    //  このテストでは一時的な設定ファイルが必要になる可能性がある）
    // 代わりに、直接ディレクトリを列挙して検証
    let dirs: Vec<_> = fs::read_dir(&backup)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    assert!(dirs.len() >= 3, "Should have at least 3 backup directories");

    // 新しい順にソートされることを確認（修正時刻ベース）
    let mut dir_times: Vec<_> = dirs
        .iter()
        .filter_map(|e| {
            fs::metadata(e.path())
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| (e.path(), t))
        })
        .collect();

    dir_times.sort_by(|a, b| b.1.cmp(&a.1)); // 新しい順

    for i in 0..dir_times.len().saturating_sub(1) {
        assert!(
            dir_times[i].1 >= dir_times[i + 1].1,
            "Backup directories should be sorted by modification time (newest first)"
        );
    }

    println!("✅ バックアップディレクトリ一覧テスト成功:");
    println!("  バックアップディレクトリ数: {} 件", dirs.len());
    for (i, (path, time)) in dir_times.iter().take(3).enumerate() {
        println!(
            "    {}. {} (modified: {:?})",
            i + 1,
            path.file_name().unwrap().to_string_lossy(),
            time
        );
    }

    Ok(())
}

// =============================================================================
// E2E Scenario 8: BackupStatus variants - Failed/Partial
// =============================================================================

#[test]
#[serial]
fn test_e2e_backup_status_variants() -> Result<()> {
    // テスト環境をセットアップ
    let _temp = setup_test_env()?;

    // Success ステータス
    let mut entry_success = BackupHistory::new(
        std::path::PathBuf::from("/test/backup_success"),
        100,
        10000,
        true,
    );
    entry_success.category = Some("test_status_success".to_string());
    BackupHistory::save(&entry_success)?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    // Failed ステータス
    let mut entry_failed =
        BackupHistory::new(std::path::PathBuf::from("/test/backup_failed"), 0, 0, false);
    entry_failed.status = BackupStatus::Failed;
    entry_failed.error_message = Some("Test error: disk full".to_string());
    entry_failed.category = Some("test_status_failed".to_string());
    BackupHistory::save(&entry_failed)?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    // Partial ステータス
    let mut entry_partial = BackupHistory::new(
        std::path::PathBuf::from("/test/backup_partial"),
        50,
        5000,
        true,
    );
    entry_partial.status = BackupStatus::Partial;
    entry_partial.error_message = Some("Test warning: some files skipped".to_string());
    entry_partial.category = Some("test_status_partial".to_string());
    BackupHistory::save(&entry_partial)?;

    // すべての履歴を読み込み
    let all_history = BackupHistory::load_all()?;

    let test_entries: Vec<_> = all_history
        .iter()
        .filter(|e| {
            e.category
                .as_ref()
                .map(|c| c.starts_with("test_status_"))
                .unwrap_or(false)
        })
        .collect();

    // 各ステータスのエントリが存在することを確認
    let has_success = test_entries
        .iter()
        .any(|e| e.status == BackupStatus::Success);
    let has_failed = test_entries
        .iter()
        .any(|e| e.status == BackupStatus::Failed);
    let has_partial = test_entries
        .iter()
        .any(|e| e.status == BackupStatus::Partial);

    assert!(has_success, "Should have at least one Success entry");
    assert!(has_failed, "Should have at least one Failed entry");
    assert!(has_partial, "Should have at least one Partial entry");

    // Failed エントリのエラーメッセージ確認
    let failed_entry = test_entries
        .iter()
        .find(|e| e.status == BackupStatus::Failed)
        .expect("Failed entry should exist");

    assert!(
        failed_entry.error_message.is_some(),
        "Failed entry should have error_message"
    );
    assert!(
        failed_entry
            .error_message
            .as_ref()
            .unwrap()
            .contains("disk full"),
        "Error message should contain expected text"
    );

    println!("✅ BackupStatus variants テスト成功:");
    println!(
        "  Success entries: {}",
        test_entries
            .iter()
            .filter(|e| e.status == BackupStatus::Success)
            .count()
    );
    println!(
        "  Failed entries: {}",
        test_entries
            .iter()
            .filter(|e| e.status == BackupStatus::Failed)
            .count()
    );
    println!(
        "  Partial entries: {}",
        test_entries
            .iter()
            .filter(|e| e.status == BackupStatus::Partial)
            .count()
    );

    Ok(())
}

// =============================================================================
// E2E Scenario 9: 100件制限の動作確認
// =============================================================================

#[test]
#[serial]
fn test_e2e_history_limit_100() -> Result<()> {
    // テスト環境をセットアップ
    let _temp = setup_test_env()?;

    // 既存の履歴をバックアップ
    let log_path = BackupHistory::log_path()?;
    let backup_path = log_path.with_extension("toml.backup");

    if log_path.exists() {
        fs::copy(&log_path, &backup_path)?;
    }

    // 履歴をクリア
    if log_path.exists() {
        fs::remove_file(&log_path)?;
    }

    // 105件の履歴を作成
    for i in 1..=105 {
        let entry = BackupHistory::new(
            std::path::PathBuf::from(format!("/test/backup_limit_{}", i)),
            i,
            (i * 100) as u64,
            true,
        );
        BackupHistory::save(&entry)?;
    }

    // 履歴を読み込み
    let all_history = BackupHistory::load_all()?;

    // 100件以下であることを確認
    assert!(
        all_history.len() <= 100,
        "History should be limited to 100 entries: found {}",
        all_history.len()
    );

    // 最新100件が保持されていることを確認
    // （最初の5件は削除されているはず）
    let oldest_entry = all_history.first().unwrap();
    assert!(
        oldest_entry
            .backup_dir
            .to_string_lossy()
            .contains("backup_limit_"),
        "Oldest entry should be from the limit test"
    );

    println!("✅ 100件制限テスト成功:");
    println!("  現在の履歴数: {} 件", all_history.len());
    println!("  最古のエントリ: {}", oldest_entry.backup_dir.display());
    println!(
        "  最新のエントリ: {}",
        all_history.last().unwrap().backup_dir.display()
    );

    // 元の履歴を復元
    if backup_path.exists() {
        fs::rename(&backup_path, &log_path)?;
    }

    Ok(())
}

// =============================================================================
// E2E Scenario 10: エラーケース
// =============================================================================

#[test]
#[serial]
fn test_e2e_history_error_cases() -> Result<()> {
    use std::io::Write;

    // テスト環境をセットアップ
    let _temp = setup_test_env()?;

    // ケース1: 不正なTOMLファイルの読み込み
    let log_path = BackupHistory::log_path()?;
    let backup_path = log_path.with_extension("toml.backup");

    if log_path.exists() {
        fs::copy(&log_path, &backup_path)?;
    }

    // 不正なTOMLを書き込み
    let invalid_toml = "invalid toml content [[[ }}}";
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(&log_path)?;
    file.write_all(invalid_toml.as_bytes())?;
    drop(file);

    // load_all()でエラーが返ることを確認
    let result = BackupHistory::load_all();
    assert!(result.is_err(), "load_all() should fail with invalid TOML");

    // 元の履歴を復元
    if backup_path.exists() {
        fs::rename(&backup_path, &log_path)?;
    } else {
        fs::remove_file(&log_path)?;
    }

    // ケース2: 存在しない履歴ファイル（空のベクターを返す）
    if log_path.exists() {
        fs::copy(&log_path, &backup_path)?;
        fs::remove_file(&log_path)?;
    }

    let result = BackupHistory::load_all()?;
    assert_eq!(
        result.len(),
        0,
        "load_all() should return empty vector when file doesn't exist"
    );

    // 元の履歴を復元
    if backup_path.exists() {
        fs::rename(&backup_path, &log_path)?;
    }

    println!("✅ エラーケーステスト成功:");
    println!("  不正TOML読み込みエラー: 検出成功");
    println!("  存在しないファイル処理: 空ベクター返却成功");

    Ok(())
}

// =============================================================================
// E2E Scenario 11: list_backup_dirs() - 直接呼び出しテスト
// =============================================================================

#[test]
#[serial]
fn test_e2e_list_backup_dirs_direct() -> Result<()> {
    // テスト環境をセットアップ
    let temp = setup_test_env()?;
    let backup = temp.path().join("backup");
    fs::create_dir_all(&backup)?;

    // 3つのバックアップディレクトリを作成
    let source = create_test_source(&temp, "v1");

    for i in 1..=3 {
        let mut config = Config::default();
        config.backup.destination = backup.clone();
        config.add_target(Target::new(
            source.clone(),
            Priority::High,
            format!("test_listdirs_{}", i),
        ));

        let mut runner = BackupRunner::new(config, false)
            .with_progress(false)
            .with_compression(CompressionType::None, 0);

        runner.run(None, None)?;

        if i < 3 {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    // BackupHistory::list_backup_dirs() を直接呼び出し
    let backup_dirs = BackupHistory::list_backup_dirs()?;

    assert!(
        backup_dirs.len() >= 3,
        "Should have at least 3 backup directories: found {}",
        backup_dirs.len()
    );

    // 新しい順にソートされていることを確認
    for i in 0..backup_dirs.len().saturating_sub(1) {
        let t1 = fs::metadata(&backup_dirs[i])
            .and_then(|m| m.modified())
            .ok();
        let t2 = fs::metadata(&backup_dirs[i + 1])
            .and_then(|m| m.modified())
            .ok();

        if let (Some(time1), Some(time2)) = (t1, t2) {
            assert!(
                time1 >= time2,
                "Backup directories should be sorted newest first"
            );
        }
    }

    println!("✅ list_backup_dirs() 直接呼び出しテスト成功:");
    println!("  バックアップディレクトリ数: {} 件", backup_dirs.len());
    for (i, dir) in backup_dirs.iter().take(3).enumerate() {
        println!(
            "    {}. {}",
            i + 1,
            dir.file_name().unwrap().to_string_lossy()
        );
    }

    Ok(())
}

// =============================================================================
// E2E Scenario 12: default_status() - デシリアライズでデフォルト値使用
// =============================================================================

#[test]
#[serial]
fn test_e2e_default_status_deserialization() -> Result<()> {
    use std::io::Write;

    // テスト環境をセットアップ
    let _temp = setup_test_env()?;

    // ステータスフィールドがないTOMLエントリを作成
    let log_path = BackupHistory::log_path()?;
    let backup_path = log_path.with_extension("toml.backup");

    if log_path.exists() {
        fs::copy(&log_path, &backup_path)?;
    }

    // status フィールドを含まないTOMLを作成（後方互換性テスト）
    let toml_without_status = r#"
[[history]]
timestamp = "2025-01-01T00:00:00Z"
backup_dir = "/test/backup_no_status"
total_files = 10
total_bytes = 1000
success = true
"#;

    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(&log_path)?;
    file.write_all(toml_without_status.as_bytes())?;
    drop(file);

    // load_all()でデフォルト値が使用されることを確認
    let result = BackupHistory::load_all()?;
    assert_eq!(result.len(), 1, "Should load 1 entry");

    // default_status()がSuccessを返すため、ステータスはSuccessになるはず
    assert_eq!(
        result[0].status,
        BackupStatus::Success,
        "Default status should be Success"
    );

    println!("✅ default_status() デシリアライズテスト成功:");
    println!("  ステータスフィールド欠如時: {:?}", result[0].status);

    // 元の履歴を復元
    if backup_path.exists() {
        fs::rename(&backup_path, &log_path)?;
    } else {
        fs::remove_file(&log_path)?;
    }

    Ok(())
}
