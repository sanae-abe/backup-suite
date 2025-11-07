#!/bin/bash
# 増分バックアップ機能のデモスクリプト

set -e

# 一時ディレクトリ作成
DEMO_DIR=$(mktemp -d)
SOURCE_DIR="$DEMO_DIR/source"
BACKUP_DIR="$DEMO_DIR/backups"

mkdir -p "$SOURCE_DIR"

echo "📁 デモディレクトリ: $DEMO_DIR"
echo ""

# 初期ファイル作成
echo "1️⃣ 初期ファイルを作成"
echo "content1" > "$SOURCE_DIR/file1.txt"
echo "content2" > "$SOURCE_DIR/file2.txt"
echo "content3" > "$SOURCE_DIR/file3.txt"
ls -lh "$SOURCE_DIR"
echo ""

# 1回目: フルバックアップ
echo "2️⃣ 1回目: フルバックアップ（--incremental指定でも初回は自動的にフルバックアップ）"
cargo run -- run --dry-run --incremental 2>&1 | grep -E "(フルバックアップ|ファイル)"
# 実際のバックアップ実行（ドライランなしで実行する場合）
# cargo run -- run --incremental
echo ""
sleep 2

# ファイル変更
echo "3️⃣ file1.txtを変更、file4.txtを新規作成"
echo "modified content1" > "$SOURCE_DIR/file1.txt"
echo "new file" > "$SOURCE_DIR/file4.txt"
ls -lh "$SOURCE_DIR"
echo ""
sleep 2

# 2回目: 増分バックアップ
echo "4️⃣ 2回目: 増分バックアップ（変更されたfile1と新規file4のみバックアップ）"
cargo run -- run --dry-run --incremental 2>&1 | grep -E "(増分バックアップ|変更ファイル|ファイル)"
echo ""

# さらにファイル変更
echo "5️⃣ file2.txtを変更"
echo "modified content2" > "$SOURCE_DIR/file2.txt"
sleep 2

# 3回目: 増分バックアップ
echo "6️⃣ 3回目: 増分バックアップ（変更されたfile2のみバックアップ）"
cargo run -- run --dry-run --incremental 2>&1 | grep -E "(増分バックアップ|変更ファイル|ファイル)"
echo ""

# 復元デモ
echo "7️⃣ 最新のバックアップから復元（増分チェーンを自動解決）"
echo "   フルバックアップ → 増分1 → 増分2 の順に適用し、最新状態を復元"
# cargo run -- restore --from <backup_name> --to ./restore
echo ""

# クリーンアップ
echo "🧹 デモディレクトリをクリーンアップ"
rm -rf "$DEMO_DIR"
echo "✅ デモ完了"
