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
| macOS x64 | [backup-suite-macos-x64.tar.gz](https://github.com/sanae-abe/backup-suite/releases/download/v1.0.0/backup-suite-macos-x64.tar.gz) | - |
| macOS ARM64 | [backup-suite-macos-arm64.tar.gz](https://github.com/sanae-abe/backup-suite/releases/download/v1.0.0/backup-suite-macos-arm64.tar.gz) | - |


## 🚀 インストール

Rustをインストールして、自分の環境で直接ビルドします。

```bash
# 1. リポジトリをクローン
git clone git@github.com:sanae-abe/backup-suite.git
cd backup-suite

# 2. Rustインストール（未インストールの場合）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 3. ビルド＆インストール
cargo build --release
mkdir -p ~/.local/bin
cp target/release/backup-suite ~/.local/bin/

# 4. PATHに追加（初回のみ）
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# 5. 動作確認
backup-suite --version
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
- GitHub Actions

## 📦 対応プラットフォーム

- macOS x64 (Intel)
- macOS ARM64 (Apple Silicon)

## 🔗 関連リンク

- [ドキュメント](https://github.com/sanae-abe/backup-suite/blob/main/README.md)
- [インストールガイド](https://github.com/sanae-abe/backup-suite/blob/main/README.md#インストール)
- [使用方法](https://github.com/sanae-abe/backup-suite/blob/main/README.md#基本的な使用方法)

---

**Developer**: sanae.a.sunny@gmail.com
