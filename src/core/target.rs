use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// バックアップの優先度
///
/// バックアップ対象の重要度を3段階で定義します。
/// スケジュール実行時に優先度別の頻度設定が可能です。
///
/// # バリアント
///
/// * `High` - 高優先度（毎日バックアップ推奨）
/// * `Medium` - 中優先度（週次バックアップ推奨）
/// * `Low` - 低優先度（月次バックアップ推奨）
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::{Priority, Target};
/// use std::path::PathBuf;
///
/// let target = Target::new(
///     PathBuf::from("/important/data"),
///     Priority::High,
///     "重要データ".to_string()
/// );
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
}

/// バックアップ対象の種別
///
/// ファイル単体かディレクトリ全体かを区別します。
///
/// # バリアント
///
/// * `File` - 単一ファイル
/// * `Directory` - ディレクトリ（配下のファイルを再帰的にバックアップ）
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::TargetType;
///
/// let file_type = TargetType::File;
/// let dir_type = TargetType::Directory;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TargetType {
    File,
    Directory,
}

/// バックアップ対象の定義
///
/// バックアップするファイルやディレクトリの情報を保持します。
///
/// # フィールド
///
/// * `path` - バックアップ対象のパス
/// * `priority` - 優先度（High/Medium/Low）
/// * `target_type` - 種別（File/Directory）
/// * `category` - カテゴリ名（ユーザー定義の分類）
/// * `added_date` - 設定追加日時
/// * `exclude_patterns` - 除外する正規表現パターンのリスト
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::{Target, Priority};
/// use std::path::PathBuf;
///
/// // 基本的な使用
/// let mut target = Target::new(
///     PathBuf::from("/home/user/documents"),
///     Priority::High,
///     "重要ドキュメント".to_string()
/// );
///
/// // 除外パターンの追加
/// target.exclude_patterns = vec![
///     r"\.tmp$".to_string(),
///     r"\.log$".to_string(),
/// ];
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub path: PathBuf,
    pub priority: Priority,
    pub target_type: TargetType,
    pub category: String,
    pub added_date: chrono::DateTime<chrono::Utc>,
    pub exclude_patterns: Vec<String>,
}

impl Target {
    /// 新しいバックアップ対象を作成
    ///
    /// パスの種別（ファイル/ディレクトリ）は自動判定されます。
    ///
    /// # 引数
    ///
    /// * `path` - バックアップ対象のパス
    /// * `priority` - 優先度
    /// * `category` - カテゴリ名
    ///
    /// # 戻り値
    ///
    /// 新しい Target インスタンス
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::{Target, Priority};
    /// use std::path::PathBuf;
    ///
    /// let target = Target::new(
    ///     PathBuf::from("/home/user/photos"),
    ///     Priority::Medium,
    ///     "写真コレクション".to_string()
    /// );
    /// ```
    pub fn new(
        path: PathBuf,
        priority: Priority,
        category: String,
    ) -> Self {
        let target_type = if path.is_file() {
            TargetType::File
        } else {
            TargetType::Directory
        };

        Self {
            path,
            priority,
            target_type,
            category,
            added_date: chrono::Utc::now(),
            exclude_patterns: vec![],
        }
    }
}
