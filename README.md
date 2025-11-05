# backup-suite

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-latest-blue.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-1.0.0-green.svg)](https://rendezvous.m3.com/sanae-abe/backup-suite/-/releases)

**M3社内向バックアップソリューション**

## ✨ 主要機能

### 🎯 **優先度別バックアップ管理**
重要度に応じてファイルを分類し、効率的にバックアップできます
- **重要な仕事ファイル**は毎日自動バックアップ
- **写真や個人ファイル**は週次バックアップ
- **アーカイブファイル**は月次バックアップ

### 🔐 **軍事レベルの暗号化保護**
銀行や政府機関と同じレベルの暗号化で、大切なファイルを完全に保護できます
- **AES-256-GCM暗号化**で解読は事実上不可能
- **パソコン盗難時**でもデータは完全に安全
- **クラウド保存時**も第三者は絶対に見れない
- **パスワード**がないと誰も開けません

### ⏰ **完全自動化されたスケジューリング**
一度設定すれば、あとは完全に自動でバックアップが実行されます
- **設定後は手動操作不要**で自動実行
- **重要度別に頻度を調整**（毎日・週次・月次）
- **バックアップ忘れ**を完全に防止

### 📊 **わかりやすい管理とメンテナンス**
バックアップの状況をひと目で確認し、簡単にメンテナンスできます
- **どれくらいバックアップしたか**統計で確認
- **いつ実行されたか**履歴で確認
- **古いバックアップ**を自動削除してディスク節約
- **データが壊れた時**の簡単復元

## 🚀 インストール

### 前提条件

**Rustツールチェーンのインストール**が必要です：

```bash
# 1. Rustup（Rustインストーラー）をダウンロード・実行
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 環境変数を読み込み
source ~/.cargo/env

# 新しいターミナルを開くか、以下を実行
# bash使用時
source ~/.bashrc

# zsh使用時（macOS標準）
source ~/.zshrc

# 3. インストール確認
rustc --version
cargo --version
```

### 🚀 インストール

```bash
# 1. リポジトリをクローン
git clone https://rendezvous.m3.com/sanae-abe/backup-suite.git
cd backup-suite

# 2. ビルド&インストール
cargo install --path .

# 3. 動作確認
backup-suite --version
backup-suite --help
```

**SSH接続を使用する場合**:
```bash
git clone git@rendezvous.m3.com:sanae-abe/backup-suite.git
cd backup-suite
cargo install --path .
```

### 🔄 アップデート

```bash
# 1. 最新ソースを取得
cd backup-suite  # プロジェクトディレクトリ
git pull

# 2. 再ビルド&インストール
cargo install --path . --force

# 3. バージョン確認
backup-suite --version
```

### 🧹 アンインストール

```bash
# backup-suiteを削除
cargo uninstall backup-suite

# 設定ファイル削除（オプション）
rm -rf ~/.config/backup-suite/
```

### 🔧 トラブルシューティング

#### よくある問題と解決策

**問題1**: `curl: (35) LibreSSL SSL routines: ST_CONNECT:tlsv1 alert protocol version`
```bash
# 解決策: ソースからビルドを使用
git clone https://rendezvous.m3.com/sanae-abe/backup-suite.git
cd backup-suite
cargo install --path .
```

**問題2**: `rustc` または `cargo` コマンドが見つからない
```bash
# 解決策: Rustツールチェーンを再インストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**問題3**: コンパイルエラーが発生する
```bash
# 解決策: Rustを最新版に更新
rustup update
cargo clean  # キャッシュクリア
cargo build  # 再ビルド
```


## 🛠️ 初期設定・基本的な使用例

### 初期設定

#### 1. 基本セットアップ
```bash
# 対話的初期設定
backup-suite init --interactive

# 設定確認
backup-suite config show

# 設定場所
# Linux/macOS: ~/.config/backup-suite/config.toml
```

#### 2. バックアップ保存先の設定
**Google Driveに保存先を設定**します：

```bash
# Google Driveの保存先を設定
backup-suite config set storage.path "/Users/あなたのユーザー名/Library/CloudStorage/GoogleDrive-your@email.com/マイドライブ/backup-storage"
```

#### 3. 設定確認
```bash
# 設定内容を確認
backup-suite config show

# バックアップ先ディレクトリの確認
backup-suite status
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

## 🏗️ 基本設定例

### 基本設定例（~/.config/backup-suite/config.toml）
```toml
[general]
log_level = "info"
log_file = "~/.local/share/backup-suite/logs/backup.log"

[storage]
type = "local"
path = "/Users/john/Library/CloudStorage/GoogleDrive-john@example.com/マイドライブ/backup-storage"
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


## 📋 全コマンドリファレンス

| コマンド     | 説明                 | 例                                            |
| ------------ | -------------------- | --------------------------------------------- |
| **init**     | 対話的初期設定       | `backup-suite init --interactive`             |
| **config**   | 設定管理             | `backup-suite config show`                    |
| **add**      | バックアップ対象追加 | `backup-suite add ~/docs --priority high`     |
| **list, ls** | 対象一覧表示         | `backup-suite list --priority medium`         |
| **remove**   | 対象削除             | `backup-suite remove ~/old-files`             |
| **run**      | バックアップ実行     | `backup-suite run --encrypt`                  |
| **restore**  | バックアップ復元     | `backup-suite restore --from backup-20251104` |
| **cleanup**  | 古いバックアップ削除 | `backup-suite cleanup --days 30`              |
| **status**   | 現在の状態表示       | `backup-suite status`                         |
| **history**  | 実行履歴表示         | `backup-suite history --days 7`               |
| **schedule** | スケジューリング管理 | `backup-suite schedule enable`                |

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

## 🚀 企業内配布状況

### 対応プラットフォーム
| OS      | アーキテクチャ | 対応状況   |
| ------- | -------------- | ---------- |
| 🐧 Linux | x86_64         | ✅ 完全対応 |
| 🐧 Linux | aarch64        | ✅ 完全対応 |
| 🍎 macOS | x86_64         | ✅ 完全対応 |
| 🍎 macOS | Apple Silicon  | ✅ 完全対応 |


---

**backup-suite**

- **開発者**: sanae-abe@m3.com
- **GitLab**: https://rendezvous.m3.com/sanae-abe/backup-suite