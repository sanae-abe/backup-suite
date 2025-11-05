# Backup Suite

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-1.0.0-green.svg)](https://github.com/user/backup-suite/releases)

**🦀 高速・型安全・インテリジェントなローカルバックアップツール**

Backup Suite は**Rust製**の高性能CLIツールです。優先度別管理・自動スケジューリング・インタラクティブファイル選択により、効率的なバックアップワークフローを実現します。

## ✨ 主要機能

### 🎯 **優先度別バックアップ管理**
```bash
backup-suite add ~/important-docs --priority high --category work
backup-suite add ~/photos --priority medium --category personal
backup-suite run --priority high  # 高優先度のみ実行
```

### 🎨 **インタラクティブファイル選択（skim統合）**
```bash
backup-suite add --interactive     # 美しいUIでファイル選択
backup-suite remove --interactive  # 既存対象から選択削除
```

### ⏰ **自動スケジューリング（macOS launchctl完全統合）**
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

## 🚀 クイックスタート

### インストール

#### 方法1: バイナリダウンロード（推奨）
```bash
# 最新リリースをダウンロード
curl -L https://github.com/user/backup-suite/releases/latest/download/backup-suite-macos-x86_64.tar.gz | tar xz

# ~/.local/bin に配置
mv backup-suite ~/.local/bin/
chmod +x ~/.local/bin/backup-suite
```

#### 方法2: Cargo（Rust）
```bash
cargo install backup-suite
```

#### 方法3: ソースからビルド
```bash
git clone https://github.com/user/backup-suite.git
cd backup-suite
cargo build --release
cp target/release/backup-suite ~/.local/bin/
```

### 初期設定
```bash
# シェル補完設定（zsh）
backup-suite completion zsh > ~/.local/share/zsh/site-functions/_backup-suite

# 基本設定確認
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
```

4. **自動化設定**
```bash
# 優先度別スケジュール設定
backup-suite schedule setup --high daily --medium weekly --low monthly
backup-suite schedule enable
```

## 🏗️ アーキテクチャ

### **設定ファイル**: `~/.config/backup-suite/config.toml`
```toml
[backup]
destination = "/Users/user/backup-suite/backups"
keep_days = 30

[[targets]]
path = "/Users/user/Documents/projects"
priority = "high"
category = "development"
```

### **技術スタック**
- **言語**: Rust 1.70+ （型安全・メモリ安全・高性能）
- **CLI**: clap 4.x （コマンドライン解析・補完生成）
- **UI**: skim （美しいファジーファインダー統合）
- **設定**: TOML （人間にとって読みやすい設定形式）
- **スケジューリング**: macOS launchctl （システムレベル自動化）

## 📋 全コマンドリファレンス

| コマンド | 説明 | 例 |
|----------|------|-----|
| **add** | バックアップ対象追加 | `backup-suite add ~/docs --priority high` |
| **list, ls** | 対象一覧表示 | `backup-suite list --priority medium` |
| **remove** | 対象削除 | `backup-suite remove ~/old-files` |
| **clear, rm** | 一括削除 | `backup-suite clear --priority low` |
| **run** | バックアップ実行 | `backup-suite run --dry-run` |
| **restore** | バックアップ復元 | `backup-suite restore --from backup-20251104` |
| **cleanup** | 古いバックアップ削除 | `backup-suite cleanup --days 30` |
| **status** | 現在の状態表示 | `backup-suite status` |
| **history** | 実行履歴表示 | `backup-suite history --days 7` |
| **dashboard** | 統計ダッシュボード | `backup-suite dashboard` |
| **schedule** | スケジューリング管理 | `backup-suite schedule enable --priority high` |
| **open** | バックアップディレクトリを開く | `backup-suite open` |
| **--version** | バージョン情報 | `backup-suite --version` |
| **completion** | シェル補完生成 | `backup-suite completion zsh` |

## 🔧 高度な使用方法

### インタラクティブワークフロー
```bash
# ファイル選択UIで対象追加
backup-suite add --interactive

# 既存対象から選択削除
backup-suite remove --interactive

# 確認しながらクリーンアップ
backup-suite cleanup --days 30 --dry-run
```

### 優先度別運用戦略
```bash
# 重要ファイル: 毎日バックアップ
backup-suite add ~/critical-data --priority high --category critical

# 通常ファイル: 週次バックアップ
backup-suite add ~/documents --priority medium --category work

# アーカイブ: 月次バックアップ
backup-suite add ~/old-projects --priority low --category archive
```

### 復元・災害復旧
```bash
# 最新バックアップから復元
backup-suite restore

# 特定日付から復元
backup-suite restore --from backup-20251104 --to ~/recovered-files

# 復元前に内容確認
backup-suite history
```

## 🛡️ セキュリティ・品質

### **型安全性**
- Rustの強力な型システムで実行時エラーを最小化
- メモリ安全性保証（バッファオーバーフロー、メモリリーク防止）
- コンパイル時エラー検出

### **データ保護**
- ローカル専用（クラウド非依存）
- 設定ファイルの適切な権限管理
- バックアップ実行前の検証

### **テスト・品質保証**
```bash
# プロジェクトで品質確認
cargo test                        # 単体テスト
cargo clippy                      # 静的解析
cargo fmt --check                # フォーマット確認
```


## 📚 ドキュメント

### 👥 ユーザー向けドキュメント
- [📦 インストールガイド](docs/user/INSTALL.md) - 詳細なインストール手順
- [📖 使用方法](docs/user/USAGE.md) - 全機能の詳細説明

### 🛠️ 開発者向けドキュメント
- [🏗️ アーキテクチャ](docs/development/ARCHITECTURE.md) - システム設計・拡張性
- [🧪 テストガイド](docs/development/TESTING_GUIDE.md) - テスト実行方法・戦略
- [🔒 セキュリティガイド](docs/development/SECURITY_QUICK_REFERENCE.md) - セキュリティベストプラクティス
- [❓ ヘルプシステム](docs/development/HELP_IMPLEMENTATION_SUMMARY.md) - ヘルプ機能実装

## 🤝 コントリビューション

Backup Suiteへの貢献を歓迎します！

### 開発環境セットアップ
```bash
git clone https://github.com/user/backup-suite.git
cd backup-suite
cargo build
cargo test
```

### 貢献方法
1. Issueで問題報告・機能提案
2. プルリクエストで改善・修正
3. ドキュメント改善・翻訳
4. 使用体験のフィードバック

## 📄 ライセンス

MIT License - 詳細は [LICENSE](LICENSE) ファイルを参照

## 🚀 ロードマップ

### v1.1.0 （計画中）
- [ ] Linux systemd統合
- [ ] Windows サポート
- [ ] 設定ファイル暗号化
- [ ] 増分バックアップ機能

## 📞 サポート

- **GitHub Issues**: [問題報告・機能要求](https://github.com/user/backup-suite/issues)
- **Discussions**: [質問・アイデア共有](https://github.com/user/backup-suite/discussions)
- **Email**: support@backup-suite.example.com

---

**🦀 Backup Suite - 高速・安全・インテリジェントなバックアップソリューション**
