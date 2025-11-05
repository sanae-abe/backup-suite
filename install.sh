#!/bin/bash
# Backup Suite 自動インストールスクリプト
# 対応OS: Linux, macOS
# バージョン: 1.0.0

set -euo pipefail

# 設定
readonly SCRIPT_NAME="backup-suite-installer"
readonly GITLAB_URL="https://rendezvous.m3.com:3789"
readonly PROJECT_PATH="sanae-abe/backup-suite"
readonly BINARY_NAME="backup-suite"
readonly INSTALL_DIR_SYSTEM="/usr/local/bin"
readonly INSTALL_DIR_USER="$HOME/.local/bin"

# 色付きログ出力
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# 前提条件チェック
check_prerequisites() {
    log_info "前提条件をチェック中..."

    # 必要コマンドの確認
    local required_commands=("curl" "tar")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            log_error "必要なコマンドが見つかりません: $cmd"
            log_error "インストールしてから再実行してください。"
            exit 1
        fi
    done

    # JSONパーサーの確認（jq推奨、なければfallback）
    if ! command -v jq &> /dev/null; then
        log_warning "jqがインストールされていません。JSON解析にfallbackメソッドを使用します。"
    fi

    # Rust/Cargoインストール推奨メッセージ
    if ! command -v cargo &> /dev/null; then
        log_warning "Cargoがインストールされていません。"
        log_info "Package Registryからのインストールを希望する場合は、以下を実行してください："
        echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo "    source ~/.cargo/env"
        echo "    cargo install $BINARY_NAME --registry company"
    fi
}

# OS・アーキテクチャ検出
detect_platform() {
    local os_type
    local arch_type

    # OS検出
    case "$(uname -s)" in
        Linux*)     os_type="linux";;
        Darwin*)    os_type="macos";;
        *)
            log_error "サポートされていないOS: $(uname -s)"
            log_error "対応OS: Linux, macOS"
            exit 1
            ;;
    esac

    # アーキテクチャ検出
    case "$(uname -m)" in
        x86_64|amd64)   arch_type="x64";;
        aarch64|arm64)  arch_type="arm64";;
        *)
            log_error "サポートされていないアーキテクチャ: $(uname -m)"
            log_error "対応アーキテクチャ: x86_64, aarch64"
            exit 1
            ;;
    esac

    readonly PLATFORM="${os_type}-${arch_type}"
    log_info "検出プラットフォーム: $PLATFORM"
}

# 最新バージョン取得
get_latest_version() {
    log_info "最新バージョンを取得中..."

    local releases_url="${GITLAB_URL}/api/v4/projects/${PROJECT_PATH//\//%2F}/releases"

    if command -v jq &> /dev/null; then
        # jqを使用
        LATEST_VERSION=$(curl -s "$releases_url" | jq -r '.[0].tag_name')
    else
        # fallback: grepとsedを使用
        LATEST_VERSION=$(curl -s "$releases_url" | grep -o '"tag_name":"[^"]*"' | head -1 | sed 's/"tag_name":"\(.*\)"/\1/')
    fi

    if [[ -z "$LATEST_VERSION" || "$LATEST_VERSION" == "null" ]]; then
        log_error "最新バージョンの取得に失敗しました"
        exit 1
    fi

    log_info "最新バージョン: $LATEST_VERSION"
    readonly LATEST_VERSION
}

# ダウンロードURL構築
build_download_url() {
    readonly ARCHIVE_NAME="${BINARY_NAME}-${PLATFORM}.tar.gz"
    readonly DOWNLOAD_URL="${GITLAB_URL}/${PROJECT_PATH}/-/releases/${LATEST_VERSION}/downloads/${ARCHIVE_NAME}"
    log_info "ダウンロードURL: $DOWNLOAD_URL"
}

# バイナリダウンロード
download_binary() {
    log_info "バイナリをダウンロード中..."

    local temp_dir
    temp_dir=$(mktemp -d)
    readonly TEMP_DIR="$temp_dir"
    readonly ARCHIVE_PATH="${TEMP_DIR}/${ARCHIVE_NAME}"

    # プログレスバー付きダウンロード
    if ! curl -L --progress-bar "$DOWNLOAD_URL" -o "$ARCHIVE_PATH"; then
        log_error "ダウンロードに失敗しました: $DOWNLOAD_URL"
        cleanup
        exit 1
    fi

    log_success "ダウンロード完了"
}

# アーカイブ解凍
extract_binary() {
    log_info "アーカイブを解凍中..."

    if ! tar -xzf "$ARCHIVE_PATH" -C "$TEMP_DIR"; then
        log_error "アーカイブの解凍に失敗しました"
        cleanup
        exit 1
    fi

    readonly BINARY_PATH="${TEMP_DIR}/${BINARY_NAME}"

    if [[ ! -f "$BINARY_PATH" ]]; then
        log_error "解凍されたバイナリが見つかりません: $BINARY_PATH"
        cleanup
        exit 1
    fi

    # 実行権限付与
    chmod +x "$BINARY_PATH"
    log_success "解凍完了"
}

# インストール先決定
determine_install_location() {
    local install_dir

    # システム全体インストールを試行
    if [[ $EUID -eq 0 ]] || [[ -w "$INSTALL_DIR_SYSTEM" ]]; then
        install_dir="$INSTALL_DIR_SYSTEM"
        log_info "システム全体にインストールします: $install_dir"
    else
        # ユーザーディレクトリにインストール
        install_dir="$INSTALL_DIR_USER"
        mkdir -p "$install_dir"
        log_info "ユーザーディレクトリにインストールします: $install_dir"

        # PATHチェック
        if [[ ":$PATH:" != *":$install_dir:"* ]]; then
            log_warning "インストール先がPATHに含まれていません: $install_dir"
            log_warning "以下のコマンドでPATHに追加してください:"
            echo "    echo 'export PATH=\"$install_dir:\$PATH\"' >> ~/.bashrc"
            echo "    source ~/.bashrc"
        fi
    fi

    readonly INSTALL_DIR="$install_dir"
}

# バイナリインストール
install_binary() {
    log_info "バイナリをインストール中..."

    local target_path="${INSTALL_DIR}/${BINARY_NAME}"

    # 既存バイナリのバックアップ
    if [[ -f "$target_path" ]]; then
        local backup_path="${target_path}.backup.$(date +%Y%m%d_%H%M%S)"
        log_warning "既存のバイナリをバックアップ: $backup_path"
        cp "$target_path" "$backup_path"
    fi

    # インストール実行
    if ! cp "$BINARY_PATH" "$target_path"; then
        log_error "インストールに失敗しました"
        log_error "権限不足の可能性があります。sudoで再実行してください："
        log_error "    curl -sSL https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/raw/main/install.sh | sudo bash"
        cleanup
        exit 1
    fi

    # 実行権限確保
    chmod +x "$target_path"

    log_success "インストール完了: $target_path"
}

# インストール確認
verify_installation() {
    log_info "インストールを確認中..."

    if command -v "$BINARY_NAME" &> /dev/null; then
        local installed_version
        installed_version=$("$BINARY_NAME" --version 2>/dev/null | head -1)
        log_success "インストール確認完了"
        log_success "インストール済みバージョン: $installed_version"
        log_info "使用方法:"
        echo "    $BINARY_NAME --help           # ヘルプ表示"
        echo "    $BINARY_NAME init --interactive # 初期設定"
        echo "    $BINARY_NAME config show     # 現在の設定表示"
    else
        log_error "インストールされたバイナリが見つかりません"
        log_error "PATHに問題がある可能性があります"
        exit 1
    fi
}

# クリーンアップ
cleanup() {
    if [[ -n "${TEMP_DIR:-}" && -d "$TEMP_DIR" ]]; then
        rm -rf "$TEMP_DIR"
        log_info "一時ファイルを削除しました"
    fi
}

# エラー時クリーンアップ
trap cleanup EXIT ERR

# ヘルプ表示
show_help() {
    cat << EOF
Backup Suite インストールスクリプト

使用方法:
    $0 [オプション]

オプション:
    -h, --help          このヘルプを表示
    -v, --verbose       詳細ログを表示
    --system            システム全体にインストール（要sudo）
    --user              ユーザーディレクトリにインストール

例:
    # 自動インストール
    curl -sSL https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/raw/main/install.sh | bash

    # システム全体にインストール
    curl -sSL https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/raw/main/install.sh | sudo bash

    # ユーザーディレクトリに強制インストール
    curl -sSL https://rendezvous.m3.com:3789/sanae-abe/backup-suite/-/raw/main/install.sh | bash -s -- --user

対応プラットフォーム:
    - Linux (x86_64, aarch64)
    - macOS (x86_64, Apple Silicon)

EOF
}

# メイン関数
main() {
    local force_system=false
    local force_user=false
    local verbose=false

    # 引数解析
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--verbose)
                verbose=true
                set -x
                shift
                ;;
            --system)
                force_system=true
                shift
                ;;
            --user)
                force_user=true
                shift
                ;;
            *)
                log_error "不明なオプション: $1"
                show_help
                exit 1
                ;;
        esac
    done

    log_info "🚀 Backup Suite インストールを開始します"

    check_prerequisites
    detect_platform
    get_latest_version
    build_download_url
    download_binary
    extract_binary

    # インストール先決定（オプション考慮）
    if [[ "$force_system" == true ]]; then
        readonly INSTALL_DIR="$INSTALL_DIR_SYSTEM"
        log_info "システム全体インストールを強制: $INSTALL_DIR"
    elif [[ "$force_user" == true ]]; then
        readonly INSTALL_DIR="$INSTALL_DIR_USER"
        mkdir -p "$INSTALL_DIR"
        log_info "ユーザーインストールを強制: $INSTALL_DIR"
    else
        determine_install_location
    fi

    install_binary
    verify_installation

    log_success "🎉 インストールが正常に完了しました！"

    # 次のステップ案内
    echo ""
    echo "次のステップ:"
    echo "1. 初期設定: $BINARY_NAME init --interactive"
    echo "2. 設定確認: $BINARY_NAME config show"
    echo "3. ヘルプ:   $BINARY_NAME --help"
    echo ""
    echo "詳細ドキュメント: ${GITLAB_URL}/${PROJECT_PATH}/-/blob/main/INSTALL.md"
}

# スクリプト実行
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi