/// 進捗表示のテスト例
///
/// 使用方法:
/// ```bash
/// cargo run --example test_progress
/// ```

use backup_suite::ui::progress::BackupProgress;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    println!("=== Backup Suite Progress Test ===\n");

    // テスト1: 基本的なプログレスバー
    println!("1. 基本的なプログレスバー:");
    let progress = BackupProgress::new(100);

    let start = Instant::now();
    for i in 0..100 {
        progress.set_message(&format!("処理中: file_{:03}.txt", i + 1));

        // 統計情報の更新
        let elapsed = start.elapsed().as_secs_f64();
        let processed_files = i + 1;
        let total_bytes = ((processed_files as u64) * 1024 * 1024) as u64; // 1MBずつ増加
        progress.update_stats(processed_files as u64, total_bytes, elapsed);

        progress.inc(1);
        thread::sleep(Duration::from_millis(50));
    }
    progress.finish("✓ バックアップ完了");

    thread::sleep(Duration::from_millis(500));

    // テスト2: スピナー（不定期間処理）
    println!("\n2. スピナー（ファイル検索中）:");
    let spinner = BackupProgress::new_spinner();
    spinner.set_message("ファイルを検索中...");

    for i in 0..30 {
        spinner.set_stats(&format!("検出: {} ファイル", i * 5));
        thread::sleep(Duration::from_millis(100));
    }
    spinner.finish("✓ 検索完了: 150 ファイル");

    thread::sleep(Duration::from_millis(500));

    // テスト3: 大容量ファイルのシミュレーション
    println!("\n3. 大容量ファイル処理:");
    let progress = BackupProgress::new(50);
    progress.set_main_message("高優先度");

    let start = Instant::now();
    for i in 0..50 {
        let file_size_mb = ((i % 10) + 1) * 50; // 50MB～500MB
        progress.set_message(&format!(
            "処理中: large_file_{:02}.dat ({} MB)",
            i + 1,
            file_size_mb
        ));

        let elapsed = start.elapsed().as_secs_f64();
        let total_bytes = ((i + 1) as u64 * 100 * 1024 * 1024) as u64; // 平均100MB
        progress.update_stats((i + 1) as u64, total_bytes, elapsed);

        progress.inc(1);
        thread::sleep(Duration::from_millis(100));
    }
    progress.finish("✓ 全ファイル処理完了");

    println!("\n=== テスト完了 ===");
}
