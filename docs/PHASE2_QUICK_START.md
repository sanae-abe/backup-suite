# Phase 2 機能クイックスタートガイド

## 📋 新機能概要

Phase 2で追加された主要機能：

1. **履歴管理の拡張** - 詳細情報、フィルタリング
2. **復元機能** - 暗号化・圧縮対応
3. **クリーンアップ機能** - 古いバックアップの自動削除
4. **除外パターン** - 不要ファイルの除外
5. **設定バリデーション** - 包括的な検証

---

## 🚀 クイックスタート

### 1. 除外パターンを使用したバックアップ

```bash
# node_modules と target を除外してバックアップ対象を追加
backup-suite add ~/projects \
  --priority high \
  --category development \
  --exclude "node_modules/" \
  --exclude "target/" \
  --exclude "*.log"

# バックアップ実行
backup-suite run --priority high
```

### 2. 暗号化・圧縮バックアップ

```bash
# AES-256-GCM暗号化 + Zstd圧縮
backup-suite run \
  --encrypt \
  --password "your-secure-password" \
  --compress zstd \
  --compress-level 3
```

### 3. 履歴の確認

```bash
# 過去7日間の履歴（デフォルト）
backup-suite history

# 詳細表示
backup-suite history --detailed

# 高優先度のみ
backup-suite history --priority high

# カテゴリフィルタ
backup-suite history --category development

# 過去30日間
backup-suite history --days 30
```

### 4. 復元

```bash
# 最新バックアップから復元
backup-suite restore --password "your-secure-password"

# 特定バックアップから復元
backup-suite restore --from backup-20251107 --to /tmp/restore

# ドライラン（復元対象を確認）
backup-suite restore --dry-run
```

### 5. クリーンアップ

```bash
# 30日以上前のバックアップを削除（ドライラン）
backup-suite cleanup --days 30 --dry-run

# 実際に削除
backup-suite cleanup --days 30
```

---

## 📊 履歴情報の詳細

### 標準表示
```
📜 バックアップ履歴（7日）

┌────────────────────┬────────────┬──────────┬─────────────┐
│ 日時               │ ファイル数 │ サイズ   │ ステータス  │
├────────────────────┼────────────┼──────────┼─────────────┤
│ 2025-11-07 12:00   │ 150        │ 1.2 GB   │ Success     │
│ 2025-11-06 12:00   │ 142        │ 1.1 GB   │ Success     │
└────────────────────┴────────────┴──────────┴─────────────┘
```

### 詳細表示（--detailed）
```
============================================================
🕒 ステータス: 2025-11-07 12:00:00
📁 パス: /Users/user/backup-suite/backups/backup_20251107_120000
🏷️  カテゴリ: development
⚡ 優先度: High
📊 ステータス: Success
📦 ファイル数: 150
💾 サイズ: 1024.00 MB
🗜️  圧縮: 有効
🔒 暗号化: 有効
⏱️  処理時間: 5.23秒
```

---

## 🔒 セキュリティ機能

### パストラバーサル対策
復元時に自動的に検出・ブロック：
```bash
# 攻撃パターンは自動的にブロックされる
# 例: ../../../etc/passwd
```

### 暗号化の詳細
- **アルゴリズム**: AES-256-GCM（認証付き暗号化）
- **鍵導出**: Argon2（メモリハード関数）
- **Nonce**: ランダム生成（再利用なし）

---

## 🧪 動作確認

### テスト実行
```bash
# Phase 2統合テスト
cargo test --test phase2_integration_tests

# 全テスト
cargo test

# リリースビルド
cargo build --release
```

### デモスクリプト
```bash
# Phase 2機能のデモ実行
./examples/phase2_usage.sh
```

---

## 📚 関連ドキュメント

- [PHASE2_IMPLEMENTATION.md](/Users/sanae.abe/projects/backup-suite/PHASE2_IMPLEMENTATION.md) - 完全な実装詳細
- [README.md](/Users/sanae.abe/projects/backup-suite/README.md) - プロジェクト概要
- [PUBLISHING.md](/Users/sanae.abe/projects/backup-suite/PUBLISHING.md) - リリース手順

---

## 💡 使用例

### シナリオ1: 開発プロジェクトのバックアップ
```bash
# ビルド成果物を除外してバックアップ
backup-suite add ~/my-project \
  --priority high \
  --category development \
  --exclude "node_modules/" \
  --exclude "target/" \
  --exclude "dist/" \
  --exclude ".next/"

# 暗号化バックアップ実行
backup-suite run --category development --encrypt --password "dev-backup-2025"
```

### シナリオ2: 定期的なクリーンアップ
```bash
# 週次: 90日以上前のバックアップを削除
backup-suite cleanup --days 90

# 月次: ディスク容量確認後にクリーンアップ
backup-suite status
backup-suite cleanup --days 60 --dry-run
backup-suite cleanup --days 60
```

### シナリオ3: 障害復旧
```bash
# 最新のバックアップから復元
backup-suite restore --password "dev-backup-2025"

# 特定の日付のバックアップから復元
backup-suite history --days 30
backup-suite restore --from backup-20251101 --password "dev-backup-2025"
```

---

## ⚙️ 設定例

### config.toml
```toml
[backup]
destination = "/Volumes/Backup/backup-suite"
auto_cleanup = true
keep_days = 90

[schedule]
enabled = true
high_frequency = "daily"
medium_frequency = "weekly"
low_frequency = "monthly"

[[targets]]
path = "/Users/user/projects"
priority = "high"
category = "development"
exclude_patterns = ["node_modules/", "target/", "*.log"]
added_date = 2025-11-07T12:00:00Z

[[targets]]
path = "/Users/user/Documents"
priority = "medium"
category = "documents"
exclude_patterns = ["~$*", "*.tmp"]
added_date = 2025-11-07T12:00:00Z
```

---

## 🎯 次のステップ

Phase 2実装完了後の推奨事項：

1. **実際のデータでテスト**: 本番データでバックアップ・復元を確認
2. **定期実行の設定**: launchd/systemdでスケジュール実行
3. **クリーンアップポリシーの調整**: ディスク容量に応じた設定
4. **セキュリティ監査**: `security-auditor` agentでレビュー
5. **パフォーマンス測定**: `performance-engineer` agentでベンチマーク

---

**更新日**: 2025-11-07
**バージョン**: backup-suite 1.0.0 (Phase 2)
