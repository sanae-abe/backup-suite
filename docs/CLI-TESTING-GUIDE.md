# backup-suite CLI Testing Guide

**cli-testing-specialist** を使用した backup-suite の包括的CLI自動テスト

---

## 📑 目次

- [概要](#概要)
- [セットアップ](#セットアップ)
- [ローカルでのテスト実行](#ローカルでのテスト実行)
- [CI/CD統合](#cicd統合)
- [テストカテゴリ](#テストカテゴリ)
- [トラブルシューティング](#トラブルシューティング)

---

## 概要

cli-testing-specialist は backup-suite CLI の品質を自動検証するフレームワークです。

### 主な機能

- ✅ **自動解析**: backup-suite のオプション・サブコマンドを自動抽出
- ✅ **包括テスト**: 9カテゴリ 45-55 テストケースを自動生成
- ✅ **セキュリティ**: OWASP準拠のセキュリティスキャン
- ✅ **CI/CD統合**: GitHub Actions で自動実行
- ✅ **4種類レポート**: Markdown, JSON, HTML, JUnit XML

---

## セットアップ

### 1. 前提条件

```bash
# Rust (stable)
rustc --version  # 1.82.0+

# BATS (テスト実行用)
## macOS
brew install bats-core

## Ubuntu/Debian
sudo apt-get install bats

# jq (レポート表示用、オプション)
brew install jq  # macOS
sudo apt-get install jq  # Ubuntu
```

### 2. cli-testing-specialist のインストール

```bash
# GitHubから最新版をインストール
cargo install --git https://github.com/sanae-abe/cli-testing-specialist --tag v1.0.2 cli-testing-specialist

# インストール確認
cli-testing-specialist --version
# cli-testing-specialist 1.0.2
```

---

## ローカルでのテスト実行

### クイックスタート（3ステップ）

```bash
# 1. backup-suite をビルド
cargo build --release

# 2. CLI解析 + テスト生成 + 実行（一括）
cli-testing-specialist analyze target/release/backup-suite -o backup-suite.json
cli-testing-specialist generate backup-suite.json -o backup-tests -c all
cli-testing-specialist run backup-tests -f all -o reports

# 3. レポート確認
open reports/backup-tests-report.html  # macOS
# または
cat reports/backup-tests-report.md
```

### 詳細手順

#### Step 1: CLI解析

```bash
# backup-suite の構造を解析
cli-testing-specialist analyze \
  target/release/backup-suite \
  --output backup-suite-analysis.json

# 解析結果確認
jq -r '.binary_name + " v" + .version' backup-suite-analysis.json
jq '.global_options | length' backup-suite-analysis.json  # オプション数
jq '.subcommands | length' backup-suite-analysis.json     # サブコマンド数
```

#### Step 2: テスト生成

```bash
# 全カテゴリのテストを生成（デフォルト: directory-traversal除外）
cli-testing-specialist generate \
  backup-suite-analysis.json \
  --output backup-tests \
  --categories all

# 生成されたテストファイル確認
ls -lh backup-tests/
# basic.bats
# security.bats
# input-validation.bats
# ...
```

**リソース集約型テストを含める場合**:
```bash
# --include-intensive フラグを使用
cli-testing-specialist generate \
  backup-suite-analysis.json \
  --output backup-tests-full \
  --categories all \
  --include-intensive
```

#### Step 3: テスト実行

```bash
# 全フォーマットでレポート生成
cli-testing-specialist run \
  backup-tests \
  --format all \
  --output reports \
  --timeout 60

# 生成されたレポート
ls -lh reports/
# backup-tests-report.html  # ブラウザで表示
# backup-tests-report.json  # CI/CD連携
# backup-tests-report.md    # GitHubで表示
# backup-tests-junit.xml    # JUnit統合
```

### 特定カテゴリのみ実行

```bash
# セキュリティテストのみ
cli-testing-specialist generate \
  backup-suite-analysis.json \
  -o security-tests \
  -c security,input-validation

cli-testing-specialist run \
  security-tests \
  -f markdown,json \
  -o security-reports
```

---

## CI/CD統合

### GitHub Actions 設定

`.github/workflows/cli-testing.yml` が自動で設定されています。

**特徴**:
- ✅ Ubuntu/macOS マトリックステスト
- ✅ セキュリティ専用ジョブ
- ✅ テスト失敗時にCI fail
- ✅ レポートアーティファクト保存（30日間）

### CI実行確認

```bash
# ローカルでCI再現
cargo build --release
cli-testing-specialist analyze target/release/backup-suite -o analysis.json
cli-testing-specialist generate analysis.json -o tests -c all
cli-testing-specialist run tests -f all -o reports --timeout 60

# 結果確認
jq '.success_rate' reports/backup-tests-report.json
```

### テスト失敗時の対応

```bash
# 失敗したテストの詳細を確認
jq -r '.suites[].tests[] | select(.status == "Failed")' reports/backup-tests-report.json

# または Markdown レポート
cat reports/backup-tests-report.md | grep "❌"
```

---

## テストカテゴリ

| カテゴリ | テスト内容 | テスト数 | デフォルト |
|---------|-----------|---------|----------|
| **basic** | ヘルプ、バージョン、終了コード | 10 | ✅ |
| **help** | 全サブコマンドヘルプ | 動的 | ✅ |
| **security** | インジェクション、機密漏洩、TOCTOU | 25 | ✅ |
| **path** | 特殊文字パス、深い階層、Unicode | 20 | ✅ |
| **multi-shell** | bash/zsh互換性 | 12 | ✅ |
| **input-validation** | 数値/パス/列挙型オプション検証 | 25 | ✅ |
| **destructive-ops** | 確認プロンプト、--yes/--force | 16 | ✅ |
| **performance** | 起動時間、メモリ使用量 | 6 | ✅ |
| **directory-traversal** | 大量ファイル、深い階層、シンボリックリンクループ | 12 | ❌ |

**デフォルト**: 8カテゴリ（45-47テスト）
**--include-intensive**: 9カテゴリ（53-55テスト）

### directory-traversal テストについて

**除外理由**:
- `/tmp` 容量100MB以上必要
- メモリ 2GB以上推奨
- CI環境でリソース不足エラー頻発

**有効化方法**:
```bash
cli-testing-specialist generate analysis.json -o tests -c all --include-intensive
```

**推奨**:
- ローカル環境でのみ実行
- backup-suite のような大量ファイル処理ツール専用

---

## トラブルシューティング

### BATS テスト失敗

```bash
# 個別に BATS ファイルを実行
bats backup-tests/security.bats

# 詳細ログ付き
bats -t backup-tests/security.bats
```

### タイムアウトエラー

```bash
# タイムアウトを延長（デフォルト: 60秒）
cli-testing-specialist run backup-tests -f json -o reports --timeout 120
```

### /tmp 容量不足（directory-traversal テスト）

```bash
# /tmp 容量確認
df -h /tmp

# 不要ファイル削除
rm -rf /tmp/cli-test-*

# または directory-traversal を除外
cli-testing-specialist generate analysis.json -o tests -c basic,security,path
```

### CI でのテスト失敗

```bash
# GitHub Actions ログから該当箇所確認
# Artifacts から cli-test-reports-ubuntu-latest をダウンロード
# backup-tests-report.md を確認

# ローカルで再現
cargo build --release
cli-testing-specialist analyze target/release/backup-suite -o analysis.json
cli-testing-specialist generate analysis.json -o tests -c all
cli-testing-specialist run tests -f json -o reports
```

---

## FAQ

### Q1: テスト生成にどれくらい時間がかかりますか？

**A**: backup-suite の場合:
- 解析: 100-200ms
- テスト生成: 1-2秒
- テスト実行: 30-60秒（カテゴリ数による）

### Q2: CI で毎回実行すべきですか？

**A**: 推奨設定:
- **push/PR**: `basic`, `security`, `input-validation` のみ（高速）
- **scheduled（日次）**: `all` カテゴリ（包括的）

### Q3: セキュリティテストで何をチェックしますか？

**A**: OWASP Top 10 準拠:
- コマンドインジェクション（`; rm -rf /`等）
- パストラバーサル（`../../etc/passwd`）
- 機密情報漏洩（API Key、パスワード表示）
- TOCTOU攻撃
- NULL byte injection

### Q4: 独自のテストを追加できますか？

**A**: はい、生成された BATS ファイルを編集可能:
```bash
# backup-tests/custom.bats を作成
@test "Custom: backup-suite specific test" {
  run backup-suite custom-command
  [ "$status" -eq 0 ]
  [[ "$output" == *"expected"* ]]
}

# 実行
bats backup-tests/custom.bats
```

---

## 参考リンク

- **cli-testing-specialist**: https://github.com/sanae-abe/cli-testing-specialist
- **BATS**: https://github.com/bats-core/bats-core
- **backup-suite**: https://github.com/sanae-abe/backup-suite
