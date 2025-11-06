# 🎉 backup-suite v1.0.0

## ✨ 主な機能

- 🚀 高速バックアップ・復元（並列処理対応）
- 🗜️ 効率的な圧縮・重複排除（Zstd/Gzip対応）
- 🔐 AES-256暗号化対応
- 📅 柔軟なスケジューリング（High/Medium/Low優先度）
- 🔄 増分・差分バックアップ
- 🎯 除外パターン（正規表現対応）
- 🛡️ セキュリティ強化（パストラバーサル対策）

## 📦 ダウンロード

| プラットフォーム | バイナリ | SHA256 |
|----------------|---------|--------|
| Linux x64 | [backup-suite-linux-x64.tar.gz](https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/releases/v1.0.0/downloads/backup-suite-linux-x64.tar.gz) | [SHA256](https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/releases/v1.0.0/downloads/backup-suite-linux-x64.tar.gz.sha256) |
| Linux ARM64 | [backup-suite-linux-arm64.tar.gz](https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/releases/v1.0.0/downloads/backup-suite-linux-arm64.tar.gz) | [SHA256](https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/releases/v1.0.0/downloads/backup-suite-linux-arm64.tar.gz.sha256) |

**注**: macOS版は手動ビルドが必要です。CI/CDパイプラインで `build:macos-universal` ジョブを手動実行してください。

## 🚀 クイックインストール

### 手動インストール（推奨）
```bash
# 1. バイナリをダウンロード
curl -LO "https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/jobs/artifacts/v1.0.0/raw/backup-suite-complete-package.tar.gz?job=package:create-distributions"

# 2. 解凍
tar -xzf backup-suite-*.tar.gz

# 3. バイナリを適切な場所に移動
sudo mv backup-suite /usr/local/bin/

# 4. 動作確認
backup-suite --version
```

### ローカルインストール
```bash
# 1. リポジトリをクローン
git clone ssh://git@rendezvous.m3.com:3789/sanae-abe/backup-suite.git
cd backup-suite

# 2. インストールスクリプトを実行
bash install.sh

# または、システム全体にインストール（/usr/local/bin）
sudo bash install.sh

# 3. 動作確認
backup-suite --version
```

### ソースからビルド
```bash
# 1. リポジトリをクローン
git clone ssh://git@rendezvous.m3.com:3789/sanae-abe/backup-suite.git
cd backup-suite

# 2. ビルド&インストール
cargo install --path .

# 3. 動作確認
backup-suite --version
backup-suite --help
```

## 📊 品質指標

- ✅ 79 単体テスト全通過
- ✅ 16 統合テスト全通過
- ✅ 68 doctest全通過
- ✅ CI/CD完全通過（validate→test→build）
- ✅ clippy警告0件
- ✅ rustfmt準拠

## 🔧 技術スタック

- Rust 1.75+
- rayon（並列処理）
- zstd/flate2（圧縮）
- aes-gcm（暗号化）
- GitLab CI/CD

## 📦 対応プラットフォーム

- Linux x64
- Linux ARM64
- macOS Universal (Intel + Apple Silicon、手動ビルド)

## 🔗 関連リンク

- [ドキュメント](https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/blob/main/README.md)
- [インストールガイド](https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/blob/main/README.md#インストール)
- [使用方法](https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/blob/main/README.md#基本的な使用方法)

---

**開発者**: sanae-abe@m3.com
