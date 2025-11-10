# Smart機能 実機テスト手順書

## 📋 テスト概要

- **対象**: backup-suite Smart機能 (Phase 1)
- **テスト環境**: macOS/Linux/Windows
- **所要時間**: 約15-20分
- **前提条件**: `cargo install --path . --all-features` 実行済み

---

## 🔧 事前準備

### 1. テスト環境セットアップ

```bash
# テスト用ディレクトリ作成
mkdir -p ~/backup-suite-test
cd ~/backup-suite-test

# テストファイル作成
mkdir -p project/{src,tests,node_modules,target,.git}
echo "console.log('test');" > project/src/index.js
echo "test code" > project/tests/test.js
echo "module" > project/node_modules/module.js
echo "compiled" > project/target/debug.out
echo "git data" > project/.git/config
touch project/.DS_Store

# バックアップ履歴用のダミーディレクトリ作成
mkdir -p ~/.local/share/backup-suite/backups
```

### 2. バージョン確認

```bash
backup-suite --version
# 期待値: Backup Suite v1.0.0
#         AES-256暗号化 & AI搭載のインテリジェントバックアップ
```

---

## ✅ テスト項目

### Test 1: AI helpコマンド

**目的**: Smart機能のヘルプ表示確認

```bash
backup-suite smart help
```

**期待される出力**:
```
🤖 AIコマンド ヘルプ

  detect           バックアップ履歴の異常検知
  analyze          ファイル重要度分析
  suggest-exclude  除外パターン提案
  auto-configure   AIによる自動設定

使用例:
  # 直近7日間の異常検知
  backup-suite smart detect --days 7

  # ファイル重要度分析
  backup-suite smart analyze /path/to/file

  # AI除外推奨を取得
  backup-suite smart suggest-exclude /path/to/dir
```

**チェックポイント**:
- [x] ヘルプが日本語で表示される
- [x] 4つのサブコマンドが表示される
- [x] 使用例が表示される

---

### Test 2: AI異常検知 (detect)

**目的**: バックアップ履歴の異常検知機能

#### 2-1. データなしケース

**前提**: 既存のバックアップ履歴を一時的に削除する必要があります

**方法1: ワンライナー（推奨）**
```bash
[ -f ~/.config/backup-suite/history.toml ] && mv ~/.config/backup-suite/history.toml ~/.config/backup-suite/history.toml.backup; backup-suite smart detect --days 7; [ -f ~/.config/backup-suite/history.toml.backup ] && mv ~/.config/backup-suite/history.toml.backup ~/.config/backup-suite/history.toml
```

**方法2: スクリプト形式**
```bash
# 既存の履歴を一時バックアップ（存在する場合のみ）
if [ -f ~/.config/backup-suite/history.toml ]; then
    mv ~/.config/backup-suite/history.toml ~/.config/backup-suite/history.toml.backup
fi

# データなしケースのテスト
backup-suite smart detect --days 7

# 履歴を復元（バックアップが存在する場合のみ）
if [ -f ~/.config/backup-suite/history.toml.backup ]; then
    mv ~/.config/backup-suite/history.toml.backup ~/.config/backup-suite/history.toml
fi
```

**Windows (PowerShell)の場合**:
```powershell
# 既存の履歴を一時バックアップ
$historyPath = "$env:USERPROFILE\.config\backup-suite\history.toml"
if (Test-Path $historyPath) {
    Move-Item $historyPath "$historyPath.backup"
}

# データなしケースのテスト
backup-suite smart detect --days 7

# 履歴を復元
if (Test-Path "$historyPath.backup") {
    Move-Item "$historyPath.backup" $historyPath
}
```

**期待される出力**:
```
🤖 AI異常検知
過去7日間のバックアップを分析中...

⚠️  データが不足しています（最低3件必要、0件しかありません）
```

**チェックポイント**:
- [x] データ不足のエラーメッセージが表示される
- [x] エラーが適切にハンドリングされている
- [x] 最低3件のデータが必要であることが明示される

#### 2-2. 通常バックアップ実行後

```bash
# バックアップ登録・実行
backup-suite add ~/backup-suite-test/project --priority high
backup-suite run --priority high

# 異常検知実行
backup-suite smart detect --days 7
```

**期待される出力**:
```
🤖 AI異常検知

異常は検出されませんでした
または
異常検知結果のテーブル表示
```

**チェックポイント**:
- [x] エラーなく実行される
- [x] 結果が適切に表示される

---

### Test 3: AI ファイル重要度分析 (analyze)

**目的**: ファイルの重要度を分析

#### 3-1. ソースコードファイル

```bash
backup-suite smart analyze ~/backup-suite-test/project/src/index.js
```

**期待される出力**:
```
🤖 AI ファイル重要度分析

ファイル: ~/backup-suite-test/project/src/index.js
重要度: 高 (High)
理由: ソースコードファイル
```

**チェックポイント**:
- [x] 重要度が「高」と判定される
- [x] ソースコードとして認識される

#### 3-2. OS固有ファイル

```bash
backup-suite smart analyze ~/backup-suite-test/project/.DS_Store
```

**期待される出力**:
```
🤖 AI ファイル重要度分析
パス: "/Users/sanae.abe/backup-suite-test/project/.DS_Store"

  重要度スコア: 5/100
  推奨優先度: Low
  カテゴリ: OS固有ファイル
  理由: OS固有ファイル (ファイル名: .DS_Store, スコア: 5)
```

**チェックポイント**:
- [x] 重要度スコアが「5/100」と判定される（極低重要度）
- [x] カテゴリが「OS固有ファイル」として認識される
- [x] 推奨優先度が「Low」と判定される

#### 3-3. 存在しないファイル

```bash
backup-suite smart analyze /nonexistent/file.txt
```

**期待される出力**:
```
エラー: ファイルシステムアクセス中にエラーが発生しました
```

**チェックポイント**:
- [x] エラーが適切にハンドリングされる
- [x] ユーザーフレンドリーなエラーメッセージ

---

### Test 4: AI 除外パターン推奨 (suggest-exclude)

**目的**: プロジェクトディレクトリの除外パターンを推奨

#### 4-1. 基本的な除外パターン提案（信頼度80%以上）

```bash
backup-suite smart suggest-exclude ~/backup-suite-test/project
```

**期待される出力**:
```
🤖 AI除外パターン提案
パス: "/Users/sanae.abe/backup-suite-test/project"

+--------------+--------+--------------+------------------------------------------------+
| パターン     | 信頼度 | 削減見込(GB) | 理由                                           |
+=======================================================================================+
| target       | 99.0%  | 0.00         | Rustビルド成果物（Cargo.tomlから再生成可能）   |
|--------------+--------+--------------+------------------------------------------------|
| node_modules | 99.0%  | 0.00         | npm/yarn依存関係（package.jsonから再生成可能） |
|--------------+--------+--------------+------------------------------------------------|
| .DS_Store    | 99.0%  | 0.00         | macOSメタデータファイル（自動生成）            |
+--------------+--------+--------------+------------------------------------------------+
```

**チェックポイント**:
- [x] node_modules が検出される（信頼度99%）
- [x] target が検出される（信頼度99%）
- [x] .DS_Store が検出される（信頼度99%）
- [x] テーブル形式で表示される
- [x] 信頼度と削減見込サイズが表示される

#### 4-2. より多くの提案を表示（信頼度60%以上）

```bash
backup-suite smart suggest-exclude ~/backup-suite-test/project --confidence 0.6
```

**期待される出力**:
```
🤖 AI除外パターン提案
パス: "/Users/sanae.abe/backup-suite-test/project"

+--------------+--------+--------------+-------------------------------------------------+
| パターン     | 信頼度 | 削減見込(GB) | 理由                                            |
+========================================================================================+
| target       | 99.0%  | 0.00         | Rustビルド成果物（Cargo.tomlから再生成可能）    |
|--------------+--------+--------------+-------------------------------------------------|
| .git         | 70.0%  | 0.00         | Gitリポジトリメタデータ（リモートから復元可能） |
|--------------+--------+--------------+-------------------------------------------------|
| node_modules | 99.0%  | 0.00         | npm/yarn依存関係（package.jsonから再生成可能）  |
|--------------+--------+--------------+-------------------------------------------------|
| .DS_Store    | 99.0%  | 0.00         | macOSメタデータファイル（自動生成）             |
+--------------+--------+--------------+-------------------------------------------------+
```

**チェックポイント**:
- [x] .git が検出される（信頼度70%）
- [x] より多くのパターンが表示される
- [x] 信頼度フィルタが機能している

---

### Test 5: AI 自動設定 (auto-configure)

**目的**: AIによる自動バックアップ設定

**前提条件**: プロジェクトディレクトリが未登録の状態

```bash
# 既に登録されている場合は削除
backup-suite remove ~/backup-suite-test/project
```

#### 5-1. 基本的な自動設定

```bash
backup-suite smart auto-configure ~/backup-suite-test/project
```

**期待される出力**:
```
🤖 AI自動設定
分析中: "/Users/sanae.abe/backup-suite-test/project"
  推奨優先度: High (スコア: 90)
  ✅ 設定に追加しました

自動設定が完了しました
  追加された項目: 1
```

**チェックポイント**:
- [x] 推奨優先度が High と判定される
- [x] スコアが 85-95 程度（ソースコードプロジェクトとして認識）
- [x] 自動的に設定に追加される

**補足**:
- 現在の実装はディレクトリ内のファイルを分析し、プロジェクトタイプ（Rust/Node.js/Python等）を判定
- src/, package.json, Cargo.toml などのプロジェクトマーカーを検出
- 高重要度ファイルの比率に基づいて優先度を決定

#### 5-2. ドライランモード

```bash
backup-suite smart auto-configure ~/backup-suite-test/project2 --dry-run
```

**期待される出力**:
```
🤖 AI自動設定
[ドライラン モード]

分析中: "/Users/sanae.abe/backup-suite-test/project2"
  推奨優先度: High (スコア: 90)
```

**チェックポイント**:
- [x] ドライランモードが明示される
- [x] 設定は実際には追加されない
- [x] 分析結果のみ表示される

**注意**: 除外パターン自動提案、圧縮推奨、対話的確認はPhase 2で実装予定

---

### Test 6: パフォーマンステスト

**目的**: 大量ファイルでのパフォーマンス確認

#### 6-1. 大量ファイル作成

```bash
# 1000ファイル作成
mkdir -p ~/backup-suite-test/large-project
for i in {1..1000}; do
    echo "file $i" > ~/backup-suite-test/large-project/file_$i.txt
done
```

#### 6-2. 除外パターン推奨（パフォーマンス測定）

```bash
time backup-suite smart suggest-exclude ~/backup-suite-test/large-project
```

**期待される結果**:
- [x] 500ms以内に完了（目標: 1000ファイルで500ms以内）
- [x] メモリ使用量が過剰でない

---

### Test 7: エラーハンドリング

**目的**: 異常系の適切な処理確認

#### 7-1. 無効なパス

```bash
backup-suite smart analyze /invalid/path/file.txt
```

**チェックポイント**:
- [x] エラーメッセージが表示される
- [x] プログラムがクラッシュしない

#### 7-2. パストラバーサル攻撃

```bash
backup-suite smart suggest-exclude "../../etc"
```

**チェックポイント**:
- [x] パストラバーサルが防止される
- [x] エラーまたは適切な処理

#### 7-3. 深すぎるディレクトリ構造

```bash
# 100階層のディレクトリ作成
mkdir -p ~/backup-suite-test/deep/$(printf 'a/%.0s' {1..100})
backup-suite smart suggest-exclude ~/backup-suite-test/deep
```

**チェックポイント**:
- [x] タイムアウトしない
- [x] メモリ不足にならない

---

### Test 8: 多言語対応

**目的**: 国際化対応の確認

#### 8-1. 英語

```bash
LANG=en_US.UTF-8 backup-suite smart help
```

**期待される出力**:
```
🤖 AI Commands

  detect           Detect anomalies in backup history
  analyze          Analyze file importance
  ...
```

**チェックポイント**:
- [x] 英語で表示される

#### 8-2. 中国語（簡体字）

```bash
LANG=zh_CN.UTF-8 backup-suite smart help
```

**期待される出力**:
```
🤖 AI命令

  detect           检测备份历史中的异常
  analyze          分析文件重要性
  ...
```

**チェックポイント**:
- [x] 中国語（簡体字）で表示される

---

### Test 9: 統合テスト

**目的**: 複数機能の組み合わせ

```bash
# 1. ディレクトリ分析
backup-suite smart suggest-exclude ~/backup-suite-test/project

# 2. ファイル重要度分析
backup-suite smart analyze ~/backup-suite-test/project/src/index.js

# 3. バックアップ実行
backup-suite run --priority high

# 4. 異常検知
backup-suite smart detect --days 7
```

**チェックポイント**:
- [x] 全てのコマンドがエラーなく実行される
- [x] 各コマンドの出力が一貫している

---

### Test 10: セキュリティテスト

**目的**: セキュリティ対策の確認

#### 10-1. シンボリックリンク攻撃

```bash
# シンボリックリンク作成
ln -s /etc/passwd ~/backup-suite-test/project/symlink

backup-suite smart suggest-exclude ~/backup-suite-test/project
```

**チェックポイント**:
- [x] シンボリックリンクが追跡されない
- [x] /etc/passwd の内容が読まれない

#### 10-2. 機密情報の漏洩防止

```bash
# 存在しないファイルでエラー
backup-suite smart analyze ~/.ssh/id_rsa 2>&1 | grep -i "password\|key\|secret"
```

**チェックポイント**:
- [x] エラーメッセージに機密情報が含まれない
- [x] パスのみが表示される

---

## 📊 テスト結果サマリー

### テスト実行チェックリスト

| # | テスト項目 | 結果 | 備考 |
|---|----------|------|------|
| 1 | AI helpコマンド | ☐ Pass / ☐ Fail |  |
| 2-1 | 異常検知（データなし） | ☐ Pass / ☐ Fail |  |
| 2-2 | 異常検知（通常） | ☐ Pass / ☐ Fail |  |
| 3-1 | 重要度分析（ソースコード） | ☐ Pass / ☐ Fail |  |
| 3-2 | 重要度分析（キャッシュ） | ☐ Pass / ☐ Fail |  |
| 3-3 | 重要度分析（エラー） | ☐ Pass / ☐ Fail |  |
| 4 | 除外パターン推奨 | ☐ Pass / ☐ Fail |  |
| 5-1 | 自動設定（基本） | ☐ Pass / ☐ Fail |  |
| 5-2 | 自動設定（ドライラン） | ☐ Pass / ☐ Fail |  |
| 6 | パフォーマンステスト | ☐ Pass / ☐ Fail |  |
| 7-1 | エラー処理（無効パス） | ☐ Pass / ☐ Fail |  |
| 7-2 | パストラバーサル防止 | ☐ Pass / ☐ Fail |  |
| 7-3 | 深いディレクトリ | ☐ Pass / ☐ Fail |  |
| 8-1 | 多言語（英語） | ☐ Pass / ☐ Fail |  |
| 8-2 | 多言語（中国語） | ☐ Pass / ☐ Fail |  |
| 9 | 統合テスト | ☐ Pass / ☐ Fail |  |
| 10-1 | シンボリックリンク | ☐ Pass / ☐ Fail |  |
| 10-2 | 機密情報漏洩防止 | ☐ Pass / ☐ Fail |  |

### 総合評価

- **合格基準**: 全テスト項目の90%以上がPass
- **クリティカル項目**: Test 7 (エラーハンドリング)、Test 10 (セキュリティ)

---

## 🧹 テスト後のクリーンアップ

```bash
# テスト用ディレクトリ削除
rm -rf ~/backup-suite-test

# バックアップ設定削除
backup-suite clear
```

---

## 📝 不具合報告テンプレート

不具合を発見した場合は、以下の情報を記録してください：

```markdown
## 不具合報告

### 環境
- OS: macOS 14.6 / Ubuntu 22.04 / Windows 11
- backup-suite バージョン: 1.0.0
- Rust バージョン: 1.82.0

### 再現手順
1.
2.
3.

### 期待される動作


### 実際の動作


### エラーメッセージ
```
エラーメッセージをここに貼り付け
```

### スクリーンショット（あれば）

```

---

**テスト完了日**: _______________
**テスト実施者**: _______________
**総合評価**: ☐ Pass / ☐ Fail
