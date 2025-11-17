use super::colors::ColorTheme;
use crate::core::{BackupHistory, Priority, Target, TargetType};
use crate::i18n::{get_message, Language, MessageKey};
/// テーブル表示モジュール
///
/// comfy-tableを使用した美しい表形式の出力
use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, Color, ContentArrangement, Table};

/// バックアップ対象一覧をテーブル表示
pub fn display_targets(targets: &[Target], theme: &ColorTheme, lang: Language) {
    if targets.is_empty() {
        println!(
            "{}",
            theme
                .warning()
                .apply_to(get_message(MessageKey::NoTargetsRegistered, lang))
        );
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("No").set_alignment(CellAlignment::Right),
            Cell::new(match lang {
                Language::English => "Priority",
                Language::Japanese => "優先度",
                Language::SimplifiedChinese => "优先级",
                Language::TraditionalChinese => "優先級",
            })
            .set_alignment(CellAlignment::Center),
            Cell::new(match lang {
                Language::English => "Type",
                Language::Japanese => "種別",
                Language::SimplifiedChinese => "类型",
                Language::TraditionalChinese => "類型",
            })
            .set_alignment(CellAlignment::Center),
            Cell::new(match lang {
                Language::English => "Path",
                Language::Japanese => "パス",
                Language::SimplifiedChinese => "路径",
                Language::TraditionalChinese => "路徑",
            }),
            Cell::new(match lang {
                Language::English => "Category",
                Language::Japanese => "カテゴリ",
                Language::SimplifiedChinese => "类别",
                Language::TraditionalChinese => "類別",
            })
            .set_alignment(CellAlignment::Center),
            Cell::new(match lang {
                Language::English => "Excludes",
                Language::Japanese => "除外パターン",
                Language::SimplifiedChinese => "排除模式",
                Language::TraditionalChinese => "排除模式",
            })
            .set_alignment(CellAlignment::Right),
            Cell::new(match lang {
                Language::English => "Added",
                Language::Japanese => "追加日",
                Language::SimplifiedChinese => "添加日期",
                Language::TraditionalChinese => "新增日期",
            }),
        ]);

    for (idx, target) in targets.iter().enumerate() {
        let priority_cell = match target.priority {
            Priority::High => Cell::new(match lang {
                Language::English => "High",
                Language::Japanese => "高",
                Language::SimplifiedChinese => "高",
                Language::TraditionalChinese => "高",
            })
            .fg(Color::Red),
            Priority::Medium => Cell::new(match lang {
                Language::English => "Medium",
                Language::Japanese => "中",
                Language::SimplifiedChinese => "中",
                Language::TraditionalChinese => "中",
            })
            .fg(Color::Yellow),
            Priority::Low => Cell::new(match lang {
                Language::English => "Low",
                Language::Japanese => "低",
                Language::SimplifiedChinese => "低",
                Language::TraditionalChinese => "低",
            })
            .fg(Color::Cyan),
        };

        let type_cell = match target.target_type {
            TargetType::File => Cell::new(match lang {
                Language::English => "📄 File",
                Language::Japanese => "📄 ファイル",
                Language::SimplifiedChinese => "📄 文件",
                Language::TraditionalChinese => "📄 檔案",
            }),
            TargetType::Directory => Cell::new(match lang {
                Language::English => "📁 Directory",
                Language::Japanese => "📁 ディレクトリ",
                Language::SimplifiedChinese => "📁 目录",
                Language::TraditionalChinese => "📁 目錄",
            }),
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

    println!(
        "\n{}",
        theme.header().apply_to(match lang {
            Language::English => "📋 Backup Targets",
            Language::Japanese => "📋 バックアップ対象一覧",
            Language::SimplifiedChinese => "📋 备份目标列表",
            Language::TraditionalChinese => "📋 備份目標清單",
        })
    );
    println!("{table}\n");
}

/// バックアップ履歴をテーブル表示
pub fn display_history(history: &[BackupHistory], theme: &ColorTheme, lang: Language) {
    if history.is_empty() {
        println!(
            "{}",
            theme.warning().apply_to(match lang {
                Language::English => "No backup history",
                Language::Japanese => "バックアップ履歴がありません",
                Language::SimplifiedChinese => "没有备份历史",
                Language::TraditionalChinese => "沒有備份歷史",
            })
        );
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("No").set_alignment(CellAlignment::Right),
            Cell::new(match lang {
                Language::English => "Date/Time",
                Language::Japanese => "日時",
                Language::SimplifiedChinese => "日期时间",
                Language::TraditionalChinese => "日期時間",
            }),
            Cell::new(match lang {
                Language::English => "Files",
                Language::Japanese => "ファイル数",
                Language::SimplifiedChinese => "文件数",
                Language::TraditionalChinese => "檔案數",
            })
            .set_alignment(CellAlignment::Right),
            Cell::new(match lang {
                Language::English => "Size",
                Language::Japanese => "サイズ",
                Language::SimplifiedChinese => "大小",
                Language::TraditionalChinese => "大小",
            })
            .set_alignment(CellAlignment::Right),
            Cell::new(match lang {
                Language::English => "Status",
                Language::Japanese => "状態",
                Language::SimplifiedChinese => "状态",
                Language::TraditionalChinese => "狀態",
            })
            .set_alignment(CellAlignment::Center),
            Cell::new(match lang {
                Language::English => "Backup Directory",
                Language::Japanese => "バックアップ先",
                Language::SimplifiedChinese => "备份目录",
                Language::TraditionalChinese => "備份目錄",
            }),
        ]);

    for (idx, entry) in history.iter().enumerate() {
        let timestamp = entry
            .timestamp
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M")
            .to_string();

        let size = format_bytes(entry.total_bytes);

        let status_cell = if entry.success {
            Cell::new(match lang {
                Language::English => "✓ Success",
                Language::Japanese => "✓ 成功",
                Language::SimplifiedChinese => "✓ 成功",
                Language::TraditionalChinese => "✓ 成功",
            })
            .fg(Color::Green)
        } else {
            Cell::new(match lang {
                Language::English => "✗ Failed",
                Language::Japanese => "✗ 失敗",
                Language::SimplifiedChinese => "✗ 失败",
                Language::TraditionalChinese => "✗ 失敗",
            })
            .fg(Color::Red)
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
    lang: Language,
) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.add_row(vec![
        Cell::new(get_message(MessageKey::TotalFilesLabel, lang)),
        Cell::new(total_files.to_string()).set_alignment(CellAlignment::Right),
    ]);

    table.add_row(vec![
        Cell::new(get_message(MessageKey::SuccessfulLabel, lang)),
        Cell::new(success_files.to_string())
            .fg(Color::Green)
            .set_alignment(CellAlignment::Right),
    ]);

    if failed_files > 0 {
        table.add_row(vec![
            Cell::new(get_message(MessageKey::FailedLabel, lang)),
            Cell::new(failed_files.to_string())
                .fg(Color::Red)
                .set_alignment(CellAlignment::Right),
        ]);
    }

    table.add_row(vec![
        Cell::new(get_message(MessageKey::TotalSizeLabel, lang)),
        Cell::new(format_bytes(total_bytes)).set_alignment(CellAlignment::Right),
    ]);

    println!(
        "\n\n{}",
        theme
            .header()
            .apply_to(get_message(MessageKey::BackupResultTitle, lang))
    );
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
            Target::new(
                temp_dir.path().to_path_buf(),
                Priority::High,
                "cat1".to_string(),
            ),
            Target::new(
                temp_dir.path().to_path_buf(),
                Priority::Medium,
                "cat2".to_string(),
            ),
            Target::new(
                temp_dir.path().to_path_buf(),
                Priority::Low,
                "cat3".to_string(),
            ),
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
