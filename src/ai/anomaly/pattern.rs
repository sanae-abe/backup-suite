//! 失敗パターン分析エンジン
//!
//! バックアップ失敗の頻発パターンを検出します。

use crate::ai::error::{AiError, AiResult};
use crate::ai::types::FailureRate;
use crate::core::history::{BackupHistory, BackupStatus};
use chrono::Timelike;
use std::collections::HashMap;

/// パターン分析器
///
/// バックアップ失敗のパターンを分析します。
///
/// # 使用例
///
/// ```rust,no_run
/// use backup_suite::ai::anomaly::PatternAnalyzer;
/// use backup_suite::BackupHistory;
///
/// let analyzer = PatternAnalyzer::new();
/// let histories = BackupHistory::load_all().unwrap();
///
/// let failure_rate = analyzer.calculate_failure_rate(&histories).unwrap();
/// println!("失敗率: {:.1}%", failure_rate.as_percentage());
///
/// let patterns = analyzer.detect_failure_patterns(&histories).unwrap();
/// for pattern in patterns {
///     println!("頻発エラー: {} ({}回)", pattern.error_message(), pattern.count());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PatternAnalyzer {
    min_occurrences: usize,
}

impl PatternAnalyzer {
    /// 新しいパターン分析器を作成
    ///
    /// # 引数
    ///
    /// * `min_occurrences` - パターンとして認識する最低発生回数
    #[must_use]
    pub const fn new() -> Self {
        Self { min_occurrences: 3 }
    }

    /// 失敗率を計算
    ///
    /// # Errors
    ///
    /// データが不足している場合はエラーを返します。
    pub fn calculate_failure_rate(&self, histories: &[BackupHistory]) -> AiResult<FailureRate> {
        if histories.is_empty() {
            return Err(AiError::InsufficientData {
                required: 1,
                actual: 0,
            });
        }

        let total = histories.len();
        let failed = histories
            .iter()
            .filter(|h| matches!(h.status, BackupStatus::Failed | BackupStatus::Partial))
            .count();

        let rate = failed as f64 / total as f64;
        FailureRate::new(rate).map_err(AiError::InvalidParameter)
    }

    /// 失敗パターンを検出
    ///
    /// # Errors
    ///
    /// データ処理に失敗した場合はエラーを返します。
    pub fn detect_failure_patterns(
        &self,
        histories: &[BackupHistory],
    ) -> AiResult<Vec<FailurePattern>> {
        // 失敗したバックアップのみを対象
        let failed_histories: Vec<_> = histories
            .iter()
            .filter(|h| matches!(h.status, BackupStatus::Failed))
            .collect();

        if failed_histories.is_empty() {
            return Ok(Vec::new());
        }

        // エラーメッセージごとにカウント
        let mut error_counts: HashMap<String, usize> = HashMap::new();

        for history in &failed_histories {
            if let Some(error_msg) = &history.error_message {
                *error_counts.entry(error_msg.clone()).or_insert(0) += 1;
            }
        }

        // 頻発するパターンを抽出
        let mut patterns: Vec<FailurePattern> = error_counts
            .into_iter()
            .filter(|(_, count)| *count >= self.min_occurrences)
            .map(|(error_message, count)| {
                let frequency = count as f64 / failed_histories.len() as f64;
                FailurePattern::new(error_message, count, frequency)
            })
            .collect();

        // 発生回数順にソート
        patterns.sort_by(|a, b| b.count.cmp(&a.count));

        Ok(patterns)
    }

    /// カテゴリ別の失敗率を計算
    ///
    /// # Errors
    ///
    /// データ処理に失敗した場合はエラーを返します。
    pub fn calculate_failure_rate_by_category(
        &self,
        histories: &[BackupHistory],
    ) -> AiResult<HashMap<String, FailureRate>> {
        if histories.is_empty() {
            return Ok(HashMap::new());
        }

        // カテゴリごとにグループ化
        let mut category_stats: HashMap<String, (usize, usize)> = HashMap::new();

        for history in histories {
            let category = history
                .category
                .clone()
                .unwrap_or_else(|| "未分類".to_string());

            let (total, failed) = category_stats.entry(category).or_insert((0, 0));
            *total += 1;
            if matches!(history.status, BackupStatus::Failed | BackupStatus::Partial) {
                *failed += 1;
            }
        }

        // 失敗率を計算
        let mut result = HashMap::new();
        for (category, (total, failed)) in category_stats {
            let rate = failed as f64 / total as f64;
            let failure_rate = FailureRate::new(rate).map_err(AiError::InvalidParameter)?;
            result.insert(category, failure_rate);
        }

        Ok(result)
    }

    /// 時間帯別の失敗率を分析
    ///
    /// # Errors
    ///
    /// データ処理に失敗した場合はエラーを返します。
    pub fn analyze_failure_by_hour(
        &self,
        histories: &[BackupHistory],
    ) -> AiResult<HashMap<u32, FailureRate>> {
        if histories.is_empty() {
            return Ok(HashMap::new());
        }

        // 時間帯ごとにグループ化（0-23時）
        let mut hour_stats: HashMap<u32, (usize, usize)> = HashMap::new();

        for history in histories {
            let hour = history.timestamp.hour();
            let (total, failed) = hour_stats.entry(hour).or_insert((0, 0));
            *total += 1;
            if matches!(history.status, BackupStatus::Failed | BackupStatus::Partial) {
                *failed += 1;
            }
        }

        // 失敗率を計算
        let mut result = HashMap::new();
        for (hour, (total, failed)) in hour_stats {
            let rate = failed as f64 / total as f64;
            let failure_rate = FailureRate::new(rate).map_err(AiError::InvalidParameter)?;
            result.insert(hour, failure_rate);
        }

        Ok(result)
    }
}

impl Default for PatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// 失敗パターン
#[derive(Debug, Clone)]
pub struct FailurePattern {
    error_message: String,
    count: usize,
    frequency: f64,
}

impl FailurePattern {
    /// 新しい失敗パターンを作成
    #[must_use]
    pub const fn new(error_message: String, count: usize, frequency: f64) -> Self {
        Self {
            error_message,
            count,
            frequency,
        }
    }

    /// エラーメッセージを取得
    #[must_use]
    pub fn error_message(&self) -> &str {
        &self.error_message
    }

    /// 発生回数を取得
    #[must_use]
    pub const fn count(&self) -> usize {
        self.count
    }

    /// 発生頻度を取得（0.0-1.0）
    #[must_use]
    pub const fn frequency(&self) -> f64 {
        self.frequency
    }

    /// 頻度をパーセンテージで取得
    #[must_use]
    pub fn frequency_percentage(&self) -> f64 {
        self.frequency * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_failed_history(error_msg: &str) -> BackupHistory {
        let mut history = BackupHistory::new(PathBuf::from("/tmp/backup"), 100, 1000, false);
        history.error_message = Some(error_msg.to_string());
        history
    }

    fn create_successful_history() -> BackupHistory {
        BackupHistory::new(PathBuf::from("/tmp/backup"), 100, 1000, true)
    }

    #[test]
    fn test_calculate_failure_rate() {
        let analyzer = PatternAnalyzer::new();
        let histories = vec![
            create_successful_history(),
            create_successful_history(),
            create_failed_history("Error 1"),
            create_successful_history(),
        ];

        let rate = analyzer.calculate_failure_rate(&histories).unwrap();
        assert_eq!(rate.as_percentage(), 25.0);
    }

    #[test]
    fn test_calculate_failure_rate_empty() {
        let analyzer = PatternAnalyzer::new();
        let histories = vec![];

        assert!(analyzer.calculate_failure_rate(&histories).is_err());
    }

    #[test]
    fn test_detect_failure_patterns() {
        let analyzer = PatternAnalyzer::new();
        let histories = vec![
            create_failed_history("Permission denied"),
            create_failed_history("Permission denied"),
            create_failed_history("Permission denied"),
            create_failed_history("Disk full"),
            create_successful_history(),
        ];

        let patterns = analyzer.detect_failure_patterns(&histories).unwrap();
        assert_eq!(patterns.len(), 1); // "Permission denied" のみが3回以上
        assert_eq!(patterns[0].error_message(), "Permission denied");
        assert_eq!(patterns[0].count(), 3);
    }

    #[test]
    fn test_detect_failure_patterns_no_failures() {
        let analyzer = PatternAnalyzer::new();
        let histories = vec![create_successful_history(), create_successful_history()];

        let patterns = analyzer.detect_failure_patterns(&histories).unwrap();
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_calculate_failure_rate_by_category() {
        let analyzer = PatternAnalyzer::new();

        let mut h1 = create_successful_history();
        h1.category = Some("documents".to_string());

        let mut h2 = create_failed_history("Error");
        h2.category = Some("documents".to_string());

        let mut h3 = create_successful_history();
        h3.category = Some("photos".to_string());

        let histories = vec![h1, h2, h3];

        let rates = analyzer
            .calculate_failure_rate_by_category(&histories)
            .unwrap();
        assert_eq!(rates.get("documents").unwrap().as_percentage(), 50.0);
        assert_eq!(rates.get("photos").unwrap().as_percentage(), 0.0);
    }

    #[test]
    fn test_analyze_failure_by_hour() {
        let analyzer = PatternAnalyzer::new();

        let mut h1 = create_successful_history();
        h1.timestamp = Utc::now().with_hour(10).unwrap();

        let mut h2 = create_failed_history("Error");
        h2.timestamp = Utc::now().with_hour(10).unwrap();

        let mut h3 = create_successful_history();
        h3.timestamp = Utc::now().with_hour(15).unwrap();

        let histories = vec![h1, h2, h3];

        let rates = analyzer.analyze_failure_by_hour(&histories).unwrap();
        assert_eq!(rates.get(&10).unwrap().as_percentage(), 50.0);
        assert_eq!(rates.get(&15).unwrap().as_percentage(), 0.0);
    }
}
