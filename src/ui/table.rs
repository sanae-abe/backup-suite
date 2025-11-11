use super::colors::ColorTheme;
use crate::core::{BackupHistory, Priority, Target, TargetType};
/// テーブル表示モジュール
///
/// comfy-tableを使用した美しい表形式の出力
use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, Color, ContentArrangement, Table};

/// バックアップ対象一覧をテーブル表示
pub fn display_targets(targets: &[Target], theme: &ColorTheme) {
    if targets.is_empty() {
        println!(
            "{}",
            theme
                .warning()
                .apply_to("バックアップ対象が登録されていません")
        );
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("No").set_alignment(CellAlignment::Right),
            Cell::new("優先度").set_alignment(CellAlignment::Center),
            Cell::new("種別").set_alignment(CellAlignment::Center),
            Cell::new("パス"),
            Cell::new("カテゴリ").set_alignment(CellAlignment::Center),
            Cell::new("除外パターン").set_alignment(CellAlignment::Right),
            Cell::new("追加日"),
        ]);

    for (idx, target) in targets.iter().enumerate() {
        let priority_cell = match target.priority {
            Priority::High => Cell::new("高").fg(Color::Red),
            Priority::Medium => Cell::new("中").fg(Color::Yellow),
            Priority::Low => Cell::new("低").fg(Color::Cyan),
        };

        let type_cell = match target.target_type {
            TargetType::File => Cell::new("📄 ファイル"),
            TargetType::Directory => Cell::new("📁 ディレクトリ"),
        };

        let exclude_count = if target.exclude_patterns.is_empty() {
            Cell::new("-").set_alignment(CellAlignment::Center)
        } else {
            Cell::new(target.exclude_patterns.len().to_string()).fg(Color::Yellow)
        };

        let added_date = target.added_date.format("%Y-%m-%d").to_string();

        table.add_row(vec![
            Cell::new((idx + 1).to_string()).set_alignment(CellAlignment::Right),
            priority_cell.set_alignment(CellAlignment::Center),
            type_cell.set_alignment(CellAlignment::Center),
            Cell::new(target.path.display().to_string()),
            Cell::new(&target.category).set_alignment(CellAlignment::Center),
            exclude_count,
            Cell::new(added_date),
        ]);
    }

    println!("\n{}", theme.header().apply_to("📋 バックアップ対象一覧"));
    println!("{table}\n");
}

/// バックアップ履歴をテーブル表示
pub fn display_history(history: &[BackupHistory], theme: &ColorTheme) {
    if history.is_empty() {
        println!(
            "{}",
            theme.warning().apply_to("バックアップ履歴がありません")
        );
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("No").set_alignment(CellAlignment::Right),
            Cell::new("日時"),
            Cell::new("ファイル数").set_alignment(CellAlignment::Right),
            Cell::new("サイズ").set_alignment(CellAlignment::Right),
            Cell::new("状態").set_alignment(CellAlignment::Center),
            Cell::new("バックアップ先"),
        ]);

    for (idx, entry) in history.iter().enumerate() {
        let timestamp = entry
            .timestamp
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M")
            .to_string();

        let size = format_bytes(entry.total_bytes);

        let status_cell = if entry.success {
            Cell::new("✓ 成功").fg(Color::Green)
        } else {
            Cell::new("✗ 失敗").fg(Color::Red)
        };

        table.add_row(vec![
            Cell::new((idx + 1).to_string()).set_alignment(CellAlignment::Right),
            Cell::new(timestamp),
            Cell::new(entry.total_files.to_string()).set_alignment(CellAlignment::Right),
            Cell::new(size).set_alignment(CellAlignment::Right),
            status_cell.set_alignment(CellAlignment::Center),
            Cell::new(entry.backup_dir.display().to_string()),
        ]);
    }

    println!("{table}\n");
}

/// バックアップ結果をテーブル表示
pub fn display_backup_result(
    total_files: usize,
    success_files: usize,
    failed_files: usize,
    total_bytes: u64,
    theme: &ColorTheme,
) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.add_row(vec![
        Cell::new("総ファイル数"),
        Cell::new(total_files.to_string()).set_alignment(CellAlignment::Right),
    ]);

    table.add_row(vec![
        Cell::new("成功"),
        Cell::new(success_files.to_string())
            .fg(Color::Green)
            .set_alignment(CellAlignment::Right),
    ]);

    if failed_files > 0 {
        table.add_row(vec![
            Cell::new("失敗"),
            Cell::new(failed_files.to_string())
                .fg(Color::Red)
                .set_alignment(CellAlignment::Right),
        ]);
    }

    table.add_row(vec![
        Cell::new("合計サイズ"),
        Cell::new(format_bytes(total_bytes)).set_alignment(CellAlignment::Right),
    ]);

    println!("\n\n{}", theme.header().apply_to("📈 バックアップ結果"));
    println!("{table}\n");
}

/// バイト数を人間が読める形式に変換
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::history::BackupStatus;
    use chrono::Utc;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1_048_576), "1.00 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.00 GB");
    }

    #[test]
    fn test_format_bytes_edge_cases() {
        assert_eq!(format_bytes(1), "1 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1025), "1.00 KB");
        assert_eq!(format_bytes(1_048_575), "1024.00 KB");
        assert_eq!(format_bytes(1_073_741_823), "1024.00 MB");
        assert_eq!(format_bytes(1_099_511_627_776), "1.00 TB");
    }

    #[test]
    fn test_display_empty_targets() {
        let targets: Vec<Target> = vec![];
        let theme = ColorTheme::auto();

        // パニックしないことを確認
        display_targets(&targets, &theme);
    }

    #[test]
    fn test_display_empty_history() {
        let history: Vec<BackupHistory> = vec![];
        let theme = ColorTheme::auto();

        // パニックしないことを確認
        display_history(&history, &theme);
    }

    #[test]
    fn test_display_targets_with_high_priority() {
        let temp_dir = TempDir::new().unwrap();
        let targets = vec![Target::new(
            temp_dir.path().to_path_buf(),
            Priority::High,
            "test-high".to_string(),
        )];
        let theme = ColorTheme::auto();

        // パニックしないことを確認
        display_targets(&targets, &theme);
    }

    #[test]
    fn test_display_targets_with_medium_priority() {
        let temp_dir = TempDir::new().unwrap();
        let targets = vec![Target::new(
            temp_dir.path().to_path_buf(),
            Priority::Medium,
            "test-medium".to_string(),
        )];
        let theme = ColorTheme::auto();

        display_targets(&targets, &theme);
    }

    #[test]
    fn test_display_targets_with_low_priority() {
        let temp_dir = TempDir::new().unwrap();
        let targets = vec![Target::new(
            temp_dir.path().to_path_buf(),
            Priority::Low,
            "test-low".to_string(),
        )];
        let theme = ColorTheme::auto();

        display_targets(&targets, &theme);
    }

    #[test]
    fn test_display_targets_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let targets = vec![
            Target::new(temp_dir.path().to_path_buf(), Priority::High, "cat1".to_string()),
            Target::new(temp_dir.path().to_path_buf(), Priority::Medium, "cat2".to_string()),
            Target::new(temp_dir.path().to_path_buf(), Priority::Low, "cat3".to_string()),
        ];
        let theme = ColorTheme::auto();

        display_targets(&targets, &theme);
    }

    #[test]
    fn test_display_targets_with_exclude_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let mut target = Target::new(
            temp_dir.path().to_path_buf(),
            Priority::High,
            "test".to_string(),
        );
        target.exclude_patterns = vec!["*.tmp".to_string(), "node_modules".to_string()];

        let theme = ColorTheme::auto();
        display_targets(&[target], &theme);
    }

    #[test]
    fn test_display_history_with_success() {
        let temp_dir = TempDir::new().unwrap();
        let history = vec![BackupHistory {
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
        }];
        let theme = ColorTheme::auto();

        display_history(&history, &theme);
    }

    #[test]
    fn test_display_history_with_failure() {
        let temp_dir = TempDir::new().unwrap();
        let history = vec![BackupHistory {
            timestamp: Utc::now(),
            backup_dir: temp_dir.path().to_path_buf(),
            total_files: 50,
            total_bytes: 524_288,
            duration_ms: 500,
            success: false,
            status: BackupStatus::Failed,
            priority: Some(Priority::Medium),
            category: Some("test".to_string()),
            compressed: false,
            encrypted: false,
            error_message: Some("Test error".to_string()),
        }];
        let theme = ColorTheme::auto();

        display_history(&history, &theme);
    }

    #[test]
    fn test_display_history_multiple_entries() {
        let temp_dir = TempDir::new().unwrap();
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
                category: Some("cat1".to_string()),
                compressed: true,
                encrypted: true,
                error_message: None,
            },
            BackupHistory {
                timestamp: Utc::now(),
                backup_dir: temp_dir.path().to_path_buf(),
                total_files: 50,
                total_bytes: 524_288,
                duration_ms: 500,
                success: false,
                status: BackupStatus::Failed,
                priority: Some(Priority::Medium),
                category: Some("cat2".to_string()),
                compressed: false,
                encrypted: false,
                error_message: Some("Error".to_string()),
            },
        ];
        let theme = ColorTheme::auto();

        display_history(&history, &theme);
    }

    #[test]
    fn test_display_backup_result_all_success() {
        let theme = ColorTheme::auto();
        display_backup_result(100, 100, 0, 1_048_576, &theme);
    }

    #[test]
    fn test_display_backup_result_with_failures() {
        let theme = ColorTheme::auto();
        display_backup_result(100, 95, 5, 1_048_576, &theme);
    }

    #[test]
    fn test_display_backup_result_all_failed() {
        let theme = ColorTheme::auto();
        display_backup_result(10, 0, 10, 0, &theme);
    }

    #[test]
    fn test_display_backup_result_zero_files() {
        let theme = ColorTheme::auto();
        display_backup_result(0, 0, 0, 0, &theme);
    }

    #[test]
    fn test_display_backup_result_large_numbers() {
        let theme = ColorTheme::auto();
        display_backup_result(10_000, 9_999, 1, 1_099_511_627_776, &theme);
    }
}
