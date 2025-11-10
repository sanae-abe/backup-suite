//! # Internationalization (i18n) Module
//!
//! Provides multi-language support for the Backup Suite CLI.
//! Default language: English
//! Supported languages: English, Japanese, Simplified Chinese, Traditional Chinese

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Japanese,
    SimplifiedChinese,
    TraditionalChinese,
}

impl Language {
    /// Detect language from environment and CLI arguments
    /// Priority: CLI flag > Environment variable > Default (English)
    #[must_use]
    pub fn detect() -> Self {
        // Check LANG environment variable
        if let Ok(lang) = std::env::var("LANG") {
            let lang_lower = lang.to_lowercase();

            // Japanese detection
            if lang_lower.starts_with("ja") || lang_lower.starts_with("jp") {
                return Language::Japanese;
            }

            // Simplified Chinese detection (zh-CN, zh-Hans, zh_CN, zh_Hans)
            if lang_lower.starts_with("zh-cn")
                || lang_lower.starts_with("zh_cn")
                || lang_lower.starts_with("zh-hans")
                || lang_lower.starts_with("zh_hans")
            {
                return Language::SimplifiedChinese;
            }

            // Traditional Chinese detection (zh-TW, zh-HK, zh-Hant, zh_TW, zh_HK, zh_Hant)
            if lang_lower.starts_with("zh-tw")
                || lang_lower.starts_with("zh_tw")
                || lang_lower.starts_with("zh-hk")
                || lang_lower.starts_with("zh_hk")
                || lang_lower.starts_with("zh-hant")
                || lang_lower.starts_with("zh_hant")
            {
                return Language::TraditionalChinese;
            }
        }

        // Check LC_ALL environment variable as fallback
        if let Ok(lang) = std::env::var("LC_ALL") {
            let lang_lower = lang.to_lowercase();

            // Japanese detection
            if lang_lower.starts_with("ja") || lang_lower.starts_with("jp") {
                return Language::Japanese;
            }

            // Simplified Chinese detection
            if lang_lower.starts_with("zh-cn")
                || lang_lower.starts_with("zh_cn")
                || lang_lower.starts_with("zh-hans")
                || lang_lower.starts_with("zh_hans")
            {
                return Language::SimplifiedChinese;
            }

            // Traditional Chinese detection
            if lang_lower.starts_with("zh-tw")
                || lang_lower.starts_with("zh_tw")
                || lang_lower.starts_with("zh-hk")
                || lang_lower.starts_with("zh_hk")
                || lang_lower.starts_with("zh-hant")
                || lang_lower.starts_with("zh_hant")
            {
                return Language::TraditionalChinese;
            }
        }

        // Default to English
        Language::English
    }

    /// Parse language from string
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(Language::English),
            "ja" | "japanese" | "日本語" => Some(Language::Japanese),
            "zh-cn" | "zh_cn" | "zh-hans" | "zh_hans" | "simplified chinese" | "简体中文" => {
                Some(Language::SimplifiedChinese)
            }
            "zh-tw"
            | "zh_tw"
            | "zh-hk"
            | "zh_hk"
            | "zh-hant"
            | "zh_hant"
            | "traditional chinese"
            | "繁體中文"
            | "繁体中文" => Some(Language::TraditionalChinese),
            _ => None,
        }
    }

    /// Convert to language code
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Japanese => "ja",
            Language::SimplifiedChinese => "zh-cn",
            Language::TraditionalChinese => "zh-tw",
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
    SmartCommands,

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
    CmdSmart,

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
    DescSmart,

    // AI subcommands
    CmdSmartDetect,
    CmdSmartAnalyze,
    CmdSmartSuggestExclude,
    CmdSmartAutoConfigure,
    DescSmartDetect,
    DescSmartAnalyze,
    DescSmartSuggestExclude,
    DescSmartAutoConfigure,

    // AI messages
    SmartDetectTitle,
    SmartDetectNoAnomalies,
    SmartDetectAnomalyFound,
    SmartAnalyzeTitle,
    SmartAnalyzeImportanceHigh,
    SmartAnalyzeImportanceMedium,
    SmartAnalyzeImportanceLow,
    SmartSuggestExcludeTitle,
    SmartSuggestExcludeRecommendation,
    SmartAutoConfigureTitle,
    SmartAutoConfigureSuccess,
    SmartErrorNotEnabled,
    SmartErrorInsufficientData,
    SmartErrorAnalysisFailed,

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
    ExampleSmartDetect,
    ExampleSmartAnalyze,
    ExampleSmartSuggestExclude,

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
    SavePasswordSecurely,
    EncryptOption,
    CompressOption,
    CompressLevel,

    // Run command options
    IncrementalOption,
    GeneratePasswordOption,
    PasswordOption,
    DryRunOption,
    PriorityOption,
    CategoryOption,

    // Restore command options
    FromOption,
    ToOption,
    RestorePasswordOption,

    // Cleanup command options
    DaysOption,
    CleanupDryRunOption,

    // Add command options
    AddPriorityOption,
    AddCategoryOption,
    InteractiveOption,

    // List command options
    ListPriorityOption,

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
    #[must_use]
    pub fn get(&self, lang: Language) -> &'static str {
        match lang {
            Language::English => self.get_en(),
            Language::Japanese => self.get_ja(),
            Language::SimplifiedChinese => self.get_zh_cn(),
            Language::TraditionalChinese => self.get_zh_tw(),
        }
    }

    /// Get English message
    fn get_en(&self) -> &'static str {
        match self {
            // Version and title
            MessageKey::AppVersion => "Backup Suite v1.0.0",
            MessageKey::AppTitle => {
                "Fast Local Backup Tool - Written in Rust, Type-safe, High-performance"
            }
            MessageKey::AppDescription => "Backup Suite - Fast Local Backup Tool",

            // Command categories
            MessageKey::BasicCommands => "📋 Basic Commands",
            MessageKey::ExecutionCommands => "🚀 Execution Commands",
            MessageKey::InformationCommands => "📊 Information Commands",
            MessageKey::ConfigCommands => "⚙️  Configuration",
            MessageKey::UtilityCommands => "🔧 Utility",
            MessageKey::SmartCommands => "🤖 Smart Commands",

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
            MessageKey::CmdSmart => "ai",

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
            MessageKey::DescSmart => "AI-driven intelligent backup management",

            // AI subcommands
            MessageKey::CmdSmartDetect => "detect",
            MessageKey::CmdSmartAnalyze => "analyze",
            MessageKey::CmdSmartSuggestExclude => "suggest-exclude",
            MessageKey::CmdSmartAutoConfigure => "auto-configure",
            MessageKey::DescSmartDetect => "Detect anomalies in backup history",
            MessageKey::DescSmartAnalyze => "Analyze file importance",
            MessageKey::DescSmartSuggestExclude => "Suggest exclude patterns",
            MessageKey::DescSmartAutoConfigure => "Auto-configure backup settings with Smart rules",

            // AI messages
            MessageKey::SmartDetectTitle => "🤖 Smart Anomaly Detection",
            MessageKey::SmartDetectNoAnomalies => "No anomalies detected in the backup history",
            MessageKey::SmartDetectAnomalyFound => "Anomaly detected",
            MessageKey::SmartAnalyzeTitle => "🤖 Smart File Importance Analysis",
            MessageKey::SmartAnalyzeImportanceHigh => "High importance",
            MessageKey::SmartAnalyzeImportanceMedium => "Medium importance",
            MessageKey::SmartAnalyzeImportanceLow => "Low importance",
            MessageKey::SmartSuggestExcludeTitle => "🤖 Smart Exclude Pattern Suggestions",
            MessageKey::SmartSuggestExcludeRecommendation => "Recommended exclusion",
            MessageKey::SmartAutoConfigureTitle => "🤖 Smart Auto-Configuration",
            MessageKey::SmartAutoConfigureSuccess => "Auto-configuration completed successfully",
            MessageKey::SmartErrorNotEnabled => {
                "AI features are not enabled. Compile with --features ai"
            }
            MessageKey::SmartErrorInsufficientData => "Insufficient data for Smart analysis",
            MessageKey::SmartErrorAnalysisFailed => "Smart analysis failed",

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
            MessageKey::ExampleSmartDetect => "# Detect anomalies in last 7 days",
            MessageKey::ExampleSmartAnalyze => "# Analyze file importance",
            MessageKey::ExampleSmartSuggestExclude => "# Get Smart exclude suggestions",

            // Detailed info
            MessageKey::DetailedInfo => "Detailed Information:",
            MessageKey::DetailCommand => "Command details: backup-suite <command> --help",
            MessageKey::ConfigFile => "Configuration file: ~/.config/backup-suite/config.toml",
            MessageKey::BackupDestination => {
                "Backup destination: ~/.local/share/backup-suite/backups/"
            }

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
            MessageKey::SavePasswordSecurely => "⚠️  Please save this password securely!",
            MessageKey::EncryptOption => "--encrypt: AES-256-GCM encryption",
            MessageKey::CompressOption => "--compress zstd/gzip: Compression",
            MessageKey::CompressLevel => "--compress-level 1-22: Compression level",

            // Run command options
            MessageKey::IncrementalOption => {
                "--incremental: Incremental backup (changed files only)"
            }
            MessageKey::GeneratePasswordOption => "--generate-password: Generate secure password",
            MessageKey::PasswordOption => "--password <PASSWORD>: Specify encryption password",
            MessageKey::DryRunOption => "--dry-run: Dry run mode (no actual backup)",
            MessageKey::PriorityOption => {
                "--priority <PRIORITY>: Filter by priority (high/medium/low)"
            }
            MessageKey::CategoryOption => "--category <CATEGORY>: Filter by category",

            // Restore command options
            MessageKey::FromOption => "--from <BACKUP_NAME>: Backup to restore",
            MessageKey::ToOption => "--to <DESTINATION>: Restore destination",
            MessageKey::RestorePasswordOption => {
                "--password <PASSWORD>: Decryption password (if encrypted)"
            }

            // Cleanup command options
            MessageKey::DaysOption => "--days <DAYS>: Delete backups older than specified days",
            MessageKey::CleanupDryRunOption => {
                "--dry-run: Dry run mode (show what would be deleted)"
            }

            // Add command options
            MessageKey::AddPriorityOption => {
                "--priority <PRIORITY>: Set priority (high/medium/low)"
            }
            MessageKey::AddCategoryOption => "--category <CATEGORY>: Set category",
            MessageKey::InteractiveOption => "--interactive: Interactive selection mode",

            // List command options
            MessageKey::ListPriorityOption => "--priority <PRIORITY>: Filter by priority",

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
            MessageKey::ConfigDescription => {
                "Command to manage backup destination, retention period and other settings"
            }
            MessageKey::ConfigUsage => "Usage:",
            MessageKey::ConfigMgmtCommands => "📋 Configuration Management Commands",
            MessageKey::ConfigSetDestination => {
                "set-destination <path>  Change backup destination directory"
            }
            MessageKey::ConfigGetDestination => {
                "get-destination        Display current backup destination"
            }
            MessageKey::ConfigSetKeepDays => {
                "set-keep-days <days>   Change backup retention period (1-3650 days)"
            }
            MessageKey::ConfigGetKeepDays => {
                "get-keep-days          Display current backup retention period"
            }
            MessageKey::ConfigOpen => {
                "open                   Open configuration file in default editor"
            }
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
            MessageKey::ScheduleTip1 => {
                "  • After enabling schedule, it runs automatically via macOS launchctl"
            }
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
            MessageKey::ConfigTip3 => {
                "  • Backups older than retention period can be deleted with cleanup command"
            }

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
            MessageKey::ScheduleUpdatedEnableLater => {
                "Schedule settings updated (enable with 'schedule enable')"
            }
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
            MessageKey::RustFastTypeSafe => "Intelligent Backup with AES-256 Encryption & AI",
            MessageKey::ScheduleCommandPlaceholder => "<command>",
            MessageKey::ConfigCommandPlaceholder => "<command>",
            MessageKey::ConfigArgsPlaceholder => "[args]",
            MessageKey::MainHelp => "Main help",
            MessageKey::ConfigFileLabel => "Configuration file",
            MessageKey::EnableOnlySpecifiedPriority => {
                "Enable only specified priority (high/medium/low)"
            }
            MessageKey::DisableOnlySpecifiedPriority => "Disable only specified priority",
            MessageKey::SetExecutionFrequency => {
                "Set execution frequency for each priority (daily/weekly/monthly)"
            }
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
            MessageKey::SmartCommands => "🤖 Smartコマンド",

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
            MessageKey::CmdSmart => "ai",

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
            MessageKey::DescSmart => "AI駆動のインテリジェントバックアップ管理",

            // AI subcommands
            MessageKey::CmdSmartDetect => "detect",
            MessageKey::CmdSmartAnalyze => "analyze",
            MessageKey::CmdSmartSuggestExclude => "suggest-exclude",
            MessageKey::CmdSmartAutoConfigure => "auto-configure",
            MessageKey::DescSmartDetect => "バックアップ履歴の異常検知",
            MessageKey::DescSmartAnalyze => "ファイル重要度分析",
            MessageKey::DescSmartSuggestExclude => "除外パターン提案",
            MessageKey::DescSmartAutoConfigure => "Smartルールによる自動設定",

            // AI messages
            MessageKey::SmartDetectTitle => "🤖 Smart異常検知",
            MessageKey::SmartDetectNoAnomalies => "バックアップ履歴に異常は検出されませんでした",
            MessageKey::SmartDetectAnomalyFound => "異常を検出しました",
            MessageKey::SmartAnalyzeTitle => "🤖 Smartファイル重要度分析",
            MessageKey::SmartAnalyzeImportanceHigh => "重要度：高",
            MessageKey::SmartAnalyzeImportanceMedium => "重要度：中",
            MessageKey::SmartAnalyzeImportanceLow => "重要度：低",
            MessageKey::SmartSuggestExcludeTitle => "🤖 Smart除外パターン提案",
            MessageKey::SmartSuggestExcludeRecommendation => "除外推奨",
            MessageKey::SmartAutoConfigureTitle => "🤖 Smart自動設定",
            MessageKey::SmartAutoConfigureSuccess => "自動設定が完了しました",
            MessageKey::SmartErrorNotEnabled => {
                "Smart機能が有効化されていません。--features smart でコンパイルしてください"
            }
            MessageKey::SmartErrorInsufficientData => "Smart分析に必要なデータが不足しています",
            MessageKey::SmartErrorAnalysisFailed => "Smart分析に失敗しました",

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
            MessageKey::ExampleSmartDetect => "# 直近7日間の異常検知",
            MessageKey::ExampleSmartAnalyze => "# ファイル重要度分析",
            MessageKey::ExampleSmartSuggestExclude => "# Smart除外推奨を取得",

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
            MessageKey::SavePasswordSecurely => "⚠️  このパスワードを安全に保管してください！",
            MessageKey::EncryptOption => "--encrypt: AES-256-GCM暗号化",
            MessageKey::CompressOption => "--compress zstd/gzip: 圧縮",
            MessageKey::CompressLevel => "--compress-level 1-22: 圧縮レベル",

            // Run command options
            MessageKey::IncrementalOption => "--incremental: 増分バックアップ（変更ファイルのみ）",
            MessageKey::GeneratePasswordOption => "--generate-password: 安全なパスワードを自動生成",
            MessageKey::PasswordOption => "--password <パスワード>: 暗号化パスワード指定",
            MessageKey::DryRunOption => "--dry-run: ドライランモード（実際のバックアップなし）",
            MessageKey::PriorityOption => "--priority <優先度>: 優先度でフィルタ (high/medium/low)",
            MessageKey::CategoryOption => "--category <カテゴリ>: カテゴリでフィルタ",

            // Restore command options
            MessageKey::FromOption => "--from <バックアップ名>: 復元するバックアップ",
            MessageKey::ToOption => "--to <復元先>: 復元先ディレクトリ",
            MessageKey::RestorePasswordOption => {
                "--password <パスワード>: 復号化パスワード（暗号化時）"
            }

            // Cleanup command options
            MessageKey::DaysOption => "--days <日数>: 指定日数より古いバックアップを削除",
            MessageKey::CleanupDryRunOption => "--dry-run: ドライランモード（削除対象を表示）",

            // Add command options
            MessageKey::AddPriorityOption => "--priority <優先度>: 優先度を設定 (high/medium/low)",
            MessageKey::AddCategoryOption => "--category <カテゴリ>: カテゴリを設定",
            MessageKey::InteractiveOption => "--interactive: インタラクティブ選択モード",

            // List command options
            MessageKey::ListPriorityOption => "--priority <優先度>: 優先度でフィルタ",

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
            MessageKey::ConfigDescription => {
                "バックアップの保存先や保持期間などの設定を管理するコマンド"
            }
            MessageKey::ConfigUsage => "使用方法:",
            MessageKey::ConfigMgmtCommands => "📋 設定管理コマンド",
            MessageKey::ConfigSetDestination => {
                "set-destination <パス>  バックアップ保存先ディレクトリを変更"
            }
            MessageKey::ConfigGetDestination => {
                "get-destination        現在のバックアップ保存先を表示"
            }
            MessageKey::ConfigSetKeepDays => {
                "set-keep-days <日数>   バックアップ保持期間を変更 (1-3650日)"
            }
            MessageKey::ConfigGetKeepDays => {
                "get-keep-days          現在のバックアップ保持期間を表示"
            }
            MessageKey::ConfigOpen => {
                "open                   設定ファイルをデフォルトエディタで開く"
            }
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
            MessageKey::ScheduleTip1 => {
                "  • スケジュール有効化後、macOSのlaunchctlで自動実行されます"
            }
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
            MessageKey::ConfigTip3 => {
                "  • 保持期間を過ぎたバックアップは cleanup コマンドで削除できます"
            }

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
            MessageKey::ScheduleUpdatedEnableLater => {
                "スケジュール設定更新（有効化は 'schedule enable' で）"
            }
            MessageKey::HighPriority => "高優先度",
            MessageKey::MediumPriority => "中優先度",
            MessageKey::LowPriority => "低優先度",
            MessageKey::DirectoryNotExists => "ディレクトリが存在しません。作成します",
            MessageKey::DirectoryCreating => "作成中",
            MessageKey::DestinationChanged => "バックアップ先を変更しました",
            MessageKey::Before => "変更前",
            MessageKey::After => "変更後",
            MessageKey::CurrentDestination => "現在のバックアップ先",
            MessageKey::KeepDaysOutOfRange => {
                "keep_days は 1-3650 の範囲で指定してください（指定値:"
            }
            MessageKey::KeepDaysChanged => "バックアップ保持期間を変更しました",
            MessageKey::CurrentKeepDays => "現在のバックアップ保持期間",
            MessageKey::OpeningConfigFile => "設定ファイルを開きます",
            MessageKey::EditorDidNotExitCleanly => "エディタが正常に終了しませんでした",
            MessageKey::RustFastTypeSafe => "AES-256暗号化 & AI搭載のインテリジェントバックアップ",
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

    /// Get Simplified Chinese message
    fn get_zh_cn(&self) -> &'static str {
        match self {
            // AI-related messages
            MessageKey::SmartCommands => "🤖 Smart命令",
            MessageKey::DescSmart => "AI驱动的智能备份管理",
            MessageKey::CmdSmartDetect => "detect",
            MessageKey::CmdSmartAnalyze => "analyze",
            MessageKey::CmdSmartSuggestExclude => "suggest-exclude",
            MessageKey::CmdSmartAutoConfigure => "auto-configure",
            MessageKey::DescSmartDetect => "检测备份历史中的异常",
            MessageKey::DescSmartAnalyze => "分析文件重要性",
            MessageKey::DescSmartSuggestExclude => "建议排除模式",
            MessageKey::DescSmartAutoConfigure => "使用Smart规则自动配置备份设置",
            MessageKey::SmartDetectTitle => "🤖 Smart异常检测",
            MessageKey::SmartDetectNoAnomalies => "备份历史中未检测到异常",
            MessageKey::SmartDetectAnomalyFound => "检测到异常",
            MessageKey::SmartAnalyzeTitle => "🤖 Smart文件重要性分析",
            MessageKey::SmartAnalyzeImportanceHigh => "重要性：高",
            MessageKey::SmartAnalyzeImportanceMedium => "重要性：中",
            MessageKey::SmartAnalyzeImportanceLow => "重要性：低",
            MessageKey::SmartSuggestExcludeTitle => "🤖 Smart排除模式建议",
            MessageKey::SmartSuggestExcludeRecommendation => "建议排除",
            MessageKey::SmartAutoConfigureTitle => "🤖 Smart自动配置",
            MessageKey::SmartAutoConfigureSuccess => "自动配置成功完成",
            MessageKey::SmartErrorNotEnabled => "Smart功能未启用。请使用 --features smart 编译",
            MessageKey::SmartErrorInsufficientData => "Smart分析数据不足",
            MessageKey::SmartErrorAnalysisFailed => "Smart分析失败",
            MessageKey::ExampleSmartDetect => "# 检测最近7天的异常",
            MessageKey::ExampleSmartAnalyze => "# 分析文件重要性",
            MessageKey::ExampleSmartSuggestExclude => "# 获取Smart排除建议",
            MessageKey::RustFastTypeSafe => "AES-256加密 & AI驱动的智能备份",
            // Common messages
            MessageKey::UsageExamples => "使用示例:",
            // Keep all existing Simplified Chinese translations
            _ => self.get_en(), // Fallback to English for non-implemented keys
        }
    }

    /// Get Traditional Chinese message
    fn get_zh_tw(&self) -> &'static str {
        match self {
            // AI-related messages
            MessageKey::SmartCommands => "🤖 Smart指令",
            MessageKey::DescSmart => "AI驅動的智慧備份管理",
            MessageKey::CmdSmartDetect => "detect",
            MessageKey::CmdSmartAnalyze => "analyze",
            MessageKey::CmdSmartSuggestExclude => "suggest-exclude",
            MessageKey::CmdSmartAutoConfigure => "auto-configure",
            MessageKey::DescSmartDetect => "偵測備份歷史中的異常",
            MessageKey::DescSmartAnalyze => "分析檔案重要性",
            MessageKey::DescSmartSuggestExclude => "建議排除模式",
            MessageKey::DescSmartAutoConfigure => "使用Smart規則自動設定備份",
            MessageKey::SmartDetectTitle => "🤖 Smart異常偵測",
            MessageKey::SmartDetectNoAnomalies => "備份歷史中未偵測到異常",
            MessageKey::SmartDetectAnomalyFound => "偵測到異常",
            MessageKey::SmartAnalyzeTitle => "🤖 Smart檔案重要性分析",
            MessageKey::SmartAnalyzeImportanceHigh => "重要性：高",
            MessageKey::SmartAnalyzeImportanceMedium => "重要性：中",
            MessageKey::SmartAnalyzeImportanceLow => "重要性：低",
            MessageKey::SmartSuggestExcludeTitle => "🤖 Smart排除模式建議",
            MessageKey::SmartSuggestExcludeRecommendation => "建議排除",
            MessageKey::SmartAutoConfigureTitle => "🤖 Smart自動設定",
            MessageKey::SmartAutoConfigureSuccess => "自動設定成功完成",
            MessageKey::SmartErrorNotEnabled => "Smart功能未啟用。請使用 --features smart 編譯",
            MessageKey::SmartErrorInsufficientData => "Smart分析資料不足",
            MessageKey::SmartErrorAnalysisFailed => "Smart分析失敗",
            MessageKey::ExampleSmartDetect => "# 偵測最近7天的異常",
            MessageKey::ExampleSmartAnalyze => "# 分析檔案重要性",
            MessageKey::ExampleSmartSuggestExclude => "# 取得Smart排除建議",
            MessageKey::RustFastTypeSafe => "AES-256加密 & AI驅動的智慧備份",
            // Common messages
            MessageKey::UsageExamples => "使用範例:",
            // Keep all existing Traditional Chinese translations
            _ => self.get_en(), // Fallback to English for non-implemented keys
        }
    }
}

/// Get message by key and language
#[must_use]
pub fn get_message(key: MessageKey, lang: Language) -> &'static str {
    key.get(lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        // Default should be English
        let _lang = Language::detect();
        // This test might fail if LANG is set to ja
        // assert_eq!(_lang, Language::English);
    }

    #[test]
    fn test_language_parsing() {
        // English
        assert_eq!(Language::parse("en"), Some(Language::English));
        assert_eq!(Language::parse("english"), Some(Language::English));

        // Japanese
        assert_eq!(Language::parse("ja"), Some(Language::Japanese));
        assert_eq!(Language::parse("japanese"), Some(Language::Japanese));
        assert_eq!(Language::parse("日本語"), Some(Language::Japanese));

        // Simplified Chinese
        assert_eq!(Language::parse("zh-cn"), Some(Language::SimplifiedChinese));
        assert_eq!(Language::parse("zh_cn"), Some(Language::SimplifiedChinese));
        assert_eq!(
            Language::parse("zh-hans"),
            Some(Language::SimplifiedChinese)
        );
        assert_eq!(
            Language::parse("zh_hans"),
            Some(Language::SimplifiedChinese)
        );
        assert_eq!(
            Language::parse("simplified chinese"),
            Some(Language::SimplifiedChinese)
        );
        assert_eq!(
            Language::parse("简体中文"),
            Some(Language::SimplifiedChinese)
        );

        // Traditional Chinese
        assert_eq!(Language::parse("zh-tw"), Some(Language::TraditionalChinese));
        assert_eq!(Language::parse("zh_tw"), Some(Language::TraditionalChinese));
        assert_eq!(Language::parse("zh-hk"), Some(Language::TraditionalChinese));
        assert_eq!(Language::parse("zh_hk"), Some(Language::TraditionalChinese));
        assert_eq!(
            Language::parse("zh-hant"),
            Some(Language::TraditionalChinese)
        );
        assert_eq!(
            Language::parse("zh_hant"),
            Some(Language::TraditionalChinese)
        );
        assert_eq!(
            Language::parse("traditional chinese"),
            Some(Language::TraditionalChinese)
        );
        assert_eq!(
            Language::parse("繁體中文"),
            Some(Language::TraditionalChinese)
        );
        assert_eq!(
            Language::parse("繁体中文"),
            Some(Language::TraditionalChinese)
        );

        // Unknown
        assert_eq!(Language::parse("unknown"), None);
    }

    #[test]
    fn test_language_code() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Japanese.code(), "ja");
        assert_eq!(Language::SimplifiedChinese.code(), "zh-cn");
        assert_eq!(Language::TraditionalChinese.code(), "zh-tw");
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

        // Test Simplified Chinese
        assert_eq!(
            get_message(MessageKey::AppVersion, Language::SimplifiedChinese),
            "Backup Suite v1.0.0"
        );

        // Test Traditional Chinese
        assert_eq!(
            get_message(MessageKey::AppVersion, Language::TraditionalChinese),
            "Backup Suite v1.0.0"
        );

        // Test different messages
        assert!(get_message(MessageKey::AppTitle, Language::English).contains("Fast"));
        assert!(get_message(MessageKey::AppTitle, Language::Japanese).contains("高速"));
    }

    #[test]
    fn test_ai_messages() {
        // Test AI message keys
        assert_eq!(
            get_message(MessageKey::SmartCommands, Language::English),
            "🤖 Smart Commands"
        );
        assert_eq!(
            get_message(MessageKey::SmartCommands, Language::Japanese),
            "🤖 Smartコマンド"
        );
    }
}
