//! # スケジューラモジュール
//!
//! 自動バックアップのスケジューリング機能を提供します。
//! macOS (launchd) と Linux (systemd) の両方をサポートします。
//!
//! # 機能
//!
//! - **macOS launchd統合**: plist設定ファイル生成と管理
//! - **Linux systemd統合**: service/timer ユニット管理
//! - **優先度別スケジュール**: High/Medium/Low で異なる頻度設定
//! - **設定の検証**: スケジュール設定の妥当性チェック
//!
//! # 使用例
//!
//! ```rust,no_run
//! use backup_suite::core::scheduler::Scheduler;
//! use backup_suite::Config;
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut config = Config::load()?;
//! config.schedule.enabled = true;
//! config.schedule.high_frequency = "daily".to_string();
//! config.save()?;
//!
//! let scheduler = Scheduler::new(config)?;
//! scheduler.setup_all()?;
//! scheduler.enable_all()?;
//!
//! println!("自動バックアップスケジュールを設定しました");
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use std::path::PathBuf;

use super::config::Config;
use super::target::Priority;

/// スケジューラ
///
/// macOS launchd または Linux systemd を使用して自動バックアップを管理します。
pub struct Scheduler {
    config: Config,
    platform: Platform,
}

/// サポートされているプラットフォーム
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// macOS (launchd使用)
    MacOS,
    /// Linux (systemd使用)
    Linux,
    /// サポート外のプラットフォーム
    Unsupported,
}

impl Platform {
    /// 現在のプラットフォームを検出
    #[must_use]
    pub fn detect() -> Self {
        #[cfg(target_os = "macos")]
        {
            Platform::MacOS
        }
        #[cfg(target_os = "linux")]
        {
            Platform::Linux
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Platform::Unsupported
        }
    }

    /// プラットフォームがサポートされているか
    #[must_use]
    pub fn is_supported(&self) -> bool {
        !matches!(self, Platform::Unsupported)
    }
}

/// スケジュール頻度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frequency {
    /// 毎日実行（午前2時）
    Daily,
    /// 毎週実行（日曜午前2時）
    Weekly,
    /// 毎月実行（1日午前2時）
    Monthly,
    /// 毎時実行
    Hourly,
}

impl Frequency {
    /// 文字列から頻度を解析
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "daily" => Ok(Frequency::Daily),
            "weekly" => Ok(Frequency::Weekly),
            "monthly" => Ok(Frequency::Monthly),
            "hourly" => Ok(Frequency::Hourly),
            _ => Err(anyhow::anyhow!("不明な頻度: {s}")),
        }
    }

    /// 頻度を文字列に変換
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Frequency::Daily => "daily",
            Frequency::Weekly => "weekly",
            Frequency::Monthly => "monthly",
            Frequency::Hourly => "hourly",
        }
    }
}

impl Scheduler {
    /// 新しいSchedulerインスタンスを作成
    ///
    /// # 引数
    ///
    /// * `config` - バックアップ設定
    ///
    /// # 戻り値
    ///
    /// 成功時は Scheduler インスタンス、失敗時はエラー
    ///
    /// # エラー
    ///
    /// * プラットフォームがサポートされていない場合
    pub fn new(config: Config) -> Result<Self> {
        let platform = Platform::detect();
        if !platform.is_supported() {
            return Err(anyhow::anyhow!(
                "このプラットフォームではスケジューリング機能はサポートされていません"
            ));
        }

        Ok(Self { config, platform })
    }

    /// 全優先度のスケジュールをセットアップ
    ///
    /// High/Medium/Low の3つの優先度のスケジュールを一度に設定します。
    ///
    /// # 戻り値
    ///
    /// 成功時は `Ok(())`、失敗時はエラー
    ///
    /// # エラー
    ///
    /// * launchd/systemd設定ファイルの作成に失敗した場合
    /// * 設定の妥当性検証に失敗した場合
    pub fn setup_all(&self) -> Result<()> {
        let priorities = [Priority::High, Priority::Medium, Priority::Low];

        for priority in &priorities {
            if let Err(e) = self.setup_priority(priority) {
                eprintln!("警告: {priority:?} 優先度のスケジュール設定失敗: {e}");
            }
        }

        Ok(())
    }

    /// 特定優先度のスケジュールをセットアップ
    ///
    /// # 引数
    ///
    /// * `priority` - 設定する優先度
    ///
    /// # 戻り値
    ///
    /// 成功時は `Ok(())`、失敗時はエラー
    pub fn setup_priority(&self, priority: &Priority) -> Result<()> {
        let frequency = self.get_frequency(priority)?;

        match self.platform {
            Platform::MacOS => self.setup_launchd(priority, frequency),
            Platform::Linux => self.setup_systemd(priority, frequency),
            Platform::Unsupported => Err(anyhow::anyhow!("サポートされていないプラットフォーム")),
        }
    }

    /// 全優先度のスケジュールを有効化
    ///
    /// # 戻り値
    ///
    /// 成功時は `Ok(())`、失敗時はエラー
    pub fn enable_all(&self) -> Result<()> {
        let priorities = [Priority::High, Priority::Medium, Priority::Low];

        for priority in &priorities {
            if let Err(e) = self.enable_priority(priority) {
                eprintln!("警告: {priority:?} 優先度の有効化失敗: {e}");
            }
        }

        Ok(())
    }

    /// 特定優先度のスケジュールを有効化
    ///
    /// # 引数
    ///
    /// * `priority` - 有効化する優先度
    ///
    /// # 戻り値
    ///
    /// 成功時は `Ok(())`、失敗時はエラー
    pub fn enable_priority(&self, priority: &Priority) -> Result<()> {
        match self.platform {
            Platform::MacOS => self.enable_launchd(priority),
            Platform::Linux => self.enable_systemd(priority),
            Platform::Unsupported => Err(anyhow::anyhow!("サポートされていないプラットフォーム")),
        }
    }

    /// 全優先度のスケジュールを無効化
    ///
    /// # 戻り値
    ///
    /// 成功時は `Ok(())`、失敗時はエラー
    pub fn disable_all(&self) -> Result<()> {
        let priorities = [Priority::High, Priority::Medium, Priority::Low];

        for priority in &priorities {
            if let Err(e) = self.disable_priority(priority) {
                eprintln!("警告: {priority:?} 優先度の無効化失敗: {e}");
            }
        }

        Ok(())
    }

    /// 特定優先度のスケジュールを無効化
    ///
    /// # 引数
    ///
    /// * `priority` - 無効化する優先度
    ///
    /// # 戻り値
    ///
    /// 成功時は `Ok(())`、失敗時はエラー
    pub fn disable_priority(&self, priority: &Priority) -> Result<()> {
        match self.platform {
            Platform::MacOS => self.disable_launchd(priority),
            Platform::Linux => self.disable_systemd(priority),
            Platform::Unsupported => Err(anyhow::anyhow!("サポートされていないプラットフォーム")),
        }
    }

    /// スケジュールの状態を確認
    ///
    /// # 戻り値
    ///
    /// 各優先度の有効/無効状態を返す
    pub fn check_status(&self) -> Result<ScheduleStatus> {
        let mut status = ScheduleStatus::default();

        for priority in &[Priority::High, Priority::Medium, Priority::Low] {
            let enabled = match self.platform {
                Platform::MacOS => self.is_launchd_enabled(priority)?,
                Platform::Linux => self.is_systemd_enabled(priority)?,
                Platform::Unsupported => false,
            };

            match priority {
                Priority::High => status.high_enabled = enabled,
                Priority::Medium => status.medium_enabled = enabled,
                Priority::Low => status.low_enabled = enabled,
            }
        }

        Ok(status)
    }

    /// 優先度に対応する頻度を取得
    fn get_frequency(&self, priority: &Priority) -> Result<Frequency> {
        let freq_str = match priority {
            Priority::High => &self.config.schedule.high_frequency,
            Priority::Medium => &self.config.schedule.medium_frequency,
            Priority::Low => &self.config.schedule.low_frequency,
        };

        Frequency::parse(freq_str)
    }

    /// 優先度を文字列に変換
    fn priority_to_string(priority: &Priority) -> &'static str {
        match priority {
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
        }
    }

    // ========== macOS launchd 実装 ==========

    /// launchd plist ファイルのパスを取得
    fn get_launchd_plist_path(&self, priority: &Priority) -> Result<PathBuf> {
        let home = dirs::home_dir().context("ホームディレクトリが見つかりません")?;
        let filename = format!(
            "com.backup-suite.{}.plist",
            Self::priority_to_string(priority)
        );
        Ok(home.join("Library/LaunchAgents").join(filename))
    }

    /// launchd スケジュールをセットアップ
    fn setup_launchd(&self, priority: &Priority, frequency: Frequency) -> Result<()> {
        let plist_path = self.get_launchd_plist_path(priority)?;

        // ディレクトリを作成
        if let Some(parent) = plist_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // plist コンテンツを生成
        let plist_content = self.generate_plist_content(priority, frequency)?;

        // ファイルに書き込み
        std::fs::write(&plist_path, plist_content)
            .context("plistファイル書き込み失敗: plist_path.display()".to_string())?;

        Ok(())
    }

    /// launchd plist コンテンツを生成
    fn generate_plist_content(&self, priority: &Priority, frequency: Frequency) -> Result<String> {
        let backup_suite_path = std::env::current_exe()?;
        let priority_str = Self::priority_to_string(priority);

        let calendar_interval = match frequency {
            Frequency::Daily => {
                r"<dict>
        <key>Hour</key>
        <integer>2</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>"
            }
            Frequency::Weekly => {
                r"<dict>
        <key>Hour</key>
        <integer>2</integer>
        <key>Minute</key>
        <integer>0</integer>
        <key>Weekday</key>
        <integer>0</integer>
    </dict>"
            }
            Frequency::Monthly => {
                r"<dict>
        <key>Hour</key>
        <integer>2</integer>
        <key>Minute</key>
        <integer>0</integer>
        <key>Day</key>
        <integer>1</integer>
    </dict>"
            }
            Frequency::Hourly => {
                r"<dict>
        <key>Minute</key>
        <integer>0</integer>
    </dict>"
            }
        };

        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.backup-suite.{priority}</string>

    <key>ProgramArguments</key>
    <array>
        <string>{backup_suite_path}</string>
        <string>run</string>
        <string>--priority</string>
        <string>{priority}</string>
    </array>

    <key>StartCalendarInterval</key>
    {calendar_interval}

    <key>RunAtLoad</key>
    <false/>

    <key>StandardOutPath</key>
    <string>/tmp/backup-suite-{priority}.log</string>

    <key>StandardErrorPath</key>
    <string>/tmp/backup-suite-{priority}.error.log</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
    </dict>
</dict>
</plist>"#,
            priority = priority_str,
            backup_suite_path = backup_suite_path.display(),
            calendar_interval = calendar_interval
        );

        Ok(plist)
    }

    /// launchd スケジュールを有効化
    fn enable_launchd(&self, priority: &Priority) -> Result<()> {
        let plist_path = self.get_launchd_plist_path(priority)?;

        if !plist_path.exists() {
            return Err(anyhow::anyhow!(
                "plistファイルが存在しません。先にsetupを実行してください: plist_path.display()"
                    .to_string()
            ));
        }

        let output = std::process::Command::new("launchctl")
            .args(["load", &plist_path.to_string_lossy()])
            .output()
            .context("launchctl load コマンド実行失敗")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("launchctl load 失敗: {stderr}"));
        }

        Ok(())
    }

    /// launchd スケジュールを無効化
    fn disable_launchd(&self, priority: &Priority) -> Result<()> {
        let plist_path = self.get_launchd_plist_path(priority)?;

        let output = std::process::Command::new("launchctl")
            .args(["unload", &plist_path.to_string_lossy()])
            .output()
            .context("launchctl unload コマンド実行失敗")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // unloadは既に無効化されている場合エラーになることがあるが、続行
            eprintln!("警告: launchctl unload 失敗: {stderr}");
        }

        // plistファイルを削除
        if plist_path.exists() {
            std::fs::remove_file(&plist_path)
                .context("plistファイル削除失敗: plist_path.display()".to_string())?;
        }

        Ok(())
    }

    /// launchd スケジュールが有効かチェック
    fn is_launchd_enabled(&self, priority: &Priority) -> Result<bool> {
        let label = format!("com.backup-suite.{}", Self::priority_to_string(priority));

        let output = std::process::Command::new("launchctl")
            .args(["list", &label])
            .output()
            .context("launchctl list コマンド実行失敗")?;

        Ok(output.status.success())
    }

    // ========== Linux systemd 実装 ==========

    /// systemd service ファイルのパスを取得
    fn get_systemd_service_path(&self, priority: &Priority) -> Result<PathBuf> {
        let home = dirs::home_dir().context("ホームディレクトリが見つかりません")?;
        let filename = format!(
            "backup-suite-{}.service",
            Self::priority_to_string(priority)
        );
        Ok(home.join(".config/systemd/user").join(filename))
    }

    /// systemd timer ファイルのパスを取得
    fn get_systemd_timer_path(&self, priority: &Priority) -> Result<PathBuf> {
        let home = dirs::home_dir().context("ホームディレクトリが見つかりません")?;
        let filename = format!("backup-suite-{}.timer", Self::priority_to_string(priority));
        Ok(home.join(".config/systemd/user").join(filename))
    }

    /// systemd スケジュールをセットアップ
    fn setup_systemd(&self, priority: &Priority, frequency: Frequency) -> Result<()> {
        let service_path = self.get_systemd_service_path(priority)?;
        let timer_path = self.get_systemd_timer_path(priority)?;

        // ディレクトリを作成
        if let Some(parent) = service_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // service ファイルを生成
        let service_content = self.generate_systemd_service_content(priority)?;
        std::fs::write(&service_path, service_content)
            .context("serviceファイル書き込み失敗: service_path.display()".to_string())?;

        // timer ファイルを生成
        let timer_content = self.generate_systemd_timer_content(priority, frequency)?;
        std::fs::write(&timer_path, timer_content)
            .context("timerファイル書き込み失敗: timer_path.display()".to_string())?;

        // systemd をリロード
        let _output = std::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output()
            .context("systemctl daemon-reload 失敗")?;

        Ok(())
    }

    /// systemd service コンテンツを生成
    fn generate_systemd_service_content(&self, priority: &Priority) -> Result<String> {
        let backup_suite_path = std::env::current_exe()?;
        let priority_str = Self::priority_to_string(priority);

        let service = format!(
            r"[Unit]
Description=Backup Suite - {priority} Priority Backup
After=network.target

[Service]
Type=oneshot
ExecStart={backup_suite_path} run --priority {priority}
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
",
            priority = priority_str,
            backup_suite_path = backup_suite_path.display()
        );

        Ok(service)
    }

    /// systemd timer コンテンツを生成
    fn generate_systemd_timer_content(
        &self,
        priority: &Priority,
        frequency: Frequency,
    ) -> Result<String> {
        let priority_str = Self::priority_to_string(priority);

        let on_calendar = match frequency {
            Frequency::Daily => "*-*-* 02:00:00",
            Frequency::Weekly => "Sun *-*-* 02:00:00",
            Frequency::Monthly => "*-*-01 02:00:00",
            Frequency::Hourly => "*-*-* *:00:00",
        };

        let timer = format!(
            r"[Unit]
Description=Backup Suite - {priority_str} Priority Backup Timer

[Timer]
OnCalendar={on_calendar}
Persistent=true

[Install]
WantedBy=timers.target
"
        );

        Ok(timer)
    }

    /// systemd スケジュールを有効化
    fn enable_systemd(&self, priority: &Priority) -> Result<()> {
        let timer_name = format!("backup-suite-{}.timer", Self::priority_to_string(priority));

        // timerを有効化
        let output = std::process::Command::new("systemctl")
            .args(["--user", "enable", &timer_name])
            .output()
            .context("systemctl enable 失敗")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("systemctl enable 失敗: {stderr}"));
        }

        // timerを開始
        let output = std::process::Command::new("systemctl")
            .args(["--user", "start", &timer_name])
            .output()
            .context("systemctl start 失敗")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("systemctl start 失敗: {stderr}"));
        }

        Ok(())
    }

    /// systemd スケジュールを無効化
    fn disable_systemd(&self, priority: &Priority) -> Result<()> {
        let timer_name = format!("backup-suite-{}.timer", Self::priority_to_string(priority));

        // timerを停止
        let _output = std::process::Command::new("systemctl")
            .args(["--user", "stop", &timer_name])
            .output()
            .context("systemctl stop 失敗")?;

        // timerを無効化
        let _output = std::process::Command::new("systemctl")
            .args(["--user", "disable", &timer_name])
            .output()
            .context("systemctl disable 失敗")?;

        // ファイルを削除
        let service_path = self.get_systemd_service_path(priority)?;
        let timer_path = self.get_systemd_timer_path(priority)?;

        if service_path.exists() {
            std::fs::remove_file(&service_path)?;
        }
        if timer_path.exists() {
            std::fs::remove_file(&timer_path)?;
        }

        // systemd をリロード
        let _output = std::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output()
            .context("systemctl daemon-reload 失敗")?;

        Ok(())
    }

    /// systemd スケジュールが有効かチェック
    fn is_systemd_enabled(&self, priority: &Priority) -> Result<bool> {
        let timer_name = format!("backup-suite-{}.timer", Self::priority_to_string(priority));

        let output = std::process::Command::new("systemctl")
            .args(["--user", "is-enabled", &timer_name])
            .output()
            .context("systemctl is-enabled 失敗")?;

        Ok(output.status.success())
    }
}

/// スケジュール状態
#[derive(Debug, Clone, Default)]
pub struct ScheduleStatus {
    /// 高優先度が有効か
    pub high_enabled: bool,
    /// 中優先度が有効か
    pub medium_enabled: bool,
    /// 低優先度が有効か
    pub low_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        #[cfg(target_os = "macos")]
        assert_eq!(platform, Platform::MacOS);
        #[cfg(target_os = "linux")]
        assert_eq!(platform, Platform::Linux);
    }

    #[test]
    fn test_platform_is_supported() {
        assert!(Platform::MacOS.is_supported());
        assert!(Platform::Linux.is_supported());
        assert!(!Platform::Unsupported.is_supported());
    }

    #[test]
    fn test_frequency_parse() {
        assert_eq!(Frequency::parse("daily").unwrap(), Frequency::Daily);
        assert_eq!(Frequency::parse("weekly").unwrap(), Frequency::Weekly);
        assert_eq!(Frequency::parse("monthly").unwrap(), Frequency::Monthly);
        assert_eq!(Frequency::parse("hourly").unwrap(), Frequency::Hourly);
        assert!(Frequency::parse("invalid").is_err());
    }

    #[test]
    fn test_frequency_parse_case_insensitive() {
        assert_eq!(Frequency::parse("DAILY").unwrap(), Frequency::Daily);
        assert_eq!(Frequency::parse("WeEkLy").unwrap(), Frequency::Weekly);
        assert_eq!(Frequency::parse("MONTHLY").unwrap(), Frequency::Monthly);
        assert_eq!(Frequency::parse("HoUrLy").unwrap(), Frequency::Hourly);
    }

    #[test]
    fn test_frequency_as_str() {
        assert_eq!(Frequency::Daily.as_str(), "daily");
        assert_eq!(Frequency::Weekly.as_str(), "weekly");
        assert_eq!(Frequency::Monthly.as_str(), "monthly");
        assert_eq!(Frequency::Hourly.as_str(), "hourly");
    }

    #[test]
    fn test_frequency_roundtrip() {
        let frequencies = vec![
            Frequency::Daily,
            Frequency::Weekly,
            Frequency::Monthly,
            Frequency::Hourly,
        ];

        for freq in frequencies {
            let str_repr = freq.as_str();
            let parsed = Frequency::parse(str_repr).unwrap();
            assert_eq!(parsed, freq);
        }
    }

    #[test]
    fn test_scheduler_new_unsupported_platform() {
        // Unsupportedプラットフォームのテストは実行環境依存のため、
        // 現在のプラットフォームがサポートされている前提でテスト
        let mut config = Config::default();
        config.schedule.enabled = true;
        config.schedule.high_frequency = "daily".to_string();

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            let result = Scheduler::new(config);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_priority_to_string() {
        assert_eq!(Scheduler::priority_to_string(&Priority::High), "high");
        assert_eq!(Scheduler::priority_to_string(&Priority::Medium), "medium");
        assert_eq!(Scheduler::priority_to_string(&Priority::Low), "low");
    }

    #[test]
    fn test_get_frequency() {
        let mut config = Config::default();
        config.schedule.high_frequency = "hourly".to_string();
        config.schedule.medium_frequency = "daily".to_string();
        config.schedule.low_frequency = "weekly".to_string();

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            let scheduler = Scheduler::new(config).unwrap();

            assert_eq!(
                scheduler.get_frequency(&Priority::High).unwrap(),
                Frequency::Hourly
            );
            assert_eq!(
                scheduler.get_frequency(&Priority::Medium).unwrap(),
                Frequency::Daily
            );
            assert_eq!(
                scheduler.get_frequency(&Priority::Low).unwrap(),
                Frequency::Weekly
            );
        }
    }

    #[test]
    fn test_get_frequency_invalid() {
        let mut config = Config::default();
        config.schedule.high_frequency = "invalid_frequency".to_string();

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            let scheduler = Scheduler::new(config).unwrap();
            assert!(scheduler.get_frequency(&Priority::High).is_err());
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_generate_plist_content_daily() {
        let config = Config::default();
        let scheduler = Scheduler::new(config).unwrap();

        let content = scheduler
            .generate_plist_content(&Priority::High, Frequency::Daily)
            .unwrap();

        assert!(content.contains("com.backup-suite.high"));
        assert!(content.contains("<key>Hour</key>"));
        assert!(content.contains("<integer>2</integer>"));
        assert!(content.contains("StartCalendarInterval"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_generate_plist_content_weekly() {
        let config = Config::default();
        let scheduler = Scheduler::new(config).unwrap();

        let content = scheduler
            .generate_plist_content(&Priority::Medium, Frequency::Weekly)
            .unwrap();

        assert!(content.contains("com.backup-suite.medium"));
        assert!(content.contains("<key>Weekday</key>"));
        assert!(content.contains("<integer>0</integer>"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_generate_plist_content_monthly() {
        let config = Config::default();
        let scheduler = Scheduler::new(config).unwrap();

        let content = scheduler
            .generate_plist_content(&Priority::Low, Frequency::Monthly)
            .unwrap();

        assert!(content.contains("com.backup-suite.low"));
        assert!(content.contains("<key>Day</key>"));
        assert!(content.contains("<integer>1</integer>"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_generate_plist_content_hourly() {
        let config = Config::default();
        let scheduler = Scheduler::new(config).unwrap();

        let content = scheduler
            .generate_plist_content(&Priority::High, Frequency::Hourly)
            .unwrap();

        assert!(content.contains("<key>Minute</key>"));
        assert!(content.contains("<integer>0</integer>"));
        // Hourlyは Hour キーを持たない
        assert!(!content.contains("<key>Hour</key>"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_get_launchd_plist_path() {
        let config = Config::default();
        let scheduler = Scheduler::new(config).unwrap();

        let path = scheduler.get_launchd_plist_path(&Priority::High).unwrap();
        let path_str = path.to_string_lossy();

        assert!(path_str.contains("Library/LaunchAgents"));
        assert!(path_str.contains("com.backup-suite.high.plist"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_generate_systemd_service_content() {
        let config = Config::default();
        let scheduler = Scheduler::new(config).unwrap();

        let content = scheduler
            .generate_systemd_service_content(&Priority::High)
            .unwrap();

        assert!(content.contains("[Unit]"));
        assert!(content.contains("[Service]"));
        assert!(content.contains("Type=oneshot"));
        assert!(content.contains("--priority"));
        assert!(content.contains("high"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_generate_systemd_timer_content_daily() {
        let config = Config::default();
        let scheduler = Scheduler::new(config).unwrap();

        let content = scheduler
            .generate_systemd_timer_content(&Priority::High, Frequency::Daily)
            .unwrap();

        assert!(content.contains("[Unit]"));
        assert!(content.contains("[Timer]"));
        assert!(content.contains("OnCalendar=*-*-* 02:00:00"));
        assert!(content.contains("[Install]"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_generate_systemd_timer_content_weekly() {
        let config = Config::default();
        let scheduler = Scheduler::new(config).unwrap();

        let content = scheduler
            .generate_systemd_timer_content(&Priority::Medium, Frequency::Weekly)
            .unwrap();

        assert!(content.contains("OnCalendar=Sun *-*-* 02:00:00"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_generate_systemd_timer_content_monthly() {
        let config = Config::default();
        let scheduler = Scheduler::new(config).unwrap();

        let content = scheduler
            .generate_systemd_timer_content(&Priority::Low, Frequency::Monthly)
            .unwrap();

        assert!(content.contains("OnCalendar=*-*-01 02:00:00"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_generate_systemd_timer_content_hourly() {
        let config = Config::default();
        let scheduler = Scheduler::new(config).unwrap();

        let content = scheduler
            .generate_systemd_timer_content(&Priority::High, Frequency::Hourly)
            .unwrap();

        assert!(content.contains("OnCalendar=*-*-* *:00:00"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_get_systemd_paths() {
        let config = Config::default();
        let scheduler = Scheduler::new(config).unwrap();

        let service_path = scheduler.get_systemd_service_path(&Priority::High).unwrap();
        let timer_path = scheduler.get_systemd_timer_path(&Priority::High).unwrap();

        let service_str = service_path.to_string_lossy();
        let timer_str = timer_path.to_string_lossy();

        assert!(service_str.contains(".config/systemd/user"));
        assert!(service_str.contains("backup-suite-high.service"));
        assert!(timer_str.contains("backup-suite-high.timer"));
    }

    #[test]
    fn test_schedule_status_default() {
        let status = ScheduleStatus::default();
        assert!(!status.high_enabled);
        assert!(!status.medium_enabled);
        assert!(!status.low_enabled);
    }
}
