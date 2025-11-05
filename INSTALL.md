# 📦 backup-suite インストールガイド

> **企業向け高速バックアップソリューション**
> バージョン: v1.0.0 | 対応OS: Linux, macOS

## 📋 前提条件

### Rustツールチェーンのインストール

Package Registryを使用する場合は、事前にRustとCargoのインストールが必要です：

```bash
# 1. Rustup（Rustインストーラー）をダウンロード・実行
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 環境変数を読み込み
source ~/.cargo/env

# 3. インストール確認
rustc --version
cargo --version

# 4. 最新版に更新（推奨）
rustup update
```

### 企業プロキシ環境での設定

```bash
# プロキシ環境変数設定（必要に応じて）
export HTTP_PROXY="http://proxy.company.com:8080"
export HTTPS_PROXY="http://proxy.company.com:8080"

# Rustup用プロキシ設定
export RUSTUP_HTTP_PROXY="$HTTP_PROXY"
export RUSTUP_HTTPS_PROXY="$HTTPS_PROXY"
```

---

## 🎯 推奨インストール方法

### 方法1: GitLab Package Registry（開発者向け・推奨）

企業内のGitLabレジストリからCargoを使用してインストール：

```bash
# 前提: Rust/Cargoがインストール済みであること
cargo --version  # 確認

# 1. レジストリ設定（初回のみ）
curl -sSL https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/raw/main/setup-cargo-registry.sh | bash

# 2. backup-suiteインストール
cargo install backup-suite --registry m3-internal

# 3. 動作確認
backup-suite --version
backup-suite --help
```

**メリット**:
- 標準的なCargoワークフロー
- 自動アップデート対応
- バージョン管理が簡単
- 依存関係として使用可能

### 方法2: バイナリ直接インストール（簡単導入）

```bash
# Linux/macOS用ワンライナー
curl -sSL https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/raw/main/install.sh | bash

# インストール確認
backup-suite --version
backup-suite --help
```

---

## 📋 対応プラットフォーム

| OS | アーキテクチャ | バイナリ名 | 配布方法 |
|----|---------------|-----------|---------|
| 🐧 Linux | x86_64 | `backup-suite-linux-x64` | Registry + Binary |
| 🐧 Linux | aarch64 | `backup-suite-linux-arm64` | Registry + Binary |
| 🍎 macOS | x86_64 | `backup-suite-macos-x64` | Registry + Binary |
| 🍎 macOS | Apple Silicon | `backup-suite-macos-arm64` | Registry + Binary |

---

## 💻 手動インストール手順

### 🐧 Linux

#### 1. バイナリダウンロード
```bash
# 最新リリースURL（例）
LATEST_VERSION="v1.0.0"
DOWNLOAD_URL="https://gitlab.company.com/tools/backup-suite/-/releases/${LATEST_VERSION}/downloads/backup-suite-linux-x64.tar.gz"

# ダウンロード
wget "$DOWNLOAD_URL" -O backup-suite.tar.gz

# または curl使用
curl -L "$DOWNLOAD_URL" -o backup-suite.tar.gz
```

#### 2. 解凍とインストール
```bash
# 解凍
tar -xzf backup-suite.tar.gz

# システム全体にインストール（管理者権限必要）
sudo mv backup-suite /usr/local/bin/
sudo chmod +x /usr/local/bin/backup-suite

# ユーザー個別インストール
mkdir -p ~/.local/bin
mv backup-suite ~/.local/bin/
chmod +x ~/.local/bin/backup-suite

# PATHに追加（~/.bashrc or ~/.zshrc）
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### 3. 動作確認
```bash
backup-suite --version
backup-suite init --help
```

### 🍎 macOS

#### 1. バイナリダウンロード
```bash
# アーキテクチャ確認
arch

# Intel Mac（x86_64）
DOWNLOAD_URL="https://gitlab.company.com/tools/backup-suite/-/releases/v1.0.0/downloads/backup-suite-macos-x64.tar.gz"

# Apple Silicon（arm64）
DOWNLOAD_URL="https://gitlab.company.com/tools/backup-suite/-/releases/v1.0.0/downloads/backup-suite-macos-arm64.tar.gz"

# ダウンロード
curl -L "$DOWNLOAD_URL" -o backup-suite.tar.gz
```

#### 2. インストールと署名許可
```bash
# 解凍
tar -xzf backup-suite.tar.gz

# インストール
sudo mv backup-suite /usr/local/bin/
sudo chmod +x /usr/local/bin/backup-suite

# Gatekeeper許可（初回実行時）
backup-suite --version
# "開発元を確認できません"エラーが出た場合：
# システム環境設定 > セキュリティとプライバシー > "実行を許可"
```

#### 3. Homebrewスタイル（オプション）
```bash
# Homebrewディレクトリにインストール
mv backup-suite /opt/homebrew/bin/  # Apple Silicon
mv backup-suite /usr/local/bin/     # Intel Mac
```

### 🪟 Windows

#### 1. PowerShellでダウンロード
```powershell
# PowerShell（管理者として実行推奨）
$DownloadUrl = "https://gitlab.company.com/tools/backup-suite/-/releases/v1.0.0/downloads/backup-suite-windows-x64.zip"
$ZipFile = "$env:TEMP\backup-suite.zip"
$ExtractDir = "C:\Tools\backup-suite"

# ダウンロード
Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipFile

# 解凍
Expand-Archive -Path $ZipFile -DestinationPath $ExtractDir -Force

# PATHに追加
$NewPath = [Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + $ExtractDir
[Environment]::SetEnvironmentVariable("Path", $NewPath, "Machine")
```

#### 2. 動作確認
```powershell
# 新しいPowerShellセッションで実行
backup-suite.exe --version
backup-suite.exe init --help
```

---

## 🔧 初期設定

### 基本設定
```bash
# 対話式設定（推奨）
backup-suite init --interactive

# 設定ファイル確認
backup-suite config show

# 設定場所
# Linux/macOS: ~/.config/backup-suite/config.toml
# Windows: %APPDATA%\backup-suite\config.toml
```

### 基本的なバックアップ設定例
```bash
# バックアップ対象追加
backup-suite add ~/Documents --name "documents" --schedule daily
backup-suite add ~/Projects --name "projects" --schedule weekly

# バックアップ先設定
backup-suite config set storage.type local
backup-suite config set storage.path /backup/storage

# 初回バックアップ実行
backup-suite run --dry-run  # 確認実行
backup-suite run            # 実際のバックアップ
```

---

---

## 🔧 GitLab Package Registry 詳細設定

### カスタムレジストリセットアップ

```bash
# 設定スクリプトダウンロード
curl -o setup-cargo-registry.sh \
  https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/raw/main/setup-cargo-registry.sh

# 実行権限付与
chmod +x setup-cargo-registry.sh

# 対話的セットアップ実行
./setup-cargo-registry.sh
```

### 手動レジストリ設定

```toml
# ~/.cargo/config.toml
[registries]
m3-internal = { index = "sparse+https://rendezvous.m3.com:3789/api/v4/projects/123/packages/cargo/" }

[registries.m3-internal]
token = "glpat-xxxxxxxxxxxxxxxxxxxx"  # GitLabアクセストークン

# 企業プロキシ設定（必要に応じて）
[http]
proxy = "http://proxy.company.com:8080"
ssl-verify = true
```

### 使用例

```bash
# プロジェクトでの依存関係として使用
# Cargo.toml
[dependencies]
backup-suite = { version = "1.0", registry = "m3-internal" }

# CLIツールとしてインストール
cargo install backup-suite --registry m3-internal

# アップデート
cargo install backup-suite --registry m3-internal --force
```

---

## 📋 企業内配布用ファイル構成

社内のファイルサーバーやイントラネットでの配布に使用する標準的なファイル構成：

```
backup-suite-distribution/
├── README.md                    # 配布パッケージ説明
├── INSTALL.md                   # 本インストールガイド
├── install.sh                   # Linux/macOS自動インストール
├── install.ps1                  # Windows自動インストール
├── binaries/                    # プラットフォーム別バイナリ
│   ├── linux-x64/
│   │   └── backup-suite
│   ├── linux-arm64/
│   │   └── backup-suite
│   ├── macos-x64/
│   │   └── backup-suite
│   ├── macos-arm64/
│   │   └── backup-suite
│   └── windows-x64/
│       └── backup-suite.exe
├── docs/                        # ドキュメント
│   ├── configuration.md
│   ├── usage-examples.md
│   └── api-reference.md
└── examples/                    # 設定例
    ├── config-templates/
    └── backup-scripts/
```

---

## 🔧 設定ファイルテンプレート

インストール後の基本設定テンプレート：

### Linux/macOS設定例
```toml
# ~/.config/backup-suite/config.toml
[general]
log_level = "info"
log_file = "~/.local/share/backup-suite/logs/backup.log"

[storage]
type = "local"
path = "/backup/storage"
compression = "gzip"
encryption = false

[schedule]
enabled = true
daily_time = "02:00"
weekly_day = "sunday"
monthly_day = 1

[targets]
[[targets.directories]]
name = "documents"
path = "~/Documents"
exclude = ["*.tmp", "*.cache"]

[[targets.directories]]
name = "projects"
path = "~/Projects"
exclude = ["node_modules/", "target/", ".git/"]
```

### プロジェクト用テンプレート
```toml
# ~/.config/backup-suite/config.toml
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

[[targets.databases]]
name = "postgres-main"
type = "postgresql"
connection_string = "postgresql://user:pass@localhost/db"
backup_format = "custom"
```

---

## 📊 ライセンス・利用規約

### 企業内利用ライセンス
```
企業内利用許可ライセンス v1.0

本ソフトウェア（backup-suite）は、[会社名]の従業員による
業務目的での使用に限り、無償で利用することができます。

制限事項：
- 第三者への再配布禁止
- 商用利用時は別途ライセンス必要
- ソースコード改変時は社内承認必要

サポート：
- 技術サポート: it-support@company.com
- バグレポート: https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/issues
- 機能要求: 上記バグレポートと同じ
```

---

## 🚨 トラブルシューティング

### よくある問題と解決方法

#### Package Registry関連

```bash
# 問題: レジストリに接続できない
# 解決: 接続確認とトークン検証
curl -H "PRIVATE-TOKEN: glpat-xxxx" \
  https://gitlab.company.com/api/v4/projects/123/packages/cargo/

# 問題: 認証エラー
# 解決: トークンの再生成と設定
# GitLab > 設定 > アクセストークン > 'read_api', 'read_registry' スコープ
```

#### バイナリインストール関連

```bash
# 問題: PATHが通らない
# 解決: PATH設定確認
echo $PATH | grep -o '/usr/local/bin\|~/.local/bin'

# 問題: 権限エラー
# 解決: ユーザーディレクトリインストール
./install.sh --user

# 問題: アーキテクチャ不一致
# 解決: 正しいバイナリのダウンロード
uname -m  # アーキテクチャ確認
```

#### プロキシ環境での問題

```toml
# ~/.cargo/config.toml
[http]
proxy = "http://proxy.company.com:8080"
ssl-verify = true
cainfo = "/etc/ssl/certs/company-ca.crt"

# 環境変数での設定
export HTTP_PROXY="http://proxy.company.com:8080"
export HTTPS_PROXY="http://proxy.company.com:8080"
```

### 診断スクリプト

```bash
# 診断実行
curl -sSL https://gitlab.company.com/tools/cmdrun/-/raw/main/diagnose.sh | bash

# 手動診断
cmdrun --version                    # バージョン確認
cargo search --registry company cmdrun  # レジストリ接続確認
which cmdrun                       # インストール場所確認
```

---

## 🗑️ アンインストール手順

### Package Registry版のアンインストール

```bash
# Cargoでインストールした場合
cargo uninstall cmdrun

# 設定ファイル削除（オプション）
rm -rf ~/.cargo/config.toml.backup.*
```

### バイナリ版のアンインストール

```bash
# システムインストールの場合（要sudo）
sudo rm /usr/local/bin/cmdrun

# ユーザーインストールの場合
rm ~/.local/bin/cmdrun

# 設定ファイル削除（オプション）
rm -rf ~/.config/cmdrun/
```

### 完全削除

```bash
# 全ての関連ファイル削除
rm -f /usr/local/bin/cmdrun ~/.local/bin/cmdrun
rm -rf ~/.config/cmdrun/
rm -f ~/.gitlab-token
```

---

## 🚀 自動インストールスクリプトの詳細

### install.sh の特徴
- **クロスプラットフォーム**: Linux/macOSでの自動OS・アーキテクチャ検出
- **エラーハンドリング**: 堅牢なエラー処理とロールバック機能
- **進捗表示**: 色付きログとプログレスバー表示
- **権限管理**: システム/ユーザーインストールの自動判定
- **バックアップ**: 既存バイナリの自動バックアップ
- **PATH管理**: 自動的なPATH設定と警告表示

### setup-cargo-registry.sh の特徴
- **対話的設定**: ユーザーフレンドリーな設定プロセス
- **自動検証**: レジストリ接続とアクセス権限の確認
- **プロキシ対応**: 企業環境での制約に対応
- **セキュアな設定**: アクセストークンの安全な管理