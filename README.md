# backup-suite

[日本語](README.md) | [English](README.en.md) 

> **M3社内向バックアップソリューション**

## 主要機能

### 🎯 優先度別バックアップ管理
- **重要な仕事ファイル**は毎日自動バックアップ
- **写真や個人ファイル**は週次バックアップ
- **アーカイブファイル**は月次バックアップ

### 🔐 軍事レベルの暗号化保護
- **AES-256-GCM暗号化**で解読は事実上不可能
- **パソコン盗難時**でもデータは完全に安全
- **クラウド保存時**も第三者は絶対に見れない
- **パスワード**がないと誰も開けません

### ⏰ 完全自動化されたスケジューリング
- **設定後は手動操作不要**で自動実行
- **重要度別に頻度を調整**（毎日・週次・月次）
- **バックアップ忘れ**を完全に防止

### 📊 わかりやすい管理とメンテナンス
- **どれくらいバックアップしたか**統計で確認
- **いつ実行されたか**履歴で確認
- **古いバックアップ**を自動削除してディスク節約
- **データが壊れた時**の簡単復元

## スクリーンショット

### ヘルプ画面
![backup-suite help](docs/screenshots/help.webp)
*コマンド一覧とオプションを日本語で表示*

### バックアップ対象一覧
![backup-suite list](docs/screenshots/list.webp)
*登録されたバックアップ対象をテーブル形式で表示*

### バックアップ実行
![backup-suite run](docs/screenshots/run.webp)
*実際のバックアップ実行画面*

### バックアップ実行（ドライラン）
![backup-suite dry-run](docs/screenshots/dry-run.webp)
*実際にファイルをコピーせずに実行内容を確認*

### バックアップ履歴
![backup-suite history](docs/screenshots/history.webp)
*過去のバックアップ実行履歴を確認*

## インストール

### 方法1: ビルド済みバイナリ（推奨）

最も簡単で高速な方法です。Rustのインストール不要。

#### 自動インストールスクリプト（推奨）

```bash
# 最新版を自動インストール
curl -sSL https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/raw/main/install.sh | bash
```

#### 手動インストール

```bash
# 1. バイナリをダウンロード
curl -LO "https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/jobs/artifacts/v1.0.0/raw/backup-suite-
  complete-package.tar.gz?job=package:create-distributions"

# 2. 解凍
tar -xzf backup-suite-*.tar.gz

# 3. バイナリを適切な場所に移動
sudo mv backup-suite /usr/local/bin/

# 4. 動作確認
backup-suite --version
```

または、[リリースページ](https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/releases/v1.0.0)から直接ダウンロード。

### 方法2: ソースからビルド

#### システム要件

- **オペレーティングシステム**: Linux、macOS
- **Rust**: 1.75以上（MSRV）

#### Rustツールチェーンのインストール

```bash
# 1. Rustup（Rustインストーラー）をダウンロード・実行
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 環境変数を読み込み
source ~/.cargo/env

# 3. インストール確認
rustc --version
cargo --version
```

#### backup-suiteのビルド&インストール

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

# 暗号化バックアップ
backup-suite run --encrypt --password "secure-password"
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
# 1. 最新ソースを取得
cd backup-suite  # プロジェクトディレクトリ
git pull

# 2. 再ビルド&インストール
cargo install --path . --force

# 3. バージョン確認
backup-suite --version
```

### アンインストール

```bash
# 1. backup-suiteを削除
cargo uninstall backup-suite

# 2. 設定ファイル削除（オプション）
rm -rf ~/.config/backup-suite/

# 3. ログファイル削除（オプション）
rm -rf ~/.local/share/backup-suite/

# 4. プロジェクトディレクトリ削除（オプション）
rm -rf ~/backup-suite  # クローンしたディレクトリ
```

## よくある問題と解決策

**問題1**: `rustc` または `cargo` コマンドが見つからない
```bash
# 解決策: Rustツールチェーンをインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 確認
rustc --version
cargo --version
```

**問題2**: `git clone` でアクセス拒否エラー
```bash
# 解決策: SSH鍵設定またはHTTPS認証
# SSH鍵が設定済みの場合
git clone git@rendezvous.m3.com:sanae-abe/backup-suite.git

# または、GitLabにログインしてHTTPSでクローン
git clone https://rendezvous.m3.com/sanae-abe/backup-suite.git
```

**問題3**: コンパイルエラーが発生する
```bash
# 解決策: Rustを最新版に更新
rustup update stable
cargo clean  # キャッシュクリア
cargo build  # 再ビルド
```

**問題4**: `backup-suite` コマンドが見つからない
```bash
# 解決策: PATHの確認と追加
echo $PATH | grep -q "$HOME/.cargo/bin" || echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# 確認
which backup-suite
backup-suite --version
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

---
**開発者**: sanae-abe@m3.com
