use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::Target;
use crate::error::{BackupError, Result as BackupResult};
use crate::security::{check_read_permission, check_write_permission};

/// スケジュール設定
///
/// バックアップの自動実行スケジュールを定義します。
/// 優先度別に異なる頻度でバックアップを実行できます。
///
/// # フィールド
///
/// * `enabled` - スケジュール機能の有効/無効
/// * `high_frequency` - 高優先度のバックアップ頻度（"daily", "weekly", "monthly"）
/// * `medium_frequency` - 中優先度のバックアップ頻度
/// * `low_frequency` - 低優先度のバックアップ頻度
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::core::config::ScheduleConfig;
///
/// let schedule = ScheduleConfig {
///     enabled: true,
///     high_frequency: "daily".to_string(),
///     medium_frequency: "weekly".to_string(),
///     low_frequency: "monthly".to_string(),
/// };
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub enabled: bool,
    pub high_frequency: String, // "daily", "weekly", "monthly"
    pub medium_frequency: String,
    pub low_frequency: String,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            high_frequency: "daily".to_string(),
            medium_frequency: "weekly".to_string(),
            low_frequency: "monthly".to_string(),
        }
    }
}

/// バックアップ設定
///
/// バックアップ先ディレクトリと保存期間を定義します。
///
/// # フィールド
///
/// * `destination` - バックアップファイルの保存先ディレクトリ
/// * `auto_cleanup` - 古いバックアップの自動削除を有効にするか
/// * `keep_days` - バックアップを保持する日数（1-3650日）
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::core::config::BackupConfig;
/// use std::path::PathBuf;
///
/// let config = BackupConfig {
///     destination: PathBuf::from("/backup/storage"),
///     auto_cleanup: true,
///     keep_days: 30,
/// };
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupConfig {
    pub destination: PathBuf,
    pub auto_cleanup: bool,
    pub keep_days: u32,
}

impl Default for BackupConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        Self {
            destination: home.join("backup-suite/backups"),
            auto_cleanup: false,
            keep_days: 30,
        }
    }
}

/// メイン設定構造体
///
/// `backup-suite` の全体設定を管理します。
/// TOML形式で永続化され、`~/.config/backup-suite/config.toml` に保存されます。
///
/// # フィールド
///
/// * `version` - 設定ファイルのバージョン
/// * `backup` - バックアップ関連の設定
/// * `schedule` - スケジュール関連の設定
/// * `targets` - バックアップ対象のリスト
///
/// # 使用例
///
/// ```no_run
/// use backup_suite::{Config, Target, Priority};
/// use std::path::PathBuf;
///
/// // デフォルト設定を作成
/// let mut config = Config::default();
///
/// // バックアップ対象を追加
/// let target = Target::new(
///     PathBuf::from("/home/user/documents"),
///     Priority::High,
///     "重要ドキュメント".to_string()
/// );
/// config.add_target(target);
///
/// // 設定を保存
/// config.save().unwrap();
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub backup: BackupConfig,
    #[serde(default)]
    pub schedule: ScheduleConfig,
    pub targets: Vec<Target>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            backup: BackupConfig::default(),
            schedule: ScheduleConfig::default(),
            targets: vec![],
        }
    }
}

impl Config {
    /// 設定ファイルのパスを取得
    ///
    /// 設定ファイルは `~/.config/backup-suite/config.toml` に配置されます。
    ///
    /// # 戻り値
    ///
    /// 成功時は設定ファイルのパス、失敗時はエラー
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// * ホームディレクトリが取得できない場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::Config;
    ///
    /// let path = Config::config_path().unwrap();
    /// println!("設定ファイル: {:?}", path);
    /// ```
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("ホームディレクトリが見つかりません")?;
        Ok(home.join(".config/backup-suite/config.toml"))
    }

    /// 設定ファイルを読み込み
    ///
    /// `~/.config/backup-suite/config.toml` から設定を読み込みます。
    /// ファイルが存在しない場合はデフォルト設定を返します。
    ///
    /// # 戻り値
    ///
    /// 成功時は `Config` インスタンス、失敗時はエラー
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// * 設定ファイルパスの取得に失敗した場合
    /// * 設定ファイルの読み込みに失敗した場合
    /// * TOML解析に失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::Config;
    ///
    /// let config = Config::load().unwrap_or_default();
    /// println!("バックアップ先: {:?}", config.backup.destination);
    /// ```
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            // 設定ファイルが存在しない場合はデフォルト設定を返す
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)
            .context(format!("設定ファイル読み込み失敗: {:?}", config_path))?;

        let config: Config = toml::from_str(&content).context("TOML解析失敗")?;

        Ok(config)
    }

    /// 設定ファイルに保存
    ///
    /// 現在の設定を `~/.config/backup-suite/config.toml` に保存します。
    /// 設定ディレクトリが存在しない場合は自動的に作成されます。
    ///
    /// # 戻り値
    ///
    /// 成功時は `Ok(())`、失敗時はエラー
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// * 設定ファイルパスの取得に失敗した場合
    /// * 設定ディレクトリの作成に失敗した場合
    /// * TOML生成に失敗した場合
    /// * ファイル書き込みに失敗した場合
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::{Config, Target, Priority};
    /// use std::path::PathBuf;
    ///
    /// let mut config = Config::default();
    /// let target = Target::new(
    ///     PathBuf::from("/path/to/backup"),
    ///     Priority::High,
    ///     "重要データ".to_string()
    /// );
    /// config.add_target(target);
    /// config.save().unwrap();
    /// ```
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // ディレクトリが存在しない場合は作成
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).context("設定ディレクトリ作成失敗")?;
        }

        let content = toml::to_string_pretty(self).context("TOML生成失敗")?;

        std::fs::write(&config_path, content)
            .context(format!("設定ファイル書き込み失敗: {:?}", config_path))?;

        Ok(())
    }

    /// バックアップ対象を追加
    ///
    /// 新しいバックアップ対象を設定に追加します。
    ///
    /// # 引数
    ///
    /// * `target` - 追加するバックアップ対象
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::{Config, Target, Priority};
    /// use std::path::PathBuf;
    ///
    /// let mut config = Config::default();
    /// let target = Target::new(
    ///     PathBuf::from("/home/user/documents"),
    ///     Priority::High,
    ///     "ドキュメント".to_string()
    /// );
    /// config.add_target(target);
    /// ```
    pub fn add_target(&mut self, target: Target) {
        // 重複チェック：同じパスがすでに存在する場合は追加しない
        if self.targets.iter().any(|t| t.path == target.path) {
            eprintln!(
                "警告: {:?} は既に登録されています。スキップします。",
                target.path
            );
            return;
        }
        self.targets.push(target);
    }

    /// バックアップ対象を削除
    ///
    /// 指定されたパスのバックアップ対象を設定から削除します。
    ///
    /// # 引数
    ///
    /// * `path` - 削除するバックアップ対象のパス
    ///
    /// # 戻り値
    ///
    /// 削除された場合は `true`、見つからなかった場合は `false`
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::Config;
    /// use std::path::PathBuf;
    ///
    /// let mut config = Config::load().unwrap();
    /// let removed = config.remove_target(&PathBuf::from("/old/path"));
    /// if removed {
    ///     config.save().unwrap();
    /// }
    /// ```
    pub fn remove_target(&mut self, path: &PathBuf) -> bool {
        let before_len = self.targets.len();
        self.targets.retain(|t| &t.path != path);
        self.targets.len() < before_len
    }

    /// 優先度でフィルタリング
    ///
    /// 指定された優先度のバックアップ対象のみを抽出します。
    ///
    /// # 引数
    ///
    /// * `priority` - フィルタリングする優先度
    ///
    /// # 戻り値
    ///
    /// 指定された優先度のバックアップ対象の参照のベクター
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::{Config, Priority};
    ///
    /// let config = Config::load().unwrap();
    /// let high_priority = config.filter_by_priority(&Priority::High);
    /// println!("高優先度のバックアップ対象: {}件", high_priority.len());
    /// ```
    pub fn filter_by_priority(&self, priority: &super::target::Priority) -> Vec<&Target> {
        self.targets
            .iter()
            .filter(|t| &t.priority >= priority)
            .collect()
    }

    /// カテゴリでバックアップ対象をフィルタ
    ///
    /// 指定されたカテゴリのバックアップ対象のみを取得します。
    ///
    /// # 引数
    ///
    /// * `category` - フィルタリングするカテゴリ名
    ///
    /// # 戻り値
    ///
    /// 指定されたカテゴリのバックアップ対象の参照のベクター
    ///
    /// # 使用例
    ///
    /// ```no_run
    /// use backup_suite::Config;
    ///
    /// let config = Config::load().unwrap();
    /// let system_targets = config.filter_by_category("system");
    /// println!("システムカテゴリのバックアップ対象: {}件", system_targets.len());
    /// ```
    pub fn filter_by_category(&self, category: &str) -> Vec<&Target> {
        self.targets
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    /// 設定の妥当性を検証
    ///
    /// すべての設定項目が正しく、実行可能であることを確認します。
    ///
    /// # 検証項目
    ///
    /// - バックアップ先ディレクトリの存在と書き込み権限
    /// - 保存期間（keep_days）の妥当性（1-3650日）
    /// - 各ターゲットの存在確認と読み取り権限
    /// - 除外パターンの正規表現の妥当性
    ///
    /// # 戻り値
    ///
    /// すべての検証に成功した場合は `Ok(())`、失敗した場合はエラー
    ///
    /// # Errors
    ///
    /// 以下の場合にエラーを返します:
    /// * `BackupError::BackupDirectoryCreationError` - バックアップ先ディレクトリの作成に失敗
    /// * `BackupError::PermissionDenied` - バックアップ先に書き込み権限がない
    /// * `BackupError::ConfigValidationError` - 保存期間（keep_days）が範囲外（1-3650日）
    /// * `BackupError::PermissionDenied` - ターゲットに読み取り権限がない
    /// * `BackupError::RegexError` - 不正な正規表現パターンが含まれている
    pub fn validate(&self) -> BackupResult<()> {
        // 1. バックアップ先の妥当性チェック
        if !self.backup.destination.exists() {
            std::fs::create_dir_all(&self.backup.destination).map_err(|_| {
                BackupError::BackupDirectoryCreationError {
                    path: self.backup.destination.clone(),
                }
            })?;
        }

        // 2. バックアップ先の書き込み権限チェック
        check_write_permission(&self.backup.destination)?;

        // 3. 保存期間の妥当性チェック
        if self.backup.keep_days == 0 || self.backup.keep_days > 3650 {
            return Err(BackupError::ConfigValidationError {
                message: format!(
                    "keep_days は 1-3650 の範囲で指定してください（現在: {}）",
                    self.backup.keep_days
                ),
            });
        }

        // 4. 各ターゲットの検証
        for target in &self.targets {
            // 4.1 ターゲットの存在確認
            if !target.path.exists() {
                eprintln!("警告: バックアップ対象が存在しません: {:?}", target.path);
                // 警告のみで処理は継続（後で追加される可能性があるため）
            } else {
                // 4.2 読み取り権限チェック
                check_read_permission(&target.path)?;
            }

            // 4.3 除外パターンの正規表現検証
            for pattern in &target.exclude_patterns {
                regex::Regex::new(pattern).map_err(|e| BackupError::RegexError {
                    pattern: pattern.clone(),
                    source: e,
                })?;
            }
        }

        // 5. ターゲットが1つもない場合は警告
        if self.targets.is_empty() {
            eprintln!("警告: バックアップ対象が設定されていません");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::target::{Priority, TargetType};

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.targets.len(), 0);
        assert_eq!(config.backup.keep_days, 30);
    }

    #[test]
    fn test_add_target() {
        let mut config = Config::default();
        let target = Target {
            path: PathBuf::from("/tmp/test.txt"),
            priority: Priority::High,
            target_type: TargetType::File,
            category: "test".to_string(),
            added_date: chrono::Utc::now(),
            exclude_patterns: vec![],
        };

        config.add_target(target);
        assert_eq!(config.targets.len(), 1);
    }

    #[test]
    fn test_remove_target() {
        let mut config = Config::default();
        let path = PathBuf::from("/tmp/test.txt");
        let target = Target {
            path: path.clone(),
            priority: Priority::High,
            target_type: TargetType::File,
            category: "test".to_string(),
            added_date: chrono::Utc::now(),
            exclude_patterns: vec![],
        };

        config.add_target(target);
        assert_eq!(config.targets.len(), 1);

        let removed = config.remove_target(&path);
        assert!(removed);
        assert_eq!(config.targets.len(), 0);
    }
}
