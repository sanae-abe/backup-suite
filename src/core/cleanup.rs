use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::{BackupHistory, Config, Priority};
use crate::security::{AuditEvent, AuditLog};

/// クリーンアップポリシー
///
/// 古いバックアップの削除条件を定義します。
#[derive(Debug, Clone)]
pub struct CleanupPolicy {
    /// 保持期間（日数）
    pub retention_days: Option<u32>,
    /// 保持数（最新N個）
    pub keep_count: Option<usize>,
    /// 最大合計サイズ（バイト）
    pub max_total_size: Option<u64>,
    /// 優先度別保持（高優先度は長く保持）
    pub priority_based: bool,
}

impl Default for CleanupPolicy {
    fn default() -> Self {
        Self {
            retention_days: Some(30),
            keep_count: None,
            max_total_size: None,
            priority_based: false,
        }
    }
}

impl CleanupPolicy {
    /// 保持期間を指定してポリシーを作成
    #[must_use]
    pub fn retention_days(days: u32) -> Self {
        Self {
            retention_days: Some(days),
            ..Default::default()
        }
    }

    /// 保持数を指定してポリシーを作成
    #[must_use]
    pub fn keep_count(count: usize) -> Self {
        Self {
            keep_count: Some(count),
            retention_days: None,
            ..Default::default()
        }
    }

    /// 最大サイズを指定してポリシーを作成
    #[must_use]
    pub fn max_size(size_bytes: u64) -> Self {
        Self {
            max_total_size: Some(size_bytes),
            retention_days: None,
            ..Default::default()
        }
    }

    /// 優先度別保持を有効化
    #[must_use]
    pub fn with_priority_based(mut self) -> Self {
        self.priority_based = true;
        self
    }
}

/// クリーンアップ結果
#[derive(Debug)]
pub struct CleanupResult {
    pub total_checked: usize,
    pub deleted: usize,
    pub freed_bytes: u64,
    pub errors: Vec<String>,
}

impl CleanupResult {
    fn new() -> Self {
        Self {
            total_checked: 0,
            deleted: 0,
            freed_bytes: 0,
            errors: Vec::new(),
        }
    }
}

/// バックアップ情報
#[derive(Debug, Clone)]
struct BackupInfo {
    path: PathBuf,
    modified_time: DateTime<Utc>,
    size: u64,
    priority: Option<Priority>,
}

/// クリーンアップエンジン
///
/// 古いバックアップを自動的に削除します。
pub struct CleanupEngine {
    policy: CleanupPolicy,
    dry_run: bool,
    interactive: bool,
    audit_log: Option<AuditLog>,
}

impl CleanupEngine {
    /// 新しいCleanupEngineを作成
    #[must_use]
    pub fn new(policy: CleanupPolicy, dry_run: bool) -> Self {
        let audit_log = AuditLog::new()
            .map_err(|e| eprintln!("警告: 監査ログの初期化に失敗しました: {e}"))
            .ok();

        Self {
            policy,
            dry_run,
            interactive: false,
            audit_log,
        }
    }

    /// 対話的削除を有効化
    #[must_use]
    pub fn with_interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// クリーンアップを実行
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// * 設定ファイルの読み込みに失敗した場合
    /// * バックアップディレクトリの列挙に失敗した場合
    /// * ファイルメタデータの取得に失敗した場合
    /// * 削除対象の決定に失敗した場合
    /// * 対話的確認の入力処理に失敗した場合
    pub fn cleanup(&mut self) -> Result<CleanupResult> {
        let user = AuditLog::current_user();
        let days = self.policy.retention_days.unwrap_or(0);

        // 監査ログ: クリーンアップ開始
        if let Some(ref mut audit_log) = self.audit_log {
            let _ = audit_log
                .log(AuditEvent::cleanup_started(&user, days))
                .map_err(|e| eprintln!("警告: 監査ログの記録に失敗しました: {e}"));
        }

        let config = Config::load()?;
        let dest = &config.backup.destination;

        if !dest.exists() {
            return Ok(CleanupResult::new());
        }

        // バックアップディレクトリ一覧を取得
        let mut backups = self.get_backup_list(dest)?;

        // ソート（新しい順）
        backups.sort_by(|a, b| b.modified_time.cmp(&a.modified_time));

        let mut result = CleanupResult::new();
        result.total_checked = backups.len();

        // 削除対象を決定
        let to_delete = self.determine_deletions(&backups)?;

        for backup in to_delete {
            if self.interactive {
                // 対話的確認
                if !self.confirm_deletion(&backup)? {
                    continue;
                }
            }

            if self.dry_run {
                println!("🗑️  [ドライラン] 削除予定: {:?}", backup.path);
                result.deleted += 1;
                result.freed_bytes += backup.size;
            } else {
                match std::fs::remove_dir_all(&backup.path) {
                    Ok(_) => {
                        println!("🗑️  削除完了: {:?}", backup.path);
                        result.deleted += 1;
                        result.freed_bytes += backup.size;
                    }
                    Err(e) => {
                        result
                            .errors
                            .push(format!("削除失敗 {:?}: {}", backup.path, e));
                    }
                }
            }
        }

        // 監査ログ: クリーンアップ完了 or 失敗
        if let Some(ref mut audit_log) = self.audit_log {
            let metadata = serde_json::json!({
                "total_checked": result.total_checked,
                "deleted": result.deleted,
                "freed_bytes": result.freed_bytes,
                "policy": format!("{:?}", self.policy),
            });

            let event = if result.errors.is_empty() {
                AuditEvent::cleanup_completed(&user, metadata)
            } else {
                AuditEvent::cleanup_failed(
                    &user,
                    format!("{}件のエラーが発生しました", result.errors.len()),
                )
            };

            let _ = audit_log
                .log(event)
                .map_err(|e| eprintln!("警告: 監査ログの記録に失敗しました: {e}"));
        }

        Ok(result)
    }

    /// バックアップ一覧を取得
    fn get_backup_list(&self, dest: &Path) -> Result<Vec<BackupInfo>> {
        let mut backups = Vec::new();

        for entry in WalkDir::new(dest)
            .max_depth(1)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if !entry.file_type().is_dir() || entry.path() == dest {
                continue;
            }

            let path = entry.path().to_path_buf();
            let metadata = std::fs::metadata(&path)?;
            let modified_time: DateTime<Utc> = metadata.modified()?.into();
            let size = self.calculate_size(&path)?;

            // 優先度を履歴から取得（可能な場合）
            let priority = self.get_priority_from_history(&path);

            backups.push(BackupInfo {
                path,
                modified_time,
                size,
                priority,
            });
        }

        Ok(backups)
    }

    /// ディレクトリサイズを計算
    fn calculate_size(&self, dir: &Path) -> Result<u64> {
        let mut total = 0;
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if entry.file_type().is_file() {
                total += entry.metadata()?.len();
            }
        }
        Ok(total)
    }

    /// 履歴から優先度を取得
    fn get_priority_from_history(&self, backup_dir: &Path) -> Option<Priority> {
        if let Ok(history) = BackupHistory::load_all() {
            history
                .iter()
                .find(|h| h.backup_dir == backup_dir)
                .and_then(|h| h.priority)
        } else {
            None
        }
    }

    /// 削除対象を決定
    fn determine_deletions(&self, backups: &[BackupInfo]) -> Result<Vec<BackupInfo>> {
        let mut to_delete = Vec::new();

        // 1. 保持期間による削除
        if let Some(days) = self.policy.retention_days {
            let cutoff = Utc::now() - chrono::Duration::days(days as i64);
            for backup in backups {
                if backup.modified_time < cutoff {
                    // 優先度別保持が有効な場合
                    if self.policy.priority_based {
                        if let Some(Priority::High) = backup.priority {
                            // 高優先度は2倍の期間保持
                            let high_priority_cutoff =
                                Utc::now() - chrono::Duration::days((days * 2) as i64);
                            if backup.modified_time < high_priority_cutoff {
                                to_delete.push(backup.clone());
                            }
                        } else {
                            to_delete.push(backup.clone());
                        }
                    } else {
                        to_delete.push(backup.clone());
                    }
                }
            }
        }

        // 2. 保持数による削除
        if let Some(keep) = self.policy.keep_count {
            if backups.len() > keep {
                to_delete.extend_from_slice(&backups[keep..]);
            }
        }

        // 3. 最大サイズによる削除
        if let Some(max_size) = self.policy.max_total_size {
            let mut current_size = 0u64;
            for backup in backups {
                current_size += backup.size;
                if current_size > max_size {
                    to_delete.push(backup.clone());
                }
            }
        }

        // 重複を排除
        to_delete.sort_by(|a, b| a.path.cmp(&b.path));
        to_delete.dedup_by(|a, b| a.path == b.path);

        Ok(to_delete)
    }

    /// 削除確認（対話的）
    fn confirm_deletion(&self, backup: &BackupInfo) -> Result<bool> {
        use dialoguer::Confirm;

        println!("\n削除候補:");
        println!("  パス: {:?}", backup.path);
        println!(
            "  作成日時: {}",
            backup.modified_time.format("%Y-%m-%d %H:%M:%S")
        );
        println!("  サイズ: {}", format_bytes(backup.size));
        if let Some(ref priority) = backup.priority {
            println!("  優先度: {priority:?}");
        }

        let confirm = Confirm::new()
            .with_prompt("このバックアップを削除しますか？")
            .default(false)
            .interact()?;

        Ok(confirm)
    }
}

/// バイト数を人間が読みやすい形式に変換
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1_048_576), "1.00 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.00 GB");
    }

    #[test]
    fn test_cleanup_policy_retention_days() {
        let policy = CleanupPolicy::retention_days(30);
        assert_eq!(policy.retention_days, Some(30));
        assert_eq!(policy.keep_count, None);
    }

    #[test]
    fn test_cleanup_policy_keep_count() {
        let policy = CleanupPolicy::keep_count(10);
        assert_eq!(policy.keep_count, Some(10));
        assert_eq!(policy.retention_days, None);
    }

    #[test]
    fn test_calculate_size() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("test_dir");
        fs::create_dir_all(&dir).unwrap();

        // テストファイルを作成
        fs::write(dir.join("file1.txt"), b"hello").unwrap();
        fs::write(dir.join("file2.txt"), b"world").unwrap();

        let engine = CleanupEngine::new(CleanupPolicy::default(), false);
        let size = engine.calculate_size(&dir).unwrap();

        assert_eq!(size, 10); // "hello" + "world" = 10 bytes
    }
}
