//! AI機能の共通型定義
//!
//! newtype patternを使用した型安全なデータ表現を提供します。

use serde::{Deserialize, Serialize};

/// バックアップサイズ（バイト単位）
///
/// # 使用例
///
/// ```rust
/// use backup_suite::smart::BackupSize;
///
/// let size = BackupSize::new(1_048_576); // 1 MiB
/// assert_eq!(size.get(), 1_048_576);
/// assert!((size.as_mb() - 1.0).abs() < 0.01); // 約1.0 MB
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BackupSize(u64);

impl BackupSize {
    /// 新しいBackupSizeインスタンスを作成
    #[must_use]
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// バイト数を取得
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// f64型として取得（統計計算用）
    #[must_use]
    pub const fn as_f64(self) -> f64 {
        self.0 as f64
    }

    /// MB単位で取得
    #[must_use]
    pub fn as_mb(self) -> f64 {
        self.0 as f64 / 1_048_576.0
    }

    /// GB単位で取得
    #[must_use]
    pub fn as_gb(self) -> f64 {
        self.0 as f64 / 1_073_741_824.0
    }
}

impl From<u64> for BackupSize {
    fn from(bytes: u64) -> Self {
        Self(bytes)
    }
}

impl From<BackupSize> for u64 {
    fn from(size: BackupSize) -> Self {
        size.0
    }
}

impl std::fmt::Display for BackupSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 >= 1_073_741_824 {
            write!(f, "{:.2} GB", self.as_gb())
        } else if self.0 >= 1_048_576 {
            write!(f, "{:.2} MB", self.as_mb())
        } else if self.0 >= 1024 {
            write!(f, "{:.2} KB", self.0 as f64 / 1024.0)
        } else {
            write!(f, "{} B", self.0)
        }
    }
}

/// 予測信頼度（0.0 - 1.0）
///
/// # 使用例
///
/// ```rust
/// use backup_suite::smart::PredictionConfidence;
///
/// let confidence = PredictionConfidence::new(0.95).unwrap();
/// assert_eq!(confidence.get(), 0.95);
/// assert_eq!(confidence.as_percentage(), 95.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PredictionConfidence(f64);

impl PredictionConfidence {
    /// 新しいPredictionConfidenceインスタンスを作成
    ///
    /// # Errors
    ///
    /// 値が0.0未満または1.0を超える場合はエラーを返します
    pub fn new(value: f64) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&value) {
            return Err(format!(
                "信頼度は0.0から1.0の範囲である必要があります: {}",
                value
            ));
        }
        Ok(Self(value))
    }

    /// 信頼度を取得
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }

    /// パーセンテージとして取得
    #[must_use]
    pub fn as_percentage(self) -> f64 {
        self.0 * 100.0
    }

    /// 高信頼度かどうか（80%以上）
    #[must_use]
    pub fn is_high(self) -> bool {
        self.0 >= 0.8
    }

    /// 中信頼度かどうか（50%-80%）
    #[must_use]
    pub fn is_medium(self) -> bool {
        (0.5..0.8).contains(&self.0)
    }

    /// 低信頼度かどうか（50%未満）
    #[must_use]
    pub fn is_low(self) -> bool {
        self.0 < 0.5
    }
}

impl std::fmt::Display for PredictionConfidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}%", self.as_percentage())
    }
}

/// ファイル重要度（0 - 100）
///
/// # 使用例
///
/// ```rust
/// use backup_suite::smart::FileImportance;
///
/// let importance = FileImportance::new(85).unwrap();
/// assert_eq!(importance.get(), 85);
/// assert!(importance.is_high());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FileImportance(u8);

impl FileImportance {
    /// 新しいFileImportanceインスタンスを作成
    ///
    /// # Errors
    ///
    /// 値が100を超える場合はエラーを返します
    pub fn new(value: u8) -> Result<Self, String> {
        if value > 100 {
            return Err(format!(
                "重要度は0から100の範囲である必要があります: {}",
                value
            ));
        }
        Ok(Self(value))
    }

    /// 重要度を取得
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// 高重要度かどうか（80以上）
    #[must_use]
    pub const fn is_high(self) -> bool {
        self.0 >= 80
    }

    /// 中重要度かどうか（40-79）
    #[must_use]
    pub const fn is_medium(self) -> bool {
        self.0 >= 40 && self.0 < 80
    }

    /// 低重要度かどうか（40未満）
    #[must_use]
    pub const fn is_low(self) -> bool {
        self.0 < 40
    }
}

impl std::fmt::Display for FileImportance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = if self.is_high() {
            "高"
        } else if self.is_medium() {
            "中"
        } else {
            "低"
        };
        write!(f, "{}/100 ({})", self.0, label)
    }
}

/// ディスク容量（バイト単位）
///
/// # 使用例
///
/// ```rust
/// use backup_suite::smart::DiskCapacity;
///
/// let capacity = DiskCapacity::new(536_870_912_000); // 500 GiB
/// assert!((capacity.as_gb() - 500.0).abs() < 0.1); // 約500 GB
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DiskCapacity(u64);

impl DiskCapacity {
    /// 新しいDiskCapacityインスタンスを作成
    #[must_use]
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// バイト数を取得
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// GB単位で取得
    #[must_use]
    pub fn as_gb(self) -> f64 {
        self.0 as f64 / 1_073_741_824.0
    }

    /// TB単位で取得
    #[must_use]
    pub fn as_tb(self) -> f64 {
        self.0 as f64 / 1_099_511_627_776.0
    }

    /// 使用率を計算（0.0-1.0）
    #[must_use]
    pub fn usage_ratio(self, used: DiskCapacity) -> f64 {
        if self.0 == 0 {
            return 0.0;
        }
        used.0 as f64 / self.0 as f64
    }
}

impl From<u64> for DiskCapacity {
    fn from(bytes: u64) -> Self {
        Self(bytes)
    }
}

impl From<DiskCapacity> for u64 {
    fn from(capacity: DiskCapacity) -> Self {
        capacity.0
    }
}

impl std::fmt::Display for DiskCapacity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 >= 1_099_511_627_776 {
            write!(f, "{:.2} TB", self.as_tb())
        } else if self.0 >= 1_073_741_824 {
            write!(f, "{:.2} GB", self.as_gb())
        } else if self.0 >= 1_048_576 {
            write!(f, "{:.2} MB", self.0 as f64 / 1_048_576.0)
        } else {
            write!(f, "{} bytes", self.0)
        }
    }
}

/// 失敗率（0.0 - 1.0）
///
/// # 使用例
///
/// ```rust
/// use backup_suite::smart::FailureRate;
///
/// let rate = FailureRate::new(0.05).unwrap();
/// assert_eq!(rate.get(), 0.05);
/// assert_eq!(rate.as_percentage(), 5.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct FailureRate(f64);

impl FailureRate {
    /// 新しいFailureRateインスタンスを作成
    ///
    /// # Errors
    ///
    /// 値が0.0未満または1.0を超える場合はエラーを返します
    pub fn new(value: f64) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&value) {
            return Err(format!(
                "失敗率は0.0から1.0の範囲である必要があります: {}",
                value
            ));
        }
        Ok(Self(value))
    }

    /// 失敗率を取得
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }

    /// パーセンテージとして取得
    #[must_use]
    pub fn as_percentage(self) -> f64 {
        self.0 * 100.0
    }

    /// 高リスクかどうか（20%以上）
    #[must_use]
    pub fn is_high_risk(self) -> bool {
        self.0 >= 0.2
    }

    /// 中リスクかどうか（5%-20%）
    #[must_use]
    pub fn is_medium_risk(self) -> bool {
        (0.05..0.2).contains(&self.0)
    }

    /// 低リスクかどうか（5%未満）
    #[must_use]
    pub fn is_low_risk(self) -> bool {
        self.0 < 0.05
    }
}

impl std::fmt::Display for FailureRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}%", self.as_percentage())
    }
}

/// 時系列データポイント
///
/// # 使用例
///
/// ```rust
/// use backup_suite::smart::TimeSeriesPoint;
/// use chrono::Utc;
///
/// let point = TimeSeriesPoint::new(Utc::now(), 1024.0);
/// assert_eq!(point.value(), 1024.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    timestamp: chrono::DateTime<chrono::Utc>,
    value: f64,
}

impl TimeSeriesPoint {
    /// 新しいTimeSeriesPointインスタンスを作成
    #[must_use]
    pub const fn new(timestamp: chrono::DateTime<chrono::Utc>, value: f64) -> Self {
        Self { timestamp, value }
    }

    /// タイムスタンプを取得
    #[must_use]
    pub const fn timestamp(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.timestamp
    }

    /// 値を取得
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.value
    }
}

impl std::fmt::Display for TimeSeriesPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {:.2}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.value
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_size() {
        let size = BackupSize::new(1_048_576);
        assert_eq!(size.get(), 1_048_576);
        assert_eq!(size.as_mb(), 1.0);
        assert_eq!(size.as_gb(), 1.0 / 1024.0);
    }

    #[test]
    fn test_prediction_confidence_valid() {
        let conf = PredictionConfidence::new(0.95).unwrap();
        assert_eq!(conf.get(), 0.95);
        assert_eq!(conf.as_percentage(), 95.0);
        assert!(conf.is_high());
    }

    #[test]
    fn test_prediction_confidence_invalid() {
        assert!(PredictionConfidence::new(-0.1).is_err());
        assert!(PredictionConfidence::new(1.5).is_err());
    }

    #[test]
    fn test_file_importance() {
        let high = FileImportance::new(85).unwrap();
        assert!(high.is_high());
        assert!(!high.is_medium());

        let medium = FileImportance::new(50).unwrap();
        assert!(medium.is_medium());

        let low = FileImportance::new(20).unwrap();
        assert!(low.is_low());
    }

    #[test]
    fn test_file_importance_invalid() {
        assert!(FileImportance::new(101).is_err());
    }

    #[test]
    fn test_disk_capacity() {
        let capacity = DiskCapacity::new(1_073_741_824); // 1GB
        assert_eq!(capacity.as_gb(), 1.0);

        let used = DiskCapacity::new(536_870_912); // 0.5GB
        assert_eq!(capacity.usage_ratio(used), 0.5);
    }

    #[test]
    fn test_failure_rate() {
        let rate = FailureRate::new(0.05).unwrap();
        assert_eq!(rate.get(), 0.05);
        assert_eq!(rate.as_percentage(), 5.0);
        assert!(rate.is_medium_risk());
    }

    #[test]
    fn test_failure_rate_invalid() {
        assert!(FailureRate::new(-0.1).is_err());
        assert!(FailureRate::new(1.5).is_err());
    }

    #[test]
    fn test_time_series_point() {
        use chrono::Utc;
        let now = Utc::now();
        let point = TimeSeriesPoint::new(now, 1024.0);
        assert_eq!(point.timestamp(), &now);
        assert_eq!(point.value(), 1024.0);
    }

    #[test]
    fn test_backup_size_display() {
        assert_eq!(BackupSize::new(500).to_string(), "500 B");
        assert_eq!(BackupSize::new(2048).to_string(), "2.00 KB");
        assert_eq!(BackupSize::new(1_048_576).to_string(), "1.00 MB");
        assert_eq!(BackupSize::new(1_073_741_824).to_string(), "1.00 GB");
    }

    #[test]
    fn test_prediction_confidence_display() {
        let conf = PredictionConfidence::new(0.85).unwrap();
        assert_eq!(conf.to_string(), "85.0%");
    }

    #[test]
    fn test_file_importance_display() {
        let high = FileImportance::new(90).unwrap();
        assert_eq!(high.to_string(), "90/100 (高)");

        let medium = FileImportance::new(60).unwrap();
        assert_eq!(medium.to_string(), "60/100 (中)");

        let low = FileImportance::new(20).unwrap();
        assert_eq!(low.to_string(), "20/100 (低)");
    }

    #[test]
    fn test_disk_capacity_display() {
        assert_eq!(DiskCapacity::new(500).to_string(), "500 bytes");
        assert_eq!(DiskCapacity::new(1_048_576).to_string(), "1.00 MB");
        assert_eq!(DiskCapacity::new(1_073_741_824).to_string(), "1.00 GB");
        assert_eq!(DiskCapacity::new(1_099_511_627_776).to_string(), "1.00 TB");
    }

    #[test]
    fn test_failure_rate_display() {
        let rate = FailureRate::new(0.15).unwrap();
        assert_eq!(rate.to_string(), "15.00%");
    }

    #[test]
    fn test_time_series_point_display() {
        use chrono::Utc;
        let point = TimeSeriesPoint::new(Utc::now(), 123.456);
        let display = point.to_string();
        assert!(display.contains("123.46"));
    }
}
