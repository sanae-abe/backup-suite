# backup-suite

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-latest-blue.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-1.0.0-green.svg)](https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/releases)

**🦀 M3社内向け高速・型安全・企業級バックアップソリューション**

backup-suite は**Rust製**の高性能CLIツールです。優先度別管理・自動スケジューリング・インテリジェント暗号化により、企業環境での効率的なバックアップワークフローを実現します。

## ✨ 主要機能

### 🎯 **優先度別バックアップ管理**
```bash
backup-suite add ~/important-docs --priority high --category work
backup-suite add ~/photos --priority medium --category personal
backup-suite run --priority high  # 高優先度のみ実行
```

### 🔐 **企業級セキュリティ**
```bash
backup-suite run --encrypt --password "your-secure-password"
backup-suite add ~/confidential --priority high --encrypt
```

### ⏰ **自動スケジューリング（macOS launchctl統合）**
```bash
backup-suite schedule setup --high daily --medium weekly --low monthly
backup-suite schedule enable      # 全優先度の自動実行を有効化
backup-suite schedule status      # 現在の設定確認
```

### 📊 **包括的な管理機能**
```bash
backup-suite dashboard            # 統計ダッシュボード
backup-suite history --days 30    # 30日間の実行履歴
backup-suite cleanup --days 7     # 7日以上古いバックアップ削除
backup-suite restore             # 最新バックアップから復元
```

## 🚀 インストール（M3社内GitLab Package Registry）

### 前提条件

**Rustツールチェーンのインストール**が必要です：

```bash
# 1. Rustup（Rustインストーラー）をダウンロード・実行
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 環境変数を読み込み
source ~/.cargo/env

# 3. インストール確認
rustc --version
cargo --version
```

### 🎯 推奨インストール方法: GitLab Package Registry

#### ステップ1: カスタムレジストリ設定（初回のみ）

```bash
# 設定スクリプトを実行（対話的セットアップ）
curl -sSL https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/raw/main/setup-cargo-registry.sh | bash
```

または手動設定：

```bash
# スクリプトをダウンロードして実行
curl -o setup-cargo-registry.sh \
  https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/raw/main/setup-cargo-registry.sh

chmod +x setup-cargo-registry.sh
./setup-cargo-registry.sh
```

#### ステップ2: backup-suiteインストール

```bash
# M3内部レジストリからインストール
cargo install backup-suite --registry m3-internal

# 動作確認
backup-suite --version
backup-suite --help
```

### 🔄 アップデート

```bash
# 最新版に更新
cargo install backup-suite --registry m3-internal --force

# バージョン確認
backup-suite --version
```

### 🧹 アンインストール

```bash
# backup-suiteを削除
cargo uninstall backup-suite

# 設定ファイル削除（オプション）
rm -rf ~/.config/backup-suite/
```

## 📦 プロジェクトでの依存関係として使用

`Cargo.toml`に追加：

```toml
[dependencies]
backup-suite = { version = "1.0", registry = "m3-internal" }
```

```bash
# 依存関係追加
cargo add backup-suite --registry m3-internal

# ビルド
cargo build
```

## 🛠️ 初期設定・基本的な使用例

### 初期設定
```bash
# 対話的初期設定
backup-suite init --interactive

# 設定確認
backup-suite config show

# 設定場所
# Linux/macOS: ~/.config/backup-suite/config.toml
```

### 基本的な使用例

1. **ファイルを追加**
```bash
backup-suite add ~/Documents/project --priority high --category development
backup-suite add ~/Photos --priority medium --category personal
```

2. **対象一覧確認**
```bash
backup-suite list
backup-suite list --priority high  # 高優先度のみ
```

3. **バックアップ実行**
```bash
backup-suite run                   # 全対象実行
backup-suite run --priority high   # 高優先度のみ
backup-suite run --category work   # 特定カテゴリのみ
backup-suite run --dry-run         # ドライラン（確認のみ）

# 暗号化バックアップ
backup-suite run --encrypt --password "secure-password"
```

4. **自動化設定**
```bash
# 優先度別スケジュール設定
backup-suite schedule setup --high daily --medium weekly --low monthly
backup-suite schedule enable
```

## 🏗️ 企業環境での設定

### 基本設定例（~/.config/backup-suite/config.toml）
```toml
[general]
log_level = "info"
log_file = "~/.local/share/backup-suite/logs/backup.log"

[storage]
type = "local"
path = "/backup/storage"
compression = "gzip"
encryption = true
encryption_key_file = "~/.config/backup-suite/keys/backup.key"

[schedule]
enabled = true
daily_time = "02:00"
weekly_day = "sunday"
monthly_day = 1

[targets]
[[targets.directories]]
name = "documents"
path = "~/Documents"
exclude = ["*.tmp", "*.cache", ".DS_Store"]

[[targets.directories]]
name = "projects"
path = "~/Projects"
exclude = ["node_modules/", "target/", ".git/", "*.log"]
```

### 企業プロキシ環境での設定
```toml
# ~/.cargo/config.toml（レジストリ設定と併記）
[http]
proxy = "http://proxy.company.com:8080"
ssl-verify = true
cainfo = "/etc/ssl/certs/company-ca.crt"

[registries.m3-internal]
index = "sparse+https://rendezvous.m3.com:3789/api/v4/projects/123/packages/cargo/"
token = "glpat-xxxxxxxxxxxxxxxxxxxx"
```

## 📋 全コマンドリファレンス

| コマンド | 説明 | 例 |
|----------|------|-----|
| **init** | 対話的初期設定 | `backup-suite init --interactive` |
| **config** | 設定管理 | `backup-suite config show` |
| **add** | バックアップ対象追加 | `backup-suite add ~/docs --priority high` |
| **list, ls** | 対象一覧表示 | `backup-suite list --priority medium` |
| **remove** | 対象削除 | `backup-suite remove ~/old-files` |
| **run** | バックアップ実行 | `backup-suite run --encrypt` |
| **restore** | バックアップ復元 | `backup-suite restore --from backup-20251104` |
| **cleanup** | 古いバックアップ削除 | `backup-suite cleanup --days 30` |
| **status** | 現在の状態表示 | `backup-suite status` |
| **history** | 実行履歴表示 | `backup-suite history --days 7` |
| **schedule** | スケジューリング管理 | `backup-suite schedule enable` |

## 🛡️ セキュリティ・品質

### **企業級セキュリティ**
- AES-256-GCM暗号化対応
- 安全なパスワードベース鍵導出（Argon2）
- ローカル専用（クラウド非依存）
- 設定ファイルの適切な権限管理

### **型安全性・メモリ安全性**
- Rustの強力な型システムで実行時エラーを最小化
- メモリ安全性保証（バッファオーバーフロー、メモリリーク防止）
- コンパイル時エラー検出

## 🔧 技術スタック

- **言語**: Rust（最新安定版）
- **CLI**: clap 4.x （コマンドライン解析・補完生成）
- **暗号化**: AES-256-GCM、Argon2
- **設定**: TOML （人間にとって読みやすい設定形式）
- **スケジューリング**: macOS launchctl、Linux systemd

## 📚 ドキュメント

### 詳細ドキュメント
- [📦 インストールガイド](INSTALL.md) - 詳細なインストール手順・トラブルシューティング
- [🔧 Package Registry設定](docs/PACKAGE_REGISTRY_SETUP.md) - GitLab Package Registry詳細設定

### 企業内サポート
- **GitLab Issues**: [問題報告・機能要求](https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/issues)
- **開発者**: sanae-abe@m3.com
- **内部Wiki**: M3社内ナレッジベースを参照

## 🚀 企業内配布状況

### 対応プラットフォーム
| OS | アーキテクチャ | 対応状況 |
|----|-----------------|----------|
| 🐧 Linux | x86_64 | ✅ 完全対応 |
| 🐧 Linux | aarch64 | ✅ 完全対応 |
| 🍎 macOS | x86_64 | ✅ 完全対応 |
| 🍎 macOS | Apple Silicon | ✅ 完全対応 |

### 配布方法
- **主要配布**: GitLab Package Registry（推奨）
- **GitLabインスタンス**: rendezvous.m3.com:3789
- **レジストリ名**: m3-internal
- **CI/CD**: 自動ビルド・テスト・配布

## 🤝 M3社内での貢献

### 開発環境セットアップ
```bash
# リポジトリクローン
git clone ssh://git@rendezvous.m3.com:3789/sanae-abe/backup-suite.git
cd backup-suite

# 開発環境構築
cargo build
cargo test
```

### 貢献方法
1. GitLab Issueで問題報告・機能提案
2. Merge Requestで改善・修正
3. ドキュメント改善
4. 使用体験のフィードバック

## 📄 ライセンス

MIT License - 詳細は [LICENSE](LICENSE) ファイルを参照

---

**🦀 backup-suite - M3社内向け高速・安全・企業級バックアップソリューション**

**開発者**: sanae-abe@m3.com
**GitLab**: https://rendezvous.m3.com:3789/sanae-abe/backup-suite
**Package Registry**: m3-internal