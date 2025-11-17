# backup-suite

[![Crates.io](https://img.shields.io/crates/v/backup-suite.svg)](https://crates.io/crates/backup-suite)
[![Documentation](https://docs.rs/backup-suite/badge.svg)](https://docs.rs/backup-suite)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.82%2B-blue.svg)](https://www.rust-lang.org)
[![CI](https://github.com/sanae-abe/backup-suite/workflows/CI/badge.svg)](https://github.com/sanae-abe/backup-suite/actions)

[日本語](README.ja.md) | [English](README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md)

> **高速・安全・インテリジェントなローカルバックアップツール**

## 目次

- [主要機能](#主要機能)
- [スクリーンショット](#スクリーンショット)
- [インストール](#インストール)
- [クイックスタート](#クイックスタート)
- [基本的な使用方法](#基本的な使用方法)
- [Smart機能（インテリジェントバックアップ）](#-smart機能インテリジェントバックアップ)
- [設定ファイル](#設定ファイル)
- [スケジューリング機能](#スケジューリング機能)
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

### 🤖 Smart駆動のインテリジェント管理
- **自動最適化**: ディレクトリ分析による最適なバックアップ設定の自動生成
- **ファイル重要度分析**: ディレクトリ内のファイルを重要度別に自動分類（~8秒/10,000ファイル）
- **除外パターン推奨**: 不要ファイル（キャッシュ、ビルド成果物）を自動検出・除外提案
- **異常検知**: 統計的分析でバックアップサイズ異常を自動検知（< 1ms）
- **完全オフライン**: すべてのSmart機能はローカルで動作、プライバシー完全保護

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
- **ディスク容量を最大70%削減**（テキストファイルの典型的なケース）

### ⚡ 増分バックアップで超高速化
- **変更ファイルのみバックアップ**で時間を大幅短縮
- **SHA-256ハッシュ**による正確な変更検出
- **バックアップ時間を90%削減**（2回目以降、変更率10%の場合）
- **ストレージ容量を85%削減**（差分のみ保存、典型的なケース）
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

### 🌍 多言語対応
- **4言語完全対応**：日本語、英語、簡体中文（中国大陸）、繁體中文（台湾・香港）
- **自動言語検出**：`LANG`環境変数から自動判定（`ja`, `en`, `zh-CN`, `zh-TW`等に対応）
- **全メッセージ翻訳済み**：CLI出力、エラーメッセージ、ヘルプ全てを各言語で表示

### 💡 使いやすいCLI
- **タイポ修正サジェスト**：コマンド名のスペルミスを自動検出し、正しいコマンドを提案
  ```bash
  $ backup-suite restor
  error: unrecognized subcommand 'restor'

  Did you mean 'restore'?

  For more information, try '--help'.
  ```
- **インテリジェントな編集距離アルゴリズム**：Levenshtein距離で類似コマンドを自動判定（最大2文字の差まで検出）
- **カラー対応**：ターミナルのカラーサポートに応じて自動調整

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
# Smart機能を有効化してインストール（推奨）
cargo install backup-suite --features smart

# Smart機能なしでインストール（軽量版）
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

# 3. ビルド＆インストール（Smart機能あり）
cargo build --release --features smart
cargo install --path . --features smart

# 4. 動作確認
backup-suite --version
```

### 🌍 Zsh補完の多言語対応

Zsh補完説明は4言語に対応しています：

- 🇬🇧 **English** (en) - デフォルト
- 🇯🇵 **日本語** (ja) - Japanese
- 🇨🇳 **简体中文** (zh-CN) - Simplified Chinese
- 🇹🇼 **繁體中文** (zh-TW) - Traditional Chinese

#### 自動言語検出

システムのロケール設定（`$LANG`環境変数）から自動で適切な言語が選択されます：

```bash
# システムロケールに基づいて自動生成
backup-suite completion zsh > ~/.zfunc/_backup-suite
```

#### 手動で言語を指定

特定の言語の補完を生成したい場合：

```bash
# 日本語
./scripts/generate-completion.sh ja

# 简体中文
./scripts/generate-completion.sh zh-CN

# 繁體中文
./scripts/generate-completion.sh zh-TW

# English
./scripts/generate-completion.sh en
```

**初回セットアップ**（Zsh補完を有効化）:

```bash
# 1. 補完ディレクトリを作成
mkdir -p ~/.zfunc

# 2. .zshrcに以下を追加
echo 'fpath=(~/.zfunc $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc

# 3. 補完スクリプトを生成
backup-suite completion zsh > ~/.zfunc/_backup-suite
# または多言語対応スクリプトを使用
./scripts/generate-completion.sh ja

# 4. 新しいシェルを起動
exec zsh
```

補完が有効になると、`backup-suite <TAB>`でコマンドと説明が選択した言語で表示されます。

**補完が動作しない場合**:

詳細なトラブルシューティング手順は [docs/shell-completion.md](docs/shell-completion.md) をご覧ください。以下は主な対処方法です：

- **補完が全く動作しない**: シェルを再起動 (`exec zsh`)、ファイルの存在確認 (`ls -la ~/.zfunc/_backup-suite`)
- **間違った言語で表示される**: `echo $LANG` で環境変数を確認、または `./scripts/generate-completion.sh ja` で手動指定
- **compinit警告が出る**: ディレクトリ権限を修正 (`chmod go-w ~/.zfunc`)

Bash/Fishの補完インストール手順、その他の詳細は [docs/shell-completion.md](docs/shell-completion.md) を参照してください。

## クイックスタート

### 1. 基本セットアップ
```bash
# 現在の設定確認
backup-suite status

# 設定ファイルの場所
# ~/.config/backup-suite/config.toml
```

**注意**: 言語は環境変数`LANG`で自動検出されます。対応言語：日本語、英語、簡体中文、繁體中文。日本語環境では自動的に日本語で表示されます。

### 2. バックアップ保存先の設定

```bash
# Google Driveの保存先を設定
backup-suite config set-destination "/Users/あなたのユーザー名/Library/CloudStorage/GoogleDrive-your@email.com/マイドライブ/backup-storage"

# ⚠️ 重要: クラウドストレージへのバックアップは必ず暗号化を有効にしてください
# Google Drive等のクラウドストレージにバックアップを保存する場合、
# 第三者による不正アクセスを防ぐため、必ず --encrypt オプションを使用してください

# 現在の設定を確認
backup-suite config get-destination
```

### 3. 設定確認

設定を確認するには、[1. 基本セットアップ](#1-基本セットアップ)の`backup-suite status`コマンドを使用してください。

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

3. **対象の設定更新**
```bash
# カテゴリのみ変更
backup-suite update ~/.config --category "設定ファイル"

# 優先度とカテゴリを同時に変更
backup-suite update ~/.ssh --priority high --category "SSH設定（秘密鍵含む）"

# 除外パターンを追加
backup-suite update ~/.ssh --exclude "known_hosts*" --exclude "*.old"

# 複数の設定を同時に更新
backup-suite update ~/Documents --priority high --category "重要書類" --exclude "*.tmp"
```

4. **バックアップ実行**
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

# 暗号化バックアップ（推奨: 対話的パスワード入力）
backup-suite run --encrypt
# → パスワードプロンプトで安全に入力（シェル履歴に残らない）

# または環境変数を使用（オプション）
export BACKUP_SUITE_PASSWORD="your-secure-password"
backup-suite run --encrypt

# 圧縮+暗号化の組み合わせ
backup-suite run --compress zstd --encrypt
# → パスワードプロンプトで対話的に入力
```

5. **自動化設定**
```bash
# 優先度別スケジュール設定
backup-suite schedule setup --high daily --medium weekly --low monthly
backup-suite schedule enable
```

## 🤖 Smart機能（インテリジェントバックアップ）

統計的異常検知・ファイル重要度分析でバックアップを最適化します。

### インストール

Smart機能を使用するには、`--features smart`フラグを付けてビルドする必要があります。

```bash
# Smart機能を有効化してビルド
cargo build --release --features smart
cargo install --path . --features smart

# または Cargo経由でインストール
cargo install backup-suite --features smart
```

### 主要機能

#### 1. Smart自動設定

ディレクトリを分析し、最適なバックアップ設定を自動生成します。

```bash
# 自動分析・設定（サブディレクトリを個別に評価）
backup-suite smart auto-configure ~/data

# 対話的に確認しながら設定（サブディレクトリと除外パターンを確認）
backup-suite smart auto-configure ~/data --interactive

# ドライラン（設定を適用せず確認のみ）
backup-suite smart auto-configure ~/data --dry-run

# サブディレクトリの探索深度を指定（デフォルト: 1）
backup-suite smart auto-configure ~/data --max-depth 2

# 処理するサブディレクトリの最大数を指定（デフォルト: 100）
backup-suite smart auto-configure ~/data --max-subdirs 50

# 大量のサブディレクトリがある場合の処理数上限を増やす
backup-suite smart auto-configure ~/data --max-subdirs 200
```

**機能**:
- **サブディレクトリごとに重要度を個別評価**（各ディレクトリに最適な優先度を設定）
- **除外パターンの自動検出・適用**（`node_modules/`, `target/`, `.cache/` 等を自動除外）
- **プロジェクトタイプの自動判定**（Rust, Node.js, Python 等）
- **信頼度80%以上のパターンのみ適用**（誤検出を防止）
- **処理数制限による性能最適化**（`--max-subdirs`で大量のサブディレクトリ対策、デフォルト: 100）

**出力例**:
```
🤖 Smart自動設定
分析中: "/Users/user/projects"
  📁 3個のサブディレクトリを発見: 3
    評価中: "/Users/user/projects/web-app"
      推奨優先度: High (スコア: 95)
      📋 除外パターン提案: 3
        - node_modules (99.0%, 2.34 GB 削減見込)
        - .cache (95.0%, 0.45 GB 削減見込)
        - .*\.tmp$ (99.0%, 0.00 GB 削減見込)
      📝 除外パターン: node_modules, .cache, .*\.tmp$
      ✅ 設定に追加しました
    評価中: "/Users/user/projects/rust-cli"
      推奨優先度: High (スコア: 95)
      📋 除外パターン提案: 2
        - target (99.0%, 1.87 GB 削減見込)
        - .cache (95.0%, 0.12 GB 削減見込)
      📝 除外パターン: target, .cache
      ✅ 設定に追加しました
    評価中: "/Users/user/projects/archive"
      推奨優先度: Low (スコア: 30)
      ✅ 設定に追加しました

自動設定が完了しました
  追加された項目: 3
  総削減量: 4.78 GB（バックアップ時間を約35%短縮）
```

**サブディレクトリ数制限時の例**:
```
🤖 Smart自動設定
分析中: "/Users/user/large-project"
  📁 100個のサブディレクトリを発見: 100
  ⚠️  制限に達したため、一部のサブディレクトリは処理されませんでした: 100 (--max-subdirs で変更可能)
```

#### 2. ファイル重要度分析

ディレクトリ内のファイルを重要度別に分類し、バックアップ戦略を最適化します。

```bash
# ディレクトリの重要度分析
backup-suite smart analyze ~/documents

# 詳細な重要度スコア表示
backup-suite smart analyze ~/documents --detailed

# 特定のファイル種別のみ分析
backup-suite smart analyze ~/projects --filter "*.rs,*.toml"
```

**評価基準**:
- **高重要度（80-100点）**: ソースコード、ドキュメント、設定ファイル
- **中重要度（40-79点）**: 画像、データファイル
- **低重要度（0-39点）**: ログ、一時ファイル

**出力例**:
```
🤖 Smartファイル重要度分析: ~/Documents

┌─────────────────────────┬──────────────┬──────────┬─────────────────────┐
│ ファイル/ディレクトリ     │ 重要度スコア   │ 提案優先度 │ 理由                 │
├─────────────────────────┼──────────────┼──────────┼─────────────────────┤
│ src/                    │ ████████ 95  │ 高        │ ソースコード（頻繁更新）│
│ reports/                │ ████████ 90  │ 高        │ ドキュメント（重要）  │
│ photos/                 │ ████░░░░ 60  │ 中        │ 画像ファイル          │
│ .cache/                 │ █░░░░░░░ 10  │ 除外推奨  │ キャッシュディレクトリ │
└─────────────────────────┴──────────────┴──────────┴─────────────────────┘
```

**パフォーマンス**: ~8秒（10,000ファイル）

#### 3. 除外パターン推奨

不要なファイルを自動検出し、除外パターンを推奨します。

```bash
# 除外パターンの推奨を表示
backup-suite smart suggest-exclude ~/projects

# 推奨パターンを自動的に設定ファイルに適用
backup-suite smart suggest-exclude ~/projects --apply

# 最小ファイルサイズを指定（デフォルト: 100MB）
backup-suite smart suggest-exclude ~/projects --min-size 50MB
```

**検出対象**:
- ビルド成果物（`target/`, `dist/`, `build/`）
- 依存関係キャッシュ（`node_modules/`, `.cargo/`）
- 一時ファイル（`*.tmp`, `*.cache`）
- 大容量メディアファイル（閾値以上のサイズ）

**出力例**:
```
🤖 Smart除外パターン推奨: ~/projects

┌──────────────────┬──────────┬──────────┬─────────────────────┐
│ パターン          │ 削減量    │ 信頼度    │ 理由                 │
├──────────────────┼──────────┼──────────┼─────────────────────┤
│ node_modules/    │ 2.34 GB  │ 99%      │ npm依存関係（再生成可能）│
│ target/          │ 1.87 GB  │ 99%      │ Rustビルド成果物      │
│ .cache/          │ 0.45 GB  │ 95%      │ キャッシュディレクトリ │
└──────────────────┴──────────┴──────────┴─────────────────────┘

💡 総削減量: 4.66 GB（バックアップ時間を約30%短縮）
```

#### 4. 異常検知

過去の履歴から統計的に異常なバックアップを検知します。

```bash
# 過去7日間の異常検知
backup-suite smart detect --days 7

# より詳細な分析（統計情報も表示）
backup-suite smart detect --days 14 --detailed
```

**検知内容**:
- バックアップサイズの急増/急減（Z-score統計分析）
- ディスク容量枯渇予測（線形回帰）
- 失敗パターンの分析（カテゴリ別・時刻別）

**出力例**:
```
🤖 Smart異常検知レポート（過去7日間）

┌────┬──────────────────┬──────────┬──────────┬─────────────────────┐
│ No │ 検出日時          │ 異常種別  │ 信頼度    │ 説明                 │
├────┼──────────────────┼──────────┼──────────┼─────────────────────┤
│ 1  │ 2025-11-09 03:15 │ サイズ急増│ 95.3%    │ ファイルサイズが通常の3倍 │
└────┴──────────────────┴──────────┴──────────┴─────────────────────┘

📊 サマリー: 1件の異常を検出
💡 推奨アクション: ~/Downloads の一時ファイルを除外設定に追加
```

**パフォーマンス**: < 1ms（100件履歴）

### Smart機能の無効化

Smart機能が不要な場合は、通常のビルドを使用してください。

```bash
# 通常ビルド（Smart機能なし）
cargo build --release
cargo install --path .
```

### セキュリティとプライバシー

すべてのSmart機能は**完全にオフライン**で動作します：

- ✅ 外部APIコール: なし
- ✅ クラウドサービス: 不要
- ✅ 機密情報の送信: ゼロ
- ✅ データ収集: なし

詳細は [Smart機能ドキュメント](docs/smart/features.md) を参照してください。

## 設定ファイル

### ~/.config/backup-suite/config.toml の例
```toml
[general]
log_level = "info"
log_file = "~/.local/share/backup-suite/logs/backup.log"

[storage]
type = "local"
path = "/Users/john/Library/CloudStorage/GoogleDrive-john@example.com/マイドライブ/backup-storage"  # クラウドストレージ使用時は encryption = true 必須
compression = "zstd"  # 圧縮タイプ: "zstd", "gzip", "none"
compression_level = 3  # 圧縮レベル: 1-22（Zstd）, 1-9（Gzip）
encryption = true
encryption_key_file = "~/.config/backup-suite/keys/backup.key"  # 重要: chmod 600で保護

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
| **update**     | 対象の設定更新                 | `backup-suite update ~/.ssh --priority high --category "SSH設定"` |
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
| **smart**         | Smart機能（要`--features smart`）    | `backup-suite smart detect --days 7`               |

## アップデート・アンインストール

### アップデート

```bash
# Homebrew
brew upgrade backup-suite

# Cargo
cargo install backup-suite --force --features smart

# ソースから
cd backup-suite
git pull origin main
cargo install --path . --force --features smart
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
- **暗号化**: AES-256-GCM（認証付き暗号化）
- **鍵導出**: Argon2id（メモリコスト19MB、反復2回）
- **Nonce衝突検出**: デバッグビルドで自動追跡（リリースビルドはゼロオーバーヘッド）
  - 全Nonce（暗号化初期化ベクトル）を追跡し、衝突を即座に検出
  - 衝突発生時は詳細なエラーメッセージでセキュリティ影響を報告
  - リリースビルドではコンパイル時に完全削除（パフォーマンス影響なし）
- **パストラバーサル対策**: Unicode正規化（NFKC）、Null byte検出、シンボリックリンク攻撃防止
- **機密データ消去**: Zeroize使用（メモリダンプ攻撃対策）
- **ローカル専用**: クラウド非依存で安全
- **権限管理**: 設定ファイルの適切な権限設定

### **型安全性・メモリ安全性**
- Rustの強力な型システムで実行時エラーを最小化
- メモリ安全性保証（バッファオーバーフロー、メモリリーク防止）
- コンパイル時エラー検出
- 機密データの安全な消去（ZeroizeOnDrop）

### **防御対象攻撃**
- ✅ **パストラバーサル攻撃**: Unicode正規化（NFKC）、Null byte検出、シンボリックリンク検証
- ✅ **ブルートフォース攻撃**: Argon2id高コスト鍵導出（メモリ19MB、反復2回）
- ✅ **メモリダンプ攻撃**: Zeroize による機密データ即座消去
- ✅ **中間者攻撃**: AES-256-GCM認証付き暗号化（改ざん検出）
- ✅ **タイミング攻撃**: 定数時間比較による鍵検証
- ✅ **インジェクション攻撃**: Rustの型システムによる安全なパス操作

## 技術スタック

- **言語**: Rust（最新安定版）
- **CLI**: clap 4.x （コマンドライン解析・補完生成）
- **圧縮**: Zstd（高速・高圧縮率）、Gzip（互換性）
- **暗号化**: AES-256-GCM、Argon2
- **設定**: TOML （人間にとって読みやすい設定形式）
- **スケジューリング**: macOS launchctl、Linux systemd
- **Smart/統計分析**: statrs（統計計算）、rayon（並列処理）

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
