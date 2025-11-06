# パッケージ公開ガイド

## crates.io への公開

### 1. 事前準備

```bash
# crates.io アカウント作成
# https://crates.io/ でGitHubアカウントでログイン

# APIトークン取得
# https://crates.io/settings/tokens

# ログイン
cargo login <YOUR_API_TOKEN>
```

### 2. 公開前チェック

```bash
# ビルド確認
cargo build --release

# テスト実行
cargo test

# パッケージング確認
cargo package --allow-dirty

# ドライラン
cargo publish --dry-run
```

### 3. 公開実行

```bash
cargo publish
```

### 4. インストール確認

```bash
cargo install backup-suite
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
