#!/bin/bash
# backup-suite デモGIF作成スクリプト

set -e

DEMO_DIR="docs/demos"
CAST_FILE="$DEMO_DIR/backup-suite-demo.cast"
GIF_FILE="$DEMO_DIR/backup-suite-demo.gif"

mkdir -p "$DEMO_DIR"

echo "🎬 backup-suite デモ録画スクリプト"
echo ""
echo "使い方:"
echo "  1. このスクリプトを実行"
echo "  2. 録画が開始されたら、backup-suite コマンドを実演"
echo "  3. 'exit' で録画終了"
echo "  4. 自動的にGIFが生成されます"
echo ""
echo "📝 推奨デモ実演内容:"
echo "  backup-suite help              # ヘルプ表示"
echo "  backup-suite add ~/Documents --priority critical"
echo "  backup-suite list              # 一覧表示"
echo "  backup-suite stats             # 統計表示"
echo ""
read -p "録画を開始しますか? (y/n) " -n 1 -r
echo

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "キャンセルしました"
    exit 0
fi

# 録画開始
echo "🔴 録画開始... (終了するには 'exit' を入力)"
asciinema rec "$CAST_FILE"

# GIF生成
echo ""
echo "🎨 GIF生成中..."
agg \
  --fps 15 \
  --speed 1.0 \
  --theme monokai \
  --font-size 14 \
  "$CAST_FILE" \
  "$GIF_FILE"

echo ""
echo "✅ デモGIF作成完了！"
echo "📁 保存場所: $GIF_FILE"
echo ""
echo "📊 ファイルサイズ:"
ls -lh "$GIF_FILE" | awk '{print $5}'
echo ""
echo "💡 次のステップ:"
echo "  README.mdの「スクリーンショット」セクションに以下を追加:"
echo ""
echo "### デモ動画"
echo "![Demo](./docs/demos/backup-suite-demo.gif)"
echo ""
echo "*基本的な使い方のデモンストレーション*"
