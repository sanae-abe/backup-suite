use anyhow::{Context, Result};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

use super::incremental::resolve_backup_chain;
use super::integrity::BackupMetadata;
use crate::crypto::{EncryptedData, KeyManager};
use crate::security::{safe_join, AuditEvent, AuditLog};
use crate::ui::progress::BackupProgress;

/// 復元結果
///
/// バックアップからの復元処理の結果を保持します。
#[derive(Debug)]
pub struct RestoreResult {
    pub total_files: usize,
    pub restored: usize,
    pub failed: usize,
    pub encrypted_files: usize,
    pub verified_files: usize,
    pub verification_failures: usize,
    pub total_bytes: u64,
    pub errors: Vec<String>,
}

// RestoreResult は直接構築されるため、new() メソッドは不要

/// 復元エンジン
///
/// バックアップからファイルを復元します。
/// 暗号化、圧縮の自動検出と展開に対応しています。
pub struct RestoreEngine {
    dry_run: bool,
    show_progress: bool,
    verify_integrity: bool,
    audit_log: Option<AuditLog>,
}

impl RestoreEngine {
    /// 新しいRestoreEngineを作成
    #[must_use]
    pub fn new(dry_run: bool) -> Self {
        let audit_log = AuditLog::new()
            .map_err(|e| eprintln!("警告: 監査ログの初期化に失敗しました: {e}"))
            .ok();

        Self {
            dry_run,
            show_progress: true,
            verify_integrity: true,
            audit_log,
        }
    }

    /// 進捗表示の有効/無効を設定
    #[must_use]
    pub fn with_progress(mut self, show_progress: bool) -> Self {
        self.show_progress = show_progress;
        self
    }

    /// 整合性検証の有効/無効を設定
    #[must_use]
    pub fn with_verification(mut self, verify: bool) -> Self {
        self.verify_integrity = verify;
        self
    }

    /// バックアップから復元
    ///
    /// # 引数
    ///
    /// * `backup_dir` - バックアップディレクトリのパス
    /// * `dest_dir` - 復元先ディレクトリ
    /// * `password` - 暗号化されている場合のパスワード（Optional）
    ///
    /// # 戻り値
    ///
    /// 成功時は RestoreResult、失敗時はエラー
    pub fn restore(
        &mut self,
        backup_dir: &Path,
        dest_dir: &Path,
        password: Option<&str>,
    ) -> Result<RestoreResult> {
        let user = AuditLog::current_user();
        let target_desc = "backup_dir.display() → dest_dir.display()".to_string();

        // 監査ログ: 復元開始
        if let Some(ref mut audit_log) = self.audit_log {
            let _ = audit_log
                .log(AuditEvent::restore_started(&target_desc, &user))
                .map_err(|e| eprintln!("警告: 監査ログの記録に失敗しました: {e}"));
        }

        if !backup_dir.exists() {
            // 監査ログ: 復元失敗
            if let Some(ref mut audit_log) = self.audit_log {
                let _ = audit_log
                    .log(AuditEvent::restore_failed(
                        &target_desc,
                        &user,
                        "バックアップディレクトリが存在しません",
                    ))
                    .map_err(|e| eprintln!("警告: 監査ログの記録に失敗しました: {e}"));
            }

            return Err(anyhow::anyhow!(
                "バックアップディレクトリが存在しません: backup_dir.display()".to_string()
            ));
        }

        // 増分バックアップチェーンの解決
        let backup_chain = resolve_backup_chain(backup_dir)?;

        if backup_chain.len() > 1 {
            println!(
                "📦 増分バックアップチェーン検出: {} 個のバックアップを順次復元",
                backup_chain.len()
            );
            for (i, backup) in backup_chain.iter().enumerate() {
                println!("  {}. {:?}", i + 1, backup.file_name().unwrap_or_default());
            }
        }

        // 復元先ディレクトリを作成
        if !self.dry_run {
            std::fs::create_dir_all(dest_dir)
                .context("復元先ディレクトリ作成失敗: dest_dir.display()".to_string())?;
        }

        // チェーン内のすべてのバックアップからファイル一覧を収集
        let mut all_files: Vec<(PathBuf, PathBuf)> = Vec::new(); // (source_backup_dir, file_path)
        for backup in &backup_chain {
            let files_in_backup: Vec<PathBuf> = WalkDir::new(backup)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.file_type().is_file())
                .filter(|e| {
                    // .integrityファイルを除外
                    e.file_name() != ".integrity"
                })
                .map(|e| e.path().to_path_buf())
                .collect();

            for file_path in files_in_backup {
                all_files.push((backup.clone(), file_path));
            }
        }

        let files: Vec<PathBuf> = all_files.iter().map(|(_, path)| path.clone()).collect();

        let total_files = files.len();

        if self.dry_run {
            println!("📋 ドライランモード: {total_files} ファイルを復元対象として検出");
            for (backup_src, file) in &all_files {
                if let Ok(relative) = file.strip_prefix(backup_src) {
                    println!("  {}", relative.display());
                }
            }
            return Ok(RestoreResult {
                total_files,
                restored: 0,
                failed: 0,
                encrypted_files: 0,
                verified_files: 0,
                verification_failures: 0,
                total_bytes: 0,
                errors: Vec::new(),
            });
        }

        // プログレスバーの初期化
        let progress = if self.show_progress {
            let pb = BackupProgress::new(total_files as u64);
            pb.set_message("復元中...");
            Some(pb)
        } else {
            None
        };

        let restored_count = AtomicUsize::new(0);
        let failed_count = AtomicUsize::new(0);
        let encrypted_count = AtomicUsize::new(0);
        let verified_count = AtomicUsize::new(0);
        let verification_failed_count = AtomicUsize::new(0);
        let total_bytes = AtomicUsize::new(0);

        // マスターキー（遅延初期化）
        let mut master_key_opt: Option<std::sync::Arc<crate::crypto::MasterKey>> = None;

        // 各バックアップディレクトリの整合性メタデータを読み込み
        let mut backup_metadata_map: std::collections::HashMap<PathBuf, BackupMetadata> =
            std::collections::HashMap::new();
        if self.verify_integrity {
            for backup in &backup_chain {
                match BackupMetadata::load(backup) {
                    Ok(metadata) => {
                        backup_metadata_map.insert(backup.clone(), metadata);
                    }
                    Err(e) => {
                        eprintln!(
                            "警告: 整合性メタデータの読み込みに失敗しました ({}): {e}",
                            backup.display()
                        );
                    }
                }
            }
            if !backup_metadata_map.is_empty() {
                println!(
                    "✓ 整合性メタデータを読み込みました（{}個のバックアップ）",
                    backup_metadata_map.len()
                );
            }
        }

        let mut errors = Vec::new();

        for (source_backup_dir, source_path) in &all_files {
            // プログレス更新
            if let Some(ref pb) = progress {
                if let Some(file_name) = source_path.file_name() {
                    pb.set_message(&format!("復元中: {file_name:?}"));
                }
            }

            // 相対パスを取得（元のバックアップディレクトリを基準に）
            let relative_path = match source_path.strip_prefix(source_backup_dir) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("相対パス取得失敗: source_path.display(): {e}"));
                    failed_count.fetch_add(1, Ordering::Relaxed);
                    if let Some(ref pb) = progress {
                        pb.inc(1);
                    }
                    continue;
                }
            };

            // 復元先パスを安全に結合（パストラバーサル対策）
            let dest_path = match safe_join(dest_dir, relative_path) {
                Ok(p) => p,
                Err(e) => {
                    errors.push(format!(
                        "パストラバーサル検出: relative_path.display(): {e}"
                    ));
                    failed_count.fetch_add(1, Ordering::Relaxed);
                    if let Some(ref pb) = progress {
                        pb.inc(1);
                    }
                    continue;
                }
            };

            // 親ディレクトリを作成
            if let Some(parent) = dest_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    errors.push(format!("ディレクトリ作成失敗: {}: {e}", parent.display()));
                    failed_count.fetch_add(1, Ordering::Relaxed);
                    if let Some(ref pb) = progress {
                        pb.inc(1);
                    }
                    continue;
                }
            }

            // ファイルを読み込み
            let file_data = match std::fs::read(source_path) {
                Ok(d) => d,
                Err(e) => {
                    errors.push(format!("ファイル読み込み失敗: source_path.display(): {e}"));
                    failed_count.fetch_add(1, Ordering::Relaxed);
                    if let Some(ref pb) = progress {
                        pb.inc(1);
                    }
                    continue;
                }
            };

            // 暗号化データかどうか判定して復号
            let final_data = if let Ok(encrypted_data) = EncryptedData::from_bytes(&file_data) {
                // 暗号化されたファイル
                encrypted_count.fetch_add(1, Ordering::Relaxed);

                // マスターキーがまだ作成されていない場合
                if master_key_opt.is_none() {
                    let pwd = match password {
                        Some(p) => p.to_string(),
                        None => {
                            errors.push(
                                "暗号化されたファイルですがパスワードが未指定: relative_path.display()".to_string()
                            );
                            failed_count.fetch_add(1, Ordering::Relaxed);
                            if let Some(ref pb) = progress {
                                pb.inc(1);
                            }
                            continue;
                        }
                    };

                    // マスターキー生成
                    let km = KeyManager::default();
                    match km.restore_master_key(&pwd, &encrypted_data.salt) {
                        Ok(mk) => {
                            master_key_opt = Some(std::sync::Arc::new(mk));
                        }
                        Err(e) => {
                            errors.push(format!("マスターキー復元失敗: {e}"));
                            failed_count.fetch_add(1, Ordering::Relaxed);
                            if let Some(ref pb) = progress {
                                pb.inc(1);
                            }
                            continue;
                        }
                    }
                }

                // 復号化
                let master_key = master_key_opt.as_ref().unwrap();
                let encryption_engine = crate::crypto::EncryptionEngine::default();

                match encryption_engine.decrypt(&encrypted_data, master_key) {
                    Ok(decrypted_data) => {
                        // 復号化されたデータを展開（圧縮されている可能性）
                        self.decompress_if_needed(&decrypted_data)?
                    }
                    Err(e) => {
                        errors.push(format!("復号化失敗: relative_path.display(): {e}"));
                        failed_count.fetch_add(1, Ordering::Relaxed);
                        if let Some(ref pb) = progress {
                            pb.inc(1);
                        }
                        continue;
                    }
                }
            } else {
                // 通常のファイル（暗号化されていない）
                // 圧縮されている可能性を確認
                self.decompress_if_needed(&file_data)?
            };

            // 復元先に書き込み
            match std::fs::write(&dest_path, &final_data) {
                Ok(_) => {
                    restored_count.fetch_add(1, Ordering::Relaxed);
                    total_bytes.fetch_add(final_data.len(), Ordering::Relaxed);

                    // 整合性検証（該当するバックアップディレクトリのメタデータを使用）
                    if let Some(metadata) = backup_metadata_map.get(source_backup_dir) {
                        match metadata.verify_file(relative_path, &dest_path) {
                            Ok(true) => {
                                verified_count.fetch_add(1, Ordering::Relaxed);
                            }
                            Ok(false) => {
                                verification_failed_count.fetch_add(1, Ordering::Relaxed);
                                errors.push(
                                    "⚠ 整合性検証失敗（ファイルが改ざんされています）: relative_path.display()".to_string()
                                );
                            }
                            Err(e) => {
                                eprintln!("警告: 整合性検証エラー: relative_path.display(): {e}");
                            }
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("ファイル書き込み失敗: dest_path.display(): {e}"));
                    failed_count.fetch_add(1, Ordering::Relaxed);
                }
            }

            if let Some(ref pb) = progress {
                pb.inc(1);
            }
        }

        // プログレスバー完了
        if let Some(pb) = progress {
            let failed = failed_count.load(Ordering::Relaxed);
            if failed == 0 {
                pb.finish("✓ 復元完了");
            } else {
                pb.finish(&format!("⚠ 復元完了（{failed}件失敗）"));
            }
        }

        let result = RestoreResult {
            total_files,
            restored: restored_count.load(Ordering::Relaxed),
            failed: failed_count.load(Ordering::Relaxed),
            encrypted_files: encrypted_count.load(Ordering::Relaxed),
            verified_files: verified_count.load(Ordering::Relaxed),
            verification_failures: verification_failed_count.load(Ordering::Relaxed),
            total_bytes: total_bytes.load(Ordering::Relaxed) as u64,
            errors,
        };

        // 監査ログ: 復元完了 or 失敗
        if let Some(ref mut audit_log) = self.audit_log {
            let metadata = serde_json::json!({
                "total_files": result.total_files,
                "restored": result.restored,
                "failed": result.failed,
                "encrypted_files": result.encrypted_files,
                "verified_files": result.verified_files,
                "verification_failures": result.verification_failures,
                "total_bytes": result.total_bytes,
            });

            let event = if result.failed == 0 {
                AuditEvent::restore_completed(&target_desc, &user, metadata)
            } else {
                AuditEvent::restore_failed(
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

    /// 圧縮されている場合に展開
    fn decompress_if_needed(&self, data: &[u8]) -> Result<Vec<u8>> {
        // zstd → gzip → 無圧縮の順で試す
        if let Ok(decompressed) = zstd::decode_all(data) {
            Ok(decompressed)
        } else {
            let mut decoder = flate2::read::GzDecoder::new(data);
            let mut decompressed = Vec::new();
            if decoder.read_to_end(&mut decompressed).is_ok() && !decompressed.is_empty() {
                Ok(decompressed)
            } else {
                // 圧縮されていないと判断
                Ok(data.to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_restore_unencrypted() {
        let temp = TempDir::new().unwrap();
        let backup_dir = temp.path().join("backup");
        let restore_dir = temp.path().join("restore");

        // テストデータを作成
        fs::create_dir_all(&backup_dir).unwrap();
        fs::write(backup_dir.join("test.txt"), b"test content").unwrap();

        let mut engine = RestoreEngine::new(false).with_progress(false);
        let result = engine.restore(&backup_dir, &restore_dir, None).unwrap();

        assert_eq!(result.total_files, 1);
        assert_eq!(result.restored, 1);
        assert_eq!(result.failed, 0);
        assert_eq!(result.encrypted_files, 0);
        // Note: verification is 0 because no .integrity metadata exists in this test
        assert_eq!(result.verified_files, 0);
        assert_eq!(result.verification_failures, 0);

        // 復元されたファイルを確認
        let restored_content = fs::read_to_string(restore_dir.join("test.txt")).unwrap();
        assert_eq!(restored_content, "test content");
    }

    #[test]
    fn test_restore_dry_run() {
        let temp = TempDir::new().unwrap();
        let backup_dir = temp.path().join("backup");
        let restore_dir = temp.path().join("restore");

        fs::create_dir_all(&backup_dir).unwrap();
        fs::write(backup_dir.join("test.txt"), b"test").unwrap();

        let mut engine = RestoreEngine::new(true).with_progress(false);
        let result = engine.restore(&backup_dir, &restore_dir, None).unwrap();

        assert_eq!(result.total_files, 1);
        assert_eq!(result.restored, 0); // ドライランなので実行なし
        assert!(!restore_dir.exists()); // ディレクトリも作成されない
    }
}
