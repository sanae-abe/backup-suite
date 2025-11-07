use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use super::{Config, Priority};

/// バックアップステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BackupStatus {
    Success,
    Failed,
    Partial,
}

/// バックアップ履歴エントリ（Phase 2拡張版）
///
/// 1回のバックアップ実行の詳細な記録を保持します。
///
/// # フィールド
///
/// * `timestamp` - バックアップ実行日時
/// * `backup_dir` - バックアップディレクトリのパス
/// * `category` - カテゴリ（Optional）
/// * `priority` - 優先度（Optional）
/// * `status` - バックアップステータス（Success/Failed/Partial）
/// * `total_files` - バックアップしたファイル数
/// * `total_bytes` - バックアップした総バイト数
/// * `compressed` - 圧縮されているか
/// * `encrypted` - 暗号化されているか
/// * `duration_ms` - 処理時間（ミリ秒）
/// * `error_message` - エラーメッセージ（失敗時）
/// * `success` - 後方互換性のための成功フラグ
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::{BackupHistory, core::history::BackupStatus, Priority};
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
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default = "default_status")]
    pub status: BackupStatus,
    pub total_files: usize,
    pub total_bytes: u64,
    #[serde(default)]
    pub compressed: bool,
    #[serde(default)]
    pub encrypted: bool,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub error_message: Option<String>,
    // 後方互換性のため残す
    pub success: bool,
}

fn default_status() -> BackupStatus {
    BackupStatus::Success
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
            category: None,
            priority: None,
            status: if success { BackupStatus::Success } else { BackupStatus::Failed },
            total_files,
            total_bytes,
            compressed: false,
            encrypted: false,
            duration_ms: 0,
            error_message: None,
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

    /// 優先度でフィルタリング
    ///
    /// 指定された優先度のバックアップ履歴のみを取得します。
    ///
    /// # 引数
    ///
    /// * `priority` - フィルタリングする優先度
    ///
    /// # 戻り値
    ///
    /// 指定された優先度の履歴エントリのベクター
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::{BackupHistory, Priority};
    ///
    /// let history = BackupHistory::load_all().unwrap();
    /// let high_priority = BackupHistory::filter_by_priority(&history, &Priority::High);
    /// println!("高優先度のバックアップ: {}件", high_priority.len());
    /// ```
    pub fn filter_by_priority<'a>(entries: &'a [BackupHistory], priority: &Priority) -> Vec<&'a BackupHistory> {
        entries
            .iter()
            .filter(|h| h.priority.as_ref() == Some(priority))
            .collect()
    }

    /// カテゴリでフィルタリング
    ///
    /// 指定されたカテゴリのバックアップ履歴のみを取得します。
    ///
    /// # 引数
    ///
    /// * `category` - フィルタリングするカテゴリ名
    ///
    /// # 戻り値
    ///
    /// 指定されたカテゴリの履歴エントリのベクター
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::BackupHistory;
    ///
    /// let history = BackupHistory::load_all().unwrap();
    /// let docs = BackupHistory::filter_by_category(&history, "documents");
    /// println!("ドキュメントカテゴリのバックアップ: {}件", docs.len());
    /// ```
    pub fn filter_by_category<'a>(entries: &'a [BackupHistory], category: &str) -> Vec<&'a BackupHistory> {
        entries
            .iter()
            .filter(|h| h.category.as_deref() == Some(category))
            .collect()
    }

    /// 最近のエントリを取得
    ///
    /// 指定された件数の最近のバックアップ履歴を取得します。
    ///
    /// # 引数
    ///
    /// * `count` - 取得する件数
    ///
    /// # 戻り値
    ///
    /// 最新のエントリから指定件数分の履歴エントリのベクター
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::BackupHistory;
    ///
    /// // 最新10件を取得
    /// let recent = BackupHistory::get_recent_entries(10).unwrap();
    /// for entry in recent {
    ///     println!("{}: {:?}", entry.timestamp, entry.backup_dir);
    /// }
    /// ```
    pub fn get_recent_entries(count: usize) -> Result<Vec<BackupHistory>> {
        let mut all = Self::load_all()?;
        all.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // 新しい順
        Ok(all.into_iter().take(count).collect())
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
