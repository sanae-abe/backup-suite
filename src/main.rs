use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use is_terminal::IsTerminal;
use skim::prelude::*;
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
fn get_color(color_code: &str) -> &'static str {
    if supports_color() {
        match color_code {
            "green" => "\x1b[32m",
            "yellow" => "\x1b[33m",
            "red" => "\x1b[31m",
            "magenta" => "\x1b[35m",
            "gray" => "\x1b[90m",
            "reset" => "\x1b[0m",
            _ => "",
        }
    } else {
        ""
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
    Enable {
        #[arg(long)]
        priority: Option<String>,
    },
    Disable {
        #[arg(long)]
        priority: Option<String>,
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
    /// ヘルプを表示（--help を使用してください）
    #[command(hide = true)]
    Help,
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

fn print_completions<G: Generator>(generator: G, cmd: &mut clap::Command) {
    generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
}

fn select_file_with_skim(prompt: &str) -> Result<Option<PathBuf>> {
    use std::io::BufReader;

    // findコマンドでファイル一覧を取得
    let options = SkimOptionsBuilder::default()
        .height("50%".to_string())
        .multi(false)
        .prompt(prompt.to_string())
        .build()
        .map_err(|e| anyhow::anyhow!("Skim options error: {}", e))?;

    // findコマンドでファイル/ディレクトリ一覧を生成
    let cmd = "find . -type f -o -type d | head -1000";
    let child = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .ok_or_else(|| anyhow::anyhow!("findコマンド実行失敗"))?;
    let reader = BufReader::new(stdout);
    let input = SkimItemReader::default().of_bufread(reader);

    let skim_output = Skim::run_with(&options, Some(input));

    // ユーザーがキャンセル（Esc）した場合は None を返す
    let selected_items = match skim_output {
        Some(out) => {
            // Escキー（abort）や Ctrl+C でキャンセルされた場合を検出
            if out.is_abort {
                return Ok(None);
            }
            out.selected_items
        }
        None => return Ok(None), // skimが失敗した場合
    };

    if let Some(item) = selected_items.first() {
        let path_str = item.output().to_string();
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

fn select_target_with_skim(config: &Config, lang: Language) -> Result<Option<PathBuf>> {
    if config.targets.is_empty() {
        println!(
            "{}⚠️ {}{}",
            get_color("yellow"),
            get_message(MessageKey::NoTargetsRegistered, lang),
            get_color("reset")
        );
        return Ok(None);
    }

    let options = SkimOptionsBuilder::default()
        .height("50%".to_string())
        .multi(false)
        .prompt("削除するバックアップ対象を選択: ".to_string())
        .build()
        .map_err(|e| anyhow::anyhow!("Skim options error: {}", e))?;

    // バックアップ対象一覧を文字列として生成
    let targets_text = config
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
        .collect::<Vec<_>>()
        .join("\n");

    let input = SkimItemReader::default().of_bufread(std::io::Cursor::new(targets_text));

    let skim_output = Skim::run_with(&options, Some(input));

    // ユーザーがキャンセル（Esc）した場合は None を返す
    let selected_items = match skim_output {
        Some(out) => {
            // Escキー（abort）や Ctrl+C でキャンセルされた場合を検出
            if out.is_abort {
                return Ok(None);
            }
            out.selected_items
        }
        None => return Ok(None), // skimが失敗した場合
    };

    if let Some(item) = selected_items.first() {
        let selected_text = item.output().to_string();
        // 最初の部分（パス）を抽出
        if let Some(path_str) = selected_text.split_whitespace().next() {
            Ok(Some(PathBuf::from(path_str)))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn parse_priority(s: &str) -> Result<Priority> {
    match s.to_lowercase().as_str() {
        "high" => Ok(Priority::High),
        "medium" => Ok(Priority::Medium),
        "low" => Ok(Priority::Low),
        _ => Err(anyhow::anyhow!("不明な優先度: {}", s)),
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
    let green = get_color("green");
    let yellow = get_color("yellow");
    let magenta = get_color("magenta");
    let gray = get_color("gray");
    let reset = get_color("reset");

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
        "  {}{}{}     {}",
        yellow,
        get_message(MessageKey::CmdList, lang),
        reset,
        get_message(MessageKey::DescList, lang)
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
        "  {}{}{}      {}",
        yellow,
        get_message(MessageKey::CmdRestore, lang),
        reset,
        get_message(MessageKey::DescRestore, lang)
    );
    println!(
        "  {}{}{}      {}",
        yellow,
        get_message(MessageKey::CmdCleanup, lang),
        reset,
        get_message(MessageKey::DescCleanup, lang)
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
    let green = get_color("green");
    let yellow = get_color("yellow");
    let magenta = get_color("magenta");
    let gray = get_color("gray");
    let reset = get_color("reset");

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

/// config サブコマンド専用のヘルプを表示
fn print_config_help(lang: Language) {
    let green = get_color("green");
    let yellow = get_color("yellow");
    let magenta = get_color("magenta");
    let gray = get_color("gray");
    let reset = get_color("reset");

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
            get_color("green"),
            get_message(MessageKey::AppVersion, lang),
            get_color("reset")
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
                    match select_file_with_skim("追加するファイル/ディレクトリを選択: ")?
                    {
                        Some(selected_path) => selected_path,
                        None => {
                            println!(
                                "{}⚠️ {}{}",
                                get_color("yellow"),
                                get_message(MessageKey::SelectionCancelled, lang),
                                get_color("reset")
                            );
                            return Ok(());
                        }
                    }
                } else {
                    p
                }
            } else {
                match select_file_with_skim("追加するファイル/ディレクトリを選択: ")?
                {
                    Some(selected_path) => selected_path,
                    None => {
                        println!(
                            "{}⚠️ {}{}",
                            get_color("yellow"),
                            get_message(MessageKey::SelectionCancelled, lang),
                            get_color("reset")
                        );
                        return Ok(());
                    }
                }
            };

            // ファイル/ディレクトリの存在確認
            if !target_path.exists() {
                println!(
                    "{}❌ {}{}: {}: {:?}",
                    get_color("red"),
                    get_message(MessageKey::Error, lang),
                    get_color("reset"),
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
                    get_color("gray"),
                    exclude_patterns.join(", "),
                    get_color("reset")
                );
            }

            config.add_target(target);
            config.save()?;
            println!(
                "{}✅ {}{}: {:?}",
                get_color("green"),
                get_message(MessageKey::Added, lang),
                get_color("reset"),
                target_path
            );
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
                    match select_target_with_skim(&config, lang)? {
                        Some(selected_path) => selected_path,
                        None => {
                            println!(
                                "{}⚠️ {}{}",
                                get_color("yellow"),
                                get_message(MessageKey::SelectionCancelled, lang),
                                get_color("reset")
                            );
                            return Ok(());
                        }
                    }
                } else {
                    p
                }
            } else {
                match select_target_with_skim(&config, lang)? {
                    Some(selected_path) => selected_path,
                    None => {
                        println!(
                            "{}⚠️ {}{}",
                            get_color("yellow"),
                            get_message(MessageKey::SelectionCancelled, lang),
                            get_color("reset")
                        );
                        return Ok(());
                    }
                }
            };

            if config.remove_target(&target_path) {
                config.save()?;
                println!(
                    "{}✅ {}{}: {:?}",
                    get_color("green"),
                    get_message(MessageKey::Removed, lang),
                    get_color("reset"),
                    target_path
                );
            } else {
                println!(
                    "{}❌ {}{}: {:?}",
                    get_color("red"),
                    get_message(MessageKey::NotInBackupConfig, lang),
                    get_color("reset"),
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
                    get_color("red"),
                    get_message(MessageKey::SpecifyPriorityOrAll, lang),
                    get_color("reset")
                );
                return Ok(());
            }
            let removed = before - config.targets.len();
            config.save()?;
            println!(
                "{}✅ {} {}{}",
                get_color("green"),
                removed,
                get_message(MessageKey::CountDeleted, lang),
                get_color("reset")
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
                get_color("green"),
                get_message(MessageKey::BackupRunning, lang),
                options_str,
                get_color("reset")
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
                        get_color("green"),
                        get_message(MessageKey::EncryptionPassword, lang),
                        get_color("reset"),
                        pwd_str
                    );
                    println!(
                        "{}⚠️  Please save this password securely!{}",
                        get_color("yellow"),
                        get_color("reset")
                    );

                    pwd_str
                } else if let Some(p) = password {
                    // コマンドラインから提供されたパスワードの強度チェック
                    let policy = PasswordPolicy::default();
                    let strength = policy.evaluate(&p);

                    if !matches!(strength, PasswordStrength::Strong) {
                        println!(
                            "{}{}{}",
                            get_color("yellow"),
                            policy.display_report(&p),
                            get_color("reset")
                        );
                    } else {
                        println!(
                            "{}✅ Password Strength: {}{}",
                            get_color("green"),
                            strength.display(),
                            get_color("reset")
                        );
                    }

                    p
                } else {
                    // パスワードプロンプト（dialoguerを使用して隠し入力）
                    use dialoguer::Password;

                    let input = Password::new()
                        .with_prompt(format!(
                            "{}{}{}",
                            get_color("yellow"),
                            get_message(MessageKey::EncryptionPassword, lang),
                            get_color("reset")
                        ))
                        .interact()?;

                    // パスワード強度チェック
                    let policy = PasswordPolicy::default();
                    let strength = policy.evaluate(&input);

                    if !matches!(strength, PasswordStrength::Strong) {
                        println!(
                            "{}{}{}",
                            get_color("yellow"),
                            policy.display_report(&input),
                            get_color("reset")
                        );
                    } else {
                        println!(
                            "{}✅ Password Strength: {}{}",
                            get_color("green"),
                            strength.display(),
                            get_color("reset")
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
                        get_color("yellow"),
                        get_message(MessageKey::ErrorDetails, lang),
                        get_color("reset")
                    );
                    for (i, error) in result.errors.iter().enumerate() {
                        println!("  {}. {}", i + 1, error);
                    }
                }
            } else {
                println!(
                    "{}📋 {}{}: {} {}",
                    get_color("gray"),
                    get_message(MessageKey::Detected, lang),
                    get_color("reset"),
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
                    get_color("red"),
                    get_message(MessageKey::NoBackups, lang),
                    get_color("reset")
                );
                return Ok(());
            }

            let backup_dir = if let Some(pattern) = from {
                dirs.iter()
                    .find(|d| d.to_string_lossy().contains(&pattern))
                    .ok_or_else(|| anyhow::anyhow!("バックアップが見つかりません: {}", pattern))?
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
                get_color("green"),
                get_message(MessageKey::RestoreStart, lang),
                get_color("reset"),
                backup_dir,
                dest
            );

            // RestoreEngineを使用して復元
            let mut engine = RestoreEngine::new(false);
            let result = engine.restore(backup_dir, &dest, password.as_deref())?;

            println!(
                "\n{}✅ {} {:?}{}",
                get_color("green"),
                get_message(MessageKey::RestoredSuccess, lang),
                dest,
                get_color("reset")
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
                    get_color("yellow"),
                    result.failed,
                    get_message(MessageKey::CountDeleted, lang),
                    get_color("reset")
                );
                for error in &result.errors {
                    println!("  - {}", error);
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
                get_color("green"),
                result.deleted,
                get_message(MessageKey::CountDeleted, lang),
                if dry_run {
                    get_message(MessageKey::DryRunParens, lang)
                } else {
                    ""
                },
                get_color("reset")
            );

            if result.freed_bytes > 0 {
                let freed_mb = result.freed_bytes as f64 / 1024.0 / 1024.0;
                println!(
                    "  {}解放容量: {:.2} MB{}",
                    get_color("gray"),
                    freed_mb,
                    get_color("reset")
                );
            }

            if !result.errors.is_empty() {
                println!(
                    "{}⚠️ エラー: {}件{}",
                    get_color("yellow"),
                    result.errors.len(),
                    get_color("reset")
                );
                for error in &result.errors {
                    println!("  - {}", error);
                }
            }
        }
        Some(Commands::Status) => {
            let config = Config::load()?;
            println!(
                "{}📊 {}{}",
                get_color("magenta"),
                get_message(MessageKey::StatusTitle, lang),
                get_color("reset")
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
                get_color("red"),
                get_message(MessageKey::High, lang),
                get_color("reset"),
                config.filter_by_priority(&Priority::High).len()
            );
            println!(
                "    {}{}{}: {}",
                get_color("yellow"),
                get_message(MessageKey::Medium, lang),
                get_color("reset"),
                config.filter_by_priority(&Priority::Medium).len()
            );
            println!(
                "    {}{}{}: {}",
                get_color("gray"),
                get_message(MessageKey::Low, lang),
                get_color("reset"),
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
                get_color("magenta"),
                get_message(MessageKey::BackupHistory, lang),
                get_color("reset"),
                days,
                get_message(MessageKey::Days, lang)
            );

            if detailed {
                // 詳細表示
                for entry in &history {
                    println!("\n{}{}{}", get_color("green"), "=".repeat(60), get_color("reset"));
                    println!("🕒 {}: {}", get_message(MessageKey::StatusTitle, lang), entry.timestamp.format("%Y-%m-%d %H:%M:%S"));
                    println!("📁 パス: {:?}", entry.backup_dir);
                    if let Some(ref cat) = entry.category {
                        println!("🏷️  カテゴリ: {}", cat);
                    }
                    if let Some(ref prio) = entry.priority {
                        println!("⚡ 優先度: {:?}", prio);
                    }
                    println!("📊 ステータス: {:?}", entry.status);
                    println!("📦 ファイル数: {}", entry.total_files);
                    println!("💾 サイズ: {:.2} MB", entry.total_bytes as f64 / 1024.0 / 1024.0);
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
                        println!("{}❌ エラー: {}{}", get_color("red"), err, get_color("reset"));
                    }
                }
            } else {
                // テーブル表示
                display_history(&history, &theme);
            }
        }
        Some(Commands::Enable { priority }) => {
            let mut config = Config::load()?;
            config.backup.auto_cleanup = true;
            config.save()?;
            println!(
                "{}✅ {}{}{}",
                get_color("green"),
                get_message(MessageKey::AutoBackupEnabled, lang),
                priority
                    .as_ref()
                    .map(|p| format!(" ({})", p))
                    .unwrap_or_default(),
                get_color("reset")
            );
        }
        Some(Commands::Disable { priority }) => {
            let mut config = Config::load()?;
            config.backup.auto_cleanup = false;
            config.save()?;
            println!(
                "{}⏸️  {}{}{}",
                get_color("yellow"),
                get_message(MessageKey::AutoBackupDisabled, lang),
                priority
                    .as_ref()
                    .map(|p| format!(" ({})", p))
                    .unwrap_or_default(),
                get_color("reset")
            );
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
                get_color("green"),
                get_message(MessageKey::OpenDirectory, lang),
                get_color("reset"),
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
                            get_color("green"),
                            get_message(MessageKey::AutoBackupEnabled, lang),
                            get_color("reset"),
                            p
                        );
                    } else {
                        scheduler.setup_all()?;
                        scheduler.enable_all()?;
                        println!(
                            "{}✅ {}{}",
                            get_color("green"),
                            get_message(MessageKey::AutoBackupEnabled, lang),
                            get_color("reset")
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
                            get_color("yellow"),
                            get_message(MessageKey::AutoBackupDisabled, lang),
                            get_color("reset"),
                            p
                        );
                    } else {
                        config.schedule.enabled = false;
                        config.save()?;
                        scheduler.disable_all()?;
                        println!(
                            "{}⏸️  {}{}",
                            get_color("yellow"),
                            get_message(MessageKey::AutoBackupDisabled, lang),
                            get_color("reset")
                        );
                    }
                }
                ScheduleAction::Status => {
                    println!(
                        "{}📅 {}{}",
                        get_color("magenta"),
                        get_message(MessageKey::ScheduleSettings, lang),
                        get_color("reset")
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
                        get_color("magenta"),
                        get_message(MessageKey::ActualScheduleStatus, lang),
                        get_color("reset")
                    );

                    println!(
                        "  high: {}{}{}",
                        if status.high_enabled {
                            get_color("green")
                        } else {
                            get_color("red")
                        },
                        if status.high_enabled { "✅ " } else { "❌ " },
                        if status.high_enabled {
                            get_message(MessageKey::Enabled, lang)
                        } else {
                            get_message(MessageKey::Disabled, lang)
                        }
                    );
                    println!("{}", get_color("reset"));

                    println!(
                        "  medium: {}{}{}",
                        if status.medium_enabled {
                            get_color("green")
                        } else {
                            get_color("red")
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
                    println!("{}", get_color("reset"));

                    println!(
                        "  low: {}{}{}",
                        if status.low_enabled {
                            get_color("green")
                        } else {
                            get_color("red")
                        },
                        if status.low_enabled { "✅ " } else { "❌ " },
                        if status.low_enabled {
                            get_message(MessageKey::Enabled, lang)
                        } else {
                            get_message(MessageKey::Disabled, lang)
                        }
                    );
                    println!("{}", get_color("reset"));
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
                            get_color("green"),
                            get_message(MessageKey::ScheduleUpdated, lang),
                            get_color("reset")
                        );
                    } else {
                        println!(
                            "{}✅ {}{}",
                            get_color("green"),
                            get_message(MessageKey::ScheduleUpdatedEnableLater, lang),
                            get_color("reset")
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
                            get_color("yellow"),
                            get_message(MessageKey::DirectoryNotExists, lang),
                            get_color("reset"),
                            path
                        );
                        std::fs::create_dir_all(&path).map_err(|e| {
                            anyhow::anyhow!("ディレクトリ作成失敗: {:?} - {}", path, e)
                        })?;
                    }

                    // 書き込み権限を確認
                    use backup_suite::security::check_write_permission;
                    check_write_permission(&path)
                        .map_err(|e| anyhow::anyhow!("書き込み権限エラー: {:?} - {}", path, e))?;

                    // 設定を更新
                    let old_destination = config.backup.destination.clone();
                    config.backup.destination = path.clone();
                    config.save()?;

                    println!(
                        "{}✅ {}{}",
                        get_color("green"),
                        get_message(MessageKey::DestinationChanged, lang),
                        get_color("reset")
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
                        get_color("magenta"),
                        get_message(MessageKey::CurrentDestination, lang),
                        get_color("reset")
                    );
                    println!("  {:?}", config.backup.destination);
                }
                ConfigAction::SetKeepDays { days } => {
                    if days == 0 || days > 3650 {
                        println!(
                            "{}❌ {}{}: {} {}）",
                            get_color("red"),
                            get_message(MessageKey::Error, lang),
                            get_color("reset"),
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
                        get_color("green"),
                        get_message(MessageKey::KeepDaysChanged, lang),
                        get_color("reset")
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
                        get_color("magenta"),
                        get_message(MessageKey::CurrentKeepDays, lang),
                        get_color("reset")
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
                        get_color("green"),
                        get_message(MessageKey::OpeningConfigFile, lang),
                        get_color("reset"),
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
                        .context(format!("エディタ起動失敗: {}", editor))?;

                    if !status.success() {
                        println!(
                            "{}⚠️ {}{}",
                            get_color("yellow"),
                            get_message(MessageKey::EditorDidNotExitCleanly, lang),
                            get_color("reset")
                        );
                    }
                }
                ConfigAction::Help => {
                    print_config_help(lang);
                }
            }
        }
        Some(Commands::Help) => {
            print_help(lang);
        }
        None => {
            print_help(lang);
        }
    }

    Ok(())
}
