# Testing Implementation Guide - Backup Suite

## 🎯 Purpose

This guide provides step-by-step instructions for implementing the comprehensive test automation strategy. Use this as your practical implementation checklist.

---

## 📁 Directory Structure Setup

### Step 1: Create Test Directories
```bash
cd /Users/sanae.abe/projects/backup-suite

# Create test directory structure
mkdir -p tests/{unit,integration,security,performance,common}
mkdir -p benches
mkdir -p fuzz/fuzz_targets

# Create placeholder files
touch tests/common/mod.rs
touch tests/common/generators.rs
```

### Expected Structure
```
backup-suite/
├── src/
│   ├── core/
│   │   ├── backup.rs      (inline #[cfg(test)] tests)
│   │   ├── config.rs      (inline #[cfg(test)] tests)
│   │   └── target.rs      (inline #[cfg(test)] tests)
│   └── security/          (to be created in Phase 1)
│       ├── mod.rs
│       ├── path_utils.rs
│       └── permissions.rs
├── tests/
│   ├── unit/              (if complex unit tests needed)
│   ├── integration/
│   │   ├── workflow_tests.rs
│   │   └── cli_tests.rs
│   ├── security/
│   │   ├── path_traversal_tests.rs
│   │   ├── permission_tests.rs
│   │   ├── input_validation_tests.rs
│   │   └── property_tests.rs
│   ├── performance/
│   │   ├── memory_tests.rs
│   │   └── regression_tests.rs
│   └── common/
│       ├── mod.rs
│       └── generators.rs
├── benches/
│   └── backup_benchmarks.rs
└── fuzz/
    ├── Cargo.toml
    └── fuzz_targets/
        ├── config_parser.rs
        ├── path_sanitizer.rs
        └── safe_join.rs
```

---

## 🔧 Phase 1: Security Testing Infrastructure (Week 1)

### Step 1.1: Update Cargo.toml Dependencies
```bash
# Add to Cargo.toml
```

```toml
[dev-dependencies]
tempfile = "3.8"
assert_cmd = "2.0"
predicates = "3.0"
proptest = "1.4"

[dependencies]
thiserror = "1.0"
regex = "1.10"  # For exclude_patterns

[profile.test]
opt-level = 1  # Faster test compilation
```

### Step 1.2: Create Security Module Structure
```bash
# Create security module
mkdir -p src/security
touch src/security/mod.rs
touch src/security/path_utils.rs
touch src/security/permissions.rs
```

### Step 1.3: Implement Path Security (src/security/path_utils.rs)
```rust
use std::path::{Path, PathBuf, Component};
use anyhow::{Result, Context};

/// Safe path joining with traversal protection
pub fn safe_join(base: &Path, child: &Path) -> Result<PathBuf> {
    // Normalize child path by removing dangerous components
    let normalized: PathBuf = child
        .components()
        .filter(|c| !matches!(c, Component::ParentDir | Component::RootDir))
        .collect();

    let result = base.join(&normalized);

    // Verify result is within base directory
    if let (Ok(canonical_base), Ok(canonical_result)) = (
        base.canonicalize(),
        result.canonicalize().or_else(|_| Ok(result.clone()))
    ) {
        if !canonical_result.starts_with(&canonical_base) {
            anyhow::bail!("Path traversal detected: {:?}", child);
        }
    }

    Ok(result)
}

/// Sanitize path component to remove dangerous characters
pub fn sanitize_path_component(name: &str) -> String {
    name.chars()
        .filter(|&c| {
            c.is_alphanumeric()
            || c == '-'
            || c == '_'
            || c == '.'
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_join_normal_path() {
        let base = PathBuf::from("/tmp/backup");
        let child = Path::new("subdir/file.txt");

        let result = safe_join(&base, child).unwrap();
        assert_eq!(result, PathBuf::from("/tmp/backup/subdir/file.txt"));
    }

    #[test]
    fn test_safe_join_rejects_parent_dir() {
        let base = PathBuf::from("/tmp/backup");
        let child = Path::new("../../../etc/passwd");

        let result = safe_join(&base, child);
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_removes_slashes() {
        assert_eq!(sanitize_path_component("normal.txt"), "normal.txt");
        assert_eq!(sanitize_path_component("path/with/slash"), "pathwithslash");
        assert_eq!(sanitize_path_component("../escape"), "..escape");
    }
}
```

### Step 1.4: Implement Permission Checks (src/security/permissions.rs)
```rust
use std::path::Path;
use anyhow::{Result, Context};

/// Check read permission for file or directory
pub fn check_read_permission(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("Cannot access path: {:?}", path))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();

        // Check if owner has read permission (0o400)
        if mode & 0o400 == 0 {
            anyhow::bail!("No read permission: {:?}", path);
        }
    }

    Ok(())
}

/// Check write permission for directory
pub fn check_write_permission(path: &Path) -> Result<()> {
    let parent = path.parent()
        .ok_or_else(|| anyhow::anyhow!("No parent directory: {:?}", path))?;

    // Test write permission by attempting temp file creation
    let test_file = parent.join(".backup_suite_write_test");

    std::fs::write(&test_file, b"test")
        .with_context(|| format!("No write permission: {:?}", parent))?;

    std::fs::remove_file(&test_file)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_check_read_permission_success() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("readable.txt");
        std::fs::write(&file, "data").unwrap();

        assert!(check_read_permission(&file).is_ok());
    }

    #[test]
    fn test_check_write_permission_success() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("writable.txt");

        assert!(check_write_permission(&file).is_ok());
    }
}
```

### Step 1.5: Update src/security/mod.rs
```rust
mod path_utils;
mod permissions;

pub use path_utils::{safe_join, sanitize_path_component};
pub use permissions::{check_read_permission, check_write_permission};
```

### Step 1.6: Update src/main.rs or src/lib.rs
```rust
mod core;
mod security;  // Add this line

// Rest of the code...
```

### Step 1.7: Create Security Tests (tests/security/path_traversal_tests.rs)
```rust
use backup_suite::security::safe_join;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[test]
fn test_path_traversal_parent_dir_attack() {
    let base = PathBuf::from("/tmp/backup");
    let attacks = vec![
        "../../../etc/passwd",
        "../../etc/shadow",
        "../../../../../root/.ssh/id_rsa",
        "subdir/../../etc/passwd",
    ];

    for attack in attacks {
        let result = safe_join(&base, Path::new(attack));
        assert!(
            result.is_err(),
            "Should reject parent dir attack: {}",
            attack
        );
    }
}

#[test]
fn test_path_traversal_absolute_path_attack() {
    let base = PathBuf::from("/tmp/backup");
    let attacks = vec![
        "/etc/passwd",
        "/var/log/system.log",
        "/root/.bashrc",
    ];

    for attack in attacks {
        let result = safe_join(&base, Path::new(attack));
        assert!(
            result.is_err() || !result.unwrap().starts_with("/etc"),
            "Should reject absolute path attack: {}",
            attack
        );
    }
}

#[test]
fn test_path_traversal_symlink_escape() {
    let temp = TempDir::new().unwrap();
    let base = temp.path().join("backup");
    std::fs::create_dir_all(&base).unwrap();

    // Create symlink pointing outside
    let outside = temp.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("secret.txt"), "secret").unwrap();

    let symlink = base.join("escape_link");

    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, &symlink).unwrap();

    // Attempt to access through symlink
    let result = safe_join(&base, Path::new("escape_link/secret.txt"));

    // Should either error or not escape base
    if let Ok(resolved) = result {
        // If canonicalization succeeds, check it's not accessing outside
        if let Ok(canonical) = resolved.canonicalize() {
            assert!(
                !canonical.starts_with(&outside),
                "Should not escape via symlink"
            );
        }
    }
}

#[test]
fn test_path_traversal_null_byte_injection() {
    let base = PathBuf::from("/tmp/backup");
    let attack = "normal.txt\0../../../etc/passwd";

    // Rust's path handling should prevent null byte issues
    let result = safe_join(&base, Path::new(attack));

    if let Ok(path) = result {
        assert!(
            !path.to_string_lossy().contains("/etc"),
            "Should not contain /etc path"
        );
    }
}

#[test]
fn test_safe_join_legitimate_paths() {
    let base = PathBuf::from("/tmp/backup");
    let valid_paths = vec![
        "file.txt",
        "subdir/file.txt",
        "a/b/c/deep/file.txt",
        "file-name_with.special.txt",
    ];

    for valid in valid_paths {
        let result = safe_join(&base, Path::new(valid));
        assert!(
            result.is_ok(),
            "Should accept legitimate path: {}",
            valid
        );

        let path = result.unwrap();
        assert!(
            path.starts_with(&base),
            "Result should be under base: {:?}",
            path
        );
    }
}
```

### Step 1.8: Create Property-Based Tests (tests/security/property_tests.rs)
```rust
use proptest::prelude::*;
use backup_suite::security::{safe_join, sanitize_path_component};
use std::path::{Path, PathBuf};

proptest! {
    #[test]
    fn prop_safe_join_never_escapes_base(
        base_components in prop::collection::vec("[a-zA-Z0-9_-]+", 1..5),
        child_components in prop::collection::vec("[a-zA-Z0-9._-]+", 1..8),
    ) {
        let base = PathBuf::from("/tmp").join(base_components.join("/"));
        let child = PathBuf::from(child_components.join("/"));

        if let Ok(result) = safe_join(&base, &child) {
            // Property: Result must be within or equal to base
            prop_assert!(
                result.starts_with(&base) || result == base,
                "safe_join escaped base: {:?} not in {:?}",
                result,
                base
            );
        }
    }

    #[test]
    fn prop_sanitize_never_produces_slashes(
        input in "\\PC*"  // Any string
    ) {
        let sanitized = sanitize_path_component(&input);

        // Properties: No path separators
        prop_assert!(!sanitized.contains('/'));
        prop_assert!(!sanitized.contains('\\'));
        prop_assert!(!sanitized.contains('\0'));
    }

    #[test]
    fn prop_sanitize_idempotent(
        input in "[a-zA-Z0-9._-]*"
    ) {
        let first = sanitize_path_component(&input);
        let second = sanitize_path_component(&first);

        // Property: Sanitizing twice should be same as once
        prop_assert_eq!(&first, &second);
    }
}
```

### Step 1.9: Run Security Tests
```bash
# Run all security tests
cargo test --test path_traversal_tests
cargo test --test property_tests

# Run with coverage
cargo tarpaulin --test path_traversal_tests --test property_tests
```

---

## 🧪 Phase 2: Integration & Unit Tests (Week 2-3)

### Step 2.1: Create Test Fixtures (tests/common/generators.rs)
```rust
use backup_suite::core::{Config, Target, Priority};
use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;

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

        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&backup_dir).unwrap();

        Self {
            temp_dir,
            source_dir,
            backup_dir,
        }
    }

    pub fn create_file(&self, relative_path: &str, content: &str) -> PathBuf {
        let file_path = self.source_dir.join(relative_path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(&file_path, content).unwrap();
        file_path
    }

    pub fn create_files(&self, count: usize) -> Vec<PathBuf> {
        (0..count)
            .map(|i| {
                self.create_file(
                    &format!("file_{}.txt", i),
                    &format!("content {}", i),
                )
            })
            .collect()
    }

    pub fn create_directory_tree(&self, depth: usize, files_per_dir: usize) {
        fn create_level(
            base: &PathBuf,
            current: usize,
            max: usize,
            files: usize,
        ) {
            if current > max {
                return;
            }

            // Create files at this level
            for i in 0..files {
                fs::write(
                    base.join(format!("file_{}.txt", i)),
                    format!("depth {} file {}", current, i),
                )
                .unwrap();
            }

            // Create subdirectories
            for i in 0..2 {
                let subdir = base.join(format!("dir_{}", i));
                fs::create_dir_all(&subdir).unwrap();
                create_level(&subdir, current + 1, max, files);
            }
        }

        create_level(&self.source_dir, 0, depth, files_per_dir);
    }

    pub fn create_config(&self) -> Config {
        let mut config = Config::default();
        config.backup.destination = self.backup_dir.clone();
        config
    }

    pub fn add_target(&self, config: &mut Config, priority: Priority) {
        let target = Target::new(
            self.source_dir.clone(),
            priority,
            "test".to_string(),
        );
        config.add_target(target);
    }
}
```

### Step 2.2: Update tests/common/mod.rs
```rust
mod generators;

pub use generators::TestFixture;
```

### Step 2.3: Create Integration Tests (tests/integration/workflow_tests.rs)
```rust
mod common;
use common::TestFixture;
use backup_suite::core::{BackupRunner, Priority};

#[test]
fn test_full_backup_workflow() {
    let fixture = TestFixture::new();
    fixture.create_files(10);

    let mut config = fixture.create_config();
    fixture.add_target(&mut config, Priority::High);

    let runner = BackupRunner::new(config, false);
    let result = runner.run(None).unwrap();

    assert_eq!(result.total_files, 10);
    assert_eq!(result.success_files, 10);
    assert_eq!(result.failed_files, 0);
    assert!(result.total_bytes > 0);
}

#[test]
fn test_directory_tree_backup() {
    let fixture = TestFixture::new();
    fixture.create_directory_tree(3, 2);

    let mut config = fixture.create_config();
    fixture.add_target(&mut config, Priority::High);

    let runner = BackupRunner::new(config, false);
    let result = runner.run(None).unwrap();

    // 2 files per dir * (1 + 2 + 4) = 14 files
    assert!(result.total_files >= 14);
    assert_eq!(result.success_files, result.total_files);
}

#[test]
fn test_priority_filtering() {
    let fixture = TestFixture::new();
    fixture.create_file("high.txt", "important");
    fixture.create_file("low.txt", "optional");

    let mut config = fixture.create_config();

    let high_target = backup_suite::core::Target::new(
        fixture.source_dir.join("high.txt"),
        Priority::High,
        "critical".into(),
    );
    let low_target = backup_suite::core::Target::new(
        fixture.source_dir.join("low.txt"),
        Priority::Low,
        "optional".into(),
    );

    config.add_target(high_target);
    config.add_target(low_target);

    // Backup only high priority
    let runner = BackupRunner::new(config, false);
    let result = runner.run(Some(&Priority::High)).unwrap();

    assert_eq!(result.total_files, 1);
}
```

### Step 2.4: Create CLI Integration Tests (tests/integration/cli_tests.rs)
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_cli_add_and_list() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "data").unwrap();

    // Add target
    Command::cargo_bin("backup-suite")
        .unwrap()
        .args(&["add", test_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("追加"));

    // List targets
    Command::cargo_bin("backup-suite")
        .unwrap()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.txt"));
}

#[test]
fn test_cli_dry_run() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source.txt");
    fs::write(&source, "data").unwrap();

    Command::cargo_bin("backup-suite")
        .unwrap()
        .args(&["add", source.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("backup-suite")
        .unwrap()
        .args(&["run", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ドライラン"));
}

#[test]
fn test_cli_invalid_priority() {
    Command::cargo_bin("backup-suite")
        .unwrap()
        .args(&["add", "/tmp/test", "--priority", "invalid"])
        .assert()
        .failure();
}
```

### Step 2.5: Run Integration Tests
```bash
cargo test --test workflow_tests
cargo test --test cli_tests
cargo test --all
```

---

## ⚡ Phase 3: Performance Testing (Week 4-5)

### Step 3.1: Create Benchmark Configuration (Cargo.toml)
```toml
[[bench]]
name = "backup_benchmarks"
harness = false

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

### Step 3.2: Create Benchmarks (benches/backup_benchmarks.rs)
```rust
use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use backup_suite::core::{BackupRunner, Config, Priority, Target};
use std::fs;
use tempfile::TempDir;

fn create_test_files(dir: &std::path::Path, count: usize, size_bytes: usize) {
    for i in 0..count {
        let content = vec![0u8; size_bytes];
        fs::write(dir.join(format!("file_{}.bin", i)), content).unwrap();
    }
}

fn benchmark_small_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_files");

    for count in [100, 500, 1000] {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &count,
            |b, &count| {
                b.iter_with_setup(
                    || {
                        let temp = TempDir::new().unwrap();
                        let source = temp.path().join("source");
                        fs::create_dir_all(&source).unwrap();
                        create_test_files(&source, count, 1024);

                        let mut config = Config::default();
                        config.backup.destination = temp.path().join("backup");
                        config.add_target(Target::new(
                            source,
                            Priority::High,
                            "bench".into(),
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

criterion_group!(benches, benchmark_small_files);
criterion_main!(benches);
```

### Step 3.3: Run Benchmarks
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench -- small_files

# Generate HTML report
cargo bench
open target/criterion/report/index.html
```

---

## 🎯 Phase 4: CI/CD Integration (Week 6)

### Step 4.1: Create GitHub Actions Workflow
```yaml
# .github/workflows/tests.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests
        run: cargo test --all

      - name: Run security tests
        run: cargo test --test security_tests

      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

---

## ✅ Verification Checklist

### Phase 1 Completion
- [ ] Security module created (`src/security/`)
- [ ] Path traversal protection implemented
- [ ] Permission checks implemented
- [ ] Path traversal tests passing (5+ scenarios)
- [ ] Property-based tests passing
- [ ] Security test coverage >95%

### Phase 2 Completion
- [ ] Test fixtures created
- [ ] Integration tests implemented (5+ tests)
- [ ] CLI tests implemented (3+ tests)
- [ ] Overall test coverage >60%
- [ ] All tests passing

### Phase 3 Completion
- [ ] Criterion benchmarks implemented
- [ ] Performance baselines established
- [ ] Memory tests passing
- [ ] No performance regressions

### Phase 4 Completion
- [ ] CI/CD pipeline configured
- [ ] All tests running in CI
- [ ] Coverage reporting enabled
- [ ] Quality gates enforced

---

## 🚀 Next Steps

After completing this implementation:

1. **Run full test suite**: `cargo test --all`
2. **Check coverage**: `cargo tarpaulin --out Html`
3. **Run benchmarks**: `cargo bench`
4. **Review results**: Check HTML reports
5. **Iterate**: Add more tests based on coverage gaps

---

**For detailed test examples and strategies, see:**
- [TEST_AUTOMATION_STRATEGY.md](../TEST_AUTOMATION_STRATEGY.md)
- [TESTING_QUICK_REFERENCE.md](../TESTING_QUICK_REFERENCE.md)
