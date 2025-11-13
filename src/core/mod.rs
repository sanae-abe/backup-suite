//! # コアモジュール
//!
//! `backup-suite` の中核となるバックアップ機能を提供します。
//!
//! # モジュール構成
//!
//! - **[`backup`]**: バックアップ実行エンジンと結果
//! - **[`config`]**: 設定管理と永続化
//! - **[`copy_engine`]**: 最適化されたファイルコピー
//! - **[`filter`]**: ファイル除外パターン
//! - **[`history`]**: バックアップ履歴管理
//! - **[`logging`]**: ログファイル管理
//! - **[`pipeline`]**: 処理パイプライン（暗号化・圧縮）
//! - **[`scheduler`]**: スケジューリング機能（macOS/Linux）
//! - **[`target`]**: バックアップ対象定義
//! - **[`validation`]**: 入力検証とセキュリティ対策
//!
//! # 使用例
//!
//! ```no_run
//! use backup_suite::core::{Config, BackupRunner, Target, Priority};
//! use std::path::PathBuf;
//!
//! // 設定を作成
//! let mut config = Config::default();
//!
//! // バックアップ対象を追加
//! let target = Target::new(
//!     PathBuf::from("/home/user/documents"),
//!     Priority::High,
//!     "重要ドキュメント".to_string()
//! );
//! config.add_target(target);
//!
//! // バックアップを実行
//! let mut runner = BackupRunner::new(config, false);
//! let result = runner.run(None, None).unwrap();
//!
//! println!("成功: {}件, 失敗: {}件", result.successful, result.failed);
//! ```

pub mod backup;
pub mod cleanup;
pub mod config;
pub mod copy_engine;
pub mod filter;
pub mod history;
pub mod incremental;
pub mod integrity;
pub mod logging;
pub mod pipeline;
pub mod restore;
pub mod scheduler;
pub mod target;
pub mod validation;

pub use backup::{BackupResult, BackupRunner};
pub use cleanup::{CleanupEngine, CleanupPolicy, CleanupResult};
pub use config::Config;
pub use copy_engine::CopyEngine;
pub use filter::{default_exclude_patterns, FileFilter};
pub use history::BackupHistory;
pub use incremental::{resolve_backup_chain, BackupType, IncrementalBackupEngine};
pub use integrity::{BackupMetadata, IntegrityChecker};
pub use logging::{LogEntry, LogFormat, LogLevel, Logger};
pub use pipeline::{
    PerformanceConfig, PipelineConfig, ProcessedData, ProcessingMetadata, ProcessingPipeline,
};
pub use restore::{RestoreEngine, RestoreResult};
pub use scheduler::{Frequency, Platform, ScheduleStatus, Scheduler};
pub use target::{Priority, Target, TargetType};
