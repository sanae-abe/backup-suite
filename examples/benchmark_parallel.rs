use rayon::prelude::*;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let source_dir = PathBuf::from(std::env::var("HOME").unwrap()).join("backup-test/files");
    let dest_dir = PathBuf::from(std::env::var("HOME").unwrap()).join("backup-test/rust-backup");

    // 前回の結果を削除
    let _ = std::fs::remove_dir_all(&dest_dir);
    std::fs::create_dir_all(&dest_dir).expect("dest_dir作成失敗");

    // ソースファイル一覧取得
    let files: Vec<PathBuf> = std::fs::read_dir(&source_dir)
        .expect("source_dir読み込み失敗")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();

    println!("=== Rust版ベンチマーク（並列処理・rayon） ===");
    println!("対象: {} ファイル", files.len());

    let start = Instant::now();

    // 並列コピー
    let results: Vec<_> = files
        .par_iter()
        .map(|file| {
            let filename = file.file_name().unwrap();
            let dest = dest_dir.join(filename);
            std::fs::copy(file, &dest)
        })
        .collect();

    let elapsed = start.elapsed();
    let success = results.iter().filter(|r| r.is_ok()).count();

    println!("完了: {success} ファイル");
    println!("実行時間: {:.3}秒", elapsed.as_secs_f64());

    // 後片付け
    let _ = std::fs::remove_dir_all(&dest_dir);
}
