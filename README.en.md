# backup-suite

[![Crates.io](https://img.shields.io/crates/v/backup-suite.svg)](https://crates.io/crates/backup-suite)
[![Documentation](https://docs.rs/backup-suite/badge.svg)](https://docs.rs/backup-suite)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.82%2B-blue.svg)](https://www.rust-lang.org)
[![CI](https://github.com/sanae-abe/backup-suite/workflows/CI/badge.svg)](https://github.com/sanae-abe/backup-suite/actions)

[日本語](README.md) | [English](README.en.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md)

> **Fast, Secure & Intelligent Local Backup Tool**

## Table of Contents

- [Key Features](#key-features)
- [Screenshots](#screenshots)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Basic Usage](#basic-usage)
- [Smart Features (Intelligent Backup)](#-smart-features-intelligent-backup)
- [Configuration File](#configuration-file)
- [Scheduling Features](#scheduling-features)
- [Command Reference](#command-reference)
- [Update & Uninstall](#update--uninstall)
- [Security & Quality](#security--quality)
- [Technology Stack](#technology-stack)
- [Supported Platforms](#supported-platforms)
- [License](#license)

## Key Features

### 🎯 Priority-Based Backup Management
- **Important work files** automatically backed up daily
- **Photos and personal files** backed up weekly
- **Archive files** backed up monthly

### 🤖 Smart-Driven Intelligent Management
- **Auto-Optimization**: Automatically generate optimal backup configuration through directory analysis
- **File Importance Analysis**: Automatically classify files in directories by importance level (~8s/10,000 files)
- **Exclude Pattern Suggestions**: Auto-detect and suggest exclusion of unnecessary files (cache, build artifacts)
- **Anomaly Detection**: Automatically detect backup size anomalies using statistical analysis (< 1ms)
- **Fully Offline**: All Smart features run locally, complete privacy protection

### 🔐 Military-Grade Encryption Protection
- **AES-256-GCM encryption** virtually impossible to decrypt
- **Argon2 key derivation** securely generates encryption keys from passwords
- **Data completely safe** even if computer is stolen
- **Third parties cannot access** when stored in cloud
- **Strong password auto-generation** ensures security

### 📦 High-Speed Compression for Storage Savings
- **Zstd compression** for fast and high compression ratio
- **Gzip compression** for compatibility focus
- **No compression** option also available
- **Reduce disk usage by up to 70%**

### ⚡ Incremental Backup for Ultra-Fast Performance
- **Backup only changed files** for massive time savings
- **SHA-256 hash-based** accurate change detection
- **90% faster backup time** (from 2nd run onwards)
- **85% storage reduction** (only differences saved)
- **Automatic fallback to full backup** (on first run)

### ⏰ Fully Automated Scheduling
- **No manual operation required** after setup - runs automatically
- **Frequency adjusted by importance** (daily/weekly/monthly)
- **Completely prevents forgotten backups**
- **macOS launchd/Linux systemd integration** for reliable automated execution

### 📊 Clear Management and Maintenance
- **Check backup statistics** to see how much has been backed up
- **View execution history** to see when backups ran
- **Automatically delete old backups** to save disk space
- **Easy restoration** when data is corrupted

### 🌍 International Language Support
- **4 languages fully supported**: English, Japanese (日本語), Simplified Chinese (简体中文), Traditional Chinese (繁體中文)
- **Automatic language detection**: Auto-detected from `LANG` environment variable (supports `ja`, `en`, `zh-CN`, `zh-TW`, etc.)
- **Complete translations**: All CLI output, error messages, and help text available in each language

### 💡 User-Friendly CLI
- **Typo correction suggestions**: Automatically detects command name typos and suggests the correct command
  ```bash
  $ backup-suite restor
  error: unrecognized subcommand 'restor'

  Did you mean 'restore'?

  For more information, try '--help'.
  ```
- **Intelligent edit distance algorithm**: Uses Levenshtein distance to automatically detect similar commands (up to 2 character differences)
- **Color support**: Automatically adjusts based on terminal color capabilities

## Screenshots

### Help Screen
<img src="docs/screenshots/help.webp" alt="backup-suite help" width="600">

*Display command list and options in Japanese*

### Backup Target List
<img src="docs/screenshots/list.webp" alt="backup-suite list" width="600">

*Display registered backup targets in table format*

### Backup Execution
<img src="docs/screenshots/run.webp" alt="backup-suite run" width="600">

*Actual backup execution screen*

### Backup Execution (Dry Run)
<img src="docs/screenshots/dry-run.webp" alt="backup-suite dry-run" width="600">

*Check execution content without actually copying files*

### Backup History
<img src="docs/screenshots/history.webp" alt="backup-suite history" width="600">

*Check past backup execution history*

## Installation

### Install via Homebrew (macOS)

```bash
brew tap sanae-abe/backup-suite
brew install backup-suite
```

### Install via Cargo

```bash
# Install with Smart features enabled (recommended)
cargo install backup-suite --features smart

# Install without Smart features (lightweight version)
cargo install backup-suite
```

### Build from Source

```bash
# 1. Clone repository
git clone git@github.com:sanae-abe/backup-suite.git
cd backup-suite

# 2. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 3. Build & Install (with Smart features)
cargo build --release --features smart
cargo install --path . --features smart

# 4. Verify operation
backup-suite --version
```

### 🌍 Shell Completion (Multilingual Support)

Shell completion is supported in **4 languages**: English, Japanese, Simplified Chinese, Traditional Chinese.

#### Quick Setup (Zsh)

```bash
# 1. Create completion directory
mkdir -p ~/.zfunc

# 2. Add to ~/.zshrc
echo 'fpath=(~/.zfunc $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc

# 3. Generate completion (auto-detects language from $LANG)
backup-suite completion zsh > ~/.zfunc/_backup-suite

# 4. Restart shell
exec zsh
```

**Manual language selection**:

```bash
# Japanese
./scripts/generate-completion.sh ja

# Simplified Chinese
./scripts/generate-completion.sh zh-CN

# Traditional Chinese
./scripts/generate-completion.sh zh-TW

# English
./scripts/generate-completion.sh en
```

**Troubleshooting**:

If completion doesn't work, see the comprehensive guide at [docs/shell-completion.md](docs/shell-completion.md). Common solutions:

- **No completion at all**: Restart shell (`exec zsh`), check file exists (`ls -la ~/.zfunc/_backup-suite`)
- **Wrong language displayed**: Check `echo $LANG`, or use `./scripts/generate-completion.sh en` to manually specify
- **compinit warnings**: Fix permissions (`chmod go-w ~/.zfunc`)

For Bash/Fish installation and detailed troubleshooting, refer to [docs/shell-completion.md](docs/shell-completion.md).

## Quick Start

### 1. Basic Setup
```bash
# Check current settings
backup-suite status

# Configuration file location
# ~/.config/backup-suite/config.toml
```

**Note**: Language is automatically detected from the `LANG` environment variable. Supported languages: English, Japanese, Simplified Chinese (简体中文), Traditional Chinese (繁體中文). It will automatically display in the appropriate language based on your system locale.

### 2. Configure Backup Destination

```bash
# Set Google Drive destination
backup-suite config set-destination "/Users/your-username/Library/CloudStorage/GoogleDrive-your@email.com/My Drive/backup-storage"

# ⚠️ IMPORTANT: Always enable encryption when backing up to cloud storage
# When storing backups on cloud storage like Google Drive,
# always use the --encrypt option to protect against unauthorized third-party access

# Check current settings
backup-suite config get-destination
```

### 3. Verify Configuration

To verify your configuration, use the `backup-suite status` command from [1. Basic Setup](#1-basic-setup).

## Basic Usage

1. **Add Files**
```bash
backup-suite add ~/Documents/project --priority high --category development
backup-suite add ~/Photos --priority medium --category personal
```

2. **Check Target List**
```bash
backup-suite list
backup-suite list --priority high  # High priority only
```

3. **Update Target Settings**
```bash
# Change category only
backup-suite update ~/.config --category "Config files"

# Change priority and category simultaneously
backup-suite update ~/.ssh --priority high --category "SSH config (private keys)"

# Add exclude patterns
backup-suite update ~/.ssh --exclude "known_hosts*" --exclude "*.old"

# Update multiple settings at once
backup-suite update ~/Documents --priority high --category "Important docs" --exclude "*.tmp"
```

4. **Execute Backup**
```bash
backup-suite run                   # Execute all targets
backup-suite run --priority high   # High priority only
backup-suite run --category work   # Specific category only
backup-suite run --dry-run         # Dry run (verification only)

# Incremental backup
backup-suite run --incremental     # Backup only changed files (recommended from 2nd run)

# Compression options
backup-suite run --compress zstd   # Zstd compression (fast, high ratio, recommended)
backup-suite run --compress gzip   # Gzip compression (compatibility focus)
backup-suite run --compress none   # No compression

# Encrypted backup (recommended: interactive password prompt)
backup-suite run --encrypt
# → Enter password securely via prompt (won't be saved in shell history)

# Or use environment variable (optional)
export BACKUP_SUITE_PASSWORD="your-secure-password"
backup-suite run --encrypt

# Compression + encryption combination
backup-suite run --compress zstd --encrypt
# → Enter password interactively via prompt
```

5. **Setup Automation**
```bash
# Set priority-based schedule
backup-suite schedule setup --high daily --medium weekly --low monthly
backup-suite schedule enable
```

## 🤖 Smart Features (Intelligent Backup)

Optimize your backups with statistical anomaly detection and file importance analysis.

### Installation

To use Smart features, you need to build with the `--features smart` flag.

```bash
# Build with Smart features enabled
cargo build --release --features smart
cargo install --path . --features smart

# Or install via Cargo
cargo install backup-suite --features smart
```

### Key Features

#### 1. Smart Auto-Configuration

Analyze directories and automatically generate optimal backup configuration.

```bash
# Auto-analyze and configure (evaluate each subdirectory individually)
backup-suite smart auto-configure ~/data

# Interactive mode (confirm each subdirectory and exclusion pattern)
backup-suite smart auto-configure ~/data --interactive

# Dry run (preview only, don't apply changes)
backup-suite smart auto-configure ~/data --dry-run

# Specify subdirectory scan depth (default: 1)
backup-suite smart auto-configure ~/data --max-depth 2

# Specify maximum number of subdirectories to process (default: 100)
backup-suite smart auto-configure ~/data --max-subdirs 50

# Increase subdirectory processing limit for large directory trees
backup-suite smart auto-configure ~/data --max-subdirs 200
```

**Features**:
- **Individual evaluation of each subdirectory** (optimal priority for each directory)
- **Automatic exclusion pattern detection** (auto-exclude `node_modules/`, `target/`, `.cache/`, etc.)
- **Project type auto-detection** (Rust, Node.js, Python, etc.)
- **Only patterns with 80%+ confidence applied** (prevents false positives)
- **Performance optimization with processing limit** (`--max-subdirs` for handling large directory trees, default: 100)

**Example Output**:
```
🤖 Smart Auto-Configuration
Analyzing: "/Users/user/projects"
  📁 Found 3 subdirectories: 3
    Evaluating: "/Users/user/projects/web-app"
      Recommended Priority: High (Score: 95)
      📋 Exclusion pattern suggestions: 3
        - node_modules (99.0%, 2.34 GB estimated reduction)
        - .cache (95.0%, 0.45 GB estimated reduction)
        - .*\.tmp$ (99.0%, 0.00 GB estimated reduction)
      📝 Exclusion patterns: node_modules, .cache, .*\.tmp$
      ✅ Added to configuration
    Evaluating: "/Users/user/projects/rust-cli"
      Recommended Priority: High (Score: 95)
      📋 Exclusion pattern suggestions: 2
        - target (99.0%, 1.87 GB estimated reduction)
        - .cache (95.0%, 0.12 GB estimated reduction)
      📝 Exclusion patterns: target, .cache
      ✅ Added to configuration
    Evaluating: "/Users/user/projects/archive"
      Recommended Priority: Low (Score: 30)
      ✅ Added to configuration

Auto-configuration completed
  Items added: 3
  Total reduction: 4.78 GB (approx. 35% faster backup time)
```

**When subdirectory limit is reached**:
```
🤖 Smart Auto-Configuration
Analyzing: "/Users/user/large-project"
  📁 Found 100 subdirectories: 100
  ⚠️  Limit reached, some subdirectories were not processed: 100 (use --max-subdirs to change)
```

#### 2. File Importance Analysis

Classify files in a directory by importance level to optimize backup strategy.

```bash
# Analyze directory importance
backup-suite smart analyze ~/documents

# Show detailed importance scores
backup-suite smart analyze ~/documents --detailed

# Analyze only specific file types
backup-suite smart analyze ~/projects --filter "*.rs,*.toml"
```

**Evaluation Criteria**:
- **High Importance (80-100 points)**: Source code, documents, configuration files
- **Medium Importance (40-79 points)**: Images, data files
- **Low Importance (0-39 points)**: Logs, temporary files

**Example Output**:
```
🤖 Smart File Importance Analysis: ~/Documents

┌─────────────────────────┬──────────────────┬──────────────┬─────────────────────┐
│ File/Directory          │ Importance Score │ Suggested    │ Reason              │
│                         │                  │ Priority     │                     │
├─────────────────────────┼──────────────────┼──────────────┼─────────────────────┤
│ src/                    │ ████████ 95      │ High         │ Source code (frequent updates) │
│ reports/                │ ████████ 90      │ High         │ Documents (important) │
│ photos/                 │ ████░░░░ 60      │ Medium       │ Image files         │
│ .cache/                 │ █░░░░░░░ 10      │ Exclude      │ Cache directory     │
└─────────────────────────┴──────────────────┴──────────────┴─────────────────────┘
```

**Performance**: ~8 seconds (10,000 files)

#### 3. Exclusion Pattern Suggestions

Automatically detect unnecessary files and suggest exclusion patterns.

```bash
# Show suggested exclusion patterns
backup-suite smart suggest-exclude ~/projects

# Automatically apply suggested patterns to config
backup-suite smart suggest-exclude ~/projects --apply

# Specify minimum file size (default: 100MB)
backup-suite smart suggest-exclude ~/projects --min-size 50MB
```

**Detection Targets**:
- Build artifacts (`target/`, `dist/`, `build/`)
- Dependency caches (`node_modules/`, `.cargo/`)
- Temporary files (`*.tmp`, `*.cache`)
- Large media files (above threshold size)

**Example Output**:
```
🤖 Smart Exclusion Pattern Suggestions: ~/projects

┌──────────────────┬──────────┬──────────┬─────────────────────┐
│ Pattern          │ Size     │ Confidence │ Reason                  │
│                  │ Saved    │            │                         │
├──────────────────┼──────────┼──────────┼─────────────────────┤
│ node_modules/    │ 2.34 GB  │ 99%      │ npm dependencies (regenerable) │
│ target/          │ 1.87 GB  │ 99%      │ Rust build artifacts    │
│ .cache/          │ 0.45 GB  │ 95%      │ Cache directory         │
└──────────────────┴──────────┴──────────┴─────────────────────┘

💡 Total Reduction: 4.66 GB (approx. 30% faster backup time)
```

#### 4. Anomaly Detection

Detect statistically abnormal backups from historical data.

```bash
# Detect anomalies in the last 7 days
backup-suite smart detect --days 7

# More detailed analysis (also shows statistics)
backup-suite smart detect --days 14 --detailed
```

**Detection Contents**:
- Backup size spikes/drops (Z-score statistical analysis)
- Disk capacity depletion prediction (linear regression)
- Failure pattern analysis (by category and time)

**Example Output**:
```
🤖 Smart Anomaly Detection Report (Last 7 Days)

┌────┬──────────────────┬──────────────┬────────────┬────────────────────────┐
│ No │ Detection Time   │ Anomaly Type │ Confidence │ Description            │
├────┼──────────────────┼──────────────┼────────────┼────────────────────────┤
│ 1  │ 2025-11-09 03:15 │ Size Surge   │ 95.3%      │ File size 3x normal    │
└────┴──────────────────┴──────────────┴────────────┴────────────────────────┘

📊 Summary: 1 anomaly detected
💡 Recommended Action: Add temporary files in ~/Downloads to exclusion settings
```

**Performance**: < 1ms (100 history entries)

### Disabling Smart Features

If Smart features are not needed, use the standard build.

```bash
# Standard build (without Smart features)
cargo build --release
cargo install --path .
```

### Security and Privacy

All Smart features operate **completely offline**:

- ✅ External API calls: None
- ✅ Cloud services: Not required
- ✅ Sensitive data transmission: Zero
- ✅ Data collection: None

For more details, see [Smart Features Documentation](docs/smart/features.md).

## Configuration File

### ~/.config/backup-suite/config.toml Example
```toml
[general]
log_level = "info"
log_file = "~/.local/share/backup-suite/logs/backup.log"

[storage]
type = "local"
path = "/Users/john/Library/CloudStorage/GoogleDrive-john@example.com/My Drive/backup-storage"  # When using cloud storage, encryption = true is required
compression = "zstd"  # Compression type: "zstd", "gzip", "none"
compression_level = 3  # Compression level: 1-22 (Zstd), 1-9 (Gzip)
encryption = true
encryption_key_file = "~/.config/backup-suite/keys/backup.key"  # Important: Protect with chmod 600

[schedule]
enabled = true
daily_time = "02:00"
weekly_day = "sunday"
monthly_day = 1

[targets]
[[targets.directories]]
name = "documents"
path = "~/Documents"
exclude = ["*.tmp", "*.cache", ".DS_Store"]

[[targets.directories]]
name = "projects"
path = "~/Projects"
exclude = ["node_modules/", "target/", ".git/", "*.log"]
```

## Command Reference

| Command        | Description               | Example                                         |
| -------------- | ------------------------- | ----------------------------------------------- |
| **add**        | Add backup target         | `backup-suite add ~/docs --priority high`       |
| **list, ls**   | Display target list       | `backup-suite list --priority medium`           |
| **remove**     | Remove target             | `backup-suite remove ~/old-files`               |
| **update**     | Update target settings    | `backup-suite update ~/.ssh --priority high --category "SSH config"` |
| **clear, rm**  | Bulk delete               | `backup-suite clear --priority low`             |
| **run**        | Execute backup            | `backup-suite run --encrypt`                    |
| **restore**    | Restore backup            | `backup-suite restore --from backup-20251104`   |
| **cleanup**    | Delete old backups        | `backup-suite cleanup --days 30`                |
| **status**     | Display current status    | `backup-suite status`                           |
| **history**    | Display execution history | `backup-suite history --days 7`                 |
| **schedule**   | Manage scheduling         | `backup-suite schedule enable`                  |
| **config**     | Manage configuration      | `backup-suite config set-destination ~/backups` |
| **open**       | Open backup directory     | `backup-suite open`                             |
| **completion** | Generate shell completion | `backup-suite completion zsh`                   |
| **smart**         | Smart features (requires `--features smart`) | `backup-suite smart detect --days 7`    |

## Update & Uninstall

### Update

```bash
# Homebrew
brew upgrade backup-suite

# Cargo
cargo install backup-suite --force --features smart

# From source
cd backup-suite
git pull origin main
cargo install --path . --force --features smart
```

### Uninstall

```bash
# 1. Remove binary
rm ~/.local/bin/backup-suite

# 2. Delete configuration files (optional)
rm -rf ~/.config/backup-suite/

# 3. Delete log files (optional)
rm -rf ~/.local/share/backup-suite/
```

## Security & Quality

### **Enterprise-Grade Security**
- **Encryption**: AES-256-GCM (Authenticated Encryption)
- **Key Derivation**: Argon2id (Memory cost 19MB, Iterations 2)
- **Nonce Collision Detection**: Auto-tracking in debug builds (zero overhead in release builds)
  - Tracks all Nonces (encryption initialization vectors) and immediately detects collisions
  - Provides detailed error messages explaining security impact when collision occurs
  - Completely removed at compile-time in release builds (no performance impact)
- **Path Traversal Protection**: Unicode normalization (NFKC), Null byte detection, Symlink attack prevention
- **Sensitive Data Erasure**: Zeroize usage (memory dump attack protection)
- **Local-only**: Cloud-independent security
- **Permission Management**: Proper configuration file permissions

### **Type Safety & Memory Safety**
- Minimize runtime errors with Rust's powerful type system
- Memory safety guarantee (prevents buffer overflow, memory leaks)
- Compile-time error detection

## Technology Stack

- **Language**: Rust (latest stable version)
- **CLI**: clap 4.x (command line parsing & completion generation)
- **Compression**: Zstd (fast & high ratio), Gzip (compatibility)
- **Encryption**: AES-256-GCM, Argon2
- **Configuration**: TOML (human-readable configuration format)
- **Scheduling**: macOS launchctl, Linux systemd
- **Smart/Statistical Analysis**: statrs (statistical computing), rayon (parallel processing)

## Supported Platforms

| OS      | Architecture  | Support Status |
| ------- | ------------- | -------------- |
| 🐧 Linux | x86_64        | ✅ Full support |
| 🐧 Linux | aarch64       | ✅ Full support |
| 🍎 macOS | x86_64        | ✅ Full support |
| 🍎 macOS | Apple Silicon | ✅ Full support |

## License

This project is licensed under the [MIT License](LICENSE).

---

## Contributing

Bug reports, feature requests, and pull requests are welcome!
Feel free to contact us via GitHub Issues or PRs.
