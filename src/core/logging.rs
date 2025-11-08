//! ログファイル管理モジュール
//!
//! バックアップツールのログ機能を提供します。
//!
//! # 機能
//!
//! - 構造化ログ（TEXT/JSON形式）
//! - ログローテーション（日次）
//! - ログレベル制御（DEBUG/INFO/WARN/ERROR）
//! - 自動クリーンアップ（保持日数設定）
//!
//! # 使用例
//!
//! ```no_run
//! use backup_suite::core::logging::{Logger, LogLevel, LogFormat};
//!
//! let logger = Logger::new(LogLevel::Info, LogFormat::Text)?;
//! logger.info("バックアップ開始");
//! logger.error("エラーが発生しました");
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// ログレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    /// デバッグレベル（詳細な診断情報）
    Debug,
    /// 情報レベル（一般的な情報メッセージ）
    Info,
    /// 警告レベル（警告メッセージ）
    Warn,
    /// エラーレベル（エラーメッセージ）
    Error,
}

impl LogLevel {
    /// 文字列から変換
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(anyhow::anyhow!("不明なログレベル: {s}")),
        }
    }

    /// 文字列表現を取得
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// ログフォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    /// プレーンテキスト形式
    Text,
    /// JSON形式
    Json,
}

impl LogFormat {
    /// 文字列から変換
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "text" | "plain" => Ok(LogFormat::Text),
            "json" => Ok(LogFormat::Json),
            _ => Err(anyhow::anyhow!("不明なログフォーマット: {s}")),
        }
    }
}

/// ログエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// ログレベル
    pub level: LogLevel,
    /// メッセージ
    pub message: String,
    /// 追加メタデータ（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl LogEntry {
    /// 新しいログエントリを作成
    #[must_use]
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            message: message.into(),
            metadata: None,
        }
    }

    /// メタデータを追加
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// テキスト形式でフォーマット
    #[must_use]
    pub fn format_text(&self) -> String {
        let timestamp = self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
        if let Some(ref meta) = self.metadata {
            format!(
                "[{}] {} {} | {}",
                timestamp,
                self.level.as_str(),
                self.message,
                serde_json::to_string(meta).unwrap_or_default()
            )
        } else {
            format!("[{}] {} {}", timestamp, self.level.as_str(), self.message)
        }
    }

    /// JSON形式でフォーマット
    pub fn format_json(&self) -> Result<String> {
        serde_json::to_string(self).context("JSON変換エラー")
    }
}

/// ロガー
pub struct Logger {
    /// ログレベル閾値
    level: LogLevel,
    /// ログフォーマット
    format: LogFormat,
    /// ログファイルパス
    log_file: PathBuf,
    /// ローテーション保持日数
    rotation_days: u32,
}

impl Logger {
    /// 新しいロガーを作成
    ///
    /// # 引数
    ///
    /// * `level` - ログレベル閾値
    /// * `format` - ログフォーマット
    ///
    /// # 戻り値
    ///
    /// Loggerインスタンス
    ///
    /// # エラー
    ///
    /// - ログディレクトリの作成に失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::core::logging::{Logger, LogLevel, LogFormat};
    ///
    /// let logger = Logger::new(LogLevel::Info, LogFormat::Text)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(level: LogLevel, format: LogFormat) -> Result<Self> {
        let log_dir = Self::log_dir()?;
        fs::create_dir_all(&log_dir).context("ログディレクトリ作成エラー")?;

        let log_file = log_dir.join("backup.log");

        Ok(Self {
            level,
            format,
            log_file,
            rotation_days: 7,
        })
    }

    /// カスタム設定でロガーを作成
    ///
    /// # 引数
    ///
    /// * `level` - ログレベル閾値
    /// * `format` - ログフォーマット
    /// * `log_file` - ログファイルパス
    /// * `rotation_days` - ローテーション保持日数
    ///
    /// # 戻り値
    ///
    /// Loggerインスタンス
    ///
    /// # エラー
    ///
    /// - ログディレクトリの作成に失敗した場合
    pub fn with_config(
        level: LogLevel,
        format: LogFormat,
        log_file: PathBuf,
        rotation_days: u32,
    ) -> Result<Self> {
        if let Some(parent) = log_file.parent() {
            fs::create_dir_all(parent).context("ログディレクトリ作成エラー")?;
        }

        Ok(Self {
            level,
            format,
            log_file,
            rotation_days,
        })
    }

    /// ログディレクトリのパスを取得
    fn log_dir() -> Result<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            let home = dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("ホームディレクトリが見つかりません"))?;
            Ok(home.join("Library/Logs/backup-suite"))
        }

        #[cfg(not(target_os = "macos"))]
        {
            let data_dir = dirs::data_local_dir()
                .ok_or_else(|| anyhow::anyhow!("データディレクトリが見つかりません"))?;
            Ok(data_dir.join("backup-suite").join("logs"))
        }
    }

    /// ログエントリを書き込み
    fn write_entry(&self, entry: &LogEntry) -> Result<()> {
        // ログレベルフィルタリング
        if entry.level < self.level {
            return Ok(());
        }

        // ローテーションチェック
        self.rotate_if_needed()?;

        // ログファイルに書き込み
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .context("ログファイルオープンエラー")?;

        let line = match self.format {
            LogFormat::Text => entry.format_text(),
            LogFormat::Json => entry.format_json()?,
        };

        writeln!(file, "{line}").context("ログ書き込みエラー")?;

        Ok(())
    }

    /// ログローテーションを実行（必要に応じて）
    fn rotate_if_needed(&self) -> Result<()> {
        if !self.log_file.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(&self.log_file).context("ログファイルメタデータ取得エラー")?;
        let modified = metadata.modified().context("最終更新日時取得エラー")?;

        let modified_datetime: DateTime<Utc> = modified.into();
        let now = Utc::now();
        let days_old = (now - modified_datetime).num_days();

        // 1日以上経過していたらローテーション
        if days_old >= 1 {
            let rotated_name = format!("backup-{}.log", modified_datetime.format("%Y%m%d"));
            let rotated_path = self.log_file.parent().unwrap().join(rotated_name);

            fs::rename(&self.log_file, &rotated_path).context("ログローテーションエラー")?;

            // 古いログファイルを削除
            self.cleanup_old_logs()?;
        }

        Ok(())
    }

    /// 古いログファイルをクリーンアップ
    fn cleanup_old_logs(&self) -> Result<()> {
        let log_dir = self.log_file.parent().unwrap();
        let cutoff_date = Utc::now() - chrono::Duration::days(self.rotation_days as i64);

        for entry in fs::read_dir(log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                // backup-YYYYMMDD.log 形式のファイルのみ処理
                if !file_name.starts_with("backup-") || !file_name.ends_with(".log") {
                    continue;
                }

                let metadata = fs::metadata(&path)?;
                let modified = metadata.modified()?;
                let modified_datetime: DateTime<Utc> = modified.into();

                if modified_datetime < cutoff_date {
                    fs::remove_file(&path)
                        .context("古いログファイル削除エラー: path.display()".to_string())?;
                }
            }
        }

        Ok(())
    }

    /// DEBUGレベルのログを出力
    pub fn debug(&self, message: impl Into<String>) {
        let _ = self.write_entry(&LogEntry::new(LogLevel::Debug, message));
    }

    /// INFOレベルのログを出力
    pub fn info(&self, message: impl Into<String>) {
        let _ = self.write_entry(&LogEntry::new(LogLevel::Info, message));
    }

    /// WARNレベルのログを出力
    pub fn warn(&self, message: impl Into<String>) {
        let _ = self.write_entry(&LogEntry::new(LogLevel::Warn, message));
    }

    /// ERRORレベルのログを出力
    pub fn error(&self, message: impl Into<String>) {
        let _ = self.write_entry(&LogEntry::new(LogLevel::Error, message));
    }

    /// メタデータ付きでログを出力
    pub fn log_with_metadata(
        &self,
        level: LogLevel,
        message: impl Into<String>,
        metadata: serde_json::Value,
    ) {
        let entry = LogEntry::new(level, message).with_metadata(metadata);
        let _ = self.write_entry(&entry);
    }

    /// ログファイルパスを取得
    #[must_use]
    pub fn log_file_path(&self) -> &Path {
        &self.log_file
    }

    /// ログレベルを取得
    #[must_use]
    pub fn level(&self) -> LogLevel {
        self.level
    }

    /// ログフォーマットを取得
    #[must_use]
    pub fn format(&self) -> LogFormat {
        self.format
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::parse("debug").unwrap(), LogLevel::Debug);
        assert_eq!(LogLevel::parse("INFO").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::parse("warn").unwrap(), LogLevel::Warn);
        assert_eq!(LogLevel::parse("ERROR").unwrap(), LogLevel::Error);
        assert!(LogLevel::parse("invalid").is_err());
    }

    #[test]
    fn test_log_format_from_str() {
        assert_eq!(LogFormat::parse("text").unwrap(), LogFormat::Text);
        assert_eq!(LogFormat::parse("json").unwrap(), LogFormat::Json);
        assert!(LogFormat::parse("invalid").is_err());
    }

    #[test]
    fn test_log_entry_format_text() {
        let entry = LogEntry::new(LogLevel::Info, "テストメッセージ");
        let formatted = entry.format_text();
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("テストメッセージ"));
    }

    #[test]
    fn test_log_entry_format_json() {
        let entry = LogEntry::new(LogLevel::Warn, "警告メッセージ");
        let formatted = entry.format_json().unwrap();
        assert!(formatted.contains("\"level\":\"Warn\""));
        assert!(formatted.contains("警告メッセージ"));
    }

    #[test]
    fn test_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("test.log");

        let logger =
            Logger::with_config(LogLevel::Info, LogFormat::Text, log_file.clone(), 7).unwrap();

        assert_eq!(logger.level(), LogLevel::Info);
        assert_eq!(logger.format(), LogFormat::Text);
    }

    #[test]
    fn test_logger_write() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("test.log");

        let logger =
            Logger::with_config(LogLevel::Info, LogFormat::Text, log_file.clone(), 7).unwrap();

        logger.info("テストログ");
        logger.debug("このログは出力されない"); // レベルがInfoなので

        let content = fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("テストログ"));
        assert!(!content.contains("このログは出力されない"));
    }

    #[test]
    fn test_logger_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("test.log");

        let logger =
            Logger::with_config(LogLevel::Debug, LogFormat::Json, log_file.clone(), 7).unwrap();

        logger.error("JSONエラーログ");

        let content = fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("\"level\":\"Error\""));
        assert!(content.contains("JSONエラーログ"));
    }

    #[test]
    fn test_logger_with_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("test.log");

        let logger =
            Logger::with_config(LogLevel::Info, LogFormat::Text, log_file.clone(), 7).unwrap();

        let metadata = serde_json::json!({
            "file_count": 42,
            "bytes": 1_024_000
        });

        logger.log_with_metadata(LogLevel::Info, "バックアップ完了", metadata);

        let content = fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("バックアップ完了"));
        assert!(content.contains("file_count"));
    }
}
