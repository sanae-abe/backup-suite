#!/bin/bash

# Backup Suite 品質チェックスクリプト
# 全品質指標を包括的にチェックし、レポートを生成

set -euo pipefail

# カラー定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ロガー関数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

log_section() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

# エラーカウンター
ERRORS=0
WARNINGS=0

# プロジェクトルートディレクトリに移動
cd "$(dirname "$0")/.."

log_section "Backup Suite 品質チェック"
log_info "実行日時: $(date)"
log_info "Git ブランチ: $(git branch --show-current 2>/dev/null || echo 'N/A')"
log_info "Git コミット: $(git rev-parse --short HEAD 2>/dev/null || echo 'N/A')"

# 1. コードフォーマットチェック
log_section "1. コードフォーマットチェック (rustfmt)"
if cargo fmt -- --check; then
    log_success "フォーマットチェック通過"
else
    log_error "フォーマットエラーが見つかりました"
    log_info "修正方法: cargo fmt"
    ERRORS=$((ERRORS + 1))
fi

# 2. Clippy リント
log_section "2. Clippy リントチェック"
if cargo clippy --all-targets --all-features -- -D warnings \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
    -W clippy::panic \
    -W clippy::unimplemented \
    -W clippy::todo \
    -W clippy::unreachable \
    -W clippy::indexing_slicing; then
    log_success "Clippy チェック通過"
else
    log_error "Clippy 警告/エラーが見つかりました"
    log_info "修正方法: cargo clippy --fix"
    ERRORS=$((ERRORS + 1))
fi

# 3. ユニットテスト
log_section "3. ユニットテスト"
if cargo test --lib --verbose; then
    log_success "ユニットテスト通過"
else
    log_error "ユニットテストが失敗しました"
    ERRORS=$((ERRORS + 1))
fi

# 4. 統合テスト
log_section "4. 統合テスト"
if cargo test --test '*' --verbose; then
    log_success "統合テスト通過"
else
    log_error "統合テストが失敗しました"
    ERRORS=$((ERRORS + 1))
fi

# 5. ドキュメントテスト
log_section "5. ドキュメントテスト"
if cargo test --doc; then
    log_success "ドキュメントテスト通過"
else
    log_error "ドキュメントテストが失敗しました"
    ERRORS=$((ERRORS + 1))
fi

# 6. ビルドチェック
log_section "6. ビルドチェック"
log_info "デバッグビルド..."
if cargo build --verbose; then
    log_success "デバッグビルド成功"
else
    log_error "デバッグビルド失敗"
    ERRORS=$((ERRORS + 1))
fi

log_info "リリースビルド..."
if cargo build --release --verbose; then
    log_success "リリースビルド成功"
else
    log_error "リリースビルド失敗"
    ERRORS=$((ERRORS + 1))
fi

# 7. セキュリティ監査
log_section "7. セキュリティ監査"

# cargo-audit のインストール確認
if ! command -v cargo-audit &> /dev/null; then
    log_warning "cargo-audit がインストールされていません"
    log_info "インストール方法: cargo install cargo-audit"
    WARNINGS=$((WARNINGS + 1))
else
    if cargo audit; then
        log_success "セキュリティ監査通過（脆弱性なし）"
    else
        log_error "セキュリティ脆弱性が検出されました"
        ERRORS=$((ERRORS + 1))
    fi
fi

# cargo-deny のインストール確認
if ! command -v cargo-deny &> /dev/null; then
    log_warning "cargo-deny がインストールされていません"
    log_info "インストール方法: cargo install cargo-deny"
    WARNINGS=$((WARNINGS + 1))
else
    if cargo deny check; then
        log_success "依存関係チェック通過"
    else
        log_error "依存関係に問題があります"
        ERRORS=$((ERRORS + 1))
    fi
fi

# 8. テストカバレッジ
log_section "8. テストカバレッジ"

if ! command -v cargo-tarpaulin &> /dev/null; then
    log_warning "cargo-tarpaulin がインストールされていません"
    log_info "インストール方法: cargo install cargo-tarpaulin"
    log_info "カバレッジチェックをスキップします"
    WARNINGS=$((WARNINGS + 1))
else
    log_info "カバレッジ測定中（時間がかかる場合があります）..."
    if cargo tarpaulin --verbose --all-features --workspace --timeout 300 --out Xml --output-dir target/coverage; then
        # カバレッジ率を抽出
        if [ -f target/coverage/cobertura.xml ]; then
            COVERAGE=$(grep -oP 'line-rate="\K[0-9.]+' target/coverage/cobertura.xml | head -1)
            COVERAGE_PERCENT=$(echo "$COVERAGE * 100" | bc)
            log_info "テストカバレッジ: ${COVERAGE_PERCENT}%"

            # 80%以上を推奨
            THRESHOLD=80
            if (( $(echo "$COVERAGE_PERCENT >= $THRESHOLD" | bc -l) )); then
                log_success "カバレッジ目標達成: ${COVERAGE_PERCENT}% >= ${THRESHOLD}%"
            else
                log_warning "カバレッジが目標未達: ${COVERAGE_PERCENT}% < ${THRESHOLD}%"
                WARNINGS=$((WARNINGS + 1))
            fi
        fi
    else
        log_error "カバレッジ測定に失敗しました"
        ERRORS=$((ERRORS + 1))
    fi
fi

# 9. ドキュメント生成
log_section "9. ドキュメント生成"
if cargo doc --no-deps --all-features; then
    log_success "ドキュメント生成成功"
else
    log_error "ドキュメント生成失敗"
    ERRORS=$((ERRORS + 1))
fi

# 10. ベンチマーク（オプション）
log_section "10. ベンチマーク（オプション）"
if [ "${RUN_BENCHMARKS:-false}" = "true" ]; then
    log_info "ベンチマーク実行中..."
    if cargo bench --no-run; then
        log_success "ベンチマークビルド成功"
    else
        log_error "ベンチマークビルド失敗"
        ERRORS=$((ERRORS + 1))
    fi
else
    log_info "ベンチマークをスキップ（RUN_BENCHMARKS=true で実行）"
fi

# 11. コードメトリクス
log_section "11. コードメトリクス"

# 行数カウント
log_info "コード行数:"
echo "  Rustコード: $(find src -name "*.rs" | xargs wc -l | tail -1 | awk '{print $1}') 行"
echo "  テストコード: $(find tests -name "*.rs" 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}' || echo 0) 行"
echo "  ベンチマーク: $(find benches -name "*.rs" 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}' || echo 0) 行"

# TODOカウント
TODO_COUNT=$(grep -r "TODO\|FIXME\|XXX\|HACK" src tests 2>/dev/null | wc -l || echo 0)
if [ "$TODO_COUNT" -gt 0 ]; then
    log_warning "TODO/FIXME: ${TODO_COUNT} 件"
    echo "  詳細: grep -rn 'TODO\|FIXME\|XXX\|HACK' src tests"
    WARNINGS=$((WARNINGS + 1))
else
    log_success "TODO/FIXMEなし"
fi

# unsafeコードブロック
UNSAFE_COUNT=$(grep -r "unsafe" src --include="*.rs" | grep -v "// SAFETY:" | wc -l || echo 0)
if [ "$UNSAFE_COUNT" -gt 0 ]; then
    log_warning "ドキュメント化されていないunsafeコード: ${UNSAFE_COUNT} 箇所"
    echo "  詳細: grep -rn 'unsafe' src --include='*.rs' | grep -v '// SAFETY:'"
    WARNINGS=$((WARNINGS + 1))
else
    log_success "すべてのunsafeコードがドキュメント化されています"
fi

# 12. 依存関係チェック
log_section "12. 依存関係チェック"

# 未使用依存関係のチェック
if command -v cargo-udeps &> /dev/null; then
    if cargo +nightly udeps; then
        log_success "未使用依存関係なし"
    else
        log_warning "未使用依存関係が検出されました"
        WARNINGS=$((WARNINGS + 1))
    fi
else
    log_info "cargo-udeps がインストールされていません（オプション）"
    log_info "インストール方法: cargo install cargo-udeps"
fi

# 13. パフォーマンスチェック
log_section "13. パフォーマンスチェック"

if [ -f "target/release/backup-suite" ]; then
    # バイナリサイズ
    BINARY_SIZE=$(du -h target/release/backup-suite | cut -f1)
    log_info "バイナリサイズ: ${BINARY_SIZE}"

    # strip後のサイズ
    if command -v strip &> /dev/null; then
        cp target/release/backup-suite target/release/backup-suite.stripped
        strip target/release/backup-suite.stripped
        STRIPPED_SIZE=$(du -h target/release/backup-suite.stripped | cut -f1)
        log_info "Strip後サイズ: ${STRIPPED_SIZE}"
        rm target/release/backup-suite.stripped
    fi
else
    log_info "リリースビルドが見つかりません"
fi

# サマリーレポート生成
log_section "品質チェックサマリー"

TOTAL_CHECKS=$((ERRORS + WARNINGS))

echo ""
echo "┌────────────────────────────────────────────┐"
echo "│         品質チェック結果サマリー            │"
echo "├────────────────────────────────────────────┤"
printf "│ %-20s %20s │\n" "エラー:" "${ERRORS} 件"
printf "│ %-20s %20s │\n" "警告:" "${WARNINGS} 件"
echo "└────────────────────────────────────────────┘"
echo ""

# 品質スコア計算（10段階）
if [ "$ERRORS" -eq 0 ] && [ "$WARNINGS" -eq 0 ]; then
    QUALITY_SCORE=10
    log_success "品質スコア: ${QUALITY_SCORE}/10 (完璧！)"
elif [ "$ERRORS" -eq 0 ]; then
    QUALITY_SCORE=$((10 - WARNINGS / 2))
    [ "$QUALITY_SCORE" -lt 7 ] && QUALITY_SCORE=7
    log_success "品質スコア: ${QUALITY_SCORE}/10 (良好)"
else
    QUALITY_SCORE=$((10 - ERRORS - WARNINGS / 2))
    [ "$QUALITY_SCORE" -lt 0 ] && QUALITY_SCORE=0
    log_warning "品質スコア: ${QUALITY_SCORE}/10 (要改善)"
fi

# レポートファイル生成
REPORT_FILE="target/quality-report.txt"
mkdir -p target
{
    echo "========================================"
    echo "Backup Suite 品質チェックレポート"
    echo "========================================"
    echo ""
    echo "実行日時: $(date)"
    echo "Git ブランチ: $(git branch --show-current 2>/dev/null || echo 'N/A')"
    echo "Git コミット: $(git rev-parse --short HEAD 2>/dev/null || echo 'N/A')"
    echo ""
    echo "結果サマリー:"
    echo "  エラー: ${ERRORS} 件"
    echo "  警告: ${WARNINGS} 件"
    echo "  品質スコア: ${QUALITY_SCORE}/10"
    echo ""
} > "$REPORT_FILE"

log_info "詳細レポート: ${REPORT_FILE}"

# 終了コード
if [ "$ERRORS" -gt 0 ]; then
    log_error "品質チェックに失敗しました"
    exit 1
else
    log_success "品質チェック完了"
    exit 0
fi
