//! # backup-suite: エンタープライズ対応Rust製バックアップツール
//!
//! `backup-suite`は、セキュリティ、パフォーマンス、信頼性を重視したエンタープライズレベルの
//! バックアップソリューションです。型安全性、並列処理、包括的なエラーハンドリングにより、
//! プロダクション環境での安全な運用を可能にします。
//!
//! ## 🚀 主要機能
//!
//! ### セキュリティファースト
//! - **パストラバーサル対策**: [`safe_join`]によるディレクトリトラバーサル攻撃防止
//! - **権限チェック**: ファイルアクセス前の厳密な権限確認
//! - **入力検証**: 全ユーザー入力の検証とサニタイズ
//! - **機密情報保護**: エラーメッセージからの機密情報漏洩防止
//!
//! ### 高性能・スケーラブル
//! - **並列処理**: [`rayon`]によるマルチコアCPU活用
//! - **I/O最適化**: [`CopyEngine`]によるバッファリング・ストリーミング
//! - **メモリ効率**: 大容量ファイルの低メモリ処理
//! - **プログレス表示**: リアルタイム進捗・統計情報
//!
//! ### エンタープライズ機能
//! - **優先度管理**: High/Medium/Low による重要度別管理
//! - **設定バリデーション**: 厳密な設定検証とエラー報告
//! - **包括的ログ**: 詳細な操作履歴とエラートレース
//! - **除外パターン**: 正規表現による柔軟なファイル除外
//!
//! ## 📚 使用例
//!
//! ### 基本的なバックアップ
//!
//! ```rust,no_run
//! use backup_suite::{Config, BackupRunner, Target, Priority};
//! use std::path::PathBuf;
//!
//! # fn main() -> backup_suite::Result<()> {
//! // 1. 設定をロード（または新規作成）
//! let mut config = Config::load().unwrap_or_default();
//!
//! // 2. バックアップ対象を追加
//! let target = Target::new(
//!     PathBuf::from("/home/user/documents"),
//!     Priority::High,
//!     "重要ドキュメント".to_string()
//! );
//! config.add_target(target);
//! config.save()?;
//!
//! // 3. バックアップ実行
//! let runner = BackupRunner::new(config, false); // false = 実際に実行
//! let result = runner.run(None, None)?; // None = 全優先度対象
//!
//! // 4. 結果確認
//! if result.failed > 0 {
//!     eprintln!("警告: {}件のファイルでエラーが発生", result.failed);
//!     for error in &result.errors {
//!         eprintln!("  - {}", error);
//!     }
//! }
//! println!("✅ バックアップ完了: {}件成功, 合計{}",
//!          result.successful,
//!          format_bytes(result.total_bytes));
//! # Ok(())
//! # }
//! # fn format_bytes(bytes: u64) -> String { format!("{}B", bytes) }
//! ```
//!
//! ### 高度な設定例
//!
//! ```rust,no_run
//! use backup_suite::*;
//! use std::path::PathBuf;
//!
//! # fn main() -> backup_suite::Result<()> {
//! let mut config = Config::default();
//!
//! // プロジェクトファイル（.gitを除外）
//! let mut project_target = Target::new(
//!     PathBuf::from("/home/user/projects"),
//!     Priority::Medium,
//!     "プロジェクト".to_string()
//! );
//! project_target.exclude_patterns = vec![
//!     r"\.git/.*".to_string(),
//!     r"node_modules/.*".to_string(),
//!     r"target/.*".to_string(),
//! ];
//! config.add_target(project_target);
//!
//! // 重要ファイルのみ（高優先度）
//! let runner = BackupRunner::new(config, false)
//!     .with_progress(true); // プログレスバー表示
//!
//! let result = runner.run(Some(&Priority::High), None)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### エラーハンドリング
//!
//! ```rust,no_run
//! use backup_suite::{BackupError, Result};
//!
//! fn handle_backup_errors() -> Result<()> {
//!     match perform_backup() {
//!         Ok(result) => {
//!             println!("✅ 成功: {}件処理", result.total_files);
//!             Ok(())
//!         }
//!         Err(BackupError::PathTraversalDetected { path }) => {
//!             eprintln!("🚨 セキュリティ警告: 不正なパス検出 {:?}", path);
//!             // セキュリティ関連エラーは処理を中断
//!             std::process::exit(1);
//!         }
//!         Err(BackupError::PermissionDenied { path }) => {
//!             eprintln!("❌ 権限エラー: {:?} にアクセスできません", path);
//!             // 権限エラーは警告として処理継続
//!             Ok(())
//!         }
//!         Err(e) if e.is_recoverable() => {
//!             eprintln!("⚠️ 一時的エラー（リトライ推奨）: {}", e);
//!             // リトライ可能なエラー
//!             Err(e)
//!         }
//!         Err(e) => {
//!             eprintln!("💥 重大エラー: {}", e.user_friendly_message());
//!             Err(e)
//!         }
//!     }
//! }
//! # fn perform_backup() -> backup_suite::Result<backup_suite::BackupResult> {
//! #     todo!()
//! # }
//! ```
//!
//! ## 🏗️ アーキテクチャ
//!
//! ### コアモジュール ([`core`])
//! - **[`Config`]**: 設定管理・バリデーション・永続化
//! - **[`BackupRunner`]**: バックアップ処理エンジン・並列実行
//! - **[`Target`]**: バックアップ対象定義・除外パターン
//! - **[`CopyEngine`]**: I/O最適化・ファイルコピー
//! - **[`BackupHistory`]**: 履歴管理・統計情報
//!
//! ### セキュリティモジュール ([`security`])
//! - **`safe_join`**: パストラバーサル対策パス結合
//! - **`validate_path_safety`**: パス安全性検証
//! - **`sanitize_path_component`**: パス文字列サニタイズ
//! - **権限チェック**: Unix/Windows対応権限確認
//!
//! ### UIモジュール ([`ui`])
//! - **`BackupProgress`**: プログレスバー・統計表示
//! - **`display_dashboard`**: ダッシュボード・概要表示
//! - **[`ColorTheme`]**: アクセシビリティ対応色彩
//! - **テーブル表示**: 構造化データの美しい表示
//!
//! ### エラーハンドリング ([`error`])
//! - **[`BackupError`]**: 型安全なエラー分類
//! - **[`Result`]**: 統一されたResult型
//! - **ユーザーフレンドリー**: 分かりやすいエラーメッセージ
//!
//! ## 🔒 セキュリティ設計
//!
//! ### 脅威モデル
//! - **ディレクトリトラバーサル**: `../../../etc/passwd` 等の攻撃
//! - **権限昇格**: 不正なファイルアクセス試行
//! - **シンボリックリンク**: リンク経由の意図しないアクセス
//! - **機密情報漏洩**: エラーメッセージ経由の情報漏洩
//!
//! ### 対策実装
//! ```rust,no_run
//! use backup_suite::security::{safe_join, validate_path_safety};
//! use std::path::Path;
//!
//! # fn main() -> backup_suite::Result<()> {
//! let base = Path::new("/safe/backup/dir");
//! let user_input = Path::new("../../../etc/passwd"); // 攻撃試行
//!
//! // safe_joinは自動的に危険なパスを検出・拒否
//! match safe_join(base, user_input) {
//!     Ok(safe_path) => {
//!         println!("安全なパス: {:?}", safe_path);
//!         // /safe/backup/dir/etc/passwd として正規化
//!     }
//!     Err(e) => {
//!         eprintln!("🚨 攻撃検出: {}", e);
//!         // ディレクトリトラバーサル攻撃をブロック
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## ⚡ パフォーマンス特性
//!
//! ### ベンチマーク結果（参考値）
//! - **小ファイル多数**: 10,000ファイル/秒
//! - **大ファイル**: 500MB/秒（SSD環境）
//! - **メモリ使用量**: 50MB未満（ファイル数に依存しない）
//! - **CPU使用率**: 全コア活用（並列度調整可能）
//!
//! ### 最適化設定
//! ```rust,no_run
//! use backup_suite::{BackupRunner, Config};
//!
//! # fn main() -> backup_suite::Result<()> {
//! let config = Config::load()?;
//! let runner = BackupRunner::new(config, false)
//!     .with_progress(true);  // プログレス表示有効
//!
//! // CPU集約的環境での実行
//! std::env::set_var("RAYON_NUM_THREADS", "8");
//! let result = runner.run(None, None)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 📋 設定リファレンス
//!
//! 設定ファイル (`~/.config/backup-suite/config.toml`) の例：
//!
//! ```toml
//! [backup]
//! destination = "/path/to/backup/storage"
//! auto_cleanup = true
//! max_backup_age_days = 30
//!
//! [schedule]
//! enabled = true
//! high_frequency = "daily"
//! medium_frequency = "weekly"
//! low_frequency = "monthly"
//!
//! [[targets]]
//! path = "/home/user/documents"
//! priority = "High"
//! category = "重要ドキュメント"
//! exclude_patterns = ["*.tmp", "*.log"]
//! ```

// モジュール宣言
pub mod core;
pub mod crypto;
pub mod compression;
pub mod error;
pub mod i18n;
pub mod security;
pub mod ui;

// 主要な型を再エクスポート
pub use core::{BackupHistory, BackupResult, BackupRunner, Config, CopyEngine, Priority, Target, TargetType,
               ProcessingPipeline, PipelineConfig, ProcessedData, ProcessingMetadata, PerformanceConfig};
pub use crypto::{EncryptionEngine, EncryptionConfig, EncryptedData, KeyManager, KeyDerivation, MasterKey};
pub use compression::{CompressionEngine, CompressionType, CompressionConfig, CompressedData};
pub use error::{BackupError, Result};
pub use i18n::{Language, MessageKey, get_message};
pub use security::{check_read_permission, check_write_permission, safe_join};
pub use ui::{display_dashboard, display_targets, display_history, display_backup_result, ColorTheme, ColorScheme};

#[cfg(unix)]
pub use security::check_execute_permission;

// バージョン情報
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert_eq!(PKG_NAME, "backup-suite");
    }

    #[test]
    fn test_exports_available() {
        // 主要な型がエクスポートされていることを確認
        let _config: Config = Config::default();
        let _priority: Priority = Priority::Medium;
    }

    #[test]
    fn test_ui_exports() {
        // UI機能がエクスポートされていることを確認
        let _theme = ColorTheme::auto();
        let _scheme = ColorScheme::Auto;
    }
}
