//! AI機能モジュール
//!
//! 統計的異常検知、ファイル重要度判定、インテリジェント推奨を提供します。

pub mod anomaly;
pub mod error;
pub mod recommendation;
pub mod types;

// 公開API
pub use error::{AiError, AiResult};
pub use types::{
    BackupSize, DiskCapacity, FailureRate, FileImportance, PredictionConfidence, TimeSeriesPoint,
};

// 異常検知エンジン
pub use anomaly::{
    AnomalyDetector, AnomalyResult, AnomalyThreshold, PatternAnalyzer, PredictionResult, Predictor,
};

// 推奨エンジン
pub use recommendation::{
    BackupSuggestion, ExcludeRecommendation, ExcludeRecommendationEngine, FileImportanceResult,
    ImportanceEvaluator, SuggestEngine,
};
