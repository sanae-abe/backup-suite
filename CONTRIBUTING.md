# Contributing to backup-suite

Thank you for your interest in contributing to backup-suite! This document provides guidelines for contributing to the project.

## Table of Contents

- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Reporting Issues](#reporting-issues)
- [Code of Conduct](#code-of-conduct)

## Development Setup

### Prerequisites

- **Rust**: 1.70.0 or later (MSRV - Minimum Supported Rust Version)
- **Git**: For version control
- **cargo-audit**: For security auditing (optional but recommended)
- **cargo-deny**: For dependency checking (optional but recommended)

### Initial Setup

```bash
# 1. Fork the repository on GitHub
# 2. Clone your fork
git clone git@github.com:YOUR_USERNAME/backup-suite.git
cd backup-suite

# 3. Add upstream remote
git remote add upstream git@github.com:sanae-abe/backup-suite.git

# 4. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 5. Build the project
cargo build

# 6. Run tests
cargo test
```

### Development Tools (Optional)

```bash
# Install cargo-audit for security auditing
cargo install cargo-audit

# Install cargo-deny for dependency checking
cargo install cargo-deny

# Install cargo-tarpaulin for code coverage (Linux only)
cargo install cargo-tarpaulin
```

## Development Workflow

### Creating a Feature Branch

```bash
# Update your main branch
git checkout main
git pull upstream main

# Create a feature branch
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

### Making Changes

1. **Write code** following our [coding standards](#coding-standards)
2. **Add tests** for new functionality
3. **Update documentation** if needed
4. **Run tests** to ensure everything works
5. **Format code** with `cargo fmt`
6. **Check with Clippy** using `cargo clippy`

### Commit Messages

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `security`: Security-related changes

**Examples:**
```bash
feat(backup): add incremental backup support
fix(crypto): prevent nonce reuse in AES-GCM
docs(readme): update installation instructions
test(security): add tests for path traversal protection
```

## Coding Standards

### Rust Style Guide

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `cargo fmt` for automatic formatting
- Use `cargo clippy` for linting

### Key Principles

1. **Security First**: Prioritize security in all code changes
   - Never hardcode sensitive information
   - Use `zeroize` for sensitive data
   - Validate all user inputs
   - Prevent path traversal attacks

2. **Type Safety**: Leverage Rust's type system
   - Use `Result<T, E>` for error handling
   - Avoid `unwrap()` in production code
   - Use newtype pattern for strong typing

3. **Performance**: Write efficient code
   - Use `rayon` for parallelization where appropriate
   - Avoid unnecessary allocations
   - Profile before optimizing

4. **Documentation**: Document public APIs
   - Use doc comments (`///`) for public functions
   - Include examples in documentation
   - Update README.md for user-facing changes

### Code Review Checklist

Before submitting a PR, ensure:
- [ ] Code compiles without warnings: `cargo build --release`
- [ ] All tests pass: `cargo test --all-features`
- [ ] Code is formatted: `cargo fmt`
- [ ] No Clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Documentation is updated
- [ ] Security considerations are addressed
- [ ] Tests are added for new functionality

## Testing

### Running Tests

```bash
# Run all tests
cargo test --all-features

# Run specific test module
cargo test --test security_tests

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel
cargo test -- --test-threads=4
```

### Test Categories

1. **Unit Tests** (135 tests)
   - Located in `src/` modules
   - Test individual functions and modules

2. **Integration Tests** (16 tests)
   - Located in `tests/` directory
   - Test end-to-end workflows

3. **Security Tests** (23 tests)
   - Test security-critical functionality
   - Located in `tests/security_tests.rs`

4. **Property-Based Tests** (37 tests)
   - Use `proptest` for property-based testing
   - Test cryptographic properties

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }
}
```

## Pull Request Process

### Before Submitting

1. **Update your branch** with the latest upstream changes:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run the full test suite**:
   ```bash
   cargo test --all-features
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   ```

3. **Update CHANGELOG.md** in the `[Unreleased]` section

### Submitting a PR

1. **Push your branch** to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create a Pull Request** on GitHub with:
   - Clear title following Conventional Commits format
   - Description of changes
   - Reference to related issues (if any)
   - Screenshots/examples (if applicable)

3. **Respond to review feedback** promptly

### PR Review Process

- Maintainers will review your PR within 7 days
- Address any requested changes
- Once approved, your PR will be merged

## Reporting Issues

### Bug Reports

When reporting bugs, please include:

1. **Description**: Clear description of the issue
2. **Steps to Reproduce**: Detailed steps to reproduce the bug
3. **Expected Behavior**: What you expected to happen
4. **Actual Behavior**: What actually happened
5. **Environment**:
   - OS and version
   - Rust version (`rustc --version`)
   - backup-suite version (`backup-suite --version`)
6. **Logs**: Relevant log output (use `--verbose` flag)

### Feature Requests

For feature requests, please include:

1. **Use Case**: Describe the problem you're trying to solve
2. **Proposed Solution**: Your suggested implementation
3. **Alternatives**: Other solutions you've considered
4. **Impact**: Who would benefit from this feature

### Security Vulnerabilities

**Do not open a public issue for security vulnerabilities.**

Please report security issues via email: sanae-abe@m3.com

See [SECURITY.md](SECURITY.md) for details.

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of:
- Age
- Body size
- Disability
- Ethnicity
- Gender identity and expression
- Level of experience
- Nationality
- Personal appearance
- Race
- Religion
- Sexual identity and orientation

### Our Standards

**Positive behavior:**
- Using welcoming and inclusive language
- Being respectful of differing viewpoints
- Accepting constructive criticism gracefully
- Focusing on what's best for the community
- Showing empathy towards others

**Unacceptable behavior:**
- Harassment, trolling, or insulting comments
- Personal or political attacks
- Public or private harassment
- Publishing others' private information
- Other conduct inappropriate in a professional setting

### Enforcement

Violations of the Code of Conduct may be reported to: sanae-abe@m3.com

All complaints will be reviewed and investigated promptly and fairly.

## Questions?

If you have questions about contributing, feel free to:
- Open a discussion on GitHub
- Contact the maintainers
- Check existing issues and PRs

---

Thank you for contributing to backup-suite! 🎉
