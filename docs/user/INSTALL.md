# インストールガイド

Backup Suite v1.0.0の詳細なインストール手順とセットアップ方法を説明します。

## 📋 前提条件

### システム要件
- **macOS**: 10.15 (Catalina) 以降
- **Linux**: Ubuntu 18.04 LTS, CentOS 7, Debian 10 以降
- **CPU**: x86_64 または ARM64
- **メモリ**: 最小 512MB RAM
- **ディスク**: 10MB 空き容量（バイナリ用）

### 依存関係
- **macOS**: Xcode Command Line Tools（launchctl統合用）
- **Linux**: systemd（将来の自動化機能用）
- **オプション**: Git（ソースからビルドする場合）

## 🚀 インストール方法

### 方法1: バイナリダウンロード（推奨）

#### macOS Intel
```bash
# 最新リリースをダウンロード
curl -L https://github.com/user/backup-suite/releases/latest/download/backup-suite-v1.0.0-macos-x86_64.tar.gz | tar xz

# ~/.local/bin に配置（PATH設定済みの場合）
mkdir -p ~/.local/bin
mv backup-suite ~/.local/bin/
chmod +x ~/.local/bin/backup-suite
```

#### macOS Apple Silicon (M1/M2)
```bash
# Apple Silicon用バイナリをダウンロード
curl -L https://github.com/user/backup-suite/releases/latest/download/backup-suite-v1.0.0-macos-arm64.tar.gz | tar xz

# ~/.local/bin に配置
mkdir -p ~/.local/bin
mv backup-suite ~/.local/bin/
chmod +x ~/.local/bin/backup-suite
```

#### Linux x86_64
```bash
# Linux用バイナリをダウンロード
curl -L https://github.com/user/backup-suite/releases/latest/download/backup-suite-v1.0.0-linux-x86_64.tar.gz | tar xz

# /usr/local/bin に配置（システム全体）
sudo mv backup-suite /usr/local/bin/
sudo chmod +x /usr/local/bin/backup-suite

# または ~/.local/bin に配置（ユーザー専用）
mkdir -p ~/.local/bin
mv backup-suite ~/.local/bin/
chmod +x ~/.local/bin/backup-suite
```

### 方法2: Homebrew（macOS推奨）

```bash
# Homebrewタップを追加
brew tap user/backup-suite

# インストール
brew install backup-suite

# アップデート
brew upgrade backup-suite

# アンインストール
brew uninstall backup-suite
```

### 方法3: Cargo（Rust）

```bash
# Rust環境が必要（rustup推奨）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Cargoでインストール
cargo install backup-suite

# アップデート
cargo install backup-suite --force

# アンインストール
cargo uninstall backup-suite
```

### 方法4: ソースからビルド

```bash
# リポジトリをクローン
git clone https://github.com/user/backup-suite.git
cd backup-suite

# 依存関係確認
cargo --version  # Rust 1.70+ 必要

# リリースビルド
cargo build --release

# バイナリをコピー
cp target/release/backup-suite ~/.local/bin/
chmod +x ~/.local/bin/backup-suite

# テスト実行
cargo test
```

## 🔧 初期設定

### PATH設定確認
```bash
# PATHに ~/.local/bin が含まれているか確認
echo $PATH | grep -q "$HOME/.local/bin" && echo "✅ PATH設定済み" || echo "❌ PATH設定が必要"

# PATH設定（必要な場合）
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc  # zsh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc # bash
source ~/.zshrc  # または ~/.bashrc
```

### インストール確認
```bash
# バージョン確認
backup-suite --version

# ヘルプ表示
backup-suite --help

# 基本ステータス確認
backup-suite status

# 言語設定（デフォルト: 英語）
backup-suite --lang en --help  # 英語でヘルプ表示
backup-suite --lang ja --help  # 日本語でヘルプ表示
```

### シェル補完設定

#### Zsh
```bash
# 補完ディレクトリ作成
mkdir -p ~/.local/share/zsh/site-functions

# 補完スクリプト生成
backup-suite completion zsh > ~/.local/share/zsh/site-functions/_backup-suite

# .zshrc に追加（まだ設定していない場合）
echo 'fpath=(~/.local/share/zsh/site-functions $fpath)' >> ~/.zshrc
echo 'autoload -U compinit && compinit' >> ~/.zshrc

# 再読み込み
source ~/.zshrc
```

#### Bash
```bash
# 補完ディレクトリ作成
mkdir -p ~/.local/share/bash-completion/completions

# 補完スクリプト生成
backup-suite completion bash > ~/.local/share/bash-completion/completions/backup-suite

# .bashrc に追加（必要な場合）
echo 'source ~/.local/share/bash-completion/completions/backup-suite' >> ~/.bashrc

# 再読み込み
source ~/.bashrc
```

#### Fish
```bash
# Fish設定ディレクトリ作成
mkdir -p ~/.config/fish/completions

# 補完スクリプト生成
backup-suite completion fish > ~/.config/fish/completions/backup-suite.fish
```

### 設定ファイル初期化

```bash
# 設定ディレクトリ作成
mkdir -p ~/.config/backup-suite

# 初期設定ファイル生成（自動）
backup-suite status  # 初回実行で設定ファイルが作成される

# 設定ファイル確認
cat ~/.config/backup-suite/config.toml
```

### バックアップディレクトリ設定

```bash
# デフォルトバックアップディレクトリ作成
mkdir -p ~/backup-suite/backups

# カスタムディレクトリを使用する場合
mkdir -p /path/to/custom/backup/location

# 設定ファイルで変更（手動編集）
# ~/.config/backup-suite/config.toml の [backup] セクション:
# destination = "/path/to/custom/backup/location"

# または config コマンドで変更（推奨）
backup-suite config set-destination /path/to/custom/backup/location
backup-suite config get-destination  # 確認
```

### 高度な機能設定

#### 暗号化・圧縮機能（v1.0.0+）

```bash
# AES-256-GCM暗号化バックアップ
backup-suite run --encrypt --password "your-password"

# zstd圧縮バックアップ（デフォルト）
backup-suite run --compress zstd --compress-level 3

# gzip圧縮バックアップ（互換性重視）
backup-suite run --compress gzip --compress-level 6

# 暗号化＋圧縮（推奨）
backup-suite run --encrypt --compress zstd

# 暗号化バックアップの復元
backup-suite restore --password "your-password"
```

#### 設定管理コマンド（v1.0.0+）

```bash
# バックアップ保存先の変更
backup-suite config set-destination ~/my-backups
backup-suite config get-destination

# バックアップ保持期間の変更
backup-suite config set-keep-days 60
backup-suite config get-keep-days

# 設定ファイルをエディタで開く
backup-suite config open
```

## 🔄 自動スケジューリング設定（macOS）

### launchctl統合設定

```bash
# スケジュール頻度設定
backup-suite schedule setup --high daily --medium weekly --low monthly

# 自動バックアップ有効化
backup-suite schedule enable

# 設定確認
backup-suite schedule status

# 特定優先度のみ有効化
backup-suite schedule enable --priority high
```

### 手動plist確認（上級者向け）

```bash
# 生成されたplistファイル確認
ls ~/Library/LaunchAgents/com.backup-suite.*.plist

# plistファイル内容確認
cat ~/Library/LaunchAgents/com.backup-suite.high.plist

# launchctl状態確認
launchctl list | grep backup-suite
```

## 🛠️ インストール後の確認

### 基本動作テスト

```bash
# 1. テストファイル作成
mkdir -p ~/test-backup
echo "test content" > ~/test-backup/test.txt

# 2. バックアップ対象追加
backup-suite add ~/test-backup --priority high --category test

# 3. ドライラン実行
backup-suite run --dry-run

# 4. 実際のバックアップ実行
backup-suite run --priority high

# 5. 結果確認
backup-suite history --days 1
backup-suite dashboard

# 6. クリーンアップ
backup-suite remove ~/test-backup
rm -rf ~/test-backup
```

### インタラクティブ機能テスト

```bash
# skim統合ファイル選択テスト
backup-suite add --interactive

# 既存対象からの選択削除テスト
backup-suite remove --interactive
```

### パフォーマンステスト

```bash
# 大量ファイルでのテスト
mkdir -p ~/performance-test
for i in {1..100}; do
    dd if=/dev/urandom of=~/performance-test/file$i.dat bs=1M count=1 2>/dev/null
done

# パフォーマンス測定
time backup-suite add ~/performance-test --priority medium --category test
time backup-suite run --priority medium --dry-run

# クリーンアップ
backup-suite remove ~/performance-test
rm -rf ~/performance-test
```

## 🔍 トラブルシューティング

### よくある問題

#### 1. "command not found: backup-suite"
```bash
# PATH確認
echo $PATH

# ファイル存在確認
ls -la ~/.local/bin/backup-suite

# 実行権限確認
chmod +x ~/.local/bin/backup-suite

# シェル再起動
source ~/.zshrc  # または ~/.bashrc
```

#### 2. 権限エラー
```bash
# 設定ディレクトリ権限確認
ls -la ~/.config/backup-suite/

# 権限修正
chmod 755 ~/.config/backup-suite/
chmod 644 ~/.config/backup-suite/config.toml
```

#### 3. launchctl エラー（macOS）
```bash
# launchctl リスト確認
launchctl list | grep backup-suite

# 手動でplist削除（必要な場合）
launchctl unload ~/Library/LaunchAgents/com.backup-suite.*.plist
rm ~/Library/LaunchAgents/com.backup-suite.*.plist

# 再設定
backup-suite schedule setup
backup-suite schedule enable
```

#### 4. skim選択が表示されない
```bash
# find コマンド確認
which find

# skim テスト
echo -e "file1\nfile2\nfile3" | fzf  # fzfがある場合

# 手動パス指定で回避
backup-suite add /absolute/path/to/file --priority high
```

### ログ確認

```bash
# 実行ログ確認（macOS）
tail -f /tmp/backup-suite-high.log
tail -f /tmp/backup-suite-high.error.log

# 設定ファイル検証
backup-suite status

# 詳細デバッグ（開発版）
RUST_LOG=debug backup-suite status
```

### 完全アンインストール

```bash
# 1. バイナリ削除
rm ~/.local/bin/backup-suite
# または
sudo rm /usr/local/bin/backup-suite

# 2. 設定ファイル削除
rm -rf ~/.config/backup-suite/

# 3. launchctl削除（macOS）
backup-suite schedule disable  # 削除前に実行
rm ~/Library/LaunchAgents/com.backup-suite.*.plist

# 4. 補完スクリプト削除
rm ~/.local/share/zsh/site-functions/_backup-suite
rm ~/.local/share/bash-completion/completions/backup-suite
rm ~/.config/fish/completions/backup-suite.fish

# 5. Homebrew削除（該当する場合）
brew uninstall backup-suite
brew untap user/backup-suite

# 6. Cargo削除（該当する場合）
cargo uninstall backup-suite
```

## 📞 サポート

インストールで問題が発生した場合：

1. **GitHub Issues**: [問題報告](https://github.com/user/backup-suite/issues)
2. **Discussions**: [質問・相談](https://github.com/user/backup-suite/discussions)
3. **Email**: support@backup-suite.example.com

---

**次のステップ**: インストール完了後は [USAGE.md](USAGE.md) で詳細な使用方法を確認してください。