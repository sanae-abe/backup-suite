/// ダッシュボードUI モジュール
///
/// 統計情報、バックアップ履歴、エラー・警告サマリー、ディスク使用量の統合ビュー
use anyhow::Result;
use chrono::Utc;
use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, Color, ContentArrangement, Table};
use std::fs;

use super::colors::ColorTheme;
use super::table::display_history;
use crate::core::{BackupHistory, Config, Priority};

/// ダッシュボード表示
///
/// # Errors
///
/// 次の場合にエラーを返します:
/// - 設定ファイルやバックアップ履歴の読み込みに失敗した場合
/// - バックアップディレクトリの情報取得に失敗した場合
/// - ディスク情報の取得に失敗した場合（Unix系のみ）
pub fn display_dashboard() -> Result<()> {
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
    display_statistics(&theme)?;

    println!();

    // ディスク使用量グラフ
    display_disk_usage(&theme)?;

    println!();

    // 最近のバックアップ一覧
    display_recent_backups(&theme)?;

    println!();

    // エラー・警告サマリー
    display_warnings_summary(&theme)?;

    println!();

    Ok(())
}

/// 統計情報表示
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn display_statistics(theme: &ColorTheme) -> Result<()> {
    let config = Config::load()?;
    let history = BackupHistory::load_all()?;

    // 統計情報の計算
    let total_targets = config.targets.len();
    let high_priority = config.filter_by_priority(&Priority::High).len();
    let medium_priority = config.filter_by_priority(&Priority::Medium).len();
    let low_priority = config.filter_by_priority(&Priority::Low).len();

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
            format!("{}日前", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{}時間前", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{}分前", duration.num_minutes())
        } else {
            "たった今".to_string()
        }
    } else {
        "未実施".to_string()
    };

    println!("{}", theme.header().apply_to("📈 統計情報"));
    println!();

    // バックアップ対象の統計
    let mut targets_table = Table::new();
    targets_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    targets_table.add_row(vec![
        Cell::new("総対象数"),
        Cell::new(total_targets.to_string())
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
    ]);
    targets_table.add_row(vec![
        Cell::new("  高優先度"),
        Cell::new(high_priority.to_string())
            .fg(Color::Red)
            .set_alignment(CellAlignment::Right),
    ]);
    targets_table.add_row(vec![
        Cell::new("  中優先度"),
        Cell::new(medium_priority.to_string())
            .fg(Color::Yellow)
            .set_alignment(CellAlignment::Right),
    ]);
    targets_table.add_row(vec![
        Cell::new("  低優先度"),
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
        Cell::new("総バックアップ回数"),
        Cell::new(total_backups.to_string())
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
    ]);
    history_table.add_row(vec![
        Cell::new("  成功"),
        Cell::new(successful_backups.to_string())
            .fg(Color::Green)
            .set_alignment(CellAlignment::Right),
    ]);
    if failed_backups > 0 {
        history_table.add_row(vec![
            Cell::new("  失敗"),
            Cell::new(failed_backups.to_string())
                .fg(Color::Red)
                .set_alignment(CellAlignment::Right),
        ]);
    }
    history_table.add_row(vec![
        Cell::new("総ファイル数"),
        Cell::new(total_files.to_string())
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
    ]);
    history_table.add_row(vec![
        Cell::new("総データサイズ"),
        Cell::new(format_bytes(total_bytes))
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
    ]);
    history_table.add_row(vec![
        Cell::new("最終バックアップ"),
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
        Cell::new("暗号化バックアップ"),
        Cell::new(format!("{encrypted_backups} ({encryption_rate:.1}%)"))
            .fg(Color::Green)
            .set_alignment(CellAlignment::Right),
    ]);
    security_table.add_row(vec![
        Cell::new("圧縮バックアップ"),
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
fn display_disk_usage(theme: &ColorTheme) -> Result<()> {
    let config = Config::load()?;
    let backup_dir = &config.backup.destination;

    println!("{}", theme.header().apply_to("💾 ディスク使用量"));
    println!();

    // バックアップディレクトリのサイズを計算
    let (used_bytes, file_count) = calculate_directory_size(backup_dir)?;

    // ディスク全体の容量を取得（macOS/Linuxのみ）
    #[cfg(unix)]
    let disk_info = get_disk_info(backup_dir)?;

    #[cfg(not(unix))]
    let disk_info = None;

    let mut disk_table = Table::new();
    disk_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    disk_table.add_row(vec![
        Cell::new("バックアップディレクトリ"),
        Cell::new(backup_dir.display().to_string())
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Left),
    ]);

    disk_table.add_row(vec![
        Cell::new("使用容量"),
        Cell::new(format_bytes(used_bytes))
            .fg(Color::Yellow)
            .set_alignment(CellAlignment::Right),
    ]);

    disk_table.add_row(vec![
        Cell::new("ファイル数"),
        Cell::new(file_count.to_string())
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
    ]);

    #[cfg(unix)]
    if let Some((total, available)) = disk_info {
        let used_percent = ((total - available) as f64 / total as f64) * 100.0;

        disk_table.add_row(vec![
            Cell::new("ディスク総容量"),
            Cell::new(format_bytes(total))
                .fg(Color::Cyan)
                .set_alignment(CellAlignment::Right),
        ]);

        disk_table.add_row(vec![
            Cell::new("ディスク空き容量"),
            Cell::new(format_bytes(available))
                .fg(if available < total / 10 {
                    Color::Red
                } else {
                    Color::Green
                })
                .set_alignment(CellAlignment::Right),
        ]);

        disk_table.add_row(vec![
            Cell::new("ディスク使用率"),
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
            Cell::new("使用状況"),
            Cell::new(graph)
                .fg(Color::Cyan)
                .set_alignment(CellAlignment::Left),
        ]);
    }

    println!("{disk_table}");

    Ok(())
}

/// ディレクトリサイズを計算
fn calculate_directory_size(dir: &std::path::Path) -> Result<(u64, usize)> {
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
fn get_disk_info(path: &std::path::Path) -> Result<Option<(u64, u64)>> {
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
    let result = unsafe { libc::statfs(path_cstr.as_ptr(), &mut stat) };

    if result == 0 {
        let block_size = stat.f_bsize as u64;
        let total_blocks = stat.f_blocks as u64;
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
fn create_usage_graph(percent: f64) -> String {
    let total_bars = 40;
    let filled_bars = ((percent / 100.0) * total_bars as f64) as usize;
    let empty_bars = total_bars - filled_bars;

    let filled = "█".repeat(filled_bars);
    let empty = "░".repeat(empty_bars);

    format!("[{filled}{empty}] {percent:.1}%")
}

/// 最近のバックアップ一覧（直近5件）
fn display_recent_backups(theme: &ColorTheme) -> Result<()> {
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
        theme.header().apply_to("🕒 最近のバックアップ（直近5件）")
    );
    display_history(&recent, theme);

    Ok(())
}

/// エラー・警告サマリー
#[allow(clippy::cast_precision_loss)]
fn display_warnings_summary(theme: &ColorTheme) -> Result<()> {
    let config = Config::load()?;
    let mut warnings = Vec::new();

    // バックアップ対象が存在しない場合の警告
    for target in &config.targets {
        if !target.path.exists() {
            warnings.push(format!(
                "バックアップ対象が存在しません: {}",
                target.path.display()
            ));
        }
    }

    // 最近のバックアップがない場合の警告
    let history = BackupHistory::load_all()?;
    if let Some(last) = history.last() {
        let days_since = Utc::now().signed_duration_since(last.timestamp).num_days();

        if days_since > 7 {
            warnings.push(format!(
                "最後のバックアップから{days_since}日経過しています"
            ));
        }
    } else {
        warnings.push("まだ一度もバックアップが実行されていません".to_string());
    }

    // 失敗したバックアップの警告
    let failed_count = history.iter().filter(|h| !h.success).count();
    if failed_count > 0 {
        warnings.push(format!("失敗したバックアップが{failed_count}件あります"));
    }

    // ディスク容量警告
    #[cfg(unix)]
    {
        if let Ok(Some((total, available))) = get_disk_info(&config.backup.destination) {
            let available_percent = (available as f64 / total as f64) * 100.0;
            if available_percent < 10.0 {
                warnings.push(format!(
                    "ディスク空き容量が少なくなっています ({available_percent:.1}%)"
                ));
            }
        }
    }

    // 警告表示
    if warnings.is_empty() {
        println!("{}", theme.success().apply_to("⚡ すべて正常です"));
    } else {
        println!("{}", theme.header().apply_to("⚠️  警告・注意事項"));
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

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_create_usage_graph() {
        let graph = create_usage_graph(50.0);
        assert!(graph.contains("50.0%"));
        assert!(graph.contains("█"));
        assert!(graph.contains("░"));
    }
}
