#!/bin/bash
# backup-suite Package Registry設定スクリプト
# M3社内GitLab用

set -euo pipefail

readonly SCRIPT_NAME="backup-suite-registry-setup"
readonly GITLAB_URL="https://rendezvous.m3.com:3789"
readonly PROJECT_PATH="sanae-abe/backup-suite"
readonly REGISTRY_NAME="m3-internal"

# 色付きログ
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly RED='\033[0;31m'
readonly NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }

# プロジェクトID取得関数
get_project_id() {
    # APIから動的にプロジェクトIDを取得
    curl -s "${GITLAB_URL}/api/v4/projects/${PROJECT_PATH//\//%2F}" | jq -r '.id' 2>/dev/null || echo "123"
}

# Rust/Cargo前提条件チェック
check_rust_installation() {
    log_info "Rust/Cargoインストール状況を確認中..."

    if ! command -v cargo &> /dev/null; then
        log_error "Cargoがインストールされていません"
        echo ""
        echo "Rustツールチェーンのインストールが必要です："
        echo "1. 以下のコマンドでRustをインストール："
        echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo ""
        echo "2. 環境変数を読み込み："
        echo "   source ~/.cargo/env"
        echo ""
        echo "3. このスクリプトを再実行"
        exit 1
    fi

    local cargo_version
    cargo_version=$(cargo --version)
    log_info "Cargo確認完了: $cargo_version"
}

# GitLabアクセストークン取得
get_access_token() {
    local token_file="$HOME/.gitlab-token"

    if [[ -f "$token_file" ]]; then
        GITLAB_TOKEN=$(cat "$token_file")
        log_info "既存のアクセストークンを使用"
    else
        echo "GitLabアクセストークンを入力してください："
        echo "（${GITLAB_URL}/-/profile/personal_access_tokens で作成）"
        echo "必要スコープ: 'read_api', 'read_registry'"
        read -r -s GITLAB_TOKEN

        # トークンをファイルに保存（権限600）
        echo "$GITLAB_TOKEN" > "$token_file"
        chmod 600 "$token_file"
        log_info "アクセストークンを保存しました: $token_file"
    fi
}

# Cargo設定ファイル作成/更新
setup_cargo_config() {
    local cargo_config="$HOME/.cargo/config.toml"
    local project_id
    project_id=$(get_project_id)
    local registry_url="sparse+${GITLAB_URL}/api/v4/projects/${project_id}/packages/cargo/"

    # .cargoディレクトリ作成
    mkdir -p "$HOME/.cargo"

    # 既存設定のバックアップ
    if [[ -f "$cargo_config" ]]; then
        log_info "既存のCargo設定ファイルをバックアップ"
        cp "$cargo_config" "${cargo_config}.backup.$(date +%Y%m%d_%H%M%S)"
    fi

    # 設定ファイル作成・更新
    log_info "Cargo設定ファイルを更新中..."
    cat >> "$cargo_config" << EOF

# backup-suite M3内部レジストリ設定（自動追加）
[registries.${REGISTRY_NAME}]
index = "${registry_url}"
token = "${GITLAB_TOKEN}"

EOF

    chmod 600 "$cargo_config"
    log_info "Cargo設定ファイルを更新: $cargo_config"
}

# 接続テスト
test_registry_connection() {
    log_info "レジストリ接続をテスト中..."

    # レジストリからの検索テスト
    if cargo search --registry "$REGISTRY_NAME" backup-suite > /dev/null 2>&1; then
        log_info "✅ レジストリ接続成功"
    else
        log_warn "⚠️  パッケージ検索に失敗（パッケージが未公開の可能性）"
    fi
}

# backup-suite インストールテスト
install_backup_suite() {
    log_info "backup-suiteのインストールを試行中..."

    if cargo install backup-suite --registry "$REGISTRY_NAME"; then
        log_info "✅ backup-suite インストール成功"

        # 動作確認
        if backup-suite --version; then
            log_info "✅ backup-suite 動作確認完了"
        else
            log_error "❌ backup-suite の実行に失敗"
        fi
    else
        log_warn "❌ backup-suite インストールに失敗"
        echo "考えられる原因:"
        echo "1. パッケージがまだレジストリに公開されていない"
        echo "2. アクセス権限の問題"
        echo "3. ネットワーク接続の問題"
    fi
}

# メイン関数
main() {
    log_info "🚀 backup-suite Package Registry 設定を開始"

    check_rust_installation
    get_access_token
    setup_cargo_config
    test_registry_connection

    echo ""
    log_info "設定完了！以下のコマンドでbackup-suiteを使用できます："
    echo "  cargo install backup-suite --registry $REGISTRY_NAME"
    echo ""
    echo "プロジェクトでの使用例："
    echo "  # Cargo.toml"
    echo "  [dependencies]"
    echo "  backup-suite = { version = \"1.0\", registry = \"$REGISTRY_NAME\" }"

    # インストールを試行するかユーザーに確認
    read -p "backup-suiteのインストールを試行しますか？ (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        install_backup_suite
    fi
}

# ヘルプ表示
show_help() {
    cat << EOF
backup-suite Package Registry セットアップスクリプト

使用方法:
    $0 [オプション]

オプション:
    -h, --help          このヘルプを表示
    --token TOKEN       GitLabアクセストークンを指定
    --test-only         設定テストのみ実行

前提条件:
    1. GitLabアクセストークンの取得
       - ${GITLAB_URL}/-/profile/personal_access_tokens
       - スコープ: 'read_api', 'read_registry'

    2. Rustツールチェーンのインストール
       - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

例:
    # 対話的セットアップ
    ./setup-cargo-registry.sh

    # トークン指定でセットアップ
    ./setup-cargo-registry.sh --token glpat-xxxxxxxxxxxxxxxxxxxx

EOF
}

# 引数解析
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        --token)
            GITLAB_TOKEN="$2"
            shift 2
            ;;
        --test-only)
            TEST_ONLY=true
            shift
            ;;
        *)
            log_error "不明なオプション: $1"
            show_help
            exit 1
            ;;
    esac
done

# スクリプト実行
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi