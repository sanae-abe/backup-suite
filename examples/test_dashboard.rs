/// ダッシュボード機能のテスト例
///
/// 使用方法:
/// ```bash
/// cargo run --example test_dashboard
/// ```

use anyhow::Result;
use backup_suite::ui::display_dashboard;

fn main() -> Result<()> {
    println!("=== Backup Suite Dashboard Test ===\n");

    // ダッシュボード表示
    display_dashboard()?;

    Ok(())
}
