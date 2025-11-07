# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-11-07

### Added

#### Core Features
- Initial release
- `add` command: Add backup targets (with duplicate registration prevention)
- `list` / `ls` command: Display target list
- `remove` command: Remove targets
- `clear` command: Clear all targets
- `run` command: Execute backup
  - `--dry-run` flag support
  - `--priority` filter support
  - `--category` filter support
  - `--incremental` incremental backup support
  - `--encrypt` AES-256-GCM encryption support
  - `--password` password specification
  - `--generate-password` strong password auto-generation
  - `--compress` zstd/gzip compression support
  - `--compress-level` compression level adjustment
- `restore` command: Restore backup (with automatic incremental chain resolution)
- `cleanup` command: Delete old backups
- `history` command: Display backup history
- `status` command: Show status
- `dashboard` command: Statistics dashboard
- `schedule` command: Automatic backup schedule management (macOS launchctl support)
- `config` command: Configuration management (destination, retention period)
- `open` command: Open backup directory
- `completion` command: Generate shell completion scripts

#### Security Features
- **AES-256-GCM Encryption**: Authenticated encryption for sensitive data protection
- **Argon2 Key Derivation**: Secure password-based key generation
- **Password Policy**: Strength evaluation and auto-generation capabilities
- **Audit Log**: Event logging with HMAC-SHA256 tampering detection
- **Integrity Verification**: SHA-256 hash-based backup verification
- **Path Traversal Protection**: safe_join implementation
- **Secure Memory Management**: Sensitive data erasure with zeroize

#### Incremental Backup
- SHA-256 hash-based change detection
- Parent backup reference management
- Automatic incremental chain resolution (during restoration)
- Automatic full backup fallback on first run

#### Compression
- Zstd fast compression (levels 1-22, default 3)
- Gzip compatible compression (levels 1-9)
- Integrated processing pipeline

#### Usability
- Multi-language support (Japanese/English, auto-detection via LANG environment variable)
- Interactive file selection (skim integration)
- Progress bar display (indicatif)
- Colorful table display
- Comprehensive help documentation (18 detailed option descriptions)

#### Configuration Management
- TOML configuration file management
- Priority system (high/medium/low)
- Category classification
- Category-based directory structure (`backup_YYYYMMDD_HHMMSS/category/`)
- Automatic file/directory detection
- Exclusion patterns (regex and glob support)

### Fixed

- **Critical incremental backup bug fix**: Resolved issue where unchanged files were incorrectly copied
  - Save all file hashes to metadata (for next incremental comparison)
  - Copy only changed files (maintain performance)
- Added informational message for first incremental backup
- Internationalized password warnings (automatic Japanese/English switching)

### Performance

- 53.6x faster than Bash version (due to Rayon parallel processing)
- Memory-efficient streaming processing
- Runtime error reduction through type safety
- Differential copy optimization via incremental backup

### Documentation

- README.md: Comprehensive usage and feature documentation
- README.en.md: English documentation
- CHANGELOG.md: Detailed change history (Japanese)
- CHANGELOG.en.md: Detailed change history (English)
- PUBLISHING.md: Release procedures
- docs/user/VERIFICATION_CHECKLIST.md: User verification checklist (20 items)
- Comprehensive inline documentation and Rustdoc

### Tests

- 343 tests passed (2 ignored)
  - Unit tests: 135 passed
  - Integration tests: 16 passed
  - Audit tests: 13 passed
  - Incremental tests: 4 passed
  - Integrity tests: 5 passed
  - Nonce verification: 5 passed
  - Phase 2 integration: 9 passed
  - Property tests: 14 passed
  - Crypto property tests: 10 passed
  - Security property tests: 13 passed
  - Security tests: 23 passed
  - Doc tests: 96 passed

## [Unreleased]

### Planned

- Remote backup support (SSH/S3/WebDAV)
- Web UI implementation (browser-based management interface)
- Plugin system (custom handler extensions)
- Cloud backup support (AWS S3/Google Cloud Storage/Azure Blob)
- Snapshot functionality (Btrfs/ZFS integration)
- Data deduplication
- Multi-version backup (Git-like history management)
