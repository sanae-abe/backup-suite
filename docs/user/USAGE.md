# 使用方法ガイド

Backup Suite v1.0.0の全機能と実践的な使用方法を詳しく説明します。

## 📋 目次

- [基本概念](#基本概念)
- [コマンドリファレンス](#コマンドリファレンス)
- [実践的ワークフロー](#実践的ワークフロー)
- [設定ファイル詳細](#設定ファイル詳細)
- [高度な使用方法](#高度な使用方法)
- [ベストプラクティス](#ベストプラクティス)

## 🎯 基本概念

### 優先度システム
Backup Suiteは3段階の優先度でバックアップを管理します：

| 優先度 | 用途 | 推奨頻度 | 例 |
|--------|------|----------|-----|
| **high** | 重要・緊急ファイル | 毎日 | 作業中プロジェクト、重要書類 |
| **medium** | 通常ファイル | 週次 | 完了プロジェクト、写真 |
| **low** | アーカイブ | 月次 | 古いファイル、参考資料 |

### カテゴリシステム
ファイルを用途別にカテゴリ分けして管理できます：
- `development` - 開発プロジェクト
- `work` - 業務ファイル
- `personal` - 個人ファイル
- `creative` - デザイン・創作
- `archive` - アーカイブ

### 対象タイプ
- `file` - 単一ファイル
- `directory` - ディレクトリ（再帰的）

## 📝 コマンドリファレンス

### `add` - バックアップ対象追加

#### 基本構文
```bash
backup-suite add [PATH] [OPTIONS]
```

#### オプション
- `--priority <PRIORITY>` - 優先度設定（high/medium/low、デフォルト: medium）
- `--category <CATEGORY>` - カテゴリ設定（デフォルト: user）
- `--interactive` - インタラクティブファイル選択モード

#### 使用例

```bash
# 基本的な追加
backup-suite add ~/Documents/project --priority high --category development

# カテゴリ指定
backup-suite add ~/Photos --priority medium --category personal

# インタラクティブ選択（パス省略 or --interactive）
backup-suite add --interactive
backup-suite add  # パス省略時は自動的にインタラクティブモード

# 現在のディレクトリを追加
backup-suite add . --priority high --category work

# 複数ファイル追加（スクリプト化）
for dir in ~/project1 ~/project2 ~/project3; do
    backup-suite add "$dir" --priority high --category development
done
```

#### 実行例と出力
```bash
$ backup-suite add ~/Documents/important --priority high --category work
✅ 追加: "/Users/user/Documents/important"

$ backup-suite add --interactive
# skimインターフェースが起動
# ファジーファインダーでファイル/ディレクトリを選択
✅ 追加: "/Users/user/selected/path"
```

---

### `list` (`ls`) - バックアップ対象一覧

#### 基本構文
```bash
backup-suite list [OPTIONS]
backup-suite ls [OPTIONS]  # エイリアス
```

#### オプション
- `--priority <PRIORITY>` - 指定優先度のみ表示

#### 使用例

```bash
# 全対象表示
backup-suite list

# 高優先度のみ表示
backup-suite list --priority high

# エイリアス使用
backup-suite ls --priority medium
```

#### 実行例と出力
```bash
$ backup-suite list
📋 バックアップ対象一覧
1. "/Users/user/Documents/project" [High] development
2. "/Users/user/Photos" [Medium] personal
3. "/Users/user/Archive" [Low] archive
合計: 3 件

$ backup-suite list --priority high
📋 バックアップ対象一覧
1. "/Users/user/Documents/project" [High] development
合計: 1 件
```

---

### `remove` - バックアップ対象削除

#### 基本構文
```bash
backup-suite remove [PATH] [OPTIONS]
```

#### オプション
- `--interactive` - インタラクティブ対象選択モード

#### 使用例

```bash
# パス指定で削除
backup-suite remove ~/Documents/old-project

# インタラクティブ選択削除
backup-suite remove --interactive

# パス省略時は自動的にインタラクティブモード
backup-suite remove
```

#### 実行例と出力
```bash
$ backup-suite remove ~/Documents/old-project
✅ 削除: "/Users/user/Documents/old-project"

$ backup-suite remove --interactive
# 既存対象から選択UI表示
削除するバックアップ対象を選択:
> /Users/user/Documents/project [High] development
  /Users/user/Photos [Medium] personal
  /Users/user/Archive [Low] archive
✅ 削除: "/Users/user/Documents/project"
```

---

### `clear` (`rm`) - 一括削除

#### 基本構文
```bash
backup-suite clear [OPTIONS]
backup-suite rm [OPTIONS]  # エイリアス
```

#### オプション
- `--priority <PRIORITY>` - 指定優先度の対象を一括削除
- `--all` - 全対象削除

#### 使用例

```bash
# 低優先度の対象をすべて削除
backup-suite clear --priority low

# 全対象削除（注意！）
backup-suite clear --all

# エイリアス使用
backup-suite rm --priority medium
```

#### 実行例と出力
```bash
$ backup-suite clear --priority low
✅ 2 件削除

$ backup-suite clear --all
✅ 5 件削除
```

---

### `run` - バックアップ実行

#### 基本構文
```bash
backup-suite run [OPTIONS]
```

#### オプション
- `--priority <PRIORITY>` - 指定優先度のみ実行
- `--category <CATEGORY>` - 指定カテゴリのみ実行
- `--dry-run` - ドライラン（実際には実行せず確認のみ）
- `--encrypt` - AES-256-GCM暗号化を有効化
- `--password <PASSWORD>` - 暗号化パスワード（省略時はプロンプト表示）
- `--compress <TYPE>` - 圧縮アルゴリズム（zstd/gzip/none、デフォルト: zstd）
- `--compress-level <LEVEL>` - 圧縮レベル（zstd: 1-22, gzip: 1-9、デフォルト: 3）

#### 使用例

```bash
# 全対象をバックアップ
backup-suite run

# 高優先度のみバックアップ
backup-suite run --priority high

# 特定カテゴリのみバックアップ
backup-suite run --category development

# 暗号化バックアップ（AES-256-GCM）
backup-suite run --encrypt --password "your-password"
backup-suite run --encrypt  # パスワードはプロンプトで入力

# 圧縮バックアップ（zstd高速圧縮）
backup-suite run --compress zstd --compress-level 3

# 圧縮バックアップ（gzip互換性重視）
backup-suite run --compress gzip --compress-level 6

# 暗号化＋圧縮バックアップ
backup-suite run --encrypt --compress zstd

# ドライラン（確認のみ）
backup-suite run --dry-run

# 中優先度のドライラン
backup-suite run --priority medium --dry-run

# 暗号化＋圧縮＋カテゴリ指定
backup-suite run --encrypt --compress zstd --category work
```

#### 実行例と出力
```bash
$ backup-suite run --priority high
🚀 バックアップ実行
📊 結果: 150/150 成功, 25.67 MB

$ backup-suite run --dry-run
🚀 バックアップ実行（ドライラン）
📋 検出: 300 ファイル

$ backup-suite run --encrypt --compress zstd
暗号化パスワード: ****
🚀 バックアップ実行（暗号化、圧縮: zstd）
📊 結果: 150/150 成功, 12.34 MB（圧縮後）

$ backup-suite run --category development
🚀 バックアップ実行（カテゴリ: development）
📊 結果: 75/75 成功, 18.42 MB
```

---

### `restore` - バックアップ復元

#### 基本構文
```bash
backup-suite restore [OPTIONS]
```

#### オプション
- `--from <PATTERN>` - 復元元バックアップ指定（パターンマッチ）
- `--to <PATH>` - 復元先ディレクトリ指定（デフォルト: ./.restored）
- `--password <PASSWORD>` - 復号化パスワード（暗号化バックアップの場合、省略時はプロンプト表示）

#### 使用例

```bash
# 最新バックアップから復元
backup-suite restore

# 特定日付のバックアップから復元
backup-suite restore --from backup-20251104

# カスタム復元先指定
backup-suite restore --to ~/recovered-files

# 特定バックアップを特定場所に復元
backup-suite restore --from backup-20251104 --to ~/project-recovery

# 暗号化バックアップの復元
backup-suite restore --password "your-password"
backup-suite restore --from backup-20251104 --password "your-password" --to ~/restored

# 暗号化バックアップ（パスワードプロンプト）
backup-suite restore  # 暗号化ファイル検出時に自動的にパスワード入力を要求
```

#### 実行例と出力
```bash
$ backup-suite restore
🔄 復元開始: "/Users/user/backup-suite/backups/backup-20251104-143000" → "./.restored/backup_20251104_143000"
✅ バックアップを "./.restored/backup_20251104_143000" に復元しました
  復元ファイル数: 150 (暗号化: 0ファイル)

$ backup-suite restore --from backup-20251104 --to ~/recovered
🔄 復元開始: "/Users/user/backup-suite/backups/backup-20251104-143000" → "/Users/user/recovered/backup_20251104_143000"
✅ バックアップを "/Users/user/recovered/backup_20251104_143000" に復元しました
  復元ファイル数: 150 (暗号化: 0ファイル)

$ backup-suite restore --password "my-password"
🔄 復元開始: "/Users/user/backup-suite/backups/backup-20251104-143000" → "./.restored/backup_20251104_143000"
✅ バックアップを "./.restored/backup_20251104_143000" に復元しました
  復元ファイル数: 150 (暗号化: 150ファイル)
```

---

### `cleanup` - 古いバックアップ削除

#### 基本構文
```bash
backup-suite cleanup [OPTIONS]
```

#### オプション
- `--days <DAYS>` - 指定日数より古いバックアップを削除（デフォルト: 30）
- `--dry-run` - ドライラン（削除せず確認のみ）

#### 使用例

```bash
# 30日より古いバックアップを削除（デフォルト）
backup-suite cleanup

# 7日より古いバックアップを削除
backup-suite cleanup --days 7

# ドライラン（削除対象確認）
backup-suite cleanup --days 30 --dry-run

# 1年より古いバックアップを削除
backup-suite cleanup --days 365
```

#### 実行例と出力
```bash
$ backup-suite cleanup --days 7 --dry-run
🗑️ 削除: "/Users/user/backup-suite/backups/backup-20251028-143000"
🗑️ 削除: "/Users/user/backup-suite/backups/backup-20251029-143000"
✅ 2 件削除（ドライラン）

$ backup-suite cleanup --days 7
🗑️ 削除: "/Users/user/backup-suite/backups/backup-20251028-143000"
🗑️ 削除: "/Users/user/backup-suite/backups/backup-20251029-143000"
✅ 2 件削除
```

---

### `status` - 現在の状態表示

#### 基本構文
```bash
backup-suite status
```

#### 使用例と出力
```bash
$ backup-suite status
📊 ステータス
  保存先: "/Users/user/backup-suite/backups"
  対象: 15
    高: 5
    中: 7
    低: 3
```

---

### `history` - バックアップ履歴表示

#### 基本構文
```bash
backup-suite history [OPTIONS]
```

#### オプション
- `--days <DAYS>` - 表示する履歴の日数（デフォルト: 7）

#### 使用例

```bash
# 過去7日間の履歴（デフォルト）
backup-suite history

# 過去30日間の履歴
backup-suite history --days 30

# 過去1日の履歴
backup-suite history --days 1
```

#### 実行例と出力
```bash
$ backup-suite history --days 7
📜 バックアップ履歴（7日間）
1. ✅ 2025-11-04 14:30:00
   /Users/user/backup-suite/backups/backup-20251104-143000: 150 ファイル, 25.67 MB
2. ✅ 2025-11-03 14:30:00
   /Users/user/backup-suite/backups/backup-20251103-143000: 148 ファイル, 25.23 MB
```

---

### `dashboard` - 統計ダッシュボード

#### 基本構文
```bash
backup-suite dashboard
```

#### 使用例と出力
```bash
$ backup-suite dashboard
╔═══════════════════════════════════════╗
║      Backup Suite Dashboard          ║
╚═══════════════════════════════════════╝

📊 統計
  登録対象: 15 件
  総バックアップ: 45 回
  成功率: 98.9%

📅 最新バックアップ
  日時: 2025-11-04 14:30:00
  ファイル: 150
  サイズ: 25.67 MB
```

---

### `schedule` - スケジューリング管理

#### 基本構文
```bash
backup-suite schedule <ACTION> [OPTIONS]
```

#### サブコマンド

##### `setup` - スケジュール設定
```bash
backup-suite schedule setup [OPTIONS]
```

**オプション:**
- `--high <FREQUENCY>` - 高優先度の実行頻度（デフォルト: daily）
- `--medium <FREQUENCY>` - 中優先度の実行頻度（デフォルト: weekly）
- `--low <FREQUENCY>` - 低優先度の実行頻度（デフォルト: monthly）

**頻度オプション:**
- `daily` - 毎日2:00AM
- `weekly` - 毎週日曜2:00AM
- `monthly` - 毎月1日2:00AM
- `hourly` - 毎時（開発・テスト用）

```bash
# デフォルト設定
backup-suite schedule setup

# カスタム頻度設定
backup-suite schedule setup --high daily --medium weekly --low monthly

# すべて週次に設定
backup-suite schedule setup --high weekly --medium weekly --low weekly
```

##### `enable` - 自動バックアップ有効化
```bash
backup-suite schedule enable [OPTIONS]
```

**オプション:**
- `--priority <PRIORITY>` - 特定優先度のみ有効化

```bash
# 全優先度の自動バックアップ有効化
backup-suite schedule enable

# 高優先度のみ有効化
backup-suite schedule enable --priority high

# 中優先度のみ有効化
backup-suite schedule enable --priority medium
```

##### `disable` - 自動バックアップ無効化
```bash
backup-suite schedule disable [OPTIONS]
```

**オプション:**
- `--priority <PRIORITY>` - 特定優先度のみ無効化

```bash
# 全優先度の自動バックアップ無効化
backup-suite schedule disable

# 高優先度のみ無効化
backup-suite schedule disable --priority high
```

##### `status` - スケジュール状態確認
```bash
backup-suite schedule status
```

#### 実行例と出力
```bash
$ backup-suite schedule setup --high daily --medium weekly --low monthly
📅 高優先度スケジュール設定完了: daily
📅 中優先度スケジュール設定完了: weekly
📅 低優先度スケジュール設定完了: monthly

$ backup-suite schedule enable
✅ 自動バックアップ有効化

$ backup-suite schedule status
📅 スケジュール設定
  有効: ✅
  高優先度: daily
  中優先度: weekly
  低優先度: monthly

📋 実際のスケジュール状態
  high: ✅ 有効
  medium: ✅ 有効
  low: ✅ 有効
```

---

### `config` - 設定管理

#### 基本構文
```bash
backup-suite config <ACTION> [ARGS]
```

#### サブコマンド

##### `set-destination` - バックアップ保存先変更
```bash
backup-suite config set-destination <PATH>
```

**引数:**
- `<PATH>` - 新しいバックアップ保存先ディレクトリパス（チルダ展開対応）

```bash
# バックアップ先を外付けHDDに変更
backup-suite config set-destination /Volumes/ExternalHDD/backups

# ホームディレクトリ内に変更（チルダ展開）
backup-suite config set-destination ~/Documents/backups

# NASに変更
backup-suite config set-destination /mnt/nas/backup-suite
```

##### `get-destination` - 現在のバックアップ保存先表示
```bash
backup-suite config get-destination
```

```bash
$ backup-suite config get-destination
📁 現在のバックアップ先
  "/Users/user/backup-suite/backups"
```

##### `set-keep-days` - バックアップ保持期間変更
```bash
backup-suite config set-keep-days <DAYS>
```

**引数:**
- `<DAYS>` - バックアップ保持日数（1-3650日）

```bash
# 保持期間を60日に変更
backup-suite config set-keep-days 60

# 保持期間を1年に変更
backup-suite config set-keep-days 365

# 保持期間を最小（1日）に変更
backup-suite config set-keep-days 1
```

##### `get-keep-days` - 現在のバックアップ保持期間表示
```bash
backup-suite config get-keep-days
```

```bash
$ backup-suite config get-keep-days
📅 現在のバックアップ保持期間
  30日
```

##### `open` - 設定ファイルをエディタで開く
```bash
backup-suite config open
```

**動作:**
- 環境変数 `$EDITOR` または `$VISUAL` で指定されたエディタで開く
- macOSでは環境変数未設定時に `open` コマンド（デフォルトエディタ）を使用
- Linuxでは `nano` をフォールバック
- Windowsでは `notepad` をフォールバック

```bash
# デフォルトエディタで開く
backup-suite config open

# 環境変数で指定したエディタで開く
EDITOR=vim backup-suite config open
EDITOR=code backup-suite config open  # VS Code
```

#### 実行例と出力

```bash
$ backup-suite config set-destination ~/my-backups
📁 ディレクトリが存在しません。作成します: "/Users/user/my-backups"
✅ バックアップ先を変更しました
  変更前: "/Users/user/backup-suite/backups"
  変更後: "/Users/user/my-backups"

$ backup-suite config get-destination
📁 現在のバックアップ先
  "/Users/user/my-backups"

$ backup-suite config set-keep-days 90
✅ バックアップ保持期間を変更しました
  変更前: 30日
  変更後: 90日

$ backup-suite config get-keep-days
📅 現在のバックアップ保持期間
  90日

$ backup-suite config open
📝 設定ファイルを開きます: "/Users/user/.config/backup-suite/config.toml"
# デフォルトエディタで設定ファイルが開かれる
```

---

### `ai` - AI駆動のインテリジェントバックアップ管理（要 `--features smart`）

Smart機能を使用するには、`--features smart` フラグを付けてビルドする必要があります。

```bash
# Smart機能を有効化してビルド
cargo build --release --features smart
cargo install --path . --features smart
```

#### サブコマンド

##### `ai detect` - 異常検知

過去の履歴から統計的に異常なバックアップを検知します。

**基本構文:**
```bash
backup-suite smart detect [OPTIONS]
```

**オプション:**
- `--days <DAYS>` - 分析する履歴の日数（デフォルト: 7）
- `--format <FORMAT>` - 出力形式（table/json/detailed、デフォルト: table）

**使用例:**
```bash
# 過去7日間の異常検知（デフォルト）
backup-suite smart detect

# 過去14日間を詳細分析
backup-suite smart detect --days 14 --format detailed

# JSON形式で出力
backup-suite smart detect --format json
```

**実行例と出力:**
```bash
$ backup-suite smart detect --days 7
🤖 AI異常検知レポート（過去7日間）

┌────┬──────────────────┬──────────┬──────────┬─────────────────────┐
│ No │ 検出日時          │ 異常種別  │ 信頼度    │ 説明                 │
├────┼──────────────────┼──────────┼──────────┼─────────────────────┤
│ 1  │ 2025-11-09 03:15 │ サイズ急増│ 95.3%    │ ファイルサイズが通常の3倍 │
└────┴──────────────────┴──────────┴──────────┴─────────────────────┘

📊 サマリー: 1件の異常を検出
💡 推奨アクション: ~/Downloads の一時ファイルを除外設定に追加
```

---

##### `ai analyze` - ファイル重要度分析

ディレクトリ内のファイルを重要度別に分類し、バックアップ戦略を最適化します。

**基本構文:**
```bash
backup-suite smart analyze <PATH> [OPTIONS]
```

**引数:**
- `<PATH>` - 分析対象のディレクトリパス

**オプション:**
- `--suggest-priority` - 推奨優先度に基づいたコマンドを提案
- `--detailed` - 詳細な分析結果を表示

**使用例:**
```bash
# ディレクトリの重要度分析
backup-suite smart analyze ~/documents

# 詳細な重要度スコア表示
backup-suite smart analyze ~/documents --detailed

# 推奨コマンド付きで表示
backup-suite smart analyze ~/projects --suggest-priority
```

**実行例と出力:**
```bash
$ backup-suite smart analyze ~/Documents
🤖 AIファイル重要度分析: ~/Documents

  重要度スコア: 90/100
  推奨優先度: High
  カテゴリ: ドキュメント
  理由: PDFファイル（頻繁に更新）

$ backup-suite smart analyze ~/projects --suggest-priority
🤖 AIファイル重要度分析: ~/projects

  重要度スコア: 95/100
  推奨優先度: High
  カテゴリ: Rustプロジェクト
  理由: Cargo.toml検出（開発中プロジェクト）

💡 推奨コマンド: backup-suite add "/Users/user/projects" --priority High
```

---

##### `ai suggest-exclude` - 除外パターン推奨

不要なファイルを自動検出し、除外パターンを推奨します。

**基本構文:**
```bash
backup-suite smart suggest-exclude <PATH> [OPTIONS]
```

**引数:**
- `<PATH>` - 分析対象のディレクトリパス

**オプション:**
- `--apply` - 推奨パターンを設定ファイルに自動適用
- `--confidence <VALUE>` - 最小信頼度（0.0-1.0、デフォルト: 0.8）

**使用例:**
```bash
# 除外パターンの推奨を表示
backup-suite smart suggest-exclude ~/projects

# 推奨パターンを自動的に設定ファイルに適用
backup-suite smart suggest-exclude ~/projects --apply

# 最小信頼度を50%に設定（より多くの候補を表示）
backup-suite smart suggest-exclude ~/projects --confidence 0.5
```

**実行例と出力:**
```bash
$ backup-suite smart suggest-exclude ~/projects
🤖 AI除外パターン推奨: ~/projects

┌──────────────────┬──────────┬──────────┬─────────────────────┐
│ パターン          │ 削減量    │ 信頼度    │ 理由                 │
├──────────────────┼──────────┼──────────┼─────────────────────┤
│ node_modules/    │ 2.34 GB  │ 99%      │ npm依存関係（再生成可能）│
│ target/          │ 1.87 GB  │ 99%      │ Rustビルド成果物      │
│ .cache/          │ 0.45 GB  │ 95%      │ キャッシュディレクトリ │
└──────────────────┴──────────┴──────────┴─────────────────────┘

💡 総削減量: 4.66 GB（バックアップ時間を約30%短縮）

$ backup-suite smart suggest-exclude ~/projects --apply
🤖 AI除外パターン推奨: ~/projects

┌──────────────────┬──────────┬──────────┬─────────────────────┐
│ パターン          │ 削減量    │ 信頼度    │ 理由                 │
├──────────────────┼──────────┼──────────┼─────────────────────┤
│ node_modules/    │ 2.34 GB  │ 99%      │ npm依存関係（再生成可能）│
│ target/          │ 1.87 GB  │ 99%      │ Rustビルド成果物      │
│ .cache/          │ 0.45 GB  │ 95%      │ キャッシュディレクトリ │
└──────────────────┴──────────┴──────────┴─────────────────────┘

"node_modules/" を除外リストに追加しますか？ (2.34GB 削減見込) (y/n): y
✅ "node_modules/" を追加しました

"target/" を除外リストに追加しますか？ (1.87GB 削減見込) (y/n): y
✅ "target/" を追加しました

".cache/" を除外リストに追加しますか？ (0.45GB 削減見込) (y/n): y
✅ ".cache/" を追加しました
```

---

##### `ai auto-configure` - AI自動設定

ディレクトリを分析し、最適なバックアップ設定を自動生成します。

**基本構文:**
```bash
backup-suite smart auto-configure <PATHS>... [OPTIONS]
```

**引数:**
- `<PATHS>...` - 設定対象のディレクトリパス（複数指定可能）

**オプション:**
- `--dry-run` - ドライラン（設定を適用せず確認のみ）
- `--interactive` - 対話モード（各サブディレクトリと除外パターンを確認）
- `--max-depth <DEPTH>` - サブディレクトリの探索深度（1 = 直下のみ、デフォルト: 1）

**使用例:**
```bash
# 自動分析・設定（サブディレクトリを個別に評価）
backup-suite smart auto-configure ~/data

# 対話的に確認しながら設定（サブディレクトリと除外パターンを確認）
backup-suite smart auto-configure ~/data --interactive

# ドライラン（設定を適用せず確認のみ）
backup-suite smart auto-configure ~/data --dry-run

# サブディレクトリの探索深度を指定（2階層まで）
backup-suite smart auto-configure ~/data --max-depth 2

# 複数ディレクトリを一度に設定
backup-suite smart auto-configure ~/projects ~/documents ~/photos
```

**機能:**
- **サブディレクトリごとに重要度を個別評価**: 各ディレクトリに最適な優先度を自動設定
- **除外パターンの自動検出・適用**: `node_modules/`, `target/`, `.cache/` 等を自動除外
- **プロジェクトタイプの自動判定**: Rust, Node.js, Python 等を検出し最適な設定を提案
- **信頼度80%以上のパターンのみ適用**: 誤検出を防止

**実行例と出力:**
```bash
$ backup-suite smart auto-configure ~/projects
🤖 AI自動設定
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

$ backup-suite smart auto-configure ~/projects --interactive
🤖 AI自動設定
分析中: "/Users/user/projects"
  📁 3個のサブディレクトリを発見: 3
    評価中: "/Users/user/projects/web-app"
      推奨優先度: High (スコア: 95)
      📋 除外パターン提案: 3
        - node_modules (99.0%, 2.34 GB 削減見込)
        - .cache (95.0%, 0.45 GB 削減見込)
        - .*\.tmp$ (99.0%, 0.00 GB 削減見込)
      "node_modules" を除外リストに追加しますか？ (y/n): y
      ".cache" を除外リストに追加しますか？ (y/n): y
      ".*\.tmp$" を除外リストに追加しますか？ (y/n): n
      📝 除外パターン: node_modules, .cache
      "/Users/user/projects/web-app" を優先度 High で追加しますか？ (y/n): y
      ✅ 設定に追加しました
    評価中: "/Users/user/projects/rust-cli"
      推奨優先度: High (スコア: 95)
      📋 除外パターン提案: 2
        - target (99.0%, 1.87 GB 削減見込)
        - .cache (95.0%, 0.12 GB 削減見込)
      "target" を除外リストに追加しますか？ (y/n): y
      ".cache" を除外リストに追加しますか？ (y/n): y
      📝 除外パターン: target, .cache
      "/Users/user/projects/rust-cli" を優先度 High で追加しますか？ (y/n): y
      ✅ 設定に追加しました
    評価中: "/Users/user/projects/archive"
      推奨優先度: Low (スコア: 30)
      "/Users/user/projects/archive" を優先度 Low で追加しますか？ (y/n): n

自動設定が完了しました
  追加された項目: 2
  総削減量: 4.78 GB（バックアップ時間を約35%短縮）

$ backup-suite smart auto-configure ~/projects --dry-run
🤖 AI自動設定
[ドライラン モード]

分析中: "/Users/user/projects"
  📁 3個のサブディレクトリを発見: 3
    評価中: "/Users/user/projects/web-app"
      推奨優先度: High (スコア: 95)
      📋 除外パターン提案: 3
        - node_modules (99.0%, 2.34 GB 削減見込)
        - .cache (95.0%, 0.45 GB 削減見込)
        - .*\.tmp$ (99.0%, 0.00 GB 削減見込)
      📝 除外パターン: node_modules, .cache, .*\.tmp$
    評価中: "/Users/user/projects/rust-cli"
      推奨優先度: High (スコア: 95)
      📋 除外パターン提案: 2
        - target (99.0%, 1.87 GB 削減見込)
        - .cache (95.0%, 0.12 GB 削減見込)
      📝 除外パターン: target, .cache
    評価中: "/Users/user/projects/archive"
      推奨優先度: Low (スコア: 30)

ドライラン完了（設定は適用されていません）
  推奨項目: 3
  総削減見込: 4.78 GB（バックアップ時間を約35%短縮）
```

**プロジェクトタイプ別の検出パターン:**

| プロジェクトタイプ | マーカーファイル | 自動除外パターン |
|------------------|---------------|----------------|
| **Rust** | `Cargo.toml` | `target/`, `.cache/` |
| **Node.js** | `package.json` | `node_modules/`, `.cache/`, `dist/`, `build/` |
| **Python** | `requirements.txt` | `__pycache__/`, `.venv/`, `.pytest_cache/` |
| **Git管理** | `.git/` | `.git/` (履歴は除外) |

**ベストプラクティス:**

1. **初回は `--dry-run` で確認**: 設定内容を確認してから適用
   ```bash
   backup-suite smart auto-configure ~/projects --dry-run
   ```

2. **対話モードで細かく制御**: 重要なプロジェクトは対話モードで確認
   ```bash
   backup-suite smart auto-configure ~/projects --interactive
   ```

3. **深度を調整**: サブプロジェクトが多い場合は深度を増やす
   ```bash
   backup-suite smart auto-configure ~/projects --max-depth 2
   ```

4. **除外パターンの確認**: 設定後は `backup-suite list` で除外パターンを確認
   ```bash
   backup-suite list
   ```

---

### `open` - バックアップディレクトリを開く

#### 基本構文
```bash
backup-suite open
```

#### 使用例と出力
```bash
$ backup-suite open
📂 開く: "/Users/user/backup-suite/backups"
# macOSではFinderでディレクトリが開かれる
```

---

### `--version` - バージョン情報

#### 基本構文
```bash
backup-suite --version
```

#### 使用例と出力
```bash
$ backup-suite --version
Backup Suite v1.0.0
🦀 Rust・高速・型安全
```

---

### `--lang` - 言語設定

#### 基本構文
```bash
backup-suite --lang <LANGUAGE> [COMMAND]
```

#### サポート言語
- `en` / `english` - 英語（デフォルト）
- `ja` / `japanese` / `日本語` - 日本語

#### 使用例
```bash
# 英語でヘルプ表示（デフォルト）
backup-suite --help
backup-suite --lang en --help

# 日本語でヘルプ表示
backup-suite --lang ja --help

# 日本語でステータス表示
backup-suite --lang ja status

# 英語でバックアップ実行
backup-suite --lang en run --priority high
```

#### 実行例と出力
```bash
$ backup-suite --lang en --help
Backup Suite v1.0.0
Fast Local Backup Tool - Written in Rust, Type-safe, High-performance
...

$ backup-suite --lang ja --help
Backup Suite v1.0.0
高速ローカルバックアップツール - Rust製・型安全・高性能
...
```

**注意**:
- デフォルト言語は英語です
- 環境変数 `LANG` は無視されます
- 全てのコマンドで `--lang` フラグが使用できます

---

### `completion` - シェル補完生成

#### 基本構文
```bash
backup-suite completion <SHELL>
```

#### サポートシェル
- `zsh`
- `bash`
- `fish`

#### 使用例
```bash
# Zsh補完生成
backup-suite completion zsh > ~/.local/share/zsh/site-functions/_backup-suite

# Bash補完生成
backup-suite completion bash > ~/.local/share/bash-completion/completions/backup-suite

# Fish補完生成
backup-suite completion fish > ~/.config/fish/completions/backup-suite.fish
```

---

## 🎯 実践的ワークフロー

### 開発者向けワークフロー

```bash
# 1. 開発プロジェクトを高優先度で追加
backup-suite add ~/projects/current-project --priority high --category development

# 2. 完了プロジェクトを中優先度に移行
backup-suite remove ~/projects/current-project
backup-suite add ~/projects/current-project --priority medium --category development

# 3. 古いプロジェクトを低優先度でアーカイブ
backup-suite add ~/projects/old-project --priority low --category archive

# 4. 日次の高優先度バックアップを自動化
backup-suite schedule setup --high daily
backup-suite schedule enable --priority high

# 5. 定期的な履歴確認
backup-suite dashboard
backup-suite history --days 7
```

### フォトグラファー向けワークフロー

```bash
# 1. 現在の撮影セッションを高優先度で管理
backup-suite add ~/Photos/2025/current-session --priority high --category creative

# 2. 編集完了写真を中優先度で保存
backup-suite add ~/Photos/2025/edited --priority medium --category creative

# 3. 古い写真をアーカイブ
backup-suite add ~/Photos/2023 --priority low --category archive

# 4. 週次の創作バックアップ設定
backup-suite schedule setup --high daily --medium weekly --low monthly
backup-suite schedule enable

# 5. ストレージ管理
backup-suite cleanup --days 90  # 3ヶ月以上古いバックアップを削除
```

### チーム開発向けワークフロー

```bash
# 1. プロジェクト別管理
backup-suite add ~/team-projects/project-alpha --priority high --category team-alpha
backup-suite add ~/team-projects/project-beta --priority medium --category team-beta

# 2. 個人作業領域
backup-suite add ~/workspace --priority high --category personal-work

# 3. ドキュメント・設定
backup-suite add ~/.config --priority medium --category config
backup-suite add ~/Documents/team-docs --priority medium --category documentation

# 4. 自動化設定
backup-suite schedule setup --high daily --medium weekly
backup-suite schedule enable

# 5. 定期的な状態確認
backup-suite status
backup-suite history --days 3
```

### 災害復旧ワークフロー

```bash
# 1. 緊急時の最新データ確認
backup-suite history --days 1

# 2. 重要データの優先復元
backup-suite restore --from latest --to ~/emergency-recovery

# 3. 特定プロジェクトの復元
backup-suite restore --from backup-20251104 --to ~/project-recovery

# 4. 復元後の確認
ls -la ~/emergency-recovery
diff -r ~/original-data ~/emergency-recovery

# 5. 新環境での設定復元
backup-suite add ~/emergency-recovery --priority high --category recovery
backup-suite run --priority high
```

## ⚙️ 設定ファイル詳細

### 設定ファイル場所
- **パス**: `~/.config/backup-suite/config.toml`
- **フォーマット**: TOML
- **エンコーディング**: UTF-8

### 設定ファイル構造

#### 完全な設定例
```toml
version = "1.0.0"

[backup]
destination = "/Users/user/backup-suite/backups"
auto_cleanup = true
keep_days = 30

[schedule]
enabled = true
high_frequency = "daily"
medium_frequency = "weekly"
low_frequency = "monthly"

# バックアップ対象（複数指定可能）
[[targets]]
path = "/Users/user/Documents/critical-project"
priority = "high"
target_type = "directory"
category = "development"
added_date = "2025-11-04T12:45:18.998137Z"
exclude_patterns = ["node_modules", ".git", "*.log"]

[[targets]]
path = "/Users/user/Photos/2025"
priority = "medium"
target_type = "directory"
category = "creative"
added_date = "2025-11-04T13:20:45.123456Z"
exclude_patterns = ["*.tmp", "cache/"]

[[targets]]
path = "/Users/user/.zshrc"
priority = "high"
target_type = "file"
category = "config"
added_date = "2025-11-04T14:10:22.789012Z"
exclude_patterns = []
```

#### セクション別説明

##### `[backup]` セクション
```toml
[backup]
destination = "/path/to/backup/directory"  # バックアップ保存先
auto_cleanup = true                        # 自動クリーンアップ有効
keep_days = 30                            # 保存日数
```

##### `[schedule]` セクション
```toml
[schedule]
enabled = true              # スケジューリング機能有効
high_frequency = "daily"    # 高優先度の実行頻度
medium_frequency = "weekly" # 中優先度の実行頻度
low_frequency = "monthly"   # 低優先度の実行頻度
```

##### `[[targets]]` セクション（配列）
```toml
[[targets]]
path = "/absolute/path/to/target"           # バックアップ対象パス（絶対パス）
priority = "high"                           # 優先度（high/medium/low）
target_type = "directory"                   # タイプ（file/directory）
category = "development"                    # カテゴリ
added_date = "2025-11-04T12:45:18.998137Z" # 追加日時（ISO 8601）
exclude_patterns = ["*.log", "cache/"]     # 除外パターン（glob形式）
```

### 設定ファイルのカスタマイズ

#### バックアップ先の変更
```toml
[backup]
destination = "/Volumes/External/backups"  # 外部ドライブ
# または
destination = "/nas/backups"               # NAS
# または
destination = "~/custom-backup-location"   # ホームディレクトリ相対
```

#### スケジュール頻度のカスタマイズ
```toml
[schedule]
high_frequency = "daily"     # 毎日2:00AM
medium_frequency = "weekly"  # 毎週日曜2:00AM
low_frequency = "monthly"    # 毎月1日2:00AM
# 将来対応予定:
# high_frequency = "hourly"  # 毎時（テスト用）
```

#### 除外パターンの設定
```toml
[[targets]]
path = "/Users/user/project"
exclude_patterns = [
    "node_modules",          # Node.js依存関係
    ".git",                  # Git履歴
    "*.log",                 # ログファイル
    "cache/",                # キャッシュディレクトリ
    ".DS_Store",             # macOS システムファイル
    "*.tmp",                 # 一時ファイル
    "build/",                # ビルド成果物
    "dist/"                  # 配布用ビルド
]
```

### 設定ファイルの管理

#### バックアップ設定のバックアップ
```bash
# 設定ファイル自体をバックアップ対象に追加
backup-suite add ~/.config/backup-suite/config.toml --priority high --category config

# 手動バックアップ
cp ~/.config/backup-suite/config.toml ~/.config/backup-suite/config.toml.backup
```

#### 設定の検証
```bash
# 設定内容確認
backup-suite status

# 対象一覧確認
backup-suite list

# 設定ファイル直接確認
cat ~/.config/backup-suite/config.toml
```

#### 設定の移行
```bash
# 設定ファイルをコピー（他のマシンから）
scp remote-machine:~/.config/backup-suite/config.toml ~/.config/backup-suite/

# 設定の一部変更
# パスの更新が必要な場合は手動編集
nano ~/.config/backup-suite/config.toml
```

## 🚀 高度な使用方法

### バッチ処理・スクリプト化

#### プロジェクト一括追加スクリプト
```bash
#!/bin/bash
# add-projects.sh

PROJECT_DIRS=(
    "$HOME/projects/active/project1"
    "$HOME/projects/active/project2"
    "$HOME/projects/active/project3"
)

for project in "${PROJECT_DIRS[@]}"; do
    if [[ -d "$project" ]]; then
        echo "Adding: $project"
        backup-suite add "$project" --priority high --category development
    else
        echo "Warning: $project not found"
    fi
done

echo "Projects added successfully"
backup-suite list --priority high
```

#### 定期メンテナンススクリプト
```bash
#!/bin/bash
# maintenance.sh

echo "=== Backup Suite Maintenance ==="

# 1. 状態確認
echo "Current status:"
backup-suite status

# 2. 古いバックアップ削除
echo "Cleaning up old backups..."
backup-suite cleanup --days 30

# 3. 最近の履歴確認
echo "Recent history:"
backup-suite history --days 3

# 4. ダッシュボード表示
echo "Dashboard:"
backup-suite dashboard

echo "Maintenance completed"
```

### 環境変数による設定

#### 一時的な設定変更
```bash
# 一時的に別のバックアップ先を使用
BACKUP_DESTINATION="/tmp/test-backup" backup-suite run --dry-run

# デバッグモード有効化
RUST_LOG=debug backup-suite status

# カラー出力無効化
NO_COLOR=1 backup-suite list
```

### CI/CD 統合

#### GitHub Actions での使用例
```yaml
name: Backup Important Files
on:
  schedule:
    - cron: '0 2 * * *'  # 毎日2:00AM UTC
  workflow_dispatch:

jobs:
  backup:
    runs-on: macos-latest
    steps:
      - name: Setup Backup Suite
        run: |
          curl -L https://github.com/user/backup-suite/releases/latest/download/backup-suite-macos-x86_64.tar.gz | tar xz
          chmod +x backup-suite
          sudo mv backup-suite /usr/local/bin/

      - name: Configure Targets
        run: |
          backup-suite add ${{ github.workspace }} --priority high --category ci

      - name: Run Backup
        run: |
          backup-suite run --priority high

      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: backup-results
          path: ~/backup-suite/backups/
```

### 外部ツールとの連携

#### rsync との組み合わせ
```bash
#!/bin/bash
# backup-and-sync.sh

# 1. ローカルバックアップ実行
backup-suite run --priority high

# 2. 最新バックアップをリモートに同期
LATEST_BACKUP=$(ls -t ~/backup-suite/backups/ | head -1)
rsync -avz ~/backup-suite/backups/"$LATEST_BACKUP"/ remote-server:/backup/

echo "Local backup and remote sync completed"
```

#### Git連携
```bash
#!/bin/bash
# git-backup-hook.sh
# Git post-commit hook として使用

# コミット後に自動的にプロジェクトをバックアップ
PROJECT_PATH=$(git rev-parse --show-toplevel)

# 既存の対象でない場合は追加
if ! backup-suite list | grep -q "$PROJECT_PATH"; then
    backup-suite add "$PROJECT_PATH" --priority high --category development
fi

# バックアップ実行
backup-suite run --priority high
```

## 💡 ベストプラクティス

### 優先度設定のガイドライン

#### `high` 優先度の適切な使用
```bash
# ✅ 適切
backup-suite add ~/current-work-project --priority high --category development
backup-suite add ~/.ssh --priority high --category security
backup-suite add ~/Documents/contracts --priority high --category legal

# ❌ 避けるべき
backup-suite add ~/Downloads --priority high  # 一時ファイルは低優先度
backup-suite add ~/Music --priority high      # エンターテイメントは中〜低優先度
```

#### `medium` 優先度の適切な使用
```bash
# ✅ 適切
backup-suite add ~/Photos/2025 --priority medium --category personal
backup-suite add ~/Documents/references --priority medium --category reference
backup-suite add ~/.config --priority medium --category config
```

#### `low` 優先度の適切な使用
```bash
# ✅ 適切
backup-suite add ~/Archive/old-projects --priority low --category archive
backup-suite add ~/Downloads --priority low --category temp
backup-suite add ~/Desktop/old-files --priority low --category cleanup
```

### 除外パターンのベストプラクティス

#### 開発プロジェクト
```toml
[[targets]]
path = "/Users/user/projects/web-app"
exclude_patterns = [
    "node_modules",      # NPM依存関係
    ".git",             # Git履歴（大容量）
    "build",            # ビルド成果物
    "dist",             # 配布用ビルド
    "*.log",            # ログファイル
    ".env",             # 環境変数（機密情報）
    "coverage",         # テストカバレッジ
    ".nyc_output"       # カバレッジ一時ファイル
]
```

#### 創作・デザインプロジェクト
```toml
[[targets]]
path = "/Users/user/creative/video-project"
exclude_patterns = [
    "*.tmp",            # 一時ファイル
    "cache",            # キャッシュディレクトリ
    "render",           # レンダリング一時ファイル
    "*.autosave",       # 自動保存ファイル
    ".DS_Store"         # macOS システムファイル
]
```

### スケジューリングのベストプラクティス

#### 推奨スケジュール設定
```bash
# バランスの取れた設定
backup-suite schedule setup --high daily --medium weekly --low monthly

# 高頻度設定（重要プロジェクト期間）
backup-suite schedule setup --high daily --medium daily --low weekly

# 低頻度設定（安定運用期間）
backup-suite schedule setup --high weekly --medium monthly --low monthly
```

#### システムリソース考慮
```bash
# 大量ファイルがある場合は頻度を下げる
backup-suite schedule setup --high weekly --medium monthly --low monthly

# 重要期間は高頻度
backup-suite schedule enable --priority high  # 高優先度のみ有効化
```

### ストレージ管理のベストプラクティス

#### 定期的なクリーンアップ
```bash
# 週次メンテナンス
backup-suite cleanup --days 7

# 月次メンテナンス
backup-suite cleanup --days 30

# 四半期メンテナンス
backup-suite cleanup --days 90
```

#### 容量監視
```bash
# バックアップディレクトリサイズ確認
du -sh ~/backup-suite/backups/

# 個別バックアップサイズ確認
ls -lah ~/backup-suite/backups/

# ディスク使用量確認
df -h ~/backup-suite/
```

### セキュリティのベストプラクティス

#### 機密ファイルの除外
```toml
[[targets]]
path = "/Users/user/projects"
exclude_patterns = [
    ".env",             # 環境変数
    "*.key",            # 秘密鍵
    "*.pem",            # 証明書
    "config/secrets",   # 機密設定
    "*.password",       # パスワードファイル
    "credentials.json"  # 認証情報
]
```

#### 設定ファイルの権限管理
```bash
# 設定ディレクトリの権限確認・修正
chmod 755 ~/.config/backup-suite/
chmod 644 ~/.config/backup-suite/config.toml

# バックアップディレクトリの権限確認
chmod 755 ~/backup-suite/
chmod 755 ~/backup-suite/backups/
```

### トラブル予防のベストプラクティス

#### 定期的な動作確認
```bash
# 月次チェックリスト
backup-suite status                    # 設定確認
backup-suite list                      # 対象確認
backup-suite run --dry-run             # ドライラン実行
backup-suite history --days 30         # 履歴確認
backup-suite dashboard                 # 統計確認
backup-suite schedule status           # スケジュール確認
```

#### バックアップの検証
```bash
# 最新バックアップの確認
LATEST=$(ls -t ~/backup-suite/backups/ | head -1)
ls -la ~/backup-suite/backups/"$LATEST"/

# ランダムファイルの整合性確認
diff ~/original-file ~/backup-suite/backups/"$LATEST"/original-file
```

#### 設定のバージョン管理
```bash
# 設定ファイルをGitで管理
cd ~/.config/backup-suite/
git init
git add config.toml
git commit -m "Initial backup-suite configuration"

# 変更時のコミット
git add config.toml
git commit -m "Update backup targets for new project"
```

---

## 📞 サポート・問い合わせ

使用方法で不明な点がある場合：

1. **GitHub Issues**: [質問・バグ報告](https://github.com/user/backup-suite/issues)
2. **Discussions**: [コミュニティ相談](https://github.com/user/backup-suite/discussions)
3. **Documentation**: [その他ドキュメント](../README.md#ドキュメント)

---

**次のステップ**: より技術的な詳細は [アーキテクチャドキュメント](../development/ARCHITECTURE.md) をご確認ください。