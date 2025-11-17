/// ダッシュボードUI モジュール
///
/// 統計情報、バックアップ履歴、エラー・警告サマリー、ディスク使用量の統合ビュー
use anyhow::Result;
use chrono::Utc;
use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, Color, ContentArrangement, Table};
#[cfg(unix)]
use std::fs;

use super::colors::ColorTheme;
use super::table::display_history;
use crate::core::{BackupHistory, Config, Priority};
use crate::i18n::{get_message, MessageKey};

/// ダッシュボード表示
///
/// # Errors
///
/// 次の場合にエラーを返します:
/// - 設定ファイルやバックアップ履歴の読み込みに失敗した場合
/// - バックアップディレクトリの情報取得に失敗した場合
/// - ディスク情報の取得に失敗した場合（Unix系のみ）
pub fn display_dashboard(lang: crate::i18n::Language) -> Result<()> {
    let theme = ColorTheme::auto();

    println!(
        "\n{}",
        theme
            .header()
            .apply_to("═══════════════════════════════════════════════════════════════")
    );
    println!(
        "{}",
        theme
            .header()
            .apply_to("                    📊 Backup Suite Dashboard")
    );
    println!(
        "{}",
        theme
            .header()
            .apply_to("═══════════════════════════════════════════════════════════════\n")
    );

    // 統計情報表示
    display_statistics(&theme, lang)?;

    println!();

    // ディスク使用量グラフ
    display_disk_usage(&theme, lang)?;

    println!();

    // 最近のバックアップ一覧
    display_recent_backups(&theme, lang)?;

    println!();

    // エラー・警告サマリー
    display_warnings_summary(&theme, lang)?;

    println!();

    Ok(())
}

/// 統計情報表示
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn display_statistics(theme: &ColorTheme, lang: crate::i18n::Language) -> Result<()> {
    let config = Config::load()?;
    let history = BackupHistory::load_all()?;

    // 統計情報の計算（各優先度の正確な件数をカウント）
    let total_targets = config.targets.len();
    let high_priority = config
        .targets
        .iter()
        .filter(|t| t.priority == Priority::High)
        .count();
    let medium_priority = config
        .targets
        .iter()
        .filter(|t| t.priority == Priority::Medium)
        .count();
    let low_priority = config
        .targets
        .iter()
        .filter(|t| t.priority == Priority::Low)
        .count();

    let total_backups = history.len();
    let successful_backups = history.iter().filter(|h| h.success).count();
    let failed_backups = history.iter().filter(|h| !h.success).count();

    let total_files: usize = history.iter().map(|h| h.total_files).sum();
    let total_bytes: u64 = history.iter().map(|h| h.total_bytes).sum();

    // 暗号化・圧縮統計
    let encrypted_backups = history.iter().filter(|h| h.encrypted).count();
    let compressed_backups = history.iter().filter(|h| h.compressed).count();

    // 最新バックアップ情報
    let last_backup = history.last();
    let last_backup_str = if let Some(backup) = last_backup {
        let duration = Utc::now().signed_duration_since(backup.timestamp);
        if duration.num_days() > 0 {
            get_message(MessageKey::DaysAgo, lang).replace("{}", &duration.num_days().to_string())
        } else if duration.num_hours() > 0 {
            get_message(MessageKey::HoursAgo, lang).replace("{}", &duration.num_hours().to_string())
        } else if duration.num_minutes() > 0 {
            get_message(MessageKey::MinutesAgo, lang)
                .replace("{}", &duration.num_minutes().to_string())
        } else {
            get_message(MessageKey::JustNow, lang).to_string()
        }
    } else {
        get_message(MessageKey::NotYetBackedUp, lang).to_string()
    };

    println!(
        "{}",
        theme
            .header()
            .apply_to(get_message(MessageKey::StatisticsTitle, lang))
    );
    println!();

    // バックアップ対象の統計
    let mut targets_table = Table::new();
    targets_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    targets_table.add_row(vec![
        Cell::new(get_message(MessageKey::TotalTargetsLabel, lang)),
        Cell::new(total_targets.to_string())
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
    ]);
    targets_table.add_row(vec![
        Cell::new(get_message(MessageKey::HighPriorityTargetsLabel, lang)),
        Cell::new(high_priority.to_string())
            .fg(Color::Red)
            .set_alignment(CellAlignment::Right),
    ]);
    targets_table.add_row(vec![
        Cell::new(get_message(MessageKey::MediumPriorityTargetsLabel, lang)),
        Cell::new(medium_priority.to_string())
            .fg(Color::Yellow)
            .set_alignment(CellAlignment::Right),
    ]);
    targets_table.add_row(vec![
        Cell::new(get_message(MessageKey::LowPriorityTargetsLabel, lang)),
        Cell::new(low_priority.to_string())
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
    ]);

    println!("{targets_table}");
    println!();

    // バックアップ履歴の統計
    let mut history_table = Table::new();
    history_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    history_table.add_row(vec![
        Cell::new(get_message(MessageKey::TotalBackupsLabel, lang)),
        Cell::new(total_backups.to_string())
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
    ]);
    history_table.add_row(vec![
        Cell::new(get_message(MessageKey::SuccessCountLabel, lang)),
        Cell::new(successful_backups.to_string())
            .fg(Color::Green)
            .set_alignment(CellAlignment::Right),
    ]);
    if failed_backups > 0 {
        history_table.add_row(vec![
            Cell::new(format!("  {}", get_message(MessageKey::FailedLabel, lang))),
            Cell::new(failed_backups.to_string())
                .fg(Color::Red)
                .set_alignment(CellAlignment::Right),
        ]);
    }
    history_table.add_row(vec![
        Cell::new(get_message(MessageKey::TotalFilesCountLabel, lang)),
        Cell::new(total_files.to_string())
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
    ]);
    history_table.add_row(vec![
        Cell::new(get_message(MessageKey::TotalDataSizeLabel, lang)),
        Cell::new(format_bytes(total_bytes))
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
    ]);
    history_table.add_row(vec![
        Cell::new(get_message(MessageKey::LastBackupLabel, lang)),
        Cell::new(&last_backup_str)
            .fg(Color::Yellow)
            .set_alignment(CellAlignment::Right),
    ]);

    println!("{history_table}");
    println!();

    // セキュリティ統計
    let mut security_table = Table::new();
    security_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    let encryption_rate = if total_backups > 0 {
        (encrypted_backups as f64 / total_backups as f64) * 100.0
    } else {
        0.0
    };

    let compression_rate = if total_backups > 0 {
        (compressed_backups as f64 / total_backups as f64) * 100.0
    } else {
        0.0
    };

    security_table.add_row(vec![
        Cell::new(get_message(MessageKey::EncryptedBackupsLabel, lang)),
        Cell::new(format!("{encrypted_backups} ({encryption_rate:.1}%)"))
            .fg(Color::Green)
            .set_alignment(CellAlignment::Right),
    ]);
    security_table.add_row(vec![
        Cell::new(get_message(MessageKey::CompressedBackupsLabel, lang)),
        Cell::new(format!("{compressed_backups} ({compression_rate:.1}%)"))
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
    ]);

    println!("{security_table}");

    Ok(())
}

/// ディスク使用量表示
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn display_disk_usage(theme: &ColorTheme, lang: crate::i18n::Language) -> Result<()> {
    let config = Config::load()?;
    let backup_dir = &config.backup.destination;

    println!(
        "{}",
        theme
            .header()
            .apply_to(get_message(MessageKey::DiskUsageTitle, lang))
    );
    println!();

    // バックアップディレクトリのサイズを計算
    let (used_bytes, file_count) = calculate_directory_size(backup_dir)?;

    // ディスク全体の容量を取得（macOS/Linuxのみ）
    #[cfg(unix)]
    let disk_info = get_disk_info(backup_dir)?;

    #[cfg(not(unix))]
    let disk_info: Option<(u64, u64)> = None;

    let mut disk_table = Table::new();
    disk_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    disk_table.add_row(vec![
        Cell::new(get_message(MessageKey::BackupDirectoryLabel, lang)),
        Cell::new(backup_dir.display().to_string())
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Left),
    ]);

    disk_table.add_row(vec![
        Cell::new(get_message(MessageKey::UsedCapacityLabel, lang)),
        Cell::new(format_bytes(used_bytes))
            .fg(Color::Yellow)
            .set_alignment(CellAlignment::Right),
    ]);

    disk_table.add_row(vec![
        Cell::new(get_message(MessageKey::FileCountLabel, lang)),
        Cell::new(file_count.to_string())
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
    ]);

    #[cfg(unix)]
    if let Some((total, available)) = disk_info {
        let used_percent = ((total - available) as f64 / total as f64) * 100.0;

        disk_table.add_row(vec![
            Cell::new(get_message(MessageKey::DiskTotalCapacityLabel, lang)),
            Cell::new(format_bytes(total))
                .fg(Color::Cyan)
                .set_alignment(CellAlignment::Right),
        ]);

        disk_table.add_row(vec![
            Cell::new(get_message(MessageKey::DiskFreeCapacityLabel, lang)),
            Cell::new(format_bytes(available))
                .fg(if available < total / 10 {
                    Color::Red
                } else {
                    Color::Green
                })
                .set_alignment(CellAlignment::Right),
        ]);

        disk_table.add_row(vec![
            Cell::new(get_message(MessageKey::DiskUsageRateLabel, lang)),
            Cell::new(format!("{used_percent:.1}%"))
                .fg(if used_percent > 90.0 {
                    Color::Red
                } else if used_percent > 75.0 {
                    Color::Yellow
                } else {
                    Color::Green
                })
                .set_alignment(CellAlignment::Right),
        ]);

        // ディスク使用率のグラフ表示
        let graph = create_usage_graph(used_percent);
        disk_table.add_row(vec![
            Cell::new(get_message(MessageKey::UsageStatusLabel, lang)),
            Cell::new(graph)
                .fg(Color::Cyan)
                .set_alignment(CellAlignment::Left),
        ]);
    }

    println!("{disk_table}");

    Ok(())
}

/// ディレクトリサイズを計算
pub fn calculate_directory_size(dir: &std::path::Path) -> Result<(u64, usize)> {
    let mut total_size = 0u64;
    let mut file_count = 0usize;

    if !dir.exists() {
        return Ok((0, 0));
    }

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
                file_count += 1;
            }
        }
    }

    Ok((total_size, file_count))
}

/// ディスク情報を取得（Unix系のみ）
#[cfg(unix)]
pub fn get_disk_info(path: &std::path::Path) -> Result<Option<(u64, u64)>> {
    // ディレクトリが存在しない場合はNoneを返す
    if !path.exists() {
        return Ok(None);
    }

    use std::os::unix::fs::MetadataExt;

    let metadata = fs::metadata(path).ok();
    if metadata.is_none() {
        return Ok(None);
    }
    let _dev = metadata.unwrap().dev();

    // statfs を使ってディスク情報を取得
    use std::ffi::CString;
    use std::mem;

    let path_cstr = match CString::new(path.to_str().unwrap_or("/")) {
        Ok(cstr) => cstr,
        Err(_) => return Ok(None),
    };
    // SAFETY: libc::statfs構造体はC言語由来のPOD型であり、
    // mem::zeroed()で初期化することが安全。すべてのフィールドが数値型で、
    // ゼロ初期化された状態は有効な初期値として機能する。
    let mut stat: libc::statfs = unsafe { mem::zeroed() };

    // SAFETY: path_cstr は有効なCStringから取得したポインタで、
    // statはゼロ初期化された有効な構造体への可変参照。
    // libc::statfsはPOSIX標準のシステムコールで、正常なパラメータで呼び出している。
    let result = unsafe { libc::statfs(path_cstr.as_ptr(), &raw mut stat) };

    if result == 0 {
        #[allow(clippy::unnecessary_cast)]
        let block_size = stat.f_bsize as u64;
        #[allow(clippy::unnecessary_cast)]
        let total_blocks = stat.f_blocks as u64;
        #[allow(clippy::unnecessary_cast)]
        let available_blocks = stat.f_bavail as u64;

        let total_bytes = total_blocks * block_size;
        let available_bytes = available_blocks * block_size;

        Ok(Some((total_bytes, available_bytes)))
    } else {
        Ok(None)
    }
}

/// 使用率グラフを作成
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn create_usage_graph(percent: f64) -> String {
    let total_bars = 40;
    let filled_bars = ((percent / 100.0) * total_bars as f64) as usize;
    let empty_bars = total_bars - filled_bars;

    let filled = "█".repeat(filled_bars);
    let empty = "░".repeat(empty_bars);

    format!("[{filled}{empty}] {percent:.1}%")
}

/// 最近のバックアップ一覧（直近5件）
fn display_recent_backups(theme: &ColorTheme, lang: crate::i18n::Language) -> Result<()> {
    let history = BackupHistory::load_all()?;

    if history.is_empty() {
        println!(
            "{}",
            theme.warning().apply_to("バックアップ履歴がありません")
        );
        return Ok(());
    }

    // 最新5件を取得
    let recent: Vec<_> = history.iter().rev().take(5).cloned().collect();

    println!(
        "{}",
        theme
            .header()
            .apply_to(get_message(MessageKey::RecentBackupsTitle, lang))
    );
    display_history(&recent, theme, lang);

    Ok(())
}

/// エラー・警告サマリー
#[allow(clippy::cast_precision_loss)]
fn display_warnings_summary(theme: &ColorTheme, lang: crate::i18n::Language) -> Result<()> {
    let config = Config::load()?;
    let mut warnings = Vec::new();

    // バックアップ対象が存在しない場合の警告
    for target in &config.targets {
        if !target.path.exists() {
            warnings.push(
                get_message(MessageKey::WarningTargetNotExists, lang)
                    .replace("{}", &target.path.display().to_string()),
            );
        }
    }

    // 最近のバックアップがない場合の警告
    let history = BackupHistory::load_all()?;
    if let Some(last) = history.last() {
        let days_since = Utc::now().signed_duration_since(last.timestamp).num_days();

        if days_since > 7 {
            warnings.push(
                get_message(MessageKey::WarningDaysSinceLastBackup, lang)
                    .replace("{}", &days_since.to_string()),
            );
        }
    } else {
        warnings.push(get_message(MessageKey::WarningNoBackupYet, lang).to_string());
    }

    // 失敗したバックアップの警告
    let failed_count = history.iter().filter(|h| !h.success).count();
    if failed_count > 0 {
        warnings.push(
            get_message(MessageKey::WarningFailedBackups, lang)
                .replace("{}", &failed_count.to_string()),
        );
    }

    // ディスク容量警告
    #[cfg(unix)]
    {
        if let Ok(Some((total, available))) = get_disk_info(&config.backup.destination) {
            let available_percent = (available as f64 / total as f64) * 100.0;
            if available_percent < 10.0 {
                warnings.push(
                    get_message(MessageKey::WarningLowDiskSpace, lang)
                        .replace("{:.1}", &format!("{:.1}", available_percent)),
                );
            }
        }
    }

    // 警告表示
    if warnings.is_empty() {
        println!(
            "{}",
            theme
                .success()
                .apply_to(get_message(MessageKey::AllNormalStatus, lang))
        );
    } else {
        println!(
            "{}",
            theme
                .header()
                .apply_to(get_message(MessageKey::WarningsTitle, lang))
        );
        println!();

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        for (idx, warning) in warnings.iter().enumerate() {
            table.add_row(vec![
                Cell::new((idx + 1).to_string())
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(warning).fg(Color::Yellow),
            ]);
        }

        println!("{table}");
        println!(
            "\n{}",
            theme
                .info()
                .apply_to("💡 ヒント: 'backup-suite run' でバックアップを実行できます")
        );
    }

    Ok(())
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
    use tempfile::TempDir;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
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
    fn test_create_usage_graph() {
        let graph = create_usage_graph(50.0);
        assert!(graph.contains("50.0%"));
        assert!(graph.contains("█"));
        assert!(graph.contains("░"));
    }

    #[test]
    fn test_create_usage_graph_zero_percent() {
        let graph = create_usage_graph(0.0);
        assert!(graph.contains("0.0%"));
        assert!(graph.contains("░"));
        assert!(!graph.contains("█"));
    }

    #[test]
    fn test_create_usage_graph_full() {
        let graph = create_usage_graph(100.0);
        assert!(graph.contains("100.0%"));
        assert!(graph.contains("█"));
        assert!(!graph.contains("░"));
    }

    #[test]
    fn test_create_usage_graph_decimal() {
        let graph = create_usage_graph(75.5);
        assert!(graph.contains("75.5%"));
        assert!(graph.contains("█"));
        assert!(graph.contains("░"));
    }

    #[test]
    fn test_calculate_directory_size_empty() {
        let temp_dir = TempDir::new().unwrap();
        let result = calculate_directory_size(temp_dir.path());

        assert!(result.is_ok());
        let (size, count) = result.unwrap();
        assert_eq!(size, 0);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_calculate_directory_size_with_files() {
        let temp_dir = TempDir::new().unwrap();

        // テストファイル作成
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        std::fs::write(&file1, "Hello World").unwrap(); // 11 bytes
        std::fs::write(&file2, "Rust").unwrap(); // 4 bytes

        let result = calculate_directory_size(temp_dir.path());

        assert!(result.is_ok());
        let (size, count) = result.unwrap();
        assert_eq!(size, 15);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_calculate_directory_size_with_subdirs() {
        let temp_dir = TempDir::new().unwrap();

        // サブディレクトリ作成
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        // ファイル作成
        let file1 = temp_dir.path().join("root.txt");
        let file2 = subdir.join("sub.txt");
        std::fs::write(&file1, "Root").unwrap(); // 4 bytes
        std::fs::write(&file2, "Subdir").unwrap(); // 6 bytes

        let result = calculate_directory_size(temp_dir.path());

        assert!(result.is_ok());
        let (size, count) = result.unwrap();
        assert_eq!(size, 10);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_calculate_directory_size_nonexistent() {
        let nonexistent_path = std::path::PathBuf::from("/nonexistent/directory/path");
        let result = calculate_directory_size(&nonexistent_path);

        assert!(result.is_ok());
        let (size, count) = result.unwrap();
        assert_eq!(size, 0);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_calculate_directory_size_large_file() {
        let temp_dir = TempDir::new().unwrap();

        // 1MBファイル作成
        let file = temp_dir.path().join("large.bin");
        let data = vec![0u8; 1_048_576];
        std::fs::write(&file, data).unwrap();

        let result = calculate_directory_size(temp_dir.path());

        assert!(result.is_ok());
        let (size, count) = result.unwrap();
        assert_eq!(size, 1_048_576);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_calculate_directory_size_many_files() {
        let temp_dir = TempDir::new().unwrap();

        // 100個のファイル作成
        for i in 0..100 {
            let file = temp_dir.path().join(format!("file_{i}.txt"));
            std::fs::write(&file, format!("File {i}")).unwrap();
        }

        let result = calculate_directory_size(temp_dir.path());

        assert!(result.is_ok());
        let (size, count) = result.unwrap();
        assert!(size > 0);
        assert_eq!(count, 100);
    }

    #[cfg(unix)]
    #[test]
    fn test_get_disk_info_existing_path() {
        // /tmpは通常存在するため、これを使ってテスト
        let tmp_path = std::path::PathBuf::from("/tmp");
        let result = get_disk_info(&tmp_path);

        assert!(result.is_ok());

        if let Ok(Some((total, available))) = result {
            assert!(total > 0);
            assert!(available > 0);
            assert!(available <= total);
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_get_disk_info_nonexistent_path() {
        let nonexistent_path = std::path::PathBuf::from("/nonexistent/path/to/nowhere");
        let result = get_disk_info(&nonexistent_path);

        assert!(result.is_ok());

        if let Ok(disk_info) = result {
            assert!(disk_info.is_none());
        }
    }
}
