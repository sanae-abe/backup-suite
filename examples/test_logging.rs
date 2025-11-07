/// ログ機能のテスト例
///
/// 使用方法:
/// ```bash
/// cargo run --example test_logging
/// ```

use anyhow::Result;
use backup_suite::core::logging::{LogFormat, LogLevel, Logger};

fn main() -> Result<()> {
    println!("=== Backup Suite Logging Test ===\n");

    // テキスト形式のロガー
    println!("1. テキスト形式ロガー:");
    let text_logger = Logger::new(LogLevel::Info, LogFormat::Text)?;
    println!("ログファイル: {:?}\n", text_logger.log_file_path());

    text_logger.info("バックアップ開始");
    text_logger.warn("警告: ディスク容量が少なくなっています");
    text_logger.error("エラー: ファイルのコピーに失敗しました");
    text_logger.debug("このログは表示されません（InfoレベルのためDebugは非表示）");

    println!("✓ テキストログを書き込みました\n");

    // JSON形式のロガー
    println!("2. JSON形式ロガー:");
    let json_logger = Logger::new(LogLevel::Debug, LogFormat::Json)?;

    json_logger.info("JSON形式のログテスト");
    json_logger.debug("デバッグ情報（Debugレベルなので表示）");

    // メタデータ付きログ
    let metadata = serde_json::json!({
        "files": 150,
        "bytes": 1048576,
        "duration_ms": 5432
    });
    json_logger.log_with_metadata(LogLevel::Info, "バックアップ完了", metadata);

    println!("✓ JSONログを書き込みました\n");

    println!("ログファイルは以下の場所に保存されています:");
    println!("  - macOS: ~/Library/Logs/backup-suite/backup.log");
    println!("  - Linux: ~/.local/share/backup-suite/logs/backup.log\n");

    println!("確認コマンド:");
    println!("  tail ~/Library/Logs/backup-suite/backup.log");

    Ok(())
}
