#!/bin/bash

# リリース準備スクリプト
# バージョンアップ、CHANGELOG更新、タグ作成を自動化

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

# 使い方表示
usage() {
    cat << EOF
使い方: $0 <version> [options]

引数:
  version     新しいバージョン番号（例: 1.2.0）

オプション:
  -h, --help      このヘルプを表示
  -d, --dry-run   実際の変更を行わず、プレビューのみ
  -f, --force     確認なしで実行

例:
  $0 1.2.0
  $0 2.0.0 --dry-run
  $0 1.1.1 --force

EOF
    exit 1
}

# 引数パース
if [ $# -eq 0 ]; then
    usage
fi

NEW_VERSION=""
DRY_RUN=false
FORCE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        *)
            if [ -z "$NEW_VERSION" ]; then
                NEW_VERSION="$1"
            else
                log_error "不明な引数: $1"
                usage
            fi
            shift
            ;;
    esac
done

# バージョン検証
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    log_error "不正なバージョン形式: $NEW_VERSION"
    log_info "正しい形式: X.Y.Z（例: 1.2.0）"
    exit 1
fi

# プロジェクトルートに移動
cd "$(dirname "$0")/.."

log_info "リリース準備を開始します: v${NEW_VERSION}"

# 現在のバージョンを取得
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
log_info "現在のバージョン: ${CURRENT_VERSION}"
log_info "新しいバージョン: ${NEW_VERSION}"

# バージョン比較
if [ "$CURRENT_VERSION" = "$NEW_VERSION" ]; then
    log_error "新しいバージョンが現在のバージョンと同じです"
    exit 1
fi

# Gitステータスチェック
if ! $FORCE; then
    if [ -n "$(git status --porcelain)" ]; then
        log_error "Gitワーキングディレクトリに未コミットの変更があります"
        log_info "変更をコミットするか、--force オプションを使用してください"
        exit 1
    fi
fi

# ブランチチェック
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ] && [ "$CURRENT_BRANCH" != "develop" ]; then
    log_warning "現在のブランチ: ${CURRENT_BRANCH}"
    if ! $FORCE; then
        read -p "main/develop以外のブランチですが、続行しますか？ [y/N]: " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "中止しました"
            exit 1
        fi
    fi
fi

# 確認プロンプト
if ! $FORCE && ! $DRY_RUN; then
    echo ""
    echo "以下の変更を実行します:"
    echo "  1. Cargo.toml のバージョンを ${NEW_VERSION} に更新"
    echo "  2. CHANGELOG.md を更新"
    echo "  3. Git コミットを作成"
    echo "  4. Git タグ v${NEW_VERSION} を作成"
    echo ""
    read -p "続行しますか？ [y/N]: " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "中止しました"
        exit 1
    fi
fi

# 1. Cargo.toml更新
log_info "Cargo.toml を更新中..."
if $DRY_RUN; then
    log_info "[DRY RUN] version = \"${CURRENT_VERSION}\" → version = \"${NEW_VERSION}\""
else
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/" Cargo.toml
    else
        # Linux
        sed -i "s/^version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/" Cargo.toml
    fi
    log_success "Cargo.toml 更新完了"
fi

# 2. CHANGELOG.md更新
log_info "CHANGELOG.md を更新中..."
TODAY=$(date +%Y-%m-%d)

if $DRY_RUN; then
    log_info "[DRY RUN] CHANGELOG.md に v${NEW_VERSION} セクションを追加"
else
    # Unreleased セクションを新バージョンセクションに変換
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/## \[Unreleased\]/## [Unreleased]\n\n### Added\n\n### Changed\n\n### Fixed\n\n### Security\n\n## [${NEW_VERSION}] - ${TODAY}/" CHANGELOG.md
    else
        sed -i "s/## \[Unreleased\]/## [Unreleased]\n\n### Added\n\n### Changed\n\n### Fixed\n\n### Security\n\n## [${NEW_VERSION}] - ${TODAY}/" CHANGELOG.md
    fi
    log_success "CHANGELOG.md 更新完了"
fi

# 3. Cargo.lockを更新
log_info "Cargo.lock を更新中..."
if ! $DRY_RUN; then
    cargo update -p backup-suite
    log_success "Cargo.lock 更新完了"
fi

# 4. ビルド確認
log_info "ビルド確認中..."
if ! $DRY_RUN; then
    if cargo build --release; then
        log_success "ビルド成功"
    else
        log_error "ビルド失敗"
        exit 1
    fi
fi

# 5. テスト実行
log_info "テスト実行中..."
if ! $DRY_RUN; then
    if cargo test; then
        log_success "テスト成功"
    else
        log_error "テスト失敗"
        exit 1
    fi
fi

# 6. Gitコミット
log_info "Git コミットを作成中..."
if $DRY_RUN; then
    log_info "[DRY RUN] git add Cargo.toml Cargo.lock CHANGELOG.md"
    log_info "[DRY RUN] git commit -m 'chore: release v${NEW_VERSION}'"
else
    git add Cargo.toml Cargo.lock CHANGELOG.md
    git commit -m "chore: release v${NEW_VERSION}

リリースバージョン: v${NEW_VERSION}
リリース日: ${TODAY}

変更内容:
- Cargo.toml のバージョンを ${NEW_VERSION} に更新
- CHANGELOG.md を更新
- Cargo.lock を更新"
    log_success "Git コミット作成完了"
fi

# 7. Gitタグ作成
log_info "Git タグを作成中..."
if $DRY_RUN; then
    log_info "[DRY RUN] git tag -a v${NEW_VERSION} -m 'Release v${NEW_VERSION}'"
else
    git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}

リリースバージョン: v${NEW_VERSION}
リリース日: ${TODAY}

このリリースの詳細については CHANGELOG.md を参照してください。"
    log_success "Git タグ作成完了"
fi

# 完了メッセージ
echo ""
log_success "リリース準備が完了しました！"
echo ""
echo "次のステップ:"
echo "  1. 変更を確認: git show"
echo "  2. リモートにプッシュ: git push origin ${CURRENT_BRANCH}"
echo "  3. タグをプッシュ: git push origin v${NEW_VERSION}"
echo ""
echo "タグをプッシュすると、GitHub Actions により自動的にリリースが作成されます。"
echo ""

if $DRY_RUN; then
    log_warning "DRY RUN モードで実行されたため、実際の変更は行われていません"
fi
