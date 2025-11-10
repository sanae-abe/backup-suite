// Clippy設定はlib.rsと同じ設定を適用
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::if_not_else)]
#![allow(clippy::single_match_else)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::float_cmp)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::format_push_string)]
#![allow(clippy::format_collect)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::unnecessary_debug_formatting)]
#![allow(clippy::incompatible_msrv)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::assigning_clones)]

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use dialoguer::FuzzySelect;
use is_terminal::IsTerminal;
use std::io::{self};
use std::path::PathBuf;

use backup_suite::core::{BackupHistory, BackupRunner, Scheduler};
use backup_suite::i18n::{get_message, Language, MessageKey};
use backup_suite::ui::{
    display_backup_result, display_dashboard, display_history, display_targets, ColorTheme,
};
use backup_suite::{Config, Priority, Target};

// カラー検出機能
fn supports_color() -> bool {
    std::io::stdout().is_terminal()
        && std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM")
            .map(|term| term != "dumb")
            .unwrap_or(true)
}

// カラーコードを返す関数（カラーサポートに応じて切り替え）
fn get_color(color_code: &str, no_color: bool) -> &'static str {
    if no_color || !supports_color() {
        return "";
    }
    match color_code {
        "green" => "\x1b[32m",
        "yellow" => "\x1b[33m",
        "red" => "\x1b[31m",
        "magenta" => "\x1b[35m",
        "gray" => "\x1b[90m",
        "reset" => "\x1b[0m",
        _ => "",
    }
}

#[derive(Parser)]
#[command(name = "backup-suite")]
#[command(about = "Backup Suite - 高速ローカルバックアップツール")]
#[command(version = "1.0.0")]
#[command(disable_help_flag = true)]
#[command(disable_version_flag = true)]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short = 'h', long = "help")]
    help: bool,

    #[arg(short = 'V', long = "version")]
    version: bool,

    #[arg(long = "lang", value_name = "LANG")]
    /// Language (en/ja)
    lang: Option<String>,

    #[arg(long = "no-color", global = true)]
    /// Disable colored output
    no_color: bool,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        /// File or directory path to add (optional - will open file selector if not provided)
        path: Option<PathBuf>,
        #[arg(long, default_value = "medium")]
        priority: String,
        #[arg(long, default_value = "user")]
        category: String,
        #[arg(long)]
        /// Use interactive file selector
        interactive: bool,
        #[arg(long = "exclude")]
        /// Exclude patterns (regex or glob, can be specified multiple times)
        exclude_patterns: Vec<String>,
    },
    #[command(alias = "ls")]
    List {
        #[arg(long)]
        priority: Option<String>,
    },
    Remove {
        /// File or directory path to remove (optional - will show selector if not provided)
        path: Option<PathBuf>,
        #[arg(long)]
        /// Use interactive target selector
        interactive: bool,
    },
    #[command(alias = "rm")]
    Clear {
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        all: bool,
    },
    Run {
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        /// Enable encryption for backup files
        encrypt: bool,
        #[arg(long)]
        /// Password for encryption (will prompt if not provided)
        password: Option<String>,
        #[arg(long)]
        /// Generate a strong random password (use with --encrypt)
        generate_password: bool,
        #[arg(long, default_value = "zstd")]
        /// Compression algorithm: zstd, gzip, none
        compress: String,
        #[arg(long, default_value = "3")]
        /// Compression level (1-22 for zstd, 1-9 for gzip)
        compress_level: i32,
        #[arg(long)]
        /// Enable incremental backup (only changed files)
        incremental: bool,
    },
    Restore {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<PathBuf>,
        #[arg(long)]
        /// Password for decryption (will prompt if not provided and file is encrypted)
        password: Option<String>,
    },
    Cleanup {
        #[arg(long, default_value = "30")]
        days: u32,
        #[arg(long)]
        dry_run: bool,
    },
    Status,
    History {
        #[arg(long, default_value = "7")]
        days: u32,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        /// Show detailed information
        detailed: bool,
    },
    Dashboard,
    Open,
    /// Generate shell completion scripts
    Completion {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
    Schedule {
        #[command(subcommand)]
        action: ScheduleAction,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// AI-driven intelligent backup management
    #[cfg(feature = "ai")]
    Ai {
        #[command(subcommand)]
        action: Option<AiAction>,

        /// Show help for AI commands
        #[arg(short = 'h', long = "help")]
        help: bool,
    },
}

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
enum ScheduleAction {
    Enable {
        #[arg(long)]
        priority: Option<String>,
    },
    Disable {
        #[arg(long)]
        priority: Option<String>,
    },
    Status,
    Setup {
        #[arg(long, default_value = "daily")]
        high: String,
        #[arg(long, default_value = "weekly")]
        medium: String,
        #[arg(long, default_value = "monthly")]
        low: String,
    },
    Help,
}

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
enum ConfigAction {
    /// Set backup destination directory
    SetDestination {
        /// New backup destination path
        path: PathBuf,
    },
    /// Get current backup destination directory
    GetDestination,
    /// Set backup retention days
    SetKeepDays {
        /// Number of days to keep backups (1-3650)
        days: u32,
    },
    /// Get current backup retention days
    GetKeepDays,
    /// Open configuration file in default editor
    Open,
    Help,
}

#[cfg(feature = "ai")]
#[derive(Subcommand)]
enum AiAction {
    /// Detect anomalies in backup history
    Detect {
        #[arg(long, default_value = "7")]
        /// Number of days to analyze
        days: u32,
        #[arg(long, default_value = "table")]
        /// Output format: table, json, detailed
        format: String,
    },
    /// Analyze file importance
    Analyze {
        /// Path to analyze
        path: PathBuf,
        #[arg(long)]
        /// Suggest priority based on importance
        suggest_priority: bool,
        #[arg(long)]
        /// Show detailed analysis
        detailed: bool,
    },
    /// Suggest exclude patterns
    SuggestExclude {
        /// Path to analyze
        path: PathBuf,
        #[arg(long)]
        /// Apply suggestions to config
        apply: bool,
        #[arg(long, default_value = "0.8")]
        /// Minimum confidence (0.0-1.0)
        confidence: f64,
    },
    /// Auto-configure backup settings with AI
    AutoConfigure {
        /// Paths to configure
        paths: Vec<PathBuf>,
        #[arg(long)]
        /// Dry run (show what would be done)
        dry_run: bool,
        #[arg(long)]
        /// Interactive mode (confirm each change)
        interactive: bool,
    },
    Help,
}

fn print_completions<G: Generator>(generator: G, cmd: &mut clap::Command) {
    generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
}

fn select_file_with_fuzzy(prompt: &str) -> Result<Option<PathBuf>> {
    use std::io::BufRead;

    // findコマンドでファイル/ディレクトリ一覧を取得
    let cmd = if cfg!(windows) {
        // Windows: dir /s /b (recursive list)
        "dir /s /b 2>nul"
    } else {
        // Unix: find command with depth limit for better performance
        "find . -maxdepth 3 -type f -o -type d 2>/dev/null | head -1000"
    };

    let output = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(["/C", cmd])
            .output()?
    } else {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()?
    };

    let paths: Vec<String> = std::io::BufReader::new(&output.stdout[..])
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.is_empty())
        .take(1000)
        .collect();

    if paths.is_empty() {
        return Ok(None);
    }

    // dialoguer::FuzzySelectで選択
    let selection = FuzzySelect::new()
        .with_prompt(prompt)
        .items(&paths)
        .default(0)
        .interact_opt()?;

    if let Some(index) = selection {
        let path_str: &str = &paths[index];
        let path = if let Some(stripped) = path_str.strip_prefix("./") {
            PathBuf::from(stripped)
        } else {
            PathBuf::from(path_str)
        };

        // 絶対パスに変換
        let absolute_path = if path.is_absolute() {
            path
        } else {
            std::env::current_dir()?.join(path)
        };

        Ok(Some(absolute_path))
    } else {
        Ok(None)
    }
}

fn select_target_with_fuzzy(config: &Config, lang: Language) -> Result<Option<PathBuf>> {
    if config.targets.is_empty() {
        println!(
            "{}⚠️ {}{}",
            get_color("yellow", false),
            get_message(MessageKey::NoTargetsRegistered, lang),
            get_color("reset", false)
        );
        return Ok(None);
    }

    // バックアップ対象一覧を文字列として生成
    let targets_display: Vec<String> = config
        .targets
        .iter()
        .map(|t| {
            format!(
                "{} [{}] {}",
                t.path.display(),
                match t.priority {
                    Priority::High => "High",
                    Priority::Medium => "Medium",
                    Priority::Low => "Low",
                },
                t.category
            )
        })
        .collect();

    // dialoguer::FuzzySelectで選択
    let selection = FuzzySelect::new()
        .with_prompt("削除するバックアップ対象を選択")
        .items(&targets_display)
        .default(0)
        .interact_opt()?;

    if let Some(index) = selection {
        Ok(Some(config.targets[index].path.clone()))
    } else {
        Ok(None)
    }
}

fn parse_priority(s: &str) -> Result<Priority> {
    match s.to_lowercase().as_str() {
        "high" => Ok(Priority::High),
        "medium" => Ok(Priority::Medium),
        "low" => Ok(Priority::Low),
        _ => Err(anyhow::anyhow!("不明な優先度: {s}")),
    }
}

/// Detect language from CLI argument and environment
fn detect_language(lang_arg: Option<&str>) -> Language {
    if let Some(lang_str) = lang_arg {
        if let Some(lang) = Language::parse(lang_str) {
            return lang;
        }
    }
    Language::detect()
}

/// Display multilingual help
fn print_help(lang: Language) {
    let green = get_color("green", false);
    let yellow = get_color("yellow", false);
    let magenta = get_color("magenta", false);
    let gray = get_color("gray", false);
    let reset = get_color("reset", false);

    println!(
        "{}{}{}",
        green,
        get_message(MessageKey::AppVersion, lang),
        reset
    );
    println!("{}", get_message(MessageKey::AppTitle, lang));
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::UsageExamples, lang)
            .split(':')
            .next()
            .unwrap_or("Usage"),
        reset
    );
    println!("  backup-suite <command> [options]");
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::BasicCommands, lang),
        reset
    );
    println!(
        "  {}{}{}          {}",
        yellow,
        get_message(MessageKey::CmdAdd, lang),
        reset,
        get_message(MessageKey::DescAdd, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::AddPriorityOption, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::AddCategoryOption, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::InteractiveOption, lang)
    );
    println!(
        "  {}{}{}     {}",
        yellow,
        get_message(MessageKey::CmdList, lang),
        reset,
        get_message(MessageKey::DescList, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::ListPriorityOption, lang)
    );
    println!(
        "  {}{}{}       {}",
        yellow,
        get_message(MessageKey::CmdRemove, lang),
        reset,
        get_message(MessageKey::DescRemove, lang)
    );
    println!(
        "  {}{}{}        {}",
        yellow,
        get_message(MessageKey::CmdClear, lang),
        reset,
        get_message(MessageKey::DescClear, lang)
    );
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::ExecutionCommands, lang),
        reset
    );
    println!(
        "  {}{}{}          {}",
        yellow,
        get_message(MessageKey::CmdRun, lang),
        reset,
        get_message(MessageKey::DescRun, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::EncryptOption, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::CompressOption, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::CompressLevel, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::IncrementalOption, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::GeneratePasswordOption, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::PasswordOption, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::DryRunOption, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::PriorityOption, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::CategoryOption, lang)
    );
    println!(
        "  {}{}{}      {}",
        yellow,
        get_message(MessageKey::CmdRestore, lang),
        reset,
        get_message(MessageKey::DescRestore, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::FromOption, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::ToOption, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::RestorePasswordOption, lang)
    );
    println!(
        "  {}{}{}      {}",
        yellow,
        get_message(MessageKey::CmdCleanup, lang),
        reset,
        get_message(MessageKey::DescCleanup, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::DaysOption, lang)
    );
    println!(
        "                 {}",
        get_message(MessageKey::CleanupDryRunOption, lang)
    );
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::InformationCommands, lang),
        reset
    );
    println!(
        "  {}{}{}       {}",
        yellow,
        get_message(MessageKey::CmdStatus, lang),
        reset,
        get_message(MessageKey::DescStatus, lang)
    );
    println!(
        "  {}{}{}      {}",
        yellow,
        get_message(MessageKey::CmdHistory, lang),
        reset,
        get_message(MessageKey::DescHistory, lang)
    );
    println!(
        "  {}{}{}    {}",
        yellow,
        get_message(MessageKey::CmdDashboard, lang),
        reset,
        get_message(MessageKey::DescDashboard, lang)
    );
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::ConfigCommands, lang),
        reset
    );
    println!(
        "  {}{}{}       {}",
        yellow,
        get_message(MessageKey::CmdEnable, lang),
        reset,
        get_message(MessageKey::DescEnable, lang)
    );
    println!(
        "  {}{}{}      {}",
        yellow,
        get_message(MessageKey::CmdDisable, lang),
        reset,
        get_message(MessageKey::DescDisable, lang)
    );
    println!(
        "  {}{}{}     {}",
        yellow,
        get_message(MessageKey::CmdSchedule, lang),
        reset,
        get_message(MessageKey::DescSchedule, lang)
    );
    println!(
        "  {}{}{}       {}",
        yellow,
        get_message(MessageKey::CmdConfig, lang),
        reset,
        get_message(MessageKey::DescConfig, lang)
    );
    println!();

    #[cfg(feature = "ai")]
    {
        println!(
            "{}{}{}",
            magenta,
            get_message(MessageKey::AiCommands, lang),
            reset
        );
        println!(
            "  {}{}{}           {}",
            yellow,
            get_message(MessageKey::CmdAi, lang),
            reset,
            get_message(MessageKey::DescAi, lang)
        );
        println!();
    }

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::UtilityCommands, lang),
        reset
    );
    println!(
        "  {}{}{}         {}",
        yellow,
        get_message(MessageKey::CmdOpen, lang),
        reset,
        get_message(MessageKey::DescOpen, lang)
    );
    println!(
        "  {}{}{}   {}",
        yellow,
        get_message(MessageKey::CmdCompletion, lang),
        reset,
        get_message(MessageKey::DescCompletion, lang)
    );
    println!();

    println!("{}{}", magenta, get_message(MessageKey::Options, lang));
    println!("{}", get_message(MessageKey::HelpOption, lang));
    println!("{}{}", get_message(MessageKey::VersionOption, lang), reset);
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::UsageExamples, lang),
        reset
    );
    println!(
        "  {}{}{}",
        gray,
        get_message(MessageKey::ExampleAddInteractive, lang),
        reset
    );
    println!("  backup-suite add --interactive");
    println!();
    println!(
        "  {}{}{}",
        gray,
        get_message(MessageKey::ExampleRunHigh, lang),
        reset
    );
    println!("  backup-suite run --priority high");
    println!();
    println!(
        "  {}{}{}",
        gray,
        get_message(MessageKey::ExampleEncrypt, lang),
        reset
    );
    println!("  backup-suite run --encrypt --password \"your-password\"");
    println!();
    println!(
        "  {}{}{}",
        gray,
        get_message(MessageKey::ExampleCompress, lang),
        reset
    );
    println!("  backup-suite run --compress zstd --compress-level 3");
    println!();
    println!(
        "  {}{}{}",
        gray,
        get_message(MessageKey::ExampleEncryptCompress, lang),
        reset
    );
    println!("  backup-suite run --encrypt --compress zstd");
    println!();
    println!(
        "  {}{}{}",
        gray,
        get_message(MessageKey::ExampleCleanup, lang),
        reset
    );
    println!("  backup-suite cleanup --days 30 --dry-run");
    println!();
    println!(
        "  {}{}{}",
        gray,
        get_message(MessageKey::ExampleSchedule, lang),
        reset
    );
    println!("  backup-suite schedule setup --high daily --medium weekly");
    println!("  backup-suite schedule enable");
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::DetailedInfo, lang),
        reset
    );
    println!("  {}", get_message(MessageKey::DetailCommand, lang));
    println!("  {}", get_message(MessageKey::ConfigFile, lang));
    println!("  {}", get_message(MessageKey::BackupDestination, lang));
}

/// schedule サブコマンド専用のヘルプを表示
fn print_schedule_help(lang: Language) {
    let green = get_color("green", false);
    let yellow = get_color("yellow", false);
    let magenta = get_color("magenta", false);
    let gray = get_color("gray", false);
    let reset = get_color("reset", false);

    println!(
        "{}{}{}",
        green,
        get_message(MessageKey::ScheduleTitle, lang),
        reset
    );
    println!("{}", get_message(MessageKey::ScheduleDescription, lang));
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::ScheduleUsage, lang),
        reset
    );
    println!(
        "  backup-suite schedule {}",
        get_message(MessageKey::ScheduleCommandPlaceholder, lang)
    );
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::ScheduleCommands, lang),
        reset
    );
    println!(
        "  {}{}{}  {}",
        yellow,
        get_message(MessageKey::ScheduleEnable, lang),
        reset,
        if lang == Language::English {
            "Enable automatic backup"
        } else {
            "自動バックアップを有効化"
        }
    );
    println!(
        "  {}{}{}  {}",
        yellow,
        get_message(MessageKey::ScheduleDisable, lang),
        reset,
        if lang == Language::English {
            "Disable automatic backup"
        } else {
            "自動バックアップを無効化"
        }
    );
    println!(
        "  {}{}{}  {}",
        yellow,
        get_message(MessageKey::ScheduleStatus, lang),
        reset,
        if lang == Language::English {
            "Display current schedule status"
        } else {
            "現在のスケジュール状態を表示"
        }
    );
    println!(
        "  {}{}{}  {}",
        yellow,
        get_message(MessageKey::ScheduleSetup, lang),
        reset,
        if lang == Language::English {
            "Setup schedule frequency"
        } else {
            "スケジュール頻度を設定"
        }
    );
    println!(
        "  {}{}{}  {}",
        yellow,
        get_message(MessageKey::ScheduleHelp, lang),
        reset,
        if lang == Language::English {
            "Display this help"
        } else {
            "このヘルプを表示"
        }
    );
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::ScheduleDetailedOptions, lang),
        reset
    );
    println!(
        "  {}{}{}",
        yellow,
        get_message(MessageKey::ScheduleEnableOption, lang),
        reset
    );
    println!(
        "    {}",
        if lang == Language::English {
            "Enable only specified priority (high/medium/low)"
        } else {
            "指定した優先度のみ有効化 (high/medium/low)"
        }
    );
    println!(
        "  {}{}{}",
        yellow,
        get_message(MessageKey::ScheduleDisableOption, lang),
        reset
    );
    println!(
        "    {}",
        if lang == Language::English {
            "Disable only specified priority"
        } else {
            "指定した優先度のみ無効化"
        }
    );
    println!(
        "  {}{}{}",
        yellow,
        get_message(MessageKey::ScheduleSetupOption, lang),
        reset
    );
    println!(
        "    {}",
        if lang == Language::English {
            "Set execution frequency for each priority (daily/weekly/monthly)"
        } else {
            "各優先度の実行頻度を設定 (daily/weekly/monthly)"
        }
    );
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::UsageExamples, lang),
        reset
    );
    println!(
        "  {}{}{}",
        gray,
        if lang == Language::English {
            "# Enable all automatic backups"
        } else {
            "# 全ての自動バックアップを有効化"
        },
        reset
    );
    println!("  backup-suite schedule enable");
    println!();
    println!(
        "  {}{}{}",
        gray,
        if lang == Language::English {
            "# Enable high priority only"
        } else {
            "# 高優先度のみ有効化"
        },
        reset
    );
    println!("  backup-suite schedule enable --priority high");
    println!();
    println!(
        "  {}{}{}",
        gray,
        if lang == Language::English {
            "# Setup schedule frequency"
        } else {
            "# スケジュール頻度を設定"
        },
        reset
    );
    println!("  backup-suite schedule setup --high daily --medium weekly");
    println!();
    println!(
        "  {}{}{}",
        gray,
        if lang == Language::English {
            "# Check current configuration"
        } else {
            "# 現在の設定状況を確認"
        },
        reset
    );
    println!("  backup-suite schedule status");
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::ScheduleFrequencies, lang),
        reset
    );
    println!(
        "  {}{}",
        yellow,
        get_message(MessageKey::ScheduleDaily, lang)
    );
    println!(
        "  {}{}",
        yellow,
        get_message(MessageKey::ScheduleWeekly, lang)
    );
    println!(
        "  {}{}{}",
        yellow,
        get_message(MessageKey::ScheduleMonthly, lang),
        reset
    );
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::ScheduleTips, lang),
        reset
    );
    println!("{}", get_message(MessageKey::ScheduleTip1, lang));
    println!("{}", get_message(MessageKey::ScheduleTip2, lang));
    println!("{}", get_message(MessageKey::ScheduleTip3, lang));
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::DetailedInfo, lang),
        reset
    );
    println!(
        "  {}: backup-suite --help",
        if lang == Language::English {
            "Main help"
        } else {
            "メインヘルプ"
        }
    );
    println!(
        "  {}: ~/.config/backup-suite/config.toml",
        if lang == Language::English {
            "Configuration file"
        } else {
            "設定ファイル"
        }
    );
}

/// AI サブコマンド専用のヘルプを表示
#[cfg(feature = "ai")]
fn print_ai_help(lang: Language) {
    let magenta = get_color("magenta", false);
    let yellow = get_color("yellow", false);
    let reset = get_color("reset", false);

    // Title
    println!(
        "{}{} {}{}",
        magenta,
        get_message(MessageKey::AiCommands, lang),
        if lang == Language::English {
            "Help"
        } else if lang == Language::Japanese {
            "ヘルプ"
        } else if lang == Language::SimplifiedChinese {
            "帮助"
        } else {
            "說明"
        },
        reset
    );
    println!();

    // Commands
    println!(
        "  {}detect{}           {}",
        yellow,
        reset,
        get_message(MessageKey::DescAiDetect, lang)
    );
    println!(
        "  {}analyze{}          {}",
        yellow,
        reset,
        get_message(MessageKey::DescAiAnalyze, lang)
    );
    println!(
        "  {}suggest-exclude{}  {}",
        yellow,
        reset,
        get_message(MessageKey::DescAiSuggestExclude, lang)
    );
    println!(
        "  {}auto-configure{}   {}",
        yellow,
        reset,
        get_message(MessageKey::DescAiAutoConfigure, lang)
    );
    println!();

    // Examples
    println!(
        "{}{}:{}",
        magenta,
        get_message(MessageKey::UsageExamples, lang)
            .split(':')
            .next()
            .unwrap_or("Examples"),
        reset
    );
    println!("  {}", get_message(MessageKey::ExampleAiDetect, lang));
    println!("  backup-suite ai detect --days 7");
    println!();
    println!("  {}", get_message(MessageKey::ExampleAiAnalyze, lang));
    println!("  backup-suite ai analyze /path/to/dir");
    println!();
    println!(
        "  {}",
        get_message(MessageKey::ExampleAiSuggestExclude, lang)
    );
    println!("  backup-suite ai suggest-exclude /path/to/dir");
}

/// config サブコマンド専用のヘルプを表示
fn print_config_help(lang: Language) {
    let green = get_color("green", false);
    let yellow = get_color("yellow", false);
    let magenta = get_color("magenta", false);
    let gray = get_color("gray", false);
    let reset = get_color("reset", false);

    println!(
        "{}{}{}",
        green,
        get_message(MessageKey::ConfigTitle, lang),
        reset
    );
    println!("{}", get_message(MessageKey::ConfigDescription, lang));
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::ConfigUsage, lang),
        reset
    );
    println!(
        "  backup-suite config {} {}",
        get_message(MessageKey::ConfigCommandPlaceholder, lang),
        get_message(MessageKey::ConfigArgsPlaceholder, lang)
    );
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::ConfigCommands, lang),
        reset
    );
    println!(
        "  {}{}{}",
        yellow,
        get_message(MessageKey::ConfigSetDestination, lang),
        reset
    );
    println!(
        "  {}{}{}",
        yellow,
        get_message(MessageKey::ConfigGetDestination, lang),
        reset
    );
    println!(
        "  {}{}{}",
        yellow,
        get_message(MessageKey::ConfigSetKeepDays, lang),
        reset
    );
    println!(
        "  {}{}{}",
        yellow,
        get_message(MessageKey::ConfigGetKeepDays, lang),
        reset
    );
    println!(
        "  {}{}{}",
        yellow,
        get_message(MessageKey::ConfigOpen, lang),
        reset
    );
    println!(
        "  {}{}{}",
        yellow,
        get_message(MessageKey::ConfigHelp, lang),
        reset
    );
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::UsageExamples, lang),
        reset
    );
    println!(
        "  {}{}{}",
        gray,
        get_message(MessageKey::ConfigExampleExternal, lang),
        reset
    );
    println!("  backup-suite config set-destination /Volumes/ExternalHDD/backups");
    println!();
    println!(
        "  {}{}{}",
        gray,
        get_message(MessageKey::ConfigExampleGetDest, lang),
        reset
    );
    println!("  backup-suite config get-destination");
    println!();
    println!(
        "  {}{}{}",
        gray,
        get_message(MessageKey::ConfigExampleSetDays, lang),
        reset
    );
    println!("  backup-suite config set-keep-days 60");
    println!();
    println!(
        "  {}{}{}",
        gray,
        get_message(MessageKey::ConfigExampleOpen, lang),
        reset
    );
    println!("  backup-suite config open");
    println!();
    println!(
        "  {}{}{}",
        gray,
        get_message(MessageKey::ConfigExampleTilde, lang),
        reset
    );
    println!("  backup-suite config set-destination ~/Documents/backups");
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::ScheduleTips, lang),
        reset
    );
    println!("{}", get_message(MessageKey::ConfigTip1, lang));
    println!("{}", get_message(MessageKey::ConfigTip2, lang));
    println!("{}", get_message(MessageKey::ConfigTip3, lang));
    println!();

    println!(
        "{}{}{}",
        magenta,
        get_message(MessageKey::DetailedInfo, lang),
        reset
    );
    println!(
        "  {}: backup-suite --help",
        if lang == Language::English {
            "Main help"
        } else {
            "メインヘルプ"
        }
    );
    println!(
        "  {}: ~/.config/backup-suite/config.toml",
        if lang == Language::English {
            "Configuration file"
        } else {
            "設定ファイル"
        }
    );
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Detect language from CLI arg or environment
    let lang = detect_language(cli.lang.as_deref());

    // --help フラグの処理
    if cli.help {
        print_help(lang);
        return Ok(());
    }

    // --version フラグの処理
    if cli.version {
        println!(
            "{}{}{}",
            get_color("green", cli.no_color),
            get_message(MessageKey::AppVersion, lang),
            get_color("reset", cli.no_color)
        );
        println!("{}", get_message(MessageKey::RustFastTypeSafe, lang));
        return Ok(());
    }

    match cli.command {
        Some(Commands::Add {
            path,
            priority,
            category,
            interactive,
            exclude_patterns,
        }) => {
            let priority = parse_priority(&priority)?;

            // パスを決定（pathが指定されていない場合、またはinteractiveフラグが立っている場合はskin選択）
            let target_path = if let Some(p) = path {
                if interactive {
                    match select_file_with_fuzzy("追加するファイル/ディレクトリを選択: ")?
                    {
                        Some(selected_path) => selected_path,
                        None => {
                            println!(
                                "{}⚠️ {}{}",
                                get_color("yellow", false),
                                get_message(MessageKey::SelectionCancelled, lang),
                                get_color("reset", false)
                            );
                            return Ok(());
                        }
                    }
                } else {
                    p
                }
            } else {
                match select_file_with_fuzzy("追加するファイル/ディレクトリを選択: ")?
                {
                    Some(selected_path) => selected_path,
                    None => {
                        println!(
                            "{}⚠️ {}{}",
                            get_color("yellow", false),
                            get_message(MessageKey::SelectionCancelled, lang),
                            get_color("reset", false)
                        );
                        return Ok(());
                    }
                }
            };

            // ファイル/ディレクトリの存在確認
            if !target_path.exists() {
                println!(
                    "{}❌ {}{}: {}: {:?}",
                    get_color("red", false),
                    get_message(MessageKey::Error, lang),
                    get_color("reset", false),
                    get_message(MessageKey::PathNotExists, lang),
                    target_path
                );
                return Ok(());
            }

            let mut config = Config::load()?;
            let mut target = Target::new(target_path.clone(), priority, category);

            // 除外パターンを追加
            if !exclude_patterns.is_empty() {
                target.exclude_patterns = exclude_patterns.clone();
                println!(
                    "{}📝 除外パターン: {}{}",
                    get_color("gray", false),
                    exclude_patterns.join(", "),
                    get_color("reset", false)
                );
            }

            if config.add_target(target) {
                config.save()?;
                println!(
                    "{}✅ {}{}: {:?}",
                    get_color("green", false),
                    get_message(MessageKey::Added, lang),
                    get_color("reset", false),
                    target_path
                );
            }
        }
        Some(Commands::List { priority }) => {
            let config = Config::load()?;
            let theme = ColorTheme::auto();

            let targets = if let Some(p) = priority {
                let prio = parse_priority(&p)?;
                config.filter_by_priority(&prio)
            } else {
                config.targets.iter().collect()
            };

            display_targets(
                &targets.iter().map(|&t| t.clone()).collect::<Vec<_>>(),
                &theme,
            );
        }
        Some(Commands::Remove { path, interactive }) => {
            let mut config = Config::load()?;

            // パスを決定（pathが指定されていない場合、またはinteractiveフラグが立っている場合はskin選択）
            let target_path = if let Some(p) = path {
                if interactive {
                    match select_target_with_fuzzy(&config, lang)? {
                        Some(selected_path) => selected_path,
                        None => {
                            println!(
                                "{}⚠️ {}{}",
                                get_color("yellow", false),
                                get_message(MessageKey::SelectionCancelled, lang),
                                get_color("reset", false)
                            );
                            return Ok(());
                        }
                    }
                } else {
                    p
                }
            } else {
                match select_target_with_fuzzy(&config, lang)? {
                    Some(selected_path) => selected_path,
                    None => {
                        println!(
                            "{}⚠️ {}{}",
                            get_color("yellow", false),
                            get_message(MessageKey::SelectionCancelled, lang),
                            get_color("reset", false)
                        );
                        return Ok(());
                    }
                }
            };

            if config.remove_target(&target_path) {
                config.save()?;
                println!(
                    "{}✅ {}{}: {:?}",
                    get_color("green", false),
                    get_message(MessageKey::Removed, lang),
                    get_color("reset", false),
                    target_path
                );
            } else {
                println!(
                    "{}❌ {}{}: {:?}",
                    get_color("red", false),
                    get_message(MessageKey::NotInBackupConfig, lang),
                    get_color("reset", false),
                    target_path
                );
            }
        }
        Some(Commands::Clear { priority, all }) => {
            let mut config = Config::load()?;
            let before = config.targets.len();
            if all {
                config.targets.clear();
            } else if let Some(p) = priority {
                let prio = parse_priority(&p)?;
                config.targets.retain(|t| t.priority != prio);
            } else {
                println!(
                    "{}❌ {}{}",
                    get_color("red", false),
                    get_message(MessageKey::SpecifyPriorityOrAll, lang),
                    get_color("reset", false)
                );
                return Ok(());
            }
            let removed = before - config.targets.len();
            config.save()?;
            println!(
                "{}✅ {} {}{}",
                get_color("green", false),
                removed,
                get_message(MessageKey::CountDeleted, lang),
                get_color("reset", false)
            );
        }
        Some(Commands::Run {
            priority,
            category,
            dry_run,
            encrypt,
            password,
            generate_password,
            compress,
            compress_level,
            incremental,
        }) => {
            let priority = priority.as_ref().map(|s| parse_priority(s)).transpose()?;
            let config = Config::load()?;
            let theme = ColorTheme::auto();

            // 圧縮タイプを変換（表示用に先に実行）
            use backup_suite::compression::CompressionType;
            let compression_type = match compress.as_str() {
                "zstd" => CompressionType::Zstd,
                "gzip" => CompressionType::Gzip,
                "none" => CompressionType::None,
                _ => CompressionType::Zstd,
            };

            // 暗号化・圧縮オプションの表示
            let mut options_info: Vec<String> = Vec::new();
            if dry_run {
                options_info.push(get_message(MessageKey::DryRun, lang).to_string());
            }
            if let Some(ref cat) = category {
                options_info.push(format!(
                    "{}: {}",
                    get_message(MessageKey::Category, lang),
                    cat
                ));
            }
            if encrypt {
                options_info.push(get_message(MessageKey::Encryption, lang).to_string());
            }
            // 実際の圧縮タイプに基づいて表示
            match compression_type {
                CompressionType::Zstd => options_info.push(format!(
                    "{}: zstd",
                    get_message(MessageKey::Compression, lang)
                )),
                CompressionType::Gzip => options_info.push(format!(
                    "{}: gzip",
                    get_message(MessageKey::Compression, lang)
                )),
                CompressionType::None => {} // 無圧縮の場合は表示しない
            }

            let options_str = if options_info.is_empty() {
                String::new()
            } else {
                format!("（{}）", options_info.join("、"))
            };

            println!(
                "{}{}{}{}",
                get_color("green", false),
                get_message(MessageKey::BackupRunning, lang),
                options_str,
                get_color("reset", false)
            );

            // BackupRunnerを構築
            let mut runner = BackupRunner::new(config, dry_run);

            // 圧縮設定
            runner = runner.with_compression(compression_type, compress_level);

            // 増分バックアップ設定
            if incremental {
                runner = runner.with_incremental(true);
            }

            // 暗号化設定
            if encrypt {
                use backup_suite::crypto::{PasswordPolicy, PasswordStrength};

                let pwd = if generate_password {
                    // 強力なパスワードを自動生成
                    let policy = PasswordPolicy::default();
                    let generated = policy.generate_password(20);
                    let pwd_str = generated.to_string();

                    println!(
                        "{}🔐 {}{}: {}",
                        get_color("green", false),
                        get_message(MessageKey::EncryptionPassword, lang),
                        get_color("reset", false),
                        pwd_str
                    );
                    println!(
                        "{}{}{}",
                        get_color("yellow", false),
                        get_message(MessageKey::SavePasswordSecurely, lang),
                        get_color("reset", false)
                    );

                    pwd_str
                } else if let Some(p) = password {
                    // コマンドラインから提供されたパスワードの強度チェック
                    let policy = PasswordPolicy::default();
                    let strength = policy.evaluate(&p);

                    if !matches!(strength, PasswordStrength::Strong) {
                        println!(
                            "{}{}{}",
                            get_color("yellow", false),
                            policy.display_report(&p),
                            get_color("reset", false)
                        );
                    } else {
                        println!(
                            "{}✅ Password Strength: {}{}",
                            get_color("green", false),
                            strength.display(),
                            get_color("reset", false)
                        );
                    }

                    p
                } else {
                    // パスワードプロンプト（dialoguerを使用して隠し入力）
                    use dialoguer::Password;

                    let input = Password::new()
                        .with_prompt(format!(
                            "{}{}{}",
                            get_color("yellow", false),
                            get_message(MessageKey::EncryptionPassword, lang),
                            get_color("reset", false)
                        ))
                        .interact()?;

                    // パスワード強度チェック
                    let policy = PasswordPolicy::default();
                    let strength = policy.evaluate(&input);

                    if !matches!(strength, PasswordStrength::Strong) {
                        println!(
                            "{}{}{}",
                            get_color("yellow", false),
                            policy.display_report(&input),
                            get_color("reset", false)
                        );
                    } else {
                        println!(
                            "{}✅ Password Strength: {}{}",
                            get_color("green", false),
                            strength.display(),
                            get_color("reset", false)
                        );
                    }

                    input
                };
                runner = runner.with_encryption(pwd);
            }

            let result = runner.run(priority.as_ref(), category.as_deref())?;

            if !dry_run {
                display_backup_result(
                    result.total_files,
                    result.successful,
                    result.failed,
                    result.total_bytes,
                    &theme,
                );

                if !result.errors.is_empty() {
                    println!(
                        "\n{}⚠️ {}{}",
                        get_color("yellow", false),
                        get_message(MessageKey::ErrorDetails, lang),
                        get_color("reset", false)
                    );
                    for (i, error) in result.errors.iter().enumerate() {
                        println!("  {}. {}", i + 1, error);
                    }
                }
            } else {
                println!(
                    "{}📋 {}{}: {} {}",
                    get_color("gray", false),
                    get_message(MessageKey::Detected, lang),
                    get_color("reset", false),
                    result.total_files,
                    get_message(MessageKey::Files, lang)
                );
            }
        }
        Some(Commands::Restore { from, to, password }) => {
            use backup_suite::RestoreEngine;

            let dirs = BackupHistory::list_backup_dirs()?;
            if dirs.is_empty() {
                println!(
                    "{}❌ {}{}",
                    get_color("red", false),
                    get_message(MessageKey::NoBackups, lang),
                    get_color("reset", false)
                );
                return Ok(());
            }

            let backup_dir = if let Some(pattern) = from {
                dirs.iter()
                    .find(|d| d.to_string_lossy().contains(&pattern))
                    .ok_or_else(|| anyhow::anyhow!("バックアップが見つかりません: {pattern}"))?
            } else {
                &dirs[0] // 最新
            };

            // バックアップ名をディレクトリ名から取得
            let backup_name = backup_dir
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| anyhow::anyhow!("バックアップ名取得失敗"))?;

            // 復元先ディレクトリ: 指定パス or ./.restored の配下にバックアップ名ディレクトリを作成
            let base_dest = to.unwrap_or_else(|| PathBuf::from("./.restored"));
            let dest = base_dest.join(backup_name);

            println!(
                "{}🔄 {}{}: {:?} → {:?}",
                get_color("green", false),
                get_message(MessageKey::RestoreStart, lang),
                get_color("reset", false),
                backup_dir,
                dest
            );

            // RestoreEngineを使用して復元
            let mut engine = RestoreEngine::new(false);
            let result = engine.restore(backup_dir, &dest, password.as_deref())?;

            println!(
                "\n{}✅ {} {:?}{}",
                get_color("green", false),
                get_message(MessageKey::RestoredSuccess, lang),
                dest,
                get_color("reset", false)
            );
            println!(
                "  {}: {} ({} {} {})",
                get_message(MessageKey::RestoredFileCount, lang),
                result.restored,
                get_message(MessageKey::EncryptedLabel, lang),
                result.encrypted_files,
                get_message(MessageKey::Files, lang)
            );

            if result.failed > 0 {
                println!(
                    "{}⚠️ {} {}{}",
                    get_color("yellow", false),
                    result.failed,
                    get_message(MessageKey::CountDeleted, lang),
                    get_color("reset", false)
                );
                for error in &result.errors {
                    println!("  - {error}");
                }
            }
        }
        Some(Commands::Cleanup { days, dry_run }) => {
            use backup_suite::{CleanupEngine, CleanupPolicy};

            let policy = CleanupPolicy::retention_days(days);
            let mut engine = CleanupEngine::new(policy, dry_run);
            let result = engine.cleanup()?;

            println!(
                "{}✅ {} {}{}{}",
                get_color("green", false),
                result.deleted,
                get_message(MessageKey::CountDeleted, lang),
                if dry_run {
                    get_message(MessageKey::DryRunParens, lang)
                } else {
                    ""
                },
                get_color("reset", false)
            );

            if result.freed_bytes > 0 {
                let freed_mb = result.freed_bytes as f64 / 1024.0 / 1024.0;
                println!(
                    "  {}解放容量: {:.2} MB{}",
                    get_color("gray", false),
                    freed_mb,
                    get_color("reset", false)
                );
            }

            if !result.errors.is_empty() {
                println!(
                    "{}⚠️ エラー: {}件{}",
                    get_color("yellow", false),
                    result.errors.len(),
                    get_color("reset", false)
                );
                for error in &result.errors {
                    println!("  - {error}");
                }
            }
        }
        Some(Commands::Status) => {
            let config = Config::load()?;
            println!(
                "{}📊 {}{}",
                get_color("magenta", false),
                get_message(MessageKey::StatusTitle, lang),
                get_color("reset", false)
            );
            println!(
                "  {}: {:?}",
                get_message(MessageKey::Destination, lang),
                config.backup.destination
            );
            println!(
                "  {}: {}",
                get_message(MessageKey::Targets, lang),
                config.targets.len()
            );
            println!(
                "    {}{}{}: {}",
                get_color("red", false),
                get_message(MessageKey::High, lang),
                get_color("reset", false),
                config.filter_by_priority(&Priority::High).len()
            );
            println!(
                "    {}{}{}: {}",
                get_color("yellow", false),
                get_message(MessageKey::Medium, lang),
                get_color("reset", false),
                config.filter_by_priority(&Priority::Medium).len()
            );
            println!(
                "    {}{}{}: {}",
                get_color("gray", false),
                get_message(MessageKey::Low, lang),
                get_color("reset", false),
                config.filter_by_priority(&Priority::Low).len()
            );
        }
        Some(Commands::History {
            days,
            priority,
            category,
            detailed,
        }) => {
            let mut history = BackupHistory::filter_by_days(days)?;
            let theme = ColorTheme::auto();

            // 優先度フィルタ適用
            if let Some(p_str) = priority {
                let prio = parse_priority(&p_str)?;
                let filtered = BackupHistory::filter_by_priority(&history, &prio);
                history = filtered.into_iter().cloned().collect();
            }

            // カテゴリフィルタ適用
            if let Some(ref cat) = category {
                let filtered = BackupHistory::filter_by_category(&history, cat);
                history = filtered.into_iter().cloned().collect();
            }

            println!(
                "\n{}📜 {}{}（{}{}）",
                get_color("magenta", false),
                get_message(MessageKey::BackupHistory, lang),
                get_color("reset", false),
                days,
                get_message(MessageKey::Days, lang)
            );

            if detailed {
                // 詳細表示
                for entry in &history {
                    println!(
                        "\n{}{}{}",
                        get_color("green", false),
                        "=".repeat(60),
                        get_color("reset", false)
                    );
                    println!(
                        "🕒 {}: {}",
                        get_message(MessageKey::StatusTitle, lang),
                        entry.timestamp.format("%Y-%m-%d %H:%M:%S")
                    );
                    println!("📁 パス: {:?}", entry.backup_dir);
                    if let Some(ref cat) = entry.category {
                        println!("🏷️  カテゴリ: {cat}");
                    }
                    if let Some(ref prio) = entry.priority {
                        println!("⚡ 優先度: {prio:?}");
                    }
                    println!("📊 ステータス: {:?}", entry.status);
                    println!("📦 ファイル数: {}", entry.total_files);
                    println!(
                        "💾 サイズ: {:.2} MB",
                        entry.total_bytes as f64 / 1024.0 / 1024.0
                    );
                    if entry.compressed {
                        println!("🗜️  圧縮: 有効");
                    }
                    if entry.encrypted {
                        println!("🔒 暗号化: 有効");
                    }
                    if entry.duration_ms > 0 {
                        println!("⏱️  処理時間: {:.2}秒", entry.duration_ms as f64 / 1000.0);
                    }
                    if let Some(ref err) = entry.error_message {
                        println!(
                            "{}❌ エラー: {}{}",
                            get_color("red", false),
                            err,
                            get_color("reset", false)
                        );
                    }
                }
            } else {
                // テーブル表示
                display_history(&history, &theme);
            }
        }
        Some(Commands::Dashboard) => {
            display_dashboard()?;
        }
        Some(Commands::Open) => {
            let config = Config::load()?;
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .arg(&config.backup.destination)
                    .spawn()?;
            }
            println!(
                "{}📂 {}{}: {:?}",
                get_color("green", false),
                get_message(MessageKey::OpenDirectory, lang),
                get_color("reset", false),
                config.backup.destination
            );
        }
        Some(Commands::Completion { shell }) => {
            let mut cmd = Cli::command();
            print_completions(shell, &mut cmd);
        }
        Some(Commands::Schedule { action }) => {
            let mut config = Config::load()?;
            match action {
                ScheduleAction::Enable { priority } => {
                    config.schedule.enabled = true;
                    config.save()?;

                    let scheduler = Scheduler::new(config)?;

                    if let Some(p) = priority {
                        let prio = parse_priority(&p)?;
                        scheduler.setup_priority(&prio)?;
                        scheduler.enable_priority(&prio)?;
                        println!(
                            "{}✅ {}{} ({})",
                            get_color("green", false),
                            get_message(MessageKey::AutoBackupEnabled, lang),
                            get_color("reset", false),
                            p
                        );
                    } else {
                        scheduler.setup_all()?;
                        scheduler.enable_all()?;
                        println!(
                            "{}✅ {}{}",
                            get_color("green", false),
                            get_message(MessageKey::AutoBackupEnabled, lang),
                            get_color("reset", false)
                        );
                    }
                }
                ScheduleAction::Disable { priority } => {
                    let scheduler = Scheduler::new(Config::load()?)?;

                    if let Some(p) = priority {
                        let prio = parse_priority(&p)?;
                        scheduler.disable_priority(&prio)?;
                        println!(
                            "{}⏸️  {}{} ({})",
                            get_color("yellow", false),
                            get_message(MessageKey::AutoBackupDisabled, lang),
                            get_color("reset", false),
                            p
                        );
                    } else {
                        config.schedule.enabled = false;
                        config.save()?;
                        scheduler.disable_all()?;
                        println!(
                            "{}⏸️  {}{}",
                            get_color("yellow", false),
                            get_message(MessageKey::AutoBackupDisabled, lang),
                            get_color("reset", false)
                        );
                    }
                }
                ScheduleAction::Status => {
                    println!(
                        "{}📅 {}{}",
                        get_color("magenta", false),
                        get_message(MessageKey::ScheduleSettings, lang),
                        get_color("reset", false)
                    );
                    println!(
                        "  {}: {}",
                        get_message(MessageKey::Enabled, lang),
                        if config.schedule.enabled {
                            "✅"
                        } else {
                            "❌"
                        }
                    );
                    println!(
                        "  {}: {}",
                        get_message(MessageKey::HighPriority, lang),
                        config.schedule.high_frequency
                    );
                    println!(
                        "  {}: {}",
                        get_message(MessageKey::MediumPriority, lang),
                        config.schedule.medium_frequency
                    );
                    println!(
                        "  {}: {}",
                        get_message(MessageKey::LowPriority, lang),
                        config.schedule.low_frequency
                    );

                    // 実際の状態確認
                    let scheduler = Scheduler::new(config)?;
                    let status = scheduler.check_status()?;

                    println!();
                    println!(
                        "{}📋 {}{}",
                        get_color("magenta", false),
                        get_message(MessageKey::ActualScheduleStatus, lang),
                        get_color("reset", false)
                    );

                    println!(
                        "  high: {}{}{}",
                        if status.high_enabled {
                            get_color("green", false)
                        } else {
                            get_color("red", false)
                        },
                        if status.high_enabled { "✅ " } else { "❌ " },
                        if status.high_enabled {
                            get_message(MessageKey::Enabled, lang)
                        } else {
                            get_message(MessageKey::Disabled, lang)
                        }
                    );
                    println!("{}", get_color("reset", false));

                    println!(
                        "  medium: {}{}{}",
                        if status.medium_enabled {
                            get_color("green", false)
                        } else {
                            get_color("red", false)
                        },
                        if status.medium_enabled {
                            "✅ "
                        } else {
                            "❌ "
                        },
                        if status.medium_enabled {
                            get_message(MessageKey::Enabled, lang)
                        } else {
                            get_message(MessageKey::Disabled, lang)
                        }
                    );
                    println!("{}", get_color("reset", false));

                    println!(
                        "  low: {}{}{}",
                        if status.low_enabled {
                            get_color("green", false)
                        } else {
                            get_color("red", false)
                        },
                        if status.low_enabled { "✅ " } else { "❌ " },
                        if status.low_enabled {
                            get_message(MessageKey::Enabled, lang)
                        } else {
                            get_message(MessageKey::Disabled, lang)
                        }
                    );
                    println!("{}", get_color("reset", false));
                }
                ScheduleAction::Setup { high, medium, low } => {
                    config.schedule.high_frequency = high.clone();
                    config.schedule.medium_frequency = medium.clone();
                    config.schedule.low_frequency = low.clone();
                    config.save()?;

                    if config.schedule.enabled {
                        let scheduler = Scheduler::new(config)?;
                        scheduler.setup_all()?;
                        println!(
                            "{}✅ {}{}",
                            get_color("green", false),
                            get_message(MessageKey::ScheduleUpdated, lang),
                            get_color("reset", false)
                        );
                    } else {
                        println!(
                            "{}✅ {}{}",
                            get_color("green", false),
                            get_message(MessageKey::ScheduleUpdatedEnableLater, lang),
                            get_color("reset", false)
                        );
                    }

                    println!(
                        "  {}: {}",
                        get_message(MessageKey::HighPriority, lang),
                        high
                    );
                    println!(
                        "  {}: {}",
                        get_message(MessageKey::MediumPriority, lang),
                        medium
                    );
                    println!("  {}: {}", get_message(MessageKey::LowPriority, lang), low);
                }
                ScheduleAction::Help => {
                    print_schedule_help(lang);
                }
            }
        }
        Some(Commands::Config { action }) => {
            let mut config = Config::load()?;
            match action {
                ConfigAction::SetDestination { path } => {
                    // パスの正規化（チルダ展開など）
                    let path = {
                        let path_str = path.to_string_lossy();
                        if path_str.starts_with("~") {
                            let home = dirs::home_dir().ok_or_else(|| {
                                anyhow::anyhow!("ホームディレクトリが見つかりません")
                            })?;
                            let relative =
                                path_str.strip_prefix("~").unwrap().trim_start_matches('/');
                            home.join(relative)
                        } else {
                            path
                        }
                    };

                    // ディレクトリが存在しない場合は作成を試みる
                    if !path.exists() {
                        println!(
                            "{}📁 {}{}: {:?}",
                            get_color("yellow", false),
                            get_message(MessageKey::DirectoryNotExists, lang),
                            get_color("reset", false),
                            path
                        );
                        std::fs::create_dir_all(&path)
                            .map_err(|e| anyhow::anyhow!("ディレクトリ作成失敗: {path:?} - {e}"))?;
                    }

                    // 書き込み権限を確認
                    use backup_suite::security::check_write_permission;
                    check_write_permission(&path)
                        .map_err(|e| anyhow::anyhow!("書き込み権限エラー: {path:?} - {e}"))?;

                    // 設定を更新
                    let old_destination = config.backup.destination.clone();
                    config.backup.destination = path.clone();
                    config.save()?;

                    println!(
                        "{}✅ {}{}",
                        get_color("green", false),
                        get_message(MessageKey::DestinationChanged, lang),
                        get_color("reset", false)
                    );
                    println!(
                        "  {}: {:?}",
                        get_message(MessageKey::Before, lang),
                        old_destination
                    );
                    println!("  {}: {:?}", get_message(MessageKey::After, lang), path);
                }
                ConfigAction::GetDestination => {
                    println!(
                        "{}📁 {}{}",
                        get_color("magenta", false),
                        get_message(MessageKey::CurrentDestination, lang),
                        get_color("reset", false)
                    );
                    println!("  {:?}", config.backup.destination);
                }
                ConfigAction::SetKeepDays { days } => {
                    if days == 0 || days > 3650 {
                        println!(
                            "{}❌ {}{}: {} {}）",
                            get_color("red", false),
                            get_message(MessageKey::Error, lang),
                            get_color("reset", false),
                            get_message(MessageKey::KeepDaysOutOfRange, lang),
                            days
                        );
                        return Ok(());
                    }

                    let old_days = config.backup.keep_days;
                    config.backup.keep_days = days;
                    config.save()?;

                    println!(
                        "{}✅ {}{}",
                        get_color("green", false),
                        get_message(MessageKey::KeepDaysChanged, lang),
                        get_color("reset", false)
                    );
                    println!(
                        "  {}: {}{}",
                        get_message(MessageKey::Before, lang),
                        old_days,
                        get_message(MessageKey::DaysUnit, lang)
                    );
                    println!(
                        "  {}: {}{}",
                        get_message(MessageKey::After, lang),
                        days,
                        get_message(MessageKey::DaysUnit, lang)
                    );
                }
                ConfigAction::GetKeepDays => {
                    println!(
                        "{}📅 {}{}",
                        get_color("magenta", false),
                        get_message(MessageKey::CurrentKeepDays, lang),
                        get_color("reset", false)
                    );
                    println!(
                        "  {}{}",
                        config.backup.keep_days,
                        get_message(MessageKey::DaysUnit, lang)
                    );
                }
                ConfigAction::Open => {
                    let config_path = Config::config_path()?;

                    println!(
                        "{}📝 {}{}: {:?}",
                        get_color("green", false),
                        get_message(MessageKey::OpeningConfigFile, lang),
                        get_color("reset", false),
                        config_path
                    );

                    // エディタを決定（環境変数 → デフォルト）
                    #[cfg(not(target_os = "windows"))]
                    let editor = std::env::var("EDITOR")
                        .or_else(|_| std::env::var("VISUAL"))
                        .unwrap_or_else(|_| {
                            // macOSではopenコマンドでデフォルトエディタを使用
                            #[cfg(target_os = "macos")]
                            {
                                "open".to_string()
                            }
                            #[cfg(not(target_os = "macos"))]
                            {
                                "nano".to_string()
                            }
                        });

                    #[cfg(target_os = "windows")]
                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "notepad".to_string());

                    // エディタで開く
                    let status = std::process::Command::new(&editor)
                        .arg(&config_path)
                        .status()
                        .context(format!("エディタ起動失敗: {editor}"))?;

                    if !status.success() {
                        println!(
                            "{}⚠️ {}{}",
                            get_color("yellow", false),
                            get_message(MessageKey::EditorDidNotExitCleanly, lang),
                            get_color("reset", false)
                        );
                    }
                }
                ConfigAction::Help => {
                    print_config_help(lang);
                }
            }
        }
        #[cfg(feature = "ai")]
        Some(Commands::Ai { action, help }) => {
            use backup_suite::ai::anomaly::AnomalyDetector;
            use backup_suite::ai::recommendation::{
                ExcludeRecommendationEngine, ImportanceEvaluator,
            };
            use backup_suite::ai::types::BackupSize;
            use comfy_table::{Cell, Table};

            // --help フラグまたは引数なしの場合はヘルプを表示
            if help || action.is_none() {
                print_ai_help(lang);
                return Ok(());
            }

            let action = action.unwrap();

            match action {
                AiAction::Detect { days, format } => {
                    println!(
                        "{}{}{}",
                        get_color("magenta", false),
                        get_message(MessageKey::AiDetectTitle, lang),
                        get_color("reset", false)
                    );
                    println!(
                        "{}{}{}...\n",
                        if lang == Language::Japanese {
                            "過去"
                        } else {
                            "Analyzing last"
                        },
                        days,
                        if lang == Language::Japanese {
                            "日間のバックアップを分析中"
                        } else {
                            " days of backups"
                        }
                    );

                    let detector = AnomalyDetector::default_detector();
                    let history = BackupHistory::filter_by_days(days)?;

                    let current_size =
                        BackupSize::new(history.last().map(|h| h.total_bytes).unwrap_or(0));

                    match detector.detect_size_anomaly(&history, current_size) {
                        Ok(Some(result)) if result.is_anomaly() => match format.as_str() {
                            "json" => {
                                let json_output = serde_json::json!({
                                    "anomaly_detected": true,
                                    "z_score": result.z_score(),
                                    "confidence": result.confidence().get(),
                                    "description": result.description(),
                                    "recommended_action": result.recommended_action().unwrap_or("None")
                                });
                                println!("{}", serde_json::to_string_pretty(&json_output)?);
                            }
                            "detailed" => {
                                println!(
                                    "{}🚨 {}{}",
                                    get_color("red", false),
                                    get_message(MessageKey::AiDetectAnomalyFound, lang),
                                    get_color("reset", false)
                                );
                                println!("  Z-score: {:.2}", result.z_score());
                                println!(
                                    "  {}: {:.1}%",
                                    if lang == Language::Japanese {
                                        "信頼度"
                                    } else {
                                        "Confidence"
                                    },
                                    result.confidence().get() * 100.0
                                );
                                println!(
                                    "  {}: {}",
                                    if lang == Language::Japanese {
                                        "説明"
                                    } else {
                                        "Description"
                                    },
                                    result.description()
                                );
                                println!(
                                    "  {}: {}",
                                    if lang == Language::Japanese {
                                        "推奨アクション"
                                    } else {
                                        "Recommended Action"
                                    },
                                    result.recommended_action().unwrap_or("None")
                                );
                            }
                            _ => {
                                let mut table = Table::new();
                                table.set_header(vec![
                                    if lang == Language::Japanese {
                                        "項目"
                                    } else {
                                        "Item"
                                    },
                                    if lang == Language::Japanese {
                                        "値"
                                    } else {
                                        "Value"
                                    },
                                ]);
                                table.add_row(vec!["Z-score", &format!("{:.2}", result.z_score())]);
                                table.add_row(vec![
                                    if lang == Language::Japanese {
                                        "信頼度"
                                    } else {
                                        "Confidence"
                                    },
                                    &format!("{:.1}%", result.confidence().get() * 100.0),
                                ]);
                                table.add_row(vec![
                                    if lang == Language::Japanese {
                                        "説明"
                                    } else {
                                        "Description"
                                    },
                                    result.description(),
                                ]);
                                println!(
                                    "{}🚨 {}{}\n",
                                    get_color("red", false),
                                    get_message(MessageKey::AiDetectAnomalyFound, lang),
                                    get_color("reset", false)
                                );
                                println!("{table}");
                            }
                        },
                        Ok(Some(_)) => {
                            // 異常なし
                            if format == "json" {
                                let json_output = serde_json::json!({
                                    "anomaly_detected": false,
                                    "message": get_message(MessageKey::AiDetectNoAnomalies, lang)
                                });
                                println!("{}", serde_json::to_string_pretty(&json_output)?);
                            } else {
                                println!(
                                    "{}✅ {}{}",
                                    get_color("green", false),
                                    get_message(MessageKey::AiDetectNoAnomalies, lang),
                                    get_color("reset", false)
                                );
                            }
                        }
                        Ok(None) => {
                            // データ不足
                            if format == "json" {
                                let json_output = serde_json::json!({
                                    "error": "insufficient_data",
                                    "message": format!(
                                        "{}（{}3{}、{}{}{}）",
                                        if lang == Language::Japanese {
                                            "データが不足しています"
                                        } else {
                                            "Insufficient data"
                                        },
                                        if lang == Language::Japanese {
                                            "最低"
                                        } else {
                                            "minimum "
                                        },
                                        if lang == Language::Japanese {
                                            "件必要"
                                        } else {
                                            " entries required"
                                        },
                                        if lang == Language::Japanese {
                                            ""
                                        } else {
                                            "found "
                                        },
                                        history.len(),
                                        if lang == Language::Japanese {
                                            "件しかありません"
                                        } else {
                                            ""
                                        }
                                    )
                                });
                                println!("{}", serde_json::to_string_pretty(&json_output)?);
                            } else {
                                println!(
                                    "{}⚠️  {}{}",
                                    get_color("yellow", false),
                                    if lang == Language::Japanese {
                                        format!(
                                            "データが不足しています（最低3件必要、{}件しかありません）",
                                            history.len()
                                        )
                                    } else {
                                        format!(
                                            "Insufficient data (minimum 3 entries required, found {})",
                                            history.len()
                                        )
                                    },
                                    get_color("reset", false)
                                );
                            }
                        }
                        Err(e) => {
                            // エラー
                            if format == "json" {
                                let json_output = serde_json::json!({
                                    "error": "analysis_failed",
                                    "message": format!("{}", e)
                                });
                                println!("{}", serde_json::to_string_pretty(&json_output)?);
                            } else {
                                println!(
                                    "{}❌ {}: {}{}",
                                    get_color("red", false),
                                    if lang == Language::Japanese {
                                        "分析エラー"
                                    } else {
                                        "Analysis error"
                                    },
                                    e,
                                    get_color("reset", false)
                                );
                            }
                        }
                    }
                }
                AiAction::Analyze {
                    path,
                    suggest_priority,
                    detailed,
                } => {
                    println!(
                        "{}{}{}",
                        get_color("magenta", false),
                        get_message(MessageKey::AiAnalyzeTitle, lang),
                        get_color("reset", false)
                    );
                    println!(
                        "{}: {:?}\n",
                        if lang == Language::Japanese {
                            "パス"
                        } else {
                            "Path"
                        },
                        path
                    );

                    let evaluator = ImportanceEvaluator::default();
                    match evaluator.evaluate(&path) {
                        Ok(result) => {
                            if detailed {
                                let mut table = Table::new();
                                table.set_header(vec![
                                    if lang == Language::Japanese {
                                        "項目"
                                    } else {
                                        "Item"
                                    },
                                    if lang == Language::Japanese {
                                        "値"
                                    } else {
                                        "Value"
                                    },
                                ]);
                                table.add_row(vec![
                                    if lang == Language::Japanese {
                                        "重要度スコア"
                                    } else {
                                        "Importance Score"
                                    },
                                    &format!("{}/100", result.score().get()),
                                ]);
                                table.add_row(vec![
                                    if lang == Language::Japanese {
                                        "推奨優先度"
                                    } else {
                                        "Recommended Priority"
                                    },
                                    &format!("{:?}", *result.priority()),
                                ]);
                                table.add_row(vec![
                                    if lang == Language::Japanese {
                                        "カテゴリ"
                                    } else {
                                        "Category"
                                    },
                                    result.category(),
                                ]);
                                table.add_row(vec![
                                    if lang == Language::Japanese {
                                        "理由"
                                    } else {
                                        "Reason"
                                    },
                                    result.reason(),
                                ]);
                                println!("{table}");
                            } else {
                                println!(
                                    "  {}: {}/100",
                                    if lang == Language::Japanese {
                                        "重要度スコア"
                                    } else {
                                        "Importance Score"
                                    },
                                    result.score().get()
                                );
                                println!(
                                    "  {}: {:?}",
                                    if lang == Language::Japanese {
                                        "推奨優先度"
                                    } else {
                                        "Recommended Priority"
                                    },
                                    *result.priority()
                                );
                                println!(
                                    "  {}: {}",
                                    if lang == Language::Japanese {
                                        "カテゴリ"
                                    } else {
                                        "Category"
                                    },
                                    result.category()
                                );
                                println!(
                                    "  {}: {}",
                                    if lang == Language::Japanese {
                                        "理由"
                                    } else {
                                        "Reason"
                                    },
                                    result.reason()
                                );
                            }

                            if suggest_priority {
                                println!(
                                    "\n{}💡 {}: backup-suite add {:?} --priority {:?}{}",
                                    get_color("yellow", false),
                                    if lang == Language::Japanese {
                                        "推奨コマンド"
                                    } else {
                                        "Recommended command"
                                    },
                                    path,
                                    *result.priority(),
                                    get_color("reset", false)
                                );
                            }
                        }
                        Err(e) => {
                            println!(
                                "{}⚠️  {}: {}{}",
                                get_color("red", false),
                                get_message(MessageKey::AiErrorAnalysisFailed, lang),
                                e,
                                get_color("reset", false)
                            );
                        }
                    }
                }
                AiAction::SuggestExclude {
                    path,
                    apply,
                    confidence,
                } => {
                    println!(
                        "{}{}{}",
                        get_color("magenta", false),
                        get_message(MessageKey::AiSuggestExcludeTitle, lang),
                        get_color("reset", false)
                    );
                    println!(
                        "{}: {:?}\n",
                        if lang == Language::Japanese {
                            "パス"
                        } else {
                            "Path"
                        },
                        path
                    );

                    let engine = ExcludeRecommendationEngine::default();
                    match engine.suggest_exclude_patterns(&path) {
                        Ok(recommendations) => {
                            let filtered: Vec<_> = recommendations
                                .into_iter()
                                .filter(|r| r.confidence().get() >= confidence)
                                .collect();

                            if filtered.is_empty() {
                                println!(
                                    "{}✅ {}{}",
                                    get_color("green", false),
                                    if lang == Language::Japanese {
                                        "除外推奨なし（すべて最適化済み）"
                                    } else {
                                        "No exclusions recommended (already optimized)"
                                    },
                                    get_color("reset", false)
                                );
                            } else {
                                let mut table = Table::new();
                                table.set_header(vec![
                                    if lang == Language::Japanese {
                                        "パターン"
                                    } else {
                                        "Pattern"
                                    },
                                    if lang == Language::Japanese {
                                        "信頼度"
                                    } else {
                                        "Confidence"
                                    },
                                    if lang == Language::Japanese {
                                        "削減見込(GB)"
                                    } else {
                                        "Reduction (GB)"
                                    },
                                    if lang == Language::Japanese {
                                        "理由"
                                    } else {
                                        "Reason"
                                    },
                                ]);
                                for rec in &filtered {
                                    table.add_row(vec![
                                        Cell::new(rec.pattern()),
                                        Cell::new(format!(
                                            "{:.1}%",
                                            rec.confidence().get() * 100.0
                                        )),
                                        Cell::new(format!("{:.2}", rec.size_reduction_gb())),
                                        Cell::new(rec.reason()),
                                    ]);
                                }
                                println!("{table}");

                                if apply {
                                    use dialoguer::Confirm;
                                    println!();
                                    for rec in &filtered {
                                        let prompt = format!(
                                            "{}\"{}\" {} ({:.2}GB {}){}",
                                            get_color("yellow", false),
                                            rec.pattern(),
                                            if lang == Language::Japanese {
                                                "を除外リストに追加しますか？"
                                            } else {
                                                "to exclude list?"
                                            },
                                            rec.size_reduction_gb(),
                                            if lang == Language::Japanese {
                                                "削減見込"
                                            } else {
                                                "reduction"
                                            },
                                            get_color("reset", false)
                                        );

                                        if Confirm::new().with_prompt(prompt).interact()? {
                                            println!(
                                                "{}✅ \"{}\" {}{}",
                                                get_color("green", false),
                                                rec.pattern(),
                                                if lang == Language::Japanese {
                                                    "を追加しました"
                                                } else {
                                                    "added"
                                                },
                                                get_color("reset", false)
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!(
                                "{}⚠️  {}: {}{}",
                                get_color("red", false),
                                get_message(MessageKey::AiErrorAnalysisFailed, lang),
                                e,
                                get_color("reset", false)
                            );
                        }
                    }
                }
                AiAction::AutoConfigure {
                    paths,
                    dry_run,
                    interactive,
                } => {
                    println!(
                        "{}{}{}",
                        get_color("magenta", false),
                        get_message(MessageKey::AiAutoConfigureTitle, lang),
                        get_color("reset", false)
                    );
                    if dry_run {
                        println!(
                            "{}[{}]{}\n",
                            get_color("yellow", false),
                            if lang == Language::Japanese {
                                "ドライラン モード"
                            } else {
                                "DRY RUN Mode"
                            },
                            get_color("reset", false)
                        );
                    }

                    let mut config = Config::load()?;
                    let evaluator = ImportanceEvaluator::default();
                    let mut added_count = 0;

                    for path in paths {
                        println!(
                            "{}: {:?}",
                            if lang == Language::Japanese {
                                "分析中"
                            } else {
                                "Analyzing"
                            },
                            path
                        );

                        // パスの存在確認
                        if !path.exists() {
                            println!(
                                "  {}❌ {}: {:?}{}",
                                get_color("red", false),
                                if lang == Language::Japanese {
                                    "パスが存在しません"
                                } else {
                                    "Path does not exist"
                                },
                                path,
                                get_color("reset", false)
                            );
                            continue;
                        }

                        match evaluator.evaluate(&path) {
                            Ok(result) => {
                                println!(
                                    "  {}: {:?} ({}: {})",
                                    if lang == Language::Japanese {
                                        "推奨優先度"
                                    } else {
                                        "Recommended Priority"
                                    },
                                    *result.priority(),
                                    if lang == Language::Japanese {
                                        "スコア"
                                    } else {
                                        "Score"
                                    },
                                    result.score().get()
                                );

                            if interactive {
                                use dialoguer::Confirm;
                                let prompt = format!(
                                    "{}{:?} {} {:?} {}{}",
                                    get_color("yellow", false),
                                    path,
                                    if lang == Language::Japanese {
                                        "を優先度"
                                    } else {
                                        "with priority"
                                    },
                                    *result.priority(),
                                    if lang == Language::Japanese {
                                        "で追加しますか？"
                                    } else {
                                        "?"
                                    },
                                    get_color("reset", false)
                                );

                                if !Confirm::new().with_prompt(prompt).interact()? {
                                    continue;
                                }
                            }

                            if !dry_run {
                                let target = Target::new(
                                    path.clone(),
                                    *result.priority(),
                                    result.category().to_string(),
                                );
                                if config.add_target(target) {
                                    added_count += 1;
                                    println!(
                                        "  {}✅ {}{}",
                                        get_color("green", false),
                                        if lang == Language::Japanese {
                                            "設定に追加しました"
                                        } else {
                                            "Added to configuration"
                                        },
                                        get_color("reset", false)
                                    );
                                }
                            }
                            }
                            Err(e) => {
                                println!(
                                    "  {}⚠️  {}: {}{}",
                                    get_color("yellow", false),
                                    if lang == Language::Japanese {
                                        "分析失敗"
                                    } else {
                                        "Analysis failed"
                                    },
                                    e,
                                    get_color("reset", false)
                                );
                            }
                        }
                    }

                    if !dry_run && added_count > 0 {
                        config.save()?;
                        println!(
                            "\n{}{}{}",
                            get_color("green", false),
                            get_message(MessageKey::AiAutoConfigureSuccess, lang),
                            get_color("reset", false)
                        );
                        println!(
                            "  {}: {}",
                            if lang == Language::Japanese {
                                "追加された項目"
                            } else {
                                "Items added"
                            },
                            added_count
                        );
                    }
                }
                AiAction::Help => {
                    print_ai_help(lang);
                }
            }
        }
        None => {
            print_help(lang);
        }
    }

    Ok(())
}
