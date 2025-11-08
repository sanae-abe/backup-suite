# backup-suite

[![Crates.io](https://img.shields.io/crates/v/backup-suite.svg)](https://crates.io/crates/backup-suite)
[![Documentation](https://docs.rs/backup-suite/badge.svg)](https://docs.rs/backup-suite)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![CI](https://github.com/sanae-abe/backup-suite/workflows/CI/badge.svg)](https://github.com/sanae-abe/backup-suite/actions)

[日本語](README.md) | [English](README.en.md)

> **高速・安全・インテリジェントなローカルバックアップツール**

## 目次

- [主要機能](#主要機能)
- [スクリーンショット](#スクリーンショット)
- [インストール](#インストール)
- [クイックスタート](#クイックスタート)
- [基本的な使用方法](#基本的な使用方法)
- [設定ファイル](#設定ファイル)
- [コマンドリファレンス](#コマンドリファレンス)
- [アップデート・アンインストール](#アップデートアンインストール)
- [セキュリティ・品質](#セキュリティ品質)
- [技術スタック](#技術スタック)
- [対応プラットフォーム](#対応プラットフォーム)
- [ライセンス](#ライセンス)

## 主要機能

### 🎯 優先度別バックアップ管理
- **重要な仕事ファイル**は毎日自動バックアップ
- **写真や個人ファイル**は週次バックアップ
- **アーカイブファイル**は月次バックアップ

### 🔐 軍事レベルの暗号化保護
- **AES-256-GCM暗号化**で解読は事実上不可能
- **Argon2鍵導出**でパスワードから安全な暗号鍵を生成
- **パソコン盗難時**でもデータは完全に安全
- **クラウド保存時**も第三者は絶対に見れない
- **強力なパスワード自動生成**で安全性を確保

### 📦 高速圧縮によるストレージ節約
- **Zstd圧縮**で高速かつ高圧縮率を実現
- **Gzip圧縮**で互換性重視の圧縮
- **圧縮なし**でも選択可能
- **ディスク容量を最大70%削減**

### ⚡ 増分バックアップで超高速化
- **変更ファイルのみバックアップ**で時間を大幅短縮
- **SHA-256ハッシュ**による正確な変更検出
- **バックアップ時間を90%削減**（2回目以降）
- **ストレージ容量を85%削減**（差分のみ保存）
- **自動的にフルバックアップに切り替え**（初回実行時）

### ⏰ 完全自動化されたスケジューリング
- **設定後は手動操作不要**で自動実行
- **重要度別に頻度を調整**（毎日・週次・月次）
- **バックアップ忘れ**を完全に防止
- **macOS launchd/Linux systemd統合**で信頼性の高い自動実行

### 📊 わかりやすい管理とメンテナンス
- **どれくらいバックアップしたか**統計で確認
- **いつ実行されたか**履歴で確認
- **古いバックアップ**を自動削除してディスク節約
- **データが壊れた時**の簡単復元

## スクリーンショット

### ヘルプ画面
<img src="docs/screenshots/help.webp" alt="backup-suite help" width="600">

*コマンド一覧とオプションを日本語で表示*

### バックアップ対象一覧
<img src="docs/screenshots/list.webp" alt="backup-suite list" width="600">

*登録されたバックアップ対象をテーブル形式で表示*

### バックアップ実行
<img src="docs/screenshots/run.webp" alt="backup-suite run" width="600">

*実際のバックアップ実行画面*

### バックアップ実行（ドライラン）
<img src="docs/screenshots/dry-run.webp" alt="backup-suite dry-run" width="600">

*実際にファイルをコピーせずに実行内容を確認*

### バックアップ履歴
<img src="docs/screenshots/history.webp" alt="backup-suite history" width="600">

*過去のバックアップ実行履歴を確認*

## インストール

### Homebrewでインストール（macOS）

```bash
brew tap sanae-abe/backup-suite
brew install backup-suite
```

### Cargoでインストール

```bash
cargo install backup-suite
```

### ソースからビルド

```bash
# 1. リポジトリをクローン
git clone git@github.com:sanae-abe/backup-suite.git
cd backup-suite

# 2. Rustインストール（未インストールの場合）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 3. ビルド＆インストール
cargo build --release
cargo install --path .

# 4. 動作確認
backup-suite --version
```

## クイックスタート

### 1. 基本セットアップ
```bash
# 現在の設定確認
backup-suite status

# 設定ファイルの場所
# ~/.config/backup-suite/config.toml
```

**注意**: 言語は環境変数`LANG`で自動検出されます。日本語環境では自動的に日本語で表示されます。

### 2. バックアップ保存先の設定

```bash
# Google Driveの保存先を設定
backup-suite config set-destination "/Users/あなたのユーザー名/Library/CloudStorage/GoogleDrive-your@email.com/マイドライブ/backup-storage"

# 現在の設定を確認
backup-suite config get-destination
```

### 3. 設定確認
```bash
# バックアップ先ディレクトリの確認
backup-suite status
```

## 基本的な使用方法

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

# 増分バックアップ
backup-suite run --incremental      # 変更分のみバックアップ（2回目以降推奨）

# 圧縮オプション
backup-suite run --compress zstd   # Zstd圧縮（高速・高圧縮率・推奨）
backup-suite run --compress gzip   # Gzip圧縮（互換性重視）
backup-suite run --compress none   # 圧縮なし

# 暗号化バックアップ
backup-suite run --encrypt --password "secure-password"

# 圧縮+暗号化の組み合わせ
backup-suite run --compress zstd --encrypt --password "secure-password"
```

4. **自動化設定**
```bash
# 優先度別スケジュール設定
backup-suite schedule setup --high daily --medium weekly --low monthly
backup-suite schedule enable
```

## 設定ファイル

### ~/.config/backup-suite/config.toml の例
```toml
[general]
log_level = "info"
log_file = "~/.local/share/backup-suite/logs/backup.log"

[storage]
type = "local"
path = "/Users/john/Library/CloudStorage/GoogleDrive-john@example.com/マイドライブ/backup-storage"
compression = "zstd"  # 圧縮タイプ: "zstd", "gzip", "none"
compression_level = 3  # 圧縮レベル: 1-22（Zstd）, 1-9（Gzip）
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

## スケジューリング機能

### 自動バックアップの設定

```bash
# スケジュール頻度を設定
backup-suite schedule setup --high daily --medium weekly --low monthly

# スケジュールを有効化
backup-suite schedule enable

# 状態確認
backup-suite schedule status
```

### プラットフォーム別の動作

#### macOS (launchd)
- 設定ファイル: `~/Library/LaunchAgents/com.backup-suite.{priority}.plist`
- ログ: `/tmp/backup-suite-{priority}.log`
- 確認: `launchctl list | grep backup-suite`

#### Linux (systemd)
- 設定ファイル: `~/.config/systemd/user/backup-suite-{priority}.{service,timer}`
- ログ: `journalctl --user -u backup-suite-{priority}.service`
- 確認: `systemctl --user list-timers backup-suite-*`

詳細は[スケジューリングガイド](docs/SCHEDULER.md)を参照してください。

## コマンドリファレンス

| コマンド       | 説明                           | 例                                              |
| -------------- | ------------------------------ | ----------------------------------------------- |
| **add**        | バックアップ対象追加           | `backup-suite add ~/docs --priority high`       |
| **list, ls**   | 対象一覧表示                   | `backup-suite list --priority medium`           |
| **remove**     | 対象削除                       | `backup-suite remove ~/old-files`               |
| **clear, rm**  | 一括削除                       | `backup-suite clear --priority low`             |
| **run**        | バックアップ実行               | `backup-suite run --encrypt`                    |
| **restore**    | バックアップ復元               | `backup-suite restore --from backup-20251104`   |
| **cleanup**    | 古いバックアップ削除           | `backup-suite cleanup --days 30`                |
| **status**     | 現在の状態表示                 | `backup-suite status`                           |
| **history**    | 実行履歴表示                   | `backup-suite history --days 7`                 |
| **schedule**   | スケジューリング管理           | `backup-suite schedule enable`                  |
| **config**     | 設定管理                       | `backup-suite config set-destination ~/backups` |
| **open**       | バックアップディレクトリを開く | `backup-suite open`                             |
| **completion** | シェル補完生成                 | `backup-suite completion zsh`                   |

## アップデート・アンインストール

### アップデート

```bash
# Homebrew
brew upgrade backup-suite

# Cargo
cargo install backup-suite --force

# ソースから
cd backup-suite
git pull origin main
cargo install --path . --force
```

### アンインストール

```bash
# 1. バイナリを削除
rm ~/.local/bin/backup-suite

# 2. 設定ファイル削除（オプション）
rm -rf ~/.config/backup-suite/

# 3. ログファイル削除（オプション）
rm -rf ~/.local/share/backup-suite/
```

## セキュリティ・品質

### **企業級セキュリティ**
- AES-256-GCM暗号化対応
- 安全なパスワードベース鍵導出（Argon2）
- ローカル専用（クラウド非依存）
- 設定ファイルの適切な権限管理

### **型安全性・メモリ安全性**
- Rustの強力な型システムで実行時エラーを最小化
- メモリ安全性保証（バッファオーバーフロー、メモリリーク防止）
- コンパイル時エラー検出

## 技術スタック

- **言語**: Rust（最新安定版）
- **CLI**: clap 4.x （コマンドライン解析・補完生成）
- **圧縮**: Zstd（高速・高圧縮率）、Gzip（互換性）
- **暗号化**: AES-256-GCM、Argon2
- **設定**: TOML （人間にとって読みやすい設定形式）
- **スケジューリング**: macOS launchctl、Linux systemd

## 対応プラットフォーム

| OS      | アーキテクチャ | 対応状況   |
| ------- | -------------- | ---------- |
| 🐧 Linux | x86_64         | ✅ 完全対応 |
| 🐧 Linux | aarch64        | ✅ 完全対応 |
| 🍎 macOS | x86_64         | ✅ 完全対応 |
| 🍎 macOS | Apple Silicon  | ✅ 完全対応 |

## ライセンス

このプロジェクトは[MITライセンス](LICENSE)の下で公開されています。

---

## コントリビューション

バグレポート・機能要望・プルリクエストを歓迎します。
GitHubのIssue・PRからお気軽にご連絡ください。
