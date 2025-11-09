//! 異常検知エンジン
//!
//! Z-score、移動平均、線形回帰による統計的異常検知を提供します。

pub mod detector;
pub mod pattern;
pub mod predictor;

pub use detector::{AnomalyDetector, AnomalyResult, AnomalyThreshold};
pub use pattern::PatternAnalyzer;
pub use predictor::{PredictionResult, Predictor, RiskLevel, Trend};
