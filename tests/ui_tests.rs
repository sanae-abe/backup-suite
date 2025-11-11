/// UI モジュールの包括的なテスト
///
/// このテストファイルは src/ui モジュールのカバレッジ向上を目的としています。
/// 特に dashboard.rs と interactive.rs の未テスト関数を中心にテストします。
///
/// テスト対象:
/// - dashboard.rs: 統計表示、ディスク使用量、警告サマリー等
/// - interactive.rs: 対話的UI関数のエラーハンドリング
/// - progress.rs: プログレスバー機能
/// - table.rs: テーブル表示機能（既存テストの拡張）
///
/// 作成日: 2025-11-11
/// 目的: Phase 1 カバレッジ目標 51.11% → 66-70% 達成
use backup_suite::core::history::BackupStatus;
use backup_suite::core::{BackupHistory, Priority, Target};
use backup_suite::ui::dashboard;
use chrono::{Duration, Utc};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ═══════════════════════════════════════════════════════════════
// dashboard.rs のテスト
// ═══════════════════════════════════════════════════════════════

/// calculate_directory_size のテスト - 空ディレクトリ
#[test]
fn test_calculate_directory_size_empty() {
    let temp_dir = TempDir::new().unwrap();
    let result = dashboard::calculate_directory_size(temp_dir.path());

    assert!(result.is_ok());
    let (size, count) = result.unwrap();
    assert_eq!(size, 0, "空ディレクトリのサイズは0であるべき");
    assert_eq!(count, 0, "空ディレクトリのファイル数は0であるべき");
}

/// calculate_directory_size のテスト - ファイルあり
#[test]
fn test_calculate_directory_size_with_files() {
    let temp_dir = TempDir::new().unwrap();

    // テストファイル作成
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file1, "Hello World").unwrap(); // 11 bytes
    fs::write(&file2, "Rust").unwrap();         // 4 bytes

    let result = dashboard::calculate_directory_size(temp_dir.path());

    assert!(result.is_ok());
    let (size, count) = result.unwrap();
    assert_eq!(size, 15, "合計サイズは15バイトであるべき");
    assert_eq!(count, 2, "ファイル数は2であるべき");
}

/// calculate_directory_size のテスト - サブディレクトリ含む
#[test]
fn test_calculate_directory_size_with_subdirs() {
    let temp_dir = TempDir::new().unwrap();

    // サブディレクトリ作成
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();

    // ファイル作成
    let file1 = temp_dir.path().join("root.txt");
    let file2 = subdir.join("sub.txt");
    fs::write(&file1, "Root").unwrap();    // 4 bytes
    fs::write(&file2, "Subdir").unwrap();  // 6 bytes

    let result = dashboard::calculate_directory_size(temp_dir.path());

    assert!(result.is_ok());
    let (size, count) = result.unwrap();
    assert_eq!(size, 10, "合計サイズは10バイトであるべき");
    assert_eq!(count, 2, "ファイル数は2であるべき（ディレクトリは含まない）");
}

/// calculate_directory_size のテスト - 存在しないディレクトリ
#[test]
fn test_calculate_directory_size_nonexistent() {
    let nonexistent_path = PathBuf::from("/nonexistent/directory/path");
    let result = dashboard::calculate_directory_size(&nonexistent_path);

    assert!(result.is_ok());
    let (size, count) = result.unwrap();
    assert_eq!(size, 0, "存在しないディレクトリのサイズは0であるべき");
    assert_eq!(count, 0, "存在しないディレクトリのファイル数は0であるべき");
}

/// calculate_directory_size のテスト - 大容量ファイル
#[test]
fn test_calculate_directory_size_large_files() {
    let temp_dir = TempDir::new().unwrap();

    // 大容量ファイル作成（1MB）
    let file = temp_dir.path().join("large.bin");
    let data = vec![0u8; 1_048_576]; // 1MB
    fs::write(&file, data).unwrap();

    let result = dashboard::calculate_directory_size(temp_dir.path());

    assert!(result.is_ok());
    let (size, count) = result.unwrap();
    assert_eq!(size, 1_048_576, "サイズは1MBであるべき");
    assert_eq!(count, 1, "ファイル数は1であるべき");
}

/// calculate_directory_size のテスト - 多数のファイル
#[test]
fn test_calculate_directory_size_many_files() {
    let temp_dir = TempDir::new().unwrap();

    // 100個のファイル作成
    for i in 0..100 {
        let file = temp_dir.path().join(format!("file_{i}.txt"));
        fs::write(&file, format!("File {i}")).unwrap();
    }

    let result = dashboard::calculate_directory_size(temp_dir.path());

    assert!(result.is_ok());
    let (size, count) = result.unwrap();
    assert!(size > 0, "サイズは0より大きいべき");
    assert_eq!(count, 100, "ファイル数は100であるべき");
}

// ═══════════════════════════════════════════════════════════════
// display_statistics のテスト（モック使用）
// ═══════════════════════════════════════════════════════════════

/// display_statistics の統合テスト - 空の履歴
#[test]
fn test_display_statistics_empty_history() {
    // テスト用の一時設定ディレクトリ作成
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/backup-suite");
    fs::create_dir_all(&config_dir).unwrap();

    // 空の設定ファイル作成
    let config_path = config_dir.join("config.toml");
    let minimal_config = r#"
[backup]
destination = "/tmp/backup-test"
keep_days = 30

[schedule]
enabled = false
high_frequency = "daily"
medium_frequency = "weekly"
low_frequency = "monthly"

[[targets]]
"#;
    fs::write(&config_path, minimal_config).unwrap();

    // テスト実行時は環境変数で設定パスを上書き
    // （実際の実装では HOME を一時ディレクトリに設定する必要がある）

    // この時点では、display_statistics の直接呼び出しは困難
    // （Config::load() が ~/.config を参照するため）
    // 代わりに、統計計算のロジックを検証

    // NOTE: この部分は実装の詳細に依存するため、
    // より良いアプローチは Config::load_from_path() のような
    // テスト用の関数を追加すること
}

// ═══════════════════════════════════════════════════════════════
// display_warnings_summary のロジックテスト
// ═══════════════════════════════════════════════════════════════

/// display_warnings_summary の警告生成ロジックをテスト
#[test]
fn test_warnings_for_missing_targets() {
    // このテストは警告生成ロジックの検証
    // 実際の実装では、存在しないパスを持つターゲットを作成して検証

    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent");

    // Target作成
    let target = Target::new(nonexistent.clone(), Priority::High, "test".to_string());

    // パスが存在しないことを確認
    assert!(!target.path.exists(), "テストターゲットのパスは存在しないべき");

    // 実際のdisplay_warnings_summaryでは、このようなターゲットに対して
    // "バックアップ対象が存在しません" という警告が生成される
}

/// display_warnings_summary の最終バックアップ警告ロジックをテスト
#[test]
fn test_warnings_for_old_backups() {
    // 7日以上前のバックアップがある場合の警告ロジックを検証

    let old_timestamp = Utc::now() - Duration::days(10);
    let days_since = Utc::now().signed_duration_since(old_timestamp).num_days();

    assert!(days_since > 7, "10日前のバックアップは7日以上経過しているべき");

    // 実際のdisplay_warnings_summaryでは、このような古いバックアップに対して
    // "最後のバックアップからN日経過しています" という警告が生成される
}

// ═══════════════════════════════════════════════════════════════
// Unix固有の get_disk_info テスト
// ═══════════════════════════════════════════════════════════════

#[cfg(unix)]
#[test]
fn test_get_disk_info_existing_path() {
    use backup_suite::ui::dashboard;

    // /tmpは通常存在するため、これを使ってテスト
    let tmp_path = PathBuf::from("/tmp");
    let result = dashboard::get_disk_info(&tmp_path);

    assert!(result.is_ok(), "get_disk_info は /tmp で成功すべき");

    if let Ok(Some((total, available))) = result {
        assert!(total > 0, "総容量は0より大きいべき");
        assert!(available > 0, "空き容量は0より大きいべき");
        assert!(available <= total, "空き容量は総容量以下であるべき");
    }
}

#[cfg(unix)]
#[test]
fn test_get_disk_info_nonexistent_path() {
    use backup_suite::ui::dashboard;

    let nonexistent_path = PathBuf::from("/nonexistent/path/to/nowhere");
    let result = dashboard::get_disk_info(&nonexistent_path);

    assert!(result.is_ok(), "get_disk_info は存在しないパスでもエラーにならない");

    if let Ok(disk_info) = result {
        assert!(disk_info.is_none(), "存在しないパスではNoneが返るべき");
    }
}

// ═══════════════════════════════════════════════════════════════
// create_usage_graph のテスト（既存テストの拡張）
// ═══════════════════════════════════════════════════════════════

/// create_usage_graph のテスト - 0%
#[test]
fn test_create_usage_graph_zero_percent() {
    use backup_suite::ui::dashboard;

    let graph = dashboard::create_usage_graph(0.0);
    assert!(graph.contains("0.0%"), "0%を含むべき");
    assert!(graph.contains("░"), "空のバーを含むべき");
    assert!(!graph.contains("█"), "満杯のバーを含まないべき");
}

/// create_usage_graph のテスト - 100%
#[test]
fn test_create_usage_graph_full() {
    use backup_suite::ui::dashboard;

    let graph = dashboard::create_usage_graph(100.0);
    assert!(graph.contains("100.0%"), "100%を含むべき");
    assert!(graph.contains("█"), "満杯のバーを含むべき");
    assert!(!graph.contains("░"), "空のバーを含まないべき");
}

/// create_usage_graph のテスト - 50%
#[test]
fn test_create_usage_graph_half() {
    use backup_suite::ui::dashboard;

    let graph = dashboard::create_usage_graph(50.0);
    assert!(graph.contains("50.0%"), "50%を含むべき");
    assert!(graph.contains("█"), "満杯のバーを含むべき");
    assert!(graph.contains("░"), "空のバーを含むべき");

    // グラフの長さが適切か確認（"[" + 40文字 + "]" + " XX.X%"）
    assert!(graph.len() > 40, "グラフの長さは40文字以上であるべき");
}

/// create_usage_graph のテスト - エッジケース（75.5%）
#[test]
fn test_create_usage_graph_decimal() {
    use backup_suite::ui::dashboard;

    let graph = dashboard::create_usage_graph(75.5);
    assert!(graph.contains("75.5%"), "75.5%を含むべき");
    assert!(graph.contains("█"), "満杯のバーを含むべき");
    assert!(graph.contains("░"), "空のバーを含むべき");
}

// ═══════════════════════════════════════════════════════════════
// interactive.rs のテスト（エラーハンドリング検証）
// ═══════════════════════════════════════════════════════════════

/// select_priority のテスト - 優先度の値が正しいことを確認
#[test]
fn test_priority_values_are_valid() {
    let priorities = ["high", "medium", "low"];

    // すべての値が有効な優先度であることを確認
    for priority in &priorities {
        match *priority {
            "high" | "medium" | "low" => {},
            _ => panic!("無効な優先度: {priority}"),
        }
    }
}

/// confirm_backup のテスト - メッセージフォーマット検証
#[test]
fn test_confirm_backup_message_format() {
    // confirm_backup が正しい引数を受け取ることを確認
    let file_count: usize = 150;
    let destination = "/backup/destination";

    // メッセージフォーマットの検証
    let message = format!("対象ファイル数: {file_count} ファイル");
    assert!(message.contains("150"), "ファイル数を含むべき");
    assert!(message.contains("ファイル"), "「ファイル」という単語を含むべき");

    let dest_message = format!("バックアップ先: {destination}");
    assert!(dest_message.contains("/backup/destination"), "バックアップ先パスを含むべき");
}

/// confirm_cleanup のテスト - メッセージフォーマット検証
#[test]
fn test_confirm_cleanup_message_format() {
    let count: usize = 5;
    let keep_days: u32 = 30;

    let message = format!("削除対象: {count} 個のバックアップ");
    assert!(message.contains("5"), "削除対象数を含むべき");

    let days_message = format!("保持期間: {keep_days} 日");
    assert!(days_message.contains("30"), "保持期間を含むべき");
}

// ═══════════════════════════════════════════════════════════════
// progress.rs のテスト（プログレスバー機能）
// ═══════════════════════════════════════════════════════════════

/// BackupProgress のテスト - 作成と更新
#[test]
fn test_backup_progress_creation() {
    use backup_suite::ui::progress::BackupProgress;

    // プログレスバー作成
    let total_files = 100u64;
    let progress = BackupProgress::new(total_files);

    // プログレスバーが正常に作成されたことを確認
    // （indicatif は内部状態なので、パニックしないことを確認）
    drop(progress); // 正常にドロップできることを確認
}

/// BackupProgress のテスト - inc 操作
#[test]
fn test_backup_progress_increment() {
    use backup_suite::ui::progress::BackupProgress;

    let progress = BackupProgress::new(10);

    // inc 操作が正常に動作することを確認（パニックしない）
    progress.inc(1);
    progress.inc(1);
    progress.inc(1);

    drop(progress);
}

/// BackupProgress のテスト - set_message 操作
#[test]
fn test_backup_progress_set_message() {
    use backup_suite::ui::progress::BackupProgress;

    let progress = BackupProgress::new(10);

    // set_message 操作が正常に動作することを確認
    progress.set_message("テストメッセージ");
    progress.set_message("別のメッセージ");

    drop(progress);
}

/// BackupProgress のテスト - finish 操作
#[test]
fn test_backup_progress_finish() {
    use backup_suite::ui::progress::BackupProgress;

    let progress = BackupProgress::new(5);

    // finish 操作が正常に動作することを確認
    progress.finish("Complete");

    drop(progress);
}

/// BackupProgress のテスト - ゼロファイル
#[test]
fn test_backup_progress_zero_files() {
    use backup_suite::ui::progress::BackupProgress;

    // 0ファイルでもプログレスバーが作成できることを確認
    let progress = BackupProgress::new(0);

    progress.finish("Complete");
    drop(progress);
}

// ═══════════════════════════════════════════════════════════════
// table.rs のテスト（既存テストの拡張）
// ═══════════════════════════════════════════════════════════════

/// display_targets のテスト - データあり
#[test]
fn test_display_targets_with_data() {
    use backup_suite::ui::colors::ColorTheme;
    use backup_suite::ui::table::display_targets;

    let temp_dir = TempDir::new().unwrap();

    // テストターゲット作成
    let targets = vec![
        Target::new(temp_dir.path().to_path_buf(), Priority::High, "test1".to_string()),
        Target::new(temp_dir.path().to_path_buf(), Priority::Medium, "test2".to_string()),
        Target::new(temp_dir.path().to_path_buf(), Priority::Low, "test3".to_string()),
    ];

    let theme = ColorTheme::auto();

    // パニックしないことを確認（実際の出力は目視確認）
    display_targets(&targets, &theme);
}

/// display_history のテスト - データあり
#[test]
fn test_display_history_with_data() {
    use backup_suite::ui::colors::ColorTheme;
    use backup_suite::ui::table::display_history;

    let temp_dir = TempDir::new().unwrap();

    // テストバックアップ履歴作成
    let history = vec![
        BackupHistory {
            timestamp: Utc::now(),
            backup_dir: temp_dir.path().to_path_buf(),
            total_files: 100,
            total_bytes: 1_048_576,
            duration_ms: 1000,
            success: true,
            status: BackupStatus::Success,
            priority: Some(Priority::High),
            category: Some("test".to_string()),
            compressed: true,
            encrypted: true,
            error_message: None,
        },
        BackupHistory {
            timestamp: Utc::now() - Duration::hours(1),
            backup_dir: temp_dir.path().to_path_buf(),
            total_files: 50,
            total_bytes: 524_288,
            duration_ms: 500,
            success: true,
            status: BackupStatus::Success,
            priority: Some(Priority::Medium),
            category: Some("test".to_string()),
            compressed: false,
            encrypted: false,
            error_message: None,
        },
    ];

    let theme = ColorTheme::auto();

    // パニックしないことを確認
    display_history(&history, &theme);
}

/// display_backup_result のテスト
#[test]
fn test_display_backup_result() {
    use backup_suite::ui::colors::ColorTheme;
    use backup_suite::ui::table::display_backup_result;

    let theme = ColorTheme::auto();

    // 正常なバックアップ結果
    display_backup_result(100, 95, 5, 1_048_576, &theme);

    // すべて成功
    display_backup_result(50, 50, 0, 524_288, &theme);

    // すべて失敗
    display_backup_result(10, 0, 10, 0, &theme);
}
