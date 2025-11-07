#!/bin/bash

# backup-suite Phase 2 機能の使用例

set -e

echo "========================================="
echo "backup-suite Phase 2 機能デモ"
echo "========================================="
echo ""

# カラー定義
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# 1. 除外パターンを使用したバックアップ対象の追加
echo -e "${GREEN}1. 除外パターンを使用したバックアップ対象の追加${NC}"
echo "   除外パターン: node_modules/, target/, *.log"
echo ""

cargo run --release -- add ~/projects \
  --priority high \
  --category "development" \
  --exclude "node_modules/" \
  --exclude "target/" \
  --exclude "*.log"

echo ""
echo -e "${GREEN}追加完了${NC}"
echo ""

# 2. バックアップの実行（暗号化・圧縮）
echo -e "${GREEN}2. バックアップの実行（暗号化・圧縮）${NC}"
echo ""

cargo run --release -- run \
  --priority high \
  --category development \
  --encrypt \
  --password "demo123" \
  --compress zstd \
  --compress-level 3

echo ""
echo -e "${GREEN}バックアップ完了${NC}"
echo ""

# 3. 履歴の確認
echo -e "${GREEN}3. 履歴の確認${NC}"
echo ""

echo -e "${YELLOW}3-1. 全履歴表示${NC}"
cargo run --release -- history --days 30

echo ""
echo -e "${YELLOW}3-2. 詳細表示${NC}"
cargo run --release -- history --days 7 --detailed

echo ""
echo -e "${YELLOW}3-3. 高優先度のみフィルタ${NC}"
cargo run --release -- history --priority high

echo ""
echo -e "${YELLOW}3-4. カテゴリフィルタ${NC}"
cargo run --release -- history --category development

echo ""

# 4. クリーンアップのドライラン
echo -e "${GREEN}4. クリーンアップのドライラン（削除対象確認）${NC}"
echo ""

cargo run --release -- cleanup --days 30 --dry-run

echo ""

# 5. 復元のデモ
echo -e "${GREEN}5. 復元のデモ${NC}"
echo ""

echo -e "${YELLOW}5-1. 利用可能なバックアップ一覧${NC}"
# 実装されている場合
# cargo run --release -- list-backups

echo -e "${YELLOW}5-2. 最新バックアップから復元（ドライラン）${NC}"
# 復元先を /tmp/restore に指定
mkdir -p /tmp/restore

# ドライランで復元対象を確認
# cargo run --release -- restore --to /tmp/restore --password "demo123" --dry-run

echo ""

# 6. ステータス確認
echo -e "${GREEN}6. 現在の設定とステータス${NC}"
echo ""

cargo run --release -- status

echo ""
echo "========================================="
echo "Phase 2 機能デモ完了"
echo "========================================="
echo ""
echo "実装済み機能:"
echo "  ✅ 除外パターン指定（--exclude）"
echo "  ✅ 履歴の詳細表示（--detailed）"
echo "  ✅ 優先度・カテゴリフィルタ"
echo "  ✅ クリーンアップ機能（保持期間）"
echo "  ✅ 復元機能（暗号化・圧縮対応）"
echo ""
echo "使用可能なコマンド:"
echo "  backup-suite add [PATH] --exclude [PATTERN] ..."
echo "  backup-suite history --days [DAYS] --detailed"
echo "  backup-suite history --priority [high|medium|low]"
echo "  backup-suite history --category [CATEGORY]"
echo "  backup-suite cleanup --days [DAYS] [--dry-run]"
echo "  backup-suite restore --from [BACKUP] --to [DEST] --password [PASS]"
echo ""
