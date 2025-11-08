use std::path::PathBuf;
use thiserror::Error;

/// backup-suite用のカスタムエラー型
///
/// すべてのバックアップ操作で発生する可能性のあるエラーを型安全に表現します。
/// thiserrorを使用して、エラーメッセージの生成とエラー変換を自動化しています。
#[derive(Error, Debug)]
pub enum BackupError {
    /// ホームディレクトリが見つからない場合
    #[error("ホームディレクトリが見つかりません")]
    HomeDirectoryNotFound,

    /// バックアップ対象が存在しない場合
    #[error("バックアップ対象が存在しません: {path}")]
    TargetNotFound { path: PathBuf },

    /// 読み取り権限がない場合
    #[error("読み取り権限がありません: {path}")]
    PermissionDenied { path: PathBuf },

    /// ディレクトリトラバーサル攻撃を検出した場合
    #[error("不正なパス（ディレクトリトラバーサル検出）: {path}")]
    PathTraversalDetected { path: PathBuf },

    /// 親ディレクトリが見つからない場合
    #[error("親ディレクトリが見つかりません: {path}")]
    ParentDirectoryNotFound { path: PathBuf },

    /// 設定ファイルの読み込みエラー
    #[error("設定ファイルの読み込みに失敗: {0}")]
    ConfigLoadError(#[from] toml::de::Error),

    /// 設定ファイルのパースエラー
    #[error("設定ファイルのパースに失敗: {message}")]
    ConfigParseError { message: String },

    /// 設定の検証エラー
    #[error("設定の検証に失敗: {message}")]
    ConfigValidationError { message: String },

    /// I/Oエラー
    #[error("I/Oエラー: {0}")]
    IoError(#[from] std::io::Error),

    /// 正規表現のコンパイルエラー
    #[error("正規表現のコンパイルに失敗: {pattern}")]
    RegexError {
        pattern: String,
        #[source]
        source: regex::Error,
    },

    /// ファイルコピーエラー
    #[error("ファイルコピーに失敗: {from} → {to}")]
    FileCopyError { from: PathBuf, to: PathBuf },

    /// バックアップディレクトリ作成エラー
    #[error("バックアップディレクトリ作成に失敗: {path}")]
    BackupDirectoryCreationError { path: PathBuf },

    /// 暗号化・復号化エラー
    #[error("暗号化エラー: {0}")]
    EncryptionError(String),

    /// 圧縮・展開エラー
    #[error("圧縮エラー: {0}")]
    CompressionError(String),

    /// その他のエラー（anyhowからの変換用）
    #[error("エラー: {0}")]
    Other(#[from] anyhow::Error),
}

/// `BackupError`用の`Result`型エイリアス
///
/// # 使用例
///
/// ```rust
/// use backup_suite::error::Result;
///
/// fn some_operation() -> Result<()> {
///     // 操作
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, BackupError>;

impl BackupError {
    /// エラーが回復可能かどうかを判定
    ///
    /// # 戻り値
    ///
    /// * `true` - リトライで回復可能な一時的エラー
    /// * `false` - 回復不可能な恒久的エラー
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            BackupError::IoError(_) | BackupError::FileCopyError { .. }
        )
    }

    /// エラーがセキュリティに関連するかどうかを判定
    ///
    /// # 戻り値
    ///
    /// * `true` - セキュリティに関連するエラー
    /// * `false` - 通常のエラー
    #[must_use]
    pub fn is_security_related(&self) -> bool {
        matches!(
            self,
            BackupError::PathTraversalDetected { .. } | BackupError::PermissionDenied { .. }
        )
    }

    /// ユーザーフレンドリーなエラーメッセージを生成
    ///
    /// # 戻り値
    ///
    /// エラーの詳細と推奨される対処法を含むメッセージ
    #[must_use]
    pub fn user_friendly_message(&self) -> String {
        match self {
            BackupError::HomeDirectoryNotFound => "ホームディレクトリが見つかりません。\n\
                 対処法: 環境変数 $HOME が設定されているか確認してください。"
                .to_string(),
            BackupError::TargetNotFound { path } => {
                format!(
                    "バックアップ対象が存在しません: {}\n\
                     対処法: パスが正しいか、ファイル/ディレクトリが存在するか確認してください。",
                    path.display()
                )
            }
            BackupError::PermissionDenied { path } => {
                format!(
                    "読み取り権限がありません: {}\n\
                     対処法: ファイル/ディレクトリの権限を確認するか、sudo で実行してください。",
                    path.display()
                )
            }
            BackupError::PathTraversalDetected { path } => {
                format!(
                    "不正なパスが検出されました: {}\n\
                     セキュリティ警告: ディレクトリトラバーサル攻撃の可能性があります。",
                    path.display()
                )
            }
            BackupError::ConfigValidationError { message } => {
                format!(
                    "設定に問題があります: {message}\n\
                     対処法: ~/.config/backup-suite/config.toml を確認してください。"
                )
            }
            BackupError::FileCopyError { from, to } => {
                format!(
                    "ファイルコピーに失敗しました:\n\
                     元: {}\n\
                     先: {}\n\
                     対処法: ディスク容量と権限を確認してください。",
                    from.display(),
                    to.display()
                )
            }
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_recoverable() {
        let io_error =
            BackupError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        assert!(io_error.is_recoverable());

        let permission_error = BackupError::PermissionDenied {
            path: PathBuf::from("/test"),
        };
        assert!(!permission_error.is_recoverable());
    }

    #[test]
    fn test_error_is_security_related() {
        let path_traversal = BackupError::PathTraversalDetected {
            path: PathBuf::from("../../../etc/passwd"),
        };
        assert!(path_traversal.is_security_related());

        let io_error =
            BackupError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        assert!(!io_error.is_security_related());
    }

    #[test]
    fn test_user_friendly_message() {
        let error = BackupError::TargetNotFound {
            path: PathBuf::from("/nonexistent"),
        };
        let message = error.user_friendly_message();
        assert!(message.contains("対処法"));
        assert!(message.contains("/nonexistent"));
    }
}
