# backup-suite

[日本語](README.md) | [English](README.en.md) 

> **M3 Corporate Backup Solution**

## Table of Contents

- [Key Features](#key-features)
- [Screenshots](#screenshots)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Basic Usage](#basic-usage)
- [Configuration File](#configuration-file)
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

### 🔐 Military-Grade Encryption Protection
- **AES-256-GCM encryption** virtually impossible to decrypt
- **Data completely safe** even if computer is stolen
- **Third parties cannot access** when stored in cloud
- **Password required** - nobody can open without it

### 📦 High-Speed Compression for Storage Savings
- **Zstd compression** for fast and high compression ratio
- **Gzip compression** for compatibility focus
- **No compression** option also available
- **Reduce disk usage by up to 70%**

### ⏰ Fully Automated Scheduling
- **No manual operation required** after setup - runs automatically
- **Frequency adjusted by importance** (daily/weekly/monthly)
- **Completely prevents forgotten backups**

### 📊 Clear Management and Maintenance
- **Check backup statistics** to see how much has been backed up
- **View execution history** to see when backups ran
- **Automatically delete old backups** to save disk space
- **Easy restoration** when data is corrupted

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

### Method 1: Manual Installation (Recommended)

Download directly from the [release page](https://rendezvous.m3.com/sanae-abe/backup-suite/-/releases/v1.0.0).

```bash
# 1. Download binary
curl -LO "https://rendezvous.m3.com/sanae-abe/backup-suite/-/jobs/artifacts/v1.0.0/raw/backup-suite-complete-package.tar.gz?job=package:create-distributions"

# 2. Extract
tar -xzf backup-suite-*.tar.gz

# 3. Move binary to appropriate location (no sudo required)
mkdir -p ~/.local/bin
mv backup-suite ~/.local/bin/

# 4. Add to PATH (first time only)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# 5. Verify operation
backup-suite --version
```

### Method 2: Local Installation

Clone the repository and run the installation script.

```bash
# 1. Clone repository
git clone ssh://git@rendezvous.m3.com:3789/sanae-abe/backup-suite.git
cd backup-suite

# 2. Run installation script
bash install.sh

# Or install system-wide (/usr/local/bin)
sudo bash install.sh

# 3. Verify operation
backup-suite --version
```

## Quick Start

### 1. Basic Setup
```bash
# Check current settings
backup-suite status

# Configuration file location
# ~/.config/backup-suite/config.toml
```

**Note**: Language is automatically detected from the `LANG` environment variable. It will automatically display in Japanese in Japanese environments.

### 2. Configure Backup Destination

```bash
# Set Google Drive destination
backup-suite config set-destination "/Users/your-username/Library/CloudStorage/GoogleDrive-your@email.com/My Drive/backup-storage"

# Check current settings
backup-suite config get-destination
```

### 3. Verify Configuration
```bash
# Check backup destination directory
backup-suite status
```

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

3. **Execute Backup**
```bash
backup-suite run                   # Execute all targets
backup-suite run --priority high   # High priority only
backup-suite run --category work   # Specific category only
backup-suite run --dry-run         # Dry run (verification only)

# Compression options
backup-suite run --compress zstd   # Zstd compression (fast, high ratio, recommended)
backup-suite run --compress gzip   # Gzip compression (compatibility focus)
backup-suite run --compress none   # No compression

# Encrypted backup
backup-suite run --encrypt --password "secure-password"

# Compression + encryption combination
backup-suite run --compress zstd --encrypt --password "secure-password"
```

4. **Setup Automation**
```bash
# Set priority-based schedule
backup-suite schedule setup --high daily --medium weekly --low monthly
backup-suite schedule enable
```

## Configuration File

### ~/.config/backup-suite/config.toml Example
```toml
[general]
log_level = "info"
log_file = "~/.local/share/backup-suite/logs/backup.log"

[storage]
type = "local"
path = "/Users/john/Library/CloudStorage/GoogleDrive-john@example.com/My Drive/backup-storage"
compression = "zstd"  # Compression type: "zstd", "gzip", "none"
compression_level = 3  # Compression level: 1-22 (Zstd), 1-9 (Gzip)
encryption = true
encryption_key_file = "~/.config/backup-suite/keys/backup.key"

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

## Update & Uninstall

### Update

```bash
# Manual installation of new version (recommended)
curl -LO "https://rendezvous.m3.com/sanae-abe/backup-suite/-/jobs/artifacts/v1.1.0/raw/backup-suite-complete-package.tar.gz?job=package:create-distributions"
tar -xzf backup-suite-*.tar.gz
mv backup-suite ~/.local/bin/
backup-suite --version
```

### Uninstall

```bash
# 1. Remove binary
sudo rm /usr/local/bin/backup-suite

# 2. Delete configuration files (optional)
rm -rf ~/.config/backup-suite/

# 3. Delete log files (optional)
rm -rf ~/.local/share/backup-suite/
```

## Security & Quality

### **Enterprise-Grade Security**
- AES-256-GCM encryption support
- Secure password-based key derivation (Argon2)
- Local-only (cloud-independent)
- Proper permission management for configuration files

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
**Developer**: sanae-abe@m3.com


