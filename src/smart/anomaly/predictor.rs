//! ディスク容量予測エンジン
//!
//! 線形回帰による容量予測を提供します。

use crate::core::history::BackupHistory;
use crate::smart::error::{SmartError, SmartResult};
use crate::smart::types::{DiskCapacity, PredictionConfidence, TimeSeriesPoint};
use statrs::statistics::{Data, Distribution};

/// 予測結果
///
/// # 使用例
///
/// ```rust
/// use backup_suite::smart::anomaly::PredictionResult;
/// use backup_suite::smart::{DiskCapacity, PredictionConfidence};
///
/// let result = PredictionResult::new(
///     DiskCapacity::new(500_000_000_000),
///     30,
///     PredictionConfidence::new(0.85).unwrap(),
///     Some("現在のペースでは約30日後に容量不足になります".to_string())
/// );
/// assert_eq!(result.predicted_capacity().get(), 500_000_000_000);
/// ```
#[derive(Debug, Clone)]
pub struct PredictionResult {
    predicted_capacity: DiskCapacity,
    days_until_full: i64,
    confidence: PredictionConfidence,
    warning_message: Option<String>,
}

impl PredictionResult {
    /// 新しい予測結果を作成
    #[must_use]
    pub const fn new(
        predicted_capacity: DiskCapacity,
        days_until_full: i64,
        confidence: PredictionConfidence,
        warning_message: Option<String>,
    ) -> Self {
        Self {
            predicted_capacity,
            days_until_full,
            confidence,
            warning_message,
        }
    }

    /// 予測容量を取得
    #[must_use]
    pub const fn predicted_capacity(&self) -> DiskCapacity {
        self.predicted_capacity
    }

    /// 満杯までの日数を取得
    #[must_use]
    pub const fn days_until_full(&self) -> i64 {
        self.days_until_full
    }

    /// 信頼度を取得
    #[must_use]
    pub const fn confidence(&self) -> PredictionConfidence {
        self.confidence
    }

    /// 警告メッセージを取得
    #[must_use]
    pub fn warning_message(&self) -> Option<&str> {
        self.warning_message.as_deref()
    }

    /// リスクレベルを判定
    #[must_use]
    pub fn risk_level(&self) -> RiskLevel {
        if self.days_until_full < 0 {
            RiskLevel::Critical
        } else if self.days_until_full < 7 {
            RiskLevel::High
        } else if self.days_until_full < 30 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }
}

/// リスクレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// 緊急（すでに容量不足）
    Critical,
    /// 高リスク（7日以内）
    High,
    /// 中リスク（30日以内）
    Medium,
    /// 低リスク（30日以上）
    Low,
}

impl RiskLevel {
    /// リスクレベルの説明を取得
    #[must_use]
    pub const fn description(&self) -> &str {
        match self {
            RiskLevel::Critical => "緊急",
            RiskLevel::High => "高リスク",
            RiskLevel::Medium => "中リスク",
            RiskLevel::Low => "低リスク",
        }
    }
}

/// 予測器
///
/// 線形回帰によるディスク容量予測を提供します。
///
/// # 使用例
///
/// ```rust,no_run
/// use backup_suite::smart::anomaly::Predictor;
/// use backup_suite::smart::DiskCapacity;
/// use backup_suite::BackupHistory;
///
/// let predictor = Predictor::new();
/// let histories = BackupHistory::load_all().unwrap();
/// let current_capacity = DiskCapacity::new(1_000_000_000_000); // 1TB
///
/// match predictor.predict_disk_usage(&histories, current_capacity, 30) {
///     Ok(Some(result)) => {
///         println!("30日後の予測使用量: {:.2}GB", result.predicted_capacity().as_gb());
///         println!("満杯までの日数: {}日", result.days_until_full());
///     }
///     Ok(None) => println!("予測に十分なデータがありません"),
///     Err(e) => eprintln!("エラー: {}", e),
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Predictor {
    min_data_points: usize,
}

impl Predictor {
    /// 新しい予測器を作成
    #[must_use]
    pub const fn new() -> Self {
        Self { min_data_points: 5 }
    }

    /// ディスク使用量の予測
    ///
    /// # 引数
    ///
    /// * `histories` - 過去のバックアップ履歴
    /// * `total_capacity` - ディスク総容量
    /// * `days_ahead` - 何日先を予測するか
    ///
    /// # 戻り値
    ///
    /// 予測結果。データが不足している場合は`None`を返します。
    ///
    /// # Errors
    ///
    /// 統計計算に失敗した場合はエラーを返します。
    pub fn predict_disk_usage(
        &self,
        histories: &[BackupHistory],
        total_capacity: DiskCapacity,
        days_ahead: u32,
    ) -> SmartResult<Option<PredictionResult>> {
        // データ不足チェック
        if histories.len() < self.min_data_points {
            return Ok(None);
        }

        // 成功したバックアップのみを対象
        let successful_histories: Vec<_> = histories.iter().filter(|h| h.success).collect();

        if successful_histories.len() < self.min_data_points {
            return Ok(None);
        }

        // 時系列データを作成
        let time_series: Vec<TimeSeriesPoint> = successful_histories
            .iter()
            .map(|h| TimeSeriesPoint::new(h.timestamp, h.total_bytes as f64))
            .collect();

        // 線形回帰
        let (slope, intercept, r_squared) = self.linear_regression(&time_series)?;

        // days_ahead日後のタイムスタンプ
        let future_timestamp = chrono::Utc::now() + chrono::Duration::days(days_ahead as i64);
        let future_x = future_timestamp.timestamp() as f64;

        // 予測値計算
        let predicted_bytes = slope * future_x + intercept;
        let predicted_capacity = DiskCapacity::new(predicted_bytes.max(0.0) as u64);

        // 信頼度計算（決定係数から導出）
        let confidence_value = r_squared.clamp(0.5, 0.99);
        let confidence =
            PredictionConfidence::new(confidence_value).map_err(SmartError::PredictionError)?;

        // 満杯までの日数を計算
        let days_until_full = if slope > 0.0 {
            let remaining_capacity = total_capacity.get() as f64 - predicted_bytes;
            let days_per_byte = 1.0 / slope;
            (remaining_capacity * days_per_byte / 86400.0) as i64 // 秒から日に変換
        } else {
            // 使用量が減少している場合は無限大
            i64::MAX
        };

        // 警告メッセージ生成
        let warning_message = if days_until_full < 0 {
            Some(
                "ディスク容量が不足しています。緊急に古いバックアップを削除してください。"
                    .to_string(),
            )
        } else if days_until_full < 7 {
            Some(format!(
                "約{}日後にディスク容量が不足する可能性があります。古いバックアップの削除を検討してください。",
                days_until_full
            ))
        } else if days_until_full < 30 {
            Some(format!(
                "約{}日後にディスク容量が不足する可能性があります。",
                days_until_full
            ))
        } else {
            None
        };

        Ok(Some(PredictionResult::new(
            predicted_capacity,
            days_until_full,
            confidence,
            warning_message,
        )))
    }

    /// 線形回帰（最小二乗法）
    ///
    /// # 戻り値
    ///
    /// (傾き, 切片, 決定係数)
    ///
    /// # Errors
    ///
    /// データ不足または計算エラーの場合はエラーを返します。
    fn linear_regression(&self, time_series: &[TimeSeriesPoint]) -> SmartResult<(f64, f64, f64)> {
        if time_series.len() < 2 {
            return Err(SmartError::InsufficientData {
                required: 2,
                actual: time_series.len(),
            });
        }

        // タイムスタンプをX、バイト数をYとする
        let mut x_values: Vec<f64> = time_series
            .iter()
            .map(|p| p.timestamp().timestamp() as f64)
            .collect();
        let mut y_values: Vec<f64> = time_series.iter().map(|p| p.value()).collect();

        let x_data = Data::new(&mut x_values[..]);
        let y_data = Data::new(&mut y_values[..]);

        let x_mean = x_data.mean().unwrap_or(0.0);
        let y_mean = y_data.mean().unwrap_or(0.0);

        // 共分散と分散を計算
        let mut covariance = 0.0;
        let mut variance = 0.0;

        for (x, y) in x_values.iter().zip(y_values.iter()) {
            covariance += (x - x_mean) * (y - y_mean);
            variance += (x - x_mean) * (x - x_mean);
        }

        // 分散が0の場合（全て同じタイムスタンプ）
        if variance == 0.0 {
            return Err(SmartError::StatisticsError(
                "分散が0です（全てのデータポイントが同じタイムスタンプ）".to_string(),
            ));
        }

        // 傾きと切片を計算
        let slope = covariance / variance;
        let intercept = y_mean - slope * x_mean;

        // 決定係数（R²）を計算
        let mut ss_total = 0.0;
        let mut ss_residual = 0.0;

        for (x, y) in x_values.iter().zip(y_values.iter()) {
            let predicted = slope * x + intercept;
            ss_residual += (y - predicted) * (y - predicted);
            ss_total += (y - y_mean) * (y - y_mean);
        }

        let r_squared: f64 = if ss_total == 0.0 {
            1.0
        } else {
            1.0 - (ss_residual / ss_total)
        };

        Ok((slope, intercept, r_squared.max(0.0)))
    }

    /// トレンド分析（増加/減少/安定）
    ///
    /// # Errors
    ///
    /// データ不足または計算エラーの場合はエラーを返します。
    pub fn analyze_trend(&self, histories: &[BackupHistory]) -> SmartResult<Option<Trend>> {
        if histories.len() < self.min_data_points {
            return Ok(None);
        }

        let successful_histories: Vec<_> = histories.iter().filter(|h| h.success).collect();

        if successful_histories.len() < self.min_data_points {
            return Ok(None);
        }

        let time_series: Vec<TimeSeriesPoint> = successful_histories
            .iter()
            .map(|h| TimeSeriesPoint::new(h.timestamp, h.total_bytes as f64))
            .collect();

        let (slope, _, _) = self.linear_regression(&time_series)?;

        // 傾きの閾値（1日あたり1MB = 1,048,576 bytes/86400秒 ≈ 12.1 bytes/秒）
        let threshold = 12.1;

        let trend = if slope > threshold {
            Trend::Increasing
        } else if slope < -threshold {
            Trend::Decreasing
        } else {
            Trend::Stable
        };

        Ok(Some(trend))
    }
}

impl Default for Predictor {
    fn default() -> Self {
        Self::new()
    }
}

/// トレンド
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    /// 増加傾向
    Increasing,
    /// 減少傾向
    Decreasing,
    /// 安定
    Stable,
}

impl Trend {
    /// トレンドの説明を取得
    #[must_use]
    pub const fn description(&self) -> &str {
        match self {
            Trend::Increasing => "増加傾向",
            Trend::Decreasing => "減少傾向",
            Trend::Stable => "安定",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use std::path::PathBuf;

    fn create_mock_history_with_timestamp(size: u64, days_ago: i64) -> BackupHistory {
        let mut history =
            BackupHistory::new(PathBuf::from("/tmp/backup"), 100, size, true, false, false);
        history.timestamp = Utc::now() - Duration::days(days_ago);
        history
    }

    #[test]
    fn test_predictor_insufficient_data() {
        let predictor = Predictor::new();
        let histories = vec![
            create_mock_history_with_timestamp(1000, 3),
            create_mock_history_with_timestamp(1100, 2),
        ];
        let capacity = DiskCapacity::new(10_000_000_000);

        let result = predictor
            .predict_disk_usage(&histories, capacity, 30)
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_predictor_stable_usage() {
        let predictor = Predictor::new();
        let histories = vec![
            create_mock_history_with_timestamp(1_000_000, 10),
            create_mock_history_with_timestamp(1_000_000, 8),
            create_mock_history_with_timestamp(1_000_000, 6),
            create_mock_history_with_timestamp(1_000_000, 4),
            create_mock_history_with_timestamp(1_000_000, 2),
        ];
        let capacity = DiskCapacity::new(100_000_000);

        let result = predictor
            .predict_disk_usage(&histories, capacity, 30)
            .unwrap()
            .unwrap();
        // 安定しているため、満杯までの日数は非常に長い
        assert!(result.days_until_full() > 100);
    }

    #[test]
    fn test_trend_analysis() {
        let predictor = Predictor::new();

        // 増加傾向（古い順に並べる: 50日前 → 10日前、1日あたり1MB以上増加）
        // 1日あたりの増加: 約1MB (1_048_576 bytes) → 秒あたり約12.1 bytes
        // 実際の増加: 40日で100MB → 1日あたり2.5MB → 秒あたり約30 bytes
        let increasing_histories = vec![
            create_mock_history_with_timestamp(10_000_000, 50),
            create_mock_history_with_timestamp(30_000_000, 40),
            create_mock_history_with_timestamp(50_000_000, 30),
            create_mock_history_with_timestamp(80_000_000, 20),
            create_mock_history_with_timestamp(110_000_000, 10),
        ];

        let trend = predictor
            .analyze_trend(&increasing_histories)
            .unwrap()
            .unwrap();
        assert_eq!(trend, Trend::Increasing);
    }

    #[test]
    fn test_risk_level() {
        let confidence = PredictionConfidence::new(0.8).unwrap();

        let critical = PredictionResult::new(DiskCapacity::new(1000), -1, confidence, None);
        assert_eq!(critical.risk_level(), RiskLevel::Critical);

        let high = PredictionResult::new(DiskCapacity::new(1000), 5, confidence, None);
        assert_eq!(high.risk_level(), RiskLevel::High);

        let medium = PredictionResult::new(DiskCapacity::new(1000), 20, confidence, None);
        assert_eq!(medium.risk_level(), RiskLevel::Medium);

        let low = PredictionResult::new(DiskCapacity::new(1000), 100, confidence, None);
        assert_eq!(low.risk_level(), RiskLevel::Low);
    }
}
