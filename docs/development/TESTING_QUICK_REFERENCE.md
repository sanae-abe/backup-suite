# Testing Quick Reference - Backup Suite

## 🚀 Quick Start Commands

### Essential Test Commands
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_backup_single_file

# Run tests in specific file
cargo test --test security_tests

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run benchmarks
cargo bench

# Generate coverage report
cargo tarpaulin --out Html
```

---

## 📋 Test Types at a Glance

| Test Type | Command | When to Use | Example |
|-----------|---------|-------------|---------|
| **Unit** | `cargo test --lib` | Testing individual functions | `test_config_add_target()` |
| **Integration** | `cargo test --test workflow_tests` | Testing component interactions | `test_full_backup_restore_workflow()` |
| **Security** | `cargo test --test security_tests` | Path traversal, permissions | `test_path_traversal_attack()` |
| **Property** | `cargo test --test property_tests` | Edge case discovery | `prop_safe_join_never_escapes()` |
| **Benchmark** | `cargo bench` | Performance measurement | `benchmark_1000_files()` |
| **E2E** | `cargo test --test cli_integration_tests` | Full user workflows | `test_cli_add_list_remove()` |

---

## 🎯 Test Coverage Targets

```
Current: 30%  ████░░░░░░░░░░░░░░░░  Target: 80%

Phase 2: 60%  ████████████░░░░░░░░  (Week 3)
Phase 3: 70%  ██████████████░░░░░░  (Week 5)
Phase 5: 80%  ████████████████░░░░  (Week 6)
```

### Critical Path Coverage (100% Required)
- ✅ Path traversal prevention
- ✅ Permission validation
- ✅ Input sanitization
- ✅ Config validation
- ✅ Backup error handling

---

## 🔒 Security Testing Checklist

### Phase 1 (Week 1) - Critical Security
- [ ] Path traversal tests (5 scenarios)
- [ ] Permission validation tests (3 scenarios)
- [ ] Input sanitization tests (4 scenarios)
- [ ] Property-based path tests
- [ ] Symlink escape prevention

### Must-Have Security Tests
```rust
// 1. Path Traversal
test_path_traversal_parent_dir_attack()
test_path_traversal_absolute_path_attack()
test_path_traversal_symlink_escape()

// 2. Permissions
test_read_permission_denied()
test_write_permission_readonly_directory()

// 3. Input Validation
test_config_validation_malformed_toml()
test_target_path_injection()
```

---

## ⚡ Performance Testing

### Benchmark Scenarios
```bash
# Run specific benchmark
cargo bench -- small_files

# Compare with baseline
cargo bench -- --save-baseline main
cargo bench -- --baseline main

# Profile memory
cargo test test_memory_usage_large_backup -- --ignored --nocapture
```

### Performance Baselines
| Scenario | Target | Current |
|----------|--------|---------|
| 1,000 files (1KB each) | <2s | 0.84s ✅ |
| 100 files (10MB each) | <5s | TBD |
| Deep directories (15 levels) | <3s | TBD |

---

## 🧪 Test Development Workflow

### TDD Cycle (Red-Green-Refactor)
```bash
# 1. Write failing test
cargo test test_new_feature  # Should fail

# 2. Implement minimum code
# ... edit src/...

# 3. Run test until pass
cargo test test_new_feature  # Should pass

# 4. Refactor
# ... improve implementation

# 5. Verify
cargo test  # All tests pass
```

### Pre-Commit Checklist
```bash
# Quick validation (< 30 seconds)
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test --lib

# Full validation (< 2 minutes)
cargo test --all
cargo bench -- --test  # Benchmark compilation only
```

---

## 🐛 Common Testing Patterns

### 1. Filesystem Testing with TempDir
```rust
use tempfile::TempDir;

#[test]
fn test_example() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.txt");
    std::fs::write(&file, "data").unwrap();

    // Test logic...

    // Cleanup automatic on drop
}
```

### 2. Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_sanitize_safe(input in ".*") {
        let result = sanitize(input);
        assert!(!result.contains('/'));
    }
}
```

### 3. CLI Testing
```rust
use assert_cmd::Command;

#[test]
fn test_cli_command() {
    Command::cargo_bin("backup-suite").unwrap()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("対象"));
}
```

### 4. Error Testing
```rust
#[test]
#[should_panic(expected = "path traversal")]
fn test_rejects_malicious_path() {
    safe_join("/tmp", "../../etc/passwd").unwrap();
}

// Or with Result
#[test]
fn test_error_result() {
    let result = safe_join("/tmp", "../etc");
    assert!(result.is_err());
}
```

---

## 📊 CI/CD Testing Gates

### Pull Request Requirements
- ✅ All tests pass
- ✅ No clippy warnings
- ✅ Code formatted (rustfmt)
- ✅ Coverage ≥ 80%
- ✅ Mutation testing (on security code)

### Main Branch Requirements
- ✅ All PR checks pass
- ✅ Performance benchmarks (no regression)
- ✅ Integration tests on all platforms
- ✅ Security audit clean

---

## 🔍 Debugging Failed Tests

### Show Test Output
```bash
# Run with output
cargo test -- --nocapture

# Show backtrace
RUST_BACKTRACE=1 cargo test

# Run single test verbosely
cargo test test_name -- --nocapture --test-threads=1
```

### Common Issues

#### Issue: Test hangs
```bash
# Solution: Run with timeout
cargo test -- --test-threads=1 --timeout 30
```

#### Issue: Flaky test
```bash
# Solution: Run multiple times
for i in {1..10}; do cargo test test_name || break; done
```

#### Issue: Permission denied
```bash
# Solution: Check temp directory cleanup
# Use TempDir instead of manual cleanup
```

---

## 📈 Coverage Analysis

### Generate HTML Coverage Report
```bash
cargo tarpaulin --out Html --output-dir coverage

# Open report
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

### Coverage by Module
```bash
cargo tarpaulin --verbose --all-features --workspace
```

### Exclude Files from Coverage
```toml
# Cargo.toml
[package.metadata.tarpaulin]
exclude-files = ["tests/*", "benches/*", "examples/*"]
```

---

## 🎯 Testing Priorities

### High Priority (Do First)
1. Security tests (path traversal, permissions)
2. Core backup workflow tests
3. Error handling tests
4. Integration tests for critical paths

### Medium Priority (Phase 2)
5. Config management tests
6. CLI integration tests
7. Performance benchmarks
8. Property-based tests

### Lower Priority (Phase 3+)
9. Edge case tests
10. UI/UX validation tests
11. Cross-platform compatibility tests
12. Mutation testing

---

## 🛠️ Test Maintenance

### Weekly
- [ ] Review failing tests in CI
- [ ] Update test data generators
- [ ] Check for flaky tests
- [ ] Review coverage reports

### Monthly
- [ ] Refactor slow tests
- [ ] Remove obsolete tests
- [ ] Update property test strategies
- [ ] Review mutation testing results

### Quarterly
- [ ] Audit test architecture
- [ ] Update benchmark baselines
- [ ] Review test documentation
- [ ] Analyze test effectiveness metrics

---

## 📚 Key Resources

### Documentation
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion Benchmarking](https://bheisler.github.io/criterion.rs/book/)
- [Proptest Book](https://altsysrq.github.io/proptest-book/intro.html)
- [TEST_AUTOMATION_STRATEGY.md](TEST_AUTOMATION_STRATEGY.md) - Comprehensive guide

### Tools
- `cargo test` - Built-in test runner
- `cargo tarpaulin` - Coverage tool
- `cargo bench` - Benchmark runner
- `cargo mutants` - Mutation testing
- `cargo fuzz` - Fuzzing framework

---

## 💡 Pro Tips

### 1. Parallel Test Execution
```rust
// Control test parallelism
cargo test -- --test-threads=4
```

### 2. Test Filtering
```bash
# Run tests matching pattern
cargo test security

# Run ignored tests
cargo test -- --ignored

# Run specific test file
cargo test --test integration_tests
```

### 3. Watch Mode
```bash
# Auto-run tests on file change
cargo install cargo-watch
cargo watch -x test
```

### 4. Test Data Setup
```rust
// Use test fixtures for common setup
mod common;
use common::TestFixture;

#[test]
fn test_with_fixture() {
    let fixture = TestFixture::new();
    // Automatic setup and cleanup
}
```

### 5. Conditional Compilation
```rust
// Platform-specific tests
#[test]
#[cfg(unix)]
fn test_unix_permissions() {
    // Unix-only test
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_feature() {
    // macOS-only test
}
```

---

## 🎓 Learning Path

### Beginner
1. Write basic unit tests for new functions
2. Use `assert_eq!` and `assert!` macros
3. Understand test organization

### Intermediate
4. Write integration tests with TempDir
5. Use property-based testing for edge cases
6. Create test fixtures and helpers

### Advanced
7. Implement benchmarks for performance
8. Use mutation testing to validate test quality
9. Set up fuzzing for security-critical code

---

## 📞 Getting Help

### Test Failures
1. Read error message carefully
2. Run with `--nocapture` for output
3. Use `RUST_BACKTRACE=1` for stack trace
4. Check test isolation (TempDir usage)

### Coverage Issues
1. Identify untested code with tarpaulin HTML report
2. Prioritize security-critical paths
3. Use property tests for complex logic
4. Add integration tests for workflows

### Performance Issues
1. Use `cargo bench` to measure
2. Profile with `cargo flamegraph`
3. Check for algorithmic improvements
4. Verify parallel execution effectiveness

---

**Last Updated**: 2025-11-04
**Next Review**: Phase 2 completion (Week 3)
