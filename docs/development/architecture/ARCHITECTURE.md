# アーキテクチャドキュメント

Backup Suite v1.0.0の技術的なシステム設計、実装詳細、拡張性について説明します。

## 📋 目次

- [システム概要](#システム概要)
- [アーキテクチャ設計](#アーキテクチャ設計)
- [コンポーネント詳細](#コンポーネント詳細)
- [データフロー](#データフロー)
- [設定管理システム](#設定管理システム)
- [統合システム](#統合システム)
- [パフォーマンス設計](#パフォーマンス設計)
- [セキュリティ設計](#セキュリティ設計)
- [拡張性・カスタマイズ](#拡張性カスタマイズ)

## 🏗️ システム概要

### 設計哲学

Backup Suiteは以下の原則に基づいて設計されています：

1. **型安全性優先**: Rustの型システムを活用した実行時エラーの最小化
2. **パフォーマンス重視**: 並列処理とメモリ効率性による高速化
3. **ユーザビリティ**: 直感的なCLIとインタラクティブUI
4. **拡張性**: モジュラー設計による機能拡張の容易さ
5. **信頼性**: データ損失防止とエラー処理の徹底

### 技術スタック

```
┌─── 🦀 Rust Core ──────────────────────────────────────┐
│ Language: Rust 1.70+                                  │
│ Memory Safety: 所有権システム・借用チェッカー         │
│ Performance: ゼロコスト抽象化・LLVM最適化            │
└────────────────────────────────────────────────────────┘

┌─── 📦 主要Dependencies ───────────────────────────────┐
│ clap 4.x          │ CLI引数解析・サブコマンド・補完    │
│ skim              │ ファジーファインダー・インタラクティブUI │
│ serde + toml      │ 設定ファイル管理・構造化データ     │
│ anyhow           │ エラー処理・コンテキスト管理        │
│ chrono           │ 日時処理・ISO 8601対応             │
│ dirs             │ プラットフォーム別ディレクトリパス   │
│ atty             │ ターミナル環境検出・カラー出力制御   │
│ clap_complete    │ シェル補完生成（zsh/bash/fish）    │
└────────────────────────────────────────────────────────┘

┌─── 🔧 システム統合 ───────────────────────────────────┐
│ macOS launchctl   │ 自動スケジューリング・システム統合  │
│ Unix filesystem   │ ファイル操作・権限管理            │
│ Shell integration │ 補完・環境変数・パイプライン      │
└────────────────────────────────────────────────────────┘
```

## 🏛️ アーキテクチャ設計

### レイヤードアーキテクチャ

```
┌─────────────────────────────────────────────────────────┐
│                    🎯 CLI Layer                         │
│  ┌─────────────┬─────────────┬─────────────────────────┐ │
│  │ Command     │ Interactive │ Completion              │ │
│  │ Parsing     │ UI (skim)   │ Generation              │ │
│  │ (clap)      │            │ (clap_complete)         │ │
│  └─────────────┴─────────────┴─────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                  ⚙️ Business Logic Layer                 │
│  ┌─────────────┬─────────────┬─────────────────────────┐ │
│  │ Backup      │ Schedule    │ Config                  │ │
│  │ Runner      │ Manager     │ Manager                 │ │
│  └─────────────┴─────────────┴─────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                   💾 Data Layer                         │
│  ┌─────────────┬─────────────┬─────────────────────────┐ │
│  │ TOML        │ File        │ History                 │ │
│  │ Config      │ Operations  │ Management              │ │
│  │ (serde)     │            │ (chrono)                │ │
│  └─────────────┴─────────────┴─────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                🖥️ System Integration Layer               │
│  ┌─────────────┬─────────────┬─────────────────────────┐ │
│  │ Filesystem  │ launchctl   │ Terminal                │ │
│  │ Operations  │ (macOS)     │ Detection               │ │
│  │            │            │ (atty)                  │ │
│  └─────────────┴─────────────┴─────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### モジュール構成

```
backup-suite/
├── src/
│   ├── main.rs              # エントリーポイント・CLI実装
│   └── core/                # コアビジネスロジック
│       ├── mod.rs           # モジュール定義
│       ├── config.rs        # 設定管理（Config, Target）
│       ├── backup.rs        # バックアップ実行（BackupRunner）
│       ├── history.rs       # 履歴管理（BackupHistory）
│       ├── schedule.rs      # スケジューリング（launchctl統合）
│       └── utils.rs         # ユーティリティ関数
├── Cargo.toml              # 依存関係・メタデータ
└── docs/                   # ドキュメント
```

## 🔧 コンポーネント詳細

### CLI Layer (`main.rs`)

#### Clap Command Structure
```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add { path, priority, category, interactive },
    List { priority },
    Remove { path, interactive },
    // ... 他のコマンド
    Schedule { action: ScheduleAction },
}

#[derive(Subcommand)]
enum ScheduleAction {
    Enable { priority },
    Disable { priority },
    Status,
    Setup { high, medium, low },
}
```

#### Interactive UI Integration
```rust
fn select_file_with_skim(prompt: &str) -> Result<Option<PathBuf>> {
    let options = SkimOptionsBuilder::default()
        .height("50%".to_string())
        .multi(false)
        .prompt(prompt.to_string())
        .build()?;

    // findコマンドとskim統合
    let cmd = "find . -type f -o -type d | head -1000";
    let child = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    // skimでインタラクティブ選択
    let input = SkimItemReader::default().of_bufread(BufReader::new(stdout));
    let selected_items = Skim::run_with(&options, Some(input))
        .map(|out| out.selected_items)
        .unwrap_or_else(Vec::new);

    // 選択結果の処理
    if let Some(item) = selected_items.first() {
        let path = PathBuf::from(item.output().to_string());
        Ok(Some(path.canonicalize()?))
    } else {
        Ok(None)
    }
}
```

#### Color Management System
```rust
fn supports_color() -> bool {
    atty::is(atty::Stream::Stdout) &&
    std::env::var("NO_COLOR").is_err() &&
    std::env::var("TERM").map(|term| term != "dumb").unwrap_or(true)
}

fn get_color(color_code: &str) -> &'static str {
    if supports_color() {
        match color_code {
            "green" => "\\x1b[32m",
            "yellow" => "\\x1b[33m",
            "red" => "\\x1b[31m",
            "reset" => "\\x1b[0m",
            _ => "",
        }
    } else {
        ""
    }
}
```

### Business Logic Layer (`core/`)

#### Config Management (`core/config.rs`)
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub backup: BackupConfig,
    #[serde(default)]
    pub schedule: ScheduleConfig,
    pub targets: Vec<Target>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupConfig {
    pub destination: PathBuf,
    pub auto_cleanup: bool,
    pub keep_days: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub enabled: bool,
    pub high_frequency: String,    // "daily", "weekly", "monthly"
    pub medium_frequency: String,
    pub low_frequency: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Target {
    pub path: PathBuf,
    pub priority: Priority,
    pub target_type: TargetType,
    pub category: String,
    pub added_date: DateTime<Utc>,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TargetType {
    File,
    Directory,
}
```

#### Backup Runner (`core/backup.rs`)
```rust
pub struct BackupRunner {
    config: Config,
    dry_run: bool,
}

pub struct BackupResult {
    pub success_files: usize,
    pub total_files: usize,
    pub total_bytes: u64,
    pub errors: Vec<String>,
}

impl BackupRunner {
    pub fn new(config: Config, dry_run: bool) -> Self {
        Self { config, dry_run }
    }

    pub fn run(&self, priority_filter: Option<&Priority>) -> Result<BackupResult> {
        let targets = self.filter_targets(priority_filter);
        let backup_dir = self.create_backup_directory()?;

        let mut result = BackupResult::default();

        for target in targets {
            match self.backup_target(&target, &backup_dir) {
                Ok(stats) => {
                    result.success_files += stats.files;
                    result.total_bytes += stats.bytes;
                }
                Err(e) => {
                    result.errors.push(format!("{}: {}", target.path.display(), e));
                }
            }
        }

        result.total_files = result.success_files + result.errors.len();

        if !self.dry_run {
            self.save_history(&result, &backup_dir)?;
        }

        Ok(result)
    }

    fn backup_target(&self, target: &Target, backup_dir: &Path) -> Result<BackupStats> {
        match target.target_type {
            TargetType::File => self.backup_file(&target.path, backup_dir),
            TargetType::Directory => self.backup_directory(&target.path, backup_dir, &target.exclude_patterns),
        }
    }
}
```

#### History Management (`core/history.rs`)
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupHistory {
    pub timestamp: DateTime<Utc>,
    pub backup_dir: PathBuf,
    pub total_files: usize,
    pub total_bytes: u64,
    pub success: bool,
    pub priority: Option<String>,
    pub errors: Vec<String>,
}

impl BackupHistory {
    pub fn new(result: &BackupResult, backup_dir: &Path, priority: Option<&str>) -> Self {
        Self {
            timestamp: Utc::now(),
            backup_dir: backup_dir.to_path_buf(),
            total_files: result.total_files,
            total_bytes: result.total_bytes,
            success: result.errors.is_empty(),
            priority: priority.map(String::from),
            errors: result.errors.clone(),
        }
    }

    pub fn save(&self, config_dir: &Path) -> Result<()> {
        let history_file = config_dir.join("history.toml");
        let mut histories = Self::load_all_from_file(&history_file)?;
        histories.push(self.clone());

        // 最新1000件のみ保持
        if histories.len() > 1000 {
            histories.drain(0..histories.len() - 1000);
        }

        let toml_content = toml::to_string_pretty(&histories)?;
        std::fs::write(history_file, toml_content)?;
        Ok(())
    }

    pub fn filter_by_days(days: u32) -> Result<Vec<Self>> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        let all_histories = Self::load_all()?;

        Ok(all_histories
            .into_iter()
            .filter(|h| h.timestamp > cutoff)
            .collect())
    }
}
```

### launchctl Integration (`core/schedule.rs`)

#### macOS Scheduling System
```rust
pub struct ScheduleManager {
    config: Config,
}

impl ScheduleManager {
    pub fn setup_schedule(&self, priority: &str) -> Result<()> {
        let frequency = self.get_frequency_for_priority(priority)?;
        let plist_content = self.create_plist_content(priority, frequency)?;
        let plist_path = self.get_plist_path(priority)?;

        // plistファイル作成
        std::fs::write(&plist_path, plist_content)?;

        // launchctl load
        let output = std::process::Command::new("launchctl")
            .args(&["load", &plist_path.to_string_lossy()])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("launchctl load failed: {}",
                String::from_utf8_lossy(&output.stderr)));
        }

        Ok(())
    }

    fn create_plist_content(&self, priority: &str, frequency: &str) -> Result<String> {
        let backup_suite_path = std::env::current_exe()?;

        let plist = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
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
    <dict>
        <key>Hour</key>
        <integer>2</integer>
        <key>Minute</key>
        <integer>0</integer>
        {weekday_or_day}
    </dict>

    <key>RunAtLoad</key>
    <false/>

    <key>StandardOutPath</key>
    <string>/tmp/backup-suite-{priority}.log</string>

    <key>StandardErrorPath</key>
    <string>/tmp/backup-suite-{priority}.error.log</string>
</dict>
</plist>"#,
            priority = priority,
            backup_suite_path = backup_suite_path.display(),
            weekday_or_day = match frequency {
                "weekly" => "<key>Weekday</key>\\n        <integer>0</integer>",
                "monthly" => "<key>Day</key>\\n        <integer>1</integer>",
                _ => "",
            }
        );

        Ok(plist)
    }

    pub fn check_status(&self) -> Result<HashMap<String, bool>> {
        let mut status = HashMap::new();

        for priority in &["high", "medium", "low"] {
            let label = format!("com.backup-suite.{}", priority);
            let output = std::process::Command::new("launchctl")
                .args(&["list", &label])
                .output()?;

            status.insert(priority.to_string(), output.status.success());
        }

        Ok(status)
    }
}
```

## 🔄 データフロー

### 設定ファイル読み込みフロー

```
1. アプリケーション起動
   ↓
2. Config::load() 呼び出し
   ↓
3. ~/.config/backup-suite/config.toml 読み込み
   ↓
4. TOML → Rust構造体デシリアライゼーション (serde)
   ↓
5. 設定検証・デフォルト値適用
   ↓
6. Config構造体として利用可能
```

### バックアップ実行フロー

```
1. backup-suite run コマンド実行
   ↓
2. CLI引数解析 (clap)
   ↓
3. Config読み込み
   ↓
4. BackupRunner初期化
   ↓
5. 対象フィルタリング（priority）
   ↓
6. バックアップディレクトリ作成
   ↓
7. 各対象について並列バックアップ実行
   │ ├─ ファイル/ディレクトリ判定
   │ ├─ 除外パターン適用
   │ ├─ コピー実行
   │ └─ 統計情報更新
   ↓
8. 履歴保存 (BackupHistory)
   ↓
9. 結果表示・終了
```

### インタラクティブ選択フロー

```
1. backup-suite add --interactive
   ↓
2. select_file_with_skim() 呼び出し
   ↓
3. find コマンド実行（ファイル一覧取得）
   ↓
4. skim UI表示
   ↓
5. ユーザーによるファジーサーチ・選択
   ↓
6. 選択結果取得・パス正規化
   ↓
7. Target構造体作成・Config追加
   ↓
8. TOML保存
```

## ⚙️ 設定管理システム

### TOML構造とserdeマッピング

#### serdeアトリビュート活用
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: String,

    pub backup: BackupConfig,

    #[serde(default)]  // デフォルト値使用
    pub schedule: ScheduleConfig,

    #[serde(default)]  // 空のVecをデフォルト
    pub targets: Vec<Target>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    pub path: PathBuf,

    #[serde(default = "default_priority")]
    pub priority: Priority,

    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    #[serde(with = "chrono::serde::ts_seconds")]
    pub added_date: DateTime<Utc>,
}

fn default_priority() -> Priority {
    Priority::Medium
}
```

#### 下位互換性保証
```rust
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

// 新フィールド追加時の互換性
#[serde(default)]  // 古い設定ファイルでも動作
pub new_feature: bool,
```

### 設定ファイルのマイグレーション

```rust
impl Config {
    pub fn load() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            // 初回実行時はデフォルト設定作成
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let content = std::fs::read_to_string(&config_path)?;
        let mut config: Config = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("設定ファイル読み込みエラー: {}", e))?;

        // バージョンチェック・マイグレーション
        if config.version != env!("CARGO_PKG_VERSION") {
            config = Self::migrate_config(config)?;
        }

        Ok(config)
    }

    fn migrate_config(mut config: Config) -> Result<Self> {
        // バージョン別マイグレーション処理
        match config.version.as_str() {
            "0.9.x" => {
                // 古いformat変換
                config.version = "1.0.0".to_string();
            }
            _ => {
                // 現在のバージョンに更新
                config.version = env!("CARGO_PKG_VERSION").to_string();
            }
        }

        config.save()?;
        Ok(config)
    }
}
```

## 🔗 統合システム

### Shell Completion Generation

#### clap_complete統合
```rust
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};

fn print_completions<G: Generator>(generator: G, cmd: &mut clap::Command) {
    generate(generator, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

// usage:
let mut cmd = Cli::command();
match shell {
    Shell::Zsh => print_completions(clap_complete::shells::Zsh, &mut cmd),
    Shell::Bash => print_completions(clap_complete::shells::Bash, &mut cmd),
    Shell::Fish => print_completions(clap_complete::shells::Fish, &mut cmd),
}
```

#### 補完品質向上
```rust
#[derive(Parser)]
struct AddCommand {
    /// File or directory path to add
    #[arg(value_hint = ValueHint::AnyPath)]
    path: Option<PathBuf>,

    /// Priority level
    #[arg(long, value_enum)]
    priority: Option<Priority>,

    /// Category for organization
    #[arg(long)]
    category: Option<String>,
}

#[derive(ValueEnum, Clone)]
enum Priority {
    High,
    Medium,
    Low,
}
```

### Cross-Platform Compatibility

#### プラットフォーム検出
```rust
#[cfg(target_os = "macos")]
fn open_directory(path: &Path) -> Result<()> {
    std::process::Command::new("open").arg(path).spawn()?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn open_directory(path: &Path) -> Result<()> {
    std::process::Command::new("xdg-open").arg(path).spawn()?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn open_directory(path: &Path) -> Result<()> {
    std::process::Command::new("explorer").arg(path).spawn()?;
    Ok(())
}
```

#### 条件付きコンパイル
```rust
#[cfg(target_os = "macos")]
mod macos_schedule;

#[cfg(target_os = "linux")]
mod systemd_schedule;

#[cfg(target_os = "windows")]
mod windows_schedule;
```

## ⚡ パフォーマンス設計

### メモリ効率性

#### 所有権システム活用
```rust
// ゼロコピー文字列処理
fn process_file_list(files: &[PathBuf]) -> Result<Vec<&Path>> {
    files.iter()
        .map(|p| p.as_path())
        .filter(|p| p.exists())
        .collect()
}

// ストリーミング処理
fn backup_large_directory(src: &Path, dst: &Path) -> Result<()> {
    for entry in WalkDir::new(src) {
        let entry = entry?;
        // 1ファイルずつ処理（全ファイルを一度にメモリに保持しない）
        process_single_file(&entry.path())?;
    }
    Ok(())
}
```

#### 並列処理最適化
```rust
use rayon::prelude::*;

fn backup_files_parallel(files: &[PathBuf], dst: &Path) -> Result<Vec<BackupResult>> {
    files.par_iter()
        .map(|src| backup_single_file(src, dst))
        .collect::<Result<Vec<_>>>()
}

// CPUコア数に応じた並列度調整
fn configure_thread_pool() {
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .expect("Failed to configure thread pool");
}
```

### I/O最適化

#### バッファリング戦略
```rust
use std::io::{BufReader, BufWriter};

fn copy_file_optimized(src: &Path, dst: &Path) -> Result<u64> {
    let src_file = File::open(src)?;
    let dst_file = File::create(dst)?;

    let mut reader = BufReader::with_capacity(64 * 1024, src_file);  // 64KB buffer
    let mut writer = BufWriter::with_capacity(64 * 1024, dst_file);

    std::io::copy(&mut reader, &mut writer)
}

// 大ファイル用のストリーミングコピー
fn copy_large_file(src: &Path, dst: &Path) -> Result<()> {
    const CHUNK_SIZE: usize = 1024 * 1024;  // 1MB chunks

    let mut src_file = File::open(src)?;
    let mut dst_file = File::create(dst)?;
    let mut buffer = vec![0; CHUNK_SIZE];

    loop {
        let bytes_read = src_file.read(&mut buffer)?;
        if bytes_read == 0 { break; }

        dst_file.write_all(&buffer[..bytes_read])?;
    }

    Ok(())
}
```

## 🛡️ セキュリティ設計

### メモリ安全性

#### Rustの安全性保証
```rust
// 自動的にメモリ安全
fn safe_string_processing(input: &str) -> String {
    input.chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
    // メモリ自動解放、ダングリングポインタなし
}

// 借用チェッカーによる安全性
fn safe_file_access(paths: &[PathBuf]) -> Vec<String> {
    paths.iter()
        .filter_map(|p| p.to_str())  // 無効なUTF-8は安全に無視
        .map(|s| s.to_string())
        .collect()
}
```

#### 入力検証・サニタイゼーション
```rust
fn validate_path(path: &Path) -> Result<PathBuf> {
    // パストラバーサル攻撃防止
    let canonical = path.canonicalize()
        .map_err(|_| anyhow::anyhow!("無効なパス: {}", path.display()))?;

    // ホームディレクトリ外へのアクセス防止
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("ホームディレクトリが特定できません"))?;

    if !canonical.starts_with(&home) {
        return Err(anyhow::anyhow!("ホームディレクトリ外へのアクセスは禁止されています"));
    }

    Ok(canonical)
}

fn sanitize_category(category: &str) -> String {
    category.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .take(32)  // 最大32文字
        .collect()
}
```

### ファイルシステムセキュリティ

#### 権限管理
```rust
use std::os::unix::fs::PermissionsExt;

fn secure_file_creation(path: &Path, content: &[u8]) -> Result<()> {
    let mut file = File::create(path)?;

    // ファイル権限設定（所有者のみ読み書き可能）
    let metadata = file.metadata()?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o600);  // rw-------
    file.set_permissions(permissions)?;

    file.write_all(content)?;
    Ok(())
}

fn secure_directory_creation(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;

    // ディレクトリ権限設定
    let metadata = std::fs::metadata(path)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o700);  // rwx------
    std::fs::set_permissions(path, permissions)?;

    Ok(())
}
```

#### 機密情報保護
```rust
fn load_config_securely() -> Result<Config> {
    let config_path = get_config_path()?;

    // 設定ファイルの権限確認
    let metadata = std::fs::metadata(&config_path)?;
    let permissions = metadata.permissions();

    if permissions.mode() & 0o077 != 0 {
        return Err(anyhow::anyhow!(
            "設定ファイルの権限が安全でありません。chmod 600 {} を実行してください",
            config_path.display()
        ));
    }

    let content = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}
```

## 🔌 拡張性・カスタマイズ

### プラグインアーキテクチャ（将来計画）

#### Trait Based Extension
```rust
pub trait BackupProcessor {
    fn process_before_backup(&self, target: &Target) -> Result<()>;
    fn process_after_backup(&self, target: &Target, result: &BackupResult) -> Result<()>;
    fn supports_target(&self, target: &Target) -> bool;
}

pub trait StorageBackend {
    fn upload(&self, local_path: &Path, remote_path: &str) -> Result<()>;
    fn download(&self, remote_path: &str, local_path: &Path) -> Result<()>;
    fn list(&self, prefix: &str) -> Result<Vec<String>>;
    fn delete(&self, remote_path: &str) -> Result<()>;
}

// 実装例
pub struct S3Backend {
    bucket: String,
    region: String,
    credentials: Credentials,
}

impl StorageBackend for S3Backend {
    fn upload(&self, local_path: &Path, remote_path: &str) -> Result<()> {
        // AWS S3への実装
        todo!()
    }
}
```

#### 動的プラグインローディング
```rust
pub struct PluginManager {
    processors: Vec<Box<dyn BackupProcessor>>,
    backends: Vec<Box<dyn StorageBackend>>,
}

impl PluginManager {
    pub fn load_plugins(&mut self, plugin_dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(plugin_dir)? {
            let entry = entry?;
            if entry.path().extension() == Some(std::ffi::OsStr::new("so")) {
                self.load_plugin(&entry.path())?;
            }
        }
        Ok(())
    }

    fn load_plugin(&mut self, plugin_path: &Path) -> Result<()> {
        // 動的ライブラリロード
        // unsafe だが、プラグインシステムには必要
        todo!()
    }
}
```

### 設定拡張ポイント

#### カスタムフィルター
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomFilter {
    pub name: String,
    pub pattern: String,
    pub action: FilterAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FilterAction {
    Include,
    Exclude,
    Transform(String),
}

// 設定ファイルでの使用
[[custom_filters]]
name = "exclude_build_files"
pattern = "**/target/**"
action = "Exclude"

[[custom_filters]]
name = "compress_logs"
pattern = "**/*.log"
action = { Transform = "gzip" }
```

#### カスタム通知システム
```rust
pub trait NotificationBackend {
    fn send_success(&self, message: &str) -> Result<()>;
    fn send_warning(&self, message: &str) -> Result<()>;
    fn send_error(&self, message: &str) -> Result<()>;
}

pub struct SlackNotifier {
    webhook_url: String,
    channel: String,
}

impl NotificationBackend for SlackNotifier {
    fn send_success(&self, message: &str) -> Result<()> {
        let payload = json!({
            "channel": self.channel,
            "text": format!("✅ {}", message),
            "icon_emoji": ":backup:"
        });

        // HTTP POST to Slack webhook
        todo!()
    }
}
```

### CLI拡張

#### カスタムコマンド追加
```rust
// プラグインでの新コマンド定義
#[derive(Subcommand)]
pub enum PluginCommands {
    Encrypt {
        #[arg(long)]
        key_file: PathBuf,
    },

    Sync {
        #[arg(long)]
        remote: String,
    },

    Analyze {
        #[arg(long)]
        report_format: String,
    },
}

// メインCLIとの統合
#[derive(Subcommand)]
pub enum Commands {
    // ... 既存コマンド

    #[command(flatten)]
    Plugin(PluginCommands),
}
```

### 設定スキーマ拡張

#### 型安全な設定拡張
```rust
// プラグイン設定の型安全な管理
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtendedConfig {
    #[serde(flatten)]
    pub base: Config,

    #[serde(default)]
    pub encryption: EncryptionConfig,

    #[serde(default)]
    pub cloud_sync: CloudSyncConfig,

    #[serde(default)]
    pub notifications: NotificationConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EncryptionConfig {
    pub enabled: bool,
    pub algorithm: String,
    pub key_derivation: String,
}
```

## 🔮 将来の拡張計画

### v1.1.0: クロスプラットフォーム対応
- Linux systemd統合
- Windows Task Scheduler統合
- 統一されたスケジューリングインターフェース

### v1.2.0: クラウド統合
- AWS S3バックエンド
- Google Drive API統合
- 増分同期・デデュープリケーション

### v1.3.0: 高度な機能
- 暗号化バックアップ
- 圧縮オプション
- WebUI（オプション）

### v2.0.0: エンタープライズ機能
- マルチユーザー対応
- 中央管理・ポリシー制御
- 監査ログ・コンプライアンス

---

このアーキテクチャ設計により、Backup Suiteは高性能・高信頼性・高拡張性を実現し、ユーザーのニーズに長期的に対応できるシステムとなっています。

## 📞 技術的サポート

アーキテクチャや実装について技術的な質問がある場合：

1. **GitHub Issues**: [技術的質問](https://github.com/user/backup-suite/issues)
2. **GitHub Discussions**: [アーキテクチャ議論](https://github.com/user/backup-suite/discussions)
3. **Developer Email**: dev@backup-suite.example.com

---

**関連ドキュメント**: [USAGE.md](USAGE.md) | [MIGRATION.md](MIGRATION.md) | [TROUBLESHOOTING.md](TROUBLESHOOTING.md)