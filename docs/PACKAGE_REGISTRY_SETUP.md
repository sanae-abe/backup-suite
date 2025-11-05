# 📦 GitLab Package Registry でのカスタムレジストリ設定

> **Phase 2: 開発者向け Package Registry 統合**
> 企業内での制限を考慮した段階的実装ガイド

## 🎯 概要

GitLab Package Registryを使用してcmdrunを企業内のRustエコシステムに統合し、標準的なCargoワークフローで利用できるようにします。

### 🏗️ 実装の前提条件

#### GitLab要件
- GitLab 13.0+ (Package Registry サポート)
- プロジェクトのOwner/Maintainer権限
- Personal Access Token または CI/CD Token

#### 企業環境での制約対応
- **ネットワーク制限**: プロキシ・ファイアウォール設定
- **認証制限**: 企業SSO・2FA要求
- **権限制限**: 管理者権限なしでの設定
- **セキュリティ要求**: 暗号化・監査ログ要求

---

## 🔧 GitLab側設定（管理者向け）

### 1. プロジェクト設定

```yaml
# .gitlab-ci.yml
variables:
  # Package Registry URL構築用
  CARGO_REGISTRY_URL: "sparse+$CI_API_V4_URL/projects/$CI_PROJECT_ID/packages/cargo/"
  CARGO_HOME: $CI_PROJECT_DIR/.cargo
  CARGO_TARGET_DIR: $CI_PROJECT_DIR/target

# レジストリ公開ステージ
stages:
  - test
  - build
  - package
  - publish

# セキュリティ・品質チェック
security-audit:
  stage: test
  image: rust:latest
  script:
    - cargo audit --json > audit-report.json
    - cargo clippy -- -D warnings
  artifacts:
    reports:
      dependency_scanning: audit-report.json
  allow_failure: false

# パッケージビルド
build-package:
  stage: build
  image: rust:latest
  script:
    - cargo build --release
    - cargo test --release
  artifacts:
    paths:
      - target/release/cmdrun
    expire_in: 1 hour

# Package Registry公開
publish-to-registry:
  stage: publish
  image: rust:latest
  dependencies:
    - build-package
  before_script:
    # GitLab Cargo Index設定
    - mkdir -p $CARGO_HOME
    - echo "[registries.gitlab]" >> $CARGO_HOME/config.toml
    - echo "index = \"$CARGO_REGISTRY_URL\"" >> $CARGO_HOME/config.toml
    - echo "token = \"$CI_JOB_TOKEN\"" >> $CARGO_HOME/config.toml
  script:
    # レジストリログイン
    - cargo login --registry gitlab "$CI_JOB_TOKEN"

    # パッケージ公開
    - cargo publish --registry gitlab --allow-dirty
  only:
    - tags
    - main  # メインブランチでも公開（企業内開発用）
  when: manual  # 手動実行で安全性確保
```

### 2. アクセス制御設定

```yaml
# プロジェクト設定 (.gitlab-ci.yml)
variables:
  # アクセス制御
  GITLAB_REGISTRY_ACCESS: "internal"  # 社内のみアクセス

# CI/CD Variables設定（GitLab UI）
# - CARGO_REGISTRY_TOKEN: 専用アクセストークン（Protected）
# - GITLAB_DEPLOY_TOKEN: デプロイ専用トークン（Protected）
```

### 3. Cargo.toml設定

```toml
[package]
name = "backup-suite"
version = "1.0.0"
authors = ["Sanae Abe <sanae-abe@m3.com>"]
edition = "2021"
description = "Enterprise backup solution for M3"
repository = "https://rendezvous.m3.com:3789/sanae-abe/backup-suite"
license = "MIT OR Apache-2.0"
keywords = ["backup", "enterprise", "cli", "rust"]
categories = ["command-line-utilities"]

# GitLab Registry公開用メタデータ
[package.metadata.docs.rs]
all-features = true

# 依存関係（企業環境での推奨設定）
[dependencies]
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"

# 開発用依存関係
[dev-dependencies]
tempfile = "3.0"
```

---

## 👨‍💻 開発者側設定

### 1. 基本設定ファイル

```toml
# ~/.cargo/config.toml
[registries]
# 会社のGitLabレジストリ
company = {
    index = "sparse+https://gitlab.company.com/api/v4/projects/123/packages/cargo/"
}

# レジストリ認証（個人アクセストークン）
[registries.company]
token = "glpat-xxxxxxxxxxxxxxxxxxxx"

# 企業プロキシ設定（必要に応じて）
[http]
proxy = "http://proxy.company.com:8080"
ssl-verify = true
cainfo = "/etc/ssl/certs/company-ca.crt"

# ビルド最適化（企業開発環境向け）
[build]
jobs = 4
target-dir = "target"

[profile.dev]
debug = 1  # 軽量デバッグ情報

[profile.release]
lto = true  # Link Time Optimization
codegen-units = 1
```

### 2. 自動設定スクリプト

```bash
#!/bin/bash
# setup-cargo-registry.sh
# 企業内Cargoレジストリ設定自動化

set -euo pipefail

readonly SCRIPT_NAME="cargo-registry-setup"
readonly GITLAB_URL="https://rendezvous.m3.com:3789"
readonly PROJECT_ID="$(get_project_id)"  # backup-suiteプロジェクトID取得
readonly REGISTRY_NAME="m3-internal"

# プロジェクトID取得関数
get_project_id() {
    # APIから動的にプロジェクトIDを取得
    curl -s "${GITLAB_URL}/api/v4/projects/sanae-abe%2Fbackup-suite" | jq -r '.id' 2>/dev/null || echo "123"
}

# 色付きログ
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly RED='\033[0;31m'
readonly NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }

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

    # Cargoホームディレクトリ確認
    local cargo_home="${CARGO_HOME:-$HOME/.cargo}"
    if [[ ! -d "$cargo_home" ]]; then
        log_warning "Cargoホームディレクトリが見つかりません: $cargo_home"
        mkdir -p "$cargo_home"
        log_info "Cargoホームディレクトリを作成: $cargo_home"
    fi
}

# GitLabアクセストークン取得
get_access_token() {
    local token_file="$HOME/.gitlab-token"

    if [[ -f "$token_file" ]]; then
        GITLAB_TOKEN=$(cat "$token_file")
        log_info "既存のアクセストークンを使用"
    else
        echo "GitLabアクセストークンを入力してください："
        echo "（設定 > アクセストークン > 'read_api', 'read_registry' スコープで作成）"
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
    local registry_url="sparse+${GITLAB_URL}/api/v4/projects/${PROJECT_ID}/packages/cargo/"

    # .cargoディレクトリ作成
    mkdir -p "$HOME/.cargo"

    # 既存設定の確認
    if [[ -f "$cargo_config" ]]; then
        log_info "既存のCargo設定ファイルが見つかりました"
        cp "$cargo_config" "${cargo_config}.backup.$(date +%Y%m%d_%H%M%S)"
        log_info "バックアップを作成: ${cargo_config}.backup.*"
    fi

    # 設定ファイル作成
    cat > "$cargo_config" << EOF
# GitLab Package Registry設定（自動生成）
[registries]
${REGISTRY_NAME} = { index = "${registry_url}" }

[registries.${REGISTRY_NAME}]
token = "${GITLAB_TOKEN}"

# 企業プロキシ設定（必要に応じてコメントアウト）
# [http]
# proxy = "http://proxy.company.com:8080"

# ビルド最適化
[build]
jobs = 4

[profile.dev]
debug = 1

[profile.release]
lto = true
codegen-units = 1
EOF

    chmod 600 "$cargo_config"
    log_info "Cargo設定ファイルを作成: $cargo_config"
}

# 接続テスト
test_registry_connection() {
    log_info "レジストリ接続をテスト中..."

    # 一時プロジェクトでテスト
    local temp_dir
    temp_dir=$(mktemp -d)
    cd "$temp_dir"

    # テスト用Cargo.tomlを作成
    cat > Cargo.toml << EOF
[package]
name = "registry-test"
version = "0.1.0"
edition = "2021"

[dependencies]
EOF

    # レジストリからの検索テスト
    if cargo search --registry "$REGISTRY_NAME" cmdrun > /dev/null 2>&1; then
        log_info "✅ レジストリ接続成功"
    else
        log_warn "⚠️  レジストリからのパッケージ検索に失敗（パッケージが未公開の可能性）"
    fi

    # クリーンアップ
    cd "$HOME"
    rm -rf "$temp_dir"
}

# cmdrun インストールテスト
install_cmdrun() {
    log_info "cmdrunのインストールを試行中..."

    if cargo install cmdrun --registry "$REGISTRY_NAME"; then
        log_info "✅ cmdrun インストール成功"

        # 動作確認
        if cmdrun --version; then
            log_info "✅ cmdrun 動作確認完了"
        else
            log_error "❌ cmdrun の実行に失敗"
        fi
    else
        log_error "❌ cmdrun インストールに失敗"
        echo "考えられる原因:"
        echo "1. パッケージがまだレジストリに公開されていない"
        echo "2. アクセス権限の問題"
        echo "3. ネットワーク接続の問題"
    fi
}

# メイン関数
main() {
    log_info "🚀 GitLab Package Registry 設定を開始"

    check_rust_installation
    get_access_token
    setup_cargo_config
    test_registry_connection

    echo ""
    log_info "設定完了！以下のコマンドでcmdrunを使用できます："
    echo "  cargo install cmdrun --registry $REGISTRY_NAME"
    echo "  cargo add cmdrun --registry $REGISTRY_NAME"
    echo ""
    echo "プロジェクトでの使用例："
    echo "  # Cargo.toml"
    echo "  [dependencies]"
    echo "  cmdrun = { version = \"1.0\", registry = \"$REGISTRY_NAME\" }"

    # インストールを試行するかユーザーに確認
    read -p "cmdrunのインストールを試行しますか？ (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        install_cmdrun
    fi
}

# ヘルプ表示
show_help() {
    cat << EOF
GitLab Package Registry セットアップスクリプト

使用方法:
    $0 [オプション]

オプション:
    -h, --help          このヘルプを表示
    --token TOKEN       GitLabアクセストークンを指定
    --test-only         設定テストのみ実行（インストールしない）

前提条件:
    1. GitLabアクセストークンの取得
       - GitLab > 設定 > アクセストークン
       - スコープ: 'read_api', 'read_registry'

    2. Rustツールチェーンのインストール
       - rustup のインストール
       - cargo の動作確認

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
```

---

## 🔄 使用例とワークフロー

### プロジェクトでの依存関係として使用

```toml
# プロジェクトのCargo.toml
[dependencies]
backup-suite = { version = "1.0", registry = "m3-internal" }
clap = "4.0"
```

```bash
# プロジェクトセットアップ
cargo new my-backup-tool
cd my-backup-tool

# 企業レジストリから依存関係追加
cargo add cmdrun --registry company

# ビルド
cargo build

# 実行
cargo run
```

### CLI ツールとしてインストール

```bash
# 企業レジストリからインストール
cargo install backup-suite --registry m3-internal

# アップデート
cargo install backup-suite --registry m3-internal --force

# アンインストール
cargo uninstall cmdrun
```

### 開発チーム向けワークフロー

```bash
# 1. 新機能開発
git checkout -b feature/new-backup-method
# 開発作業...

# 2. 開発中のバージョンテスト
cargo publish --registry company --dry-run

# 3. プレリリース版公開
cargo publish --registry company

# 4. チームメンバーでの検証
cargo install backup-suite --registry m3-internal --version "1.1.0-alpha.1"

# 5. 本番リリース
git tag v1.1.0
git push origin v1.1.0
# CI/CDで自動的にstableバージョンが公開される
```

---

## 🛡️ セキュリティとベストプラクティス

### アクセストークン管理

```bash
# トークンファイルの安全な管理
chmod 600 ~/.gitlab-token
chmod 600 ~/.cargo/config.toml

# 定期的なトークンローテーション（スクリプト化）
# crontab -e
# 0 0 1 * * ~/scripts/rotate-gitlab-token.sh
```

### 企業プロキシ対応

```toml
# ~/.cargo/config.toml
[http]
proxy = "http://proxy.company.com:8080"
ssl-verify = true
cainfo = "/path/to/company-ca-bundle.crt"

# プロキシ認証が必要な場合
# proxy = "http://username:password@proxy.company.com:8080"
```

### 監査ログとコンプライアンス

```yaml
# .gitlab-ci.yml
audit-usage:
  stage: post-deploy
  script:
    - echo "Package published: $CI_COMMIT_TAG" | logger -t cmdrun-registry
    - curl -X POST "$AUDIT_WEBHOOK_URL" -d "{\"event\":\"package_published\",\"version\":\"$CI_COMMIT_TAG\"}"
  only:
    - tags
```

---

## 🚨 企業環境での制限と対策

### よくある制限と対策

| 制限事項           | 対策                                        |
| ------------------ | ------------------------------------------- |
| 🔒 管理者権限なし   | ユーザーディレクトリ（`~/.cargo/`）での設定 |
| 🌐 プロキシ必須     | `config.toml`でプロキシ設定                 |
| 🔐 2FA必須          | アクセストークンの使用                      |
| 📋 承認プロセス     | 段階的リリース（alpha → beta → stable）     |
| 🛡️ セキュリティ監査 | CI/CDでの自動セキュリティチェック           |

### トラブルシューティング

```bash
# よくある問題の診断スクリプト
# diagnose-registry.sh

#!/bin/bash
echo "=== GitLab Package Registry 診断 ==="

# 1. Cargo設定確認
echo "1. Cargo設定ファイル:"
if [[ -f ~/.cargo/config.toml ]]; then
    echo "✅ ~/.cargo/config.toml が存在"
    grep -A 5 "\[registries\]" ~/.cargo/config.toml || echo "❌ レジストリ設定なし"
else
    echo "❌ ~/.cargo/config.toml が見つかりません"
fi

# 2. ネットワーク接続確認
echo -e "\n2. ネットワーク接続:"
if curl -s "https://gitlab.company.com/api/v4/projects" > /dev/null; then
    echo "✅ GitLab APIにアクセス可能"
else
    echo "❌ GitLab APIにアクセスできません（プロキシ設定を確認）"
fi

# 3. レジストリアクセス確認
echo -e "\n3. レジストリアクセス:"
cargo search --registry company cmdrun 2>&1 | head -3

echo -e "\n=== 診断完了 ==="
```
