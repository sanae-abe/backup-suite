//! # Internationalization (i18n) Module
//!
//! Provides multi-language support for the Backup Suite CLI.
//! Default language: English
//! Supported languages: English, Japanese, Simplified Chinese, Traditional Chinese

use std::sync::OnceLock;

/// Application version string (generated once at runtime)
fn app_version() -> &'static str {
    static VERSION_STRING: OnceLock<String> = OnceLock::new();
    VERSION_STRING.get_or_init(|| format!("Backup Suite v{}", env!("CARGO_PKG_VERSION")))
}

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
    ///
    /// # Security
    /// - Rejects null bytes (security vulnerability)
    /// - Rejects command injection patterns (semicolons, pipes, etc.)
    /// - Rejects path traversal attempts (../, ..\)
    /// - Rejects excessively long inputs (> 100 chars)
    /// - Whitelist-based validation (only known language codes accepted)
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        // Security: Reject null bytes
        if s.contains('\0') {
            return None;
        }

        // Security: Reject command injection patterns
        if s.contains(';')
            || s.contains('|')
            || s.contains('&')
            || s.contains('`')
            || s.contains('$')
            || s.contains('(')
            || s.contains(')')
        {
            return None;
        }

        // Security: Reject path traversal attempts
        if s.contains("..") || s.contains('/') || s.contains('\\') {
            return None;
        }

        // Security: Reject excessively long inputs (DoS prevention)
        if s.len() > 100 {
            return None;
        }

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
    ConfirmClearAll,
    ConfirmClearPriority,
    ConfirmCleanup,
    DaysOutOfRange,
    PromptSelectTarget,
    PromptSelectFile,
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

    // Backup progress and result messages
    FilesDetected,
    FullBackupMode,
    IncrementalBackupMode,
    BackupComplete,
    BackupCompleteWithFailures,
    BackupResultTitle,
    TotalFilesLabel,
    SuccessfulLabel,
    FailedLabel,
    TotalSizeLabel,

    // Remove/Update command messages
    ConfirmRemoveTarget,
    UpdatedTarget,
    PathLabel,
    PriorityLabel,
    CategoryLabel,
    ExcludePatternsLabel,

    // Smart Analyze labels
    ItemLabel,
    ValueLabel,
    ImportanceScoreLabel,
    RecommendedPriorityLabel,
    ReasonLabel,

    // Smart Auto-Configure labels
    AnalyzingLabel,
    AddedToConfiguration,
    ItemsAdded,
    ExistingBackupTargets,
    AddNewTargets,

    // History detailed view
    TimestampLabel,
    PathHistoryLabel,
    StatusHistoryLabel,
    FilesHistoryLabel,
    SizeLabel,
    CompressionLabel,
    EncryptionLabel,
    DurationLabel,
    EnabledLabel,
    SecondsUnit,

    // Schedule table headers
    ScheduleHeaderLabel,

    // Dashboard sections
    StatisticsTitle,
    DiskUsageTitle,
    AllNormalStatus,
    WarningsTitle,

    // Dashboard statistics labels
    TotalTargetsLabel,
    HighPriorityTargetsLabel,
    MediumPriorityTargetsLabel,
    LowPriorityTargetsLabel,
    TotalBackupsLabel,
    SuccessCountLabel,
    TotalFilesCountLabel,
    TotalDataSizeLabel,
    LastBackupLabel,
    EncryptedBackupsLabel,
    CompressedBackupsLabel,
    BackupDirectoryLabel,
    UsedCapacityLabel,
    FileCountLabel,
    DiskTotalCapacityLabel,
    DiskFreeCapacityLabel,
    DiskUsageRateLabel,
    UsageStatusLabel,
    RecentBackupsTitle,

    // Incremental backup messages
    PreviousBackupLabel,
    ChangedFilesLabel,
    NoBackupsFound,
    FullBackupFallback,
    MetadataLoadFailed,
    DryRunMode,

    // Relative time messages
    DaysAgo,
    HoursAgo,
    MinutesAgo,
    JustNow,
    NotYetBackedUp,

    // Dashboard warning messages
    WarningTargetNotExists,
    WarningDaysSinceLastBackup,
    WarningNoBackupYet,
    WarningFailedBackups,
    WarningLowDiskSpace,
    DashboardHintRunBackup,

    // Interactive prompts
    PromptPleaseSelect,
    PromptDeleteBackup,
    PromptDeleteOldBackups,
    PromptDeleteTarget,
    PromptDeleteCount,
    PromptConfirmDelete,
    PromptSelectPriority,
    PromptBackupConfirm,

    // Smart Analyze categories
    SmartCategoryDirectory,
    SmartCategoryRustProject,
    SmartCategoryNodeJsProject,
    SmartCategoryPythonProject,
    SmartCategorySourceCodeProject,
    SmartCategoryGitManaged,
    SmartCategoryLowPriority,

    // Smart Analyze reasons
    SmartReasonSampling,
    SmartReasonScore,
    SmartReasonSecurityDir,
    SmartReasonLowPriorityDir,

    // Smart Exclude reasons
    ExcludeReasonNpmDeps,
    ExcludeReasonRustBuild,
    ExcludeReasonVendor,
    ExcludeReasonPythonCache,
    ExcludeReasonPytestCache,
    ExcludeReasonBuildArtifacts,
    ExcludeReasonCacheDir,
    ExcludeReasonGitMetadata,
    ExcludeReasonSvnMetadata,
    ExcludeReasonTempFile,
    ExcludeReasonBackupFile,
    ExcludeReasonEditorTemp,
    ExcludeReasonLogFile,
    ExcludeReasonMacOsMetadata,
    ExcludeReasonWindowsThumb,
    ExcludeReasonWindowsDesktop,

    // Password strength messages
    PasswordStrengthLabel,
    PasswordStrengthWeak,
    PasswordStrengthMedium,
    PasswordStrengthStrong,
    PasswordStrengthWeakMessage,
    PasswordStrengthMediumMessage,
    PasswordStrengthStrongMessage,
    PasswordStrengthTip,

    // Editor and config
    EditorLaunchFailed,

    // Smart feature progress
    SubdirectoriesFound,
    ProgressEvaluating,

    // Backup confirmation prompts
    ConfirmBackupTitle,
    ConfirmBackupTargetFiles,
    ConfirmBackupDestination,

    // Cleanup confirmation prompts
    ConfirmCleanupTitle,
    ConfirmCleanupTargetCount,
    ConfirmCleanupRetentionDays,

    // Cleanup progress messages
    CleanupDryRunScheduled,
    CleanupCompleted,
    CleanupFailed,

    // Restore progress messages
    RestoreDryRunDetected,
    RestoreInProgress,
    RestoreProgressFile,
    RestoreIntegrityMetadataLoaded,
    RestoreCompleted,
    RestoreCompletedWithFailures,

    // Restore error messages
    ErrorRelativePathFailed,
    ErrorPathTraversalDetected,
    ErrorDirectoryCreateFailed,
    ErrorFileReadFailed,
    ErrorFileOpenFailedSymlink,
    ErrorEncryptedButNoPassword,
    ErrorMasterKeyRestoreFailed,
    ErrorDecryptionFailed,
    ErrorIntegrityVerificationFailed,
    ErrorFileWriteFailed,
    ErrorFileCountFailed,

    // Backup progress and error messages
    BackupProgressProcessing,
    ErrorBackupDirectoryCreateFailed,
    ErrorBackupWriteFailed,
    ErrorBackupProcessFailed,
    ErrorBackupCopyFailed,
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
            MessageKey::AppVersion => app_version(),
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
            MessageKey::ConfirmClearAll => {
                "⚠️  Warning: Delete all {} backup targets. Are you sure?"
            }
            MessageKey::ConfirmClearPriority => {
                "⚠️  Warning: Delete {} backup targets with {} priority. Are you sure?"
            }
            MessageKey::ConfirmCleanup => "Delete backups older than {} days. Are you sure?",
            MessageKey::DaysOutOfRange => "days must be in the range 1-3650 (specified: {})",
            MessageKey::PromptSelectTarget => "Select backup target to remove",
            MessageKey::PromptSelectFile => "Select file/directory to add: ",
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
            MessageKey::RustFastTypeSafe => {
                "Intelligent Backup with AES-256 Encryption & Smart Analysis"
            }
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

            // Backup progress and result messages
            MessageKey::FilesDetected => "files detected",
            MessageKey::FullBackupMode => "Full Backup Mode (all files)",
            MessageKey::IncrementalBackupMode => "Incremental Backup Mode (changed files only)",
            MessageKey::BackupComplete => "Backup complete",
            MessageKey::BackupCompleteWithFailures => "Backup complete (with failures)",
            MessageKey::BackupResultTitle => "Backup Result",
            MessageKey::TotalFilesLabel => "Total Files",
            MessageKey::SuccessfulLabel => "Successful",
            MessageKey::FailedLabel => "Failed",
            MessageKey::TotalSizeLabel => "Total Size",

            // Remove/Update command messages
            MessageKey::ConfirmRemoveTarget => {
                "Are you sure you want to remove {} from backup targets?"
            }
            MessageKey::UpdatedTarget => "Updated backup target",
            MessageKey::PathLabel => "Path",
            MessageKey::PriorityLabel => "Priority",
            MessageKey::CategoryLabel => "Category",
            MessageKey::ExcludePatternsLabel => "Exclude Patterns",

            // Smart Analyze labels
            MessageKey::ItemLabel => "Item",
            MessageKey::ValueLabel => "Value",
            MessageKey::ImportanceScoreLabel => "Importance Score",
            MessageKey::RecommendedPriorityLabel => "Recommended Priority",
            MessageKey::ReasonLabel => "Reason",

            // Smart Auto-Configure labels
            MessageKey::AnalyzingLabel => "Analyzing",
            MessageKey::AddedToConfiguration => "Added to configuration",
            MessageKey::ItemsAdded => "Items added",
            MessageKey::ExistingBackupTargets => "You have {} existing backup targets",
            MessageKey::AddNewTargets => "Add new targets?",

            // History detailed view
            MessageKey::TimestampLabel => "Timestamp",
            MessageKey::PathHistoryLabel => "Path",
            MessageKey::StatusHistoryLabel => "Status",
            MessageKey::FilesHistoryLabel => "Files",
            MessageKey::SizeLabel => "Size",
            MessageKey::CompressionLabel => "Compression",
            MessageKey::EncryptionLabel => "Encryption",
            MessageKey::DurationLabel => "Duration",
            MessageKey::EnabledLabel => "Enabled",
            MessageKey::SecondsUnit => "seconds",

            // Dashboard sections
            MessageKey::StatisticsTitle => "📈 Statistics",
            MessageKey::DiskUsageTitle => "💾 Disk Usage",
            MessageKey::AllNormalStatus => "⚡ All Normal",
            MessageKey::WarningsTitle => "⚠️  Warnings",

            // Incremental backup messages
            MessageKey::PreviousBackupLabel => "Previous backup",
            MessageKey::ChangedFilesLabel => "Changed files",
            MessageKey::NoBackupsFound => "ℹ️  No previous backup found. Performing full backup.",
            MessageKey::FullBackupFallback => {
                "⚠️  Failed to load previous metadata. Falling back to full backup."
            }
            MessageKey::MetadataLoadFailed => "   Details",
            MessageKey::DryRunMode => "📋 Dry run mode: detected {} files for backup",

            // Dashboard statistics labels
            MessageKey::TotalTargetsLabel => "Total Targets",
            MessageKey::HighPriorityTargetsLabel => "  High Priority",
            MessageKey::MediumPriorityTargetsLabel => "  Medium Priority",
            MessageKey::LowPriorityTargetsLabel => "  Low Priority",
            MessageKey::TotalBackupsLabel => "Total Backups",
            MessageKey::SuccessCountLabel => "  Success",
            MessageKey::TotalFilesCountLabel => "Total Files",
            MessageKey::TotalDataSizeLabel => "Total Data Size",
            MessageKey::LastBackupLabel => "Last Backup",
            MessageKey::EncryptedBackupsLabel => "Encrypted Backups",
            MessageKey::CompressedBackupsLabel => "Compressed Backups",
            MessageKey::BackupDirectoryLabel => "Backup Directory",
            MessageKey::UsedCapacityLabel => "Used Capacity",
            MessageKey::FileCountLabel => "File Count",
            MessageKey::DiskTotalCapacityLabel => "Disk Total Capacity",
            MessageKey::DiskFreeCapacityLabel => "Disk Free Capacity",
            MessageKey::DiskUsageRateLabel => "Disk Usage Rate",
            MessageKey::UsageStatusLabel => "Usage Status",
            MessageKey::RecentBackupsTitle => "🕒 Recent Backups (Latest 5)",

            // Schedule table headers
            MessageKey::ScheduleHeaderLabel => "Schedule",

            // Relative time messages
            MessageKey::DaysAgo => "{} days ago",
            MessageKey::HoursAgo => "{} hours ago",
            MessageKey::MinutesAgo => "{} minutes ago",
            MessageKey::JustNow => "Just now",
            MessageKey::NotYetBackedUp => "Not yet",

            // Dashboard warning messages
            MessageKey::WarningTargetNotExists => "Backup target does not exist: {}",
            MessageKey::WarningDaysSinceLastBackup => "It has been {} days since the last backup",
            MessageKey::WarningNoBackupYet => "No backup has been performed yet",
            MessageKey::WarningFailedBackups => "There are {} failed backups",
            MessageKey::WarningLowDiskSpace => "Disk space is running low ({:.1}%)",
            MessageKey::DashboardHintRunBackup => {
                "💡 Hint: Run 'backup-suite run' to perform a backup"
            }

            // Interactive prompts
            MessageKey::PromptPleaseSelect => "Please select",
            MessageKey::PromptDeleteBackup => "Do you want to delete this backup?",
            MessageKey::PromptDeleteOldBackups => "🗑️  Delete old backups",
            MessageKey::PromptDeleteTarget => "Targets to delete: {} backups",
            MessageKey::PromptDeleteCount => "targets",
            MessageKey::PromptConfirmDelete => "Do you want to proceed with deletion?",
            MessageKey::PromptSelectPriority => "Select priority",
            MessageKey::PromptBackupConfirm => "Do you want to perform a backup?",

            // Smart Analyze categories
            MessageKey::SmartCategoryDirectory => "Directory",
            MessageKey::SmartCategoryRustProject => "Rust Project",
            MessageKey::SmartCategoryNodeJsProject => "Node.js Project",
            MessageKey::SmartCategoryPythonProject => "Python Project",
            MessageKey::SmartCategorySourceCodeProject => "Source Code Project",
            MessageKey::SmartCategoryGitManaged => "Git-managed Directory",
            MessageKey::SmartCategoryLowPriority => "Low Priority Directory",

            // Smart Analyze reasons
            MessageKey::SmartReasonSampling => {
                "(Sampling: {} files, high importance: {}, score: {})"
            }
            MessageKey::SmartReasonScore => "(Score: {})",
            MessageKey::SmartReasonSecurityDir => {
                "Credentials/Secret keys ({} directory, encryption required, score: 95)"
            }
            MessageKey::SmartReasonLowPriorityDir => {
                "Cache/Log/Archive etc. (Directory: {}, score: 20)"
            }

            // Smart Exclude reasons
            MessageKey::ExcludeReasonNpmDeps => {
                "npm/yarn dependencies (regenerable from package.json)"
            }
            MessageKey::ExcludeReasonRustBuild => {
                "Rust build artifacts (regenerable from Cargo.toml)"
            }
            MessageKey::ExcludeReasonVendor => "Dependency vendoring (regenerable)",
            MessageKey::ExcludeReasonPythonCache => "Python cache (auto-generated)",
            MessageKey::ExcludeReasonPytestCache => "pytest cache (auto-generated)",
            MessageKey::ExcludeReasonBuildArtifacts => "Build artifacts directory (rebuildable)",
            MessageKey::ExcludeReasonCacheDir => "Cache directory (temporary data)",
            MessageKey::ExcludeReasonGitMetadata => {
                "Git repository metadata (recoverable from remote)"
            }
            MessageKey::ExcludeReasonSvnMetadata => {
                "SVN repository metadata (recoverable from remote)"
            }
            MessageKey::ExcludeReasonTempFile => "Temporary file",
            MessageKey::ExcludeReasonBackupFile => {
                "Backup file (unnecessary if original file exists)"
            }
            MessageKey::ExcludeReasonEditorTemp => "Editor temporary file",
            MessageKey::ExcludeReasonLogFile => "Log file (old logs usually unnecessary)",
            MessageKey::ExcludeReasonMacOsMetadata => "macOS metadata file (auto-generated)",
            MessageKey::ExcludeReasonWindowsThumb => "Windows thumbnail cache (auto-generated)",
            MessageKey::ExcludeReasonWindowsDesktop => {
                "Windows desktop settings file (auto-generated)"
            }

            // Password strength messages
            MessageKey::PasswordStrengthLabel => "Password Strength:",
            MessageKey::PasswordStrengthWeak => "Weak",
            MessageKey::PasswordStrengthMedium => "Medium",
            MessageKey::PasswordStrengthStrong => "Strong",
            MessageKey::PasswordStrengthWeakMessage => {
                "This password may be vulnerable to attacks. Consider using a longer password with varied characters."
            }
            MessageKey::PasswordStrengthMediumMessage => {
                "This password provides moderate security. Adding special characters or length would improve it."
            }
            MessageKey::PasswordStrengthStrongMessage => {
                "This password provides strong security."
            }
            MessageKey::PasswordStrengthTip => {
                "Tip: Use --generate-password to create a strong random password."
            }

            // Editor and config
            MessageKey::EditorLaunchFailed => "Failed to launch editor: {}",

            // Smart feature progress
            MessageKey::SubdirectoriesFound => "Found {} subdirectories",
            MessageKey::ProgressEvaluating => "Progress - Evaluating: {:?}",

            // Backup confirmation prompts
            MessageKey::ConfirmBackupTitle => "📦 Backup Execution Confirmation",
            MessageKey::ConfirmBackupTargetFiles => "Target files: {} files",
            MessageKey::ConfirmBackupDestination => "Backup destination: {}",

            // Cleanup confirmation prompts
            MessageKey::ConfirmCleanupTitle => "🗑️  Delete Old Backups",
            MessageKey::ConfirmCleanupTargetCount => "Deletion targets: {} backups",
            MessageKey::ConfirmCleanupRetentionDays => "Retention period: {} days",

            // Cleanup progress messages
            MessageKey::CleanupDryRunScheduled => "🗑️  [Dry Run] Scheduled for deletion: {:?}",
            MessageKey::CleanupCompleted => "🗑️  Deletion completed: {:?}",
            MessageKey::CleanupFailed => "Deletion failed {:?}: {}",

            // Restore progress messages
            MessageKey::RestoreDryRunDetected => "📋 Dry run mode: {} files detected for restore",
            MessageKey::RestoreInProgress => "Restoring...",
            MessageKey::RestoreProgressFile => "Restoring: {:?}",
            MessageKey::RestoreIntegrityMetadataLoaded => "✓ Integrity metadata loaded ({} backups)",
            MessageKey::RestoreCompleted => "✓ Restore completed",
            MessageKey::RestoreCompletedWithFailures => "⚠ Restore completed ({} failed)",

            // Restore error messages
            MessageKey::ErrorRelativePathFailed => "Failed to get relative path {}: {}",
            MessageKey::ErrorPathTraversalDetected => "Path traversal detected {}: {}",
            MessageKey::ErrorDirectoryCreateFailed => "Failed to create directory {}: {}",
            MessageKey::ErrorFileReadFailed => "Failed to read file: {}",
            MessageKey::ErrorFileOpenFailedSymlink => "Failed to open file (possible symlink attack): {}",
            MessageKey::ErrorEncryptedButNoPassword => "Encrypted file but no password specified: {}",
            MessageKey::ErrorMasterKeyRestoreFailed => "Failed to restore master key: {}",
            MessageKey::ErrorDecryptionFailed => "Decryption failed {}: {}",
            MessageKey::ErrorIntegrityVerificationFailed => "⚠ Integrity verification failed (file tampered): {}",
            MessageKey::ErrorFileWriteFailed => "Failed to write file {}: {}",
            MessageKey::ErrorFileCountFailed => "Errors occurred in {} files",

            // Backup progress and error messages
            MessageKey::BackupProgressProcessing => "Processing: {:?}",
            MessageKey::ErrorBackupDirectoryCreateFailed => "Failed to create directory {}: {}",
            MessageKey::ErrorBackupWriteFailed => "Write failed {}: {}",
            MessageKey::ErrorBackupProcessFailed => "Processing failed {}: {}",
            MessageKey::ErrorBackupCopyFailed => "Copy failed {}: {}",
        }
    }

    /// Get Japanese message
    fn get_ja(&self) -> &'static str {
        match self {
            // Version and title
            MessageKey::AppVersion => app_version(),
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
            MessageKey::ConfirmClearAll => "⚠️  警告: {}個すべてのバックアップ対象を削除します。本当によろしいですか？",
            MessageKey::ConfirmClearPriority => "⚠️  警告: {}優先度のバックアップ対象{}個を削除します。本当によろしいですか？",
            MessageKey::ConfirmCleanup => "{}日以前の古いバックアップを削除します。よろしいですか？",
            MessageKey::DaysOutOfRange => "days は 1-3650 の範囲で指定してください（指定値: {}）",
            MessageKey::PromptSelectTarget => "削除するバックアップ対象を選択",
            MessageKey::PromptSelectFile => "追加するファイル/ディレクトリを選択: ",
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
            MessageKey::RustFastTypeSafe => {
                "AES-256暗号化 & Smart分析機能搭載のインテリジェントバックアップ"
            }
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

            // Backup progress and result messages
            MessageKey::FilesDetected => "ファイルを検出",
            MessageKey::FullBackupMode => "📦 フルバックアップモード（全ファイル）",
            MessageKey::IncrementalBackupMode => "📦 増分バックアップモード（変更ファイルのみ）",
            MessageKey::BackupComplete => "✓ バックアップ完了",
            MessageKey::BackupCompleteWithFailures => "⚠ バックアップ完了（失敗あり）",
            MessageKey::BackupResultTitle => "📈 バックアップ結果",
            MessageKey::TotalFilesLabel => "総ファイル数",
            MessageKey::SuccessfulLabel => "成功",
            MessageKey::FailedLabel => "失敗",
            MessageKey::TotalSizeLabel => "合計サイズ",

            // Remove/Update command messages
            MessageKey::ConfirmRemoveTarget => "本当に {} をバックアップ対象から削除しますか？",
            MessageKey::UpdatedTarget => "バックアップ対象を更新しました",
            MessageKey::PathLabel => "パス",
            MessageKey::PriorityLabel => "優先度",
            MessageKey::CategoryLabel => "カテゴリ",
            MessageKey::ExcludePatternsLabel => "除外パターン",

            // Smart Analyze labels
            MessageKey::ItemLabel => "項目",
            MessageKey::ValueLabel => "値",
            MessageKey::ImportanceScoreLabel => "重要度スコア",
            MessageKey::RecommendedPriorityLabel => "推奨優先度",
            MessageKey::ReasonLabel => "理由",

            // Smart Auto-Configure labels
            MessageKey::AnalyzingLabel => "分析中",
            MessageKey::AddedToConfiguration => "設定に追加しました",
            MessageKey::ItemsAdded => "追加された項目",
            MessageKey::ExistingBackupTargets => "現在{}個のバックアップ対象が登録されています",
            MessageKey::AddNewTargets => "新しいターゲットを追加しますか？",

            // History detailed view
            MessageKey::TimestampLabel => "日時",
            MessageKey::PathHistoryLabel => "パス",
            MessageKey::StatusHistoryLabel => "ステータス",
            MessageKey::FilesHistoryLabel => "ファイル数",
            MessageKey::SizeLabel => "サイズ",
            MessageKey::CompressionLabel => "圧縮",
            MessageKey::EncryptionLabel => "暗号化",
            MessageKey::DurationLabel => "処理時間",
            MessageKey::EnabledLabel => "有効",
            MessageKey::SecondsUnit => "秒",

            // Dashboard sections
            MessageKey::StatisticsTitle => "📈 統計情報",
            MessageKey::DiskUsageTitle => "💾 ディスク使用量",
            MessageKey::AllNormalStatus => "⚡ すべて正常です",
            MessageKey::WarningsTitle => "⚠️  警告・注意事項",

            // Incremental backup messages
            MessageKey::PreviousBackupLabel => "前回バックアップ",
            MessageKey::ChangedFilesLabel => "変更ファイル数",
            MessageKey::NoBackupsFound => "ℹ️  前回のバックアップが見つかりません。フルバックアップを実行します。",
            MessageKey::FullBackupFallback => "⚠️  前回のメタデータ読み込みに失敗しました。フルバックアップにフォールバックします。",
            MessageKey::MetadataLoadFailed => "   詳細",
            MessageKey::DryRunMode => "📋 ドライランモード: {} ファイルをバックアップ対象として検出",

            // Dashboard statistics labels
            MessageKey::TotalTargetsLabel => "総対象数",
            MessageKey::HighPriorityTargetsLabel => "  高優先度",
            MessageKey::MediumPriorityTargetsLabel => "  中優先度",
            MessageKey::LowPriorityTargetsLabel => "  低優先度",
            MessageKey::TotalBackupsLabel => "総バックアップ回数",
            MessageKey::SuccessCountLabel => "  成功",
            MessageKey::TotalFilesCountLabel => "総ファイル数",
            MessageKey::TotalDataSizeLabel => "総データサイズ",
            MessageKey::LastBackupLabel => "最終バックアップ",
            MessageKey::EncryptedBackupsLabel => "暗号化バックアップ",
            MessageKey::CompressedBackupsLabel => "圧縮バックアップ",
            MessageKey::BackupDirectoryLabel => "バックアップディレクトリ",
            MessageKey::UsedCapacityLabel => "使用容量",
            MessageKey::FileCountLabel => "ファイル数",
            MessageKey::DiskTotalCapacityLabel => "ディスク総容量",
            MessageKey::DiskFreeCapacityLabel => "ディスク空き容量",
            MessageKey::DiskUsageRateLabel => "ディスク使用率",
            MessageKey::UsageStatusLabel => "使用状況",
            MessageKey::RecentBackupsTitle => "🕒 最近のバックアップ（直近5件）",

            // Schedule table headers
            MessageKey::ScheduleHeaderLabel => "スケジュール",

            // Relative time messages
            MessageKey::DaysAgo => "{}日前",
            MessageKey::HoursAgo => "{}時間前",
            MessageKey::MinutesAgo => "{}分前",
            MessageKey::JustNow => "たった今",
            MessageKey::NotYetBackedUp => "未実施",

            // Dashboard warning messages
            MessageKey::WarningTargetNotExists => "バックアップ対象が存在しません: {}",
            MessageKey::WarningDaysSinceLastBackup => "最後のバックアップから{}日経過しています",
            MessageKey::WarningNoBackupYet => "まだ一度もバックアップが実行されていません",
            MessageKey::WarningFailedBackups => "失敗したバックアップが{}件あります",
            MessageKey::WarningLowDiskSpace => "ディスク空き容量が少なくなっています ({:.1}%)",
            MessageKey::DashboardHintRunBackup => "💡 ヒント: 'backup-suite run' でバックアップを実行できます",

            // Interactive prompts
            MessageKey::PromptPleaseSelect => "選択してください",
            MessageKey::PromptDeleteBackup => "このバックアップを削除しますか？",
            MessageKey::PromptDeleteOldBackups => "🗑️  古いバックアップを削除",
            MessageKey::PromptDeleteTarget => "削除対象: {} 件のバックアップ",
            MessageKey::PromptDeleteCount => "件",
            MessageKey::PromptConfirmDelete => "削除を実行しますか？",
            MessageKey::PromptSelectPriority => "優先度を選択してください",
            MessageKey::PromptBackupConfirm => "バックアップを実行しますか？",

            // Smart Analyze categories
            MessageKey::SmartCategoryDirectory => "ディレクトリ",
            MessageKey::SmartCategoryRustProject => "Rustプロジェクト",
            MessageKey::SmartCategoryNodeJsProject => "Node.jsプロジェクト",
            MessageKey::SmartCategoryPythonProject => "Pythonプロジェクト",
            MessageKey::SmartCategorySourceCodeProject => "ソースコードプロジェクト",
            MessageKey::SmartCategoryGitManaged => "Git管理ディレクトリ",
            MessageKey::SmartCategoryLowPriority => "低優先度ディレクトリ",

            // Smart Analyze reasons
            MessageKey::SmartReasonSampling => "(サンプリング: {}ファイル, 高重要度: {}件, スコア: {})",
            MessageKey::SmartReasonScore => "(スコア: {})",
            MessageKey::SmartReasonSecurityDir => "認証情報・秘密鍵（{}ディレクトリ、暗号化必須、スコア: 95）",
            MessageKey::SmartReasonLowPriorityDir => "キャッシュ/ログ/アーカイブ等 (ディレクトリ: {}, スコア: 20)",

            // Smart Exclude reasons
            MessageKey::ExcludeReasonNpmDeps => "npm/yarn依存関係（package.jsonから再生成可能）",
            MessageKey::ExcludeReasonRustBuild => "Rustビルド成果物（Cargo.tomlから再生成可能）",
            MessageKey::ExcludeReasonVendor => "依存関係ベンダリング（再生成可能）",
            MessageKey::ExcludeReasonPythonCache => "Pythonキャッシュ（自動生成）",
            MessageKey::ExcludeReasonPytestCache => "pytestキャッシュ（自動生成）",
            MessageKey::ExcludeReasonBuildArtifacts => "ビルド成果物ディレクトリ（再ビルド可能）",
            MessageKey::ExcludeReasonCacheDir => "キャッシュディレクトリ（一時データ）",
            MessageKey::ExcludeReasonGitMetadata => "Gitリポジトリメタデータ（リモートから復元可能）",
            MessageKey::ExcludeReasonSvnMetadata => "SVNリポジトリメタデータ（リモートから復元可能）",
            MessageKey::ExcludeReasonTempFile => "一時ファイル",
            MessageKey::ExcludeReasonBackupFile => "バックアップファイル（元ファイルがあれば不要）",
            MessageKey::ExcludeReasonEditorTemp => "エディタ一時ファイル",
            MessageKey::ExcludeReasonLogFile => "ログファイル（古いログは通常不要）",
            MessageKey::ExcludeReasonMacOsMetadata => "macOSメタデータファイル（自動生成）",
            MessageKey::ExcludeReasonWindowsThumb => "Windowsサムネイルキャッシュ（自動生成）",
            MessageKey::ExcludeReasonWindowsDesktop => "Windowsデスクトップ設定ファイル（自動生成）",

            // Password strength messages
            MessageKey::PasswordStrengthLabel => "パスワード強度:",
            MessageKey::PasswordStrengthWeak => "弱い",
            MessageKey::PasswordStrengthMedium => "普通",
            MessageKey::PasswordStrengthStrong => "強い",
            MessageKey::PasswordStrengthWeakMessage => {
                "このパスワードは攻撃に対して脆弱な可能性があります。より長く、多様な文字を含むパスワードの使用を検討してください。"
            }
            MessageKey::PasswordStrengthMediumMessage => {
                "このパスワードは中程度のセキュリティを提供します。特殊文字の追加や長さの延長で改善できます。"
            }
            MessageKey::PasswordStrengthStrongMessage => {
                "このパスワードは強力なセキュリティを提供します。"
            }
            MessageKey::PasswordStrengthTip => {
                "ヒント: --generate-password を使用すると強力なランダムパスワードを生成できます。"
            }

            // Editor and config
            MessageKey::EditorLaunchFailed => "エディタ起動失敗: {}",

            // Smart feature progress
            MessageKey::SubdirectoriesFound => "{}個のサブディレクトリを発見",
            MessageKey::ProgressEvaluating => "処理進捗 - 評価中: {:?}",

            // Backup confirmation prompts
            MessageKey::ConfirmBackupTitle => "📦 バックアップ実行確認",
            MessageKey::ConfirmBackupTargetFiles => "対象ファイル数: {} ファイル",
            MessageKey::ConfirmBackupDestination => "バックアップ先: {}",

            // Cleanup confirmation prompts
            MessageKey::ConfirmCleanupTitle => "🗑️  古いバックアップの削除",
            MessageKey::ConfirmCleanupTargetCount => "削除対象: {} 個のバックアップ",
            MessageKey::ConfirmCleanupRetentionDays => "保持期間: {} 日",

            // Cleanup progress messages
            MessageKey::CleanupDryRunScheduled => "🗑️  [ドライラン] 削除予定: {:?}",
            MessageKey::CleanupCompleted => "🗑️  削除完了: {:?}",
            MessageKey::CleanupFailed => "削除失敗 {:?}: {}",

            // Restore progress messages
            MessageKey::RestoreDryRunDetected => "📋 ドライランモード: {} ファイルを復元対象として検出",
            MessageKey::RestoreInProgress => "復元中...",
            MessageKey::RestoreProgressFile => "復元中: {:?}",
            MessageKey::RestoreIntegrityMetadataLoaded => "✓ 整合性メタデータ読み込み完了（{} バックアップ）",
            MessageKey::RestoreCompleted => "✓ 復元完了",
            MessageKey::RestoreCompletedWithFailures => "⚠ 復元完了（{}件失敗）",

            // Restore error messages
            MessageKey::ErrorRelativePathFailed => "相対パス取得失敗 {}: {}",
            MessageKey::ErrorPathTraversalDetected => "パストラバーサル検出 {}: {}",
            MessageKey::ErrorDirectoryCreateFailed => "ディレクトリ作成失敗 {}: {}",
            MessageKey::ErrorFileReadFailed => "ファイル読み込み失敗: {}",
            MessageKey::ErrorFileOpenFailedSymlink => "ファイルオープン失敗（シンボリックリンク攻撃の可能性）: {}",
            MessageKey::ErrorEncryptedButNoPassword => "暗号化ファイルですがパスワード未指定: {}",
            MessageKey::ErrorMasterKeyRestoreFailed => "マスターキー復元失敗: {}",
            MessageKey::ErrorDecryptionFailed => "復号化失敗 {}: {}",
            MessageKey::ErrorIntegrityVerificationFailed => "⚠ 整合性検証失敗（ファイル改ざんの可能性）: {}",
            MessageKey::ErrorFileWriteFailed => "ファイル書き込み失敗 {}: {}",
            MessageKey::ErrorFileCountFailed => "{}ファイルでエラー発生",

            // Backup progress and error messages
            MessageKey::BackupProgressProcessing => "処理中: {:?}",
            MessageKey::ErrorBackupDirectoryCreateFailed => "ディレクトリ作成失敗 {}: {}",
            MessageKey::ErrorBackupWriteFailed => "書き込み失敗 {}: {}",
            MessageKey::ErrorBackupProcessFailed => "処理失敗 {}: {}",
            MessageKey::ErrorBackupCopyFailed => "コピー失敗 {}: {}",
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
            MessageKey::RustFastTypeSafe => "AES-256加密 & Smart分析功能的智能备份",

            // Status messages
            MessageKey::Added => "已添加",
            MessageKey::Removed => "已删除",
            MessageKey::Deleted => "已删除",
            MessageKey::Error => "错误",
            MessageKey::Warning => "⚠️",
            MessageKey::BackupRunning => "🚀 正在备份",
            MessageKey::RestoreStarting => "🔄 开始恢复",

            // Encryption and compression
            MessageKey::EncryptionPassword => "加密密码",
            MessageKey::SavePasswordSecurely => "⚠️  请安全保存此密码！",
            MessageKey::EncryptOption => "--encrypt: AES-256-GCM加密",
            MessageKey::CompressOption => "--compress zstd/gzip: 压缩",
            MessageKey::CompressLevel => "--compress-level 1-22: 压缩级别",

            // Run command options
            MessageKey::IncrementalOption => "--incremental: 增量备份（仅变更文件）",
            MessageKey::GeneratePasswordOption => "--generate-password: 自动生成安全密码",
            MessageKey::PasswordOption => "--password <密码>: 指定加密密码",
            MessageKey::DryRunOption => "--dry-run: 演习模式（不实际备份）",
            MessageKey::PriorityOption => "--priority <优先级>: 按优先级过滤 (high/medium/low)",
            MessageKey::CategoryOption => "--category <类别>: 按类别过滤",

            // Restore command options
            MessageKey::FromOption => "--from <备份名称>: 要恢复的备份",
            MessageKey::ToOption => "--to <目标路径>: 恢复目标路径",
            MessageKey::RestorePasswordOption => "--password <密码>: 解密密码（如已加密）",

            // Runtime messages
            MessageKey::NoTargetsRegistered => "未注册备份目标",
            MessageKey::SelectionCancelled => "选择已取消",
            MessageKey::ConfirmClearAll => "⚠️  警告：删除所有 {} 个备份目标。确定吗？",
            MessageKey::ConfirmClearPriority => "⚠️  警告：删除 {} 个{}优先级备份目标。确定吗？",
            MessageKey::ConfirmCleanup => "删除 {} 天之前的旧备份。确定吗？",
            MessageKey::DaysOutOfRange => "days 必须在 1-3650 范围内（指定值：{}）",
            MessageKey::PathNotExists => "路径不存在",
            MessageKey::NotInBackupConfig => "未在备份配置中注册",
            MessageKey::SpecifyPriorityOrAll => "请指定 --priority 或 --all",
            MessageKey::CountDeleted => "已删除",
            MessageKey::DryRun => "演习模式",
            MessageKey::Category => "类别",
            MessageKey::Encryption => "加密",
            MessageKey::Compression => "压缩",
            MessageKey::ErrorDetails => "错误详情",
            MessageKey::Detected => "检测到",
            MessageKey::Files => "文件",
            MessageKey::Days => "天",
            MessageKey::DryRunParens => "（演习模式）",
            MessageKey::DaysUnit => "天",

            // Common messages
            MessageKey::UsageExamples => "使用示例:",

            // Backup progress and result messages
            MessageKey::FilesDetected => "检测到文件",
            MessageKey::FullBackupMode => "📦 完全备份模式（所有文件）",
            MessageKey::IncrementalBackupMode => "📦 增量备份模式（仅变更文件）",
            MessageKey::BackupComplete => "✓ 备份完成",
            MessageKey::BackupCompleteWithFailures => "⚠ 备份完成（有失败）",
            MessageKey::BackupResultTitle => "📈 备份结果",
            MessageKey::TotalFilesLabel => "总文件数",
            MessageKey::SuccessfulLabel => "成功",
            MessageKey::FailedLabel => "失败",
            MessageKey::TotalSizeLabel => "总大小",

            // Remove/Update command messages
            MessageKey::ConfirmRemoveTarget => "确定要从备份目标中删除 {} 吗？",
            MessageKey::UpdatedTarget => "已更新备份目标",
            MessageKey::PathLabel => "路径",
            MessageKey::PriorityLabel => "优先级",
            MessageKey::CategoryLabel => "类别",
            MessageKey::ExcludePatternsLabel => "排除模式",

            // Smart Analyze labels
            MessageKey::ItemLabel => "项目",
            MessageKey::ValueLabel => "值",
            MessageKey::ImportanceScoreLabel => "重要性分数",
            MessageKey::RecommendedPriorityLabel => "推荐优先级",
            MessageKey::ReasonLabel => "原因",

            // Smart Auto-Configure labels
            MessageKey::AnalyzingLabel => "分析中",
            MessageKey::AddedToConfiguration => "已添加到配置",
            MessageKey::ItemsAdded => "已添加项目",
            MessageKey::ExistingBackupTargets => "您现有{}个备份目标",
            MessageKey::AddNewTargets => "添加新目标？",

            // History detailed view
            MessageKey::TimestampLabel => "时间",
            MessageKey::PathHistoryLabel => "路径",
            MessageKey::StatusHistoryLabel => "状态",
            MessageKey::FilesHistoryLabel => "文件数",
            MessageKey::SizeLabel => "大小",
            MessageKey::CompressionLabel => "压缩",
            MessageKey::EncryptionLabel => "加密",
            MessageKey::DurationLabel => "处理时间",
            MessageKey::EnabledLabel => "已启用",
            MessageKey::SecondsUnit => "秒",

            // Dashboard sections
            MessageKey::StatisticsTitle => "📈 统计信息",
            MessageKey::DiskUsageTitle => "💾 磁盘使用量",
            MessageKey::AllNormalStatus => "⚡ 一切正常",
            MessageKey::WarningsTitle => "⚠️  警告·注意事项",

            // Incremental backup messages
            MessageKey::PreviousBackupLabel => "上次备份",
            MessageKey::ChangedFilesLabel => "变更文件数",
            MessageKey::NoBackupsFound => "ℹ️  未找到上次备份。执行完全备份。",
            MessageKey::FullBackupFallback => "⚠️  加载元数据失败。回退到完全备份。",
            MessageKey::MetadataLoadFailed => "   详情",
            MessageKey::DryRunMode => "📋 演习模式: 检测到 {} 个文件待备份",

            // Dashboard statistics labels
            MessageKey::TotalTargetsLabel => "总目标数",
            MessageKey::HighPriorityTargetsLabel => "  高优先级",
            MessageKey::MediumPriorityTargetsLabel => "  中优先级",
            MessageKey::LowPriorityTargetsLabel => "  低优先级",
            MessageKey::TotalBackupsLabel => "总备份次数",
            MessageKey::SuccessCountLabel => "  成功",
            MessageKey::TotalFilesCountLabel => "总文件数",
            MessageKey::TotalDataSizeLabel => "总数据大小",
            MessageKey::LastBackupLabel => "最后备份",
            MessageKey::EncryptedBackupsLabel => "加密备份",
            MessageKey::CompressedBackupsLabel => "压缩备份",
            MessageKey::BackupDirectoryLabel => "备份目录",
            MessageKey::UsedCapacityLabel => "已用容量",
            MessageKey::FileCountLabel => "文件数",
            MessageKey::DiskTotalCapacityLabel => "磁盘总容量",
            MessageKey::DiskFreeCapacityLabel => "磁盘可用容量",
            MessageKey::DiskUsageRateLabel => "磁盘使用率",
            MessageKey::UsageStatusLabel => "使用状态",
            MessageKey::RecentBackupsTitle => "🕒 最近备份（最新5次）",

            // Schedule table headers
            MessageKey::ScheduleHeaderLabel => "计划",

            // Relative time messages
            MessageKey::DaysAgo => "{}天前",
            MessageKey::HoursAgo => "{}小时前",
            MessageKey::MinutesAgo => "{}分钟前",
            MessageKey::JustNow => "刚刚",
            MessageKey::NotYetBackedUp => "尚未执行",

            // Dashboard warning messages
            MessageKey::WarningTargetNotExists => "备份目标不存在: {}",
            MessageKey::WarningDaysSinceLastBackup => "距离上次备份已过去{}天",
            MessageKey::WarningNoBackupYet => "尚未执行过备份",
            MessageKey::WarningFailedBackups => "有{}个失败的备份",
            MessageKey::WarningLowDiskSpace => "磁盘空间不足 ({:.1}%)",
            MessageKey::DashboardHintRunBackup => "💡 提示: 运行 'backup-suite run' 执行备份",

            // Interactive prompts
            MessageKey::PromptPleaseSelect => "请选择",
            MessageKey::PromptDeleteBackup => "确定要删除此备份吗？",
            MessageKey::PromptDeleteOldBackups => "🗑️  删除旧备份",
            MessageKey::PromptDeleteTarget => "删除目标: {} 个备份",
            MessageKey::PromptDeleteCount => "个",
            MessageKey::PromptConfirmDelete => "确定要执行删除吗？",
            MessageKey::PromptSelectPriority => "选择优先级",
            MessageKey::PromptBackupConfirm => "确定要执行备份吗？",
            MessageKey::PromptSelectTarget => "选择要删除的备份目标",
            MessageKey::PromptSelectFile => "选择要添加的文件/目录: ",

            // Smart Analyze categories
            MessageKey::SmartCategoryDirectory => "目录",
            MessageKey::SmartCategoryRustProject => "Rust项目",
            MessageKey::SmartCategoryNodeJsProject => "Node.js项目",
            MessageKey::SmartCategoryPythonProject => "Python项目",
            MessageKey::SmartCategorySourceCodeProject => "源代码项目",
            MessageKey::SmartCategoryGitManaged => "Git管理目录",
            MessageKey::SmartCategoryLowPriority => "低优先级目录",

            // Smart Analyze reasons
            MessageKey::SmartReasonSampling => "(采样: {}文件, 高重要性: {}个, 分数: {})",
            MessageKey::SmartReasonScore => "(分数: {})",
            MessageKey::SmartReasonSecurityDir => "凭证/密钥（{}目录，需要加密，分数: 95）",
            MessageKey::SmartReasonLowPriorityDir => "缓存/日志/存档等 (目录: {}, 分数: 20)",

            // Smart Exclude reasons
            MessageKey::ExcludeReasonNpmDeps => "npm/yarn依赖（可从package.json重新生成）",
            MessageKey::ExcludeReasonRustBuild => "Rust构建产物（可从Cargo.toml重新生成）",
            MessageKey::ExcludeReasonVendor => "依赖供应（可重新生成）",
            MessageKey::ExcludeReasonPythonCache => "Python缓存（自动生成）",
            MessageKey::ExcludeReasonPytestCache => "pytest缓存（自动生成）",
            MessageKey::ExcludeReasonBuildArtifacts => "构建产物目录（可重新构建）",
            MessageKey::ExcludeReasonCacheDir => "缓存目录（临时数据）",
            MessageKey::ExcludeReasonGitMetadata => "Git仓库元数据（可从远程恢复）",
            MessageKey::ExcludeReasonSvnMetadata => "SVN仓库元数据（可从远程恢复）",
            MessageKey::ExcludeReasonTempFile => "临时文件",
            MessageKey::ExcludeReasonBackupFile => "备份文件（如果原文件存在则不需要）",
            MessageKey::ExcludeReasonEditorTemp => "编辑器临时文件",
            MessageKey::ExcludeReasonLogFile => "日志文件（旧日志通常不需要）",
            MessageKey::ExcludeReasonMacOsMetadata => "macOS元数据文件（自动生成）",
            MessageKey::ExcludeReasonWindowsThumb => "Windows缩略图缓存（自动生成）",
            MessageKey::ExcludeReasonWindowsDesktop => "Windows桌面设置文件（自动生成）",

            // Password strength messages
            MessageKey::PasswordStrengthLabel => "密码强度:",
            MessageKey::PasswordStrengthWeak => "弱",
            MessageKey::PasswordStrengthMedium => "中等",
            MessageKey::PasswordStrengthStrong => "强",
            MessageKey::PasswordStrengthWeakMessage => {
                "此密码可能容易受到攻击。建议使用更长且包含多种字符的密码。"
            }
            MessageKey::PasswordStrengthMediumMessage => {
                "此密码提供中等安全性。添加特殊字符或增加长度可以改善。"
            }
            MessageKey::PasswordStrengthStrongMessage => "此密码提供强大的安全性。",
            MessageKey::PasswordStrengthTip => "提示: 使用 --generate-password 生成强随机密码。",

            // Editor and config
            MessageKey::EditorLaunchFailed => "启动编辑器失败: {}",

            // Smart feature progress
            MessageKey::SubdirectoriesFound => "发现{}个子目录",
            MessageKey::ProgressEvaluating => "处理进度 - 评估中: {:?}",

            // Backup confirmation prompts
            MessageKey::ConfirmBackupTitle => "📦 备份执行确认",
            MessageKey::ConfirmBackupTargetFiles => "目标文件数: {} 文件",
            MessageKey::ConfirmBackupDestination => "备份目标: {}",

            // Cleanup confirmation prompts
            MessageKey::ConfirmCleanupTitle => "🗑️  删除旧备份",
            MessageKey::ConfirmCleanupTargetCount => "删除目标: {} 个备份",
            MessageKey::ConfirmCleanupRetentionDays => "保留期限: {} 天",

            // Cleanup progress messages
            MessageKey::CleanupDryRunScheduled => "🗑️  [演习模式] 计划删除: {:?}",
            MessageKey::CleanupCompleted => "🗑️  删除完成: {:?}",
            MessageKey::CleanupFailed => "删除失败 {:?}: {}",

            // Restore progress messages
            MessageKey::RestoreDryRunDetected => "📋 演习模式: 检测到 {} 个文件待还原",
            MessageKey::RestoreInProgress => "还原中...",
            MessageKey::RestoreProgressFile => "还原中: {:?}",
            MessageKey::RestoreIntegrityMetadataLoaded => "✓ 完整性元数据已加载（{} 个备份）",
            MessageKey::RestoreCompleted => "✓ 还原完成",
            MessageKey::RestoreCompletedWithFailures => "⚠ 还原完成（{}个失败）",

            // Restore error messages
            MessageKey::ErrorRelativePathFailed => "获取相对路径失败 {}: {}",
            MessageKey::ErrorPathTraversalDetected => "检测到路径遍历 {}: {}",
            MessageKey::ErrorDirectoryCreateFailed => "创建目录失败 {}: {}",
            MessageKey::ErrorFileReadFailed => "读取文件失败: {}",
            MessageKey::ErrorFileOpenFailedSymlink => "打开文件失败（可能是符号链接攻击）: {}",
            MessageKey::ErrorEncryptedButNoPassword => "加密文件但未指定密码: {}",
            MessageKey::ErrorMasterKeyRestoreFailed => "恢复主密钥失败: {}",
            MessageKey::ErrorDecryptionFailed => "解密失败 {}: {}",
            MessageKey::ErrorIntegrityVerificationFailed => {
                "⚠ 完整性验证失败（文件可能被篡改）: {}"
            }
            MessageKey::ErrorFileWriteFailed => "写入文件失败 {}: {}",
            MessageKey::ErrorFileCountFailed => "{}个文件发生错误",

            // Backup progress and error messages
            MessageKey::BackupProgressProcessing => "处理中: {:?}",
            MessageKey::ErrorBackupDirectoryCreateFailed => "创建目录失败 {}: {}",
            MessageKey::ErrorBackupWriteFailed => "写入失败 {}: {}",
            MessageKey::ErrorBackupProcessFailed => "处理失败 {}: {}",
            MessageKey::ErrorBackupCopyFailed => "复制失败 {}: {}",

            // Newly added translations for Simplified Chinese
            MessageKey::NoBackups => "无备份",
            MessageKey::RestoreStart => "开始恢复",
            MessageKey::Restoring => "恢复中...",
            MessageKey::RestoredSuccess => "成功恢复备份到",
            MessageKey::RestoredFileCount => "恢复文件数:",
            MessageKey::BackupHistory => "备份历史",
            MessageKey::ActualScheduleStatus => "实际调度状态",
            MessageKey::Enabled => "已启用",
            MessageKey::Disabled => "已禁用",
            MessageKey::ScheduleSettings => "调度设置",
            MessageKey::ScheduleUpdated => "调度已更新并应用",
            MessageKey::ScheduleUpdatedEnableLater => {
                "调度设置已更新（使用 'schedule enable' 启用）"
            }
            MessageKey::HighPriority => "高优先级",
            MessageKey::MediumPriority => "中优先级",
            MessageKey::LowPriority => "低优先级",
            MessageKey::CurrentDestination => "当前备份目标",
            MessageKey::DestinationChanged => "备份目标已更改",
            MessageKey::Before => "之前",
            MessageKey::After => "之后",
            MessageKey::KeepDaysOutOfRange => "keep_days 必须在 1-3650 之间（指定值：",
            MessageKey::KeepDaysChanged => "备份保留期限已更改",
            MessageKey::CurrentKeepDays => "当前备份保留期限",
            MessageKey::OpeningConfigFile => "打开配置文件",
            MessageKey::EditorDidNotExitCleanly => "编辑器未正常退出",
            MessageKey::AutoBackupEnabled => "已启用自动备份",
            MessageKey::AutoBackupDisabled => "已禁用自动备份",

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
            MessageKey::RustFastTypeSafe => "AES-256加密 & Smart分析功能的智慧備份",

            // Status messages
            MessageKey::Added => "已加入",
            MessageKey::Removed => "已刪除",
            MessageKey::Deleted => "已刪除",
            MessageKey::Error => "錯誤",
            MessageKey::Warning => "⚠️",
            MessageKey::BackupRunning => "🚀 正在備份",
            MessageKey::RestoreStarting => "🔄 開始還原",

            // Encryption and compression
            MessageKey::EncryptionPassword => "加密密碼",
            MessageKey::SavePasswordSecurely => "⚠️  請安全儲存此密碼！",
            MessageKey::EncryptOption => "--encrypt: AES-256-GCM加密",
            MessageKey::CompressOption => "--compress zstd/gzip: 壓縮",
            MessageKey::CompressLevel => "--compress-level 1-22: 壓縮級別",

            // Run command options
            MessageKey::IncrementalOption => "--incremental: 增量備份（僅變更檔案）",
            MessageKey::GeneratePasswordOption => "--generate-password: 自動生成安全密碼",
            MessageKey::PasswordOption => "--password <密碼>: 指定加密密碼",
            MessageKey::DryRunOption => "--dry-run: 演習模式（不實際備份）",
            MessageKey::PriorityOption => "--priority <優先級>: 按優先級過濾 (high/medium/low)",
            MessageKey::CategoryOption => "--category <類別>: 按類別過濾",

            // Restore command options
            MessageKey::FromOption => "--from <備份名稱>: 要還原的備份",
            MessageKey::ToOption => "--to <目標路徑>: 還原目標路徑",
            MessageKey::RestorePasswordOption => "--password <密碼>: 解密密碼（如已加密）",

            // Runtime messages
            MessageKey::NoTargetsRegistered => "未註冊備份目標",
            MessageKey::SelectionCancelled => "選擇已取消",
            MessageKey::ConfirmClearAll => "⚠️  警告：刪除所有 {} 個備份目標。確定嗎？",
            MessageKey::ConfirmClearPriority => "⚠️  警告：刪除 {} 個{}優先級備份目標。確定嗎？",
            MessageKey::ConfirmCleanup => "刪除 {} 天之前的舊備份。確定嗎？",
            MessageKey::DaysOutOfRange => "days 必須在 1-3650 範圍內（指定值：{}）",
            MessageKey::PathNotExists => "路徑不存在",
            MessageKey::NotInBackupConfig => "未在備份設定中註冊",
            MessageKey::SpecifyPriorityOrAll => "請指定 --priority 或 --all",
            MessageKey::CountDeleted => "已刪除",
            MessageKey::DryRun => "演習模式",
            MessageKey::Category => "類別",
            MessageKey::Encryption => "加密",
            MessageKey::Compression => "壓縮",
            MessageKey::ErrorDetails => "錯誤詳情",
            MessageKey::Detected => "檢測到",
            MessageKey::Files => "檔案",
            MessageKey::Days => "天",
            MessageKey::DryRunParens => "（演習模式）",
            MessageKey::DaysUnit => "天",

            // Common messages
            MessageKey::UsageExamples => "使用範例:",

            // Backup progress and result messages
            MessageKey::FilesDetected => "檢測到檔案",
            MessageKey::FullBackupMode => "📦 完全備份模式（所有檔案）",
            MessageKey::IncrementalBackupMode => "📦 增量備份模式（僅變更檔案）",
            MessageKey::BackupComplete => "✓ 備份完成",
            MessageKey::BackupCompleteWithFailures => "⚠ 備份完成（有失敗）",
            MessageKey::BackupResultTitle => "📈 備份結果",
            MessageKey::TotalFilesLabel => "總檔案數",
            MessageKey::SuccessfulLabel => "成功",
            MessageKey::FailedLabel => "失敗",
            MessageKey::TotalSizeLabel => "總大小",

            // Remove/Update command messages
            MessageKey::ConfirmRemoveTarget => "確定要從備份目標中刪除 {} 嗎？",
            MessageKey::UpdatedTarget => "已更新備份目標",
            MessageKey::PathLabel => "路徑",
            MessageKey::PriorityLabel => "優先級",
            MessageKey::CategoryLabel => "類別",
            MessageKey::ExcludePatternsLabel => "排除模式",

            // Smart Analyze labels
            MessageKey::ItemLabel => "項目",
            MessageKey::ValueLabel => "值",
            MessageKey::ImportanceScoreLabel => "重要性分數",
            MessageKey::RecommendedPriorityLabel => "推薦優先級",
            MessageKey::ReasonLabel => "原因",

            // Smart Auto-Configure labels
            MessageKey::AnalyzingLabel => "分析中",
            MessageKey::AddedToConfiguration => "已加入至設定",
            MessageKey::ItemsAdded => "已加入項目",
            MessageKey::ExistingBackupTargets => "您現有{}個備份目標",
            MessageKey::AddNewTargets => "加入新目標？",

            // History detailed view
            MessageKey::TimestampLabel => "時間",
            MessageKey::PathHistoryLabel => "路徑",
            MessageKey::StatusHistoryLabel => "狀態",
            MessageKey::FilesHistoryLabel => "檔案數",
            MessageKey::SizeLabel => "大小",
            MessageKey::CompressionLabel => "壓縮",
            MessageKey::EncryptionLabel => "加密",
            MessageKey::DurationLabel => "處理時間",
            MessageKey::EnabledLabel => "已啟用",
            MessageKey::SecondsUnit => "秒",

            // Dashboard sections
            MessageKey::StatisticsTitle => "📈 統計資訊",
            MessageKey::DiskUsageTitle => "💾 磁碟使用量",
            MessageKey::AllNormalStatus => "⚡ 一切正常",
            MessageKey::WarningsTitle => "⚠️  警告·注意事項",

            // Incremental backup messages
            MessageKey::PreviousBackupLabel => "上次備份",
            MessageKey::ChangedFilesLabel => "變更檔案數",
            MessageKey::NoBackupsFound => "ℹ️  未找到上次備份。執行完全備份。",
            MessageKey::FullBackupFallback => "⚠️  載入元數據失敗。回退到完全備份。",
            MessageKey::MetadataLoadFailed => "   詳情",
            MessageKey::DryRunMode => "📋 演習模式: 檢測到 {} 個檔案待備份",

            // Dashboard statistics labels
            MessageKey::TotalTargetsLabel => "總目標數",
            MessageKey::HighPriorityTargetsLabel => "  高優先級",
            MessageKey::MediumPriorityTargetsLabel => "  中優先級",
            MessageKey::LowPriorityTargetsLabel => "  低優先級",
            MessageKey::TotalBackupsLabel => "總備份次數",
            MessageKey::SuccessCountLabel => "  成功",
            MessageKey::TotalFilesCountLabel => "總檔案數",
            MessageKey::TotalDataSizeLabel => "總資料大小",
            MessageKey::LastBackupLabel => "最後備份",
            MessageKey::EncryptedBackupsLabel => "加密備份",
            MessageKey::CompressedBackupsLabel => "壓縮備份",
            MessageKey::BackupDirectoryLabel => "備份目錄",
            MessageKey::UsedCapacityLabel => "已用容量",
            MessageKey::FileCountLabel => "檔案數",
            MessageKey::DiskTotalCapacityLabel => "磁碟總容量",
            MessageKey::DiskFreeCapacityLabel => "磁碟可用容量",
            MessageKey::DiskUsageRateLabel => "磁碟使用率",
            MessageKey::UsageStatusLabel => "使用狀態",
            MessageKey::RecentBackupsTitle => "🕒 最近備份（最新5次）",

            // Schedule table headers
            MessageKey::ScheduleHeaderLabel => "計劃",

            // Relative time messages
            MessageKey::DaysAgo => "{}天前",
            MessageKey::HoursAgo => "{}小時前",
            MessageKey::MinutesAgo => "{}分鐘前",
            MessageKey::JustNow => "剛剛",
            MessageKey::NotYetBackedUp => "尚未執行",

            // Dashboard warning messages
            MessageKey::WarningTargetNotExists => "備份目標不存在: {}",
            MessageKey::WarningDaysSinceLastBackup => "距離上次備份已過去{}天",
            MessageKey::WarningNoBackupYet => "尚未執行過備份",
            MessageKey::WarningFailedBackups => "有{}個失敗的備份",
            MessageKey::WarningLowDiskSpace => "磁碟空間不足 ({:.1}%)",
            MessageKey::DashboardHintRunBackup => "💡 提示: 執行 'backup-suite run' 進行備份",

            // Interactive prompts
            MessageKey::PromptPleaseSelect => "請選擇",
            MessageKey::PromptDeleteBackup => "確定要刪除此備份嗎？",
            MessageKey::PromptDeleteOldBackups => "🗑️  刪除舊備份",
            MessageKey::PromptDeleteTarget => "刪除目標: {} 個備份",
            MessageKey::PromptDeleteCount => "個",
            MessageKey::PromptConfirmDelete => "確定要執行刪除嗎？",
            MessageKey::PromptSelectPriority => "選擇優先級",
            MessageKey::PromptBackupConfirm => "確定要執行備份嗎？",
            MessageKey::PromptSelectTarget => "選擇要刪除的備份目標",
            MessageKey::PromptSelectFile => "選擇要新增的檔案/目錄: ",

            // Smart Analyze categories
            MessageKey::SmartCategoryDirectory => "目錄",
            MessageKey::SmartCategoryRustProject => "Rust專案",
            MessageKey::SmartCategoryNodeJsProject => "Node.js專案",
            MessageKey::SmartCategoryPythonProject => "Python專案",
            MessageKey::SmartCategorySourceCodeProject => "原始碼專案",
            MessageKey::SmartCategoryGitManaged => "Git管理目錄",
            MessageKey::SmartCategoryLowPriority => "低優先級目錄",

            // Smart Analyze reasons
            MessageKey::SmartReasonSampling => "(採樣: {}檔案, 高重要性: {}個, 分數: {})",
            MessageKey::SmartReasonScore => "(分數: {})",
            MessageKey::SmartReasonSecurityDir => "憑證/密鑰（{}目錄，需要加密，分數: 95）",
            MessageKey::SmartReasonLowPriorityDir => "快取/日誌/封存等 (目錄: {}, 分數: 20)",

            // Smart Exclude reasons
            MessageKey::ExcludeReasonNpmDeps => "npm/yarn依賴（可從package.json重新生成）",
            MessageKey::ExcludeReasonRustBuild => "Rust建置產物（可從Cargo.toml重新生成）",
            MessageKey::ExcludeReasonVendor => "依賴供應（可重新生成）",
            MessageKey::ExcludeReasonPythonCache => "Python快取（自動生成）",
            MessageKey::ExcludeReasonPytestCache => "pytest快取（自動生成）",
            MessageKey::ExcludeReasonBuildArtifacts => "建置產物目錄（可重新建置）",
            MessageKey::ExcludeReasonCacheDir => "快取目錄（暫存資料）",
            MessageKey::ExcludeReasonGitMetadata => "Git儲存庫元數據（可從遠端恢復）",
            MessageKey::ExcludeReasonSvnMetadata => "SVN儲存庫元數據（可從遠端恢復）",
            MessageKey::ExcludeReasonTempFile => "暫存檔案",
            MessageKey::ExcludeReasonBackupFile => "備份檔案（如果原檔案存在則不需要）",
            MessageKey::ExcludeReasonEditorTemp => "編輯器暫存檔案",
            MessageKey::ExcludeReasonLogFile => "日誌檔案（舊日誌通常不需要）",
            MessageKey::ExcludeReasonMacOsMetadata => "macOS元數據檔案（自動生成）",
            MessageKey::ExcludeReasonWindowsThumb => "Windows縮圖快取（自動生成）",
            MessageKey::ExcludeReasonWindowsDesktop => "Windows桌面設定檔案（自動生成）",

            // Password strength messages
            MessageKey::PasswordStrengthLabel => "密碼強度:",
            MessageKey::PasswordStrengthWeak => "弱",
            MessageKey::PasswordStrengthMedium => "中等",
            MessageKey::PasswordStrengthStrong => "強",
            MessageKey::PasswordStrengthWeakMessage => {
                "此密碼可能容易受到攻擊。建議使用更長且包含多種字元的密碼。"
            }
            MessageKey::PasswordStrengthMediumMessage => {
                "此密碼提供中等安全性。新增特殊字元或增加長度可以改善。"
            }
            MessageKey::PasswordStrengthStrongMessage => "此密碼提供強大的安全性。",
            MessageKey::PasswordStrengthTip => "提示: 使用 --generate-password 生成強隨機密碼。",

            // Editor and config
            MessageKey::EditorLaunchFailed => "啟動編輯器失敗: {}",

            // Smart feature progress
            MessageKey::SubdirectoriesFound => "發現{}個子目錄",
            MessageKey::ProgressEvaluating => "處理進度 - 評估中: {:?}",

            // Backup confirmation prompts
            MessageKey::ConfirmBackupTitle => "📦 備份執行確認",
            MessageKey::ConfirmBackupTargetFiles => "目標檔案數: {} 檔案",
            MessageKey::ConfirmBackupDestination => "備份目標: {}",

            // Cleanup confirmation prompts
            MessageKey::ConfirmCleanupTitle => "🗑️  刪除舊備份",
            MessageKey::ConfirmCleanupTargetCount => "刪除目標: {} 個備份",
            MessageKey::ConfirmCleanupRetentionDays => "保留期限: {} 天",

            // Cleanup progress messages
            MessageKey::CleanupDryRunScheduled => "🗑️  [演習模式] 計劃刪除: {:?}",
            MessageKey::CleanupCompleted => "🗑️  刪除完成: {:?}",
            MessageKey::CleanupFailed => "刪除失敗 {:?}: {}",

            // Restore progress messages
            MessageKey::RestoreDryRunDetected => "📋 演習模式: 檢測到 {} 個檔案待還原",
            MessageKey::RestoreInProgress => "還原中...",
            MessageKey::RestoreProgressFile => "還原中: {:?}",
            MessageKey::RestoreIntegrityMetadataLoaded => "✓ 完整性元數據已載入（{} 個備份）",
            MessageKey::RestoreCompleted => "✓ 還原完成",
            MessageKey::RestoreCompletedWithFailures => "⚠ 還原完成（{}個失敗）",

            // Restore error messages
            MessageKey::ErrorRelativePathFailed => "取得相對路徑失敗 {}: {}",
            MessageKey::ErrorPathTraversalDetected => "偵測到路徑遍歷 {}: {}",
            MessageKey::ErrorDirectoryCreateFailed => "建立目錄失敗 {}: {}",
            MessageKey::ErrorFileReadFailed => "讀取檔案失敗: {}",
            MessageKey::ErrorFileOpenFailedSymlink => "開啟檔案失敗（可能是符號連結攻擊）: {}",
            MessageKey::ErrorEncryptedButNoPassword => "加密檔案但未指定密碼: {}",
            MessageKey::ErrorMasterKeyRestoreFailed => "恢復主金鑰失敗: {}",
            MessageKey::ErrorDecryptionFailed => "解密失敗 {}: {}",
            MessageKey::ErrorIntegrityVerificationFailed => {
                "⚠ 完整性驗證失敗（檔案可能被竄改）: {}"
            }
            MessageKey::ErrorFileWriteFailed => "寫入檔案失敗 {}: {}",
            MessageKey::ErrorFileCountFailed => "{}個檔案發生錯誤",

            // Backup progress and error messages
            MessageKey::BackupProgressProcessing => "處理中: {:?}",
            MessageKey::ErrorBackupDirectoryCreateFailed => "建立目錄失敗 {}: {}",
            MessageKey::ErrorBackupWriteFailed => "寫入失敗 {}: {}",
            MessageKey::ErrorBackupProcessFailed => "處理失敗 {}: {}",
            MessageKey::ErrorBackupCopyFailed => "複製失敗 {}: {}",

            // Newly added translations for Traditional Chinese
            MessageKey::NoBackups => "無備份",
            MessageKey::RestoreStart => "開始還原",
            MessageKey::Restoring => "還原中...",
            MessageKey::RestoredSuccess => "成功還原備份到",
            MessageKey::RestoredFileCount => "還原檔案數:",
            MessageKey::BackupHistory => "備份歷史",
            MessageKey::ActualScheduleStatus => "實際排程狀態",
            MessageKey::Enabled => "已啟用",
            MessageKey::Disabled => "已停用",
            MessageKey::ScheduleSettings => "排程設定",
            MessageKey::ScheduleUpdated => "排程已更新並套用",
            MessageKey::ScheduleUpdatedEnableLater => {
                "排程設定已更新（使用 'schedule enable' 啟用）"
            }
            MessageKey::HighPriority => "高優先級",
            MessageKey::MediumPriority => "中優先級",
            MessageKey::LowPriority => "低優先級",
            MessageKey::CurrentDestination => "目前備份目標",
            MessageKey::DestinationChanged => "備份目標已變更",
            MessageKey::Before => "之前",
            MessageKey::After => "之後",
            MessageKey::KeepDaysOutOfRange => "keep_days 必須在 1-3650 之間（指定值：",
            MessageKey::KeepDaysChanged => "備份保留期限已變更",
            MessageKey::CurrentKeepDays => "目前備份保留期限",
            MessageKey::OpeningConfigFile => "開啟設定檔",
            MessageKey::EditorDidNotExitCleanly => "編輯器未正常結束",
            MessageKey::AutoBackupEnabled => "已啟用自動備份",
            MessageKey::AutoBackupDisabled => "已停用自動備份",

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
    fn test_language_parse_null_byte_rejection() {
        // Security: Null bytes should be rejected
        assert_eq!(Language::parse("en\0"), None);
        assert_eq!(Language::parse("\0ja"), None);
        assert_eq!(Language::parse("test\0malicious"), None);
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
            app_version()
        );

        // Test Japanese
        assert_eq!(
            get_message(MessageKey::AppVersion, Language::Japanese),
            app_version()
        );

        // Test Simplified Chinese
        assert_eq!(
            get_message(MessageKey::AppVersion, Language::SimplifiedChinese),
            app_version()
        );

        // Test Traditional Chinese
        assert_eq!(
            get_message(MessageKey::AppVersion, Language::TraditionalChinese),
            app_version()
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
