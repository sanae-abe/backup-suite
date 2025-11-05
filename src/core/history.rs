use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use super::Config;

/// バックアップ履歴エントリ
///
/// 1回のバックアップ実行の記録を保持します。
///
/// # フィールド
///
/// * `timestamp` - バックアップ実行日時
/// * `backup_dir` - バックアップディレクトリのパス
/// * `total_files` - バックアップしたファイル数
/// * `total_bytes` - バックアップした総バイト数
/// * `success` - バックアップが成功したかどうか
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::BackupHistory;
/// use std::path::PathBuf;
///
/// let history = BackupHistory::new(
///     PathBuf::from("/backup/backup_20250105_120000"),
///     150,
///     1024000,
///     true
/// );
/// BackupHistory::save(&history).unwrap();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupHistory {
    pub timestamp: DateTime<Utc>,
    pub backup_dir: PathBuf,
    pub total_files: usize,
    pub total_bytes: u64,
    pub success: bool,
}

impl BackupHistory {
    /// 新しいバックアップ履歴エントリを作成
    ///
    /// # 引数
    ///
    /// * `backup_dir` - バックアップディレクトリのパス
    /// * `total_files` - バックアップしたファイル数
    /// * `total_bytes` - バックアップした総バイト数
    /// * `success` - 成功フラグ
    ///
    /// # 戻り値
    ///
    /// 現在時刻をタイムスタンプとする新しい BackupHistory インスタンス
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::BackupHistory;
    /// use std::path::PathBuf;
    ///
    /// let history = BackupHistory::new(
    ///     PathBuf::from("/backup/backup_20250105_120000"),
    ///     100,
    ///     500000,
    ///     true
    /// );
    /// ```
    pub fn new(backup_dir: PathBuf, total_files: usize, total_bytes: u64, success: bool) -> Self {
        Self {
            timestamp: Utc::now(),
            backup_dir,
            total_files,
            total_bytes,
            success,
        }
    }

    /// 履歴ファイルのパスを取得
    ///
    /// # 戻り値
    ///
    /// 成功時は履歴ファイルのパス（`~/.config/backup-suite/history.toml`）、失敗時はエラー
    ///
    /// # エラー
    ///
    /// * 設定ディレクトリのパス取得に失敗した場合
    pub fn log_path() -> Result<PathBuf> {
        let config_dir = Config::config_path()?
            .parent()
            .context("設定ディレクトリ取得失敗")?
            .to_path_buf();
        Ok(config_dir.join("history.toml"))
    }

    /// 履歴エントリを保存
    ///
    /// 新しいバックアップ履歴を履歴ファイルに追加します。
    /// 履歴は最新100件のみ保持され、古いエントリは自動削除されます。
    ///
    /// # 引数
    ///
    /// * `entry` - 保存する履歴エントリ
    ///
    /// # 戻り値
    ///
    /// 成功時は `Ok(())`、失敗時はエラー
    ///
    /// # エラー
    ///
    /// * 履歴ファイルの読み書きに失敗した場合
    /// * TOML生成に失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::BackupHistory;
    /// use std::path::PathBuf;
    ///
    /// let history = BackupHistory::new(
    ///     PathBuf::from("/backup/backup_20250105_120000"),
    ///     100,
    ///     500000,
    ///     true
    /// );
    /// BackupHistory::save(&history).unwrap();
    /// ```
    pub fn save(entry: &BackupHistory) -> Result<()> {
        let log_path = Self::log_path()?;
        let mut history = Self::load_all()?;
        history.push(entry.clone());

        // 最新100件のみ保持
        if history.len() > 100 {
            history.drain(0..history.len() - 100);
        }

        let content = toml::to_string_pretty(&HistoryFile { history })?;
        fs::write(&log_path, content)?;
        Ok(())
    }

    /// すべての履歴エントリを読み込み
    ///
    /// 履歴ファイルからすべてのバックアップ履歴を読み込みます。
    ///
    /// # 戻り値
    ///
    /// 成功時はすべての履歴エントリのベクター、失敗時はエラー
    ///
    /// # エラー
    ///
    /// * 履歴ファイルの読み込みに失敗した場合
    /// * TOML解析に失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::BackupHistory;
    ///
    /// let history = BackupHistory::load_all().unwrap();
    /// println!("過去のバックアップ数: {}", history.len());
    /// ```
    pub fn load_all() -> Result<Vec<BackupHistory>> {
        let log_path = Self::log_path()?;
        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&log_path)?;
        let file: HistoryFile = toml::from_str(&content)?;
        Ok(file.history)
    }

    /// 指定日数以内の履歴をフィルタリング
    ///
    /// 指定された日数以内に実行されたバックアップ履歴のみを返します。
    ///
    /// # 引数
    ///
    /// * `days` - フィルタリングする日数
    ///
    /// # 戻り値
    ///
    /// 成功時は指定日数以内の履歴エントリのベクター、失敗時はエラー
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::BackupHistory;
    ///
    /// // 過去7日間の履歴を取得
    /// let recent = BackupHistory::filter_by_days(7).unwrap();
    /// println!("過去7日間のバックアップ: {}件", recent.len());
    /// ```
    pub fn filter_by_days(days: u32) -> Result<Vec<BackupHistory>> {
        let all = Self::load_all()?;
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        Ok(all.into_iter().filter(|h| h.timestamp > cutoff).collect())
    }

    /// バックアップディレクトリのリストを取得
    ///
    /// バックアップ先ディレクトリに存在するすべてのバックアップディレクトリを取得します。
    /// 新しい順にソートされます。
    ///
    /// # 戻り値
    ///
    /// 成功時はバックアップディレクトリのパスのベクター、失敗時はエラー
    ///
    /// # エラー
    ///
    /// * 設定ファイルの読み込みに失敗した場合
    /// * バックアップディレクトリの列挙に失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::BackupHistory;
    ///
    /// let dirs = BackupHistory::list_backup_dirs().unwrap();
    /// for dir in dirs.iter().take(5) {
    ///     println!("バックアップ: {:?}", dir);
    /// }
    /// ```
    pub fn list_backup_dirs() -> Result<Vec<PathBuf>> {
        let config = Config::load()?;
        let dest = &config.backup.destination;

        if !dest.exists() {
            return Ok(Vec::new());
        }

        let mut dirs: Vec<PathBuf> = WalkDir::new(dest)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
            .map(|e| e.path().to_path_buf())
            .filter(|p| p != dest)
            .collect();

        dirs.sort_by(|a, b| {
            let ta = fs::metadata(a).and_then(|m| m.modified()).ok();
            let tb = fs::metadata(b).and_then(|m| m.modified()).ok();
            tb.cmp(&ta) // 新しい順
        });

        Ok(dirs)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct HistoryFile {
    history: Vec<BackupHistory>,
}
