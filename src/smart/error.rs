//! Smart機能のエラー型定義
//!
//! thiserrorを使用した型安全なエラーハンドリングを提供します。

use thiserror::Error;

/// Smart機能のエラー型
#[derive(Error, Debug)]
pub enum SmartError {
    /// 統計計算エラー
    #[error("統計計算エラー: {0}")]
    StatisticsError(String),

    /// 予測モデルエラー
    #[error("予測モデルエラー: {0}")]
    PredictionError(String),

    /// データ不足エラー
    #[error("データ不足: 最低{required}件必要ですが、{actual}件しかありません")]
    InsufficientData {
        /// 必要なデータ数
        required: usize,
        /// 実際のデータ数
        actual: usize,
    },

    /// 無効なパラメータ
    #[error("無効なパラメータ: {0}")]
    InvalidParameter(String),

    /// 数値範囲外エラー
    #[error("数値が範囲外です: {value} (範囲: {min} - {max})")]
    OutOfRange {
        /// 入力値
        value: f64,
        /// 最小値
        min: f64,
        /// 最大値
        max: f64,
    },

    /// オーバーフロー/アンダーフローエラー
    #[error("数値演算エラー: {0}")]
    ArithmeticError(String),

    /// I/Oエラー
    #[error("I/Oエラー: {0}")]
    IoError(#[from] std::io::Error),

    /// セキュリティエラー（BackupErrorから変換）
    #[error("セキュリティエラー: {0}")]
    SecurityError(String),

    /// LLM通信エラー
    #[error("LLM通信エラー: {0}")]
    LlmCommunicationError(String),

    /// Ollama未インストールエラー
    #[error("Ollama未インストール: Ollamaがインストールされていません。https://ollama.ai からインストールしてください")]
    OllamaNotInstalled,

    /// その他のエラー
    #[error("Smartエラー: {0}")]
    Other(#[from] anyhow::Error),
}

impl From<crate::error::BackupError> for SmartError {
    fn from(err: crate::error::BackupError) -> Self {
        match err {
            crate::error::BackupError::PathTraversalDetected { path } => {
                SmartError::SecurityError(format!("パストラバーサル検出: {:?}", path))
            }
            crate::error::BackupError::PermissionDenied { path } => {
                SmartError::SecurityError(format!("権限エラー: {:?}", path))
            }
            crate::error::BackupError::IoError(e) => SmartError::IoError(e),
            other => SmartError::Other(anyhow::Error::new(other)),
        }
    }
}

impl SmartError {
    /// ユーザーフレンドリーなエラーメッセージを生成
    ///
    /// # 使用例
    ///
    /// ```rust
    /// use backup_suite::smart::error::SmartError;
    ///
    /// let error = SmartError::InsufficientData {
    ///     required: 10,
    ///     actual: 3,
    /// };
    /// let message = error.user_friendly_message();
    /// assert!(message.contains("データが不足"));
    /// ```
    #[must_use]
    pub fn user_friendly_message(&self) -> String {
        match self {
            SmartError::StatisticsError(_) | SmartError::PredictionError(_) => {
                format!("分析処理中にエラーが発生しました: {}", self)
            }
            SmartError::InsufficientData { required, actual } => {
                format!(
                    "分析に必要なデータが不足しています。最低{}件必要ですが、{}件しかありません。",
                    required, actual
                )
            }
            SmartError::InvalidParameter(msg) => {
                format!("設定値が不正です: {}", msg)
            }
            SmartError::OutOfRange { value, min, max } => {
                format!("値{}が許容範囲({} - {})外です", value, min, max)
            }
            SmartError::ArithmeticError(msg) => {
                format!("計算処理中にエラーが発生しました: {}", msg)
            }
            SmartError::IoError(e) => {
                format!("ファイル操作中にエラーが発生しました: {}", e)
            }
            SmartError::SecurityError(msg) => {
                format!("セキュリティエラー: {}", msg)
            }
            SmartError::LlmCommunicationError(msg) => {
                format!(
                    "AI推論エンジンとの通信に失敗しました: {}。Ollamaサービスが起動しているか確認してください。",
                    msg
                )
            }
            SmartError::OllamaNotInstalled => {
                "Ollamaがインストールされていません。AI機能を使用するには https://ollama.ai からOllamaをインストールしてください。".to_string()
            }
            SmartError::Other(e) => {
                format!("エラーが発生しました: {}", e)
            }
        }
    }

    /// ユーザーフレンドリーなエラーメッセージを生成（英語）
    ///
    /// # 使用例
    ///
    /// ```rust
    /// use backup_suite::smart::error::SmartError;
    ///
    /// let error = SmartError::OllamaNotInstalled;
    /// let message = error.user_friendly_message_en();
    /// assert!(message.contains("Ollama"));
    /// ```
    #[must_use]
    pub fn user_friendly_message_en(&self) -> String {
        match self {
            SmartError::StatisticsError(_) | SmartError::PredictionError(_) => {
                format!("An error occurred during analysis: {}", self)
            }
            SmartError::InsufficientData { required, actual } => {
                format!(
                    "Insufficient data for analysis. At least {} items required, but only {} available.",
                    required, actual
                )
            }
            SmartError::InvalidParameter(msg) => {
                format!("Invalid parameter: {}", msg)
            }
            SmartError::OutOfRange { value, min, max } => {
                format!("Value {} is out of range ({} - {})", value, min, max)
            }
            SmartError::ArithmeticError(msg) => {
                format!("Arithmetic error occurred: {}", msg)
            }
            SmartError::IoError(e) => {
                format!("I/O error occurred: {}", e)
            }
            SmartError::SecurityError(msg) => {
                format!("Security error: {}", msg)
            }
            SmartError::LlmCommunicationError(msg) => {
                format!(
                    "Failed to communicate with AI inference engine: {}. Please check if Ollama service is running.",
                    msg
                )
            }
            SmartError::OllamaNotInstalled => {
                "Ollama is not installed. Please install Ollama from https://ollama.ai to use AI features.".to_string()
            }
            SmartError::Other(e) => {
                format!("An error occurred: {}", e)
            }
        }
    }

    /// リトライ可能なエラーかどうかを判定
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            SmartError::IoError(_) | SmartError::LlmCommunicationError(_)
        )
    }

    /// 一時的なエラーかどうかを判定
    #[must_use]
    pub const fn is_transient(&self) -> bool {
        matches!(
            self,
            SmartError::IoError(_) | SmartError::LlmCommunicationError(_)
        )
    }
}

/// Smart機能のResult型エイリアス
pub type SmartResult<T> = std::result::Result<T, SmartError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insufficient_data_error() {
        let error = SmartError::InsufficientData {
            required: 10,
            actual: 3,
        };
        let msg = error.to_string();
        assert!(msg.contains("最低10件必要"));
        assert!(msg.contains("3件しか"));
    }

    #[test]
    fn test_out_of_range_error() {
        let error = SmartError::OutOfRange {
            value: 150.0,
            min: 0.0,
            max: 100.0,
        };
        let msg = error.to_string();
        assert!(msg.contains("150"));
        assert!(msg.contains("0"));
        assert!(msg.contains("100"));
    }

    #[test]
    fn test_user_friendly_message() {
        let error = SmartError::InvalidParameter("閾値が負の数です".to_string());
        let msg = error.user_friendly_message();
        assert!(msg.contains("設定値が不正"));
    }

    #[test]
    fn test_error_recovery() {
        let io_error =
            SmartError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        assert!(io_error.is_recoverable());
        assert!(io_error.is_transient());

        let stat_error = SmartError::StatisticsError("test".to_string());
        assert!(!stat_error.is_recoverable());
        assert!(!stat_error.is_transient());
    }

    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let smart_error: SmartError = io_error.into();
        assert!(matches!(smart_error, SmartError::IoError(_)));
    }

    #[test]
    fn test_llm_communication_error() {
        let error = SmartError::LlmCommunicationError("connection timeout".to_string());
        let msg = error.user_friendly_message();
        assert!(msg.contains("AI推論エンジン"));
        assert!(msg.contains("Ollama"));

        let msg_en = error.user_friendly_message_en();
        assert!(msg_en.contains("AI inference engine"));
        assert!(msg_en.contains("Ollama"));

        assert!(error.is_recoverable());
        assert!(error.is_transient());
    }

    #[test]
    fn test_ollama_not_installed() {
        let error = SmartError::OllamaNotInstalled;
        let msg = error.user_friendly_message();
        assert!(msg.contains("Ollama"));
        assert!(msg.contains("https://ollama.ai"));

        let msg_en = error.user_friendly_message_en();
        assert!(msg_en.contains("Ollama"));
        assert!(msg_en.contains("https://ollama.ai"));

        assert!(!error.is_recoverable());
        assert!(!error.is_transient());
    }

    #[test]
    fn test_user_friendly_message_en() {
        let error = SmartError::InsufficientData {
            required: 10,
            actual: 3,
        };
        let msg = error.user_friendly_message_en();
        assert!(msg.contains("Insufficient data"));
        assert!(msg.contains("10"));
        assert!(msg.contains("3"));
    }
}
