/// ダッシュボードUI モジュール
///
/// 統計情報、バックアップ履歴、エラー・警告サマリーの統合ビュー
use anyhow::Result;
use chrono::Utc;
use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, Color, ContentArrangement, Table};

use super::colors::ColorTheme;
use super::table::display_history;
use crate::core::{BackupHistory, Config, Priority};

/// ダッシュボード表示
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

    // 最近のバックアップ一覧
    display_recent_backups(&theme)?;

    println!();

    // エラー・警告サマリー
    display_warnings_summary(&theme)?;

    println!();

    Ok(())
}

/// 統計情報表示
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

    // テーブル作成
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

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

    println!("{}", targets_table);
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

    println!("{}", history_table);

    Ok(())
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
                "最後のバックアップから{}日経過しています",
                days_since
            ));
        }
    } else {
        warnings.push("まだ一度もバックアップが実行されていません".to_string());
    }

    // 失敗したバックアップの警告
    let failed_count = history.iter().filter(|h| !h.success).count();
    if failed_count > 0 {
        warnings.push(format!("失敗したバックアップが{}件あります", failed_count));
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

        println!("{}", table);
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
    }
}
