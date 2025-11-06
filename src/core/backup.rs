use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

use super::copy_engine::CopyEngine;
use super::filter::FileFilter;
use super::pipeline::{ProcessingPipeline, PipelineConfig};
use super::{Config, Priority, Target, TargetType};
use crate::compression::CompressionType;
use crate::crypto::{EncryptionConfig, KeyManager};
use crate::security::safe_join;
use crate::ui::progress::BackupProgress;

/// バックアップ実行結果
///
/// バックアップ処理の結果とエラー情報を保持します。
///
/// # フィールド
///
/// * `total_files` - 処理対象の総ファイル数
/// * `successful` - 成功したファイル数
/// * `failed` - 失敗したファイル数
/// * `total_bytes` - コピーした総バイト数
/// * `errors` - エラーメッセージのリスト
/// * `backup_name` - 作成されたバックアップディレクトリ名
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::{Config, BackupRunner};
///
/// let config = Config::load().unwrap();
/// let runner = BackupRunner::new(config, false);
/// let result = runner.run(None, None).unwrap();
///
/// if result.failed > 0 {
///     eprintln!("エラー: {}件のファイルでバックアップ失敗", result.failed);
///     for error in &result.errors {
///         eprintln!("  {}", error);
///     }
/// }
/// println!("✓ 成功: {}件 ({}バイト)", result.successful, result.total_bytes);
/// ```
#[derive(Debug)]
pub struct BackupResult {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub total_bytes: u64,
    pub errors: Vec<String>,
    pub backup_name: String,
}

impl BackupResult {
    fn new() -> Self {
        Self {
            total_files: 0,
            successful: 0,
            failed: 0,
            total_bytes: 0,
            errors: Vec::new(),
            backup_name: String::new(),
        }
    }
}

/// バックアップ実行エンジン
///
/// 設定に基づいてバックアップを並列実行します。
/// ドライランモード、進捗表示、優先度フィルタリングをサポートします。
///
/// # フィールド
///
/// * `config` - バックアップ設定
/// * `dry_run` - ドライランモード（実際のコピーを行わない）
/// * `show_progress` - 進捗バーの表示有無
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::{Config, BackupRunner, Priority};
///
/// // 基本的なバックアップ実行
/// let config = Config::load().unwrap();
/// let runner = BackupRunner::new(config, false);
/// let result = runner.run(None, None).unwrap();
///
/// // 高優先度のみ実行
/// let config = Config::load().unwrap();
/// let runner = BackupRunner::new(config, false)
///     .with_progress(true);
/// let result = runner.run(Some(&Priority::High), None).unwrap();
/// ```
pub struct BackupRunner {
    config: Config,
    dry_run: bool,
    show_progress: bool,
    enable_encryption: bool,
    password: Option<String>,
    compression_type: CompressionType,
    compression_level: i32,
}

impl BackupRunner {
    /// 新しいBackupRunnerを作成
    ///
    /// # 引数
    ///
    /// * `config` - バックアップ設定
    /// * `dry_run` - `true` の場合、実際のコピーを行わず処理対象のみを表示
    ///
    /// # 戻り値
    ///
    /// 進捗表示が有効な BackupRunner インスタンス
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::{Config, BackupRunner};
    ///
    /// let config = Config::load().unwrap();
    /// let runner = BackupRunner::new(config, false);
    /// ```
    pub fn new(config: Config, dry_run: bool) -> Self {
        Self {
            config,
            dry_run,
            show_progress: true, // デフォルトで進捗表示を有効化
            enable_encryption: false,
            password: None,
            compression_type: CompressionType::Zstd,
            compression_level: 3,
        }
    }

    /// 進捗表示の有効/無効を設定
    ///
    /// # 引数
    ///
    /// * `show_progress` - `true` で進捗バーを表示、`false` で非表示
    ///
    /// # 戻り値
    ///
    /// 設定を更新した BackupRunner インスタンス
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::{Config, BackupRunner};
    ///
    /// let config = Config::load().unwrap();
    /// let runner = BackupRunner::new(config, false)
    ///     .with_progress(false); // 進捗表示を無効化
    /// ```
    pub fn with_progress(mut self, show_progress: bool) -> Self {
        self.show_progress = show_progress;
        self
    }

    /// 暗号化を有効化
    pub fn with_encryption(mut self, password: String) -> Self {
        self.enable_encryption = true;
        self.password = Some(password);
        self
    }

    /// 圧縮設定
    pub fn with_compression(mut self, compression_type: CompressionType, level: i32) -> Self {
        self.compression_type = compression_type;
        self.compression_level = level;
        self
    }

    /// バックアップを実行
    ///
    /// 設定に基づいて並列バックアップを実行します。
    /// 優先度フィルタを指定することで、特定の優先度のファイルのみをバックアップできます。
    ///
    /// # 引数
    ///
    /// * `priority_filter` - バックアップ対象の優先度（`None` で全優先度）
    ///
    /// # 戻り値
    ///
    /// 成功時は `Ok(BackupResult)` でバックアップ結果、失敗時はエラー
    ///
    /// # エラー
    ///
    /// * バックアップディレクトリの作成に失敗した場合
    /// * 設定の検証に失敗した場合
    /// * ファイルコピーで致命的なエラーが発生した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::{Config, BackupRunner, Priority};
    ///
    /// let config = Config::load().unwrap();
    /// let runner = BackupRunner::new(config, false);
    ///
    /// // 全ファイルをバックアップ
    /// let result = runner.run(None, None).unwrap();
    ///
    /// // 高優先度のみバックアップ
    /// let config = Config::load().unwrap();
    /// let runner = BackupRunner::new(config, false);
    /// let result = runner.run(Some(&Priority::High), None).unwrap();
    /// ```
    pub fn run(&self, priority_filter: Option<&Priority>, category_filter: Option<&str>) -> Result<BackupResult> {
        // バックアップ対象をフィルタ（優先度 → カテゴリの順）
        let mut targets: Vec<&Target> = if let Some(priority) = priority_filter {
            self.config.filter_by_priority(priority)
        } else {
            self.config.targets.iter().collect()
        };

        // カテゴリフィルタの適用
        if let Some(category) = category_filter {
            targets.retain(|t| t.category == category);
        }

        if targets.is_empty() {
            return Ok(BackupResult::new());
        }

        // バックアップ先ディレクトリの準備（バックアップ名/カテゴリ階層構造）
        let dest_base = &self.config.backup.destination;
        let now = chrono::Local::now();
        let timestamp = now.format("%Y%m%d_%H%M%S");
        let backup_name = format!("backup_{}", timestamp);
        let backup_base = dest_base.join(&backup_name);

        // 暗号化が有効な場合、KeyManagerとmaster keyを準備
        let (_key_manager, master_key, encryption_salt) = if self.enable_encryption && self.password.is_some() {
            let km = KeyManager::default();
            let password = self.password.as_ref().unwrap();
            let (mk, salt) = km.create_master_key(password)
                .context("マスターキー生成失敗")?;
            (Some(km), Some(Arc::new(mk)), Some(salt))
        } else {
            (None, None, None)
        };

        // 各ターゲットからファイルリストを収集
        let mut all_files: Vec<(PathBuf, PathBuf)> = Vec::new();

        // スピナー表示（ファイル収集中）
        let collection_spinner = if self.show_progress {
            let spinner = BackupProgress::new_spinner();
            spinner.set_message("バックアップ対象ファイルを収集中...");
            Some(spinner)
        } else {
            None
        };

        for target in &targets {
            // 各ターゲットのカテゴリをディレクトリ名に使用
            // （カテゴリフィルタは221-223行で既に適用済み）
            let category = target.category.clone();
            let backup_dir = backup_base.join(&category);

            // カテゴリディレクトリを作成
            std::fs::create_dir_all(&backup_dir)
                .context(format!("バックアップディレクトリ作成失敗: {:?}", backup_dir))?;

            // FileFilterの準備
            let filter = if !target.exclude_patterns.is_empty() {
                match FileFilter::new(&target.exclude_patterns) {
                    Ok(f) => Some(f),
                    Err(e) => {
                        eprintln!("警告: 除外パターンの処理に失敗: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            match target.target_type {
                TargetType::File => {
                    if target.path.exists() {
                        // 除外フィルタチェック
                        if let Some(ref f) = filter {
                            if f.should_exclude(&target.path) {
                                continue;
                            }
                        }

                        // ファイル名を安全に取得してバックアップ先を決定
                        if let Some(file_name) = target.path.file_name() {
                            // safe_joinを使用してディレクトリトラバーサル対策
                            match safe_join(&backup_dir, std::path::Path::new(file_name)) {
                                Ok(dest) => all_files.push((target.path.clone(), dest)),
                                Err(e) => eprintln!("警告: ファイルパス処理エラー: {}", e),
                            }
                        }
                    }
                }
                TargetType::Directory => {
                    for entry in WalkDir::new(&target.path)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        if entry.file_type().is_file() {
                            let source = entry.path().to_path_buf();

                            // 相対パスを保持してバックアップ先を決定（セキュリティ強化版）
                            match source.strip_prefix(&target.path) {
                                Ok(relative) => {
                                    // 除外フィルタチェック（相対パスに対して）
                                    if let Some(ref f) = filter {
                                        if f.should_exclude(relative) {
                                            continue;
                                        }
                                    }

                                    // safe_joinを使用してディレクトリトラバーサル対策
                                    match safe_join(&backup_dir, relative) {
                                        Ok(dest) => all_files.push((source, dest)),
                                        Err(e) => eprintln!("警告: パストラバーサル検出、スキップ: {}", e),
                                    }
                                }
                                Err(e) => {
                                    eprintln!("警告: パスのstrip_prefixに失敗: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }

        // スピナー完了
        if let Some(spinner) = collection_spinner {
            spinner.finish(&format!("{}ファイルを検出", all_files.len()));
        }

        let total_files = all_files.len();

        if self.dry_run {
            println!("📋 ドライランモード: {} ファイルをバックアップ対象として検出", total_files);
            for (source, dest) in &all_files {
                println!("  {:?} → {:?}", source, dest);
            }
            return Ok(BackupResult {
                total_files,
                successful: 0,
                failed: 0,
                total_bytes: 0,
                errors: Vec::new(),
                backup_name,
            });
        }

        // ProcessingPipelineの作成（暗号化または圧縮が有効な場合）
        let pipeline = if self.enable_encryption || self.compression_type != CompressionType::None {
            // CompressionConfigを作成（compression_typeに応じたデフォルトからlevelを変更）
            let mut compression_config = match self.compression_type {
                CompressionType::Zstd => crate::compression::CompressionConfig::zstd_default(),
                CompressionType::Gzip => crate::compression::CompressionConfig::gzip_default(),
                CompressionType::None => crate::compression::CompressionConfig::none(),
            };
            compression_config.level = self.compression_level;

            let mut config = PipelineConfig::default()
                .with_compression(self.compression_type, compression_config);

            if self.enable_encryption {
                config = config.with_encryption(EncryptionConfig::default());
            }

            Some(Arc::new(ProcessingPipeline::new(config)))
        } else {
            None
        };

        // プログレスバーの初期化
        let progress = if self.show_progress {
            Some(Arc::new(BackupProgress::new(total_files as u64)))
        } else {
            None
        };

        // CopyEngineの初期化（I/O最適化）
        let copy_engine = Arc::new(CopyEngine::new());

        // 並列バックアップ処理
        let success_count = AtomicUsize::new(0);
        let failed_count = AtomicUsize::new(0);
        let total_bytes = AtomicUsize::new(0);

        let errors: Vec<String> = all_files.par_iter()
            .filter_map(|(source, dest)| {
                // 進捗表示更新
                if let Some(ref pb) = progress {
                    if let Some(file_name) = source.file_name() {
                        pb.set_message(&format!("処理中: {:?}", file_name));
                    }
                }

                // バックアップ先のディレクトリを作成
                if let Some(parent) = dest.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        failed_count.fetch_add(1, Ordering::Relaxed);
                        if let Some(ref pb) = progress {
                            pb.inc(1);
                        }
                        return Some(format!("ディレクトリ作成失敗 {:?}: {}", parent, e));
                    }
                }

                // ProcessingPipelineまたはCopyEngineでファイル処理
                if let Some(ref pipeline) = pipeline {
                    // 暗号化・圧縮パイプライン使用
                    match pipeline.process_file(source, master_key.as_ref().map(|k| k.as_ref()), encryption_salt) {
                        Ok(processed) => {
                            // 処理後のデータをファイルに書き込み
                            match std::fs::write(dest, &processed.data) {
                                Ok(_) => {
                                    success_count.fetch_add(1, Ordering::Relaxed);
                                    total_bytes.fetch_add(processed.metadata.final_size as usize, Ordering::Relaxed);
                                    if let Some(ref pb) = progress {
                                        pb.inc(1);
                                    }
                                    None
                                }
                                Err(e) => {
                                    failed_count.fetch_add(1, Ordering::Relaxed);
                                    if let Some(ref pb) = progress {
                                        pb.inc(1);
                                    }
                                    Some(format!("書き込み失敗 {:?}: {}", dest, e))
                                }
                            }
                        }
                        Err(e) => {
                            failed_count.fetch_add(1, Ordering::Relaxed);
                            if let Some(ref pb) = progress {
                                pb.inc(1);
                            }
                            Some(format!("処理失敗 {:?}: {}", source, e))
                        }
                    }
                } else {
                    // 従来のCopyEngine使用（暗号化・圧縮なし）
                    match copy_engine.copy_file(source, dest) {
                        Ok(bytes) => {
                            success_count.fetch_add(1, Ordering::Relaxed);
                            total_bytes.fetch_add(bytes as usize, Ordering::Relaxed);
                            if let Some(ref pb) = progress {
                                pb.inc(1);
                            }
                            None
                        }
                        Err(e) => {
                            failed_count.fetch_add(1, Ordering::Relaxed);
                            if let Some(ref pb) = progress {
                                pb.inc(1);
                            }
                            Some(format!("コピー失敗 {:?}: {}", source, e))
                        }
                    }
                }
            })
            .collect();

        // プログレスバー完了
        if let Some(pb) = progress {
            let success = failed_count.load(Ordering::Relaxed);
            if success == 0 {
                pb.finish("✓ バックアップ完了");
            } else {
                pb.finish(&format!("⚠ バックアップ完了（{}件失敗）", success));
            }
        }

        let result = BackupResult {
            total_files,
            successful: success_count.load(Ordering::Relaxed),
            failed: failed_count.load(Ordering::Relaxed),
            total_bytes: total_bytes.load(Ordering::Relaxed) as u64,
            errors,
            backup_name,
        };

        // 履歴保存（バックアップ全体のベースディレクトリを使用）
        let success = result.failed == 0;
        if let Err(e) = super::BackupHistory::save(&super::BackupHistory::new(
            backup_base.clone(),
            result.total_files,
            result.total_bytes,
            success,
        )) {
            eprintln!("履歴保存失敗: {}", e);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_backup_single_file() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("test.txt");
        let mut file = File::create(&source).unwrap();
        file.write_all(b"test content").unwrap();

        let mut config = Config::default();
        let target = Target::new(source.clone(), Priority::High, "test".to_string());
        config.add_target(target);
        config.backup.destination = temp.path().join("backups");

        let runner = BackupRunner::new(config, false);
        let result = runner.run(None, None).unwrap();

        assert_eq!(result.total_files, 1);
        assert_eq!(result.successful, 1);
        assert_eq!(result.failed, 0);
        assert!(result.total_bytes > 0);
    }

    #[test]
    fn test_backup_dry_run() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("test.txt");
        File::create(&source).unwrap();

        let mut config = Config::default();
        let target = Target::new(source.clone(), Priority::High, "test".to_string());
        config.add_target(target);
        config.backup.destination = temp.path().join("backups");

        let runner = BackupRunner::new(config, true);
        let result = runner.run(None, None).unwrap();

        assert_eq!(result.total_files, 1);
        assert_eq!(result.successful, 0); // ドライランなので実行なし
        assert_eq!(result.total_bytes, 0);
    }
}
