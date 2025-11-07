# パッケージ公開ガイド（v1.0.0+）

## 📋 事前準備

### 必須チェックリスト

- [ ] 全テスト合格（343 tests passed）
- [ ] リリースビルド成功（警告0件）
- [ ] README.md / README.en.md 更新完了
- [ ] CHANGELOG.md / CHANGELOG.en.md 更新完了
- [ ] Cargo.toml のバージョン更新

## 🚀 GitHub Release 作成

### 1. タグ作成・push

```bash
# Cargo.toml のバージョン確認
grep "^version" Cargo.toml

# ビルド・テスト確認
cargo build --release
cargo test

# Git commit & tag
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to v1.1.0"
git tag -a v1.1.0 -m "backup-suite v1.1.0

主要機能:
- ...

🤖 Generated with Claude Code"

git push origin main --tags
```

### 2. GitHub Release作成（gh コマンド推奨）

```bash
# GitHub CLI でリリース作成
gh release create v1.1.0 \
  --title "v1.1.0 - リリースタイトル" \
  --notes-file CHANGELOG.md

# または Web UI で作成
# https://github.com/sanae-abe/backup-suite/releases/new
```

## 📦 crates.io 公開

### 1. crates.io ログイン（初回のみ）

```bash
# APIトークン取得
# https://crates.io/settings/tokens

# ログイン
cargo login <YOUR_API_TOKEN>
```

### 2. 公開前チェック

```bash
# パッケージング確認
cargo package

# ドライラン
cargo publish --dry-run
```

### 3. 公開実行

```bash
cargo publish
```

### 4. インストール確認

```bash
# crates.io から直接インストール
cargo install backup-suite

# バージョン確認
backup-suite --version
```

## Homebrew への公開

### 方法1: Homebrew Tap リポジトリ作成（推奨）

```bash
# 1. Tap リポジトリ作成
# GitHub で新規リポジトリ作成: homebrew-backup-suite

# 2. Formula 配置
git clone git@github.com:sanae-abe/homebrew-backup-suite.git
cd homebrew-backup-suite
cp ../backup-suite/Formula/backup-suite.rb .
git add backup-suite.rb
git commit -m "Add backup-suite formula"
git push origin main

# 3. ユーザーがインストール
brew tap sanae-abe/backup-suite
brew install backup-suite
```

### 方法2: Homebrew Core への PR（人気が出てから）

1. リリースを作成（GitHub Releases）
2. SHA256計算
```bash
curl -L https://github.com/sanae-abe/backup-suite/archive/refs/tags/v1.0.0.tar.gz | shasum -a 256
```
3. Formula の sha256 を更新
4. Homebrew/homebrew-core へ PR

## バージョン更新手順

### 1. バージョン更新

```bash
# Cargo.toml のバージョン更新
# version = "1.0.0" → "1.1.0"

# 動作確認
cargo build --release
cargo test
```

### 2. Git タグ作成

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to v1.1.0"
git tag v1.1.0
git push origin main --tags
```

### 3. パッケージ更新

```bash
# crates.io
cargo publish

# Homebrew Formula
# Formula/backup-suite.rb の url, sha256 を更新
```

## トラブルシューティング

### crates.io で公開エラー

```bash
# 名前衝突の場合は名前変更
# Cargo.toml の name を変更

# 依存関係エラー
cargo update
cargo check
```

### Homebrew でビルドエラー

```bash
# Formula テスト
brew install --build-from-source ./Formula/backup-suite.rb
brew test backup-suite
```
