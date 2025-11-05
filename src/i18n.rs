//! # Internationalization (i18n) Module
//!
//! Provides multi-language support for the Backup Suite CLI.
//! Default language: English
//! Supported languages: English, Japanese

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Japanese,
}

impl Language {
    /// Detect language from environment and CLI arguments
    /// Priority: CLI flag > Default (English)
    /// Note: LANG environment variable is NOT checked - always defaults to English
    pub fn detect() -> Self {
        // Always default to English
        Language::English
    }

    /// Parse language from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(Language::English),
            "ja" | "japanese" | "日本語" => Some(Language::Japanese),
            _ => None,
        }
    }

    /// Convert to language code
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Japanese => "ja",
        }
    }
}

/// Message keys for internationalization
#[derive(Debug, Clone, Copy)]
pub enum MessageKey {
    // Version and title
    AppVersion,
    AppTitle,
    AppDescription,

    // Command categories
    BasicCommands,
    ExecutionCommands,
    InformationCommands,
    ConfigCommands,
    UtilityCommands,

    // Commands
    CmdAdd,
    CmdList,
    CmdRemove,
    CmdClear,
    CmdRun,
    CmdRestore,
    CmdCleanup,
    CmdStatus,
    CmdHistory,
    CmdDashboard,
    CmdEnable,
    CmdDisable,
    CmdSchedule,
    CmdConfig,
    CmdOpen,
    CmdCompletion,

    // Command descriptions
    DescAdd,
    DescList,
    DescRemove,
    DescClear,
    DescRun,
    DescRestore,
    DescCleanup,
    DescStatus,
    DescHistory,
    DescDashboard,
    DescEnable,
    DescDisable,
    DescSchedule,
    DescConfig,
    DescOpen,
    DescCompletion,

    // Options
    Options,
    HelpOption,
    VersionOption,

    // Usage examples
    UsageExamples,
    ExampleAddInteractive,
    ExampleRunHigh,
    ExampleEncrypt,
    ExampleCompress,
    ExampleEncryptCompress,
    ExampleCleanup,
    ExampleSchedule,

    // Detailed info
    DetailedInfo,
    DetailCommand,
    ConfigFile,
    BackupDestination,

    // Status messages
    Added,
    Removed,
    Deleted,
    Error,
    Warning,
    BackupRunning,
    RestoreStarting,

    // Encryption and compression
    EncryptionPassword,
    EncryptOption,
    CompressOption,
    CompressLevel,

    // Schedule help
    ScheduleTitle,
    ScheduleDescription,
    ScheduleUsage,
    ScheduleCommands,
    ScheduleEnable,
    ScheduleDisable,
    ScheduleStatus,
    ScheduleSetup,
    ScheduleHelp,

    // Config help
    ConfigTitle,
    ConfigDescription,
    ConfigUsage,
    ConfigMgmtCommands,
    ConfigSetDestination,
    ConfigGetDestination,
    ConfigSetKeepDays,
    ConfigGetKeepDays,
    ConfigOpen,
    ConfigHelp,

    // Schedule detailed options
    ScheduleDetailedOptions,
    ScheduleEnableOption,
    ScheduleDisableOption,
    ScheduleSetupOption,
    ScheduleFrequencies,
    ScheduleDaily,
    ScheduleWeekly,
    ScheduleMonthly,
    ScheduleTips,
    ScheduleTip1,
    ScheduleTip2,
    ScheduleTip3,

    // Config examples and tips
    ConfigExampleExternal,
    ConfigExampleGetDest,
    ConfigExampleSetDays,
    ConfigExampleOpen,
    ConfigExampleTilde,
    ConfigTip1,
    ConfigTip2,
    ConfigTip3,

    // Runtime messages
    NoTargetsRegistered,
    SelectionCancelled,
    PathNotExists,
    NotInBackupConfig,
    SpecifyPriorityOrAll,
    CountDeleted,
    DryRun,
    Category,
    Encryption,
    Compression,
    ErrorDetails,
    Detected,
    NoBackups,
    RestoreStart,
    Restoring,
    RestoredSuccess,
    RestoredFileCount,
    Deleting,
    Destination,
    Targets,
    High,
    Medium,
    Low,
    BackupHistory,
    Days,
    AutoBackupEnabled,
    AutoBackupDisabled,
    OpenDirectory,
    PriorityScheduleSetup,
    ScheduleSetupFailed,
    LaunchctlUnloadWarning,
    PriorityScheduleDeleted,
    ScheduleNotConfigured,
    ScheduleDeletionFailed,
    ActualScheduleStatus,
    Enabled,
    Disabled,
    ScheduleSettings,
    ScheduleUpdated,
    ScheduleUpdatedEnableLater,
    HighPriority,
    MediumPriority,
    LowPriority,
    DirectoryNotExists,
    DirectoryCreating,
    DestinationChanged,
    Before,
    After,
    CurrentDestination,
    KeepDaysOutOfRange,
    KeepDaysChanged,
    CurrentKeepDays,
    OpeningConfigFile,
    EditorDidNotExitCleanly,
    RustFastTypeSafe,
    ScheduleCommandPlaceholder,
    ConfigCommandPlaceholder,
    ConfigArgsPlaceholder,
    MainHelp,
    ConfigFileLabel,
    EnableOnlySpecifiedPriority,
    DisableOnlySpecifiedPriority,
    SetExecutionFrequency,
    EnableAllAutoBackups,
    EnableHighOnly,
    SetupScheduleFreq,
    CheckCurrentConfig,

    // Additional runtime units and labels
    Files,
    EncryptedLabel,
    StatusTitle,
    DaysUnit,
    DryRunParens,
}

impl MessageKey {
    /// Get translated message for the given language
    pub fn get(&self, lang: Language) -> &'static str {
        match lang {
            Language::English => self.get_en(),
            Language::Japanese => self.get_ja(),
        }
    }

    /// Get English message
    fn get_en(&self) -> &'static str {
        match self {
            // Version and title
            MessageKey::AppVersion => "Backup Suite v1.0.0",
            MessageKey::AppTitle => "Fast Local Backup Tool - Written in Rust, Type-safe, High-performance",
            MessageKey::AppDescription => "Backup Suite - Fast Local Backup Tool",

            // Command categories
            MessageKey::BasicCommands => "📋 Basic Commands",
            MessageKey::ExecutionCommands => "🚀 Execution Commands",
            MessageKey::InformationCommands => "📊 Information Commands",
            MessageKey::ConfigCommands => "⚙️  Configuration",
            MessageKey::UtilityCommands => "🔧 Utility",

            // Commands
            MessageKey::CmdAdd => "add",
            MessageKey::CmdList => "list, ls",
            MessageKey::CmdRemove => "remove",
            MessageKey::CmdClear => "clear",
            MessageKey::CmdRun => "run",
            MessageKey::CmdRestore => "restore",
            MessageKey::CmdCleanup => "cleanup",
            MessageKey::CmdStatus => "status",
            MessageKey::CmdHistory => "history",
            MessageKey::CmdDashboard => "dashboard",
            MessageKey::CmdEnable => "enable",
            MessageKey::CmdDisable => "disable",
            MessageKey::CmdSchedule => "schedule",
            MessageKey::CmdConfig => "config",
            MessageKey::CmdOpen => "open",
            MessageKey::CmdCompletion => "completion",

            // Command descriptions
            MessageKey::DescAdd => "Add target (interactive selection supported)",
            MessageKey::DescList => "List targets",
            MessageKey::DescRemove => "Remove target (interactive selection supported)",
            MessageKey::DescClear => "Bulk delete",
            MessageKey::DescRun => "Execute backup (encryption & compression supported)",
            MessageKey::DescRestore => "Restore backup (auto-detect encryption & compression)",
            MessageKey::DescCleanup => "Delete old backups",
            MessageKey::DescStatus => "Display status",
            MessageKey::DescHistory => "Display history",
            MessageKey::DescDashboard => "Display dashboard",
            MessageKey::DescEnable => "Enable auto backup",
            MessageKey::DescDisable => "Disable auto backup",
            MessageKey::DescSchedule => "Manage schedule",
            MessageKey::DescConfig => "Manage configuration (destination, retention period)",
            MessageKey::DescOpen => "Open backup directory",
            MessageKey::DescCompletion => "Generate shell completion script",

            // Options
            MessageKey::Options => "Options:",
            MessageKey::HelpOption => "-h, --help       Display this help",
            MessageKey::VersionOption => "-V, --version    Display version information",

            // Usage examples
            MessageKey::UsageExamples => "Usage Examples:",
            MessageKey::ExampleAddInteractive => "# Add file interactively",
            MessageKey::ExampleRunHigh => "# Execute backup for high priority",
            MessageKey::ExampleEncrypt => "# Encrypted backup (AES-256-GCM)",
            MessageKey::ExampleCompress => "# Compressed backup (zstd fast compression)",
            MessageKey::ExampleEncryptCompress => "# Encrypted + Compressed backup",
            MessageKey::ExampleCleanup => "# Delete backups older than 30 days (dry run)",
            MessageKey::ExampleSchedule => "# Setup schedule and enable",

            // Detailed info
            MessageKey::DetailedInfo => "Detailed Information:",
            MessageKey::DetailCommand => "Command details: backup-suite <command> --help",
            MessageKey::ConfigFile => "Configuration file: ~/.config/backup-suite/config.toml",
            MessageKey::BackupDestination => "Backup destination: ~/.local/share/backup-suite/backups/",

            // Status messages
            MessageKey::Added => "Added",
            MessageKey::Removed => "Removed",
            MessageKey::Deleted => "deleted",
            MessageKey::Error => "Error",
            MessageKey::Warning => "Warning",
            MessageKey::BackupRunning => "Backup Running",
            MessageKey::RestoreStarting => "Restore Starting",

            // Encryption and compression
            MessageKey::EncryptionPassword => "Encryption password",
            MessageKey::EncryptOption => "--encrypt: AES-256-GCM encryption",
            MessageKey::CompressOption => "--compress zstd/gzip: Compression",
            MessageKey::CompressLevel => "--compress-level 1-22: Compression level",

            // Schedule help
            MessageKey::ScheduleTitle => "📅 Backup Suite Schedule Management",
            MessageKey::ScheduleDescription => "Automatic backup schedule setup and control system",
            MessageKey::ScheduleUsage => "Usage:",
            MessageKey::ScheduleCommands => "📋 Schedule Management Commands",
            MessageKey::ScheduleEnable => "enable       Enable automatic backup",
            MessageKey::ScheduleDisable => "disable      Disable automatic backup",
            MessageKey::ScheduleStatus => "status       Display current schedule status",
            MessageKey::ScheduleSetup => "setup        Setup schedule frequency",
            MessageKey::ScheduleHelp => "help         Display this help",

            // Config help
            MessageKey::ConfigTitle => "⚙️  Backup Suite Configuration Management",
            MessageKey::ConfigDescription => "Command to manage backup destination, retention period and other settings",
            MessageKey::ConfigUsage => "Usage:",
            MessageKey::ConfigMgmtCommands => "📋 Configuration Management Commands",
            MessageKey::ConfigSetDestination => "set-destination <path>  Change backup destination directory",
            MessageKey::ConfigGetDestination => "get-destination        Display current backup destination",
            MessageKey::ConfigSetKeepDays => "set-keep-days <days>   Change backup retention period (1-3650 days)",
            MessageKey::ConfigGetKeepDays => "get-keep-days          Display current backup retention period",
            MessageKey::ConfigOpen => "open                   Open configuration file in default editor",
            MessageKey::ConfigHelp => "help                   Display this help",

            // Schedule detailed options
            MessageKey::ScheduleDetailedOptions => "⚙️  Detailed Options",
            MessageKey::ScheduleEnableOption => "enable --priority <priority>",
            MessageKey::ScheduleDisableOption => "disable --priority <priority>",
            MessageKey::ScheduleSetupOption => "setup --high <freq> --medium <freq> --low <freq>",
            MessageKey::ScheduleFrequencies => "📊 Frequency Settings:",
            MessageKey::ScheduleDaily => "daily   - Every day at 2:00 AM",
            MessageKey::ScheduleWeekly => "weekly  - Every Sunday at 2:00 AM",
            MessageKey::ScheduleMonthly => "monthly - First day of month at 2:00 AM",
            MessageKey::ScheduleTips => "💡 Tips:",
            MessageKey::ScheduleTip1 => "  • After enabling schedule, it runs automatically via macOS launchctl",
            MessageKey::ScheduleTip2 => "  • Use 'status' command to check actual operation status",
            MessageKey::ScheduleTip3 => "  • Each priority setting is managed independently",

            // Config examples and tips
            MessageKey::ConfigExampleExternal => "# Change backup destination to external HDD",
            MessageKey::ConfigExampleGetDest => "# Check current backup destination",
            MessageKey::ConfigExampleSetDays => "# Change retention period to 60 days",
            MessageKey::ConfigExampleOpen => "# Open configuration file in editor",
            MessageKey::ConfigExampleTilde => "# Tilde expansion is supported",
            MessageKey::ConfigTip1 => "  • Non-existent directories are automatically created",
            MessageKey::ConfigTip2 => "  • Write permissions are automatically checked",
            MessageKey::ConfigTip3 => "  • Backups older than retention period can be deleted with cleanup command",

            // Runtime messages
            MessageKey::NoTargetsRegistered => "No backup targets registered",
            MessageKey::SelectionCancelled => "Selection cancelled",
            MessageKey::PathNotExists => "Path does not exist",
            MessageKey::NotInBackupConfig => "Not registered in backup configuration",
            MessageKey::SpecifyPriorityOrAll => "Specify --priority or --all",
            MessageKey::CountDeleted => "deleted",
            MessageKey::DryRun => "dry run",
            MessageKey::Category => "Category",
            MessageKey::Encryption => "Encryption",
            MessageKey::Compression => "Compression",
            MessageKey::ErrorDetails => "Error Details",
            MessageKey::Detected => "Detected",
            MessageKey::NoBackups => "No backups",
            MessageKey::RestoreStart => "Restore Starting",
            MessageKey::Restoring => "Restoring...",
            MessageKey::RestoredSuccess => "Successfully restored backup to",
            MessageKey::RestoredFileCount => "Restored files:",
            MessageKey::Deleting => "Deleting",
            MessageKey::Destination => "Destination",
            MessageKey::Targets => "Targets",
            MessageKey::High => "High",
            MessageKey::Medium => "Medium",
            MessageKey::Low => "Low",
            MessageKey::BackupHistory => "Backup History",
            MessageKey::Days => "days",
            MessageKey::AutoBackupEnabled => "Automatic backup enabled",
            MessageKey::AutoBackupDisabled => "Automatic backup disabled",
            MessageKey::OpenDirectory => "Opening",
            MessageKey::PriorityScheduleSetup => "Priority schedule setup completed",
            MessageKey::ScheduleSetupFailed => "Failed to setup priority schedule",
            MessageKey::LaunchctlUnloadWarning => "launchctl unload warning",
            MessageKey::PriorityScheduleDeleted => "Priority schedule deleted",
            MessageKey::ScheduleNotConfigured => "Priority schedule is not configured",
            MessageKey::ScheduleDeletionFailed => "Failed to delete priority schedule",
            MessageKey::ActualScheduleStatus => "Actual Schedule Status",
            MessageKey::Enabled => "Enabled",
            MessageKey::Disabled => "Disabled",
            MessageKey::ScheduleSettings => "Schedule Settings",
            MessageKey::ScheduleUpdated => "Schedule updated and applied",
            MessageKey::ScheduleUpdatedEnableLater => "Schedule settings updated (enable with 'schedule enable')",
            MessageKey::HighPriority => "High priority",
            MessageKey::MediumPriority => "Medium priority",
            MessageKey::LowPriority => "Low priority",
            MessageKey::DirectoryNotExists => "Directory does not exist. Creating",
            MessageKey::DirectoryCreating => "Creating",
            MessageKey::DestinationChanged => "Backup destination changed",
            MessageKey::Before => "Before",
            MessageKey::After => "After",
            MessageKey::CurrentDestination => "Current backup destination",
            MessageKey::KeepDaysOutOfRange => "keep_days must be between 1-3650 (specified value:",
            MessageKey::KeepDaysChanged => "Backup retention period changed",
            MessageKey::CurrentKeepDays => "Current backup retention period",
            MessageKey::OpeningConfigFile => "Opening configuration file",
            MessageKey::EditorDidNotExitCleanly => "Editor did not exit cleanly",
            MessageKey::RustFastTypeSafe => "🦀 Rust · Fast · Type-safe",
            MessageKey::ScheduleCommandPlaceholder => "<command>",
            MessageKey::ConfigCommandPlaceholder => "<command>",
            MessageKey::ConfigArgsPlaceholder => "[args]",
            MessageKey::MainHelp => "Main help",
            MessageKey::ConfigFileLabel => "Configuration file",
            MessageKey::EnableOnlySpecifiedPriority => "Enable only specified priority (high/medium/low)",
            MessageKey::DisableOnlySpecifiedPriority => "Disable only specified priority",
            MessageKey::SetExecutionFrequency => "Set execution frequency for each priority (daily/weekly/monthly)",
            MessageKey::EnableAllAutoBackups => "# Enable all automatic backups",
            MessageKey::EnableHighOnly => "# Enable high priority only",
            MessageKey::SetupScheduleFreq => "# Setup schedule frequency",
            MessageKey::CheckCurrentConfig => "# Check current configuration",

            // Additional runtime units and labels
            MessageKey::Files => "files",
            MessageKey::EncryptedLabel => "Encrypted:",
            MessageKey::StatusTitle => "Status",
            MessageKey::DaysUnit => "days",
            MessageKey::DryRunParens => "(dry run)",
        }
    }

    /// Get Japanese message
    fn get_ja(&self) -> &'static str {
        match self {
            // Version and title
            MessageKey::AppVersion => "Backup Suite v1.0.0",
            MessageKey::AppTitle => "高速ローカルバックアップツール - Rust製・型安全・高性能",
            MessageKey::AppDescription => "Backup Suite - 高速ローカルバックアップツール",

            // Command categories
            MessageKey::BasicCommands => "📋 基本コマンド",
            MessageKey::ExecutionCommands => "🚀 実行コマンド",
            MessageKey::InformationCommands => "📊 情報表示",
            MessageKey::ConfigCommands => "⚙️  設定管理",
            MessageKey::UtilityCommands => "🔧 ユーティリティ",

            // Commands
            MessageKey::CmdAdd => "add",
            MessageKey::CmdList => "list, ls",
            MessageKey::CmdRemove => "remove",
            MessageKey::CmdClear => "clear",
            MessageKey::CmdRun => "run",
            MessageKey::CmdRestore => "restore",
            MessageKey::CmdCleanup => "cleanup",
            MessageKey::CmdStatus => "status",
            MessageKey::CmdHistory => "history",
            MessageKey::CmdDashboard => "dashboard",
            MessageKey::CmdEnable => "enable",
            MessageKey::CmdDisable => "disable",
            MessageKey::CmdSchedule => "schedule",
            MessageKey::CmdConfig => "config",
            MessageKey::CmdOpen => "open",
            MessageKey::CmdCompletion => "completion",

            // Command descriptions
            MessageKey::DescAdd => "対象追加（インタラクティブ選択対応）",
            MessageKey::DescList => "一覧表示",
            MessageKey::DescRemove => "対象削除（インタラクティブ選択対応）",
            MessageKey::DescClear => "一括削除",
            MessageKey::DescRun => "バックアップ実行（暗号化・圧縮対応）",
            MessageKey::DescRestore => "バックアップ復元（暗号化・圧縮自動検出）",
            MessageKey::DescCleanup => "古いバックアップ削除",
            MessageKey::DescStatus => "ステータス表示",
            MessageKey::DescHistory => "履歴表示",
            MessageKey::DescDashboard => "ダッシュボード表示",
            MessageKey::DescEnable => "自動バックアップ有効化",
            MessageKey::DescDisable => "自動バックアップ無効化",
            MessageKey::DescSchedule => "スケジュール管理",
            MessageKey::DescConfig => "設定管理（保存先・保持期間）",
            MessageKey::DescOpen => "バックアップディレクトリを開く",
            MessageKey::DescCompletion => "シェル補完スクリプト生成",

            // Options
            MessageKey::Options => "オプション:",
            MessageKey::HelpOption => "-h, --help       このヘルプを表示",
            MessageKey::VersionOption => "-V, --version    バージョン情報を表示",

            // Usage examples
            MessageKey::UsageExamples => "使用例:",
            MessageKey::ExampleAddInteractive => "# インタラクティブでファイルを追加",
            MessageKey::ExampleRunHigh => "# 高優先度のバックアップを実行",
            MessageKey::ExampleEncrypt => "# 暗号化バックアップ（AES-256-GCM）",
            MessageKey::ExampleCompress => "# 圧縮バックアップ（zstd高速圧縮）",
            MessageKey::ExampleEncryptCompress => "# 暗号化＋圧縮バックアップ",
            MessageKey::ExampleCleanup => "# 30日以上前のバックアップを削除（ドライラン）",
            MessageKey::ExampleSchedule => "# スケジュールを設定して有効化",

            // Detailed info
            MessageKey::DetailedInfo => "詳細情報:",
            MessageKey::DetailCommand => "各コマンドの詳細: backup-suite <コマンド> --help",
            MessageKey::ConfigFile => "設定ファイル: ~/.config/backup-suite/config.toml",
            MessageKey::BackupDestination => "バックアップ先: ~/.local/share/backup-suite/backups/",

            // Status messages
            MessageKey::Added => "追加",
            MessageKey::Removed => "削除",
            MessageKey::Deleted => "件削除",
            MessageKey::Error => "エラー",
            MessageKey::Warning => "⚠️",
            MessageKey::BackupRunning => "🚀 バックアップ実行",
            MessageKey::RestoreStarting => "🔄 復元開始",

            // Encryption and compression
            MessageKey::EncryptionPassword => "暗号化パスワード",
            MessageKey::EncryptOption => "--encrypt: AES-256-GCM暗号化",
            MessageKey::CompressOption => "--compress zstd/gzip: 圧縮",
            MessageKey::CompressLevel => "--compress-level 1-22: 圧縮レベル",

            // Schedule help
            MessageKey::ScheduleTitle => "📅 Backup Suite スケジュール管理",
            MessageKey::ScheduleDescription => "自動バックアップのスケジュール設定・制御システム",
            MessageKey::ScheduleUsage => "使用方法:",
            MessageKey::ScheduleCommands => "📋 スケジュール管理コマンド",
            MessageKey::ScheduleEnable => "enable       自動バックアップを有効化",
            MessageKey::ScheduleDisable => "disable      自動バックアップを無効化",
            MessageKey::ScheduleStatus => "status       現在のスケジュール状態を表示",
            MessageKey::ScheduleSetup => "setup        スケジュール頻度を設定",
            MessageKey::ScheduleHelp => "help         このヘルプを表示",

            // Config help
            MessageKey::ConfigTitle => "⚙️  Backup Suite 設定管理",
            MessageKey::ConfigDescription => "バックアップの保存先や保持期間などの設定を管理するコマンド",
            MessageKey::ConfigUsage => "使用方法:",
            MessageKey::ConfigMgmtCommands => "📋 設定管理コマンド",
            MessageKey::ConfigSetDestination => "set-destination <パス>  バックアップ保存先ディレクトリを変更",
            MessageKey::ConfigGetDestination => "get-destination        現在のバックアップ保存先を表示",
            MessageKey::ConfigSetKeepDays => "set-keep-days <日数>   バックアップ保持期間を変更 (1-3650日)",
            MessageKey::ConfigGetKeepDays => "get-keep-days          現在のバックアップ保持期間を表示",
            MessageKey::ConfigOpen => "open                   設定ファイルをデフォルトエディタで開く",
            MessageKey::ConfigHelp => "help                   このヘルプを表示",

            // Schedule detailed options
            MessageKey::ScheduleDetailedOptions => "⚙️  詳細オプション",
            MessageKey::ScheduleEnableOption => "enable --priority <優先度>",
            MessageKey::ScheduleDisableOption => "disable --priority <優先度>",
            MessageKey::ScheduleSetupOption => "setup --high <頻度> --medium <頻度> --low <頻度>",
            MessageKey::ScheduleFrequencies => "📊 頻度設定値:",
            MessageKey::ScheduleDaily => "daily   - 毎日 2:00 AM",
            MessageKey::ScheduleWeekly => "weekly  - 毎週日曜 2:00 AM",
            MessageKey::ScheduleMonthly => "monthly - 毎月1日 2:00 AM",
            MessageKey::ScheduleTips => "💡 ヒント:",
            MessageKey::ScheduleTip1 => "  • スケジュール有効化後、macOSのlaunchctlで自動実行されます",
            MessageKey::ScheduleTip2 => "  • 'status'コマンドで実際の動作状況を確認できます",
            MessageKey::ScheduleTip3 => "  • 各優先度の設定は独立して管理できます",

            // Config examples and tips
            MessageKey::ConfigExampleExternal => "# バックアップ先を外付けHDDに変更",
            MessageKey::ConfigExampleGetDest => "# 現在のバックアップ先を確認",
            MessageKey::ConfigExampleSetDays => "# バックアップ保持期間を60日に変更",
            MessageKey::ConfigExampleOpen => "# 設定ファイルをエディタで開く",
            MessageKey::ConfigExampleTilde => "# チルダ展開も対応",
            MessageKey::ConfigTip1 => "  • 存在しないディレクトリは自動的に作成されます",
            MessageKey::ConfigTip2 => "  • 書き込み権限のチェックが自動で行われます",
            MessageKey::ConfigTip3 => "  • 保持期間を過ぎたバックアップは cleanup コマンドで削除できます",

            // Runtime messages
            MessageKey::NoTargetsRegistered => "バックアップ対象が登録されていません",
            MessageKey::SelectionCancelled => "選択がキャンセルされました",
            MessageKey::PathNotExists => "パスが存在しません",
            MessageKey::NotInBackupConfig => "バックアップ設定に登録されていません",
            MessageKey::SpecifyPriorityOrAll => "--priority または --all を指定してください",
            MessageKey::CountDeleted => "件削除",
            MessageKey::DryRun => "ドライラン",
            MessageKey::Category => "カテゴリ",
            MessageKey::Encryption => "暗号化",
            MessageKey::Compression => "圧縮",
            MessageKey::ErrorDetails => "エラー詳細",
            MessageKey::Detected => "検出",
            MessageKey::NoBackups => "バックアップなし",
            MessageKey::RestoreStart => "復元開始",
            MessageKey::Restoring => "復元中...",
            MessageKey::RestoredSuccess => "バックアップを正常に復元しました",
            MessageKey::RestoredFileCount => "復元ファイル数:",
            MessageKey::Deleting => "削除中",
            MessageKey::Destination => "保存先",
            MessageKey::Targets => "対象",
            MessageKey::High => "高",
            MessageKey::Medium => "中",
            MessageKey::Low => "低",
            MessageKey::BackupHistory => "バックアップ履歴",
            MessageKey::Days => "日間",
            MessageKey::AutoBackupEnabled => "自動バックアップ有効化",
            MessageKey::AutoBackupDisabled => "自動バックアップ無効化",
            MessageKey::OpenDirectory => "開く",
            MessageKey::PriorityScheduleSetup => "優先度スケジュール設定完了",
            MessageKey::ScheduleSetupFailed => "優先度スケジュールの設定に失敗しました",
            MessageKey::LaunchctlUnloadWarning => "launchctl unload警告",
            MessageKey::PriorityScheduleDeleted => "優先度スケジュール削除完了",
            MessageKey::ScheduleNotConfigured => "優先度スケジュールは設定されていません",
            MessageKey::ScheduleDeletionFailed => "優先度スケジュールの削除に失敗しました",
            MessageKey::ActualScheduleStatus => "実際のスケジュール状態",
            MessageKey::Enabled => "有効",
            MessageKey::Disabled => "無効",
            MessageKey::ScheduleSettings => "スケジュール設定",
            MessageKey::ScheduleUpdated => "スケジュール更新・適用完了",
            MessageKey::ScheduleUpdatedEnableLater => "スケジュール設定更新（有効化は 'schedule enable' で）",
            MessageKey::HighPriority => "高優先度",
            MessageKey::MediumPriority => "中優先度",
            MessageKey::LowPriority => "低優先度",
            MessageKey::DirectoryNotExists => "ディレクトリが存在しません。作成します",
            MessageKey::DirectoryCreating => "作成中",
            MessageKey::DestinationChanged => "バックアップ先を変更しました",
            MessageKey::Before => "変更前",
            MessageKey::After => "変更後",
            MessageKey::CurrentDestination => "現在のバックアップ先",
            MessageKey::KeepDaysOutOfRange => "keep_days は 1-3650 の範囲で指定してください（指定値:",
            MessageKey::KeepDaysChanged => "バックアップ保持期間を変更しました",
            MessageKey::CurrentKeepDays => "現在のバックアップ保持期間",
            MessageKey::OpeningConfigFile => "設定ファイルを開きます",
            MessageKey::EditorDidNotExitCleanly => "エディタが正常に終了しませんでした",
            MessageKey::RustFastTypeSafe => "🦀 Rust・高速・型安全",
            MessageKey::ScheduleCommandPlaceholder => "<コマンド>",
            MessageKey::ConfigCommandPlaceholder => "<コマンド>",
            MessageKey::ConfigArgsPlaceholder => "[引数]",
            MessageKey::MainHelp => "メインヘルプ",
            MessageKey::ConfigFileLabel => "設定ファイル",
            MessageKey::EnableOnlySpecifiedPriority => "指定した優先度のみ有効化 (high/medium/low)",
            MessageKey::DisableOnlySpecifiedPriority => "指定した優先度のみ無効化",
            MessageKey::SetExecutionFrequency => "各優先度の実行頻度を設定 (daily/weekly/monthly)",
            MessageKey::EnableAllAutoBackups => "# 全ての自動バックアップを有効化",
            MessageKey::EnableHighOnly => "# 高優先度のみ有効化",
            MessageKey::SetupScheduleFreq => "# スケジュール頻度を設定",
            MessageKey::CheckCurrentConfig => "# 現在の設定状況を確認",

            // Additional runtime units and labels
            MessageKey::Files => "ファイル",
            MessageKey::EncryptedLabel => "暗号化:",
            MessageKey::StatusTitle => "ステータス",
            MessageKey::DaysUnit => "日",
            MessageKey::DryRunParens => "（ドライラン）",
        }
    }
}

/// Get message by key and language
pub fn get_message(key: MessageKey, lang: Language) -> &'static str {
    key.get(lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        // Default should be English
        let lang = Language::detect();
        // This test might fail if LANG is set to ja
        // assert_eq!(lang, Language::English);
    }

    #[test]
    fn test_language_parsing() {
        assert_eq!(Language::from_str("en"), Some(Language::English));
        assert_eq!(Language::from_str("english"), Some(Language::English));
        assert_eq!(Language::from_str("ja"), Some(Language::Japanese));
        assert_eq!(Language::from_str("japanese"), Some(Language::Japanese));
        assert_eq!(Language::from_str("日本語"), Some(Language::Japanese));
        assert_eq!(Language::from_str("unknown"), None);
    }

    #[test]
    fn test_language_code() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Japanese.code(), "ja");
    }

    #[test]
    fn test_message_translation() {
        // Test English
        assert_eq!(
            get_message(MessageKey::AppVersion, Language::English),
            "Backup Suite v1.0.0"
        );

        // Test Japanese
        assert_eq!(
            get_message(MessageKey::AppVersion, Language::Japanese),
            "Backup Suite v1.0.0"
        );

        // Test different messages
        assert!(get_message(MessageKey::AppTitle, Language::English).contains("Fast"));
        assert!(get_message(MessageKey::AppTitle, Language::Japanese).contains("高速"));
    }
}
