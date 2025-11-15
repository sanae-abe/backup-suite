# Shell Completion Guide

This guide covers installation and troubleshooting for shell completion in **Zsh**, **Bash**, and **Fish**.

**Supported Languages**: English (en), Japanese (ja), Simplified Chinese (zh-CN), Traditional Chinese (zh-TW)

---

## Table of Contents

- [Zsh Completion](#zsh-completion)
  - [Basic Installation](#zsh-basic-installation)
  - [Multilingual Support](#zsh-multilingual-support)
  - [Manual Language Selection](#zsh-manual-language-selection)
- [Bash Completion](#bash-completion)
  - [Basic Installation](#bash-basic-installation)
  - [Verification](#bash-verification)
- [Fish Completion](#fish-completion)
  - [Basic Installation](#fish-basic-installation)
  - [Verification](#fish-verification)
- [Troubleshooting](#troubleshooting)
  - [Common Issues](#common-issues)
  - [Zsh-Specific Issues](#zsh-specific-issues)
  - [Bash-Specific Issues](#bash-specific-issues)
  - [Fish-Specific Issues](#fish-specific-issues)

---

## Zsh Completion

### Zsh Basic Installation

Zsh completion provides intelligent command and option suggestions with descriptions in your preferred language.

**Step 1: Create completion directory**

```bash
mkdir -p ~/.zfunc
```

**Step 2: Add to `.zshrc`**

Add these lines to your `~/.zshrc` file:

```bash
# Enable completion system
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
```

**Step 3: Generate completion script**

```bash
# Auto-detect language from $LANG environment variable
backup-suite completion zsh > ~/.zfunc/_backup-suite
```

**Step 4: Reload shell**

```bash
exec zsh
```

**Step 5: Test completion**

```bash
backup-suite <TAB>
```

You should see command suggestions with descriptions in your system's language.

---

### Zsh Multilingual Support

backup-suite supports **4 languages** for completion descriptions:

| Language | Code | Example `$LANG` |
|----------|------|-----------------|
| 🇬🇧 English | `en` | `en_US.UTF-8` |
| 🇯🇵 Japanese | `ja` | `ja_JP.UTF-8` |
| 🇨🇳 Simplified Chinese | `zh-CN` | `zh_CN.UTF-8` |
| 🇹🇼 Traditional Chinese | `zh-TW` | `zh_TW.UTF-8`, `zh_HK.UTF-8` |

**Auto-Detection Logic**:

```bash
# Language is detected from $LANG environment variable:
# ja_* or jp_*  → Japanese
# zh_CN*        → Simplified Chinese
# zh_TW*, zh_HK* → Traditional Chinese
# others        → English (default)
```

**Check your current language**:

```bash
echo $LANG
# Example output: ja_JP.UTF-8 (Japanese)
```

---

### Zsh Manual Language Selection

Use `scripts/generate-completion.sh` to manually specify a language:

```bash
# Japanese
./scripts/generate-completion.sh ja

# Simplified Chinese
./scripts/generate-completion.sh zh-CN

# Traditional Chinese
./scripts/generate-completion.sh zh-TW

# English
./scripts/generate-completion.sh en

# Auto-detect (default)
./scripts/generate-completion.sh
```

**What the script does**:

1. Generates base completion with `backup-suite completion zsh`
2. Translates command descriptions to the selected language
3. Saves to `~/.zfunc/_backup-suite`

**Reload after changing language**:

```bash
source ~/.zfunc/_backup-suite
# or restart shell
exec zsh
```

---

## Bash Completion

### Bash Basic Installation

Bash completion provides command and option suggestions (English only).

**Step 1: Locate Bash completion directory**

```bash
# macOS (Homebrew)
COMPLETION_DIR="$(brew --prefix)/etc/bash_completion.d"

# Linux (Debian/Ubuntu)
COMPLETION_DIR="/etc/bash_completion.d"

# Or user-specific directory
COMPLETION_DIR="~/.local/share/bash-completion/completions"
```

**Step 2: Generate completion script**

```bash
# For system-wide installation (requires sudo)
sudo backup-suite completion bash > /etc/bash_completion.d/backup-suite

# For user-specific installation
mkdir -p ~/.local/share/bash-completion/completions
backup-suite completion bash > ~/.local/share/bash-completion/completions/backup-suite
```

**Step 3: Enable bash-completion (if not already enabled)**

Add to `~/.bashrc` or `~/.bash_profile`:

```bash
# macOS (Homebrew)
if [ -f $(brew --prefix)/etc/bash_completion ]; then
  . $(brew --prefix)/etc/bash_completion
fi

# Linux
if [ -f /etc/bash_completion ]; then
  . /etc/bash_completion
fi
```

**Step 4: Reload shell**

```bash
source ~/.bashrc
# or
exec bash
```

### Bash Verification

Test completion:

```bash
backup-suite <TAB><TAB>
```

You should see available commands listed.

---

## Fish Completion

### Fish Basic Installation

Fish completion provides intelligent suggestions (English only).

**Step 1: Generate completion script**

```bash
# Fish completions directory
mkdir -p ~/.config/fish/completions

# Generate completion
backup-suite completion fish > ~/.config/fish/completions/backup-suite.fish
```

**Step 2: Reload Fish configuration**

```bash
# Reload configuration
source ~/.config/fish/config.fish

# or restart Fish
exec fish
```

### Fish Verification

Test completion:

```bash
backup-suite <TAB>
```

You should see command suggestions with descriptions.

---

## Troubleshooting

### Common Issues

#### ❌ Issue: Completion not working at all

**Symptoms**: No suggestions appear when pressing `<TAB>`

**Solutions**:

1. **Verify completion file exists**:
   ```bash
   # Zsh
   ls -la ~/.zfunc/_backup-suite

   # Bash
   ls -la /etc/bash_completion.d/backup-suite
   # or
   ls -la ~/.local/share/bash-completion/completions/backup-suite

   # Fish
   ls -la ~/.config/fish/completions/backup-suite.fish
   ```

2. **Check file permissions**:
   ```bash
   # File should be readable
   chmod 644 <completion-file>
   ```

3. **Restart shell**:
   ```bash
   # Zsh
   exec zsh

   # Bash
   exec bash

   # Fish
   exec fish
   ```

---

#### ❌ Issue: Wrong language displayed

**Symptoms**: Completion shows English but you want Japanese (or vice versa)

**Solutions**:

1. **Check your `$LANG` environment variable**:
   ```bash
   echo $LANG
   # Should match your desired language (e.g., ja_JP.UTF-8)
   ```

2. **Manually specify language** (Zsh only):
   ```bash
   ./scripts/generate-completion.sh ja
   source ~/.zfunc/_backup-suite
   ```

3. **Change system language**:
   ```bash
   # Add to ~/.zshrc (or ~/.bashrc)
   export LANG=ja_JP.UTF-8  # For Japanese
   export LANG=zh_CN.UTF-8  # For Simplified Chinese
   export LANG=zh_TW.UTF-8  # For Traditional Chinese
   export LANG=en_US.UTF-8  # For English
   ```

---

#### ❌ Issue: Completion file not found error

**Symptoms**: Shell reports `command not found: backup-suite` when generating completion

**Solutions**:

1. **Verify backup-suite is installed**:
   ```bash
   which backup-suite
   backup-suite --version
   ```

2. **Check PATH**:
   ```bash
   echo $PATH
   # Should include ~/.cargo/bin or Homebrew bin directory
   ```

3. **Reinstall if necessary**:
   ```bash
   # Homebrew
   brew reinstall backup-suite

   # Cargo
   cargo install backup-suite --features smart --force
   ```

---

### Zsh-Specific Issues

#### ❌ Issue: `compinit` warnings about insecure directories

**Symptoms**: Warning messages like `Ignore insecure directories` when starting Zsh

**Solution**: Fix directory permissions:

```bash
chmod go-w ~/.zfunc
chmod 644 ~/.zfunc/_backup-suite
compaudit | xargs chmod go-w
```

---

#### ❌ Issue: Completion not appearing after `.zshrc` changes

**Symptoms**: Added `fpath` and `compinit` to `.zshrc`, but completion still doesn't work

**Solutions**:

1. **Verify `.zshrc` order** (must be in this order):
   ```bash
   # CORRECT ORDER:
   fpath=(~/.zfunc $fpath)        # 1. Add to fpath FIRST
   autoload -Uz compinit && compinit  # 2. Initialize completion SECOND
   ```

2. **Clear completion cache**:
   ```bash
   rm -f ~/.zcompdump*
   exec zsh
   ```

3. **Debug fpath**:
   ```bash
   echo $fpath
   # Should include: /Users/<username>/.zfunc
   ```

---

#### ❌ Issue: Multilingual script fails with "sed: command not found"

**Symptoms**: `./scripts/generate-completion.sh ja` fails

**Solution**: Ensure GNU sed or BSD sed is installed:

```bash
# macOS (BSD sed is built-in, should work)
which sed  # /usr/bin/sed

# Linux (GNU sed)
which sed  # /bin/sed or /usr/bin/sed
```

If `sed` is missing, install it:

```bash
# macOS (Homebrew)
brew install gnu-sed
# Then use gsed instead of sed in the script

# Linux
sudo apt install sed  # Debian/Ubuntu
sudo yum install sed  # CentOS/RHEL
```

---

### Bash-Specific Issues

#### ❌ Issue: Completion only works for root user

**Symptoms**: Completion works with `sudo`, but not as regular user

**Solution**: Install to user-specific directory:

```bash
mkdir -p ~/.local/share/bash-completion/completions
backup-suite completion bash > ~/.local/share/bash-completion/completions/backup-suite

# Add to ~/.bashrc
if [ -d ~/.local/share/bash-completion/completions ]; then
  for file in ~/.local/share/bash-completion/completions/*; do
    [ -r "$file" ] && . "$file"
  done
fi

source ~/.bashrc
```

---

#### ❌ Issue: bash-completion package not installed

**Symptoms**: Completion doesn't work even after following installation steps

**Solution**: Install bash-completion package:

```bash
# macOS (Homebrew)
brew install bash-completion@2

# Add to ~/.bash_profile
if [ -f $(brew --prefix)/etc/bash_completion ]; then
  . $(brew --prefix)/etc/bash_completion
fi

# Linux (Debian/Ubuntu)
sudo apt install bash-completion

# Linux (CentOS/RHEL)
sudo yum install bash-completion
```

---

### Fish-Specific Issues

#### ❌ Issue: Old completion cached

**Symptoms**: Completion shows outdated commands after updating backup-suite

**Solution**: Clear Fish completion cache:

```bash
# Remove cached completions
rm -rf ~/.cache/fish
rm -rf ~/.local/share/fish/generated_completions

# Regenerate
backup-suite completion fish > ~/.config/fish/completions/backup-suite.fish
exec fish
```

---

#### ❌ Issue: Completion not auto-loading

**Symptoms**: Completion only works after manually sourcing the file

**Solution**: Verify Fish version and configuration:

```bash
# Check Fish version (3.0+ required)
fish --version

# Ensure config directory exists
mkdir -p ~/.config/fish/completions

# Verify file is in correct location
ls -la ~/.config/fish/completions/backup-suite.fish
```

---

## Additional Resources

- **Official Documentation**: [README.md](../README.md)
- **Multilingual Script**: [scripts/generate-completion.sh](../scripts/generate-completion.sh)
- **Issue Tracker**: [GitHub Issues](https://github.com/sanae-abe/backup-suite/issues)

---

## Quick Reference

### Commands Summary

| Shell | Generate Completion | Install Location |
|-------|---------------------|------------------|
| **Zsh** | `backup-suite completion zsh > ~/.zfunc/_backup-suite` | `~/.zfunc/_backup-suite` |
| **Bash** | `backup-suite completion bash > ~/.local/share/bash-completion/completions/backup-suite` | `~/.local/share/bash-completion/completions/backup-suite` |
| **Fish** | `backup-suite completion fish > ~/.config/fish/completions/backup-suite.fish` | `~/.config/fish/completions/backup-suite.fish` |

### Language Codes (Zsh Only)

| Language | Code | Script Command |
|----------|------|----------------|
| Auto-detect | auto | `./scripts/generate-completion.sh` |
| English | en | `./scripts/generate-completion.sh en` |
| Japanese | ja | `./scripts/generate-completion.sh ja` |
| Simplified Chinese | zh-CN | `./scripts/generate-completion.sh zh-CN` |
| Traditional Chinese | zh-TW | `./scripts/generate-completion.sh zh-TW` |

---

**Last Updated**: 2025-11-15
