# Usage Guide

Comprehensive guide to all features and practical usage of Backup Suite v1.0.0.

## 📋 Table of Contents

- [Core Concepts](#core-concepts)
- [Command Reference](#command-reference)
- [Practical Workflows](#practical-workflows)
- [Configuration Details](#configuration-details)
- [Advanced Usage](#advanced-usage)
- [Best Practices](#best-practices)

## 🎯 Core Concepts

### Priority System
Backup Suite manages backups with three priority levels:

| Priority | Purpose | Recommended Frequency | Examples |
|----------|---------|----------------------|----------|
| **high** | Critical/urgent files | Daily | Active projects, important documents |
| **medium** | Regular files | Weekly | Completed projects, photos |
| **low** | Archive | Monthly | Old files, reference materials |

### Category System
Organize files by purpose with custom categories:
- `development` - Development projects
- `work` - Work files
- `personal` - Personal files
- `creative` - Design/creative work
- `archive` - Archived files

### Target Types
- `file` - Single file
- `directory` - Directory (recursive)

## 📝 Command Reference

### `add` - Add Backup Target

#### Basic Syntax
```bash
backup-suite add [PATH] [OPTIONS]
```

#### Options
- `--priority <PRIORITY>` - Set priority (high/medium/low, default: medium)
- `--category <CATEGORY>` - Set category (default: user)
- `--interactive` - Interactive file selection mode

#### Usage Examples

```bash
# Basic addition
backup-suite add ~/Documents/project --priority high --category development

# Specify category
backup-suite add ~/Photos --priority medium --category personal

# Interactive selection (omit path or --interactive)
backup-suite add --interactive
backup-suite add  # Auto-switches to interactive mode when path is omitted

# Add current directory
backup-suite add . --priority high --category work

# Add multiple files (scripting)
for dir in ~/project1 ~/project2 ~/project3; do
    backup-suite add "$dir" --priority high --category development
done
```

#### Example Output
```bash
$ backup-suite add ~/Documents/important --priority high --category work
✅ Added: "/Users/user/Documents/important"

$ backup-suite add --interactive
# Launches skim interface
# Select file/directory with fuzzy finder
✅ Added: "/Users/user/selected/path"
```

---

### `list` (`ls`) - List Backup Targets

#### Basic Syntax
```bash
backup-suite list [OPTIONS]
backup-suite ls [OPTIONS]  # Alias
```

#### Options
- `--priority <PRIORITY>` - Show only specified priority

#### Usage Examples

```bash
# Show all targets
backup-suite list

# Show only high priority
backup-suite list --priority high

# Use alias
backup-suite ls --priority medium
```

#### Example Output
```bash
$ backup-suite list
📋 Backup Targets
1. "/Users/user/Documents/project" [High] development
2. "/Users/user/Photos" [Medium] personal
3. "/Users/user/Archive" [Low] archive
Total: 3 items

$ backup-suite list --priority high
📋 Backup Targets
1. "/Users/user/Documents/project" [High] development
Total: 1 item
```

---

### `remove` - Remove Backup Target

#### Basic Syntax
```bash
backup-suite remove [PATH] [OPTIONS]
```

#### Options
- `--interactive` - Interactive target selection mode

#### Usage Examples

```bash
# Remove by path
backup-suite remove ~/Documents/old-project

# Interactive removal
backup-suite remove --interactive

# Auto-switches to interactive mode when path is omitted
backup-suite remove
```

#### Example Output
```bash
$ backup-suite remove ~/Documents/old-project
✅ Removed: "/Users/user/Documents/old-project"

$ backup-suite remove --interactive
# Shows selection UI from existing targets
Select backup target to remove:
> /Users/user/Documents/project [High] development
  /Users/user/Photos [Medium] personal
  /Users/user/Archive [Low] archive
✅ Removed: "/Users/user/Documents/project"
```

---

### `clear` (`rm`) - Bulk Removal

#### Basic Syntax
```bash
backup-suite clear [OPTIONS]
backup-suite rm [OPTIONS]  # Alias
```

#### Options
- `--priority <PRIORITY>` - Remove all targets with specified priority
- `--all` - Remove all targets

#### Usage Examples

```bash
# Remove all low priority targets
backup-suite clear --priority low

# Remove all targets (caution!)
backup-suite clear --all

# Use alias
backup-suite rm --priority medium
```

#### Example Output
```bash
$ backup-suite clear --priority low
✅ Removed 2 items

$ backup-suite clear --all
✅ Removed 5 items
```

---

### `run` - Execute Backup

#### Basic Syntax
```bash
backup-suite run [OPTIONS]
```

#### Options
- `--priority <PRIORITY>` - Execute only specified priority
- `--category <CATEGORY>` - Execute only specified category
- `--dry-run` - Dry run (verify without executing)
- `--encrypt` - Enable AES-256-GCM encryption
- `--password <PASSWORD>` - Encryption password (prompts if omitted)
- `--compress <TYPE>` - Compression algorithm (zstd/gzip/none, default: zstd)
- `--compress-level <LEVEL>` - Compression level (zstd: 1-22, gzip: 1-9, default: 3)

#### Usage Examples

```bash
# Backup all targets
backup-suite run

# Backup only high priority
backup-suite run --priority high

# Backup specific category
backup-suite run --category development

# Encrypted backup (AES-256-GCM)
backup-suite run --encrypt --password "your-password"
backup-suite run --encrypt  # Password prompted

# Compressed backup (zstd fast compression)
backup-suite run --compress zstd --compress-level 3

# Compressed backup (gzip compatibility)
backup-suite run --compress gzip --compress-level 6

# Encrypted + compressed backup
backup-suite run --encrypt --compress zstd

# Dry run (verification only)
backup-suite run --dry-run

# Medium priority dry run
backup-suite run --priority medium --dry-run

# Encrypted + compressed + category specified
backup-suite run --encrypt --compress zstd --category work
```

#### Example Output
```bash
$ backup-suite run --priority high
🚀 Backup Execution
📊 Result: 150/150 succeeded, 25.67 MB

$ backup-suite run --dry-run
🚀 Backup Execution (Dry Run)
📋 Detected: 300 files

$ backup-suite run --encrypt --compress zstd
Encryption password: ****
🚀 Backup Execution (Encrypted, Compression: zstd)
📊 Result: 150/150 succeeded, 12.34 MB (compressed)

$ backup-suite run --category development
🚀 Backup Execution (Category: development)
📊 Result: 75/75 succeeded, 18.42 MB
```

---

### `restore` - Restore Backup

#### Basic Syntax
```bash
backup-suite restore [OPTIONS]
```

#### Options
- `--from <PATTERN>` - Specify source backup (pattern matching)
- `--to <PATH>` - Specify restore destination directory (default: ./.restored)
- `--password <PASSWORD>` - Decryption password (for encrypted backups, prompts if omitted)

#### Usage Examples

```bash
# Restore from latest backup
backup-suite restore

# Restore from specific date backup
backup-suite restore --from backup-20251104

# Specify custom restore destination
backup-suite restore --to ~/recovered-files

# Restore specific backup to specific location
backup-suite restore --from backup-20251104 --to ~/project-recovery

# Restore encrypted backup
backup-suite restore --password "your-password"
backup-suite restore --from backup-20251104 --password "your-password" --to ~/restored

# Encrypted backup (password prompt)
backup-suite restore  # Automatically prompts for password when encrypted files detected
```

#### Example Output
```bash
$ backup-suite restore
🔄 Restore started: "/Users/user/backup-suite/backups/backup-20251104-143000" → "./.restored/backup_20251104_143000"
✅ Backup restored to "./.restored/backup_20251104_143000"
  Restored files: 150 (encrypted: 0 files)

$ backup-suite restore --from backup-20251104 --to ~/recovered
🔄 Restore started: "/Users/user/backup-suite/backups/backup-20251104-143000" → "/Users/user/recovered/backup_20251104_143000"
✅ Backup restored to "/Users/user/recovered/backup_20251104_143000"
  Restored files: 150 (encrypted: 0 files)

$ backup-suite restore --password "my-password"
🔄 Restore started: "/Users/user/backup-suite/backups/backup-20251104-143000" → "./.restored/backup_20251104_143000"
✅ Backup restored to "./.restored/backup_20251104_143000"
  Restored files: 150 (encrypted: 150 files)
```

---

### `cleanup` - Remove Old Backups

#### Basic Syntax
```bash
backup-suite cleanup [OPTIONS]
```

#### Options
- `--days <DAYS>` - Remove backups older than specified days (default: 30)
- `--dry-run` - Dry run (verify without deleting)

#### Usage Examples

```bash
# Remove backups older than 30 days (default)
backup-suite cleanup

# Remove backups older than 7 days
backup-suite cleanup --days 7

# Dry run (verify targets)
backup-suite cleanup --days 30 --dry-run

# Remove backups older than 1 year
backup-suite cleanup --days 365
```

#### Example Output
```bash
$ backup-suite cleanup --days 7 --dry-run
🗑️ Deleting: "/Users/user/backup-suite/backups/backup-20251028-143000"
🗑️ Deleting: "/Users/user/backup-suite/backups/backup-20251029-143000"
✅ 2 items deleted (Dry Run)

$ backup-suite cleanup --days 7
🗑️ Deleting: "/Users/user/backup-suite/backups/backup-20251028-143000"
🗑️ Deleting: "/Users/user/backup-suite/backups/backup-20251029-143000"
✅ 2 items deleted
```

---

### `status` - Show Current Status

#### Basic Syntax
```bash
backup-suite status
```

#### Example Output
```bash
$ backup-suite status
📊 Status
  Destination: "/Users/user/backup-suite/backups"
  Targets: 15
    High: 5
    Medium: 7
    Low: 3
```

---

### `history` - Show Backup History

#### Basic Syntax
```bash
backup-suite history [OPTIONS]
```

#### Options
- `--days <DAYS>` - Number of days of history to show (default: 7)

#### Usage Examples

```bash
# Show last 7 days (default)
backup-suite history

# Show last 30 days
backup-suite history --days 30

# Show last 1 day
backup-suite history --days 1
```

#### Example Output
```bash
$ backup-suite history --days 7
📜 Backup History (7 days)
1. ✅ 2025-11-04 14:30:00
   /Users/user/backup-suite/backups/backup-20251104-143000: 150 files, 25.67 MB
2. ✅ 2025-11-03 14:30:00
   /Users/user/backup-suite/backups/backup-20251103-143000: 148 files, 25.23 MB
```

---

### `dashboard` - Statistics Dashboard

#### Basic Syntax
```bash
backup-suite dashboard
```

#### Example Output
```bash
$ backup-suite dashboard
╔═══════════════════════════════════════╗
║      Backup Suite Dashboard          ║
╚═══════════════════════════════════════╝

📊 Statistics
  Registered targets: 15 items
  Total backups: 45 times
  Success rate: 98.9%

📅 Latest Backup
  Date: 2025-11-04 14:30:00
  Files: 150
  Size: 25.67 MB
```

---

### `schedule` - Schedule Management

#### Basic Syntax
```bash
backup-suite schedule <ACTION> [OPTIONS]
```

#### Subcommands

##### `setup` - Configure Schedule
```bash
backup-suite schedule setup [OPTIONS]
```

**Options:**
- `--high <FREQUENCY>` - High priority execution frequency (default: daily)
- `--medium <FREQUENCY>` - Medium priority execution frequency (default: weekly)
- `--low <FREQUENCY>` - Low priority execution frequency (default: monthly)

**Frequency Options:**
- `daily` - Every day at 2:00 AM
- `weekly` - Every Sunday at 2:00 AM
- `monthly` - 1st day of each month at 2:00 AM
- `hourly` - Every hour (for development/testing)

```bash
# Default configuration
backup-suite schedule setup

# Custom frequency configuration
backup-suite schedule setup --high daily --medium weekly --low monthly

# Set all to weekly
backup-suite schedule setup --high weekly --medium weekly --low weekly
```

##### `enable` - Enable Automatic Backup
```bash
backup-suite schedule enable [OPTIONS]
```

**Options:**
- `--priority <PRIORITY>` - Enable only specific priority

```bash
# Enable all priorities
backup-suite schedule enable

# Enable only high priority
backup-suite schedule enable --priority high

# Enable only medium priority
backup-suite schedule enable --priority medium
```

##### `disable` - Disable Automatic Backup
```bash
backup-suite schedule disable [OPTIONS]
```

**Options:**
- `--priority <PRIORITY>` - Disable only specific priority

```bash
# Disable all priorities
backup-suite schedule disable

# Disable only high priority
backup-suite schedule disable --priority high
```

##### `status` - Check Schedule Status
```bash
backup-suite schedule status
```

#### Example Output
```bash
$ backup-suite schedule setup --high daily --medium weekly --low monthly
📅 High priority schedule configured: daily
📅 Medium priority schedule configured: weekly
📅 Low priority schedule configured: monthly

$ backup-suite schedule enable
✅ Automatic backup enabled

$ backup-suite schedule status
📅 Schedule Configuration
  Enabled: ✅
  High priority: daily
  Medium priority: weekly
  Low priority: monthly

📋 Actual Schedule Status
  high: ✅ Enabled
  medium: ✅ Enabled
  low: ✅ Enabled
```

---

### `config` - Configuration Management

#### Basic Syntax
```bash
backup-suite config <ACTION> [ARGS]
```

#### Subcommands

##### `set-destination` - Change Backup Destination
```bash
backup-suite config set-destination <PATH>
```

**Arguments:**
- `<PATH>` - New backup destination directory path (supports tilde expansion)

```bash
# Change to external HDD
backup-suite config set-destination /Volumes/ExternalHDD/backups

# Change to home directory (tilde expansion)
backup-suite config set-destination ~/Documents/backups

# Change to NAS
backup-suite config set-destination /mnt/nas/backup-suite
```

##### `get-destination` - Show Current Backup Destination
```bash
backup-suite config get-destination
```

```bash
$ backup-suite config get-destination
📁 Current Backup Destination
  "/Users/user/backup-suite/backups"
```

##### `set-keep-days` - Change Backup Retention Period
```bash
backup-suite config set-keep-days <DAYS>
```

**Arguments:**
- `<DAYS>` - Backup retention days (1-3650 days)

```bash
# Change retention to 60 days
backup-suite config set-keep-days 60

# Change retention to 1 year
backup-suite config set-keep-days 365

# Change retention to minimum (1 day)
backup-suite config set-keep-days 1
```

##### `get-keep-days` - Show Current Backup Retention Period
```bash
backup-suite config get-keep-days
```

```bash
$ backup-suite config get-keep-days
📅 Current Backup Retention Period
  30 days
```

##### `open` - Open Configuration File in Editor
```bash
backup-suite config open
```

**Behavior:**
- Opens with editor specified in `$EDITOR` or `$VISUAL` environment variable
- On macOS, uses `open` command (default editor) when environment variable is not set
- On Linux, falls back to `nano`
- On Windows, falls back to `notepad`

```bash
# Open with default editor
backup-suite config open

# Open with specified editor
EDITOR=vim backup-suite config open
EDITOR=code backup-suite config open  # VS Code
```

#### Example Output

```bash
$ backup-suite config set-destination ~/my-backups
📁 Directory does not exist. Creating: "/Users/user/my-backups"
✅ Backup destination changed
  Previous: "/Users/user/backup-suite/backups"
  New: "/Users/user/my-backups"

$ backup-suite config get-destination
📁 Current Backup Destination
  "/Users/user/my-backups"

$ backup-suite config set-keep-days 90
✅ Backup retention period changed
  Previous: 30 days
  New: 90 days

$ backup-suite config get-keep-days
📅 Current Backup Retention Period
  90 days

$ backup-suite config open
📝 Opening configuration file: "/Users/user/.config/backup-suite/config.toml"
# Configuration file opens in default editor
```

---

### `open` - Open Backup Directory

#### Basic Syntax
```bash
backup-suite open
```

#### Example Output
```bash
$ backup-suite open
📂 Opening: "/Users/user/backup-suite/backups"
# Directory opens in Finder on macOS
```

---

### `--version` - Version Information

#### Basic Syntax
```bash
backup-suite --version
```

#### Example Output
```bash
$ backup-suite --version
Backup Suite v1.0.0
🦀 Rust・Fast・Type-safe
```

---

### `--lang` - Language Setting

#### Basic Syntax
```bash
backup-suite --lang <LANGUAGE> [COMMAND]
```

#### Supported Languages
- `en` / `english` - English (default)
- `ja` / `japanese` / `日本語` - Japanese

#### Usage Examples
```bash
# Show help in English (default)
backup-suite --help
backup-suite --lang en --help

# Show help in Japanese
backup-suite --lang ja --help

# Show status in Japanese
backup-suite --lang ja status

# Execute backup in English
backup-suite --lang en run --priority high
```

#### Example Output
```bash
$ backup-suite --lang en --help
Backup Suite v1.0.0
Fast Local Backup Tool - Written in Rust, Type-safe, High-performance
...

$ backup-suite --lang ja --help
Backup Suite v1.0.0
高速ローカルバックアップツール - Rust製・型安全・高性能
...
```

**Notes**:
- Default language is English
- Environment variable `LANG` is ignored
- `--lang` flag can be used with all commands

---

### `completion` - Generate Shell Completion

#### Basic Syntax
```bash
backup-suite completion <SHELL>
```

#### Supported Shells
- `zsh`
- `bash`
- `fish`

#### Usage Examples
```bash
# Generate Zsh completion
backup-suite completion zsh > ~/.local/share/zsh/site-functions/_backup-suite

# Generate Bash completion
backup-suite completion bash > ~/.local/share/bash-completion/completions/backup-suite

# Generate Fish completion
backup-suite completion fish > ~/.config/fish/completions/backup-suite.fish
```

---

## 🎯 Practical Workflows

### Developer Workflow

```bash
# 1. Add current project with high priority
backup-suite add ~/projects/current-project --priority high --category development

# 2. Migrate completed project to medium priority
backup-suite remove ~/projects/current-project
backup-suite add ~/projects/current-project --priority medium --category development

# 3. Archive old projects with low priority
backup-suite add ~/projects/old-project --priority low --category archive

# 4. Automate daily high priority backups
backup-suite schedule setup --high daily
backup-suite schedule enable --priority high

# 5. Regular history check
backup-suite dashboard
backup-suite history --days 7
```

### Photographer Workflow

```bash
# 1. Manage current shooting session with high priority
backup-suite add ~/Photos/2025/current-session --priority high --category creative

# 2. Save edited photos with medium priority
backup-suite add ~/Photos/2025/edited --priority medium --category creative

# 3. Archive old photos
backup-suite add ~/Photos/2023 --priority low --category archive

# 4. Configure weekly creative backup
backup-suite schedule setup --high daily --medium weekly --low monthly
backup-suite schedule enable

# 5. Storage management
backup-suite cleanup --days 90  # Remove backups older than 3 months
```

### Team Development Workflow

```bash
# 1. Project-based management
backup-suite add ~/team-projects/project-alpha --priority high --category team-alpha
backup-suite add ~/team-projects/project-beta --priority medium --category team-beta

# 2. Personal workspace
backup-suite add ~/workspace --priority high --category personal-work

# 3. Documentation & configuration
backup-suite add ~/.config --priority medium --category config
backup-suite add ~/Documents/team-docs --priority medium --category documentation

# 4. Automation setup
backup-suite schedule setup --high daily --medium weekly
backup-suite schedule enable

# 5. Regular status check
backup-suite status
backup-suite history --days 3
```

### Disaster Recovery Workflow

```bash
# 1. Emergency data verification
backup-suite history --days 1

# 2. Priority restoration of critical data
backup-suite restore --from latest --to ~/emergency-recovery

# 3. Restore specific project
backup-suite restore --from backup-20251104 --to ~/project-recovery

# 4. Verification after restoration
ls -la ~/emergency-recovery
diff -r ~/original-data ~/emergency-recovery

# 5. Configuration restoration in new environment
backup-suite add ~/emergency-recovery --priority high --category recovery
backup-suite run --priority high
```

## ⚙️ Configuration Details

### Configuration File Location
- **Path**: `~/.config/backup-suite/config.toml`
- **Format**: TOML
- **Encoding**: UTF-8

### Configuration File Structure

#### Complete Configuration Example
```toml
version = "1.0.0"

[backup]
destination = "/Users/user/backup-suite/backups"
auto_cleanup = true
keep_days = 30

[schedule]
enabled = true
high_frequency = "daily"
medium_frequency = "weekly"
low_frequency = "monthly"

# Backup targets (multiple allowed)
[[targets]]
path = "/Users/user/Documents/critical-project"
priority = "high"
target_type = "directory"
category = "development"
added_date = "2025-11-04T12:45:18.998137Z"
exclude_patterns = ["node_modules", ".git", "*.log"]

[[targets]]
path = "/Users/user/Photos/2025"
priority = "medium"
target_type = "directory"
category = "creative"
added_date = "2025-11-04T13:20:45.123456Z"
exclude_patterns = ["*.tmp", "cache/"]

[[targets]]
path = "/Users/user/.zshrc"
priority = "high"
target_type = "file"
category = "config"
added_date = "2025-11-04T14:10:22.789012Z"
exclude_patterns = []
```

#### Section Details

##### `[backup]` Section
```toml
[backup]
destination = "/path/to/backup/directory"  # Backup destination
auto_cleanup = true                        # Auto-cleanup enabled
keep_days = 30                            # Retention days
```

##### `[schedule]` Section
```toml
[schedule]
enabled = true              # Scheduling feature enabled
high_frequency = "daily"    # High priority execution frequency
medium_frequency = "weekly" # Medium priority execution frequency
low_frequency = "monthly"   # Low priority execution frequency
```

##### `[[targets]]` Section (Array)
```toml
[[targets]]
path = "/absolute/path/to/target"           # Backup target path (absolute)
priority = "high"                           # Priority (high/medium/low)
target_type = "directory"                   # Type (file/directory)
category = "development"                    # Category
added_date = "2025-11-04T12:45:18.998137Z" # Added date (ISO 8601)
exclude_patterns = ["*.log", "cache/"]     # Exclude patterns (glob format)
```

### Configuration File Customization

#### Change Backup Destination
```toml
[backup]
destination = "/Volumes/External/backups"  # External drive
# or
destination = "/nas/backups"               # NAS
# or
destination = "~/custom-backup-location"   # Relative to home directory
```

#### Customize Schedule Frequency
```toml
[schedule]
high_frequency = "daily"     # Every day at 2:00 AM
medium_frequency = "weekly"  # Every Sunday at 2:00 AM
low_frequency = "monthly"    # 1st of each month at 2:00 AM
# Future support planned:
# high_frequency = "hourly"  # Every hour (for testing)
```

#### Configure Exclude Patterns
```toml
[[targets]]
path = "/Users/user/project"
exclude_patterns = [
    "node_modules",          # Node.js dependencies
    ".git",                  # Git history
    "*.log",                 # Log files
    "cache/",                # Cache directory
    ".DS_Store",             # macOS system files
    "*.tmp",                 # Temporary files
    "build/",                # Build artifacts
    "dist/"                  # Distribution build
]
```

### Configuration File Management

#### Backup Configuration Itself
```bash
# Add configuration file as backup target
backup-suite add ~/.config/backup-suite/config.toml --priority high --category config

# Manual backup
cp ~/.config/backup-suite/config.toml ~/.config/backup-suite/config.toml.backup
```

#### Validate Configuration
```bash
# Check configuration contents
backup-suite status

# Check target list
backup-suite list

# Check configuration file directly
cat ~/.config/backup-suite/config.toml
```

#### Migrate Configuration
```bash
# Copy configuration file (from another machine)
scp remote-machine:~/.config/backup-suite/config.toml ~/.config/backup-suite/

# Partial modification
# Manual edit if paths need updating
nano ~/.config/backup-suite/config.toml
```

## 🚀 Advanced Usage

### Batch Processing & Scripting

#### Bulk Project Addition Script
```bash
#!/bin/bash
# add-projects.sh

PROJECT_DIRS=(
    "$HOME/projects/active/project1"
    "$HOME/projects/active/project2"
    "$HOME/projects/active/project3"
)

for project in "${PROJECT_DIRS[@]}"; do
    if [[ -d "$project" ]]; then
        echo "Adding: $project"
        backup-suite add "$project" --priority high --category development
    else
        echo "Warning: $project not found"
    fi
done

echo "Projects added successfully"
backup-suite list --priority high
```

#### Regular Maintenance Script
```bash
#!/bin/bash
# maintenance.sh

echo "=== Backup Suite Maintenance ==="

# 1. Status check
echo "Current status:"
backup-suite status

# 2. Clean up old backups
echo "Cleaning up old backups..."
backup-suite cleanup --days 30

# 3. Recent history check
echo "Recent history:"
backup-suite history --days 3

# 4. Dashboard display
echo "Dashboard:"
backup-suite dashboard

echo "Maintenance completed"
```

### Environment Variable Configuration

#### Temporary Configuration Changes
```bash
# Temporarily use different backup destination
BACKUP_DESTINATION="/tmp/test-backup" backup-suite run --dry-run

# Enable debug mode
RUST_LOG=debug backup-suite status

# Disable color output
NO_COLOR=1 backup-suite list
```

### CI/CD Integration

#### GitHub Actions Usage Example
```yaml
name: Backup Important Files
on:
  schedule:
    - cron: '0 2 * * *'  # Every day at 2:00 AM UTC
  workflow_dispatch:

jobs:
  backup:
    runs-on: macos-latest
    steps:
      - name: Setup Backup Suite
        run: |
          curl -L https://github.com/user/backup-suite/releases/latest/download/backup-suite-macos-x86_64.tar.gz | tar xz
          chmod +x backup-suite
          sudo mv backup-suite /usr/local/bin/

      - name: Configure Targets
        run: |
          backup-suite add ${{ github.workspace }} --priority high --category ci

      - name: Run Backup
        run: |
          backup-suite run --priority high

      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: backup-results
          path: ~/backup-suite/backups/
```

### External Tool Integration

#### Integration with rsync
```bash
#!/bin/bash
# backup-and-sync.sh

# 1. Execute local backup
backup-suite run --priority high

# 2. Sync latest backup to remote
LATEST_BACKUP=$(ls -t ~/backup-suite/backups/ | head -1)
rsync -avz ~/backup-suite/backups/"$LATEST_BACKUP"/ remote-server:/backup/

echo "Local backup and remote sync completed"
```

#### Git Integration
```bash
#!/bin/bash
# git-backup-hook.sh
# Use as Git post-commit hook

# Automatically backup project after commit
PROJECT_PATH=$(git rev-parse --show-toplevel)

# Add if not already a target
if ! backup-suite list | grep -q "$PROJECT_PATH"; then
    backup-suite add "$PROJECT_PATH" --priority high --category development
fi

# Execute backup
backup-suite run --priority high
```

## 💡 Best Practices

### Priority Setting Guidelines

#### Appropriate Use of `high` Priority
```bash
# ✅ Appropriate
backup-suite add ~/current-work-project --priority high --category development
backup-suite add ~/.ssh --priority high --category security
backup-suite add ~/Documents/contracts --priority high --category legal

# ❌ Avoid
backup-suite add ~/Downloads --priority high  # Temporary files should be low priority
backup-suite add ~/Music --priority high      # Entertainment should be medium~low priority
```

#### Appropriate Use of `medium` Priority
```bash
# ✅ Appropriate
backup-suite add ~/Photos/2025 --priority medium --category personal
backup-suite add ~/Documents/references --priority medium --category reference
backup-suite add ~/.config --priority medium --category config
```

#### Appropriate Use of `low` Priority
```bash
# ✅ Appropriate
backup-suite add ~/Archive/old-projects --priority low --category archive
backup-suite add ~/Downloads --priority low --category temp
backup-suite add ~/Desktop/old-files --priority low --category cleanup
```

### Exclude Pattern Best Practices

#### Development Projects
```toml
[[targets]]
path = "/Users/user/projects/web-app"
exclude_patterns = [
    "node_modules",      # NPM dependencies
    ".git",             # Git history (large)
    "build",            # Build artifacts
    "dist",             # Distribution build
    "*.log",            # Log files
    ".env",             # Environment variables (sensitive)
    "coverage",         # Test coverage
    ".nyc_output"       # Coverage temp files
]
```

#### Creative/Design Projects
```toml
[[targets]]
path = "/Users/user/creative/video-project"
exclude_patterns = [
    "*.tmp",            # Temporary files
    "cache",            # Cache directory
    "render",           # Render temp files
    "*.autosave",       # Auto-save files
    ".DS_Store"         # macOS system files
]
```

### Scheduling Best Practices

#### Recommended Schedule Configuration
```bash
# Balanced configuration
backup-suite schedule setup --high daily --medium weekly --low monthly

# High frequency (important project period)
backup-suite schedule setup --high daily --medium daily --low weekly

# Low frequency (stable operation period)
backup-suite schedule setup --high weekly --medium monthly --low monthly
```

#### System Resource Considerations
```bash
# Lower frequency for large file counts
backup-suite schedule setup --high weekly --medium monthly --low monthly

# Higher frequency for critical periods
backup-suite schedule enable --priority high  # Enable only high priority
```

### Storage Management Best Practices

#### Regular Cleanup
```bash
# Weekly maintenance
backup-suite cleanup --days 7

# Monthly maintenance
backup-suite cleanup --days 30

# Quarterly maintenance
backup-suite cleanup --days 90
```

#### Capacity Monitoring
```bash
# Check backup directory size
du -sh ~/backup-suite/backups/

# Check individual backup sizes
ls -lah ~/backup-suite/backups/

# Check disk usage
df -h ~/backup-suite/
```

### Security Best Practices

#### Exclude Sensitive Files
```toml
[[targets]]
path = "/Users/user/projects"
exclude_patterns = [
    ".env",             # Environment variables
    "*.key",            # Private keys
    "*.pem",            # Certificates
    "config/secrets",   # Secret configuration
    "*.password",       # Password files
    "credentials.json"  # Credentials
]
```

#### Configuration File Permissions Management
```bash
# Check and modify configuration directory permissions
chmod 755 ~/.config/backup-suite/
chmod 644 ~/.config/backup-suite/config.toml

# Check backup directory permissions
chmod 755 ~/backup-suite/
chmod 755 ~/backup-suite/backups/
```

### Troubleshooting Prevention Best Practices

#### Regular Operation Verification
```bash
# Monthly checklist
backup-suite status                    # Configuration check
backup-suite list                      # Target check
backup-suite run --dry-run             # Dry run execution
backup-suite history --days 30         # History check
backup-suite dashboard                 # Statistics check
backup-suite schedule status           # Schedule check
```

#### Backup Verification
```bash
# Check latest backup
LATEST=$(ls -t ~/backup-suite/backups/ | head -1)
ls -la ~/backup-suite/backups/"$LATEST"/

# Random file integrity check
diff ~/original-file ~/backup-suite/backups/"$LATEST"/original-file
```

#### Configuration Version Control
```bash
# Manage configuration file with Git
cd ~/.config/backup-suite/
git init
git add config.toml
git commit -m "Initial backup-suite configuration"

# Commit when modified
git add config.toml
git commit -m "Update backup targets for new project"
```

---

## 📞 Support & Contact

If you have questions about usage:

1. **GitHub Issues**: [Questions & Bug Reports](https://github.com/user/backup-suite/issues)
2. **Discussions**: [Community Support](https://github.com/user/backup-suite/discussions)
3. **Documentation**: [Other Documentation](../README.md#documentation)

---

**Next Steps**: For more technical details, see [Architecture Documentation](../development/ARCHITECTURE.md).
