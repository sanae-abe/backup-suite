//! Z-score異常検知エンジン
//!
//! 統計的手法によるバックアップサイズの異常検知を提供します。

use crate::core::history::BackupHistory;
use crate::smart::error::{SmartError, SmartResult};
use crate::smart::types::{BackupSize, PredictionConfidence};
use statrs::statistics::{Data, Distribution};

/// 異常検知閾値
///
/// # 使用例
///
/// ```rust
/// use backup_suite::smart::anomaly::AnomalyThreshold;
///
/// let threshold = AnomalyThreshold::default();
/// assert_eq!(threshold.z_score(), 3.0);
/// assert_eq!(threshold.window_size(), 7);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct AnomalyThreshold {
    z_score: f64,
    window_size: usize,
}

impl AnomalyThreshold {
    /// 新しい閾値を作成
    ///
    /// # Errors
    ///
    /// Z-scoreが負の値、またはウィンドウサイズが0の場合はエラーを返します
    pub fn new(z_score: f64, window_size: usize) -> SmartResult<Self> {
        if z_score < 0.0 {
            return Err(SmartError::InvalidParameter(format!(
                "Z-scoreは正の値である必要があります: {}",
                z_score
            )));
        }
        if window_size == 0 {
            return Err(SmartError::InvalidParameter(
                "ウィンドウサイズは1以上である必要があります".to_string(),
            ));
        }
        Ok(Self {
            z_score,
            window_size,
        })
    }

    /// Z-score閾値を取得
    #[must_use]
    pub const fn z_score(&self) -> f64 {
        self.z_score
    }

    /// ウィンドウサイズを取得
    #[must_use]
    pub const fn window_size(&self) -> usize {
        self.window_size
    }
}

impl Default for AnomalyThreshold {
    fn default() -> Self {
        Self {
            z_score: 3.0,   // 3σ（99.7%信頼区間）
            window_size: 7, // 7日間
        }
    }
}

/// 異常検知結果
///
/// # 使用例
///
/// ```rust
/// use backup_suite::smart::anomaly::AnomalyResult;
/// use backup_suite::smart::PredictionConfidence;
///
/// let result = AnomalyResult::new(
///     true,
///     3.5,
///     PredictionConfidence::new(0.95).unwrap(),
///     "サイズが通常の3倍です".to_string(),
///     Some("一時ファイルを確認してください".to_string())
/// );
/// assert!(result.is_anomaly());
/// ```
#[derive(Debug, Clone)]
pub struct AnomalyResult {
    is_anomaly: bool,
    z_score: f64,
    confidence: PredictionConfidence,
    description: String,
    recommended_action: Option<String>,
}

impl AnomalyResult {
    /// 新しい異常検知結果を作成
    #[must_use]
    pub const fn new(
        is_anomaly: bool,
        z_score: f64,
        confidence: PredictionConfidence,
        description: String,
        recommended_action: Option<String>,
    ) -> Self {
        Self {
            is_anomaly,
            z_score,
            confidence,
            description,
            recommended_action,
        }
    }

    /// 異常かどうか
    #[must_use]
    pub const fn is_anomaly(&self) -> bool {
        self.is_anomaly
    }

    /// Z-scoreを取得
    #[must_use]
    pub const fn z_score(&self) -> f64 {
        self.z_score
    }

    /// 信頼度を取得
    #[must_use]
    pub const fn confidence(&self) -> PredictionConfidence {
        self.confidence
    }

    /// 説明を取得
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 推奨アクションを取得
    #[must_use]
    pub fn recommended_action(&self) -> Option<&str> {
        self.recommended_action.as_deref()
    }
}

/// 異常検知器
///
/// Z-score法による統計的異常検知を提供します。
///
/// # 使用例
///
/// ```rust,no_run
/// use backup_suite::smart::anomaly::{AnomalyDetector, AnomalyThreshold};
/// use backup_suite::smart::BackupSize;
/// use backup_suite::BackupHistory;
///
/// let detector = AnomalyDetector::new(AnomalyThreshold::default());
/// let histories = BackupHistory::load_all().unwrap();
/// let current_size = BackupSize::new(10_000_000);
///
/// match detector.detect_size_anomaly(&histories, current_size) {
///     Ok(Some(result)) => {
///         if result.is_anomaly() {
///             println!("異常検出: {}", result.description());
///         }
///     }
///     Ok(None) => println!("データ不足"),
///     Err(e) => eprintln!("エラー: {}", e),
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AnomalyDetector {
    threshold: AnomalyThreshold,
}

impl AnomalyDetector {
    /// 新しい異常検知器を作成
    #[must_use]
    pub const fn new(threshold: AnomalyThreshold) -> Self {
        Self { threshold }
    }

    /// デフォルト設定で異常検知器を作成
    #[must_use]
    pub fn default_detector() -> Self {
        Self::new(AnomalyThreshold::default())
    }

    /// バックアップサイズの異常検知
    ///
    /// # 引数
    ///
    /// * `histories` - 過去のバックアップ履歴
    /// * `current_size` - 現在のバックアップサイズ
    ///
    /// # 戻り値
    ///
    /// 異常検知結果。データが不足している場合は`None`を返します。
    ///
    /// # Errors
    ///
    /// 統計計算に失敗した場合はエラーを返します。
    pub fn detect_size_anomaly(
        &self,
        histories: &[BackupHistory],
        current_size: BackupSize,
    ) -> SmartResult<Option<AnomalyResult>> {
        // データ不足チェック
        if histories.len() < 3 {
            return Ok(None);
        }

        // 成功したバックアップのみを対象
        let successful_histories: Vec<_> = histories.iter().filter(|h| h.success).collect();

        if successful_histories.len() < 3 {
            return Ok(None);
        }

        // サイズデータを抽出
        let mut sizes: Vec<f64> = successful_histories
            .iter()
            .map(|h| h.total_bytes as f64)
            .collect();

        // 統計計算（Vec<f64>をmutableスライスとしてDataに渡す）
        let data = Data::new(&mut sizes[..]);
        let mean = data.mean().unwrap_or(0.0);
        let std_dev = data.std_dev().unwrap_or(0.0);

        // 標準偏差が0の場合（全て同じサイズ）
        if std_dev == 0.0 {
            if current_size.as_f64() == mean {
                return Ok(Some(AnomalyResult::new(
                    false,
                    0.0,
                    PredictionConfidence::new(1.0).map_err(SmartError::StatisticsError)?,
                    "サイズは通常範囲内です".to_string(),
                    None,
                )));
            }
            return Ok(Some(AnomalyResult::new(
                true,
                f64::INFINITY,
                PredictionConfidence::new(0.99).map_err(SmartError::StatisticsError)?,
                format!(
                    "サイズが急変しました（通常: {:.2}MB → 現在: {:.2}MB）",
                    mean / 1_048_576.0,
                    current_size.as_mb()
                ),
                Some("バックアップ対象の変更を確認してください".to_string()),
            )));
        }

        // Z-score計算
        let z_score = (current_size.as_f64() - mean).abs() / std_dev;

        // 異常判定
        let is_anomaly = z_score > self.threshold.z_score;

        // 信頼度計算（Z-scoreから信頼度を導出）
        let confidence_value = if is_anomaly {
            // 異常の場合: Z-scoreが大きいほど信頼度が高い
            (z_score / (self.threshold.z_score + 3.0)).min(0.99)
        } else {
            // 正常の場合: Z-scoreが小さいほど信頼度が高い
            (1.0_f64 - (z_score / self.threshold.z_score)).max(0.5)
        };

        let confidence =
            PredictionConfidence::new(confidence_value).map_err(SmartError::StatisticsError)?;

        // 説明文生成
        let description = if is_anomaly {
            let ratio = current_size.as_f64() / mean;
            if ratio > 1.5 {
                format!(
                    "バックアップサイズが通常の{:.1}倍に増加しています（通常: {:.2}MB → 現在: {:.2}MB）",
                    ratio,
                    mean / 1_048_576.0,
                    current_size.as_mb()
                )
            } else if ratio < 0.5 {
                format!(
                    "バックアップサイズが通常の{:.0}%に減少しています（通常: {:.2}MB → 現在: {:.2}MB）",
                    ratio * 100.0,
                    mean / 1_048_576.0,
                    current_size.as_mb()
                )
            } else {
                format!(
                    "バックアップサイズが異常値です（通常: {:.2}MB ± {:.2}MB → 現在: {:.2}MB）",
                    mean / 1_048_576.0,
                    std_dev / 1_048_576.0,
                    current_size.as_mb()
                )
            }
        } else {
            format!(
                "バックアップサイズは通常範囲内です（平均: {:.2}MB, 標準偏差: {:.2}MB）",
                mean / 1_048_576.0,
                std_dev / 1_048_576.0
            )
        };

        // 推奨アクション
        let recommended_action = if is_anomaly {
            let ratio = current_size.as_f64() / mean;
            if ratio > 2.0 {
                Some("一時ファイルや大容量ファイルが追加されていないか確認してください".to_string())
            } else if ratio < 0.3 {
                Some("重要なファイルが削除されていないか確認してください".to_string())
            } else {
                Some("バックアップ対象の変更履歴を確認してください".to_string())
            }
        } else {
            None
        };

        Ok(Some(AnomalyResult::new(
            is_anomaly,
            z_score,
            confidence,
            description,
            recommended_action,
        )))
    }

    /// 移動平均の計算
    ///
    /// # Errors
    ///
    /// データ不足の場合はエラーを返します。
    pub fn calculate_moving_average(&self, histories: &[BackupHistory]) -> SmartResult<Vec<f64>> {
        if histories.len() < self.threshold.window_size {
            return Err(SmartError::InsufficientData {
                required: self.threshold.window_size,
                actual: histories.len(),
            });
        }

        let mut moving_averages = Vec::new();
        let sizes: Vec<f64> = histories.iter().map(|h| h.total_bytes as f64).collect();

        for i in 0..=sizes.len().saturating_sub(self.threshold.window_size) {
            let window = &sizes[i..i + self.threshold.window_size];
            let avg = window.iter().sum::<f64>() / self.threshold.window_size as f64;
            moving_averages.push(avg);
        }

        Ok(moving_averages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_mock_history(size: u64) -> BackupHistory {
        BackupHistory::new(PathBuf::from("/tmp/backup"), 100, size, true, false, false)
    }

    #[test]
    fn test_anomaly_threshold_default() {
        let threshold = AnomalyThreshold::default();
        assert_eq!(threshold.z_score(), 3.0);
        assert_eq!(threshold.window_size(), 7);
    }

    #[test]
    fn test_anomaly_threshold_validation() {
        assert!(AnomalyThreshold::new(-1.0, 7).is_err());
        assert!(AnomalyThreshold::new(3.0, 0).is_err());
        assert!(AnomalyThreshold::new(3.0, 7).is_ok());
    }

    #[test]
    fn test_anomaly_detector_insufficient_data() {
        let detector = AnomalyDetector::default_detector();
        let histories = vec![create_mock_history(1000), create_mock_history(1100)];
        let current = BackupSize::new(5000);

        let result = detector.detect_size_anomaly(&histories, current).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_anomaly_detector_normal_size() {
        let detector = AnomalyDetector::default_detector();
        let histories = vec![
            create_mock_history(1000),
            create_mock_history(1100),
            create_mock_history(900),
            create_mock_history(1050),
        ];
        let current = BackupSize::new(1000);

        let result = detector
            .detect_size_anomaly(&histories, current)
            .unwrap()
            .unwrap();
        assert!(!result.is_anomaly());
    }

    #[test]
    fn test_anomaly_detector_anomaly_size() {
        let detector = AnomalyDetector::default_detector();
        let histories = vec![
            create_mock_history(1000),
            create_mock_history(1100),
            create_mock_history(900),
            create_mock_history(1050),
        ];
        // 通常の5倍のサイズ
        let current = BackupSize::new(5000);

        let result = detector
            .detect_size_anomaly(&histories, current)
            .unwrap()
            .unwrap();
        assert!(result.is_anomaly());
        assert!(result.confidence().is_high());
    }

    #[test]
    fn test_moving_average() {
        let detector = AnomalyDetector::new(AnomalyThreshold::new(3.0, 3).unwrap());
        let histories = vec![
            create_mock_history(1000),
            create_mock_history(2000),
            create_mock_history(3000),
            create_mock_history(4000),
        ];

        let averages = detector.calculate_moving_average(&histories).unwrap();
        assert_eq!(averages.len(), 2);
        assert_eq!(averages[0], 2000.0); // (1000 + 2000 + 3000) / 3
        assert_eq!(averages[1], 3000.0); // (2000 + 3000 + 4000) / 3
    }
}
