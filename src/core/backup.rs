use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

use super::copy_engine::CopyEngine;
use super::filter::FileFilter;
use super::incremental::{BackupType, IncrementalBackupEngine};
use super::integrity::IntegrityChecker;
use super::pipeline::{PipelineConfig, ProcessingPipeline};
use super::{Config, Priority, Target, TargetType};
use crate::compression::CompressionType;
use crate::crypto::{EncryptionConfig, KeyManager};
use crate::i18n::{get_message, MessageKey};
use crate::security::{safe_join, AuditEvent, AuditLog};
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
/// let mut runner = BackupRunner::new(config, false);
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
/// let mut runner = BackupRunner::new(config, false);
/// let result = runner.run(None, None).unwrap();
///
/// // 高優先度のみ実行
/// let config = Config::load().unwrap();
/// let mut runner = BackupRunner::new(config, false)
///     .with_progress(true);
/// let result = runner.run(Some(&Priority::High), None).unwrap();
/// ```
#[allow(clippy::struct_excessive_bools)]
pub struct BackupRunner {
    config: Config,
    dry_run: bool,
    show_progress: bool,
    enable_encryption: bool,
    password: Option<String>,
    compression_type: CompressionType,
    compression_level: i32,
    verify_integrity: bool,
    audit_log: Option<AuditLog>,
    incremental: bool,
    lang: crate::i18n::Language,
}

impl BackupRunner {
    /// 新しい`BackupRunner`を作成
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
    /// let mut runner = BackupRunner::new(config, false);
    /// ```
    #[must_use]
    pub fn new(config: Config, dry_run: bool) -> Self {
        // 監査ログの初期化（失敗してもバックアップ処理は継続）
        let audit_log = AuditLog::new()
            .map_err(|e| eprintln!("警告: 監査ログの初期化に失敗しました: {e}"))
            .ok();

        Self {
            config,
            dry_run,
            show_progress: true, // デフォルトで進捗表示を有効化
            enable_encryption: false,
            password: None,
            compression_type: CompressionType::Zstd,
            compression_level: 3,
            verify_integrity: true, // デフォルトで整合性検証を有効化
            audit_log,
            incremental: false,
            lang: crate::i18n::Language::detect(),
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
    /// let mut runner = BackupRunner::new(config, false)
    ///     .with_progress(false); // 進捗表示を無効化
    /// ```
    #[must_use]
    pub fn with_progress(mut self, show_progress: bool) -> Self {
        self.show_progress = show_progress;
        self
    }

    /// 暗号化を有効化
    #[must_use]
    pub fn with_encryption(mut self, password: String) -> Self {
        self.enable_encryption = true;
        self.password = Some(password);
        self
    }

    /// 圧縮設定
    #[must_use]
    pub fn with_compression(mut self, compression_type: CompressionType, level: i32) -> Self {
        self.compression_type = compression_type;
        self.compression_level = level;
        self
    }

    /// 整合性検証の有効/無効を設定
    #[must_use]
    pub fn with_verification(mut self, verify: bool) -> Self {
        self.verify_integrity = verify;
        self
    }

    /// 増分バックアップを有効化
    #[must_use]
    pub fn with_incremental(mut self, incremental: bool) -> Self {
        self.incremental = incremental;
        self
    }

    /// 言語を設定
    #[must_use]
    pub fn with_language(mut self, lang: crate::i18n::Language) -> Self {
        self.lang = lang;
        self
    }

    /// バックアップを実行
    ///
    /// 設定に基づいて並列バックアップを実行します。
    /// 優先度フィルタを指定することで、特定の優先度のファイルのみをバックアップできます。
    ///
    /// # 引数
    ///
    /// * `priority_filter` - バックアップ対象の優先度（None で全優先度）
    ///
    /// # 戻り値
    ///
    /// 成功時は `Ok(BackupResult)` でバックアップ結果、失敗時はエラー
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// * バックアップディレクトリの作成に失敗した場合
    /// * マスターキー生成に失敗した場合（暗号化有効時）
    /// * バックアップタイプの決定に失敗した場合（増分バックアップ有効時）
    /// * 前回のメタデータ読み込みに失敗した場合（増分バックアップ時）
    /// * 整合性メタデータの保存に失敗した場合
    /// * バックアップ履歴の保存に失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::{Config, BackupRunner, Priority};
    ///
    /// let config = Config::load().unwrap();
    /// let mut runner = BackupRunner::new(config, false);
    ///
    /// // 全ファイルをバックアップ
    /// let result = runner.run(None, None).unwrap();
    ///
    /// // 高優先度のみバックアップ
    /// let config = Config::load().unwrap();
    /// let mut runner = BackupRunner::new(config, false);
    /// let result = runner.run(Some(&Priority::High), None).unwrap();
    /// ```
    pub fn run(
        &mut self,
        priority_filter: Option<&Priority>,
        category_filter: Option<&str>,
    ) -> Result<BackupResult> {
        let user = AuditLog::current_user();
        let target_desc = format!("priority={priority_filter:?}, category={category_filter:?}");

        // 監査ログ: バックアップ開始
        if let Some(ref mut audit_log) = self.audit_log {
            let _ = audit_log
                .log(AuditEvent::backup_started(&target_desc, &user))
                .map_err(|e| eprintln!("警告: 監査ログの記録に失敗しました: {e}"));
        }

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
        let backup_name = format!("backup_{timestamp}");
        let backup_base = dest_base.join(&backup_name);

        // 暗号化が有効な場合、KeyManagerとmaster keyを準備
        let (_key_manager, master_key, encryption_salt) =
            if self.enable_encryption && self.password.is_some() {
                let km = KeyManager::default();
                let password = self.password.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("暗号化が有効ですがパスワードが設定されていません")
                })?;
                let (mk, salt) = km
                    .create_master_key(password)
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
            spinner.set_message("Collecting backup target files...");
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
                .context("バックアップディレクトリ作成失敗: backup_dir.display()".to_string())?;

            // FileFilterの準備
            let filter = if !target.exclude_patterns.is_empty() {
                match FileFilter::new(&target.exclude_patterns) {
                    Ok(f) => Some(f),
                    Err(e) => {
                        eprintln!("警告: 除外パターンの処理に失敗: {e}");
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

                        // ファイルサイズチェック（100GB超の警告）
                        if let Ok(metadata) = target.path.metadata() {
                            let file_size = metadata.len();
                            const LARGE_FILE_THRESHOLD: u64 = 100 * 1024 * 1024 * 1024; // 100GB

                            if file_size > LARGE_FILE_THRESHOLD {
                                eprintln!(
                                    "⚠️  警告: 大容量ファイル検出 ({}GB): {:?}",
                                    file_size / (1024 * 1024 * 1024),
                                    target.path
                                );
                                eprintln!("    メモリ不足のリスクがあります。処理を続行しますが、システム監視を推奨します。");
                            }
                        }

                        // ファイル名を安全に取得してバックアップ先を決定
                        if let Some(file_name) = target.path.file_name() {
                            // safe_joinを使用してディレクトリトラバーサル対策
                            match safe_join(&backup_dir, std::path::Path::new(file_name)) {
                                Ok(dest) => all_files.push((target.path.clone(), dest)),
                                Err(e) => eprintln!("警告: ファイルパス処理エラー: {e}"),
                            }
                        }
                    }
                }
                TargetType::Directory => {
                    // ディレクトリ名を保持するため、親ディレクトリを基準にする
                    // 例: ~/.ssh をバックアップする場合、~ を基準にして .ssh/id_rsa のような相対パスを作成
                    let base_path = target.path.parent().unwrap_or(&target.path);

                    for entry in WalkDir::new(&target.path)
                        .into_iter()
                        .filter_map(std::result::Result::ok)
                    {
                        if entry.file_type().is_file() {
                            let source = entry.path().to_path_buf();

                            // ファイルサイズチェック（100GB超の警告）
                            if let Ok(metadata) = entry.metadata() {
                                let file_size = metadata.len();
                                const LARGE_FILE_THRESHOLD: u64 = 100 * 1024 * 1024 * 1024; // 100GB

                                if file_size > LARGE_FILE_THRESHOLD {
                                    eprintln!(
                                        "⚠️  警告: 大容量ファイル検出 ({}GB): {:?}",
                                        file_size / (1024 * 1024 * 1024),
                                        source
                                    );
                                    eprintln!("    メモリ不足のリスクがあります。処理を続行しますが、システム監視を推奨します。");
                                }
                            }

                            // ディレクトリ名を含めた相対パスを保持してバックアップ先を決定
                            match source.strip_prefix(base_path) {
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
                                        Err(e) => {
                                            eprintln!("警告: パストラバーサル検出、スキップ: {e}")
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("警告: パスのstrip_prefixに失敗: {e}");
                                }
                            }
                        }
                    }
                }
            }
        }

        // スピナー完了
        if let Some(spinner) = collection_spinner {
            spinner.finish(&format!(
                "{} {}",
                all_files.len(),
                get_message(MessageKey::FilesDetected, self.lang)
            ));
        }

        // 増分バックアップ処理
        let inc_engine = IncrementalBackupEngine::new(dest_base.clone());
        let backup_type = if self.incremental {
            inc_engine.determine_backup_type()?
        } else {
            BackupType::Full
        };

        // 増分バックアップの場合、前回のメタデータを読み込み（失敗した場合はフルバックアップにフォールバック）
        let (actual_backup_type, parent_backup_name, files_to_backup) =
            if backup_type == BackupType::Incremental {
                match inc_engine.load_previous_metadata() {
                    Ok(previous_metadata) => {
                        println!(
                            "{}",
                            get_message(MessageKey::IncrementalBackupMode, self.lang)
                        );

                        // バックアップディレクトリからの相対パスを計算
                        let files_with_relative: Vec<(PathBuf, PathBuf)> = all_files
                            .iter()
                            .filter_map(|(source, dest)| {
                                dest.strip_prefix(&backup_base)
                                    .ok()
                                    .map(|rel| (rel.to_path_buf(), source.clone()))
                            })
                            .collect();

                        let changed_files_relative = inc_engine
                            .detect_changed_files(&files_with_relative, &previous_metadata)?;

                        // 元のall_files形式に戻す（source, dest）
                        let changed_files: Vec<(PathBuf, PathBuf)> = changed_files_relative
                            .iter()
                            .filter_map(|(_relative_path, source_path)| {
                                all_files
                                    .iter()
                                    .find(|(src, _)| src == source_path)
                                    .cloned()
                            })
                            .collect();

                        let parent_name = inc_engine.get_previous_backup_name()?;
                        println!(
                            "  {}: {parent_name:?}",
                            get_message(MessageKey::PreviousBackupLabel, self.lang)
                        );
                        println!(
                            "  {}: {}/{}",
                            get_message(MessageKey::ChangedFilesLabel, self.lang),
                            changed_files.len(),
                            all_files.len()
                        );

                        (BackupType::Incremental, parent_name, changed_files)
                    }
                    Err(e) => {
                        // エラーメッセージの内容で初回実行時か実際のエラーかを判別
                        let error_msg = e.to_string();
                        if error_msg.contains("前回のバックアップが見つかりません")
                            || error_msg.contains("前回のバックアップメタデータ読み込み失敗")
                        {
                            // 初回実行時: 情報レベルのメッセージ
                            println!("{}", get_message(MessageKey::NoBackupsFound, self.lang));
                        } else {
                            // 実際のエラー時（メタデータ破損など）: 警告レベルのメッセージ
                            eprintln!("{}", get_message(MessageKey::FullBackupFallback, self.lang));
                            eprintln!(
                                "{}: {e}",
                                get_message(MessageKey::MetadataLoadFailed, self.lang)
                            );
                        }
                        println!("{}", get_message(MessageKey::FullBackupMode, self.lang));
                        (BackupType::Full, None, all_files.clone())
                    }
                }
            } else {
                // --incremental フラグが指定されているが、前回のバックアップがない場合
                if self.incremental {
                    println!("{}", get_message(MessageKey::NoBackupsFound, self.lang));
                }
                println!("{}", get_message(MessageKey::FullBackupMode, self.lang));
                (BackupType::Full, None, all_files.clone())
            };

        let total_files = files_to_backup.len();

        if self.dry_run {
            println!(
                "{}",
                get_message(MessageKey::DryRunMode, self.lang)
                    .replace("{}", &total_files.to_string())
            );
            for (_source, _dest) in &files_to_backup {
                println!("  _source.display() → _dest.display()");
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
            Some(Arc::new(BackupProgress::with_language(
                total_files as u64,
                self.lang,
            )))
        } else {
            None
        };

        // CopyEngineの初期化（I/O最適化）
        let copy_engine = Arc::new(CopyEngine::new());

        // 整合性検証チェッカーの初期化
        let integrity_checker = if self.verify_integrity {
            Some(Arc::new(std::sync::Mutex::new(IntegrityChecker::new())))
        } else {
            None
        };

        // 並列バックアップ処理
        let success_count = AtomicUsize::new(0);
        let failed_count = AtomicUsize::new(0);
        let total_bytes = AtomicUsize::new(0);

        let errors: Vec<String> = files_to_backup
            .par_iter()
            .filter_map(|(source, dest)| {
                // 進捗表示更新
                if let Some(ref pb) = progress {
                    if let Some(file_name) = source.file_name() {
                        pb.set_message(&format!("処理中: {file_name:?}"));
                    }
                }

                // バックアップディレクトリからの相対パスを計算（整合性検証用）
                let relative_path = dest.strip_prefix(&backup_base).ok();

                // バックアップ先のディレクトリを作成
                if let Some(parent) = dest.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        failed_count.fetch_add(1, Ordering::Relaxed);
                        if let Some(ref pb) = progress {
                            pb.inc(1);
                        }
                        return Some(format!("ディレクトリ作成失敗 parent.display(): {e}"));
                    }
                }

                // ProcessingPipelineまたはCopyEngineでファイル処理
                let copy_result = if let Some(ref pipeline) = pipeline {
                    // 暗号化・圧縮パイプライン使用
                    match pipeline.process_file(
                        source,
                        master_key.as_ref().map(std::convert::AsRef::as_ref),
                        encryption_salt,
                    ) {
                        Ok(processed) => {
                            // 処理後のデータをファイルに書き込み
                            match std::fs::write(dest, &processed.data) {
                                Ok(_) => {
                                    success_count.fetch_add(1, Ordering::Relaxed);
                                    total_bytes.fetch_add(
                                        processed.metadata.final_size as usize,
                                        Ordering::Relaxed,
                                    );
                                    if let Some(ref pb) = progress {
                                        pb.inc(1);
                                    }
                                    Ok(())
                                }
                                Err(e) => {
                                    failed_count.fetch_add(1, Ordering::Relaxed);
                                    if let Some(ref pb) = progress {
                                        pb.inc(1);
                                    }
                                    Err(format!("書き込み失敗 dest.display(): {e}"))
                                }
                            }
                        }
                        Err(e) => {
                            failed_count.fetch_add(1, Ordering::Relaxed);
                            if let Some(ref pb) = progress {
                                pb.inc(1);
                            }
                            Err(format!("処理失敗 source.display(): {e}"))
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
                            Ok(())
                        }
                        Err(e) => {
                            failed_count.fetch_add(1, Ordering::Relaxed);
                            if let Some(ref pb) = progress {
                                pb.inc(1);
                            }
                            Err(format!("コピー失敗 source.display(): {e}"))
                        }
                    }
                };

                // 整合性検証：元ファイルのハッシュを計算して保存
                if copy_result.is_ok() {
                    if let Some(ref checker) = integrity_checker {
                        if let Some(rel_path) = relative_path {
                            if let Ok(mut guard) = checker.lock() {
                                if let Ok(hash) = guard.compute_hash(source) {
                                    guard.add_file_hash(rel_path.to_path_buf(), hash);
                                }
                            }
                        }
                    }
                }

                copy_result.err()
            })
            .collect();

        // プログレスバー完了
        if let Some(pb) = progress {
            let failed = failed_count.load(Ordering::Relaxed);
            if failed == 0 {
                pb.finish(get_message(MessageKey::BackupComplete, self.lang));
            } else {
                pb.finish(&format!(
                    "{} ({} {})",
                    get_message(MessageKey::BackupCompleteWithFailures, self.lang)
                        .replace("（失敗あり）", "")
                        .replace("（有失败）", "")
                        .replace("（有失敗）", "")
                        .replace("(with failures)", ""),
                    failed,
                    get_message(MessageKey::FailedLabel, self.lang)
                ));
            }
        }

        // 整合性メタデータを保存（増分情報を含む）
        if let Some(ref checker) = integrity_checker {
            if let Ok(mut guard) = checker.lock() {
                // 増分バックアップ情報を追加
                guard.metadata.backup_type = actual_backup_type;
                guard.metadata.parent_backup = parent_backup_name;
                guard.metadata.changed_files = files_to_backup
                    .iter()
                    .filter_map(|(_, dest)| {
                        dest.strip_prefix(&backup_base)
                            .ok()
                            .map(std::path::Path::to_path_buf)
                    })
                    .collect();

                // 増分バックアップの場合、変更されなかったファイルのハッシュも保存
                // （次回の増分バックアップで正しく比較できるようにするため）
                if actual_backup_type == BackupType::Incremental {
                    for (source, dest) in &all_files {
                        if let Ok(rel_path) = dest.strip_prefix(&backup_base) {
                            // 既にハッシュが保存されているファイルはスキップ
                            if !guard.metadata.file_hashes.contains_key(rel_path) {
                                if let Ok(hash) = guard.compute_hash(source) {
                                    guard.add_file_hash(rel_path.to_path_buf(), hash);
                                }
                            }
                        }
                    }
                }

                if let Err(e) = guard.save_metadata(&backup_base) {
                    eprintln!("警告: 整合性メタデータの保存に失敗しました: {e}");
                }
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
            eprintln!("履歴保存失敗: {e}");
        }

        // 監査ログ: バックアップ完了 or 失敗
        if let Some(ref mut audit_log) = self.audit_log {
            let metadata = serde_json::json!({
                "total_files": result.total_files,
                "successful": result.successful,
                "failed": result.failed,
                "total_bytes": result.total_bytes,
                "backup_name": result.backup_name,
            });

            let event = if success {
                AuditEvent::backup_completed(&target_desc, &user, metadata)
            } else {
                AuditEvent::backup_failed(
                    &target_desc,
                    &user,
                    format!("{}件のファイルでエラーが発生しました", result.failed),
                )
            };

            let _ = audit_log
                .log(event)
                .map_err(|e| eprintln!("警告: 監査ログの記録に失敗しました: {e}"));
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

        let mut runner = BackupRunner::new(config, false);
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

        let mut runner = BackupRunner::new(config, true);
        let result = runner.run(None, None).unwrap();

        assert_eq!(result.total_files, 1);
        assert_eq!(result.successful, 0); // ドライランなので実行なし
        assert_eq!(result.total_bytes, 0);
    }
}
