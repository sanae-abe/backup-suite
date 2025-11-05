# CI/CD Pipeline Documentation

このドキュメントでは、backup-suiteプロジェクトのCI/CDパイプラインの設定、運用、トラブルシューティングについて説明します。

## 📋 目次

- [ワークフロー概要](#ワークフロー概要)
- [CI Pipeline](#ci-pipeline)
- [Release Pipeline](#release-pipeline)
- [品質ゲート](#品質ゲート)
- [セキュリティチェック](#セキュリティチェック)
- [運用ガイド](#運用ガイド)
- [トラブルシューティング](#トラブルシューティング)
- [ベストプラクティス](#ベストプラクティス)

---

## ワークフロー概要

### 利用可能なワークフロー

| ワークフロー | トリガー | 目的 | 実行時間（目安） |
|-------------|---------|------|----------------|
| `ci.yml` | プッシュ、PR、日次スケジュール | 継続的インテグレーション | 20-30分 |
| `ci-enhanced.yml` | プッシュ、PR | エンタープライズグレードCI | 25-35分 |
| `release.yml` | タグプッシュ、手動 | リリース作成・配布 | 30-45分 |
| `release-enhanced.yml` | タグプッシュ、手動 | 強化版リリースパイプライン | 40-60分 |
| `security.yml` | PR、日次スケジュール | セキュリティスキャン | 10-15分 |
| `coverage.yml` | プッシュ、PR | コードカバレッジ測定 | 15-20分 |
| `benchmark.yml` | プッシュ（main）、手動 | パフォーマンスベンチマーク | 20-30分 |

### ワークフローの選択

#### 通常の開発フロー
```bash
# 機能開発 → ci.yml（自動実行）
git push origin feature/new-feature

# Pull Request → ci.yml + security.yml + coverage.yml（自動実行）
gh pr create --base main --head feature/new-feature

# リリース → release.yml（自動実行）
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

#### エンタープライズ環境
```bash
# より厳格な品質チェック → ci-enhanced.yml
# .github/workflows/ci.yml を ci-enhanced.yml で置き換える

# SLSA対応リリース → release-enhanced.yml
# .github/workflows/release.yml を release-enhanced.yml で置き換える
```

---

## CI Pipeline

### ci-enhanced.yml の構成

#### Phase 1: 高速品質ゲート（5-10分）

```yaml
jobs:
  format:        # コードフォーマット検証
  clippy:        # 静的解析（厳格モード）
  msrv-check:    # 最小サポートRustバージョン検証
```

**目的**: 早期失敗により開発者へ即座にフィードバック

**成功基準**:
- `cargo fmt -- --check`: すべてのファイルが正しくフォーマットされている
- `cargo clippy`: 警告0件（`-D warnings -D clippy::all -D clippy::pedantic`）
- MSRV: Cargo.tomlの`rust-version`でビルド成功

#### Phase 2: セキュリティスキャン（5-10分並列実行）

```yaml
jobs:
  security-audit:  # cargo-audit: CVE脆弱性検出
  cargo-deny:      # 依存関係ポリシー検証
  sbom-generation: # Software Bill of Materials生成
```

**セキュリティポリシー**:
- 脆弱性: `deny` → CI失敗
- メンテナンス終了: `warn` → CI継続、警告表示
- Yankedクレート: `deny` → CI失敗
- 不許可ライセンス: `deny` → CI失敗

#### Phase 3: テスト実行（20-30分）

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta, nightly]
```

**テスト戦略**:
- 単体テスト: `cargo test --lib`
- 統合テスト: `cargo test --test integration_tests`
- プロパティベーステスト: `cargo test --test proptest`
- ドキュメントテスト: `cargo test --doc`

#### Phase 4: カバレッジ測定（15-20分）

```bash
cargo tarpaulin --all-features --workspace --timeout 300
```

**カバレッジ閾値**: 70%（Phase 2完了後は80%に引き上げ予定）

**除外対象**:
- ベンチマークコード: `benches/*`
- テストコード: `tests/*`
- サンプルコード: `examples/*`

#### Phase 5: 統合チェック

```yaml
ci-success:
  needs: [format, clippy, msrv-check, security-audit, cargo-deny, test, coverage, documentation]
```

すべてのチェックが成功した場合のみ、CIが成功とマークされます。

### 必須チェック項目

すべてのPRは以下のチェックをパスする必要があります：

- ✅ Format Check: コードフォーマット
- ✅ Clippy (Strict Mode): 静的解析
- ✅ MSRV Compatibility: 最小サポートRustバージョン
- ✅ Security Audit: セキュリティ脆弱性
- ✅ Dependency Policy: 依存関係ポリシー
- ✅ Test Suite: 全テスト成功
- ✅ Code Coverage: カバレッジ70%以上
- ✅ Documentation: ドキュメント生成成功

---

## Release Pipeline

### release-enhanced.yml の構成

#### Phase 1: リリース検証（5分）

```yaml
validate-release:
  - バージョン形式検証
  - Cargo.tomlとの整合性確認
  - CHANGELOG.md確認
```

**バージョン形式**:
- セマンティックバージョニング: `MAJOR.MINOR.PATCH[-PRERELEASE]`
- 例: `1.0.0`, `1.0.0-alpha.1`, `2.1.3-rc.2`

#### Phase 2: リリースノート生成（5分）

自動生成される内容:
- 変更内容（CHANGELOG.mdから抽出）
- コミット履歴
- コントリビューター一覧
- インストール手順
- チェックサム（ビルド後に更新）

#### Phase 3: クロスプラットフォームビルド（30-45分）

```yaml
strategy:
  matrix:
    - Linux (x86_64, aarch64, musl)
    - macOS (x86_64, aarch64)
    - Windows (x86_64)
```

**ビルド成果物**:
- バイナリアーカイブ（`.tar.gz` / `.zip`）
- SHA256チェックサム（`.sha256`）
- SLSA Provenance（計画中）

#### Phase 4: リリース作成（10分）

GitHub Releasesに以下をアップロード:
- リリースノート
- すべてのプラットフォームのバイナリ
- チェックサムファイル

#### Phase 5: 配布（オプション、15-20分）

- **crates.io**: Rustクレート公開（正式リリースのみ）
- **Docker Hub / GHCR**: コンテナイメージ公開
- **Homebrew**: Formula更新（手動）

#### Phase 6: リリース後処理（5分）

- 次のバージョンへCargo.tomlを更新
- バージョンバンプPRの自動作成

### リリース手順

#### 1. 通常リリース

```bash
# 1. バージョン更新
vim Cargo.toml  # version = "1.0.0"
vim CHANGELOG.md  # ## [1.0.0] - 2025-01-01

# 2. コミット
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 1.0.0"
git push origin develop

# 3. タグ作成・プッシュ（リリースパイプライン自動実行）
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

#### 2. プレリリース

```bash
# バージョン: 1.0.0-alpha.1
git tag -a v1.0.0-alpha.1 -m "Pre-release v1.0.0-alpha.1"
git push origin v1.0.0-alpha.1

# または手動トリガー
gh workflow run release-enhanced.yml \
  -f version=1.0.0-alpha.1 \
  -f prerelease=true
```

#### 3. ホットフィックスリリース

```bash
# mainブランチからホットフィックスブランチ作成
git checkout main
git checkout -b hotfix/critical-bug

# 修正実施・コミット
git add .
git commit -m "fix: critical security issue"

# パッチバージョン更新
vim Cargo.toml  # version = "1.0.1"
git add Cargo.toml
git commit -m "chore: bump version to 1.0.1"

# タグ作成・プッシュ
git tag -a v1.0.1 -m "Hotfix v1.0.1"
git push origin v1.0.1

# mainとdevelopへマージ
git checkout main
git merge hotfix/critical-bug
git push origin main

git checkout develop
git merge hotfix/critical-bug
git push origin develop
```

---

## 品質ゲート

### コードフォーマット

```bash
# ローカル実行
cargo fmt --all

# CI検証コマンド
cargo fmt --all -- --check
```

**設定ファイル**: `rustfmt.toml`（プロジェクトルート）

### 静的解析（Clippy）

```bash
# ローカル実行（標準）
cargo clippy --all-targets --all-features

# CI厳格モード
cargo clippy --all-targets --all-features -- \
  -D warnings \
  -D clippy::all \
  -D clippy::pedantic \
  -D clippy::cargo \
  -A clippy::multiple-crate-versions \
  -A clippy::module-name-repetitions
```

**許可されるLint例外**:
- `clippy::multiple-crate-versions`: 依存関係の複数バージョン（一時的許可）
- `clippy::module-name-repetitions`: モジュール名の繰り返し（可読性優先）

### テストカバレッジ

```bash
# ローカル実行
cargo tarpaulin --all-features --workspace --out Html

# カバレッジレポート確認
open tarpaulin-report.html
```

**目標カバレッジ**:
- Phase 1: 60%以上
- Phase 2: 70%以上（現在）
- Phase 3: 80%以上（目標）

---

## セキュリティチェック

### cargo-audit（CVE脆弱性）

```bash
# インストール
cargo install cargo-audit

# 実行
cargo audit

# JSON形式レポート
cargo audit --json > audit-report.json
```

**重大度別対応**:
- Critical: 即座に対応（24時間以内）
- High: 優先対応（1週間以内）
- Medium: 計画的対応（1ヶ月以内）
- Low: 次回メンテナンス時

### cargo-deny（依存関係ポリシー）

```bash
# インストール
cargo install cargo-deny

# 実行
cargo deny check

# 個別チェック
cargo deny check advisories  # セキュリティ勧告
cargo deny check licenses    # ライセンス
cargo deny check bans        # 禁止依存関係
cargo deny check sources     # ソース検証
```

**設定ファイル**: `deny.toml`

```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
copyleft = "deny"

[bans]
multiple-versions = "warn"
wildcards = "deny"
```

### SBOM（Software Bill of Materials）

```bash
# インストール
cargo install cargo-sbom

# SBOM生成（CycloneDX形式）
cargo sbom --output-format json > sbom.json

# SBOM生成（SPDX形式）
cargo sbom --output-format spdx > sbom.spdx
```

**用途**:
- サプライチェーンセキュリティ
- ライセンスコンプライアンス
- 脆弱性追跡

---

## 運用ガイド

### ブランチ戦略

```
main (保護)
  ├── develop (統合)
  │   ├── feature/* (機能開発)
  │   ├── fix/* (バグ修正)
  │   └── refactor/* (リファクタリング)
  └── hotfix/* (緊急修正)
```

**ブランチ保護設定**:
- `main`: マージ前に全CIチェック必須、レビュー2名必須
- `develop`: マージ前に全CIチェック必須、レビュー1名必須

### CI失敗時の対応フロー

#### 1. Format Check失敗

```bash
# ローカルで修正
cargo fmt --all

# コミット・プッシュ
git add .
git commit -m "style: apply cargo fmt"
git push
```

#### 2. Clippy失敗

```bash
# 警告内容確認
cargo clippy --all-targets --all-features

# 修正後、再確認
cargo clippy --all-targets --all-features -- -D warnings

# コミット・プッシュ
git add .
git commit -m "fix: resolve clippy warnings"
git push
```

#### 3. Test失敗

```bash
# 失敗したテストの特定
cargo test --verbose

# 詳細ログ確認
RUST_BACKTRACE=1 cargo test <test_name> -- --nocapture

# 修正後、全テスト実行
cargo test --all-features

# コミット・プッシュ
git add .
git commit -m "test: fix failing tests"
git push
```

#### 4. Security Audit失敗

```bash
# 脆弱性詳細確認
cargo audit

# 依存関係更新
cargo update <crate_name>

# または、Cargo.toml で固定バージョン更新
vim Cargo.toml

# 再チェック
cargo audit

# コミット・プッシュ
git add Cargo.toml Cargo.lock
git commit -m "chore: update dependencies to fix vulnerabilities"
git push
```

### キャッシュ管理

GitHub Actionsのキャッシュは7日間で期限切れになります。

```yaml
# キャッシュキー構造
key: ${{ runner.os }}-cargo-<type>-${{ hashFiles('**/Cargo.lock') }}
```

**キャッシュクリア手順**:
1. GitHub UI: Actions → Caches → 削除
2. または、Cargo.lockを更新してキャッシュキーを変更

### シークレット管理

必要なシークレット:

| シークレット名 | 用途 | 必須 |
|--------------|------|------|
| `CARGO_REGISTRY_TOKEN` | crates.io公開 | リリース時 |
| `DOCKER_USERNAME` | Docker Hub公開 | オプション |
| `DOCKER_PASSWORD` | Docker Hub公開 | オプション |
| `CODECOV_TOKEN` | Codecovアップロード | オプション |

**設定方法**:
1. GitHub: Settings → Secrets and variables → Actions
2. New repository secret
3. 名前と値を入力

---

## トラブルシューティング

### よくある問題と解決策

#### 1. CI タイムアウト

**症状**: ジョブが制限時間（デフォルト: 60分）を超える

**原因**:
- 依存関係のビルドに時間がかかる
- テストが遅い
- キャッシュが効いていない

**解決策**:

```yaml
# タイムアウト延長
jobs:
  test:
    timeout-minutes: 90  # 60分 → 90分

# キャッシュ確認
- name: Check cache hit
  if: steps.cache.outputs.cache-hit != 'true'
  run: echo "Cache miss"

# 並列実行数削減
strategy:
  matrix:
    rust: [stable]  # beta, nightlyを削除
```

#### 2. フラキーテスト（不安定なテスト）

**症状**: テストが間欠的に失敗する

**原因**:
- タイムアウト依存のテスト
- ファイルシステム競合
- 並列実行の競合

**解決策**:

```rust
// タイムアウト延長
#[tokio::test]
#[timeout(10000)]  // 10秒
async fn test_async_operation() { ... }

// テスト分離
#[test]
fn test_isolated() {
    let temp_dir = tempdir().unwrap();
    // temp_dir内で操作
}

// リトライロジック
#[test]
fn test_with_retry() {
    for _ in 0..3 {
        if test_logic().is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    panic!("Test failed after 3 retries");
}
```

#### 3. カバレッジ閾値未達

**症状**: カバレッジが70%未満でCI失敗

**原因**:
- 新規コードにテストがない
- エラーハンドリングパスが未テスト
- テスト除外設定の問題

**解決策**:

```bash
# カバレッジレポート確認
cargo tarpaulin --out Html
open tarpaulin-report.html

# 未カバー部分の特定
cargo tarpaulin --out Json | jq '.files[] | select(.coverage < 70)'

# テスト追加
vim tests/new_tests.rs

# 一時的に閾値を下げる（緊急時のみ）
# ci-enhanced.yml の COVERAGE_THRESHOLD を調整
```

#### 4. 依存関係の競合

**症状**: `cargo deny check` で依存関係警告

**原因**:
- 複数のクレートが同じ依存関係の異なるバージョンを使用
- 依存関係の推移的競合

**解決策**:

```bash
# 依存関係ツリー確認
cargo tree -d

# 特定クレートのバージョン統一
[dependencies]
serde = "1.0"

[dev-dependencies]
serde = "1.0"  # 同じバージョンを指定

# または、deny.tomlで一時的に許可
[bans]
skip = [
    { name = "serde", version = "1.0.193" },
]
```

#### 5. MSRV ビルド失敗

**症状**: MSRV（最小サポートRustバージョン）でビルド失敗

**原因**:
- 新しいRust機能の使用
- 依存関係がMSRVより新しいRustを要求

**解決策**:

```bash
# MSRV確認
rustup install 1.70.0
rustup default 1.70.0
cargo build --all-features

# Cargo.tomlでMSRV明示
[package]
rust-version = "1.70.0"

# 依存関係のMSRV確認
cargo tree --depth 1 | grep -v "backup-suite"
```

### ログ分析

#### GitHub Actions ログ確認

```bash
# ローカルでGitHub CLIを使用
gh run list --workflow=ci-enhanced.yml
gh run view <run-id> --log

# 失敗したジョブのみ確認
gh run view <run-id> --log-failed
```

#### Artifact ダウンロード

```bash
# CI成果物のダウンロード
gh run download <run-id>

# 特定のartifactのみ
gh run download <run-id> -n coverage-report
```

---

## ベストプラクティス

### 1. コミット前ローカルチェック

```bash
#!/bin/bash
# pre-push.sh - ローカルCI検証スクリプト

set -e

echo "🔍 Running local CI checks..."

echo "1️⃣ Format check..."
cargo fmt --all -- --check

echo "2️⃣ Clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "3️⃣ Tests..."
cargo test --all-features

echo "4️⃣ Security audit..."
cargo audit

echo "✅ All checks passed! Ready to push."
```

### 2. プルリクエスト前チェックリスト

- [ ] コードフォーマット確認（`cargo fmt`）
- [ ] Clippy警告0件（`cargo clippy`）
- [ ] すべてのテスト成功（`cargo test`）
- [ ] 新機能にテスト追加
- [ ] CHANGELOG.md更新（該当する場合）
- [ ] ドキュメント更新（該当する場合）
- [ ] セキュリティ監査OK（`cargo audit`）

### 3. CI高速化テクニック

#### キャッシュ最適化

```yaml
# 細かいキャッシュ分離
- name: Cache cargo registry
  uses: actions/cache@v3
  with:
    path: ~/.cargo/registry
    key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

- name: Cache cargo build
  uses: actions/cache@v3
  with:
    path: target
    key: ${{ runner.os }}-cargo-build-${{ hashFiles('**/Cargo.lock') }}
```

#### 並列実行最大化

```yaml
# ジョブ並列実行
jobs:
  format: ...
  clippy: ...
  security-audit: ...  # format, clippyと並列実行

  test:
    needs: [format, clippy]  # 依存関係を最小限に
```

#### 不要なビルド回避

```yaml
# 特定パス変更時のみ実行
on:
  push:
    paths:
      - 'src/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
  pull_request:
    paths:
      - 'src/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
```

### 4. セキュリティベストプラクティス

- 依存関係は定期的に更新（週次）
- セキュリティアドバイザリを監視（GitHub Dependabot有効化）
- 最小権限の原則でシークレット管理
- SBOM生成でサプライチェーン透明性確保

### 5. リリースベストプラクティス

- セマンティックバージョニング厳守
- CHANGELOGの詳細記載
- リリースノートのレビュー
- バイナリのチェックサム検証
- Pre-releaseで十分なテスト期間

---

## 参考リンク

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI Best Practices](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [Semantic Versioning](https://semver.org/)

---

## サポート

問題が発生した場合:

1. **ドキュメント確認**: このREADME、ワークフローファイルのコメント
2. **ログ分析**: GitHub Actions のログを確認
3. **Issue作成**: 問題を詳細に記載してGitHub Issueを作成
4. **コミュニティ**: Rust Discord、GitHub Discussions

---

*Last updated: 2025-01-05*
