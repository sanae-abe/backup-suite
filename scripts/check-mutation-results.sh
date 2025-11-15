#!/usr/bin/env bash
# Mutation Testing 結果確認スクリプト

set -euo pipefail

echo "==================================="
echo "Mutation Testing Results Summary"
echo "==================================="
echo ""

# 各モジュールの結果を確認
check_module() {
    local module=$1
    local output_dir=$2

    echo "📊 $module"
    echo "-----------------------------------"

    if [ ! -d "$output_dir" ]; then
        echo "  Status: ⏳ Not started or in progress"
        echo ""
        return
    fi

    local results_dir="$output_dir/$output_dir"

    if [ ! -f "$results_dir/caught.txt" ]; then
        echo "  Status: ⏳ In progress (no results yet)"
        echo ""
        return
    fi

    local caught=$(wc -l < "$results_dir/caught.txt" 2>/dev/null || echo "0")
    local missed=$(wc -l < "$results_dir/missed.txt" 2>/dev/null || echo "0")
    local timeout=$(wc -l < "$results_dir/timeout.txt" 2>/dev/null || echo "0")
    local unviable=$(wc -l < "$results_dir/unviable.txt" 2>/dev/null || echo "0")

    local total=$((caught + missed + timeout + unviable))

    if [ "$total" -eq 0 ]; then
        echo "  Status: ⏳ In progress (no results yet)"
        echo ""
        return
    fi

    local score=0
    if [ "$total" -gt 0 ] && [ "$caught" -gt 0 ]; then
        score=$(awk "BEGIN {printf \"%.1f\", ($caught / $total) * 100}")
    fi

    echo "  Status: ✅ Completed"
    echo "  Total: $total"
    echo "  Caught: $caught"
    echo "  Missed: $missed"
    echo "  Timeout: $timeout"
    echo "  Unviable: $unviable"
    echo "  Mutation Score: $score%"
    echo ""

    if [ "$missed" -gt 0 ]; then
        echo "  ⚠️  Missed Mutants:"
        head -5 "$results_dir/missed.txt" 2>/dev/null | sed 's/^/    /'
        if [ "$(wc -l < "$results_dir/missed.txt")" -gt 5 ]; then
            echo "    ... and $(($(wc -l < "$results_dir/missed.txt") - 5)) more"
        fi
        echo ""
    fi
}

# 各モジュールをチェック
check_module "encryption.rs" "mutants.out"
check_module "path.rs (security)" "mutants-path.out"
check_module "key_management.rs (crypto)" "mutants-key.out"

echo "==================================="
echo "Overall Summary"
echo "==================================="

# 統合スコアを計算
total_caught=0
total_total=0

for dir in mutants.out mutants-path.out mutants-key.out; do
    if [ -d "$dir/$dir" ]; then
        caught=$(wc -l < "$dir/$dir/caught.txt" 2>/dev/null || echo "0")
        missed=$(wc -l < "$dir/$dir/missed.txt" 2>/dev/null || echo "0")
        timeout=$(wc -l < "$dir/$dir/timeout.txt" 2>/dev/null || echo "0")
        unviable=$(wc -l < "$dir/$dir/unviable.txt" 2>/dev/null || echo "0")

        total=$((caught + missed + timeout + unviable))
        total_caught=$((total_caught + caught))
        total_total=$((total_total + total))
    fi
done

if [ "$total_total" -gt 0 ]; then
    overall_score=$(awk "BEGIN {printf \"%.1f\", ($total_caught / $total_total) * 100}")
    echo "Total Mutants: $total_total"
    echo "Total Caught: $total_caught"
    echo "Overall Mutation Score: $overall_score%"
else
    echo "No results available yet."
fi

echo ""
echo "==================================="
echo "Logs:"
echo "  - mutation-testing.log (encryption.rs)"
echo "  - mutation-testing-path.log (path.rs)"
echo "  - mutation-testing-key.log (key_management.rs)"
echo "==================================="
