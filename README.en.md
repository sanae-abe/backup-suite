# Backup Suite

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-1.0.0-green.svg)](https://github.com/user/backup-suite/releases)

**🦀 Fast, Type-Safe, and Intelligent Local Backup Tool**

Backup Suite is a high-performance CLI tool **built with Rust**. It delivers efficient backup workflows through priority-based management, automatic scheduling, and interactive file selection.

## ✨ Key Features

### 🎯 **Priority-Based Backup Management**
```bash
backup-suite add ~/important-docs --priority high --category work
backup-suite add ~/photos --priority medium --category personal
backup-suite run --priority high  # Execute high-priority backups only
```

### 🎨 **Interactive File Selection (skim Integration)**
```bash
backup-suite add --interactive     # Select files with beautiful UI
backup-suite remove --interactive  # Select from existing targets to remove
```

### ⏰ **Automatic Scheduling (Full macOS launchctl Integration)**
```bash
backup-suite schedule setup --high daily --medium weekly --low monthly
backup-suite schedule enable      # Enable automatic execution for all priorities
backup-suite schedule status      # Check current settings
```

### 📊 **Comprehensive Management Features**
```bash
backup-suite dashboard            # Statistics dashboard
backup-suite history --days 30    # Execution history for the last 30 days
backup-suite cleanup --days 7     # Delete backups older than 7 days
backup-suite restore             # Restore from latest backup
```

## 🚀 Quick Start

### Installation

#### Method 1: Binary Download (Recommended)
```bash
# Download the latest release
curl -L https://github.com/user/backup-suite/releases/latest/download/backup-suite-macos-x86_64.tar.gz | tar xz

# Place in ~/.local/bin
mv backup-suite ~/.local/bin/
chmod +x ~/.local/bin/backup-suite
```

#### Method 2: Cargo (Rust)
```bash
cargo install backup-suite
```

#### Method 3: Build from Source
```bash
git clone https://github.com/user/backup-suite.git
cd backup-suite
cargo build --release
cp target/release/backup-suite ~/.local/bin/
```

### Initial Setup
```bash
# Shell completion setup (zsh)
backup-suite completion zsh > ~/.local/share/zsh/site-functions/_backup-suite

# Verify basic configuration
backup-suite status
```

### Basic Usage Examples

1. **Add Files**
```bash
backup-suite add ~/Documents/project --priority high --category development
backup-suite add ~/Photos --priority medium --category personal
```

2. **List Targets**
```bash
backup-suite list
backup-suite list --priority high  # High-priority only
```

3. **Execute Backup**
```bash
backup-suite run                   # Execute all targets
backup-suite run --priority high   # High-priority only
backup-suite run --category work   # Specific category only
backup-suite run --dry-run         # Dry run (verification only)
```

4. **Configure Automation**
```bash
# Set up priority-based schedules
backup-suite schedule setup --high daily --medium weekly --low monthly
backup-suite schedule enable
```

## 🏗️ Architecture

### **Configuration File**: `~/.config/backup-suite/config.toml`
```toml
[backup]
destination = "/Users/user/backup-suite/backups"
keep_days = 30

[[targets]]
path = "/Users/user/Documents/projects"
priority = "high"
category = "development"
```

### **Technology Stack**
- **Language**: Rust 1.70+ (type-safe, memory-safe, high-performance)
- **CLI**: clap 4.x (command-line parsing, completion generation)
- **UI**: skim (beautiful fuzzy finder integration)
- **Configuration**: TOML (human-readable configuration format)
- **Scheduling**: macOS launchctl (system-level automation)

## 📋 Complete Command Reference

| Command | Description | Example |
|----------|------|-----|
| **add** | Add backup target | `backup-suite add ~/docs --priority high` |
| **list, ls** | List targets | `backup-suite list --priority medium` |
| **remove** | Remove target | `backup-suite remove ~/old-files` |
| **clear, rm** | Batch removal | `backup-suite clear --priority low` |
| **run** | Execute backup | `backup-suite run --dry-run` |
| **restore** | Restore backup | `backup-suite restore --from backup-20251104` |
| **cleanup** | Delete old backups | `backup-suite cleanup --days 30` |
| **status** | Display current status | `backup-suite status` |
| **history** | Display execution history | `backup-suite history --days 7` |
| **dashboard** | Statistics dashboard | `backup-suite dashboard` |
| **schedule** | Manage scheduling | `backup-suite schedule enable --priority high` |
| **open** | Open backup directory | `backup-suite open` |
| **--version** | Version information | `backup-suite --version` |
| **completion** | Generate shell completion | `backup-suite completion zsh` |

## 🔧 Advanced Usage

### Interactive Workflow
```bash
# Add targets with file selection UI
backup-suite add --interactive

# Select from existing targets to remove
backup-suite remove --interactive

# Cleanup with verification
backup-suite cleanup --days 30 --dry-run
```

### Priority-Based Operation Strategy
```bash
# Critical files: Daily backups
backup-suite add ~/critical-data --priority high --category critical

# Regular files: Weekly backups
backup-suite add ~/documents --priority medium --category work

# Archives: Monthly backups
backup-suite add ~/old-projects --priority low --category archive
```

### Restore & Disaster Recovery
```bash
# Restore from latest backup
backup-suite restore

# Restore from specific date
backup-suite restore --from backup-20251104 --to ~/recovered-files

# Verify contents before restoration
backup-suite history
```

## 🛡️ Security & Quality

### **Type Safety**
- Minimize runtime errors with Rust's powerful type system
- Memory safety guarantees (prevents buffer overflows and memory leaks)
- Compile-time error detection

### **Data Protection**
- Local-only (cloud-independent)
- Proper permission management for configuration files
- Validation before backup execution

### **Testing & Quality Assurance**
```bash
# Quality verification in the project
cargo test                        # Unit tests
cargo clippy                      # Static analysis
cargo fmt --check                # Format verification
```


## 📚 Documentation

### 👥 User Documentation
- [📦 Installation Guide](docs/user/INSTALL.md) - Detailed installation instructions
- [📖 Usage Guide](docs/user/USAGE.md) - Comprehensive feature explanations

### 🛠️ Developer Documentation
- [🏗️ Architecture](docs/development/ARCHITECTURE.md) - System design & extensibility
- [🧪 Testing Guide](docs/development/TESTING_GUIDE.md) - Test execution methods & strategies
- [🔒 Security Guide](docs/development/SECURITY_QUICK_REFERENCE.md) - Security best practices
- [❓ Help System](docs/development/HELP_IMPLEMENTATION_SUMMARY.md) - Help feature implementation

## 🤝 Contributing

Contributions to Backup Suite are welcome!

### Development Environment Setup
```bash
git clone https://github.com/user/backup-suite.git
cd backup-suite
cargo build
cargo test
```

### How to Contribute
1. Report issues or propose features via Issues
2. Submit improvements or fixes via Pull Requests
3. Contribute to documentation improvements or translations
4. Share your user experience feedback

## 📄 License

MIT License - See [LICENSE](LICENSE) file for details

## 🚀 Roadmap

### v1.1.0 (Planned)
- [ ] Linux systemd integration
- [ ] Windows support
- [ ] Configuration file encryption
- [ ] Incremental backup functionality

## 📞 Support

- **GitHub Issues**: [Bug reports & feature requests](https://github.com/user/backup-suite/issues)
- **Discussions**: [Questions & idea sharing](https://github.com/user/backup-suite/discussions)
- **Email**: support@backup-suite.example.com

---

**🦀 Backup Suite - Fast, Safe, and Intelligent Backup Solution**
