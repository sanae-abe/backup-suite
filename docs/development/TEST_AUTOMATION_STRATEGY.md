# Test Automation Strategy - Backup Suite

## 📋 Executive Summary

**Project**: backup-suite v1.0.0
**Test Framework**: Rust native testing + specialized frameworks
**Current Coverage**: ~30% (basic unit tests)
**Target Coverage**: 80% (comprehensive testing)
**Timeline**: 6 weeks (aligned with IMPROVEMENT_PLAN.md)

---

## 🎯 Testing Objectives

### Primary Goals
1. **Security Assurance**: 100% coverage of security-critical paths (path traversal, permissions)
2. **Reliability**: Comprehensive testing of backup/restore workflows
3. **Performance**: Validate 50x speed improvement claims with benchmarks
4. **Regression Prevention**: Automated testing in CI/CD pipeline
5. **TDD Support**: Enable test-first development for new features

### Quality Targets

| Metric | Current | Phase 2 | Phase 5 (Final) |
|--------|---------|---------|-----------------|
| Unit Test Coverage | 30% | 60% | 80% |
| Integration Test Coverage | 0% | 40% | 60% |
| Security Test Coverage | 0% | 90% | 100% |
| Performance Tests | 0 | 3 | 8 |
| E2E Scenarios | 0 | 5 | 12 |

---

## 🏗️ Test Architecture

### Framework Selection

#### 1. Unit Testing
- **Primary**: Built-in Rust test framework (`#[test]`, `#[cfg(test)]`)
- **Rationale**: Native, zero-dependency, excellent IDE integration
- **Location**: Inline in `src/` files and `tests/` directory

#### 2. Property-Based Testing
- **Framework**: `proptest` 1.4+
- **Use Cases**: Path sanitization, configuration validation, edge case discovery
- **Coverage**: Security-critical functions

#### 3. Integration Testing
- **Framework**: Custom test harness with `tempfile` for isolation
- **Scope**: Full backup workflows, CLI commands, configuration management
- **Isolation**: Each test gets temporary filesystem

#### 4. Performance Testing
- **Framework**: `criterion` 0.5+
- **Metrics**: Throughput, latency, memory usage, CPU utilization
- **Baseline**: Establish performance regression detection

#### 5. Mutation Testing
- **Framework**: `cargo-mutants` 24.x
- **Purpose**: Test quality validation, identify weak tests
- **Frequency**: Weekly during development

#### 6. Fuzzing
- **Framework**: `cargo-fuzz` (libFuzzer)
- **Targets**: Config parsing, path handling, input sanitization
- **Duration**: Continuous integration runs

---

## 📊 Test Pyramid Implementation

```
        ┌─────────────┐
        │   E2E (5%)  │  Full CLI workflows, user scenarios
        │   12 tests  │
        └─────────────┘
       ┌───────────────┐
       │ Integration   │  Module interactions, file I/O
       │   (25%)       │  Component integration
       │   60 tests    │
       └───────────────┘
      ┌─────────────────┐
      │  Unit (70%)     │  Function-level, logic validation
      │  168 tests      │  Fast, isolated, comprehensive
      └─────────────────┘
```

### Test Distribution Strategy
- **70% Unit Tests**: Fast feedback, logic validation, TDD enablement
- **25% Integration Tests**: Component interaction, I/O operations
- **5% E2E Tests**: Critical user workflows, CLI validation

---

## 🔒 Security Testing Strategy

### Phase 1: Critical Security Tests (Week 1)

#### 1.1 Path Traversal Protection
```rust
// tests/security/path_traversal_tests.rs
use backup_suite::security::safe_join;
use std::path::{Path, PathBuf};

#[test]
fn test_path_traversal_parent_dir_attack() {
    let base = PathBuf::from("/tmp/backup");
    let malicious = Path::new("../../../etc/passwd");

    let result = safe_join(&base, malicious);
    assert!(result.is_err(), "Should reject parent directory traversal");
}

#[test]
fn test_path_traversal_absolute_path_attack() {
    let base = PathBuf::from("/tmp/backup");
    let malicious = Path::new("/etc/passwd");

    let result = safe_join(&base, malicious);
    assert!(result.is_err(), "Should reject absolute path attacks");
}

#[test]
fn test_path_traversal_symlink_escape() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    let base = temp.path().join("backup");
    std::fs::create_dir_all(&base).unwrap();

    // Create symlink pointing outside base
    let symlink = base.join("escape");
    std::os::unix::fs::symlink("/etc", &symlink).unwrap();

    let result = safe_join(&base, Path::new("escape/passwd"));
    assert!(result.is_err(), "Should reject symlink escape attempts");
}

#[test]
fn test_path_traversal_unicode_normalization() {
    // Test Unicode normalization attacks
    let base = PathBuf::from("/tmp/backup");

    // U+2215 (DIVISION SLASH) vs U+002F (SOLIDUS)
    let unicode_attack = Path::new("\u{2215}etc\u{2215}passwd");

    let result = safe_join(&base, unicode_attack);
    assert!(result.is_err() || !result.unwrap().starts_with("/etc"),
            "Should handle Unicode normalization");
}

#[test]
fn test_path_component_sanitization() {
    use backup_suite::security::sanitize_path_component;

    assert_eq!(sanitize_path_component("normal.txt"), "normal.txt");
    assert_eq!(sanitize_path_component("../../../etc"), "etc");
    assert_eq!(sanitize_path_component("fi<le>na:me"), "filename");
    assert_eq!(sanitize_path_component("test\0null"), "testnull");
}
```

#### 1.2 Permission Validation Tests
```rust
// tests/security/permission_tests.rs
use backup_suite::security::{check_read_permission, check_write_permission};
use tempfile::TempDir;
use std::fs;
use std::os::unix::fs::PermissionsExt;

#[test]
#[cfg(unix)]
fn test_read_permission_denied() {
    let temp = TempDir::new().unwrap();
    let no_read_file = temp.path().join("no_read.txt");
    fs::write(&no_read_file, "secret").unwrap();

    // Remove read permission
    let mut perms = fs::metadata(&no_read_file).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&no_read_file, perms).unwrap();

    let result = check_read_permission(&no_read_file);
    assert!(result.is_err(), "Should detect missing read permission");
}

#[test]
fn test_write_permission_to_readonly_directory() {
    let temp = TempDir::new().unwrap();
    let readonly_dir = temp.path().join("readonly");
    fs::create_dir(&readonly_dir).unwrap();

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o555); // r-xr-xr-x
        fs::set_permissions(&readonly_dir, perms).unwrap();
    }

    let test_file = readonly_dir.join("test.txt");
    let result = check_write_permission(&test_file);

    assert!(result.is_err(), "Should detect write permission failure");
}

#[test]
fn test_permission_race_condition() {
    // TOCTOU (Time-of-check to Time-of-use) protection
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("race.txt");
    fs::write(&file, "data").unwrap();

    // Simulate permission change between check and use
    let check_result = check_read_permission(&file);
    assert!(check_result.is_ok());

    // Change permissions
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&file).unwrap().permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&file, perms).unwrap();
    }

    // Verify actual operation fails gracefully
    let read_result = fs::read_to_string(&file);
    assert!(read_result.is_err());
}
```

#### 1.3 Input Validation Tests
```rust
// tests/security/input_validation_tests.rs
use backup_suite::core::{Config, Target, Priority};
use std::path::PathBuf;

#[test]
fn test_config_validation_malformed_toml() {
    let malformed_toml = r#"
        [backup]
        destination = "/tmp/backup"
        keep_days = "not_a_number"  # Should be integer
    "#;

    let result: Result<Config, _> = toml::from_str(malformed_toml);
    assert!(result.is_err(), "Should reject malformed configuration");
}

#[test]
fn test_config_validation_invalid_keep_days() {
    let mut config = Config::default();
    config.backup.keep_days = 0; // Invalid

    let result = config.validate();
    assert!(result.is_err(), "Should reject keep_days = 0");

    config.backup.keep_days = 10000; // Too large
    let result = config.validate();
    assert!(result.is_err(), "Should reject excessive keep_days");
}

#[test]
fn test_target_path_injection() {
    let injection_paths = vec![
        "/tmp/backup\0/etc/passwd",  // Null byte injection
        "/tmp/backup/../../../etc/passwd",  // Path traversal
        "/tmp/backup; rm -rf /",  // Command injection
    ];

    for malicious_path in injection_paths {
        let result = std::panic::catch_unwind(|| {
            Target::new(
                PathBuf::from(malicious_path),
                Priority::High,
                "test".to_string()
            )
        });

        // Should either panic or create safe path
        if let Ok(target) = result {
            assert!(!target.path.to_string_lossy().contains('\0'));
            assert!(!target.path.to_string_lossy().contains(';'));
        }
    }
}
```

### Property-Based Security Testing
```rust
// tests/security/property_tests.rs
use proptest::prelude::*;
use backup_suite::security::safe_join;

proptest! {
    #[test]
    fn prop_safe_join_never_escapes_base(
        base_components in prop::collection::vec("[a-zA-Z0-9_-]+", 1..5),
        child_components in prop::collection::vec("[a-zA-Z0-9._-]+", 1..10),
    ) {
        let base = PathBuf::from("/tmp").join(base_components.join("/"));
        let child = PathBuf::from(child_components.join("/"));

        if let Ok(result) = safe_join(&base, &child) {
            // Property: Result must always be within base directory
            assert!(
                result.starts_with(&base) || result == base,
                "safe_join result {:?} escaped base {:?}",
                result, base
            );
        }
    }

    #[test]
    fn prop_sanitize_never_produces_dangerous_chars(
        input in "[\\PC]*"  // Any Unicode string
    ) {
        use backup_suite::security::sanitize_path_component;

        let sanitized = sanitize_path_component(&input);

        // Properties: No dangerous characters
        assert!(!sanitized.contains('/'));
        assert!(!sanitized.contains('\\'));
        assert!(!sanitized.contains('\0'));
        assert!(!sanitized.contains(':'));
        assert!(!sanitized.contains('<'));
        assert!(!sanitized.contains('>'));
        assert!(!sanitized.contains('|'));
    }
}
```

---

## 🧪 Unit Testing Strategy

### Phase 2: Comprehensive Unit Tests (Week 2-3)

#### 2.1 Core Module Tests

##### Config Management
```rust
// src/core/config.rs - Enhanced tests
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_roundtrip_serialization() {
        let mut config = Config::default();
        config.add_target(Target::new(
            PathBuf::from("/tmp/test"),
            Priority::High,
            "test".to_string()
        ));

        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(config.version, deserialized.version);
        assert_eq!(config.targets.len(), deserialized.targets.len());
    }

    #[test]
    fn test_config_migration_old_version() {
        // Test backward compatibility with old config versions
        let old_config = r#"
            version = "0.9.0"
            [backup]
            destination = "/tmp/backup"
            keep_days = 30

            [[targets]]
            path = "/tmp/test"
            priority = "high"
            category = "test"
        "#;

        let config: Result<Config, _> = toml::from_str(old_config);
        assert!(config.is_ok(), "Should parse old config format");
    }

    #[test]
    fn test_config_save_creates_directory() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config/nested/config.toml");

        let mut config = Config::default();
        // Override config path for testing
        std::env::set_var("BACKUP_SUITE_CONFIG", config_path.to_str().unwrap());

        config.save().unwrap();
        assert!(config_path.exists());
    }

    #[test]
    fn test_filter_by_priority_empty_config() {
        let config = Config::default();
        let high = config.filter_by_priority(&Priority::High);

        assert_eq!(high.len(), 0);
    }

    #[test]
    fn test_filter_by_priority_multiple_targets() {
        let mut config = Config::default();
        config.add_target(Target::new(PathBuf::from("/tmp/a"), Priority::High, "a".into()));
        config.add_target(Target::new(PathBuf::from("/tmp/b"), Priority::Medium, "b".into()));
        config.add_target(Target::new(PathBuf::from("/tmp/c"), Priority::High, "c".into()));

        let high = config.filter_by_priority(&Priority::High);
        assert_eq!(high.len(), 2);

        let medium = config.filter_by_priority(&Priority::Medium);
        assert_eq!(medium.len(), 1);

        let low = config.filter_by_priority(&Priority::Low);
        assert_eq!(low.len(), 0);
    }
}
```

##### Backup Workflow Tests
```rust
// src/core/backup.rs - Enhanced tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_backup_preserves_directory_structure() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");

        // Create nested structure
        fs::create_dir_all(source.join("a/b/c")).unwrap();
        fs::write(source.join("a/file1.txt"), "content1").unwrap();
        fs::write(source.join("a/b/file2.txt"), "content2").unwrap();
        fs::write(source.join("a/b/c/file3.txt"), "content3").unwrap();

        let mut config = Config::default();
        config.backup.destination = temp.path().join("backup");
        config.add_target(Target::new(source.clone(), Priority::High, "test".into()));

        let runner = BackupRunner::new(config, false);
        let result = runner.run(None).unwrap();

        assert_eq!(result.total_files, 3);
        assert_eq!(result.success_files, 3);

        // Verify structure preserved
        assert!(temp.path().join("backup").exists());
        let backup_dirs = fs::read_dir(temp.path().join("backup")).unwrap();
        assert_eq!(backup_dirs.count(), 1); // One timestamped backup
    }

    #[test]
    fn test_backup_handles_concurrent_modifications() {
        use std::thread;
        use std::time::Duration;

        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("file.txt"), "initial").unwrap();

        let mut config = Config::default();
        config.backup.destination = temp.path().join("backup");
        config.add_target(Target::new(source.clone(), Priority::High, "test".into()));

        let runner = BackupRunner::new(config, false);

        // Simulate file modification during backup
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            let _ = fs::write(source.join("file.txt"), "modified");
        });

        let result = runner.run(None);
        assert!(result.is_ok(), "Should handle concurrent modifications gracefully");
    }

    #[test]
    fn test_backup_excludes_patterns() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        fs::create_dir_all(&source).unwrap();

        fs::write(source.join("include.txt"), "keep").unwrap();
        fs::write(source.join("exclude.tmp"), "skip").unwrap();
        fs::write(source.join("data.log"), "skip").unwrap();

        let mut config = Config::default();
        config.backup.destination = temp.path().join("backup");

        let mut target = Target::new(source.clone(), Priority::High, "test".into());
        target.exclude_patterns = vec![r".*\.tmp$".into(), r".*\.log$".into()];
        config.add_target(target);

        let runner = BackupRunner::new(config, false);
        let result = runner.run(None).unwrap();

        assert_eq!(result.total_files, 1, "Should exclude .tmp and .log files");
    }

    #[test]
    fn test_backup_large_file_handling() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("large.bin");

        // Create 100MB file
        let large_data = vec![0u8; 100 * 1024 * 1024];
        fs::write(&source, large_data).unwrap();

        let mut config = Config::default();
        config.backup.destination = temp.path().join("backup");
        config.add_target(Target::new(source.clone(), Priority::High, "test".into()));

        let runner = BackupRunner::new(config, false);
        let result = runner.run(None).unwrap();

        assert_eq!(result.total_files, 1);
        assert_eq!(result.success_files, 1);
        assert_eq!(result.total_bytes, 100 * 1024 * 1024);
    }

    #[test]
    fn test_backup_error_accumulation() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        fs::create_dir_all(&source).unwrap();

        // Create unreadable file
        let unreadable = source.join("unreadable.txt");
        fs::write(&unreadable, "data").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&unreadable).unwrap().permissions();
            perms.set_mode(0o000);
            fs::set_permissions(&unreadable, perms).unwrap();
        }

        let mut config = Config::default();
        config.backup.destination = temp.path().join("backup");
        config.add_target(Target::new(source.clone(), Priority::High, "test".into()));

        let runner = BackupRunner::new(config, false);
        let result = runner.run(None).unwrap();

        assert!(result.failed_files > 0);
        assert!(!result.errors.is_empty());
    }
}
```

#### 2.2 Test Data Generators
```rust
// tests/common/generators.rs
use backup_suite::core::{Config, Target, Priority};
use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestFixture {
    pub temp_dir: TempDir,
    pub source_dir: PathBuf,
    pub backup_dir: PathBuf,
}

impl TestFixture {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let backup_dir = temp_dir.path().join("backup");

        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::create_dir_all(&backup_dir).unwrap();

        Self { temp_dir, source_dir, backup_dir }
    }

    pub fn create_file(&self, relative_path: &str, content: &str) -> PathBuf {
        let file_path = self.source_dir.join(relative_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&file_path, content).unwrap();
        file_path
    }

    pub fn create_files(&self, count: usize) -> Vec<PathBuf> {
        (0..count)
            .map(|i| self.create_file(&format!("file_{}.txt", i), &format!("content {}", i)))
            .collect()
    }

    pub fn create_directory_tree(&self, depth: usize, files_per_dir: usize) {
        fn create_recursive(base: &PathBuf, current_depth: usize, max_depth: usize, files_per_dir: usize) {
            if current_depth > max_depth {
                return;
            }

            for i in 0..files_per_dir {
                std::fs::write(
                    base.join(format!("file_{}.txt", i)),
                    format!("depth {} file {}", current_depth, i)
                ).unwrap();
            }

            for i in 0..2 {
                let subdir = base.join(format!("dir_{}", i));
                std::fs::create_dir_all(&subdir).unwrap();
                create_recursive(&subdir, current_depth + 1, max_depth, files_per_dir);
            }
        }

        create_recursive(&self.source_dir, 1, depth, files_per_dir);
    }

    pub fn create_config(&self) -> Config {
        let mut config = Config::default();
        config.backup.destination = self.backup_dir.clone();
        config
    }
}

// Example usage in tests
#[test]
fn test_with_fixture() {
    let fixture = TestFixture::new();
    fixture.create_files(10);

    let mut config = fixture.create_config();
    config.add_target(Target::new(
        fixture.source_dir.clone(),
        Priority::High,
        "test".into()
    ));

    // Test implementation...
}
```

---

## 🔗 Integration Testing Strategy

### Phase 2-3: Integration Tests (Week 2-4)

#### 3.1 Full Workflow Integration Tests
```rust
// tests/integration_tests.rs
use backup_suite::core::{BackupRunner, Config, Target, Priority, BackupHistory};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_full_backup_restore_workflow() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backups");
    let restore_dest = temp.path().join("restored");

    // Setup source data
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("file1.txt"), "content1").unwrap();
    fs::create_dir_all(source.join("subdir")).unwrap();
    fs::write(source.join("subdir/file2.txt"), "content2").unwrap();

    // Backup
    let mut config = Config::default();
    config.backup.destination = backup_dest.clone();
    config.add_target(Target::new(source.clone(), Priority::High, "test".into()));

    let runner = BackupRunner::new(config.clone(), false);
    let backup_result = runner.run(None).unwrap();

    assert_eq!(backup_result.total_files, 2);
    assert_eq!(backup_result.success_files, 2);

    // Verify history saved
    let history = BackupHistory::load_all().unwrap();
    assert!(!history.is_empty());

    // Restore
    let backup_dirs = BackupHistory::list_backup_dirs().unwrap();
    let latest_backup = &backup_dirs[backup_dirs.len() - 1];

    copy_dir_all(latest_backup, &restore_dest).unwrap();

    // Verify restored files
    assert_eq!(
        fs::read_to_string(restore_dest.join("file1.txt")).unwrap(),
        "content1"
    );
    assert_eq!(
        fs::read_to_string(restore_dest.join("subdir/file2.txt")).unwrap(),
        "content2"
    );
}

#[test]
fn test_incremental_backup_workflow() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    fs::create_dir_all(&source).unwrap();

    let mut config = Config::default();
    config.backup.destination = temp.path().join("backups");
    config.add_target(Target::new(source.clone(), Priority::High, "test".into()));

    // First backup
    fs::write(source.join("file1.txt"), "v1").unwrap();
    let runner = BackupRunner::new(config.clone(), false);
    let result1 = runner.run(None).unwrap();
    assert_eq!(result1.total_files, 1);

    // Add more files
    std::thread::sleep(std::time::Duration::from_secs(2)); // Ensure different timestamp
    fs::write(source.join("file2.txt"), "v1").unwrap();

    let result2 = runner.run(None).unwrap();
    assert_eq!(result2.total_files, 2);

    // Verify multiple backup directories exist
    let backup_dirs = BackupHistory::list_backup_dirs().unwrap();
    assert!(backup_dirs.len() >= 2);
}

#[test]
fn test_priority_filtering_workflow() {
    let temp = TempDir::new().unwrap();
    let source_high = temp.path().join("high");
    let source_low = temp.path().join("low");

    fs::create_dir_all(&source_high).unwrap();
    fs::create_dir_all(&source_low).unwrap();
    fs::write(source_high.join("important.txt"), "critical").unwrap();
    fs::write(source_low.join("optional.txt"), "optional").unwrap();

    let mut config = Config::default();
    config.backup.destination = temp.path().join("backups");
    config.add_target(Target::new(source_high, Priority::High, "critical".into()));
    config.add_target(Target::new(source_low, Priority::Low, "optional".into()));

    // Backup only high priority
    let runner = BackupRunner::new(config, false);
    let result = runner.run(Some(&Priority::High)).unwrap();

    assert_eq!(result.total_files, 1);
}

#[test]
fn test_cleanup_old_backups_workflow() {
    let temp = TempDir::new().unwrap();
    let backup_dest = temp.path().join("backups");

    // Create old backup directories
    for i in 0..5 {
        let old_date = chrono::Utc::now() - chrono::Duration::days(40 - i);
        let dir_name = format!("backup_{}", old_date.format("%Y%m%d_%H%M%S"));
        let backup_dir = backup_dest.join(dir_name);
        fs::create_dir_all(&backup_dir).unwrap();
        fs::write(backup_dir.join("file.txt"), "data").unwrap();
    }

    let mut config = Config::default();
    config.backup.destination = backup_dest.clone();
    config.backup.keep_days = 30;

    // Cleanup old backups (>30 days)
    let cutoff = chrono::Utc::now() - chrono::Duration::days(30);
    let mut removed = 0;

    for dir in BackupHistory::list_backup_dirs().unwrap() {
        if let Ok(metadata) = fs::metadata(&dir) {
            if let Ok(modified) = metadata.modified() {
                let modified_time: chrono::DateTime<chrono::Utc> = modified.into();
                if modified_time < cutoff {
                    fs::remove_dir_all(&dir).unwrap();
                    removed += 1;
                }
            }
        }
    }

    assert!(removed > 0, "Should have removed old backups");
}
```

#### 3.2 CLI Integration Tests
```rust
// tests/cli_integration_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_cli_add_list_remove() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std::fs::write(&test_file, "data").unwrap();

    // Test add
    Command::cargo_bin("backup-suite").unwrap()
        .args(&["add", test_file.to_str().unwrap(), "--priority", "high"])
        .assert()
        .success()
        .stdout(predicate::str::contains("追加"));

    // Test list
    Command::cargo_bin("backup-suite").unwrap()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.txt"));

    // Test remove
    Command::cargo_bin("backup-suite").unwrap()
        .args(&["remove", test_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("削除"));
}

#[test]
fn test_cli_dry_run() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source.txt");
    std::fs::write(&source, "data").unwrap();

    Command::cargo_bin("backup-suite").unwrap()
        .args(&["add", source.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("backup-suite").unwrap()
        .args(&["run", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ドライラン"));
}

#[test]
fn test_cli_error_handling_invalid_priority() {
    Command::cargo_bin("backup-suite").unwrap()
        .args(&["add", "/tmp/test", "--priority", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("不明な優先度"));
}
```

---

## ⚡ Performance Testing Strategy

### Phase 3: Performance & Benchmarking (Week 4-5)

#### 4.1 Criterion Benchmarks
```rust
// benches/backup_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use backup_suite::core::{BackupRunner, Config, Target, Priority};
use tempfile::TempDir;
use std::fs;

fn create_test_files(dir: &std::path::Path, count: usize, size_bytes: usize) {
    for i in 0..count {
        let content = vec![0u8; size_bytes];
        fs::write(dir.join(format!("file_{}.bin", i)), content).unwrap();
    }
}

fn benchmark_small_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_files");

    for file_count in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*file_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(file_count),
            file_count,
            |b, &count| {
                b.iter_with_setup(
                    || {
                        let temp = TempDir::new().unwrap();
                        let source = temp.path().join("source");
                        fs::create_dir_all(&source).unwrap();
                        create_test_files(&source, count, 1024); // 1KB files

                        let mut config = Config::default();
                        config.backup.destination = temp.path().join("backup");
                        config.add_target(Target::new(source, Priority::High, "bench".into()));

                        (temp, config)
                    },
                    |(temp, config)| {
                        let runner = BackupRunner::new(config, false);
                        black_box(runner.run(None).unwrap());
                        drop(temp); // Cleanup
                    },
                );
            },
        );
    }
    group.finish();
}

fn benchmark_large_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_files");
    group.sample_size(10); // Reduce sample size for large files

    for size_mb in [10, 50, 100].iter() {
        group.throughput(Throughput::Bytes((size_mb * 1024 * 1024) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}MB", size_mb)),
            size_mb,
            |b, &size| {
                b.iter_with_setup(
                    || {
                        let temp = TempDir::new().unwrap();
                        let source = temp.path().join("source");
                        fs::create_dir_all(&source).unwrap();
                        create_test_files(&source, 1, size * 1024 * 1024);

                        let mut config = Config::default();
                        config.backup.destination = temp.path().join("backup");
                        config.add_target(Target::new(source, Priority::High, "bench".into()));

                        (temp, config)
                    },
                    |(temp, config)| {
                        let runner = BackupRunner::new(config, false);
                        black_box(runner.run(None).unwrap());
                        drop(temp);
                    },
                );
            },
        );
    }
    group.finish();
}

fn benchmark_directory_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory_depth");

    for depth in [5, 10, 15].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(depth),
            depth,
            |b, &d| {
                b.iter_with_setup(
                    || {
                        let temp = TempDir::new().unwrap();
                        let mut current = temp.path().join("source");
                        fs::create_dir_all(&current).unwrap();

                        // Create deep directory structure
                        for i in 0..d {
                            current = current.join(format!("level_{}", i));
                            fs::create_dir_all(&current).unwrap();
                            fs::write(current.join("file.txt"), "data").unwrap();
                        }

                        let mut config = Config::default();
                        config.backup.destination = temp.path().join("backup");
                        config.add_target(Target::new(
                            temp.path().join("source"),
                            Priority::High,
                            "bench".into()
                        ));

                        (temp, config)
                    },
                    |(temp, config)| {
                        let runner = BackupRunner::new(config, false);
                        black_box(runner.run(None).unwrap());
                        drop(temp);
                    },
                );
            },
        );
    }
    group.finish();
}

fn benchmark_parallel_vs_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallelism");

    let file_count = 1000;

    // Parallel (current implementation)
    group.bench_function("parallel", |b| {
        b.iter_with_setup(
            || {
                let temp = TempDir::new().unwrap();
                let source = temp.path().join("source");
                fs::create_dir_all(&source).unwrap();
                create_test_files(&source, file_count, 10240); // 10KB files

                let mut config = Config::default();
                config.backup.destination = temp.path().join("backup");
                config.add_target(Target::new(source, Priority::High, "bench".into()));

                (temp, config)
            },
            |(temp, config)| {
                let runner = BackupRunner::new(config, false);
                black_box(runner.run(None).unwrap());
                drop(temp);
            },
        );
    });

    // TODO: Add sequential implementation for comparison

    group.finish();
}

criterion_group!(
    benches,
    benchmark_small_files,
    benchmark_large_files,
    benchmark_directory_depth,
    benchmark_parallel_vs_sequential
);
criterion_main!(benches);
```

#### 4.2 Memory Profiling Tests
```rust
// tests/performance/memory_tests.rs
#[test]
#[ignore] // Run with --ignored for memory tests
fn test_memory_usage_large_backup() {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TrackingAllocator;

    static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

    unsafe impl GlobalAlloc for TrackingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let ret = System.alloc(layout);
            if !ret.is_null() {
                ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
            }
            ret
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            System.dealloc(ptr, layout);
            ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
        }
    }

    #[global_allocator]
    static GLOBAL: TrackingAllocator = TrackingAllocator;

    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    fs::create_dir_all(&source).unwrap();

    // Create 10,000 files
    for i in 0..10_000 {
        fs::write(source.join(format!("file_{}.txt", i)), "content").unwrap();
    }

    let initial_memory = ALLOCATED.load(Ordering::SeqCst);

    let mut config = Config::default();
    config.backup.destination = temp.path().join("backup");
    config.add_target(Target::new(source, Priority::High, "test".into()));

    let runner = BackupRunner::new(config, false);
    runner.run(None).unwrap();

    let peak_memory = ALLOCATED.load(Ordering::SeqCst);
    let memory_used_mb = (peak_memory - initial_memory) as f64 / 1_048_576.0;

    println!("Memory used: {:.2} MB", memory_used_mb);
    assert!(memory_used_mb < 50.0, "Memory usage should be under 50MB");
}
```

#### 4.3 Performance Regression Tests
```rust
// tests/performance/regression_tests.rs
use std::time::Instant;

#[test]
fn test_performance_baseline_1000_files() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    fs::create_dir_all(&source).unwrap();

    for i in 0..1000 {
        fs::write(source.join(format!("file_{}.txt", i)), "test content").unwrap();
    }

    let mut config = Config::default();
    config.backup.destination = temp.path().join("backup");
    config.add_target(Target::new(source, Priority::High, "test".into()));

    let runner = BackupRunner::new(config, false);

    let start = Instant::now();
    runner.run(None).unwrap();
    let duration = start.elapsed();

    println!("1000 files backup time: {:?}", duration);

    // Performance baseline: Should complete in under 2 seconds
    assert!(duration.as_secs() < 2, "Performance regression detected: {:?}", duration);
}
```

---

## 🔍 Mutation Testing Strategy

### Phase 5: Test Quality Validation (Week 6)

#### 5.1 Cargo Mutants Configuration
```toml
# Cargo.toml
[package.metadata.mutants]
exclude_dirs = ["tests", "benches", "examples"]
timeout_multiplier = 3.0
minimum_test_time = 1.0

[[package.metadata.mutants.mutation]]
name = "security"
paths = ["src/security/**/*.rs"]
priority = "high"

[[package.metadata.mutants.mutation]]
name = "core"
paths = ["src/core/**/*.rs"]
priority = "medium"
```

#### 5.2 Expected Mutation Coverage
- **Security Module**: >95% mutation kill rate
- **Core Logic**: >85% mutation kill rate
- **CLI**: >70% mutation kill rate

---

## 🐛 Fuzzing Strategy

### Phase 5: Continuous Fuzzing (Week 6+)

#### 6.1 Fuzz Targets
```rust
// fuzz/fuzz_targets/config_parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use backup_suite::core::Config;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _result: Result<Config, _> = toml::from_str(s);
        // Just ensure it doesn't panic
    }
});

// fuzz/fuzz_targets/path_sanitizer.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use backup_suite::security::sanitize_path_component;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let sanitized = sanitize_path_component(s);

        // Ensure sanitized output never contains dangerous characters
        assert!(!sanitized.contains('/'));
        assert!(!sanitized.contains('\\'));
        assert!(!sanitized.contains('\0'));
    }
});

// fuzz/fuzz_targets/safe_join.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use backup_suite::security::safe_join;
use std::path::{Path, PathBuf};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let base = PathBuf::from("/tmp/base");
        let child = Path::new(s);

        if let Ok(result) = safe_join(&base, child) {
            // If successful, result must be under base
            assert!(result.starts_with(&base) || result == base);
        }
    }
});
```

---

## 🚀 CI/CD Testing Pipeline

### Phase 5: Automated Quality Gates (Week 6)

#### 7.1 GitHub Actions Workflow
```yaml
# .github/workflows/comprehensive-tests.yml
name: Comprehensive Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  unit-tests:
    name: Unit Tests
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta, nightly]

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust ${{ matrix.rust }}
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        override: true
        components: rustfmt, clippy

    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ matrix.rust }}-${{ hashFiles('**/Cargo.lock') }}

    - name: Check formatting
      run: cargo fmt -- --check

    - name: Clippy
      run: cargo clippy --all-targets --all-features -- -D warnings

    - name: Unit tests
      run: cargo test --lib --bins --verbose

    - name: Doc tests
      run: cargo test --doc

  integration-tests:
    name: Integration Tests
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true

    - name: Run integration tests
      run: cargo test --test '*' --verbose

    - name: Run CLI tests
      run: cargo test --test cli_integration_tests

  security-tests:
    name: Security Tests
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true

    - name: Security audit
      uses: actions-rs/audit-check@v1
      with:
        token: ${{ secrets.GITHUB_TOKEN }}

    - name: Run security tests
      run: cargo test --test security_tests --verbose

    - name: Property-based tests
      run: cargo test --test property_tests --verbose

  performance-tests:
    name: Performance Benchmarks
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true

    - name: Run benchmarks
      run: cargo bench --no-fail-fast

    - name: Store benchmark results
      uses: benchmark-action/github-action-benchmark@v1
      with:
        tool: 'cargo'
        output-file-path: target/criterion/reports/index.html
        github-token: ${{ secrets.GITHUB_TOKEN }}
        auto-push: true

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true

    - name: Install tarpaulin
      run: cargo install cargo-tarpaulin

    - name: Generate coverage
      run: |
        cargo tarpaulin \
          --verbose \
          --all-features \
          --workspace \
          --timeout 300 \
          --out Xml \
          --exclude-files 'fuzz/*' 'benches/*'

    - name: Upload to codecov.io
      uses: codecov/codecov-action@v3
      with:
        token: ${{ secrets.CODECOV_TOKEN }}
        fail_ci_if_error: true

    - name: Check coverage threshold
      run: |
        COVERAGE=$(cargo tarpaulin --print-summary | grep -oP '\d+\.\d+(?=%)')
        if (( $(echo "$COVERAGE < 80" | bc -l) )); then
          echo "Coverage $COVERAGE% is below 80% threshold"
          exit 1
        fi

  mutation-tests:
    name: Mutation Testing
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true

    - name: Install cargo-mutants
      run: cargo install cargo-mutants

    - name: Run mutation tests
      run: cargo mutants --no-shuffle -- --all-features
      continue-on-error: true

    - name: Upload mutation report
      uses: actions/upload-artifact@v3
      with:
        name: mutation-report
        path: mutants.out/

  fuzz-tests:
    name: Fuzzing (Continuous)
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust nightly
      uses: actions-rs/toolchain@v1
      with:
        toolchain: nightly
        override: true

    - name: Install cargo-fuzz
      run: cargo install cargo-fuzz

    - name: Run fuzz tests (5 min each)
      run: |
        cargo fuzz run config_parser -- -max_total_time=300
        cargo fuzz run path_sanitizer -- -max_total_time=300
        cargo fuzz run safe_join -- -max_total_time=300
```

---

## 📈 Test Coverage Goals

### Coverage Targets by Phase

| Phase | Unit | Integration | Security | E2E | Overall |
|-------|------|-------------|----------|-----|---------|
| Current | 30% | 0% | 0% | 0% | 30% |
| Phase 2 | 60% | 40% | 90% | 20% | 55% |
| Phase 3 | 70% | 50% | 95% | 40% | 65% |
| Phase 5 | 80% | 60% | 100% | 60% | 80% |

### Critical Path Coverage (Must be 100%)
- Path traversal prevention
- Permission validation
- Input sanitization
- Config validation
- Error handling in backup operations

---

## 🎯 Test Execution Strategy

### Local Development
```bash
# Quick feedback loop
cargo test --lib

# Full test suite
cargo test --all

# With coverage
cargo tarpaulin --out Html

# Benchmarks
cargo bench

# Property tests (extended)
cargo test --test property_tests -- --ignored

# Security focus
cargo test --test security_tests --test property_tests
```

### Pre-commit Hooks
```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running pre-commit tests..."

# Fast checks
cargo fmt -- --check || exit 1
cargo clippy -- -D warnings || exit 1
cargo test --lib || exit 1

echo "Pre-commit checks passed!"
```

### CI/CD Strategy
- **On Pull Request**: Full test suite + mutation testing
- **On Main Push**: Full suite + fuzzing + performance benchmarks
- **Nightly**: Extended property tests + long-running fuzz tests
- **Weekly**: Full mutation testing suite

---

## 📝 Test Documentation Standards

### Test Naming Convention
```rust
// Pattern: test_<component>_<scenario>_<expected_result>

#[test]
fn test_backup_single_file_success() { }

#[test]
fn test_config_validation_invalid_keep_days_error() { }

#[test]
fn test_path_traversal_parent_dir_rejected() { }
```

### Test Organization
```
tests/
├── unit/
│   ├── config_tests.rs
│   ├── target_tests.rs
│   └── backup_tests.rs
├── integration/
│   ├── workflow_tests.rs
│   ├── cli_tests.rs
│   └── restore_tests.rs
├── security/
│   ├── path_traversal_tests.rs
│   ├── permission_tests.rs
│   ├── input_validation_tests.rs
│   └── property_tests.rs
├── performance/
│   ├── memory_tests.rs
│   └── regression_tests.rs
└── common/
    ├── mod.rs
    └── generators.rs
```

---

## 🎓 Testing Best Practices

### 1. Test Isolation
- Each test uses `TempDir` for filesystem isolation
- No shared state between tests
- Parallel execution safe

### 2. Deterministic Tests
- No reliance on system time (use fixed timestamps)
- Seed random generators
- Control all external dependencies

### 3. Fast Feedback
- Unit tests complete in <1s
- Integration tests in <5s
- Full suite in <2min

### 4. Readable Tests
- Clear arrange-act-assert structure
- Descriptive variable names
- Inline comments for complex scenarios

### 5. Test Maintenance
- Regular review and refactoring
- Remove obsolete tests
- Update tests with feature changes

---

## 📊 Success Metrics

### Test Quality Metrics
- **Mutation Kill Rate**: >85% overall, >95% for security code
- **Flakiness**: <1% test flakiness rate
- **Execution Time**: <2 minutes for full suite
- **Coverage**: 80% line coverage, 90% branch coverage for critical paths

### Development Metrics
- **TDD Adoption**: 50% of new code written test-first
- **Bug Detection**: 80% of bugs caught by automated tests
- **Regression Prevention**: Zero regressions after Phase 5

---

## 🚀 Implementation Timeline

### Week 1: Security Tests (Phase 1)
- ✅ Path traversal tests
- ✅ Permission validation tests
- ✅ Input sanitization tests
- ✅ Property-based security tests

### Week 2-3: Unit & Integration Tests (Phase 2)
- ✅ Core module unit tests (60% coverage)
- ✅ Integration test framework
- ✅ Workflow integration tests
- ✅ CLI integration tests

### Week 4-5: Performance & E2E (Phase 3)
- ✅ Criterion benchmarks
- ✅ Memory profiling
- ✅ E2E workflow tests
- ✅ Performance regression tests

### Week 6: Quality Assurance (Phase 5)
- ✅ Mutation testing setup
- ✅ Fuzzing infrastructure
- ✅ CI/CD pipeline completion
- ✅ 80% coverage achievement

---

## 🎯 Conclusion

This comprehensive test automation strategy provides:

1. **Security-First Approach**: 100% coverage of security-critical code
2. **Modern Frameworks**: Property-based testing, fuzzing, mutation testing
3. **Performance Validation**: Benchmarking and regression detection
4. **CI/CD Integration**: Automated quality gates
5. **TDD Enablement**: Fast feedback loops for test-driven development

**Expected Outcomes**:
- 80% test coverage (from 30%)
- Zero security vulnerabilities in tested code
- Validated 50x performance improvement
- Production-ready reliability

This strategy directly complements and enhances the IMPROVEMENT_PLAN.md, providing concrete test implementations for all security improvements and feature developments.
